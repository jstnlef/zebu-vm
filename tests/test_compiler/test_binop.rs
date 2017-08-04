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
use self::mu::linkutils;
use mu::utils::LinkedHashMap;

use std::sync::Arc;
use self::mu::linkutils::aot;
use self::mu::runtime::thread::check_result;
use self::mu::compiler::*;

use std::u64;
use std::i64;

#[test]
fn test_udiv() {
    build_and_run_test!(udiv, udiv_test1);
    build_and_run_test!(udiv, udiv_test2);
}

fn udiv() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int64 = mu_int(64));

    funcsig!    ((vm) udiv_sig = (int64, int64) -> (int64));
    funcdecl!   ((vm) <udiv_sig> udiv);
    funcdef!    ((vm) <udiv_sig> udiv VERSION udiv_v1);

    // %entry(<@int64> %a, <@int64> %b):
    block!      ((vm, udiv_v1) blk_entry);
    ssa!        ((vm, udiv_v1) <int64> a);
    ssa!        ((vm, udiv_v1) <int64> b);

    // %r = UDIV %a %b
    ssa!        ((vm, udiv_v1) <int64> r);
    inst!       ((vm, udiv_v1) blk_entry_udiv:
        r = BINOP (BinOp::Udiv) a b
    );

    // RET %r
    inst!       ((vm, udiv_v1) blk_entry_ret:
        RET (r)
    );

    define_block!((vm, udiv_v1) blk_entry(a, b) {
        blk_entry_udiv,
        blk_entry_ret
    });

    define_func_ver!((vm) udiv_v1(entry: blk_entry) {
        blk_entry
    });

    emit_test! ((vm)
        udiv, udiv_test1, udiv_test1_v1,
        Int, Int RET Int,
        EQ,
        udiv_sig,
        int64(22), int64(4) RET int64(5),
    );

    emit_test! ((vm)
        udiv, udiv_test2, udiv_test2_v1,
        Int, Int RET Int,
        EQ,
        udiv_sig,
        int64(27), int64(7) RET int64(3),
    );

    vm
}

#[test]
fn test_sdiv() {
    build_and_run_test!(sdiv, sdiv_test1);
    build_and_run_test!(sdiv, sdiv_test2);
}

fn sdiv() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int64 = mu_int(64));

    funcsig!    ((vm) sdiv_sig = (int64, int64) -> (int64));
    funcdecl!   ((vm) <sdiv_sig> sdiv);
    funcdef!    ((vm) <sdiv_sig> sdiv VERSION sdiv_v1);

    // %entry(<@int64> %a, <@int64> %b):
    block!      ((vm, sdiv_v1) blk_entry);
    ssa!        ((vm, sdiv_v1) <int64> a);
    ssa!        ((vm, sdiv_v1) <int64> b);

    // %r = sdiv %a %b
    ssa!        ((vm, sdiv_v1) <int64> r);
    inst!       ((vm, sdiv_v1) blk_entry_sdiv:
        r = BINOP (BinOp::Sdiv) a b
    );

    // RET %r
    inst!       ((vm, sdiv_v1) blk_entry_ret:
        RET (r)
    );

    define_block!((vm, sdiv_v1) blk_entry(a, b) {
        blk_entry_sdiv,
        blk_entry_ret
    });

    define_func_ver!((vm) sdiv_v1(entry: blk_entry) {
        blk_entry
    });

    emit_test! ((vm)
        sdiv, sdiv_test1, sdiv_test1_v1,
        Int, Int RET Int,
        EQ,
        sdiv_sig,
        int64(8), int64(2) RET int64(4),
    );

    emit_test! ((vm)
        sdiv, sdiv_test2, sdiv_test2_v1,
        Int, Int RET Int,
        EQ,
        sdiv_sig,
        int64(8), int64(-3i64 as u64) RET int64(-2i64 as u64),
    );

    vm
}

#[test]
fn test_shl() {
    build_and_run_test!(shl, shl_test1);
    build_and_run_test!(shl, shl_test2);
}

