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

extern crate mu_gc;
extern crate mu_utils;
extern crate log;

use self::mu_gc::*;
use self::mu_gc::heap;
use self::mu_gc::heap::*;
use self::mu_gc::heap::immix::*;
use self::mu_gc::heap::gc::*;
use self::mu_gc::objectmodel::sidemap::*;
use self::mu_utils::*;
use std::sync::atomic::Ordering;

#[allow(dead_code)]
const SPACIOUS_SPACE_SIZE: usize = 500 << 20; // 500mb
#[allow(dead_code)]
const LIMITED_SPACE_SIZE: usize = 20 << 20; // 20mb
#[allow(dead_code)]
const SMALL_SPACE_SIZE: usize = 1 << 19; // 512kb

#[allow(dead_code)]
const IMMIX_SPACE_SIZE: usize = SPACIOUS_SPACE_SIZE;
#[allow(dead_code)]
const LO_SPACE_SIZE: usize = SPACIOUS_SPACE_SIZE;

#[test]
pub fn test_tiny_immix_alloc() {
    const OBJECT_SIZE: usize = 16;
    const OBJECT_ALIGN: usize = 8;
    const WORK_LOAD: usize = BYTES_IN_BLOCK / OBJECT_SIZE;
    // we should see the slow paths get invoked exactly twice

    start_logging_trace();
    gc_init(GCConfig {
        immix_tiny_size: IMMIX_SPACE_SIZE,
        immix_normal_size: 0,
        lo_size: 0,
        n_gcthreads: 8,
        enable_gc: false
    });
    let (tiny_space, _) = get_spaces();
    let mutator = new_mutator();
    for _ in 0..WORK_LOAD {
        yieldpoint(mutator);
        let res = muentry_alloc_tiny(mutator, OBJECT_SIZE, OBJECT_ALIGN);
    }
    assert_eq!(tiny_space.n_used_blocks(), 0);

    let res = muentry_alloc_tiny(mutator, OBJECT_SIZE, OBJECT_ALIGN);
    assert_eq!(tiny_space.n_used_blocks(), 1);

    drop_mutator(mutator);
    gc_destroy();
}

#[test]
pub fn test_tiny_immix_gc() {
    const OBJECT_SIZE: usize = 16;
    const OBJECT_ALIGN: usize = 8;
    const WORK_LOAD: usize = BYTES_IN_BLOCK / OBJECT_SIZE;
    // we should see the slow paths get invoked exactly twice

    start_logging_trace();
    gc_init(GCConfig {
        immix_tiny_size: IMMIX_SPACE_SIZE,
        immix_normal_size: 0,
        lo_size: 0,
        n_gcthreads: 8,
        enable_gc: true
    });
    let (tiny_space, _) = get_spaces();
    let mutator = new_mutator();
    let tiny_header = TinyObjectEncode::new(0b0u8);

    // doing one allocation
    let res = muentry_alloc_tiny(mutator, OBJECT_SIZE, OBJECT_ALIGN);
    muentry_init_tiny_object(mutator, res, tiny_header);

    // add the object to the root and force a gc
    add_to_root(res);
    force_gc(mutator);

    // one line should be alive - and another line is conservatively alive
    assert_eq!(tiny_space.last_gc_used_lines, 2);

    // another allocation
    let res2 = muentry_alloc_tiny(mutator, OBJECT_SIZE, OBJECT_ALIGN);
    muentry_init_tiny_object(mutator, res2, tiny_header);

    // remove the object, and force a gc
    remove_root(res);
    force_gc(mutator);

    // no line should be alive
    assert_eq!(tiny_space.last_gc_used_lines, 0);

    drop_mutator(mutator);
    gc_destroy();
}

