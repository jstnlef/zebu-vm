use mu::runtime::mm;
use mu::runtime::mm::heap;
use std::sync::atomic::Ordering;

const OBJECT_SIZE : usize = 24;
const OBJECT_ALIGN: usize = 8;

const WORK_LOAD : usize = 10000000;

const IMMIX_SPACE_SIZE : usize = 40 << 20;
const LO_SPACE_SIZE    : usize = 40 << 20; 

#[test]
fn test_gc_no_alive() {
    unsafe {heap::gc::set_low_water_mark();}
    
    mm::gc_init(IMMIX_SPACE_SIZE, LO_SPACE_SIZE, 8);
    let mut mutator = mm::new_mutator();
    
    println!("Trying to allocate {} objects of (size {}, align {}). ", WORK_LOAD, OBJECT_SIZE, OBJECT_ALIGN);
    const ACTUAL_OBJECT_SIZE : usize = OBJECT_SIZE;
    println!("Considering header size of {}, an object should be {}. ", 0, ACTUAL_OBJECT_SIZE);
    println!("This would take {} bytes of {} bytes heap", WORK_LOAD * ACTUAL_OBJECT_SIZE, heap::IMMIX_SPACE_SIZE.load(Ordering::SeqCst));
    
    for _ in 0..WORK_LOAD {
        mutator.yieldpoint();
        
        let res = mutator.alloc(OBJECT_SIZE, OBJECT_ALIGN);
        mutator.init_object(res, 0b1100_0011);  
    }
    
    mutator.destroy();
}