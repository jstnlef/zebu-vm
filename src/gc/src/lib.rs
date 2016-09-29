extern crate utils;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate simple_logger;
extern crate aligned_alloc;
extern crate crossbeam;

use std::sync::atomic::Ordering;

pub mod common;
pub mod objectmodel;
pub mod heap;

use utils::ObjectReference;
use heap::immix::BYTES_IN_LINE;
use heap::immix::ImmixSpace;
use heap::immix::ImmixMutatorLocal;
use heap::freelist;
use heap::freelist::FreeListSpace;

use std::fmt;
use std::sync::Arc;
use std::sync::RwLock;

pub const LARGE_OBJECT_THRESHOLD : usize = BYTES_IN_LINE;

pub use heap::immix::ImmixMutatorLocal as Mutator;
pub use heap::immix::CURSOR_OFFSET as ALLOCATOR_CURSOR_OFFSET;
pub use heap::immix::LIMIT_OFFSET as ALLOCATOR_LIMIT_OFFSET;

#[repr(C)]
pub struct GC {
    immix_space: Arc<ImmixSpace>,
    lo_space   : Arc<RwLock<FreeListSpace>>
}

impl fmt::Debug for GC {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GC\n").unwrap();
        write!(f, "{}", self.immix_space).unwrap();
        
        let lo_lock = self.lo_space.read().unwrap();
        write!(f, "{}", *lo_lock)
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
pub extern fn get_spaces() -> (Arc<ImmixSpace>, Arc<RwLock<FreeListSpace>>) {
    let space_lock = MY_GC.read().unwrap();
    let space = space_lock.as_ref().unwrap();
    
    (space.immix_space.clone(), space.lo_space.clone())
}

#[no_mangle]
pub extern fn gc_init(immix_size: usize, lo_size: usize, n_gcthreads: usize) {
    // set this line to turn on certain level of debugging info
//    simple_logger::init_with_level(log::LogLevel::Trace).ok();
    
    // init space size
    heap::IMMIX_SPACE_SIZE.store(immix_size, Ordering::SeqCst);
    heap::LO_SPACE_SIZE.store(lo_size, Ordering::SeqCst);
    
    let (immix_space, lo_space) = {
        let immix_space = Arc::new(ImmixSpace::new(immix_size));
        let lo_space    = Arc::new(RwLock::new(FreeListSpace::new(lo_size)));

        heap::gc::init(immix_space.clone(), lo_space.clone());        
        
        (immix_space, lo_space)
    };
    
    *MY_GC.write().unwrap() = Some(GC {immix_space: immix_space, lo_space: lo_space});
    println!("heap is {} bytes (immix: {} bytes, lo: {} bytes) . ", immix_size + lo_size, immix_size, lo_size);
    
    // gc threads
    heap::gc::GC_THREADS.store(n_gcthreads, Ordering::SeqCst);
    println!("{} gc threads", n_gcthreads);
    
    // init object model
    objectmodel::init();
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

#[cfg(target_arch = "x86_64")]
#[link(name = "gc_clib_x64")]
extern "C" {
    pub fn set_low_water_mark();
}

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

#[no_mangle]
#[inline(always)]
pub extern fn alloc(mutator: *mut ImmixMutatorLocal, size: usize, align: usize) -> ObjectReference {
    let addr = unsafe {mutator.as_mut().unwrap()}.alloc(size, align);
    unsafe {addr.to_object_reference()}
}

#[no_mangle]
#[inline(never)]
pub extern fn muentry_alloc_slow(mutator: *mut ImmixMutatorLocal, size: usize, align: usize) -> ObjectReference {
    let ret = unsafe {mutator.as_mut().unwrap()}.try_alloc_from_local(size, align);
    unsafe {ret.to_object_reference()}
}

#[no_mangle]
pub extern fn alloc_large(mutator: *mut ImmixMutatorLocal, size: usize, align: usize) -> ObjectReference {
    let ret = freelist::alloc_large(size, align, unsafe {mutator.as_mut().unwrap()}, MY_GC.read().unwrap().as_ref().unwrap().lo_space.clone());
    unsafe {ret.to_object_reference()}
}