#[test]
pub fn test_tiny_immix_exhaust() {
    const IMMIX_SPACE_SIZE: usize = SMALL_SPACE_SIZE;
    const OBJECT_SIZE: usize = 16;
    const OBJECT_ALIGN: usize = 8;
    // to trigger GC exactly 2 times
    const WORK_LOAD: usize = (IMMIX_SPACE_SIZE / OBJECT_SIZE) * 2 + 1;

    start_logging_trace();
    gc_init(GCConfig {
        immix_tiny_size: IMMIX_SPACE_SIZE,
        immix_normal_size: 0,
        lo_size: 0,
        n_gcthreads: 8,
        enable_gc: true
    });

    let (tiny_space, _) = get_spaces();
    let mutator = new_mutator();
    let tiny_header = TinyObjectEncode::new(0b0u8);

    for _ in 0..WORK_LOAD {
        yieldpoint(mutator);
        let res = muentry_alloc_tiny(mutator, OBJECT_SIZE, OBJECT_ALIGN);
        muentry_init_tiny_object(mutator, res, tiny_header);
    }
    assert_eq!(tiny_space.n_used_blocks(), 0);
    assert_eq!(GC_COUNT.load(Ordering::SeqCst), 2);

    drop_mutator(mutator);
    gc_destroy();
}

#[test]
pub fn test_tiny_immix_linkedlist() {
    const IMMIX_SPACE_SIZE: usize = SMALL_SPACE_SIZE;
    const OBJECT_SIZE: usize = 16;
    const OBJECT_ALIGN: usize = 8;
    const WORK_LOAD: usize = 4;

    start_logging_trace();
    gc_init(GCConfig {
        immix_tiny_size: IMMIX_SPACE_SIZE,
        immix_normal_size: 0,
        lo_size: 0,
        n_gcthreads: 1,
        enable_gc: true
    });

    let (tiny_space, _) = get_spaces();
    let mutator = new_mutator();
    // first field is a reference, size 16
    let header = TinyObjectEncode::new(0b00000001u8);

    let mut last_obj: Address = unsafe { Address::zero() };
    for _ in 0..WORK_LOAD {
        yieldpoint(mutator);
        let res = muentry_alloc_tiny(mutator, OBJECT_SIZE, OBJECT_ALIGN);
        muentry_init_tiny_object(mutator, res, header);
        // the first field of this object points to the last object
        unsafe { res.to_address().store(last_obj); }
        last_obj = res.to_address();
    }

    // keep the linked list alive
    let last_obj = unsafe { last_obj.to_object_reference() };
    add_to_root(last_obj);
    force_gc(mutator);
    assert_eq!(GC_COUNT.load(Ordering::SeqCst), 1);
    assert_eq!(tiny_space.last_gc_used_lines, 2);

    // another gc
    force_gc(mutator);
    assert_eq!(GC_COUNT.load(Ordering::SeqCst), 2);
    assert_eq!(tiny_space.last_gc_used_lines, 2);

    // set the linked list free, and do gc
    remove_root(last_obj);
    force_gc(mutator);
    assert_eq!(GC_COUNT.load(Ordering::SeqCst), 3);
    assert_eq!(tiny_space.last_gc_used_lines, 0);

    drop_mutator(mutator);
    gc_destroy();
}

