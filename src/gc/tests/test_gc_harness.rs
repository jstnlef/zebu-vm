extern crate gc;
extern crate utils;
extern crate simple_logger;
extern crate log;

use self::log::LogLevel;
use self::gc::heap;
use self::gc::objectmodel;
use self::utils::Address;
use std::sync::atomic::Ordering;

pub fn start_logging() {

    match simple_logger::init_with_level(LogLevel::Trace) {
        Ok(_) => {},
        Err(_) => {}
    }
}

const OBJECT_SIZE : usize = 24;
const OBJECT_ALIGN: usize = 8;

const WORK_LOAD : usize = 10000;

const SPACIOUS_SPACE_SIZE : usize = 500 << 20;  // 500mb
const LIMITED_SPACE_SIZE  : usize = 20  << 20;  // 20mb
const SMALL_SPACE_SIZE    : usize = 1   << 19;  // 512kb

const IMMIX_SPACE_SIZE : usize = SPACIOUS_SPACE_SIZE;
const LO_SPACE_SIZE    : usize = SPACIOUS_SPACE_SIZE;

#[test]
fn test_exhaust_alloc() {
    gc::gc_init(IMMIX_SPACE_SIZE, LO_SPACE_SIZE, 8);
    let mut mutator = gc::new_mutator();

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

const LARGE_OBJECT_SIZE : usize = 256;

#[test]
#[allow(unused_variables)]
fn test_exhaust_alloc_large() {
    gc::gc_init(IMMIX_SPACE_SIZE, LO_SPACE_SIZE, 8);
    let mut mutator = gc::new_mutator();

    start_logging();

    for _ in 0..WORK_LOAD {
        mutator.yieldpoint();

        let res = gc::muentry_alloc_large(&mut mutator, LARGE_OBJECT_SIZE, OBJECT_ALIGN);
        gc::muentry_init_large_object(&mut mutator, res, 0b1100_0000);
    }

    mutator.destroy();
}

#[test]
#[allow(unused_variables)]
fn test_alloc_large_trigger_gc() {
    gc::gc_init(SMALL_SPACE_SIZE, 4096 * 10, 8);
    let mut mutator = gc::new_mutator();

    start_logging();

    for _ in 0..WORK_LOAD {
        mutator.yieldpoint();

        let res = gc::muentry_alloc_large(&mut mutator, LARGE_OBJECT_SIZE, OBJECT_ALIGN);
        gc::muentry_init_large_object(&mut mutator, res, 0b1100_0000);
    }

    mutator.destroy();
}

#[test]
#[allow(unused_variables)]
fn test_alloc_large_trigger_gc2() {
    gc::gc_init(SMALL_SPACE_SIZE, 4096 * 10, 8);
    let mut mutator = gc::new_mutator();

    start_logging();

    // this will exhaust the lo space
    for _ in 0..10 {
        mutator.yieldpoint();

        let res = gc::muentry_alloc_large(&mut mutator, LARGE_OBJECT_SIZE, OBJECT_ALIGN);
        gc::muentry_init_large_object(&mut mutator, res, 0b1100_0000);
    }

    // this will trigger a gc, and allocate it in the collected space
    let res = gc::muentry_alloc_large(&mut mutator, LARGE_OBJECT_SIZE, OBJECT_ALIGN);
    gc::muentry_init_large_object(&mut mutator, res, 0b1100_0000);

    // this will trigger gcs for immix space
    for _ in 0..100000 {
        mutator.yieldpoint();

        let res = mutator.alloc(OBJECT_SIZE, OBJECT_ALIGN);
        mutator.init_object(res, 0b1100_0011);
    }

    mutator.destroy();
}

#[test]
fn test_alloc_mark() {
    gc::gc_init(IMMIX_SPACE_SIZE, LO_SPACE_SIZE, 8);
    let mut mutator = gc::new_mutator();

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

    let (shared_space, _) = gc::get_spaces();

    println!("Start marking");
    let mark_state = objectmodel::load_mark_state();

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

    mutator.destroy();
}

#[allow(dead_code)]
struct Node<'a> {
    hdr  : u64,
    next : &'a Node<'a>,
    unused_ptr : usize,
    unused_int : i32,
    unused_int2: i32
}

#[test]
fn test_alloc_trace() {
    gc::gc_init(IMMIX_SPACE_SIZE, LO_SPACE_SIZE, 8);
    let mut mutator = gc::new_mutator();
    let (shared_space, lo_space) = gc::get_spaces();

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

    mutator.destroy();
}