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
use mu::ast::inst::*;
use mu::ast::op::*;
use mu::vm::*;
use mu::testutil;
use mu::utils::LinkedHashMap;

#[test]
fn test_inline_add_simple() {
    let lib = testutil::compile_fncs("add_trampoline", vec!["add_trampoline", "add"], &inline_add);

    unsafe {
        let inline_add : libloading::Symbol<unsafe extern fn(u64, u64) -> u64> = lib.get(b"add_trampoline").unwrap();

        let inline_add_1_1 = inline_add(1, 1);
        println!("add(1, 1) = {}", inline_add_1_1);
        assert!(inline_add_1_1 == 2);
    }
}

fn inline_add() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int64 = mu_int(64));
    funcsig!    ((vm) sig = (int64, int64) -> (int64));

    funcdecl!   ((vm) <sig> add);
    {
        // add
        funcdef!    ((vm) <sig> add VERSION add_v1);

        block!      ((vm, add_v1) blk_entry);
        ssa!        ((vm, add_v1) <int64> x);
        ssa!        ((vm, add_v1) <int64> y);

        ssa!        ((vm, add_v1) <int64> res);
        inst!       ((vm, add_v1) blk_entry_add:
            res = BINOP (BinOp::Add) x y
        );

        inst!       ((vm, add_v1) blk_entry_ret:
            RET (res)
        );

        define_block!   ((vm, add_v1) blk_entry(x, y) {blk_entry_add, blk_entry_ret});

        define_func_ver!((vm) add_v1 (entry: blk_entry) {blk_entry});
    }

    {
        // add_trampoline
        typedef!    ((vm) funcref_to_sig = mu_funcref(sig));
        constdef!   ((vm) <funcref_to_sig> funcref_add = Constant::FuncRef(add));

        funcdecl!   ((vm) <sig> add_trampoline);
        funcdef!    ((vm) <sig> add_trampoline VERSION add_trampoline_v1);

        block!      ((vm, add_trampoline_v1) tramp_blk_entry);
        ssa!        ((vm, add_trampoline_v1) <int64> tramp_x);
        ssa!        ((vm, add_trampoline_v1) <int64> tramp_y);

        consta!     ((vm, add_trampoline_v1) funcref_add_local = funcref_add);
        ssa!        ((vm, add_trampoline_v1) <int64> tramp_res);
        inst!       ((vm, add_trampoline_v1) tramp_blk_call:
            tramp_res = EXPRCALL (CallConvention::Mu, is_abort: false) funcref_add_local (tramp_x, tramp_y)
        );

        inst!       ((vm, add_trampoline_v1) tramp_blk_ret:
            RET (tramp_res)
        );

        define_block!   ((vm, add_trampoline_v1) tramp_blk_entry(tramp_x, tramp_y) {tramp_blk_call, tramp_blk_ret});

        define_func_ver!((vm) add_trampoline_v1 (entry: tramp_blk_entry) {tramp_blk_entry});
    }

    vm
}

#[test]
fn test_inline_add_twice() {
    let lib = testutil::compile_fncs("add_twice", vec!["add_twice", "add"], &inline_add_twice);

    unsafe {
        let add_twice : libloading::Symbol<unsafe extern fn(u64, u64, u64) -> u64> = lib.get(b"add_twice").unwrap();

        let res = add_twice(1, 1, 1);
        println!("add(1, 1, 1) = {}", res);
        assert!(res == 3);
    }
}

fn inline_add_twice() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int64 = mu_int(64));
    funcsig!    ((vm) sig = (int64, int64) -> (int64));

    funcdecl!   ((vm) <sig> add);
    {
        // add
        funcdef!    ((vm) <sig> add VERSION add_v1);

        block!      ((vm, add_v1) blk_entry);
        ssa!        ((vm, add_v1) <int64> x);
        ssa!        ((vm, add_v1) <int64> y);

        ssa!        ((vm, add_v1) <int64> res);
        inst!       ((vm, add_v1) blk_entry_add:
            res = BINOP (BinOp::Add) x y
        );

        inst!       ((vm, add_v1) blk_entry_ret:
            RET (res)
        );

        define_block!   ((vm, add_v1) blk_entry(x, y) {blk_entry_add, blk_entry_ret});

        define_func_ver!((vm) add_v1 (entry: blk_entry) {blk_entry});
    }

    {
        // add_twice
        typedef!    ((vm) funcref_to_sig = mu_funcref(sig));
        constdef!   ((vm) <funcref_to_sig> funcref_add = Constant::FuncRef(add));

        funcsig!    ((vm) add_twice_sig = (int64, int64, int64) -> (int64));

        funcdecl!   ((vm) <add_twice_sig> add_twice);
        funcdef!    ((vm) <add_twice_sig> add_twice VERSION add_twice_v1);

        block!      ((vm, add_twice_v1) blk_entry);
        ssa!        ((vm, add_twice_v1) <int64> x);
        ssa!        ((vm, add_twice_v1) <int64> y);
        ssa!        ((vm, add_twice_v1) <int64> z);

        consta!     ((vm, add_twice_v1) funcref_add_local = funcref_add);
        ssa!        ((vm, add_twice_v1) <int64> add_twice_res1);
        inst!       ((vm, add_twice_v1) call:
            add_twice_res1 = EXPRCALL (CallConvention::Mu, is_abort: false) funcref_add_local (x, y)
        );

        ssa!        ((vm, add_twice_v1) <int64> add_twice_res2);
        inst!       ((vm, add_twice_v1) call2:
            add_twice_res2 = EXPRCALL (CallConvention::Mu, is_abort: false) funcref_add_local (add_twice_res1, z)
        );

        inst!       ((vm, add_twice_v1) ret:
            RET (add_twice_res2)
        );

        define_block!   ((vm, add_twice_v1) blk_entry(x, y, z) {call, call2, ret});

        define_func_ver!((vm) add_twice_v1 (entry: blk_entry) {blk_entry});
    }

    vm
}

