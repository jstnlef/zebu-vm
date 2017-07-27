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
extern crate extprim;

use mu::ast::ir::*;
use mu::ast::inst::*;
use mu::ast::types::*;
use mu::ast::op::*;
use mu::vm::*;

use mu::utils::LinkedHashMap;
use mu::testutil;

use mu::compiler::*;
use mu::testutil::aot;
use std::sync::Arc;
use std::u64;

use self::extprim::u128::u128;

#[test]
fn test_add_u128() {
    build_and_run_test!(add_u128, add_u128_test1);
    build_and_run_test!(add_u128, add_u128_test2);
}

pub fn add_u128() -> VM {
    let vm = VM::new_with_opts("init_mu --disable-inline");

    typedef!    ((vm) u128 = mu_int(128));

    funcsig!    ((vm) sig = (u128, u128) -> (u128));
    funcdecl!   ((vm) <sig> add_u128);
    funcdef!    ((vm) <sig> add_u128 VERSION add_u128_v1);

    block!      ((vm, add_u128_v1) blk_entry);
    ssa!        ((vm, add_u128_v1) <u128> a);
    ssa!        ((vm, add_u128_v1) <u128> b);

    // sum = Add %a %b
    ssa!        ((vm, add_u128_v1) <u128> sum);
    inst!       ((vm, add_u128_v1) blk_entry_add_u128:
        sum = BINOP (BinOp::Add) a b
    );

    inst!       ((vm, add_u128_v1) blk_entry_ret:
        RET (sum)
    );

    define_block!   ((vm, add_u128_v1) blk_entry(a, b) {
        blk_entry_add_u128, blk_entry_ret
    });

    define_func_ver!((vm) add_u128_v1 (entry: blk_entry) {blk_entry});
    
    emit_test!      ((vm) (add_u128 add_u128_test1 add_u128_test1_v1 IntEx,IntEx,IntEx,EQ (sig, u128(vec![1, 0]), u128(vec![1, 0]), u128(vec![2, 0]))));
    emit_test!      ((vm) (add_u128 add_u128_test2 add_u128_test2_v1 IntEx,IntEx,IntEx,EQ (sig, u128(vec![u64::MAX, 0]), u128(vec![1, 0]), u128(vec![0, 1]))));
    
    vm
}

#[test]
fn test_sub_u128() {
    build_and_run_test!(sub_u128, sub_u128_test1);
    build_and_run_test!(sub_u128, sub_u128_test2);
}

fn sub_u128() -> VM {
    let vm = VM::new();

    typedef!    ((vm) u128 = mu_int(128));

    funcsig!    ((vm) sig = (u128, u128) -> (u128));
    funcdecl!   ((vm) <sig> sub_u128);
    funcdef!    ((vm) <sig> sub_u128 VERSION sub_u128_v1);

    block!      ((vm, sub_u128_v1) blk_entry);
    ssa!        ((vm, sub_u128_v1) <u128> a);
    ssa!        ((vm, sub_u128_v1) <u128> b);

    // sum = sub %a %b
    ssa!        ((vm, sub_u128_v1) <u128> sum);
    inst!       ((vm, sub_u128_v1) blk_entry_sub_u128:
        sum = BINOP (BinOp::Sub) a b
    );

    inst!       ((vm, sub_u128_v1) blk_entry_ret:
        RET (sum)
    );

    define_block!   ((vm, sub_u128_v1) blk_entry(a, b) {
        blk_entry_sub_u128, blk_entry_ret
    });

    define_func_ver!((vm) sub_u128_v1 (entry: blk_entry) {blk_entry});
    
    emit_test!      ((vm) (sub_u128 sub_u128_test1 sub_u128_test1_v1 IntEx,IntEx,IntEx,EQ (sig, u128(vec![1, 0]), u128(vec![1, 0]), u128(vec![0, 0]))));
    emit_test!      ((vm) (sub_u128 sub_u128_test2 sub_u128_test2_v1 IntEx,IntEx,IntEx,EQ (sig, u128(vec![u64::MAX, 0]), u128(vec![u64::MAX, u64::MAX]), u128(vec![0, 1]))));
    
    vm
}
#[test]
fn test_add_const_u128() {
    build_and_run_test!(add_const_u128, add_const_u128_test1);
    build_and_run_test!(add_const_u128, add_const_u128_test2);
}

