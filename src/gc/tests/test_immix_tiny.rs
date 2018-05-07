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

extern crate zebu_gc;
extern crate zebu_utils;
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
pub const SPACIOUS_SPACE_SIZE: usize = 500 << 20; // 500mb
#[allow(dead_code)]
pub const LIMITED_SPACE_SIZE: usize = 20 << 20; // 20mb
#[allow(dead_code)]
pub const SMALL_SPACE_SIZE: usize = 1 << 19; // 512kb

#[test]
pub fn test_tiny_immix_alloc() {
    const IMMIX_SPACE_SIZE: usize = SPACIOUS_SPACE_SIZE;
    const OBJECT_SIZE: usize = 16;
    const OBJECT_ALIGN: usize = 8;
    const WORK_LOAD: usize = BYTES_IN_BLOCK / OBJECT_SIZE;

    start_logging_trace();
    gc_init(GCConfig {
        immix_tiny_size: IMMIX_SPACE_SIZE,
        immix_normal_size: 0,
        lo_size: 0,
        n_gcthreads: 8,
        enable_gc: false
    });
    let tiny_space = get_space_immix_tiny();
    let mutator = new_mutator_ptr();
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
    const IMMIX_SPACE_SIZE: usize = SPACIOUS_SPACE_SIZE;
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
    let tiny_space = get_space_immix_tiny();
    let mutator = new_mutator_ptr();
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

    let tiny_space = get_space_immix_tiny();
    let mutator = new_mutator_ptr();
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

    let tiny_space = get_space_immix_tiny();
    let mutator = new_mutator_ptr();
    // first field is a reference, size 16
    let header = TinyObjectEncode::new(0b00000001u8);

    let mut last_obj: Address = unsafe { Address::zero() };
    for _ in 0..WORK_LOAD {
        yieldpoint(mutator);
        let res = muentry_alloc_tiny(mutator, OBJECT_SIZE, OBJECT_ALIGN);
        muentry_init_tiny_object(mutator, res, header);
        // the first field of this object points to the last object
        unsafe {
            res.to_address().store(last_obj);
        }
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
