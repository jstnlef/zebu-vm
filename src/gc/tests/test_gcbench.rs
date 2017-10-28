//// Copyright 2017 The Australian National University
////
//// Licensed under the Apache License, Version 2.0 (the "License");
//// you may not use this file except in compliance with the License.
//// You may obtain a copy of the License at
////
////     http://www.apache.org/licenses/LICENSE-2.0
////
//// Unless required by applicable law or agreed to in writing, software
//// distributed under the License is distributed on an "AS IS" BASIS,
//// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//// See the License for the specific language governing permissions and
//// limitations under the License.
//
//#![allow(non_upper_case_globals)]
//#![allow(non_camel_case_types)]
//#![allow(non_snake_case)]
//#![allow(unused_variables)]
//#![allow(dead_code)]
//#![allow(unused_imports)]
//
//extern crate mu_gc as gc;
//extern crate mu_utils as utils;
//extern crate time;
//
//use self::gc::start_logging_trace;
//use self::gc::heap;
//use self::gc::heap::immix::ImmixAllocator;
//use self::gc::heap::immix::ImmixSpace;
//use self::gc::heap::freelist;
//use self::gc::heap::freelist::FreeListSpace;
//use self::gc::objectmodel;
//use self::utils::{ObjectReference, Address};
//use std::mem::size_of;
//use std::sync::atomic::Ordering;
//
//extern crate log;
//
//const IMMIX_SPACE_SIZE: usize = 40 << 20;
//const LO_SPACE_SIZE: usize = 40 << 20;
//
//const kStretchTreeDepth: i32 = 18;
//const kLongLivedTreeDepth: i32 = 16;
//const kArraySize: i32 = 500000;
//const kMinTreeDepth: i32 = 4;
//const kMaxTreeDepth: i32 = 16;
//
//struct Node {
//    left: *mut Node,
//    right: *mut Node,
//    i: i32,
//    j: i32
//}
//
//struct Array {
//    value: [f64; kArraySize as usize]
//}
//
//fn init_Node(me: *mut Node, l: *mut Node, r: *mut Node) {
//    unsafe {
//        (*me).left = l;
//        (*me).right = r;
//    }
//}
//
//fn TreeSize(i: i32) -> i32 {
//    (1 << (i + 1)) - 1
//}
//
//fn NumIters(i: i32) -> i32 {
//    2 * TreeSize(kStretchTreeDepth) / TreeSize(i)
//}
//
//fn Populate(iDepth: i32, thisNode: *mut Node, mutator: &mut ImmixAllocator) {
//    if iDepth <= 0 {
//        return;
//    } else {
//        unsafe {
//            (*thisNode).left = alloc(mutator);
//            (*thisNode).right = alloc(mutator);
//            Populate(iDepth - 1, (*thisNode).left, mutator);
//            Populate(iDepth - 1, (*thisNode).right, mutator);
//        }
//    }
//}
//
//fn MakeTree(iDepth: i32, mutator: &mut ImmixAllocator) -> *mut Node {
//    if iDepth <= 0 {
//        alloc(mutator)
//    } else {
//        let left = MakeTree(iDepth - 1, mutator);
//        let right = MakeTree(iDepth - 1, mutator);
//        let result = alloc(mutator);
//        init_Node(result, left, right);
//
//        result
//    }
//}
//
//fn PrintDiagnostics() {}
//
//fn TimeConstruction(depth: i32, mutator: &mut ImmixAllocator) {
//    let iNumIters = NumIters(depth);
//    println!("creating {} trees of depth {}", iNumIters, depth);
//
//    let tStart = time::now_utc();
//    for _ in 0..iNumIters {
//        let tempTree = alloc(mutator);
//        Populate(depth, tempTree, mutator);
//
//        // destroy tempTree
//    }
//    let tFinish = time::now_utc();
//    println!(
//        "\tTop down construction took {} msec",
//        (tFinish - tStart).num_milliseconds()
//    );
//
//    let tStart = time::now_utc();
//    for _ in 0..iNumIters {
//        let tempTree = MakeTree(depth, mutator);
//    }
//    let tFinish = time::now_utc();
//    println!(
//        "\tButtom up construction took {} msec",
//        (tFinish - tStart).num_milliseconds()
//    );
//}
//
//#[cfg(feature = "use-sidemap")]
//const FIXSIZE_REFx2_ENCODE: u64 = 0b1100_0011u64;
//#[cfg(not(feature = "use-sidemap"))]
//const FIXSIZE_REFx2_ENCODE: u64 = 0xb000000000000003u64;
//
//#[inline(always)]
//#[cfg(feature = "use-sidemap")]
//fn alloc(mutator: &mut ImmixAllocator) -> *mut Node {
//    let addr = mutator.alloc(size_of::<Node>(), 8);
//    mutator.init_object(addr, FIXSIZE_REFx2_ENCODE);
//
//    addr.to_ptr_mut::<Node>()
//}
//
//#[inline(always)]
//#[cfg(not(feature = "use-sidemap"))]
//fn alloc(mutator: &mut ImmixAllocator) -> *mut Node {
//    let addr = mutator.alloc(size_of::<Node>(), 8);
//    mutator.init_object(addr, FIXSIZE_REFx2_ENCODE);
//
//    if cfg!(debug_assertions) {
//        unsafe {
//            let hdr = (addr + objectmodel::OBJECT_HEADER_OFFSET).load::<u64>();
//            assert!(objectmodel::header_is_object_start(hdr));
//        }
//    }
//
//    addr.to_ptr_mut::<Node>()
//}
//
//#[test]
//fn start() {
//    unsafe {
//        heap::gc::set_low_water_mark();
//    }
//
//    start_logging_trace();
//
//    gc::gc_init(IMMIX_SPACE_SIZE, LO_SPACE_SIZE, 1, true);
//    gc::print_gc_context();
//
//    let mut mutator = gc::new_mutator();
//
//    println!("Garbage Collector Test");
//    println!(" Node size = {}", size_of::<Node>());
//    println!(
//        " Live storage will peak at {} bytes.\n",
//        2 * (size_of::<Node>() as i32) * TreeSize(kLongLivedTreeDepth) +
//            (size_of::<Array>() as i32)
//    );
//
//    println!(
//        " Stretching memory with a binary tree or depth {}",
//        kStretchTreeDepth
//    );
//    PrintDiagnostics();
//
//    let tStart = time::now_utc();
//    // Stretch the memory space quickly
//    let tempTree = MakeTree(kStretchTreeDepth, &mut mutator);
//    // destroy tree
//
//    // Create a long lived object
//    println!(
//        " Creating a long-lived binary tree of depth {}",
//        kLongLivedTreeDepth
//    );
//    let longLivedTree = alloc(&mut mutator);
//    Populate(kLongLivedTreeDepth, longLivedTree, &mut mutator);
//    gc::add_to_root(unsafe {
//        Address::from_mut_ptr(longLivedTree).to_object_reference()
//    });
//
//    println!(" Creating a long-lived array of {} doubles", kArraySize);
//    //    mm::alloc_large(&mut mutator, size_of::<Array>(), 8);
//
//    PrintDiagnostics();
//
//    let mut d = kMinTreeDepth;
//    while d <= kMaxTreeDepth {
//        TimeConstruction(d, &mut mutator);
//        d += 2;
//    }
//
//    if longLivedTree.is_null() {
//        println!("Failed(long lived tree wrong)");
//    }
//
//    //    if array.array[1000] != 1.0f64 / (1000 as f64) {
//    //        println!("Failed(array element wrong)");
//    //    }
//
//    let tFinish = time::now_utc();
//    let tElapsed = (tFinish - tStart).num_milliseconds();
//
//    PrintDiagnostics();
//    println!("Completed in {} msec", tElapsed);
//    println!(
//        "Finished with {} collections",
//        heap::gc::GC_COUNT.load(Ordering::SeqCst)
//    );
//
//    mutator.destroy();
//}