//
//const LARGE_OBJECT_SIZE: usize = 256;
//
//#[test]
//#[allow(unused_variables)]
//fn test_exhaust_alloc_large() {
//    gc::gc_init(IMMIX_SPACE_SIZE, LO_SPACE_SIZE, 8, false);
//    let mut mutator = gc::new_mutator();
//
//    start_logging_trace();
//
//    for _ in 0..WORK_LOAD {
//        mutator.yieldpoint();
//
//        let res = gc::muentry_alloc_large(&mut mutator, LARGE_OBJECT_SIZE, OBJECT_ALIGN);
//        gc::muentry_init_object(&mut mutator, res, FIXSIZE_NOREF_ENCODE);
//    }
//
//    mutator.destroy();
//}
//
//#[test]
//#[allow(unused_variables)]
//fn test_alloc_large_lo_trigger_gc() {
//    const KEEP_N_ROOTS: usize = 1;
//    let mut roots: usize = 0;
//
//    gc::gc_init(SMALL_SPACE_SIZE, 4096 * 10, 8, true);
//    let mut mutator = gc::new_mutator();
//
//    start_logging_trace();
//
//    for _ in 0..WORK_LOAD {
//        mutator.yieldpoint();
//
//        let res = gc::muentry_alloc_large(&mut mutator, LARGE_OBJECT_SIZE, OBJECT_ALIGN);
//        gc::muentry_init_object(&mut mutator, res, FIXSIZE_NOREF_ENCODE);
//
//        if roots < KEEP_N_ROOTS {
//            gc::add_to_root(res);
//            roots += 1;
//        }
//    }
//
//    mutator.destroy();
//}
//
//#[test]
//#[allow(unused_variables)]
//fn test_alloc_large_both_trigger_gc() {
//    gc::gc_init(SMALL_SPACE_SIZE, 4096 * 10, 8, true);
//    let mut mutator = gc::new_mutator();
//
//    start_logging_trace();
//
//    // this will exhaust the lo space
//    for _ in 0..10 {
//        mutator.yieldpoint();
//
//        let res = gc::muentry_alloc_large(&mut mutator, LARGE_OBJECT_SIZE, OBJECT_ALIGN);
//        gc::muentry_init_object(&mut mutator, res, FIXSIZE_NOREF_ENCODE);
//    }
//
//    // this will trigger a gc, and allocate it in the collected space
//    let res = gc::muentry_alloc_large(&mut mutator, LARGE_OBJECT_SIZE, OBJECT_ALIGN);
//    gc::muentry_init_object(&mut mutator, res, FIXSIZE_NOREF_ENCODE);
//
//    // this will trigger gcs for immix space
//    for _ in 0..100000 {
//        mutator.yieldpoint();
//
//        let res = mutator.alloc(OBJECT_SIZE, OBJECT_ALIGN);
//        mutator.init_object(res, FIXSIZE_REFx2_ENCODE);
//    }
//
//    mutator.destroy();
//}
//
//#[test]
//#[cfg(feature = "use-sidemap")]
//fn test_alloc_mark() {
//    gc::gc_init(IMMIX_SPACE_SIZE, LO_SPACE_SIZE, 8, false);
//    let mut mutator = gc::new_mutator();
//
//    println!(
//        "Trying to allocate 1 object of (size {}, align {}). ",
//        OBJECT_SIZE,
//        OBJECT_ALIGN
//    );
//    const ACTUAL_OBJECT_SIZE: usize = OBJECT_SIZE;
//    println!(
//        "Considering header size of {}, an object should be {}. ",
//        0,
//        ACTUAL_OBJECT_SIZE
//    );
//
//    println!(
//        "Trying to allocate {} objects, which will take roughly {} bytes",
//        WORK_LOAD,
//        WORK_LOAD * ACTUAL_OBJECT_SIZE
//    );
//    let mut objs = vec![];
//    for _ in 0..WORK_LOAD {
//        let res = mutator.alloc(ACTUAL_OBJECT_SIZE, OBJECT_ALIGN);
//        mutator.init_object(res, FIXSIZE_REFx2_ENCODE);
//
//        objs.push(unsafe { res.to_object_reference() });
//    }
//
//    let (shared_space, _) = gc::get_spaces();
//
//    println!("Start marking");
//    let mark_state = objectmodel::load_mark_state();
//
//    let line_mark_table = shared_space.line_mark_table();
//    let (space_start, space_end) = (shared_space.start(), shared_space.end());
//
//    let trace_map = shared_space.trace_map.ptr;
//
//    for i in 0..objs.len() {
//        let obj = unsafe { *objs.get_unchecked(i) };
//
//        // mark the object as traced
//        objectmodel::mark_as_traced(trace_map, space_start, obj, mark_state);
//
//        // mark meta-data
//        if obj.to_address() >= space_start && obj.to_address() < space_end {
//            line_mark_table.mark_line_live2(space_start, obj.to_address());
//        }
//    }
//
//    mutator.destroy();
//}
//
//#[test]
//#[cfg(not(feature = "use-sidemap"))]
//fn test_alloc_mark() {
//    gc::gc_init(IMMIX_SPACE_SIZE, LO_SPACE_SIZE, 8, false);
//    let mut mutator = gc::new_mutator();
//
//    println!(
//        "Trying to allocate 1 object of (size {}, align {}). ",
//        OBJECT_SIZE,
//        OBJECT_ALIGN
//    );
//    const ACTUAL_OBJECT_SIZE: usize = OBJECT_SIZE;
//    println!(
//        "Considering header size of {}, an object should be {}. ",
//        0,
//        ACTUAL_OBJECT_SIZE
//    );
//
//    println!(
//        "Trying to allocate {} objects, which will take roughly {} bytes",
//        WORK_LOAD,
//        WORK_LOAD * ACTUAL_OBJECT_SIZE
//    );
//    let mut objs = vec![];
//    for _ in 0..WORK_LOAD {
//        let res = mutator.alloc(ACTUAL_OBJECT_SIZE, OBJECT_ALIGN);
//        mutator.init_object(res, FIXSIZE_REFx2_ENCODE);
//
//        objs.push(unsafe { res.to_object_reference() });
//    }
//
//    let (shared_space, _) = gc::get_spaces();
//
//    println!("Start marking");
//    let mark_state = objectmodel::load_mark_state();
//
//    let line_mark_table = shared_space.line_mark_table();
//    let (space_start, space_end) = (shared_space.start(), shared_space.end());
//
//    let trace_map = shared_space.trace_map.ptr;
//
//    for i in 0..objs.len() {
//        let obj = unsafe { *objs.get_unchecked(i) };
//
//        // mark the object as traced
//        objectmodel::mark_as_traced(obj, mark_state);
//
//        // mark meta-data
//        if obj.to_address() >= space_start && obj.to_address() < space_end {
//            line_mark_table.mark_line_live2(space_start, obj.to_address());
//        }
//    }
//
//    mutator.destroy();
//}
//
//#[allow(dead_code)]
//struct Node<'a> {
//    hdr: u64,
//    next: &'a Node<'a>,
//    unused_ptr: usize,
//    unused_int: i32,
//    unused_int2: i32
//}
//
//#[test]
//fn test_alloc_trace() {
//    gc::gc_init(IMMIX_SPACE_SIZE, LO_SPACE_SIZE, 8, false);
//    let mut mutator = gc::new_mutator();
//    let (shared_space, lo_space) = gc::get_spaces();
//
//    println!(
//        "Trying to allocate 1 object of (size {}, align {}). ",
//        OBJECT_SIZE,
//        OBJECT_ALIGN
//    );
//    const ACTUAL_OBJECT_SIZE: usize = OBJECT_SIZE;
//    println!(
//        "Considering header size of {}, an object should be {}. ",
//        0,
//        ACTUAL_OBJECT_SIZE
//    );
//
//    println!(
//        "Trying to allocate {} objects, which will take roughly {} bytes",
//        WORK_LOAD,
//        WORK_LOAD * ACTUAL_OBJECT_SIZE
//    );
//    let root = mutator.alloc(ACTUAL_OBJECT_SIZE, OBJECT_ALIGN);
//    mutator.init_object(root, FIXSIZE_REFx1_ENCODE);
//
//    let mut prev = root;
//    for _ in 0..WORK_LOAD - 1 {
//        let res = mutator.alloc(ACTUAL_OBJECT_SIZE, OBJECT_ALIGN);
//        mutator.init_object(res, FIXSIZE_REFx1_ENCODE);
//
//        // set prev's 1st field (offset 0) to this object
//        unsafe { prev.store::<Address>(res) };
//
//        prev = res;
//    }
//
//    println!("Start tracing");
//    let mut roots = vec![unsafe { root.to_object_reference() }];
//
//    heap::gc::start_trace(&mut roots, shared_space, lo_space);
//
//    mutator.destroy();
//}