fn shl() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int64 = mu_int(64));

    funcsig!    ((vm) shl_sig = (int64, int64) -> (int64));
    funcdecl!   ((vm) <shl_sig> shl);
    funcdef!    ((vm) <shl_sig> shl VERSION shl_v1);

    // %entry(<@int64> %a, <@int64> %b):
    block!      ((vm, shl_v1) blk_entry);
    ssa!        ((vm, shl_v1) <int64> a);
    ssa!        ((vm, shl_v1) <int64> b);

    // %r = shl %a %b
    ssa!        ((vm, shl_v1) <int64> r);
    inst!       ((vm, shl_v1) blk_entry_shl:
        r = BINOP (BinOp::Shl) a b
    );

    // RET %r
    inst!       ((vm, shl_v1) blk_entry_ret:
        RET (r)
    );

    define_block!((vm, shl_v1) blk_entry(a, b) {
        blk_entry_shl,
        blk_entry_ret
    });

    define_func_ver!((vm) shl_v1(entry: blk_entry) {
        blk_entry
    });

    emit_test! ((vm)
        shl, shl_test1, shl_test1_v1,
        Int, Int RET Int,
        EQ,
        shl_sig,
        int64(1), int64(2) RET int64(4),
    );
    emit_test! ((vm)
        shl, shl_test2, shl_test2_v1,
        Int, Int RET Int,
        EQ,
        shl_sig,
        int64(2), int64(2) RET int64(8),
    );

    vm
}

#[test]
fn test_lshr() {
    build_and_run_test!(lshr, lshr_test1);
}

fn lshr() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int64 = mu_int(64));

    funcsig!    ((vm) lshr_sig = (int64, int64) -> (int64));
    funcdecl!   ((vm) <lshr_sig> lshr);
    funcdef!    ((vm) <lshr_sig> lshr VERSION lshr_v1);

    // %entry(<@int64> %a, <@int64> %b):
    block!      ((vm, lshr_v1) blk_entry);
    ssa!        ((vm, lshr_v1) <int64> a);
    ssa!        ((vm, lshr_v1) <int64> b);

    // %r = lshr %a %b
    ssa!        ((vm, lshr_v1) <int64> r);
    inst!       ((vm, lshr_v1) blk_entry_lshr:
        r = BINOP (BinOp::Lshr) a b
    );

    // RET %r
    inst!       ((vm, lshr_v1) blk_entry_ret:
        RET (r)
    );

    define_block!((vm, lshr_v1) blk_entry(a, b) {
        blk_entry_lshr,
        blk_entry_ret
    });

    define_func_ver!((vm) lshr_v1(entry: blk_entry) {
        blk_entry
    });

    emit_test! ((vm)
        lshr, lshr_test1, lshr_test1_v1,
        Int, Int RET Int,
        EQ,
        lshr_sig,
        int64(8), int64(3) RET int64(1),
    );

    vm
}

#[test]
fn test_add_simple() {
    build_and_run_test! (add, add_test1);
}

fn add() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int64 = mu_int(64));

    funcsig!    ((vm) sig = (int64, int64) -> (int64));
    funcdecl!   ((vm) <sig> add);
    funcdef!    ((vm) <sig> add VERSION add_v1);

    block!      ((vm, add_v1) blk_entry);
    ssa!        ((vm, add_v1) <int64> a);
    ssa!        ((vm, add_v1) <int64> b);

    // sum = Add %a %b
    ssa!        ((vm, add_v1) <int64> sum);
    inst!       ((vm, add_v1) blk_entry_add:
        sum = BINOP (BinOp::Add) a b
    );

    inst!       ((vm, add_v1) blk_entry_ret:
        RET (sum)
    );

    define_block!   ((vm, add_v1) blk_entry(a, b) {
        blk_entry_add, blk_entry_ret
    });

    define_func_ver!((vm) add_v1 (entry: blk_entry) {blk_entry});

    emit_test! ((vm)
        add, add_test1, add_test1_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int64(22), int64(27) RET int64(49),
    );

    vm
}