fn add_const_u128() -> VM {
    let vm = VM::new();

    typedef!    ((vm) u128 = mu_int(128));

    constdef!   ((vm) <u128> b = Constant::IntEx(vec![1, 0]));

    funcsig!    ((vm) sig = (u128) -> (u128));
    funcdecl!   ((vm) <sig> add_const_u128);
    funcdef!    ((vm) <sig> add_const_u128 VERSION add_const_u128_v1);

    block!      ((vm, add_const_u128_v1) blk_entry);
    ssa!        ((vm, add_const_u128_v1) <u128> a);

    // sum = Add %a %b
    ssa!        ((vm, add_const_u128_v1) <u128> sum);
    consta!     ((vm, add_const_u128_v1) b_local = b);
    inst!       ((vm, add_const_u128_v1) blk_entry_add_const_u128:
        sum = BINOP (BinOp::Add) a b_local
    );

    inst!       ((vm, add_const_u128_v1) blk_entry_ret:
        RET (sum)
    );

    define_block!   ((vm, add_const_u128_v1) blk_entry(a) {
        blk_entry_add_const_u128, blk_entry_ret
    });

    define_func_ver!((vm) add_const_u128_v1 (entry: blk_entry) {blk_entry});
    
    emit_test!      ((vm) (add_const_u128 add_const_u128_test1 add_const_u128_test1_v1 IntEx,IntEx,EQ (sig, u128(vec![1, 0]), u128(vec![2, 0]))));
    emit_test!      ((vm) (add_const_u128 add_const_u128_test2 add_const_u128_test2_v1 IntEx,IntEx,EQ (sig, u128(vec![u64::MAX, 0]), u128(vec![0, 1]))));
    
    vm
}

#[test]
fn test_mul_u128() {
    build_and_run_test!(mul_u128, mul_u128_test1);
    build_and_run_test!(mul_u128, mul_u128_test2);
}

fn mul_u128() -> VM {
    let vm = VM::new();

    typedef!    ((vm) u128 = mu_int(128));

    funcsig!    ((vm) sig = (u128, u128) -> (u128));
    funcdecl!   ((vm) <sig> mul_u128);
    funcdef!    ((vm) <sig> mul_u128 VERSION mul_u128_v1);

    block!      ((vm, mul_u128_v1) blk_entry);
    ssa!        ((vm, mul_u128_v1) <u128> a);
    ssa!        ((vm, mul_u128_v1) <u128> b);

    // sum = Add %a %b
    ssa!        ((vm, mul_u128_v1) <u128> sum);
    inst!       ((vm, mul_u128_v1) blk_entry_mul_u128:
        sum = BINOP (BinOp::Mul) a b
    );

    inst!       ((vm, mul_u128_v1) blk_entry_ret:
        RET (sum)
    );

    define_block!   ((vm, mul_u128_v1) blk_entry(a, b) {
        blk_entry_mul_u128, blk_entry_ret
    });

    define_func_ver!((vm) mul_u128_v1 (entry: blk_entry) {blk_entry});
    
    emit_test!      ((vm) (mul_u128 mul_u128_test1 mul_u128_test1_v1 IntEx,IntEx,IntEx,EQ (sig, u128(vec![6, 0]), u128(vec![7, 0]), u128(vec![42, 0]))));
    emit_test!      ((vm) (mul_u128 mul_u128_test2 mul_u128_test2_v1 IntEx,IntEx,IntEx,EQ (sig, u128(vec![6, 6]), u128(vec![7, 7]), u128(vec![42, 84]))));
    
    vm
}

//#[ignore]   // this test uses runtime function, should run it as bootimage
#[test]
fn test_udiv_u128() {
    build_and_run_test!(udiv_u128, udiv_u128_test1);
    build_and_run_test!(udiv_u128, udiv_u128_test2);
}