#[test]
fn test_inline_add_with_extra_norm_args() {
    let lib = testutil::compile_fncs("inline_add_with_extra_norm_args", vec!["add_with_extra_norm_args", "add"], &inline_add_with_extra_norm_args);

    unsafe {
        let add_twice : libloading::Symbol<unsafe extern fn(u64, u64, u64) -> u64> = lib.get(b"add_with_extra_norm_args").unwrap();

        let res = add_twice(1, 1, 1);
        println!("add(1, 1, 1) = {}", res);
        assert!(res == 103);
    }
}

fn inline_add_with_extra_norm_args() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int64 = mu_int(64));
    funcsig!    ((vm) sig = (int64, int64) -> (int64));

    funcdecl!   ((vm) <sig> add);
    {
        // add
        funcdef!    ((vm) <sig> add VERSION add_v1);

        block!      ((vm, add_v1) blk_entry);
        ssa!        ((vm, add_v1) <int64> x);
        ssa!        ((vm, add_v1) <int64> y);

        ssa!        ((vm, add_v1) <int64> res);
        inst!       ((vm, add_v1) blk_entry_add:
            res = BINOP (BinOp::Add) x y
        );

        inst!       ((vm, add_v1) blk_entry_ret:
            RET (res)
        );

        define_block!   ((vm, add_v1) blk_entry(x, y) {blk_entry_add, blk_entry_ret});

        define_func_ver!((vm) add_v1 (entry: blk_entry) {blk_entry});
    }

    {
        // inline_add_with_extra_norm_args
        typedef!    ((vm) funcref_to_sig = mu_funcref(sig));
        constdef!   ((vm) <funcref_to_sig> funcref_add = Constant::FuncRef(add));
        constdef!   ((vm) <int64> int64_0   = Constant::Int(0));
        constdef!   ((vm) <int64> int64_100 = Constant::Int(100));

        funcsig!    ((vm) sig_add_with_extra_norm_args = (int64, int64, int64) -> (int64));

        funcdecl!   ((vm) <sig_add_with_extra_norm_args> add_with_extra_norm_args);
        funcdef!    ((vm) <sig_add_with_extra_norm_args> add_with_extra_norm_args VERSION add_with_extra_norm_args_v1);

        block!      ((vm, add_with_extra_norm_args_v1) blk_entry);
        ssa!        ((vm, add_with_extra_norm_args_v1) <int64> x);
        ssa!        ((vm, add_with_extra_norm_args_v1) <int64> y);
        ssa!        ((vm, add_with_extra_norm_args_v1) <int64> arg);

        block!      ((vm, add_with_extra_norm_args_v1) blk_norm);
        block!      ((vm, add_with_extra_norm_args_v1) blk_exn);
        consta!     ((vm, add_with_extra_norm_args_v1) funcref_add_local = funcref_add);
        consta!     ((vm, add_with_extra_norm_args_v1) int64_100_local = int64_100);
        ssa!        ((vm, add_with_extra_norm_args_v1) <int64> res);
        inst!       ((vm, add_with_extra_norm_args_v1) call:
            //          0                , 1, 2, 3  , 4  , 5
            res = CALL (funcref_add_local, x, y, res, arg, int64_100_local) FUNC(0) (vec![1, 2]) CallConvention::Mu,
                  normal: blk_norm (vec![DestArg::Normal(3), DestArg::Normal(4), DestArg::Normal(5)]),
                  exc: blk_exn (vec![])
        );

        define_block!((vm, add_with_extra_norm_args_v1) blk_entry(x, y, arg) {call});

        // blk_normal
        ssa!        ((vm, add_with_extra_norm_args_v1) <int64> normal_a);
        ssa!        ((vm, add_with_extra_norm_args_v1) <int64> normal_b);
        ssa!        ((vm, add_with_extra_norm_args_v1) <int64> normal_c);

        // normal_res1 = add a, b
        ssa!        ((vm, add_with_extra_norm_args_v1) <int64> normal_res1);
        inst!       ((vm, add_with_extra_norm_args_v1) add1:
            normal_res1 = BINOP (BinOp::Add) normal_a normal_b
        );

        // normal_res2 = add normal_res1, c
        ssa!        ((vm, add_with_extra_norm_args_v1) <int64> normal_res2);
        inst!       ((vm, add_with_extra_norm_args_v1) add2:
            normal_res2 = BINOP (BinOp::Add) normal_res1 normal_c
        );

        // RET normal_res2
        inst!       ((vm, add_with_extra_norm_args_v1) normal_ret:
            RET (normal_res2)
        );

        define_block!((vm, add_with_extra_norm_args_v1) blk_norm(normal_a, normal_b, normal_c) {
            add1, add2, normal_ret
        });

        // blk_exn
        consta!     ((vm, add_with_extra_norm_args_v1) int64_0_local = int64_0);
        inst!       ((vm, add_with_extra_norm_args_v1) exn_ret:
            RET (int64_0_local)
        );

        define_block!((vm, add_with_extra_norm_args_v1) blk_exn() {
            exn_ret
        });

        define_func_ver!((vm) add_with_extra_norm_args_v1 (entry: blk_entry) {blk_entry, blk_norm, blk_exn});
    }

    vm
}