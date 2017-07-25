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

#[allow(dead_code)]
const SPACIOUS_SPACE_SIZE : usize = 500 << 20;  // 500mb
#[allow(dead_code)]
const LIMITED_SPACE_SIZE  : usize = 20  << 20;  // 20mb
#[allow(dead_code)]
const SMALL_SPACE_SIZE    : usize = 1   << 19;  // 512kb

#[allow(dead_code)]
const IMMIX_SPACE_SIZE : usize = SPACIOUS_SPACE_SIZE;
#[allow(dead_code)]
const LO_SPACE_SIZE    : usize = SPACIOUS_SPACE_SIZE;

#[cfg(feature = "use-sidemap")]
const FIXSIZE_NOREF_ENCODE : u64 = 0b1100_0000u64;
#[cfg(not(feature = "use-sidemap"))]
const FIXSIZE_NOREF_ENCODE : u64 = 0xb000000000000000u64;

#[cfg(feature = "use-sidemap")]
const FIXSIZE_REFx2_ENCODE : u64 = 0b1100_0011u64;
#[cfg(not(feature = "use-sidemap"))]
const FIXSIZE_REFx2_ENCODE : u64 = 0xb000000000000003u64;

#[cfg(feature = "use-sidemap")]
const FIXSIZE_REFx1_ENCODE : u64 = 0b1100_0001u64;
#[cfg(not(feature = "use-sidemap"))]
const FIXSIZE_REFx1_ENCODE : u64 = 0xb000000000000001u64;


#[test]
pub fn test_exhaust_alloc() {
    gc::gc_init(IMMIX_SPACE_SIZE, LO_SPACE_SIZE, 8, false);
    let mut mutator = gc::new_mutator();

    println!("Trying to allocate {} objects of (size {}, align {}). ", WORK_LOAD, OBJECT_SIZE, OBJECT_ALIGN);
    const ACTUAL_OBJECT_SIZE : usize = OBJECT_SIZE;
    println!("Considering header size of {}, an object should be {}. ", 0, ACTUAL_OBJECT_SIZE);
    println!("This would take {} bytes of {} bytes heap", WORK_LOAD * ACTUAL_OBJECT_SIZE, heap::IMMIX_SPACE_SIZE.load(Ordering::SeqCst));

    for _ in 0..WORK_LOAD {
        mutator.yieldpoint();

        let res = mutator.alloc(OBJECT_SIZE, OBJECT_ALIGN);
        mutator.init_object(res, FIXSIZE_NOREF_ENCODE);
    }

    mutator.destroy();
}

const LARGE_OBJECT_SIZE : usize = 256;

#[test]
#[allow(unused_variables)]
fn test_exhaust_alloc_large() {
    gc::gc_init(IMMIX_SPACE_SIZE, LO_SPACE_SIZE, 8, false);
    let mut mutator = gc::new_mutator();

    start_logging();

    for _ in 0..WORK_LOAD {
        mutator.yieldpoint();

        let res = gc::muentry_alloc_large(&mut mutator, LARGE_OBJECT_SIZE, OBJECT_ALIGN);
        gc::muentry_init_object(&mut mutator, res, FIXSIZE_NOREF_ENCODE);
    }

    mutator.destroy();
}

#[test]
#[allow(unused_variables)]
fn test_alloc_large_lo_trigger_gc() {
    const KEEP_N_ROOTS : usize = 1;
    let mut roots : usize = 0;

    gc::gc_init(SMALL_SPACE_SIZE, 4096 * 10, 8, true);
    let mut mutator = gc::new_mutator();

    start_logging();

    for _ in 0..WORK_LOAD {
        mutator.yieldpoint();

        let res = gc::muentry_alloc_large(&mut mutator, LARGE_OBJECT_SIZE, OBJECT_ALIGN);
        gc::muentry_init_object(&mut mutator, res, FIXSIZE_NOREF_ENCODE);

        if roots < KEEP_N_ROOTS {
            gc::add_to_root(res);
            roots += 1;
        }
    }

    mutator.destroy();
}

