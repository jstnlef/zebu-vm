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

#![allow(unused_imports)]

extern crate mu;

use self::mu::ast::types::*;
use self::mu::ast::ir::*;
use self::mu::ast::inst::*;
use self::mu::ast::op::*;
use self::mu::vm::*;
use self::mu::compiler::*;
use self::mu::utils::LinkedHashMap;
use self::mu::testutil::aot;

use std::sync::Arc;
use std::sync::RwLock;

#[test]
fn test_thread_create() {
    build_and_run_test!(primordial_main, primordial_main_test1);
}

fn primordial_main() -> VM {
    let vm = VM::new();

    funcsig!    ((vm) sig = () -> ());
    funcdecl!   ((vm) <sig> primordial_main);
    funcdef!    ((vm) <sig> primordial_main VERSION primordial_main_v1);
    
    block!      ((vm, primordial_main_v1) blk_entry);
    inst!       ((vm, primordial_main_v1) blk_entry_threadexit:
        THREADEXIT
    );

    define_block!((vm, primordial_main_v1) blk_entry() {
        blk_entry_threadexit
    });

    define_func_ver!((vm) primordial_main_v1(entry: blk_entry) {
        blk_entry
    });
    
    emit_test!      ((vm) (primordial_main primordial_main_test1 primordial_main_test1_v1 (sig)));
    
    vm
}