fn udiv_u128() -> VM {
    let vm = VM::new();

    typedef!    ((vm) u128 = mu_int(128));

    funcsig!    ((vm) sig = (u128, u128) -> (u128));
    funcdecl!   ((vm) <sig> udiv_u128);
    funcdef!    ((vm) <sig> udiv_u128 VERSION udiv_u128_v1);

    block!      ((vm, udiv_u128_v1) blk_entry);
    ssa!        ((vm, udiv_u128_v1) <u128> a);
    ssa!        ((vm, udiv_u128_v1) <u128> b);

    // sum = Add %a %b
    ssa!        ((vm, udiv_u128_v1) <u128> sum);
    inst!       ((vm, udiv_u128_v1) blk_entry_udiv_u128:
        sum = BINOP (BinOp::Udiv) a b
    );

    inst!       ((vm, udiv_u128_v1) blk_entry_ret:
        RET (sum)
    );

    define_block!   ((vm, udiv_u128_v1) blk_entry(a, b) {
        blk_entry_udiv_u128, blk_entry_ret
    });

    define_func_ver!((vm) udiv_u128_v1 (entry: blk_entry) {blk_entry});
    
    emit_test!      ((vm) (udiv_u128 udiv_u128_test1 udiv_u128_test1_v1 IntEx,IntEx,IntEx,EQ (sig, u128(vec![42, 0]), u128(vec![7, 0]), u128(vec![6, 0]))));
    let a = u128::from_parts(42, 41); // hi, lo
    let b = u128::from_parts(7, 6);
    let expect = a.wrapping_div(b);
    let exp_low = expect.low64();
    let exp_high = expect.high64();
    emit_test!      ((vm) (udiv_u128 udiv_u128_test2 udiv_u128_test2_v1 IntEx,IntEx,IntEx,EQ (sig, u128(vec![42, 41]), u128(vec![7, 6]), u128(vec![exp_low, exp_high]))));
    
    vm
}

#[test]
fn test_shl_u128() {
    build_and_run_test!(shl_u128, shl_u128_test1);
    build_and_run_test!(shl_u128, shl_u128_test2);
}

fn shl_u128() -> VM {
    let vm = VM::new();

    typedef!    ((vm) u128 = mu_int(128));

    funcsig!    ((vm) sig = (u128, u128) -> (u128));
    funcdecl!   ((vm) <sig> shl_u128);
    funcdef!    ((vm) <sig> shl_u128 VERSION shl_u128_v1);

    block!      ((vm, shl_u128_v1) blk_entry);
    ssa!        ((vm, shl_u128_v1) <u128> a);
    ssa!        ((vm, shl_u128_v1) <u128> b);

    // sum = Add %a %b
    ssa!        ((vm, shl_u128_v1) <u128> sum);
    inst!       ((vm, shl_u128_v1) blk_entry_shl_u128:
        sum = BINOP (BinOp::Shl) a b
    );

    inst!       ((vm, shl_u128_v1) blk_entry_ret:
        RET (sum)
    );

    define_block!   ((vm, shl_u128_v1) blk_entry(a, b) {
        blk_entry_shl_u128, blk_entry_ret
    });

    define_func_ver!((vm) shl_u128_v1 (entry: blk_entry) {blk_entry});
    
    emit_test!      ((vm) (shl_u128 shl_u128_test1 shl_u128_test1_v1 IntEx,IntEx,IntEx,EQ (sig, u128(vec![1, 0]), u128(vec![64, 0]), u128(vec![0, 1]))));
    emit_test!      ((vm) (shl_u128 shl_u128_test2 shl_u128_test2_v1 IntEx,IntEx,IntEx,EQ (sig, u128(vec![1, 1]), u128(vec![64, 0]), u128(vec![0, 1]))));
    
    vm
}

#[test]
fn test_lshr_u128() {
    build_and_run_test!(lshr_u128, lshr_u128_test1);
    build_and_run_test!(lshr_u128, lshr_u128_test2);
}

