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

extern crate utils;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate simple_logger;
extern crate aligned_alloc;
extern crate crossbeam;
extern crate rustc_serialize;
#[macro_use]
extern crate field_offset;

use std::sync::atomic::Ordering;

pub mod common;
pub mod objectmodel;
pub mod heap;

use common::gctype::GCType;
use utils::ObjectReference;
use heap::immix::BYTES_IN_LINE;
use heap::immix::ImmixSpace;
use heap::immix::ImmixMutatorLocal;
use heap::freelist;
use heap::freelist::FreeListSpace;
use common::objectdump;

use utils::LinkedHashSet;
use utils::Address;

use std::fmt;
use std::sync::Arc;
use std::sync::RwLock;

pub const GC_MOVES_OBJECT : bool = false;

pub const LARGE_OBJECT_THRESHOLD : usize = BYTES_IN_LINE;

pub use heap::immix::ImmixMutatorLocal as Mutator;
pub use heap::immix::CURSOR_OFFSET as ALLOCATOR_CURSOR_OFFSET;
pub use heap::immix::LIMIT_OFFSET as ALLOCATOR_LIMIT_OFFSET;

#[repr(C)]
pub struct GC {
    immix_space: Arc<ImmixSpace>,
    lo_space   : Arc<FreeListSpace>,

    gc_types   : Vec<Arc<GCType>>,
    roots      : LinkedHashSet<ObjectReference>
}

impl fmt::Debug for GC {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GC\n").unwrap();
        write!(f, "{}", self.immix_space).unwrap();

        write!(f, "{}", self.lo_space)
    }
}

lazy_static! {
    pub static ref MY_GC : RwLock<Option<GC>> = RwLock::new(None);
}

#[no_mangle]
pub extern fn gc_stats() {
    println!("{:?}", MY_GC.read().unwrap().as_ref().unwrap());
}

#[no_mangle]
pub extern fn get_spaces() -> (Arc<ImmixSpace>, Arc<FreeListSpace>) {
    let space_lock = MY_GC.read().unwrap();
    let space = space_lock.as_ref().unwrap();
    
    (space.immix_space.clone(), space.lo_space.clone())
}

#[no_mangle]
pub extern fn add_gc_type(mut ty: GCType) -> Arc<GCType> {
    let mut gc_guard = MY_GC.write().unwrap();
    let mut gc = gc_guard.as_mut().unwrap();

    let index = gc.gc_types.len() as u32;
    ty.id = index;

    let ty = Arc::new(ty);

    gc.gc_types.push(ty.clone());

    ty
}

#[no_mangle]
pub extern fn get_gc_type_encode(id: u32) -> u64 {
    let gc_lock = MY_GC.read().unwrap();
    let ref gctype  = gc_lock.as_ref().unwrap().gc_types[id as usize];

    if gctype.is_hybrid() {
        objectmodel::gen_hybrid_gctype_encode(gctype, 0) // fake length
    } else {
        objectmodel::gen_gctype_encode(gctype)
    }
}

#[no_mangle]
pub extern fn gc_init(immix_size: usize, lo_size: usize, n_gcthreads: usize, enable_gc: bool) {
    // set this line to turn on certain level of debugging info
//    simple_logger::init_with_level(log::LogLevel::Trace).ok();

    // init object model - init this first, since spaces may use it
    objectmodel::init();
    
    // init space size
    heap::IMMIX_SPACE_SIZE.store(immix_size, Ordering::SeqCst);
    heap::LO_SPACE_SIZE.store(lo_size, Ordering::SeqCst);
    
    let (immix_space, lo_space) = {
        let immix_space = Arc::new(ImmixSpace::new(immix_size));
        let lo_space    = Arc::new(FreeListSpace::new(lo_size));

        heap::gc::init(n_gcthreads);
        
        (immix_space, lo_space)
    };
    
    *MY_GC.write().unwrap() = Some(GC {
        immix_space: immix_space,
        lo_space: lo_space,

        gc_types: vec![],
        roots   : LinkedHashSet::new()
    });

    if enable_gc {
        heap::gc::ENABLE_GC.store(true, Ordering::Relaxed);
    } else {
        heap::gc::ENABLE_GC.store(false, Ordering::Relaxed);
    }

    info!("heap is {} bytes (immix: {} bytes, lo: {} bytes) . ", immix_size + lo_size, immix_size, lo_size);
    info!("{} gc threads", n_gcthreads);
    if !enable_gc {
        warn!("GC disabled (panic when a collection is triggered)");
    }
}