#[test]
#[allow(unused_variables)]
fn test_alloc_large_both_trigger_gc() {
    gc::gc_init(SMALL_SPACE_SIZE, 4096 * 10, 8, true);
    let mut mutator = gc::new_mutator();

    start_logging();

    // this will exhaust the lo space
    for _ in 0..10 {
        mutator.yieldpoint();

        let res = gc::muentry_alloc_large(&mut mutator, LARGE_OBJECT_SIZE, OBJECT_ALIGN);
        gc::muentry_init_object(&mut mutator, res, FIXSIZE_NOREF_ENCODE);
    }

    // this will trigger a gc, and allocate it in the collected space
    let res = gc::muentry_alloc_large(&mut mutator, LARGE_OBJECT_SIZE, OBJECT_ALIGN);
    gc::muentry_init_object(&mut mutator, res, FIXSIZE_NOREF_ENCODE);

    // this will trigger gcs for immix space
    for _ in 0..100000 {
        mutator.yieldpoint();

        let res = mutator.alloc(OBJECT_SIZE, OBJECT_ALIGN);
        mutator.init_object(res, FIXSIZE_REFx2_ENCODE);
    }

    mutator.destroy();
}

#[test]
#[cfg(feature = "use-sidemap")]
fn test_alloc_mark() {
    gc::gc_init(IMMIX_SPACE_SIZE, LO_SPACE_SIZE, 8, false);
    let mut mutator = gc::new_mutator();

    println!("Trying to allocate 1 object of (size {}, align {}). ", OBJECT_SIZE, OBJECT_ALIGN);
    const ACTUAL_OBJECT_SIZE : usize = OBJECT_SIZE;
    println!("Considering header size of {}, an object should be {}. ", 0, ACTUAL_OBJECT_SIZE);

    println!("Trying to allocate {} objects, which will take roughly {} bytes", WORK_LOAD, WORK_LOAD * ACTUAL_OBJECT_SIZE);
    let mut objs = vec![];
    for _ in 0..WORK_LOAD {
        let res = mutator.alloc(ACTUAL_OBJECT_SIZE, OBJECT_ALIGN);
        mutator.init_object(res, FIXSIZE_REFx2_ENCODE);

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

#[test]
#[cfg(not(feature = "use-sidemap"))]
fn test_alloc_mark() {
    gc::gc_init(IMMIX_SPACE_SIZE, LO_SPACE_SIZE, 8, false);
    let mut mutator = gc::new_mutator();

    println!("Trying to allocate 1 object of (size {}, align {}). ", OBJECT_SIZE, OBJECT_ALIGN);
    const ACTUAL_OBJECT_SIZE : usize = OBJECT_SIZE;
    println!("Considering header size of {}, an object should be {}. ", 0, ACTUAL_OBJECT_SIZE);

    println!("Trying to allocate {} objects, which will take roughly {} bytes", WORK_LOAD, WORK_LOAD * ACTUAL_OBJECT_SIZE);
    let mut objs = vec![];
    for _ in 0..WORK_LOAD {
        let res = mutator.alloc(ACTUAL_OBJECT_SIZE, OBJECT_ALIGN);
        mutator.init_object(res, FIXSIZE_REFx2_ENCODE);

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
        objectmodel::mark_as_traced(obj, mark_state);

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
    gc::gc_init(IMMIX_SPACE_SIZE, LO_SPACE_SIZE, 8, false);
    let mut mutator = gc::new_mutator();
    let (shared_space, lo_space) = gc::get_spaces();

    println!("Trying to allocate 1 object of (size {}, align {}). ", OBJECT_SIZE, OBJECT_ALIGN);
    const ACTUAL_OBJECT_SIZE : usize = OBJECT_SIZE;
    println!("Considering header size of {}, an object should be {}. ", 0, ACTUAL_OBJECT_SIZE);

    println!("Trying to allocate {} objects, which will take roughly {} bytes", WORK_LOAD, WORK_LOAD * ACTUAL_OBJECT_SIZE);
    let root = mutator.alloc(ACTUAL_OBJECT_SIZE, OBJECT_ALIGN);
    mutator.init_object(root, FIXSIZE_REFx1_ENCODE);

    let mut prev = root;
    for _ in 0..WORK_LOAD - 1 {
        let res = mutator.alloc(ACTUAL_OBJECT_SIZE, OBJECT_ALIGN);
        mutator.init_object(res, FIXSIZE_REFx1_ENCODE);

        // set prev's 1st field (offset 0) to this object
        unsafe {prev.store::<Address>(res)};

        prev = res;
    }

    println!("Start tracing");
    let mut roots = vec![unsafe {root.to_object_reference()}];

    heap::gc::start_trace(&mut roots, shared_space, lo_space);

    mutator.destroy();
}
