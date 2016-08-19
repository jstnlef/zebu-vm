use mu::runtime::mem;
use mu::runtime::mem::heap;
use mu::runtime::mem::heap::immix::ImmixMutatorLocal;
use mu::runtime::mem::heap::immix::ImmixSpace;
use mu::runtime::mem::heap::freelist::FreeListSpace;
use mu::runtime::mem::common::Address;
use mu::runtime::mem::common::ObjectReference;
use mu::runtime::mem::objectmodel;

use std::sync::RwLock;
use std::sync::Arc;
use std::sync::atomic::Ordering;

const OBJECT_SIZE : usize = 24;
const OBJECT_ALIGN: usize = 8;

const WORK_LOAD : usize = 500000;

const IMMIX_SPACE_SIZE : usize = 500 << 20;
const LO_SPACE_SIZE    : usize = 500 << 20; 

#[test]
fn test_exhaust_alloc() {
    mem::gc_init(IMMIX_SPACE_SIZE, LO_SPACE_SIZE, 8);
    
    let shared_space : Arc<ImmixSpace> = {
        let space : ImmixSpace = ImmixSpace::new(heap::IMMIX_SPACE_SIZE.load(Ordering::SeqCst));
        
        Arc::new(space)
    };
    let lo_space : Arc<RwLock<FreeListSpace>> = {
        let space : FreeListSpace = FreeListSpace::new(heap::LO_SPACE_SIZE.load(Ordering::SeqCst));
        Arc::new(RwLock::new(space))
    };
    heap::gc::init(shared_space.clone(), lo_space.clone());

    let mut mutator = mem::new_mutator();
    
    println!("Trying to allocate {} objects of (size {}, align {}). ", WORK_LOAD, OBJECT_SIZE, OBJECT_ALIGN);
    const ACTUAL_OBJECT_SIZE : usize = OBJECT_SIZE;
    println!("Considering header size of {}, an object should be {}. ", 0, ACTUAL_OBJECT_SIZE);
    println!("This would take {} bytes of {} bytes heap", WORK_LOAD * ACTUAL_OBJECT_SIZE, heap::IMMIX_SPACE_SIZE.load(Ordering::SeqCst));
    
    for _ in 0..WORK_LOAD {
        mutator.yieldpoint();
        
        let res = mutator.alloc(OBJECT_SIZE, OBJECT_ALIGN);
        mutator.init_object(res, 0b1100_0011);  
    }
}

#[test]
fn test_alloc_mark() {
    heap::IMMIX_SPACE_SIZE.store(IMMIX_SPACE_SIZE, Ordering::SeqCst);
    heap::LO_SPACE_SIZE.store(LO_SPACE_SIZE, Ordering::SeqCst);
    
    let shared_space : Arc<ImmixSpace> = {
        let space : ImmixSpace = ImmixSpace::new(heap::IMMIX_SPACE_SIZE.load(Ordering::SeqCst));
        
        Arc::new(space)
    };
    let lo_space : Arc<RwLock<FreeListSpace>> = {
        let space : FreeListSpace = FreeListSpace::new(heap::LO_SPACE_SIZE.load(Ordering::SeqCst));
        Arc::new(RwLock::new(space))
    };
    heap::gc::init(shared_space.clone(), lo_space.clone());

    let mut mutator = ImmixMutatorLocal::new(shared_space.clone());
    
    println!("Trying to allocate 1 object of (size {}, align {}). ", OBJECT_SIZE, OBJECT_ALIGN);
    const ACTUAL_OBJECT_SIZE : usize = OBJECT_SIZE;
    println!("Considering header size of {}, an object should be {}. ", 0, ACTUAL_OBJECT_SIZE);
    
    println!("Trying to allocate {} objects, which will take roughly {} bytes", WORK_LOAD, WORK_LOAD * ACTUAL_OBJECT_SIZE);
    let mut objs = vec![];
    for _ in 0..WORK_LOAD {
        let res = mutator.alloc(ACTUAL_OBJECT_SIZE, OBJECT_ALIGN);
        mutator.init_object(res, 0b1100_0011);
        
        objs.push(unsafe {res.to_object_reference()});
    }
    
    println!("Start marking");
    let mark_state = objectmodel::MARK_STATE.load(Ordering::SeqCst) as u8;
    
    let line_mark_table = shared_space.line_mark_table();
    let (space_start, space_end) = (shared_space.start(), shared_space.end());
    
    let trace_map = shared_space.trace_map.ptr;
    
    for i in 0..objs.len() {
        let obj = unsafe {*objs.get_unchecked(i)};
            
        // mark the object as traced
        objectmodel::mark_as_traced(trace_map, space_start, obj, mark_state);
        
        // mark meta-data
        if obj.to_address() >= space_start && obj.to_address() < space_end {
            line_mark_table.mark_line_live2(space_start, obj.to_address());
        } 
    } 
}

struct Node<'a> {
    hdr  : u64,
    next : &'a Node<'a>,
    unused_ptr : usize,
    unused_int : i32,
    unused_int2: i32
}

#[test]
fn test_alloc_trace() {
    heap::IMMIX_SPACE_SIZE.store(IMMIX_SPACE_SIZE, Ordering::SeqCst);
    heap::LO_SPACE_SIZE.store(LO_SPACE_SIZE, Ordering::SeqCst);
    
    let shared_space : Arc<ImmixSpace> = {
        let space : ImmixSpace = ImmixSpace::new(heap::IMMIX_SPACE_SIZE.load(Ordering::SeqCst));
        
        Arc::new(space)
    };
    let lo_space : Arc<RwLock<FreeListSpace>> = {
        let space : FreeListSpace = FreeListSpace::new(heap::LO_SPACE_SIZE.load(Ordering::SeqCst));
        Arc::new(RwLock::new(space))
    };
    heap::gc::init(shared_space.clone(), lo_space.clone());

    let mut mutator = ImmixMutatorLocal::new(shared_space.clone());
    
    println!("Trying to allocate 1 object of (size {}, align {}). ", OBJECT_SIZE, OBJECT_ALIGN);
    const ACTUAL_OBJECT_SIZE : usize = OBJECT_SIZE;
    println!("Considering header size of {}, an object should be {}. ", 0, ACTUAL_OBJECT_SIZE);
    
    println!("Trying to allocate {} objects, which will take roughly {} bytes", WORK_LOAD, WORK_LOAD * ACTUAL_OBJECT_SIZE);
    let root = mutator.alloc(ACTUAL_OBJECT_SIZE, OBJECT_ALIGN);
    mutator.init_object(root, 0b1100_0001);
    
    let mut prev = root;
    for _ in 0..WORK_LOAD - 1 {
        let res = mutator.alloc(ACTUAL_OBJECT_SIZE, OBJECT_ALIGN);
        mutator.init_object(res, 0b1100_0001);
        
        // set prev's 1st field (offset 0) to this object
        unsafe {prev.store::<Address>(res)};
        
        prev = res;
    }

    println!("Start tracing");
    let mut roots = vec![unsafe {root.to_object_reference()}];

    heap::gc::start_trace(&mut roots, shared_space, lo_space);
}