#[no_mangle]
pub extern fn new_mutator() -> ImmixMutatorLocal {
    ImmixMutatorLocal::new(MY_GC.read().unwrap().as_ref().unwrap().immix_space.clone())
}

#[no_mangle]
#[allow(unused_variables)]
pub extern fn drop_mutator(mutator: *mut ImmixMutatorLocal) {
    unsafe {mutator.as_mut().unwrap()}.destroy();
    
    // rust will reclaim the boxed mutator
}

pub use heap::gc::set_low_water_mark;

// explicitly control roots

#[no_mangle]
#[inline(always)]
pub extern fn add_to_root(obj: ObjectReference) {
    let mut gc = MY_GC.write().unwrap();
    gc.as_mut().unwrap().roots.insert(obj);
}

#[no_mangle]
#[inline(always)]
pub extern fn remove_root(obj: ObjectReference) {
    let mut gc = MY_GC.write().unwrap();
    gc.as_mut().unwrap().roots.remove(&obj);
}

// pin/unpin

#[no_mangle]
pub extern fn muentry_pin_object(obj: ObjectReference) -> Address {
    add_to_root(obj);
    obj.to_address()
}

#[no_mangle]
pub extern fn muentry_unpin_object(obj: Address) {
    remove_root(unsafe {obj.to_object_reference()});
}

// yieldpoint

#[no_mangle]
#[inline(always)]
pub extern fn yieldpoint(mutator: *mut ImmixMutatorLocal) {
    unsafe {mutator.as_mut().unwrap()}.yieldpoint();
}

#[no_mangle]
#[inline(never)]
pub extern fn yieldpoint_slow(mutator: *mut ImmixMutatorLocal) {
    unsafe {mutator.as_mut().unwrap()}.yieldpoint_slow()
}

// allocation

#[inline(always)]
pub fn alloc(mutator: *mut ImmixMutatorLocal, size: usize, align: usize) -> ObjectReference {
    let addr = unsafe {&mut *mutator}.alloc(size, align);
    unsafe {addr.to_object_reference()}
}

#[no_mangle]
#[inline(never)]
/// size doesn't include HEADER_SIZE, return value is offset by HEADER_OFFSET
pub extern fn muentry_alloc_fast(mutator: *mut ImmixMutatorLocal, size: usize, align: usize) -> ObjectReference {
    let ret = alloc(mutator, size, align);
    trace!("muentry_alloc_fast(mutator: {:?}, size: {}, align: {}) = {}", mutator, size, align, ret);

    ret
}

#[no_mangle]
#[inline(never)]
pub extern fn muentry_init_object(mutator: *mut ImmixMutatorLocal, obj: ObjectReference, encode: u64) {
    unsafe {&mut *mutator}.init_object(obj.to_address(), encode);
}

#[no_mangle]
#[inline(never)]
pub extern fn muentry_init_hybrid(mutator: *mut ImmixMutatorLocal, obj: ObjectReference, encode: u64, length: u64) {
    unsafe {&mut *mutator}.init_hybrid(obj.to_address(), encode, length);
}

#[no_mangle]
#[inline(never)]
/// this function is supposed to be called by an inlined fastpath
/// size _includes_ HEADER_SIZE, return value is _NOT_ offset by HEADER_OFFSET
pub extern fn muentry_alloc_slow(mutator: *mut ImmixMutatorLocal, size: usize, align: usize) -> ObjectReference {
    let ret = unsafe {&mut *mutator}.try_alloc_from_local(size, align);
    trace!("muentry_alloc_slow(mutator: {:?}, size: {}, align: {}) = {}", mutator, size, align, ret);

    unsafe {ret.to_object_reference()}
}

#[no_mangle]
/// size doesn't include HEADER_SIZE, return value is offset by HEADER_OFFSET
pub extern fn muentry_alloc_large(mutator: *mut ImmixMutatorLocal, size: usize, align: usize) -> ObjectReference {
    let ret = freelist::alloc_large(size, align, unsafe {mutator.as_mut().unwrap()}, MY_GC.read().unwrap().as_ref().unwrap().lo_space.clone());
    trace!("muentry_alloc_large(mutator: {:?}, size: {}, align: {}) = {}", mutator, size, align, ret);

    unsafe {ret.to_object_reference()}
}

// force gc
#[no_mangle]
pub extern fn force_gc() {
    heap::gc::trigger_gc();
}

// dump heap
#[no_mangle]
pub extern fn persist_heap(roots: Vec<Address>) -> objectdump::HeapDump {
    objectdump::HeapDump::from_roots(roots)
}