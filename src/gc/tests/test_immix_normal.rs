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
pub fn test_normal_immix_linkedlist() {
    const IMMIX_SPACE_SIZE: usize = SMALL_SPACE_SIZE;
    const OBJECT_SIZE: usize = 32; // small object
    const OBJECT_ALIGN: usize = 8;
    const WORK_LOAD: usize = 4;

    start_logging_trace();
    gc_init(GCConfig {
        immix_tiny_size: 0,
        immix_normal_size: IMMIX_SPACE_SIZE,
        lo_size: 0,
        n_gcthreads: 1,
        enable_gc: true
    });

    // insert type (id:0, 32 bytes, 1st field is a reference)
    let small_header = {
        let fix_ty = {
            let mut ret = [0u8; 63];
            ret[0] = 0b00000001u8;
            ret
        };
        let id = GlobalTypeTable::insert_small_entry(
            ShortTypeEncode::new(OBJECT_ALIGN, 4, fix_ty, 0, [0; 63])
        );
        println!("type id = {}", id);
        let raw_encode = 0b1000_0000_0000_0000u16 | ((id & 0b0001_1111_1111_1111usize) as u16);
        SmallObjectEncode::new(raw_encode)
    };
    println!("Small Header: {:?}", small_header);

    let normal_space = get_space_immix_normal();
    let mutator = new_mutator_ptr();

    let mut last_obj: Address = unsafe { Address::zero() };
    for _ in 0..WORK_LOAD {
        yieldpoint(mutator);
        let res = muentry_alloc_normal(mutator, OBJECT_SIZE, OBJECT_ALIGN);
        muentry_init_small_object(mutator, res, small_header);
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
    assert_eq!(normal_space.last_gc_used_lines, 2);

    // another gc
    force_gc(mutator);
    assert_eq!(GC_COUNT.load(Ordering::SeqCst), 2);
    assert_eq!(normal_space.last_gc_used_lines, 2);

    // set the linked list free, and do gc
    remove_root(last_obj);
    force_gc(mutator);
    assert_eq!(GC_COUNT.load(Ordering::SeqCst), 3);
    assert_eq!(normal_space.last_gc_used_lines, 0);

    drop_mutator(mutator);
    gc_destroy();
}

#[test]
pub fn test_normal_immix_hybrid() {
    const IMMIX_SPACE_SIZE: usize = SMALL_SPACE_SIZE;
    const OBJECT_SIZE: usize = 16; // small object
    const OBJECT_ALIGN: usize = 8;
    const WORK_LOAD: usize = 4;

    const HYBRID_LEN: usize = 4;
    const HYBRID_FIX_SIZE: usize = 32;
    const HYBRID_VAR_SIZE: usize = OBJECT_SIZE;
    const HYBRID_SIZE: usize = HYBRID_FIX_SIZE + HYBRID_LEN * HYBRID_VAR_SIZE;

    start_logging_trace();
    gc_init(GCConfig {
        immix_tiny_size: IMMIX_SPACE_SIZE,
        immix_normal_size: IMMIX_SPACE_SIZE,
        lo_size: 0,
        n_gcthreads: 1,
        enable_gc: true
    });

    let tiny_header = TinyObjectEncode::new(0);
    let hybrid_header = {
        let var_ty = {
            let mut ret = [0u8; 63];
            ret[0] = 0b01u8;
            ret
        };
        let encode = ShortTypeEncode::new(
            OBJECT_ALIGN,
            (HYBRID_FIX_SIZE >> LOG_POINTER_SIZE) as u8,
            [0; 63],
            (HYBRID_VAR_SIZE >> LOG_POINTER_SIZE) as u8,
            var_ty
        );
        let id = GlobalTypeTable::insert_large_entry(encode);
        println!("hybrid type id = {}", id);
        let raw_encode = ((id << 8) | 0b100usize) as u32;
        MediumObjectEncode::new(raw_encode)
    };
    println!("Tiny header: {:?}", tiny_header);
    println!("Hybrid header: {:?}", hybrid_header);

    let tiny_space = get_space_immix_tiny();
    let normal_space = get_space_immix_normal();
    let mutator = new_mutator_ptr();

    // alloc 4 tiny object
    let mut tiny_objects = vec![];
    for _ in 0..WORK_LOAD {
        yieldpoint(mutator);
        let res = muentry_alloc_tiny(mutator, OBJECT_SIZE, OBJECT_ALIGN);
        muentry_init_tiny_object(mutator, res, tiny_header);
        tiny_objects.push(res);
    }

    // alloc a hybrid
    let hyb = muentry_alloc_normal(mutator, HYBRID_SIZE, OBJECT_ALIGN);
    muentry_init_medium_object(mutator, hyb, hybrid_header);

    // put references to tiny objects as var part of the hybrid
    let hyb_base = hyb.to_address();
    for i in 0..WORK_LOAD {
        let offset: ByteOffset = (HYBRID_FIX_SIZE + (i * HYBRID_VAR_SIZE)) as isize;
        unsafe { (hyb_base + offset).store(tiny_objects[i]) }
    }

    add_to_root(hyb);
    force_gc(mutator);
    assert_eq!(GC_COUNT.load(Ordering::SeqCst), 1);
    assert_eq!(tiny_space.last_gc_used_lines, 2);
    assert_eq!(normal_space.last_gc_used_lines, 2);

    force_gc(mutator);
    assert_eq!(GC_COUNT.load(Ordering::SeqCst), 2);
    assert_eq!(tiny_space.last_gc_used_lines, 2);
    assert_eq!(normal_space.last_gc_used_lines, 2);

    remove_root(hyb);
    force_gc(mutator);
    assert_eq!(GC_COUNT.load(Ordering::SeqCst), 3);
    assert_eq!(tiny_space.last_gc_used_lines, 0);
    assert_eq!(normal_space.last_gc_used_lines, 0);

    drop_mutator(mutator);
    gc_destroy();
}

#[test]
pub fn test_normal_immix_straddle() {
    const IMMIX_SPACE_SIZE: usize = SMALL_SPACE_SIZE;
    const OBJECT_SIZE: usize = 1024; // 4 lines
    const OBJECT_ALIGN: usize = 8;
    const WORK_LOAD: usize = 4;

    start_logging_trace();
    gc_init(GCConfig {
        immix_tiny_size: 0,
        immix_normal_size: IMMIX_SPACE_SIZE,
        lo_size: 0,
        n_gcthreads: 1,
        enable_gc: true
    });

    let header = {
        let ty_encode = ShortTypeEncode::new(OBJECT_ALIGN, 64, [0; 63], 0, [0; 63]);
        let id = GlobalTypeTable::insert_large_entry(ty_encode);
        let raw_encode = ((id << 8) | 0b1111000usize) as u32;
        MediumObjectEncode::new(raw_encode)
    };
    println!("Header: {:?}", header);

    let normal_space = get_space_immix_normal();
    let mutator = new_mutator_ptr();

    // alloc 4 objects
    let mut objects = vec![];
    for _ in 0..WORK_LOAD {
        let res = muentry_alloc_normal(mutator, OBJECT_SIZE, OBJECT_ALIGN);
        muentry_init_medium_object(mutator, res, header);
        objects.push(res);
    }

    for obj in objects.iter() {
        add_to_root(*obj);
    }
    force_gc(mutator);
    assert_eq!(GC_COUNT.load(Ordering::SeqCst), 1);
    assert_eq!(normal_space.last_gc_used_lines, 16);

    force_gc(mutator);
    assert_eq!(GC_COUNT.load(Ordering::SeqCst), 2);
    assert_eq!(normal_space.last_gc_used_lines, 16);

    for obj in objects.iter() {
        remove_root(*obj);
    }
    force_gc(mutator);
    assert_eq!(GC_COUNT.load(Ordering::SeqCst), 3);
    assert_eq!(normal_space.last_gc_used_lines, 0);

    drop_mutator(mutator);
    gc_destroy();
}

#[test]
pub fn test_normal_immix_mix() {
    const IMMIX_SPACE_SIZE: usize = SMALL_SPACE_SIZE;
    const STRADDLE_OBJECT_SIZE: usize = 1024; // 4 lines
    const NORMAL_OBJECT_SIZE: usize = 64;
    const OBJECT_ALIGN: usize = 8;
    const WORK_LOAD: usize = 4;

    start_logging_trace();
    gc_init(GCConfig {
        immix_tiny_size: 0,
        immix_normal_size: IMMIX_SPACE_SIZE,
        lo_size: 0,
        n_gcthreads: 1,
        enable_gc: true
    });

    let straddle_header = {
        let ty_encode = ShortTypeEncode::new(OBJECT_ALIGN, 64, [0; 63], 0, [0; 63]);
        let id = GlobalTypeTable::insert_large_entry(ty_encode);
        let raw_encode = ((id << 8) | 0b1111000usize) as u32;
        MediumObjectEncode::new(raw_encode)
    };
    let normal_header = {
        let ty_encode = ShortTypeEncode::new(OBJECT_ALIGN, 8, [0; 63], 0, [0; 63]);
        let id = GlobalTypeTable::insert_large_entry(ty_encode);
        let raw_encode = ((id << 8) | 0usize) as u32;
        MediumObjectEncode::new(raw_encode)
    };
    println!("Straddle Header: {:?}", straddle_header);
    println!("Normal Header: {:?}", normal_header);

    let normal_space = get_space_immix_normal();
    let mutator = new_mutator_ptr();

    // alloc 4 straddle objects and 1 normal object
    let mut objects = vec![];
    for _ in 0..WORK_LOAD {
        let res = muentry_alloc_normal(mutator, STRADDLE_OBJECT_SIZE, OBJECT_ALIGN);
        muentry_init_medium_object(mutator, res, straddle_header);
        objects.push(res);
    }
    let res = muentry_alloc_normal(mutator, NORMAL_OBJECT_SIZE, OBJECT_ALIGN);
    muentry_init_medium_object(mutator, res, normal_header);
    objects.push(res);

    for obj in objects.iter() {
        add_to_root(*obj);
    }
    force_gc(mutator);
    assert_eq!(GC_COUNT.load(Ordering::SeqCst), 1);
    assert_eq!(normal_space.last_gc_used_lines, 18);

    force_gc(mutator);
    assert_eq!(GC_COUNT.load(Ordering::SeqCst), 2);
    assert_eq!(normal_space.last_gc_used_lines, 18);

    for obj in objects.iter() {
        remove_root(*obj);
    }
    force_gc(mutator);
    assert_eq!(GC_COUNT.load(Ordering::SeqCst), 3);
    assert_eq!(normal_space.last_gc_used_lines, 0);

    drop_mutator(mutator);
    gc_destroy();
}