fn lshr_u128() -> VM {
    let vm = VM::new();

    typedef!    ((vm) u128 = mu_int(128));

    funcsig!    ((vm) sig = (u128, u128) -> (u128));
    funcdecl!   ((vm) <sig> lshr_u128);
    funcdef!    ((vm) <sig> lshr_u128 VERSION lshr_u128_v1);

    block!      ((vm, lshr_u128_v1) blk_entry);
    ssa!        ((vm, lshr_u128_v1) <u128> a);
    ssa!        ((vm, lshr_u128_v1) <u128> b);

    // sum = Add %a %b
    ssa!        ((vm, lshr_u128_v1) <u128> sum);
    inst!       ((vm, lshr_u128_v1) blk_entry_lshr_u128:
        sum = BINOP (BinOp::Lshr) a b
    );

    inst!       ((vm, lshr_u128_v1) blk_entry_ret:
        RET (sum)
    );

    define_block!   ((vm, lshr_u128_v1) blk_entry(a, b) {
        blk_entry_lshr_u128, blk_entry_ret
    });

    define_func_ver!((vm) lshr_u128_v1 (entry: blk_entry) {blk_entry});
    
    emit_test!      ((vm) (lshr_u128 lshr_u128_test1 lshr_u128_test1_v1 IntEx,IntEx,IntEx,EQ (sig, u128(vec![1, 1]), u128(vec![64, 0]), u128(vec![1, 0]))));
    emit_test!      ((vm) (lshr_u128 lshr_u128_test2 lshr_u128_test2_v1 IntEx,IntEx,IntEx,EQ (sig, u128(vec![1, 0xffffffffffffffff]), u128(vec![64, 0]), u128(vec![0xffffffffffffffff, 0]))));
    
    vm
}

#[test]
fn test_ashr_u128() {
    build_and_run_test!(ashr_u128, ashr_u128_test1);
}

fn ashr_u128() -> VM {
    let vm = VM::new();

    typedef!    ((vm) u128 = mu_int(128));

    funcsig!    ((vm) sig = (u128, u128) -> (u128));
    funcdecl!   ((vm) <sig> ashr_u128);
    funcdef!    ((vm) <sig> ashr_u128 VERSION ashr_u128_v1);

    block!      ((vm, ashr_u128_v1) blk_entry);
    ssa!        ((vm, ashr_u128_v1) <u128> a);
    ssa!        ((vm, ashr_u128_v1) <u128> b);

    // sum = Add %a %b
    ssa!        ((vm, ashr_u128_v1) <u128> sum);
    inst!       ((vm, ashr_u128_v1) blk_entry_ashr_u128:
        sum = BINOP (BinOp::Ashr) a b
    );

    inst!       ((vm, ashr_u128_v1) blk_entry_ret:
        RET (sum)
    );

    define_block!   ((vm, ashr_u128_v1) blk_entry(a, b) {
        blk_entry_ashr_u128, blk_entry_ret
    });

    define_func_ver!((vm) ashr_u128_v1 (entry: blk_entry) {blk_entry});
    
    emit_test!      ((vm) (ashr_u128 ashr_u128_test1 ashr_u128_test1_v1 IntEx,IntEx,IntEx,EQ (sig, u128(vec![1, 0xffffffffffffffff]), u128(vec![64, 0]), u128(vec![0xffffffffffffffff, 0xffffffffffffffff]))));
    
    vm
}

#[test]
fn test_store_load_u128() {
    let lib = testutil::compile_fnc("store_load_u128", &store_load_u128);

    unsafe {
        use mu::utils::mem::memsec::malloc;
        let ptr = match malloc::<u64>(16) {
            Some(ptr) => ptr,
            None => panic!("failed to alloc memory for testing")
        };

        let store_load_u128 : libloading::Symbol<unsafe extern fn(u64, u64, *mut u64) -> (u64, u64)> = lib.get(b"store_load_u128").unwrap();

        let res = store_load_u128(1, 2, ptr);
        println!("store_load(1, 2, ptr) = {:?}", res);
        assert!(res == (1, 2));
    }
}

fn store_load_u128() -> VM {
    let vm = VM::new();

    typedef!    ((vm) u128 = mu_int(128));
    typedef!    ((vm) uptr_u128 = mu_uptr(u128));

    funcsig!    ((vm) sig = (u128, uptr_u128) -> (u128));
    funcdecl!   ((vm) <sig> store_load_u128);
    funcdef!    ((vm) <sig> store_load_u128 VERSION store_load_u128_v1);

    block!      ((vm, store_load_u128_v1) blk_entry);
    ssa!        ((vm, store_load_u128_v1) <u128> x);
    ssa!        ((vm, store_load_u128_v1) <uptr_u128> ptr);

    // store
    inst!       ((vm, store_load_u128_v1) blk_entry_store:
        STORE ptr x (is_ptr: true, order: MemoryOrder::Relaxed)
    );

    // load
    ssa!        ((vm, store_load_u128_v1) <u128> val);
    inst!       ((vm, store_load_u128_v1) blk_entry_load:
        val = LOAD ptr (is_ptr: true, order: MemoryOrder::Relaxed)
    );

    // ret
    inst!       ((vm, store_load_u128_v1) blk_entry_ret:
        RET (val)
    );

    define_block!((vm, store_load_u128_v1) blk_entry(x, ptr) {
        blk_entry_store,
        blk_entry_load,
        blk_entry_ret
    });

    define_func_ver!((vm) store_load_u128_v1(entry: blk_entry) {
        blk_entry
    });

    vm
}

