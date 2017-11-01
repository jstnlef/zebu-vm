// Copyright 2017 The Australian National University
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # An Immix garbage collector implementation
//!
//! This crate implements a garbage collector for Zebu. We carefully designed
//! the interface so the garbage collector is a standalone crate from the VM,
//! and it should be able to reuse easily outside Zebu project.
//!
//! The GC implements immix for small object allocation/reclamation, and
//! treadmill for large objects. It uses an object model with 64-bits object header
//! before the start of the object. Allocation always returns an ObjectReference
//! pointing to the start of the object.
//!
//! The idea of the GC implementation is discussed in the paper: Rust as a language
//! for high performance GC implementation (ISMM'16).
//!
//! A user who tries to use this GC (Zebu or other user) should do the following:
//!
//! 1. initialize the GC by calling gc_init()
//! 2. for a running mutator thread, call new_mutator() to create a mutator
//!    (and store it somewhere in TLS). And call set_low_water_mark() to inform
//!    the GC so that when it conservatively scans stack, it will not scan beyond
//!    the low water mark
//! 3. insert yieldpoint() occasionally in the code so that the GC can synchronise
//!    with the thread, or insert yieldpoint_slow() if user decides to implement an inlined
//!    fastpath
//! 4. call alloc_fast() to ask for allocation, or alloc_slow()
//!    if user decides to implement an inlined fastpath
//! 5. the allocation may trigger a GC, and it is guaranteed to return a valid address
//! 6. call init_object() or init_hybrid() to initialize the object
//! 7. when the thread quits, call drop_mutator() to properly destroy a mutator.
//!
//! Other utility functions provided by the GC:
//!
//! * explicit control of root set - add_to_root()/remove_root():
//!   the GC treats stacks and registers as default root set, however the client may
//!   explicitly add references as root
//! * explicit control of object movement/liveness - pin_object()/unpin_object():
//!   the GC will keep the object alive, and in place (does not move it)
//! * capability of persisting the heap as a relocatable boot image - persist_heap():
//!   the GC will traverse the heap from given roots, and dump all reachable objects
//!   in a structured way so that the user can use the data structure to access every
//!   object and persist them in their own approach
//!
//! Issues (going to be fixed in a major GC rewrite):
//!
//! * currently collection is disabled: due to bugs (and the fact that we are going to
//!   majorly change the GC)
//! * we are using a 64-bits header for each object, we will switch to sidemap object
//!   model (Issue #12)
//! * we are allocating the whole heap, and initialize it all at once during startup. We
//!   should allow dynamic growth of heap (Issue #56)
//! * pin/unpin operations is different from Mu spec (Issue #33)
//! * we are using some utility C functions (heap/gc/clib_(architecture).c/.S) to help acquire
//!   some information for GC. And those C functions are not returning accurate results
//!   (Issue #21)

#[macro_use]
extern crate rodal;
extern crate mu_utils as utils;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate stderrlog;
extern crate aligned_alloc;
extern crate crossbeam;
#[macro_use]
extern crate field_offset;

use common::gctype::GCType;
use common::objectdump;
use common::ptr::*;
use heap::*;
use heap::immix::BYTES_IN_LINE;
use heap::immix::ImmixSpace;
use heap::immix::ImmixAllocator;
use utils::*;
use objectmodel::sidemap::*;

use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::Ordering;

/// data structures for the GC and the user
pub mod common;

/// object model (metadata for objects managed by the GC)
/// this allows the user to know some GC semantics, and to be able to implement
/// fastpath on their side
//  FIXME: this mod can be private (we expose it only because tests are using it)
//  we should consider moving those tests within the mod
pub mod objectmodel;
/// object header size (in byte)
pub use objectmodel::OBJECT_HEADER_SIZE;
/// offset from an object reference to the header (in byte, can be negative)
pub use objectmodel::OBJECT_HEADER_OFFSET;

/// the main GC crate, heap structures (including collection, immix space, freelist space)
//  FIXME: this mod can be private (we expose it only because tests are using it)
//  we should consider moving those tests within the mod
pub mod heap;

/// whether this GC will move objects?
/// (does an object have a fixed address once allocated before it is reclaimed)
pub const GC_MOVES_OBJECT: bool = false;

