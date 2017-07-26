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

extern crate mu;
extern crate log;
extern crate libloading;

use self::mu::ast::types::*;
use self::mu::ast::ir::*;
use self::mu::ast::inst::*;
use self::mu::ast::op::*;
use self::mu::vm::*;
use self::mu::testutil;
use mu::utils::LinkedHashMap;

use self::mu::compiler::*;
use self::mu::testutil::aot;
use std::sync::Arc;

#[test]
fn test_truncate_then_call() {
//    let lib = testutil::compile_fncs("truncate_then_call", vec!["truncate_then_call", "dummy_call"], &truncate_then_call);
//
//    unsafe {
//        let truncate_then_call : libloading::Symbol<unsafe extern fn(u64) -> u32> = lib.get(b"truncate_then_call").unwrap();
//
//        let res = truncate_then_call(1);
//        println!("truncate_then_call(1) = {}", res);
//        assert!(res == 1);
//    }
    build_and_run_test!(truncate_then_call AND dummy_call, truncate_then_call_test1);
}

fn truncate_then_call() -> VM {
    let vm = VM::new_with_opts("init_mu --disable-inline");

    typedef! ((vm) u64 = mu_int(64));
    typedef! ((vm) u32 = mu_int(32));

    funcsig! ((vm) dummy_call_sig = (u32) -> (u32));
    funcdecl!((vm) <dummy_call_sig> dummy_call);
    {
        // --- dummy call ---
        funcdef! ((vm) <dummy_call_sig> dummy_call VERSION dummy_call_v1);

        // entry
        block! ((vm, dummy_call_v1) blk_entry);
        ssa!   ((vm, dummy_call_v1) <u32> x);

        inst!  ((vm, dummy_call_v1) ret:
            RET (x)
        );

        define_block!((vm, dummy_call_v1) blk_entry(x) {
            ret
        });

        define_func_ver!((vm) dummy_call_v1 (entry: blk_entry) {
            blk_entry
        });
    }

//    {
        // --- truncate_then_call ---
        typedef! ((vm) funcref_to_dummy = mu_funcref(dummy_call_sig));
        constdef!((vm) <funcref_to_dummy> funcref_dummy = Constant::FuncRef(dummy_call));

        funcsig! ((vm) sig = (u64) -> (u32));
        funcdecl!((vm) <sig> truncate_then_call);
        funcdef! ((vm) <sig> truncate_then_call VERSION truncate_then_call_v1);

        // entry
        block!((vm, truncate_then_call_v1) blk_entry);
        ssa!  ((vm, truncate_then_call_v1) <u64> arg);

        // %arg_u32 = TRUNC <u64 u32> arg
        ssa! ((vm, truncate_then_call_v1) <u32> arg_u32);
        inst!((vm, truncate_then_call_v1) blk_entry_truncate:
            arg_u32 = CONVOP (ConvOp::TRUNC) <u64 u32> arg
        );

        // %ret = CALL dummy_call (arg_u32)
        ssa!    ((vm, truncate_then_call_v1) <u32> res);
        consta! ((vm, truncate_then_call_v1) funcref_dummy_local = funcref_dummy);
        inst!   ((vm, truncate_then_call_v1) blk_entry_call:
            res = EXPRCALL (CallConvention::Mu, is_abort: false) funcref_dummy_local (arg_u32)
        );

        inst!((vm, truncate_then_call_v1) blk_entry_ret:
            RET (arg)
        );

        define_block!((vm, truncate_then_call_v1) blk_entry(arg) {
            blk_entry_truncate,
            blk_entry_call,
            blk_entry_ret
        });

        define_func_ver!((vm) truncate_then_call_v1 (entry: blk_entry) {
            blk_entry
        });
//    }
    
    emit_test!      ((vm) (truncate_then_call truncate_then_call_test1 truncate_then_call_test1_v1 Int,Int,EQ (sig, u64(1u64), u32(1u64))));
    
    vm
}