#[test]
fn test_ugt_u128() {
    build_and_run_test!(ugt_u128, ugt_u128_test1);
    build_and_run_test!(ugt_u128, ugt_u128_test2);
    build_and_run_test!(ugt_u128, ugt_u128_test3);
    build_and_run_test!(ugt_u128, ugt_u128_test4);
    build_and_run_test!(ugt_u128, ugt_u128_test5);
    build_and_run_test!(ugt_u128, ugt_u128_test6);
}

fn ugt_u128() -> VM {
    let vm = VM::new();

    typedef!    ((vm) u128 = mu_int(128));
    typedef!    ((vm) u64  = mu_int(64));
    typedef!    ((vm) u1   = mu_int(1));

    constdef!   ((vm) <u64> u64_0 = Constant::Int(0));
    constdef!   ((vm) <u64> u64_1 = Constant::Int(1));

    funcsig!    ((vm) sig = (u128, u128) -> (u64));
    funcdecl!   ((vm) <sig> ugt_u128);
    funcdef!    ((vm) <sig> ugt_u128 VERSION ugt_u128_v1);

    // blk entry
    block!      ((vm, ugt_u128_v1) blk_entry);
    ssa!        ((vm, ugt_u128_v1) <u128> a);
    ssa!        ((vm, ugt_u128_v1) <u128> b);

    // cond = UGT a b
    ssa!        ((vm, ugt_u128_v1) <u1> cond);
    inst!       ((vm, ugt_u128_v1) blk_entry_ugt:
        cond = CMPOP (CmpOp::UGT) a b
    );

    // BRANCH2 cond (blk_ret: 1) (blk_ret: 0)
    block!      ((vm, ugt_u128_v1) blk_ret);
    consta!     ((vm, ugt_u128_v1) u64_0_local = u64_0);
    consta!     ((vm, ugt_u128_v1) u64_1_local = u64_1);
    inst!       ((vm, ugt_u128_v1) blk_entry_branch2:
        BRANCH2 (cond, u64_1_local, u64_0_local)
            IF (OP 0)
            THEN blk_ret (vec![1]) WITH 0.5f32,
            ELSE blk_ret (vec![2])
    );

    define_block!((vm, ugt_u128_v1) blk_entry(a, b) {
        blk_entry_ugt, blk_entry_branch2
    });

    // blk ret (res)
    ssa!        ((vm, ugt_u128_v1) <u64> res);
    // RET res
    inst!       ((vm, ugt_u128_v1) blk_ret_ret:
        RET (res)
    );

    define_block!((vm, ugt_u128_v1) blk_ret(res) {
        blk_ret_ret
    });

    define_func_ver!((vm) ugt_u128_v1(entry: blk_entry) {
        blk_entry, blk_ret
    });
    
    emit_test!      ((vm) (ashr_u128 ashr_u128_test1 ashr_u128_test1_v1 IntEx,IntEx,Int,EQ (sig, u128(vec![1, 0]), u128(vec![2, 0]), u64(0u64))));
    emit_test!      ((vm) (ashr_u128 ashr_u128_test1 ashr_u128_test1_v1 IntEx,IntEx,Int,EQ (sig, u128(vec![1, 0]), u128(vec![1, 0]), u64(0u64))));
    emit_test!      ((vm) (ashr_u128 ashr_u128_test1 ashr_u128_test1_v1 IntEx,IntEx,Int,EQ (sig, u128(vec![1, 0]), u128(vec![0, 0]), u64(1u64))));
    emit_test!      ((vm) (ashr_u128 ashr_u128_test1 ashr_u128_test1_v1 IntEx,IntEx,Int,EQ (sig, u128(vec![1, 0xffffffffffffffff]), u128(vec![2, 0xffffffffffffffff]), u64(0u64))));
    emit_test!      ((vm) (ashr_u128 ashr_u128_test1 ashr_u128_test1_v1 IntEx,IntEx,Int,EQ (sig, u128(vec![1, 0xffffffffffffffff]), u128(vec![1, 0xffffffffffffffff]), u64(0u64))));
    emit_test!      ((vm) (ashr_u128 ashr_u128_test1 ashr_u128_test1_v1 IntEx,IntEx,Int,EQ (sig, u128(vec![1, 0xffffffffffffffff]), u128(vec![0, 0xffffffffffffffff]), u64(1u64))));
    
    vm
}

