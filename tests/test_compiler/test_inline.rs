extern crate libloading;

use mu::ast::types::*;
use mu::ast::ir::*;
use mu::ast::inst::*;
use mu::ast::op::*;
use mu::vm::*;
use mu::compiler::*;
use mu::testutil;

use std::sync::Arc;
use std::sync::RwLock;
use mu::testutil::aot;

#[test]
fn test_inline_add() {
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