#[test]
fn test_add_int64_n() {
    build_and_run_test! (add_int64_n, add_int64_n_test1);
    build_and_run_test! (add_int64_n, add_int64_n_test2);
    build_and_run_test! (add_int64_n, add_int64_n_test3);
    build_and_run_test! (add_int64_n, add_int64_n_test4);
}

fn add_int64_n() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int64 = mu_int(64));
    typedef!    ((vm) int1  = mu_int(1));

    funcsig!    ((vm) sig = (int64, int64) -> (int1));
    funcdecl!   ((vm) <sig> add_int64_n);
    funcdef!    ((vm) <sig> add_int64_n VERSION add_int64_n_v1);

    block!      ((vm, add_int64_n_v1) blk_entry);
    ssa!        ((vm, add_int64_n_v1) <int64> a);
    ssa!        ((vm, add_int64_n_v1) <int64> b);

    // (sum, flag_n) = Add #N %a %b
    ssa!        ((vm, add_int64_n_v1) <int64> sum);
    ssa!        ((vm, add_int64_n_v1) <int1> flag_n);
    inst!       ((vm, add_int64_n_v1) blk_entry_add:
        sum, flag_n = BINOP_STATUS (BinOp::Add) (BinOpStatus::n()) a b
    );

    inst!       ((vm, add_int64_n_v1) blk_entry_ret:
        RET (flag_n)
    );

    define_block!   ((vm, add_int64_n_v1) blk_entry(a, b) {
        blk_entry_add, blk_entry_ret
    });

    define_func_ver!((vm) add_int64_n_v1 (entry: blk_entry) {blk_entry});

    emit_test! ((vm)
        add_int64_n, add_int64_n_test1, add_int64_n_test1_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int64(1), int64(1) RET int1(0),
    );
    emit_test! ((vm)
        add_int64_n, add_int64_n_test2, add_int64_n_test2_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int64(1), int64(-2i64 as u64) RET int1(1),
    );
    emit_test! ((vm)
        add_int64_n, add_int64_n_test3, add_int64_n_test3_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int64(1), int64(-1i64 as u64) RET int1(0),
    );
    emit_test! ((vm)
        add_int64_n, add_int64_n_test4, add_int64_n_test4_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int64(-1i64 as u64), int64(-1i64 as u64) RET int1(1),
    );

    vm
}

#[test]
fn test_add_int64_z() {
    build_and_run_test! (add_int64_z, add_int64_z_test1);
    build_and_run_test! (add_int64_z, add_int64_z_test2);
    build_and_run_test! (add_int64_z, add_int64_z_test3);
}

fn add_int64_z() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int64 = mu_int(64));
    typedef!    ((vm) int1  = mu_int(1));

    funcsig!    ((vm) sig = (int64, int64) -> (int1));
    funcdecl!   ((vm) <sig> add_int64_z);
    funcdef!    ((vm) <sig> add_int64_z VERSION add_int64_z_v1);

    block!      ((vm, add_int64_z_v1) blk_entry);
    ssa!        ((vm, add_int64_z_v1) <int64> a);
    ssa!        ((vm, add_int64_z_v1) <int64> b);

    // (sum, flag_n) = Add #N %a %b
    ssa!        ((vm, add_int64_z_v1) <int64> sum);
    ssa!        ((vm, add_int64_z_v1) <int1> flag_z);
    inst!       ((vm, add_int64_z_v1) blk_entry_add:
        sum, flag_z = BINOP_STATUS (BinOp::Add) (BinOpStatus::z()) a b
    );

    inst!       ((vm, add_int64_z_v1) blk_entry_ret:
        RET (flag_z)
    );

    define_block!   ((vm, add_int64_z_v1) blk_entry(a, b) {
        blk_entry_add, blk_entry_ret
    });

    define_func_ver!((vm) add_int64_z_v1 (entry: blk_entry) {blk_entry});

    emit_test! ((vm)
        add_int64_z, add_int64_z_test1, add_int64_z_test1_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int64(1), int64(1) RET int1(0),
    );
    emit_test! ((vm)
        add_int64_z, add_int64_z_test2, add_int64_z_test2_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int64(1), int64(-2i64 as u64) RET int1(0),
    );
    emit_test! ((vm)
        add_int64_z, add_int64_z_test3, add_int64_z_test3_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int64(1), int64(-1i64 as u64) RET int1(1),
    );

    vm
}