#[test]
fn test_sgt_i128() {
    build_and_run_test!(sgt_i128, sgt_i128_test1);
    build_and_run_test!(sgt_i128, sgt_i128_test2);
    build_and_run_test!(sgt_i128, sgt_i128_test3);
    build_and_run_test!(sgt_i128, sgt_i128_test4);
    build_and_run_test!(sgt_i128, sgt_i128_test5);
}

fn sgt_i128() -> VM {
    let vm = VM::new();

    typedef!    ((vm) i128 = mu_int(128));
    typedef!    ((vm) u64  = mu_int(64));
    typedef!    ((vm) u1   = mu_int(1));

    constdef!   ((vm) <u64> u64_0 = Constant::Int(0));
    constdef!   ((vm) <u64> u64_1 = Constant::Int(1));

    funcsig!    ((vm) sig = (i128, i128) -> (u64));
    funcdecl!   ((vm) <sig> sgt_i128);
    funcdef!    ((vm) <sig> sgt_i128 VERSION sgt_i128_v1);

    // blk entry
    block!      ((vm, sgt_i128_v1) blk_entry);
    ssa!        ((vm, sgt_i128_v1) <i128> a);
    ssa!        ((vm, sgt_i128_v1) <i128> b);

    // cond = UGT a b
    ssa!        ((vm, sgt_i128_v1) <u1> cond);
    inst!       ((vm, sgt_i128_v1) blk_entry_ugt:
        cond = CMPOP (CmpOp::SGT) a b
    );

    // BRANCH2 cond (blk_ret: 1) (blk_ret: 0)
    block!      ((vm, sgt_i128_v1) blk_ret);
    consta!     ((vm, sgt_i128_v1) u64_0_local = u64_0);
    consta!     ((vm, sgt_i128_v1) u64_1_local = u64_1);
    inst!       ((vm, sgt_i128_v1) blk_entry_branch2:
        BRANCH2 (cond, u64_1_local, u64_0_local)
            IF (OP 0)
            THEN blk_ret (vec![1]) WITH 0.5f32,
            ELSE blk_ret (vec![2])
    );

    define_block!((vm, sgt_i128_v1) blk_entry(a, b) {
        blk_entry_ugt, blk_entry_branch2
    });

    // blk ret (res)
    ssa!        ((vm, sgt_i128_v1) <u64> res);
    // RET res
    inst!       ((vm, sgt_i128_v1) blk_ret_ret:
        RET (res)
    );

    define_block!((vm, sgt_i128_v1) blk_ret(res) {
        blk_ret_ret
    });

    define_func_ver!((vm) sgt_i128_v1(entry: blk_entry) {
        blk_entry, blk_ret
    });
    
    emit_test!      ((vm) (sgt_i128 sgt_i128_test1 sgt_i128_test1_v1 IntEx,IntEx,Int,EQ (sig, i128(vec![1, 0]), i128(vec![2, 0]), u64(0u64))));
    emit_test!      ((vm) (sgt_i128 sgt_i128_test2 sgt_i128_test2_v1 IntEx,IntEx,Int,EQ (sig, i128(vec![1, 0]), i128(vec![1, 0]), u64(0u64))));
    emit_test!      ((vm) (sgt_i128 sgt_i128_test3 sgt_i128_test3_v1 IntEx,IntEx,Int,EQ (sig, i128(vec![1, 0]), i128(vec![0, 0]), u64(1u64))));
    emit_test!      ((vm) (sgt_i128 sgt_i128_test4 sgt_i128_test4_v1 IntEx,IntEx,Int,EQ (sig, i128(vec![0xffffffffffffffff, 0xffffffffffffffff]), i128(vec![1, 0]), u64(0u64))));
    emit_test!      ((vm) (sgt_i128 sgt_i128_test5 sgt_i128_test5_v1 IntEx,IntEx,Int,EQ (sig, i128(vec![0xffffffffffffffff, 0xffffffffffffffff]), i128(vec![0xfffffffffffffffe, 0xffffffffffffffff]), u64(1u64))));
    
    vm
}