/// threshold for small objects. Use small object allocator (immix) for objects that
/// are smaller than this threshold. Otherwise use large object allocator (freelist)
pub const LARGE_OBJECT_THRESHOLD: usize = BYTES_IN_LINE;

/// the mutator that the user is supposed to put to every mutator thread
/// Most interface functions provided by the GC require a pointer to this mutator.
pub use heap::Mutator;

//  these two offsets help user's compiler to generate inlined fastpath code

/// offset to the immix allocator cursor from its pointer
//pub use heap::immix::CURSOR_OFFSET as ALLOCATOR_CURSOR_OFFSET;
/// offset to the immix allocator limit from its pointer
//pub use heap::immix::LIMIT_OFFSET as ALLOCATOR_LIMIT_OFFSET;
/// GC represents the context for the current running GC instance
struct GC {
    immix_tiny: Raw<ImmixSpace>,
    immix_normal: Raw<ImmixSpace>,
    //    lo: Arc<FreeListSpace>,
    gc_types: Vec<Arc<GCType>>,
    roots: LinkedHashSet<ObjectReference>
}

lazy_static! {
    static ref MY_GC : RwLock<Option<GC>> = RwLock::new(None);
}

impl GC {
    pub fn is_heap_object(&self, addr: Address) -> bool {
        self.immix_tiny.addr_in_space(addr) || self.immix_normal.addr_in_space(addr)
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct GCConfig {
    pub immix_tiny_size: ByteSize,
    pub immix_normal_size: ByteSize,
    pub lo_size: ByteSize,
    pub n_gcthreads: usize,
    pub enable_gc: bool
}

//  the implementation of this GC will be changed dramatically in the future,
//  but the exposed interface is likely to stay the same.
/// initializes the GC
#[no_mangle]
pub extern "C" fn gc_init(config: GCConfig) {
    trace!("Initializing GC...");
    // init object model - init this first, since spaces may use it
    objectmodel::init();

    // init spaces
    trace!("  initializing tiny immix space...");
    let immix_tiny = ImmixSpace::new(SpaceDescriptor::ImmixTiny, config.immix_tiny_size);
    trace!("  initializing normal immix space...");
    let immix_normal = ImmixSpace::new(SpaceDescriptor::ImmixNormal, config.immix_normal_size);
    //    trace!("  initializing large object space...");
    //    let lo_space = Arc::new(FreeListSpace::new(lo_size));

    // init GC
    heap::gc::init(config.n_gcthreads);
    *MY_GC.write().unwrap() = Some(GC {
        immix_tiny,
        immix_normal,
        gc_types: vec![],
        roots: LinkedHashSet::new()
    });
    heap::gc::ENABLE_GC.store(config.enable_gc, Ordering::Relaxed);

    info!(
        "heap is {} bytes (immix_tiny: {} bytes, immix_normal: {} bytes) . ",
        config.immix_tiny_size + config.immix_normal_size,
        config.immix_tiny_size,
        config.immix_normal_size
    );
    info!("{} gc threads", config.n_gcthreads);
    if !config.enable_gc {
        warn!("GC disabled (panic when a collection is triggered)");
    }
}

/// destroys current GC instance
#[no_mangle]
pub extern "C" fn gc_destroy() {
    objectmodel::cleanup();
    let mut gc_lock = MY_GC.write().unwrap();
    {
        let gc = gc_lock.as_ref().unwrap();
        gc.immix_tiny.cleanup();
        gc.immix_normal.cleanup();
    }
    *gc_lock = None;
}

/// creates a mutator
#[no_mangle]
pub extern "C" fn new_mutator() -> *mut Mutator {
    let gc_lock = MY_GC.read().unwrap();
    let gc: &GC = gc_lock.as_ref().unwrap();

    let global = Arc::new(MutatorGlobal::new());
    let m: *mut Mutator = Box::into_raw(Box::new(Mutator::new(
        ImmixAllocator::new(gc.immix_tiny.clone()),
        ImmixAllocator::new(gc.immix_normal.clone()),
        global
    )));

    // allocators have a back pointer to the mutator
    unsafe { (&mut *m) }.tiny.set_mutator(m);
    unsafe { (&mut *m) }.normal.set_mutator(m);

    m
}

/// destroys a mutator
/// Note the user has to explicitly drop mutator that they are not using, otherwise
/// the GC may not be able to stop all the mutators before GC, and ends up in an endless
/// pending status
#[no_mangle]
#[allow(unused_variables)]
pub extern "C" fn drop_mutator(mutator: *mut Mutator) {
    unsafe { mutator.as_mut().unwrap() }.destroy();

    // rust will reclaim the boxed mutator
}

/// sets low water mark for current thread
/// When the GC conservatively scans stack for root, it will not scan beyond the low
/// water mark
pub use heap::gc::set_low_water_mark;

/// adds an object reference to the root set
#[no_mangle]
#[inline(always)]
pub extern "C" fn add_to_root(obj: ObjectReference) {
    let mut gc = MY_GC.write().unwrap();
    gc.as_mut().unwrap().roots.insert(obj);
}

/// removes an object reference from the root set
#[no_mangle]
#[inline(always)]
pub extern "C" fn remove_root(obj: ObjectReference) {
    let mut gc = MY_GC.write().unwrap();
    gc.as_mut().unwrap().roots.remove(&obj);
}

/// pins an object so that it will be moved or reclaimed
#[no_mangle]
pub extern "C" fn muentry_pin_object(obj: ObjectReference) -> Address {
    add_to_root(obj);
    obj.to_address()
}

/// unpins an object so that it can be freely moved/reclaimed as normal objects
#[no_mangle]
pub extern "C" fn muentry_unpin_object(obj: Address) {
    remove_root(unsafe { obj.to_object_reference() });
}

/// a regular check to see if the mutator should stop for synchronisation
#[no_mangle]
#[inline(always)]
pub extern "C" fn yieldpoint(mutator: *mut Mutator) {
    unsafe { mutator.as_mut().unwrap() }.yieldpoint();
}

/// the slowpath for yieldpoint
/// We assume for performance, the user will implement an inlined fastpath, we provide
/// constants, offsets to fields and this slowpath function for the user
#[no_mangle]
#[inline(never)]
pub extern "C" fn yieldpoint_slow(mutator: *mut Mutator) {
    unsafe { mutator.as_mut().unwrap() }.yieldpoint_slow()
}

#[inline(always)]
fn mutator_ref(m: *mut Mutator) -> &'static mut Mutator {
    unsafe { &mut *m }
}

/// allocates an object in the immix space
#[inline(always)]
#[no_mangle]
pub extern "C" fn muentry_alloc_tiny(
    mutator: *mut Mutator,
    size: usize,
    align: usize
) -> ObjectReference {
    let m = mutator_ref(mutator);
    unsafe { m.tiny.alloc(size, align).to_object_reference() }
}

#[inline(always)]
#[no_mangle]
pub extern "C" fn muentry_alloc_normal(
    mutator: *mut Mutator,
    size: usize,
    align: usize
) -> ObjectReference {
    let m = mutator_ref(mutator);
    let res = m.normal.alloc(size, align);
    m.normal.post_alloc(res, size, align);
    unsafe { res.to_object_reference() }
}

/// allocates an object with slowpath in the immix space
#[no_mangle]
#[inline(never)]
pub extern "C" fn muentry_alloc_tiny_slow(
    mutator: *mut Mutator,
    size: usize,
    align: usize
) -> Address {
    let m = mutator_ref(mutator);
    m.tiny.alloc_slow(size, align)
}

/// allocates an object with slowpath in the immix space
#[no_mangle]
#[inline(never)]
pub extern "C" fn muentry_alloc_normal_slow(
    mutator: *mut Mutator,
    size: usize,
    align: usize
) -> Address {
    let m = mutator_ref(mutator);
    let res = m.normal.alloc_slow(size, align);
    m.normal.post_alloc(res, size, align);
    res
}

/// allocates an object in the freelist space (large object space)
#[no_mangle]
#[inline(never)]
#[allow(unused_variables)]
pub extern "C" fn muentry_alloc_large(
    mutator: *mut Mutator,
    size: usize,
    align: usize
) -> ObjectReference {
    //    let ret = freelist::alloc_large(
    //        size,
    //        align,
    //        unsafe { mutator.as_mut().unwrap() },
    //        MY_GC.read().unwrap().as_ref().unwrap().lo.clone()
    //    );
    //    trace!(
    //        "muentry_alloc_large(mutator: {:?}, size: {}, align: {}) = {}",
    //        mutator,
    //        size,
    //        align,
    //        ret
    //    );
    //
    //    unsafe { ret.to_object_reference() }
    unimplemented!()
}

#[no_mangle]
//  size doesn't include HEADER_SIZE
pub extern "C" fn muentry_alloc_any(
    mutator: *mut Mutator,
    size: usize,
    align: usize
) -> ObjectReference {
    let size = size + OBJECT_HEADER_SIZE;
    if size < MAX_TINY_OBJECT {
        muentry_alloc_tiny(mutator, size, align)
    } else if size < MAX_MEDIUM_OBJECT {
        muentry_alloc_normal(mutator, size, align)
    } else {
        muentry_alloc_large(mutator, size, align)
    }
}

/// initializes a fix-sized object
#[no_mangle]
#[inline(always)]
pub extern "C" fn muentry_init_tiny_object(
    mutator: *mut Mutator,
    obj: ObjectReference,
    encode: TinyObjectEncode
) {
    unsafe { &mut *mutator }
        .tiny
        .init_object(obj.to_address(), encode);
}

/// initializes a fix-sized object
#[no_mangle]
#[inline(always)]
pub extern "C" fn muentry_init_small_object(
    mutator: *mut Mutator,
    obj: ObjectReference,
    encode: SmallObjectEncode
) {
    unsafe { &mut *mutator }
        .normal
        .init_object(obj.to_address(), encode);
}

/// initializes a fix-sized object
#[no_mangle]
#[inline(always)]
pub extern "C" fn muentry_init_medium_object(
    mutator: *mut Mutator,
    obj: ObjectReference,
    encode: MediumObjectEncode
) {
    unsafe { &mut *mutator }
        .normal
        .init_object(obj.to_address(), encode);
}

/// initializes a hybrid type object
#[no_mangle]
#[inline(never)]
pub extern "C" fn muentry_init_hybrid(
    mutator: *mut ImmixAllocator,
    obj: ObjectReference,
    encode: u64,
    length: u64
) {
    unsafe { &mut *mutator }.init_hybrid(obj.to_address(), encode, length);
}

/// forces gc to happen
/// (this is not a 'hint' - world will be stopped, and heap traversal will start)
#[no_mangle]
pub extern "C" fn force_gc(mutator: *mut Mutator) {
    heap::gc::trigger_gc();
    yieldpoint(mutator);
}

/// traces reachable objects and record them as a data structure
/// so that the user can inspect the reachable heap and persist it in their way
#[no_mangle]
pub extern "C" fn persist_heap(roots: Vec<Address>) -> objectdump::HeapDump {
    objectdump::HeapDump::from_roots(roots)
}

// the following API functions may get removed in the future

/// gets immix space and freelist space
#[no_mangle]
pub extern "C" fn get_spaces() -> (Raw<ImmixSpace>, Raw<ImmixSpace>) {
    let space_lock = MY_GC.read().unwrap();
    let space = space_lock.as_ref().unwrap();

    (
        space.immix_tiny.clone(),
        space.immix_normal.clone() //        space.lo.clone()
    )
}

/// informs GC of a GCType
#[no_mangle]
pub extern "C" fn add_gc_type(mut ty: GCType) -> Arc<GCType> {
    let mut gc_guard = MY_GC.write().unwrap();
    let mut gc = gc_guard.as_mut().unwrap();

    let index = gc.gc_types.len() as u32;
    ty.id = index;

    let ty = Arc::new(ty);

    gc.gc_types.push(ty.clone());

    ty
}

/// gets the encoding for a given GC type (by ID)
#[no_mangle]
pub extern "C" fn get_gc_type_encode(id: u32) -> u64 {
    let gc_lock = MY_GC.read().unwrap();
    let ref gctype = gc_lock.as_ref().unwrap().gc_types[id as usize];

    if gctype.is_hybrid() {
        objectmodel::gen_hybrid_gctype_encode(gctype, 0) // fake length
    } else {
        objectmodel::gen_gctype_encode(gctype)
    }
}

pub fn start_logging_trace() {
    match stderrlog::new().verbosity(4).init() {
        Ok(()) => { info!("logger initialized") }
        Err(e) => {
            error!(
                "failed to init logger, probably already initialized: {:?}",
                e
            )
        }
    }
}