#[test]
fn test_add_int64_c() {
    build_and_run_test! (add_int64_c, add_int64_c_test1);
    build_and_run_test! (add_int64_c, add_int64_c_test2);
}

fn add_int64_c() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int64 = mu_int(64));
    typedef!    ((vm) int1  = mu_int(1));

    funcsig!    ((vm) sig = (int64, int64) -> (int1));
    funcdecl!   ((vm) <sig> add_int64_c);
    funcdef!    ((vm) <sig> add_int64_c VERSION add_int64_c_v1);

    block!      ((vm, add_int64_c_v1) blk_entry);
    ssa!        ((vm, add_int64_c_v1) <int64> a);
    ssa!        ((vm, add_int64_c_v1) <int64> b);

    // (sum, flag_n) = Add #N %a %b
    ssa!        ((vm, add_int64_c_v1) <int64> sum);
    ssa!        ((vm, add_int64_c_v1) <int1> flag_c);
    inst!       ((vm, add_int64_c_v1) blk_entry_add:
        sum, flag_c = BINOP_STATUS (BinOp::Add) (BinOpStatus::c()) a b
    );

    inst!       ((vm, add_int64_c_v1) blk_entry_ret:
        RET (flag_c)
    );

    define_block!   ((vm, add_int64_c_v1) blk_entry(a, b) {
        blk_entry_add, blk_entry_ret
    });

    define_func_ver!((vm) add_int64_c_v1 (entry: blk_entry) {blk_entry});

    emit_test! ((vm)
        add_int64_c, add_int64_c_test1, add_int64_c_test1_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int64(u64::MAX), int64(1) RET int1(1),
    );
    emit_test! ((vm)
        add_int64_c, add_int64_c_test2, add_int64_c_test2_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int64(i64::MAX as u64), int64(0) RET int1(0),
    );

    vm
}

#[test]
fn test_add_int64_v() {
    build_and_run_test! (add_int64_v, add_int64_v_test1);
    build_and_run_test! (add_int64_v, add_int64_v_test2);
    build_and_run_test! (add_int64_v, add_int64_v_test3);
    build_and_run_test! (add_int64_v, add_int64_v_test4);
}

fn add_int64_v() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int64 = mu_int(64));
    typedef!    ((vm) int1  = mu_int(1));

    funcsig!    ((vm) sig = (int64, int64) -> (int1));
    funcdecl!   ((vm) <sig> add_int64_v);
    funcdef!    ((vm) <sig> add_int64_v VERSION add_int64_v_v1);

    block!      ((vm, add_int64_v_v1) blk_entry);
    ssa!        ((vm, add_int64_v_v1) <int64> a);
    ssa!        ((vm, add_int64_v_v1) <int64> b);

    // (sum, flag_n) = Add #N %a %b
    ssa!        ((vm, add_int64_v_v1) <int64> sum);
    ssa!        ((vm, add_int64_v_v1) <int1> flag_v);
    inst!       ((vm, add_int64_v_v1) blk_entry_add:
        sum, flag_v = BINOP_STATUS (BinOp::Add) (BinOpStatus::v()) a b
    );

    inst!       ((vm, add_int64_v_v1) blk_entry_ret:
        RET (flag_v)
    );

    define_block!   ((vm, add_int64_v_v1) blk_entry(a, b) {
        blk_entry_add, blk_entry_ret
    });

    define_func_ver!((vm) add_int64_v_v1 (entry: blk_entry) {blk_entry});

    emit_test! ((vm)
        add_int64_v, add_int64_v_test1, add_int64_v_test1_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int64(i64::MAX as u64), int64(1) RET int1(1),
    );
    emit_test! ((vm)
        add_int64_v, add_int64_v_test2, add_int64_v_test2_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int64(i64::MAX as u64), int64(0) RET int1(0),
    );
    emit_test! ((vm)
        add_int64_v, add_int64_v_test3, add_int64_v_test3_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int64(i64::MIN as u64), int64(0) RET int1(0),
    );
    emit_test! ((vm)
        add_int64_v, add_int64_v_test4, add_int64_v_test4_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int64(i64::MIN as u64), int64(-1i64 as u64) RET int1(1),
    );

    vm
}