#[test]
fn test_ult_u128() {
    build_and_run_test!(ult_u128, ult_u128_test1);
    build_and_run_test!(ult_u128, ult_u128_test2);
    build_and_run_test!(ult_u128, ult_u128_test3);
    build_and_run_test!(ult_u128, ult_u128_test4);
    build_and_run_test!(ult_u128, ult_u128_test5);
    build_and_run_test!(ult_u128, ult_u128_test6);
}

fn ult_u128() -> VM {
    let vm = VM::new();

    typedef!    ((vm) u128 = mu_int(128));
    typedef!    ((vm) u64  = mu_int(64));
    typedef!    ((vm) u1   = mu_int(1));

    constdef!   ((vm) <u64> u64_0 = Constant::Int(0));
    constdef!   ((vm) <u64> u64_1 = Constant::Int(1));

    funcsig!    ((vm) sig = (u128, u128) -> (u64));
    funcdecl!   ((vm) <sig> ult_u128);
    funcdef!    ((vm) <sig> ult_u128 VERSION ult_u128_v1);

    // blk entry
    block!      ((vm, ult_u128_v1) blk_entry);
    ssa!        ((vm, ult_u128_v1) <u128> a);
    ssa!        ((vm, ult_u128_v1) <u128> b);

    // cond = UGT a b
    ssa!        ((vm, ult_u128_v1) <u1> cond);
    inst!       ((vm, ult_u128_v1) blk_entry_ugt:
        cond = CMPOP (CmpOp::ULT) a b
    );

    // BRANCH2 cond (blk_ret: 1) (blk_ret: 0)
    block!      ((vm, ult_u128_v1) blk_ret);
    consta!     ((vm, ult_u128_v1) u64_0_local = u64_0);
    consta!     ((vm, ult_u128_v1) u64_1_local = u64_1);
    inst!       ((vm, ult_u128_v1) blk_entry_branch2:
        BRANCH2 (cond, u64_1_local, u64_0_local)
            IF (OP 0)
            THEN blk_ret (vec![1]) WITH 0.5f32,
            ELSE blk_ret (vec![2])
    );

    define_block!((vm, ult_u128_v1) blk_entry(a, b) {
        blk_entry_ugt, blk_entry_branch2
    });

    // blk ret (res)
    ssa!        ((vm, ult_u128_v1) <u64> res);
    // RET res
    inst!       ((vm, ult_u128_v1) blk_ret_ret:
        RET (res)
    );

    define_block!((vm, ult_u128_v1) blk_ret(res) {
        blk_ret_ret
    });

    define_func_ver!((vm) ult_u128_v1(entry: blk_entry) {
        blk_entry, blk_ret
    });
    
    emit_test!      ((vm) (ult_u128 ult_u128_test1 ult_u128_test1_v1 IntEx,IntEx,Int,EQ (sig, u128(vec![1, 0]), u128(vec![2, 0]), u64(1u64))));
    emit_test!      ((vm) (ult_u128 ult_u128_test2 ult_u128_test2_v1 IntEx,IntEx,Int,EQ (sig, u128(vec![1, 0]), u128(vec![1, 0]), u64(0u64))));
    emit_test!      ((vm) (ult_u128 ult_u128_test3 ult_u128_test3_v1 IntEx,IntEx,Int,EQ (sig, u128(vec![1, 0]), u128(vec![0, 0]), u64(0u64))));
    emit_test!      ((vm) (ult_u128 ult_u128_test4 ult_u128_test4_v1 IntEx,IntEx,Int,EQ (sig, u128(vec![1, 0xffffffffffffffff]), u128(vec![2, 0xffffffffffffffff]), u64(1u64))));
    emit_test!      ((vm) (ult_u128 ult_u128_test5 ult_u128_test5_v1 IntEx,IntEx,Int,EQ (sig, u128(vec![1, 0xffffffffffffffff]), u128(vec![1, 0xffffffffffffffff]), u64(0u64))));
    emit_test!      ((vm) (ult_u128 ult_u128_test6 ult_u128_test6_v1 IntEx,IntEx,Int,EQ (sig, u128(vec![1, 0xffffffffffffffff]), u128(vec![0, 0xffffffffffffffff]), u64(0u64))));
    
    vm
}

