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
pub fn test_freelist_linkedlist() {
    const FREELIST_SPACE_SIZE: usize = SPACIOUS_SPACE_SIZE;
    const OBJECT_SIZE: usize = 4096;
    const OBJECT_ALIGN: usize = 8;
    const WORK_LOAD: usize = 4;

    start_logging_trace();
    gc_init(GCConfig {
        immix_tiny_size: 0,
        immix_normal_size: 0,
        lo_size: FREELIST_SPACE_SIZE,
        n_gcthreads: 1,
        enable_gc: true
    });

    let header = {
        let mut fix = vec![WordType::NonRef; 512];
        fix[0] = WordType::Ref;
        let id = GlobalTypeTable::insert_full_entry(FullTypeEncode {
            align: 8,
            fix,
            var: vec![]
        });
        LargeObjectEncode::new(OBJECT_SIZE, id)
    };
    println!("Header: {:?}", header);

    let lo_space = get_space_freelist();
    let mutator = new_mutator_ptr();

    let mut last_obj: Address = unsafe { Address::zero() };
    for _ in 0..WORK_LOAD {
        yieldpoint(mutator);
        let res = muentry_alloc_large(mutator, OBJECT_SIZE, OBJECT_ALIGN);
        muentry_init_large_object(mutator, res, header);
        unsafe {
            res.to_address().store(last_obj);
        }
        last_obj = res.to_address();
    }

    let last_obj = unsafe { last_obj.to_object_reference() };
    add_to_root(last_obj);
    force_gc(mutator);
    assert_eq!(GC_COUNT.load(Ordering::SeqCst), 1);
    assert_eq!(lo_space.last_gc_used_pages, 4);

    force_gc(mutator);
    assert_eq!(GC_COUNT.load(Ordering::SeqCst), 2);
    assert_eq!(lo_space.last_gc_used_pages, 4);

    remove_root(last_obj);
    force_gc(mutator);
    assert_eq!(GC_COUNT.load(Ordering::SeqCst), 3);
    assert_eq!(lo_space.last_gc_used_pages, 0);

    drop_mutator(mutator);
    gc_destroy();
}
