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

extern crate libloading;

use mu::ast::types::*;
use mu::ast::ir::*;
use mu::ast::ptr::*;
use mu::ast::inst::*;
use mu::ast::op::*;
use mu::vm::*;
use mu::compiler::*;

use std::sync::Arc;
use mu::linkutils;
use mu::linkutils::aot;
use mu::utils::LinkedHashMap;

#[test]
fn test_infinite_loop1() {
    VM::start_logging_trace();

    let vm = Arc::new(infinite_loop1());

    let func_id = vm.id_of("infinite_loop1");
    let func_handle = vm.handle_from_func(func_id);
    let test_name = "test_infinite_loop1";
    vm.make_boot_image(
        vec![func_id],
        Some(&func_handle),
        None,
        None,
        vec![],
        vec![],
        vec![],
        vec![],
        test_name.to_string()
    );
}

fn infinite_loop1() -> VM {
    let vm = VM::new();

    funcsig!    ((vm) sig = () -> ());
    funcdecl!   ((vm) <sig> infinite_loop1);
    funcdef!    ((vm) <sig> infinite_loop1 VERSION infinite_loop1_v1);

    // entry:
    block!      ((vm, infinite_loop1_v1) blk_entry);
    block!      ((vm, infinite_loop1_v1) blk_loop);

    inst!       ((vm, infinite_loop1_v1) blk_entry_branch:
        BRANCH blk_loop ()
    );

    define_block!((vm, infinite_loop1_v1) blk_entry() {
        blk_entry_branch
    });

    // loop
    inst!       ((vm, infinite_loop1_v1) blk_loop_branch:
        BRANCH blk_loop ()
    );

    define_block!((vm, infinite_loop1_v1) blk_loop() {
        blk_loop_branch
    });

    define_func_ver!((vm) infinite_loop1_v1 (entry: blk_entry) {
        blk_entry, blk_loop
    });

    vm
}

#[test]
fn test_infinite_loop2() {
    VM::start_logging_trace();

    let vm = Arc::new(infinite_loop2());

    let func_id = vm.id_of("infinite_loop2");
    let func_handle = vm.handle_from_func(func_id);
    let test_name = "test_infinite_loop2";
    vm.make_boot_image(
        vec![func_id],
        Some(&func_handle),
        None,
        None,
        vec![],
        vec![],
        vec![],
        vec![],
        test_name.to_string()
    );
}

fn infinite_loop2() -> VM {
    let vm = VM::new();

    funcsig!    ((vm) sig = () -> ());
    funcdecl!   ((vm) <sig> infinite_loop2);
    funcdef!    ((vm) <sig> infinite_loop2 VERSION infinite_loop2_v1);

    // entry:
    block!      ((vm, infinite_loop2_v1) blk_entry);
    block!      ((vm, infinite_loop2_v1) blk_a);
    block!      ((vm, infinite_loop2_v1) blk_b);

    inst!       ((vm, infinite_loop2_v1) blk_entry_branch:
        BRANCH blk_a ()
    );

    define_block!((vm, infinite_loop2_v1) blk_entry() {
        blk_entry_branch
    });

    // blk a
    inst!       ((vm, infinite_loop2_v1) blk_a_branch:
        BRANCH blk_b ()
    );

    define_block!((vm, infinite_loop2_v1) blk_a() {
        blk_a_branch
    });

    // blk b
    inst!       ((vm, infinite_loop2_v1) blk_b_branch:
        BRANCH blk_a ()
    );

    define_block!((vm, infinite_loop2_v1) blk_b() {
        blk_b_branch
    });

    define_func_ver!((vm) infinite_loop2_v1 (entry: blk_entry) {
        blk_entry, blk_a, blk_b
    });

    vm
}