#[test]
fn test_slt_i128() {
    build_and_run_test!(slt_i128, slt_i128_test1);
    build_and_run_test!(slt_i128, slt_i128_test2);
    build_and_run_test!(slt_i128, slt_i128_test3);
    build_and_run_test!(slt_i128, slt_i128_test4);
    build_and_run_test!(slt_i128, slt_i128_test5);
}

fn slt_i128() -> VM {
    let vm = VM::new();

    typedef!    ((vm) i128 = mu_int(128));
    typedef!    ((vm) u64  = mu_int(64));
    typedef!    ((vm) u1   = mu_int(1));

    constdef!   ((vm) <u64> u64_0 = Constant::Int(0));
    constdef!   ((vm) <u64> u64_1 = Constant::Int(1));

    funcsig!    ((vm) sig = (i128, i128) -> (u64));
    funcdecl!   ((vm) <sig> slt_i128);
    funcdef!    ((vm) <sig> slt_i128 VERSION slt_i128_v1);

    // blk entry
    block!      ((vm, slt_i128_v1) blk_entry);
    ssa!        ((vm, slt_i128_v1) <i128> a);
    ssa!        ((vm, slt_i128_v1) <i128> b);

    // cond = UGT a b
    ssa!        ((vm, slt_i128_v1) <u1> cond);
    inst!       ((vm, slt_i128_v1) blk_entry_ugt:
        cond = CMPOP (CmpOp::SLT) a b
    );

    // BRANCH2 cond (blk_ret: 1) (blk_ret: 0)
    block!      ((vm, slt_i128_v1) blk_ret);
    consta!     ((vm, slt_i128_v1) u64_0_local = u64_0);
    consta!     ((vm, slt_i128_v1) u64_1_local = u64_1);
    inst!       ((vm, slt_i128_v1) blk_entry_branch2:
        BRANCH2 (cond, u64_1_local, u64_0_local)
            IF (OP 0)
            THEN blk_ret (vec![1]) WITH 0.5f32,
            ELSE blk_ret (vec![2])
    );

    define_block!((vm, slt_i128_v1) blk_entry(a, b) {
        blk_entry_ugt, blk_entry_branch2
    });

    // blk ret (res)
    ssa!        ((vm, slt_i128_v1) <u64> res);
    // RET res
    inst!       ((vm, slt_i128_v1) blk_ret_ret:
        RET (res)
    );

    define_block!((vm, slt_i128_v1) blk_ret(res) {
        blk_ret_ret
    });

    define_func_ver!((vm) slt_i128_v1(entry: blk_entry) {
        blk_entry, blk_ret
    });
    
    emit_test!      ((vm) (slt_i128 slt_i128_test1 slt_i128_test1_v1 IntEx,IntEx,Int,EQ (sig, i128(vec![1, 0]), i128(vec![2, 0]), u64(1u64))));
    emit_test!      ((vm) (slt_i128 slt_i128_test2 slt_i128_test2_v1 IntEx,IntEx,Int,EQ (sig, i128(vec![1, 0]), i128(vec![1, 0]), u64(0u64))));
    emit_test!      ((vm) (slt_i128 slt_i128_test3 slt_i128_test3_v1 IntEx,IntEx,Int,EQ (sig, i128(vec![1, 0]), i128(vec![0, 0]), u64(0u64))));
    emit_test!      ((vm) (slt_i128 slt_i128_test4 slt_i128_test4_v1 IntEx,IntEx,Int,EQ (sig, i128(vec![0xffffffffffffffff, 0xffffffffffffffff]), i128(vec![1, 0]), u64(1u64))));
    emit_test!      ((vm) (slt_i128 slt_i128_test5 slt_i128_test5_v1 IntEx,IntEx,Int,EQ (sig, i128(vec![0xffffffffffffffff, 0xffffffffffffffff]), i128(vec![0xfffffffffffffffe, 0xffffffffffffffff]), u64(0u64))));
    
    vm
}