#[test]
fn test_add_int64_nzc() {
    build_and_run_test! (add_int64_nzc, add_int64_nzc_test1);
    build_and_run_test! (add_int64_nzc, add_int64_nzc_test2);
}

fn add_int64_nzc() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int64 = mu_int(64));
    typedef!    ((vm) int8  = mu_int(8));
    typedef!    ((vm) int1  = mu_int(1));

    constdef!   ((vm) <int8> int8_1 = Constant::Int(1));
    constdef!   ((vm) <int8> int8_2 = Constant::Int(2));
    constdef!   ((vm) <int8> int8_3 = Constant::Int(3));

    funcsig!    ((vm) sig = (int64, int64) -> (int1));
    funcdecl!   ((vm) <sig> add_int64_nzc);
    funcdef!    ((vm) <sig> add_int64_nzc VERSION add_int64_nzc_v1);

    block!      ((vm, add_int64_nzc_v1) blk_entry);
    ssa!        ((vm, add_int64_nzc_v1) <int64> a);
    ssa!        ((vm, add_int64_nzc_v1) <int64> b);

    // (sum, flag_n) = Add #N %a %b
    ssa!        ((vm, add_int64_nzc_v1) <int64> sum);
    ssa!        ((vm, add_int64_nzc_v1) <int1> flag_n);
    ssa!        ((vm, add_int64_nzc_v1) <int1> flag_z);
    ssa!        ((vm, add_int64_nzc_v1) <int1> flag_c);

    inst!       ((vm, add_int64_nzc_v1) blk_entry_add:
        sum, flag_n, flag_z, flag_c =
            BINOP_STATUS (BinOp::Add)
            (BinOpStatus{flag_n: true, flag_z: true, flag_c: true, flag_v: false}) a b
    );

    ssa!        ((vm, add_int64_nzc_v1) <int8> shift_z);
    consta!     ((vm, add_int64_nzc_v1) int8_1_local = int8_1);
    inst!       ((vm, add_int64_nzc_v1) blk_entry_shift_z:
        shift_z = BINOP (BinOp::Shl) flag_z int8_1_local
    );

    ssa!        ((vm, add_int64_nzc_v1) <int8> ret);
    inst!       ((vm, add_int64_nzc_v1) blk_entry_add_ret1:
        ret = BINOP (BinOp::Add) flag_n shift_z
    );

    ssa!        ((vm, add_int64_nzc_v1) <int8> shift_c);
    consta!     ((vm, add_int64_nzc_v1) int8_2_local = int8_2);
    inst!       ((vm, add_int64_nzc_v1) blk_entry_shift_c:
        shift_c = BINOP (BinOp::Shl) flag_c int8_2_local
    );

    ssa!        ((vm, add_int64_nzc_v1) <int8> ret2);
    inst!       ((vm, add_int64_nzc_v1) blk_entry_add_ret2:
        ret2 = BINOP (BinOp::Add) ret shift_c
    );

    inst!       ((vm, add_int64_nzc_v1) blk_entry_ret:
        RET (ret2)
    );

    define_block!   ((vm, add_int64_nzc_v1) blk_entry(a, b) {
        blk_entry_add, blk_entry_shift_z, blk_entry_add_ret1,
        blk_entry_shift_c, blk_entry_add_ret2, blk_entry_ret
    });

    define_func_ver!((vm) add_int64_nzc_v1 (entry: blk_entry) {blk_entry});

    emit_test! ((vm)
        add_int64_nzc, add_int64_nzc_test1, add_int64_nzc_test1_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int64(u64::MAX), int64(1) RET int8(0b110),
    );
    emit_test! ((vm)
        add_int64_nzc, add_int64_nzc_test2, add_int64_nzc_test2_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int64(u64::MAX), int64(0) RET int8(0b001),
    );

    vm
}
