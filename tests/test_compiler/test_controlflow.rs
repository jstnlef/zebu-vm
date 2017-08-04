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
use mu::linkutils;
use mu::utils::LinkedHashMap;

use std::sync::Arc;
use mu::linkutils::aot;
use mu::runtime::thread::check_result;
use mu::compiler::*;

#[test]
fn test_switch() {
    build_and_run_test!(switch, switch_test1);
    build_and_run_test!(switch, switch_test2);
    build_and_run_test!(switch, switch_test3);
    build_and_run_test!(switch, switch_test4);
}

fn switch() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int64 = mu_int(64));

    constdef!   ((vm) <int64> int64_0  = Constant::Int(0));
    constdef!   ((vm) <int64> int64_1  = Constant::Int(1));
    constdef!   ((vm) <int64> int64_2  = Constant::Int(2));
    constdef!   ((vm) <int64> int64_99 = Constant::Int(99));

    funcsig!    ((vm) switch_sig = (int64) -> (int64));
    funcdecl!   ((vm) <switch_sig> switch);
    funcdef!    ((vm) <switch_sig> switch VERSION switch_v1);

    // %entry(<@int64> %a):
    block!      ((vm, switch_v1) blk_entry);
    ssa!        ((vm, switch_v1) <int64> a);

    // SWITCH %a %blk_default (0 -> %blk_ret0, 1 -> %blk_ret1, 2 -> %blk_ret2)
    block!      ((vm, switch_v1) blk_ret0);
    block!      ((vm, switch_v1) blk_ret1);
    block!      ((vm, switch_v1) blk_ret2);
    block!      ((vm, switch_v1) blk_default);

    consta!     ((vm, switch_v1) int64_0_local = int64_0);
    consta!     ((vm, switch_v1) int64_1_local = int64_1);
    consta!     ((vm, switch_v1) int64_2_local = int64_2);

    let blk_entry_switch = switch_v1.new_inst(Instruction {
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: None,
        ops: vec![
            a.clone(), // 0
            int64_0_local.clone(), // 1
            int64_1_local.clone(), // 2
            int64_2_local.clone(), // 3
        ],
        v: Instruction_::Switch {
            cond: 0,
            default: Destination {
                target: blk_default.id(),
                args: vec![]
            },
            branches: vec![
                (1, Destination{target: blk_ret0.id(), args: vec![]}),
                (2, Destination{target: blk_ret1.id(), args: vec![]}),
                (3, Destination{target: blk_ret2.id(), args: vec![]})
            ]
        }
    });

    define_block!((vm, switch_v1) blk_entry(a) {
        blk_entry_switch
    });

    // blk_default
    consta!     ((vm, switch_v1) int64_99_local = int64_99);
    inst!       ((vm, switch_v1) blk_default_ret:
        RET (int64_99_local)
    );

    define_block!((vm, switch_v1) blk_default() {
        blk_default_ret
    });

    // blk_ret0
    inst!       ((vm, switch_v1) blk_ret0_ret:
        RET (int64_0_local)
    );

    define_block!((vm, switch_v1) blk_ret0() {
        blk_ret0_ret
    });

    // blk_ret1
    inst!       ((vm, switch_v1) blk_ret1_ret:
        RET (int64_1_local)
    );

    define_block!((vm, switch_v1) blk_ret1() {
        blk_ret1_ret
    });

    // blk_ret2
    inst!       ((vm, switch_v1) blk_ret2_ret:
        RET (int64_2_local)
    );

    define_block!((vm, switch_v1) blk_ret2() {
        blk_ret2_ret
    });

    define_func_ver!((vm) switch_v1 (entry: blk_entry) {
        blk_entry,
        blk_default,
        blk_ret0,
        blk_ret1,
        blk_ret2
    });
    
    emit_test! ((vm)
        switch, switch_test1, switch_test1_v1,
        Int RET Int,
        EQ,
        switch_sig,
        int64(0u64) RET int64(0u64),
    );
    emit_test! ((vm)
        switch, switch_test2, switch_test2_v1,
        Int RET Int,
        EQ,
        switch_sig,
        int64(1u64) RET int64(1u64),
    );
    emit_test! ((vm)
        switch, switch_test3, switch_test3_v1,
        Int RET Int,
        EQ,
        switch_sig,
        int64(2u64) RET int64(2u64),
    );
    emit_test! ((vm)
        switch, switch_test4, switch_test4_v1,
        Int RET Int,
        EQ,
        switch_sig,
        int64(3u64) RET int64(99u64),
    );
    
    vm
}

#[test]
fn test_select_eq_zero() {
    build_and_run_test!(select_eq_zero, select_eq_zero_test1);
    build_and_run_test!(select_eq_zero, select_eq_zero_test2);
}

fn select_eq_zero() -> VM {
    let vm = VM::new();

    typedef! ((vm) int64 = mu_int(64));
    typedef! ((vm) int1  = mu_int(1));
    constdef!((vm) <int64> int64_0 = Constant::Int(0));
    constdef!((vm) <int64> int64_1 = Constant::Int(1));

    funcsig! ((vm) sig = (int64) -> (int64));
    funcdecl!((vm) <sig> select_eq_zero);
    funcdef! ((vm) <sig> select_eq_zero VERSION select_v1);

    // blk entry
    block! ((vm, select_v1) blk_entry);
    ssa!   ((vm, select_v1) <int64> blk_entry_n);

    ssa!   ((vm, select_v1) <int1> blk_entry_cond);
    consta!((vm, select_v1) int64_0_local = int64_0);
    consta!((vm, select_v1) int64_1_local = int64_1);
    inst!  ((vm, select_v1) blk_entry_inst_cmp:
        blk_entry_cond = CMPOP (CmpOp::EQ) blk_entry_n int64_0_local
    );

    ssa!   ((vm, select_v1) <int64> blk_entry_ret);
    inst!  ((vm, select_v1) blk_entry_inst_select:
        blk_entry_ret = SELECT blk_entry_cond int64_1_local int64_0_local
    );

    inst!  ((vm, select_v1) blk_entry_inst_ret:
        RET (blk_entry_ret)
    );

    define_block!   ((vm, select_v1) blk_entry(blk_entry_n){
        blk_entry_inst_cmp, blk_entry_inst_select, blk_entry_inst_ret
    });

    define_func_ver!((vm) select_v1 (entry: blk_entry) {blk_entry});
    
    emit_test! ((vm)
        select_eq_zero, select_eq_zero_test1, select_eq_zero_test1_v1,
        Int RET Int,
        EQ,
        sig,
        int64(0u64) RET int64(1u64),
    );
    emit_test! ((vm)
        select_eq_zero, select_eq_zero_test2, select_eq_zero_test2_v1,
        Int RET Int,
        EQ,
        sig,
        int64(1u64) RET int64(0u64),
    );
    
    vm
}

#[test]
fn test_select_eq_zero_double() {
    build_and_run_test!(select_eq_zero_double, select_eq_zero_double_test1);
    build_and_run_test!(select_eq_zero_double, select_eq_zero_double_test2);
}

fn select_eq_zero_double() -> VM {
    let vm = VM::new();

    typedef! ((vm) int64  = mu_int(64));
    typedef! ((vm) int1   = mu_int(1));
    typedef! ((vm) double = mu_double);
    constdef!((vm) <int64>  int64_0  = Constant::Int(0));
    constdef!((vm) <double> double_0 = Constant::Double(0f64));
    constdef!((vm) <double> double_1 = Constant::Double(1f64));

    funcsig! ((vm) sig = (int64) -> (double));
    funcdecl!((vm) <sig> select_eq_zero_double);
    funcdef! ((vm) <sig> select_eq_zero_double VERSION select_double_v1);

    // blk entry
    block! ((vm, select_double_v1) blk_entry);
    ssa!   ((vm, select_double_v1) <int64> blk_entry_n);

    ssa!   ((vm, select_double_v1) <int1> blk_entry_cond);
    consta!((vm, select_double_v1) int64_0_local = int64_0);
    inst!  ((vm, select_double_v1) blk_entry_inst_cmp:
        blk_entry_cond = CMPOP (CmpOp::EQ) blk_entry_n int64_0_local
    );

    ssa!   ((vm, select_double_v1) <double> blk_entry_ret);
    consta!((vm, select_double_v1) double_0_local = double_0);
    consta!((vm, select_double_v1) double_1_local = double_1);
    inst!  ((vm, select_double_v1) blk_entry_inst_select:
        blk_entry_ret = SELECT blk_entry_cond double_1_local double_0_local
    );

    inst!  ((vm, select_double_v1) blk_entry_inst_ret:
        RET (blk_entry_ret)
    );

    define_block!   ((vm, select_double_v1) blk_entry(blk_entry_n){
        blk_entry_inst_cmp, blk_entry_inst_select, blk_entry_inst_ret
    });

    define_func_ver!((vm) select_double_v1 (entry: blk_entry) {blk_entry});
    
    emit_test! ((vm)
        select_eq_zero_double, select_eq_zero_double_test1, select_eq_zero_double_test1_v1,
        Int RET Double,
        FOEQ,
        sig,
        int64(0u64) RET double(1f64),
    );
    emit_test! ((vm)
        select_eq_zero_double, select_eq_zero_double_test2, select_eq_zero_double_test2_v1,
        Int RET Double,
        FOEQ,
        sig,
        int64(1u64) RET double(0f64),
    );
    
    vm
}

#[test]
fn test_select_u8_eq_zero() {
    build_and_run_test!(select_u8_eq_zero, select_u8_eq_zero_test1);
    build_and_run_test!(select_u8_eq_zero, select_u8_eq_zero_test2);
}

fn select_u8_eq_zero() -> VM {
    let vm = VM::new();

    typedef! ((vm) int8 = mu_int(8));
    typedef! ((vm) int1  = mu_int(1));
    constdef!((vm) <int8> int8_0 = Constant::Int(0));
    constdef!((vm) <int8> int8_1 = Constant::Int(1));

    funcsig! ((vm) sig = (int8) -> (int8));
    funcdecl!((vm) <sig> select_u8_eq_zero);
    funcdef! ((vm) <sig> select_u8_eq_zero VERSION select_u8_eq_zero_v1);

    // blk entry
    block! ((vm, select_u8_eq_zero_v1) blk_entry);
    ssa!   ((vm, select_u8_eq_zero_v1) <int8> blk_entry_n);

    ssa!   ((vm, select_u8_eq_zero_v1) <int1> blk_entry_cond);
    consta!((vm, select_u8_eq_zero_v1) int8_0_local = int8_0);
    consta!((vm, select_u8_eq_zero_v1) int8_1_local = int8_1);
    inst!  ((vm, select_u8_eq_zero_v1) blk_entry_inst_cmp:
        blk_entry_cond = CMPOP (CmpOp::EQ) blk_entry_n int8_0_local
    );

    ssa!   ((vm, select_u8_eq_zero_v1) <int8> blk_entry_ret);
    inst!  ((vm, select_u8_eq_zero_v1) blk_entry_inst_select:
        blk_entry_ret = SELECT blk_entry_cond int8_1_local int8_0_local
    );

    inst!  ((vm, select_u8_eq_zero_v1) blk_entry_inst_ret:
        RET (blk_entry_ret)
    );

    define_block!   ((vm, select_u8_eq_zero_v1) blk_entry(blk_entry_n){
        blk_entry_inst_cmp, blk_entry_inst_select, blk_entry_inst_ret
    });

    define_func_ver!((vm) select_u8_eq_zero_v1 (entry: blk_entry) {blk_entry});
    
    emit_test! ((vm)
        select_u8_eq_zero, select_u8_eq_zero_test1, select_u8_eq_zero_test1_v1,
        Int RET Int,
        EQ,
        sig,
        int8(0u64) RET int8(1u64),
    );
    emit_test! ((vm)
        select_u8_eq_zero, select_u8_eq_zero_test2, select_u8_eq_zero_test2_v1,
        Int RET Int,
        EQ,
        sig,
        int8(1u64) RET int8(0u64),
    );
    
    vm
}

#[test]
fn test_select_sge_zero() {
    build_and_run_test!(select_sge_zero, select_sge_zero_test1);
    build_and_run_test!(select_sge_zero, select_sge_zero_test2);
    build_and_run_test!(select_sge_zero, select_sge_zero_test3);
}

fn select_sge_zero() -> VM {
    let vm = VM::new();

    typedef! ((vm) int64 = mu_int(64));
    typedef! ((vm) int1  = mu_int(1));
    constdef!((vm) <int64> int64_0 = Constant::Int(0));
    constdef!((vm) <int64> int64_1 = Constant::Int(1));

    funcsig! ((vm) sig = (int64) -> (int64));
    funcdecl!((vm) <sig> select_sge_zero);
    funcdef! ((vm) <sig> select_sge_zero VERSION select_v1);

    // blk entry
    block! ((vm, select_v1) blk_entry);
    ssa!   ((vm, select_v1) <int64> blk_entry_n);

    ssa!   ((vm, select_v1) <int1> blk_entry_cond);
    consta!((vm, select_v1) int64_0_local = int64_0);
    consta!((vm, select_v1) int64_1_local = int64_1);
    inst!  ((vm, select_v1) blk_entry_inst_cmp:
        blk_entry_cond = CMPOP (CmpOp::SGE) blk_entry_n int64_0_local
    );

    ssa!   ((vm, select_v1) <int64> blk_entry_ret);
    inst!  ((vm, select_v1) blk_entry_inst_select:
        blk_entry_ret = SELECT blk_entry_cond int64_1_local int64_0_local
    );

    inst!  ((vm, select_v1) blk_entry_inst_ret:
        RET (blk_entry_ret)
    );

    define_block!   ((vm, select_v1) blk_entry(blk_entry_n){
        blk_entry_inst_cmp, blk_entry_inst_select, blk_entry_inst_ret
    });

    define_func_ver!((vm) select_v1 (entry: blk_entry) {blk_entry});
    
    emit_test! ((vm)
        select_sge_zero, select_sge_zero_test1, select_sge_zero_test1_v1,
        Int RET Int,
        EQ,
        sig,
        int64(0u64) RET int64(1u64),
    );
    emit_test! ((vm)
        select_sge_zero, select_sge_zero_test2, select_sge_zero_test2_v1,
        Int RET Int,
        EQ,
        sig,
        int64(1u64) RET int64(1u64),
    );
    emit_test! ((vm)
        select_sge_zero, select_sge_zero_test3, select_sge_zero_test3_v1,
        Int RET Int,
        EQ,
        sig,
        int64(-1i64 as u64) RET int64(0u64),
    );
    
    vm
}

#[test]
fn test_sgt_value() {
    build_and_run_test!(sgt_value, sgt_value_test1);
    build_and_run_test!(sgt_value, sgt_value_test2);
    build_and_run_test!(sgt_value, sgt_value_test3);
}

fn sgt_value() -> VM {
    let vm = VM::new();

    typedef! ((vm) int64 = mu_int(64));
    typedef! ((vm) int1  = mu_int(1));
    constdef!((vm) <int64> int64_0 = Constant::Int(0));
    constdef!((vm) <int64> int64_1 = Constant::Int(1));

    funcsig! ((vm) sig = (int64, int64) -> (int1));
    funcdecl!((vm) <sig> sgt_value);
    funcdef! ((vm) <sig> sgt_value VERSION sgt_value_v1);

    // blk entry
    block! ((vm, sgt_value_v1) blk_entry);
    ssa!   ((vm, sgt_value_v1) <int64> blk_entry_op1);
    ssa!   ((vm, sgt_value_v1) <int64> blk_entry_op2);

    ssa!   ((vm, sgt_value_v1) <int1> blk_entry_cond);
    inst!  ((vm, sgt_value_v1) blk_entry_inst_cmp:
        blk_entry_cond = CMPOP (CmpOp::SGT) blk_entry_op1 blk_entry_op2
    );

    inst!  ((vm, sgt_value_v1) blk_entry_inst_ret:
        RET (blk_entry_cond)
    );

    define_block!   ((vm, sgt_value_v1) blk_entry(blk_entry_op1, blk_entry_op2){
        blk_entry_inst_cmp, blk_entry_inst_ret
    });

    define_func_ver!((vm) sgt_value_v1 (entry: blk_entry) {blk_entry});
    
    emit_test! ((vm)
        sgt_value, sgt_value_test1, sgt_value_test1_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int64(255u64), int64(0u64) RET int1(1u64),
    );
    emit_test! ((vm)
        sgt_value, sgt_value_test2, sgt_value_test2_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int64(255u64), int64(255u64) RET int1(0u64),
    );
    emit_test! ((vm)
        sgt_value, sgt_value_test3, sgt_value_test3_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int64(0u64), int64(255u64) RET int1(0u64),
    );
    
    vm
}

#[test]
fn test_sgt_u8_value() {
    build_and_run_test!(sgt_u8_value, sgt_u8_value_test1);
    build_and_run_test!(sgt_u8_value, sgt_u8_value_test2);
    build_and_run_test!(sgt_u8_value, sgt_u8_value_test3);
    build_and_run_test!(sgt_u8_value, sgt_u8_value_test4);
    build_and_run_test!(sgt_u8_value, sgt_u8_value_test5);
    build_and_run_test!(sgt_u8_value, sgt_u8_value_test6);
}

fn sgt_u8_value() -> VM {
    let vm = VM::new();

    typedef! ((vm) int8  = mu_int(8));
    typedef! ((vm) int1  = mu_int(1));
    constdef!((vm) <int8> int8_0 = Constant::Int(0));
    constdef!((vm) <int8> int8_1 = Constant::Int(1));

    funcsig! ((vm) sig = (int8, int8) -> (int1));
    funcdecl!((vm) <sig> sgt_u8_value);
    funcdef! ((vm) <sig> sgt_u8_value VERSION sgt_u8_value_v1);

    // blk entry
    block! ((vm, sgt_u8_value_v1) blk_entry);
    ssa!   ((vm, sgt_u8_value_v1) <int8> blk_entry_op1);
    ssa!   ((vm, sgt_u8_value_v1) <int8> blk_entry_op2);

    ssa!   ((vm, sgt_u8_value_v1) <int1> blk_entry_cond);
    inst!  ((vm, sgt_u8_value_v1) blk_entry_inst_cmp:
        blk_entry_cond = CMPOP (CmpOp::SGT) blk_entry_op1 blk_entry_op2
    );

    inst!  ((vm, sgt_u8_value_v1) blk_entry_inst_ret:
        RET (blk_entry_cond)
    );

    define_block!   ((vm, sgt_u8_value_v1) blk_entry(blk_entry_op1, blk_entry_op2){
        blk_entry_inst_cmp, blk_entry_inst_ret
    });

    define_func_ver!((vm) sgt_u8_value_v1 (entry: blk_entry) {blk_entry});
    
    emit_test! ((vm)
        sgt_u8_value, sgt_u8_value_test1, sgt_u8_value_test1_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int8(-1i8 as u64), int8(0i8 as u64) RET int1(0u64),
    );
    emit_test! ((vm)
        sgt_u8_value, sgt_u8_value_test2, sgt_u8_value_test2_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int8(0i8 as u64), int8(-1i8 as u64) RET int1(1u64),
    );
    emit_test! ((vm)
        sgt_u8_value, sgt_u8_value_test3, sgt_u8_value_test3_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int8(2i8 as u64), int8(1i8 as u64) RET int1(1u64),
    );
    emit_test! ((vm)
        sgt_u8_value, sgt_u8_value_test4, sgt_u8_value_test4_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int8(1i8 as u64), int8(2i8 as u64) RET int1(0u64),
    );
    emit_test! ((vm)
        sgt_u8_value, sgt_u8_value_test5, sgt_u8_value_test5_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int8(-2i8 as u64), int8(-1i8 as u64) RET int1(0u64),
    );
    emit_test! ((vm)
        sgt_u8_value, sgt_u8_value_test6, sgt_u8_value_test6_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int8(-1i8 as u64), int8(-2i8 as u64) RET int1(1u64),
    );
    
    vm
}

#[test]
fn test_sgt_i32_branch() {
    build_and_run_test!(sgt_i32_branch, sgt_i32_branch_test1);
    build_and_run_test!(sgt_i32_branch, sgt_i32_branch_test2);
    build_and_run_test!(sgt_i32_branch, sgt_i32_branch_test3);
    build_and_run_test!(sgt_i32_branch, sgt_i32_branch_test4);
    build_and_run_test!(sgt_i32_branch, sgt_i32_branch_test5);
    build_and_run_test!(sgt_i32_branch, sgt_i32_branch_test6);
    build_and_run_test!(sgt_i32_branch, sgt_i32_branch_test7);
    build_and_run_test!(sgt_i32_branch, sgt_i32_branch_test8);
    build_and_run_test!(sgt_i32_branch, sgt_i32_branch_test9);
}

fn sgt_i32_branch() -> VM {
    let vm = VM::new();

    typedef! ((vm) int32 = mu_int(32));
    typedef! ((vm) int1  = mu_int(1));

    constdef!((vm) <int32> int32_0 = Constant::Int(0));
    constdef!((vm) <int32> int32_1 = Constant::Int(1));

    funcsig! ((vm) sig = (int32, int32) -> (int32));
    funcdecl!((vm) <sig> sgt_i32_branch);
    funcdef! ((vm) <sig> sgt_i32_branch VERSION sgt_i32_branch_v1);

    // blk entry
    block!   ((vm, sgt_i32_branch_v1) blk_entry);
    ssa!     ((vm, sgt_i32_branch_v1) <int32> blk_entry_a);
    ssa!     ((vm, sgt_i32_branch_v1) <int32> blk_entry_b);

    ssa!     ((vm, sgt_i32_branch_v1) <int1> blk_entry_cond);
    inst!    ((vm, sgt_i32_branch_v1) blk_entry_cmp:
        blk_entry_cond = CMPOP (CmpOp::SGT) blk_entry_a blk_entry_b
    );

    block!   ((vm, sgt_i32_branch_v1) blk_ret1);
    consta!  ((vm, sgt_i32_branch_v1) int32_1_local = int32_1);
    block!   ((vm, sgt_i32_branch_v1) blk_ret0);
    consta!  ((vm, sgt_i32_branch_v1) int32_0_local = int32_0);

    inst!   ((vm, sgt_i32_branch_v1) blk_entry_branch:
        BRANCH2 (blk_entry_cond, int32_1_local, int32_0_local)
            IF (OP 0)
            THEN blk_ret1 (vec![1]) WITH 0.6f32,
            ELSE blk_ret0 (vec![2])
    );

    define_block! ((vm, sgt_i32_branch_v1) blk_entry(blk_entry_a, blk_entry_b){
        blk_entry_cmp, blk_entry_branch
    });

    // blk_ret1
    ssa!    ((vm, sgt_i32_branch_v1) <int32> blk_ret1_res);
    inst!   ((vm, sgt_i32_branch_v1) blk_ret1_inst:
        RET (blk_ret1_res)
    );

    define_block! ((vm, sgt_i32_branch_v1) blk_ret1(blk_ret1_res){
        blk_ret1_inst
    });

    // blk_ret0
    ssa!    ((vm, sgt_i32_branch_v1) <int32> blk_ret0_res);
    inst!   ((vm, sgt_i32_branch_v1) blk_ret0_inst:
        RET (blk_ret0_res)
    );

    define_block! ((vm, sgt_i32_branch_v1) blk_ret0(blk_ret0_res){
        blk_ret0_inst
    });

    define_func_ver!((vm) sgt_i32_branch_v1 (entry: blk_entry) {
        blk_entry, blk_ret1, blk_ret0
    });
    
    emit_test! ((vm)
        sgt_i32_branch, sgt_i32_branch_test1, sgt_i32_branch_test1_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int32(-1i32 as u64), int32(0i32 as u64) RET int32(0u64),
    );
    emit_test! ((vm)
        sgt_i32_branch, sgt_i32_branch_test2, sgt_i32_branch_test2_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int32(0i32 as u64), int32(-1i32 as u64) RET int32(1u64),
    );
    emit_test! ((vm)
        sgt_i32_branch, sgt_i32_branch_test3, sgt_i32_branch_test3_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int32(-1i32 as u64), int32(-1i32 as u64) RET int32(0u64),
    );
    emit_test! ((vm)
        sgt_i32_branch, sgt_i32_branch_test4, sgt_i32_branch_test4_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int32(2i32 as u64), int32(1i32 as u64) RET int32(1u64),
    );
    emit_test! ((vm)
        sgt_i32_branch, sgt_i32_branch_test5, sgt_i32_branch_test5_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int32(1i32 as u64), int32(2i32 as u64) RET int32(0u64),
    );
    emit_test! ((vm)
        sgt_i32_branch, sgt_i32_branch_test6, sgt_i32_branch_test6_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int32(2i32 as u64), int32(2i32 as u64) RET int32(0u64),
    );
    emit_test! ((vm)
        sgt_i32_branch, sgt_i32_branch_test7, sgt_i32_branch_test7_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int32(-2i32 as u64), int32(-1i32 as u64) RET int32(0u64),
    );
    emit_test! ((vm)
        sgt_i32_branch, sgt_i32_branch_test8, sgt_i32_branch_test8_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int32(-1i32 as u64), int32(-2i32 as u64) RET int32(1u64),
    );
    emit_test! ((vm)
        sgt_i32_branch, sgt_i32_branch_test9, sgt_i32_branch_test9_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int32(0i32 as u64), int32(0i32 as u64) RET int32(0u64),
    );
    
    vm
}

#[test]
fn test_sge_i32_branch() {
    build_and_run_test!(sge_i32_branch, sge_i32_branch_test1);
    build_and_run_test!(sge_i32_branch, sge_i32_branch_test2);
    build_and_run_test!(sge_i32_branch, sge_i32_branch_test3);
    build_and_run_test!(sge_i32_branch, sge_i32_branch_test4);
    build_and_run_test!(sge_i32_branch, sge_i32_branch_test5);
    build_and_run_test!(sge_i32_branch, sge_i32_branch_test6);
    build_and_run_test!(sge_i32_branch, sge_i32_branch_test7);
    build_and_run_test!(sge_i32_branch, sge_i32_branch_test8);
    build_and_run_test!(sge_i32_branch, sge_i32_branch_test9);
}

fn sge_i32_branch() -> VM {
    let vm = VM::new();

    typedef! ((vm) int32 = mu_int(32));
    typedef! ((vm) int1  = mu_int(1));

    constdef!((vm) <int32> int32_0 = Constant::Int(0));
    constdef!((vm) <int32> int32_1 = Constant::Int(1));

    funcsig! ((vm) sig = (int32, int32) -> (int32));
    funcdecl!((vm) <sig> sge_i32_branch);
    funcdef! ((vm) <sig> sge_i32_branch VERSION sge_i32_branch_v1);

    // blk entry
    block!   ((vm, sge_i32_branch_v1) blk_entry);
    ssa!     ((vm, sge_i32_branch_v1) <int32> blk_entry_a);
    ssa!     ((vm, sge_i32_branch_v1) <int32> blk_entry_b);

    ssa!     ((vm, sge_i32_branch_v1) <int1> blk_entry_cond);
    inst!    ((vm, sge_i32_branch_v1) blk_entry_cmp:
        blk_entry_cond = CMPOP (CmpOp::SGE) blk_entry_a blk_entry_b
    );

    block!   ((vm, sge_i32_branch_v1) blk_ret1);
    consta!  ((vm, sge_i32_branch_v1) int32_1_local = int32_1);
    block!   ((vm, sge_i32_branch_v1) blk_ret0);
    consta!  ((vm, sge_i32_branch_v1) int32_0_local = int32_0);

    inst!   ((vm, sge_i32_branch_v1) blk_entry_branch:
        BRANCH2 (blk_entry_cond, int32_1_local, int32_0_local)
            IF (OP 0)
            THEN blk_ret1 (vec![1]) WITH 0.6f32,
            ELSE blk_ret0 (vec![2])
    );

    define_block! ((vm, sge_i32_branch_v1) blk_entry(blk_entry_a, blk_entry_b){
        blk_entry_cmp, blk_entry_branch
    });

    // blk_ret1
    ssa!    ((vm, sge_i32_branch_v1) <int32> blk_ret1_res);
    inst!   ((vm, sge_i32_branch_v1) blk_ret1_inst:
        RET (blk_ret1_res)
    );

    define_block! ((vm, sge_i32_branch_v1) blk_ret1(blk_ret1_res){
        blk_ret1_inst
    });

    // blk_ret0
    ssa!    ((vm, sge_i32_branch_v1) <int32> blk_ret0_res);
    inst!   ((vm, sge_i32_branch_v1) blk_ret0_inst:
        RET (blk_ret0_res)
    );

    define_block! ((vm, sge_i32_branch_v1) blk_ret0(blk_ret0_res){
        blk_ret0_inst
    });

    define_func_ver!((vm) sge_i32_branch_v1 (entry: blk_entry) {
        blk_entry, blk_ret1, blk_ret0
    });
    
    emit_test! ((vm)
        sge_i32_branch, sge_i32_branch_test1, sge_i32_branch_test1_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int32(-1i32 as u64), int32(0i32 as u64) RET int32(0u64),
    );
    emit_test! ((vm)
        sge_i32_branch, sge_i32_branch_test2, sge_i32_branch_test2_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int32(0i32 as u64), int32(-1i32 as u64) RET int32(1u64),
    );
    emit_test! ((vm)
        sge_i32_branch, sge_i32_branch_test3, sge_i32_branch_test3_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int32(-1i32 as u64), int32(-1i32 as u64) RET int32(1u64),
    );
    emit_test! ((vm)
        sge_i32_branch, sge_i32_branch_test4, sge_i32_branch_test4_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int32(2i32 as u64), int32(1i32 as u64) RET int32(1u64),
    );
    emit_test! ((vm)
        sge_i32_branch, sge_i32_branch_test5, sge_i32_branch_test5_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int32(1i32 as u64), int32(2i32 as u64) RET int32(0u64),
    );
    emit_test! ((vm)
        sge_i32_branch, sge_i32_branch_test6, sge_i32_branch_test6_v1,
         Int, Int RET Int,
         EQ,
         sig,
         int32(2i32 as u64), int32(2i32 as u64) RET int32(1u64),
    );
    emit_test! ((vm)
        sge_i32_branch, sge_i32_branch_test7, sge_i32_branch_test7_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int32(-2i32 as u64), int32(-1i32 as u64) RET int32(0u64),
    );
    emit_test! ((vm)
        sge_i32_branch, sge_i32_branch_test8, sge_i32_branch_test8_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int32(-1i32 as u64), int32(-2i32 as u64) RET int32(1u64),
    );
    emit_test! ((vm)
        sge_i32_branch, sge_i32_branch_test9, sge_i32_branch_test9_v1,
        Int, Int RET Int,
        EQ,
        sig,
        int32(0i32 as u64), int32(0i32 as u64) RET int32(1u64),
    );
    
    vm
}

#[test]
fn test_branch2_eq_50p_1() {
    build_and_run_test!(branch2_eq_50p_1, branch2_eq_50p_1_test1);
    build_and_run_test!(branch2_eq_50p_1, branch2_eq_50p_1_test2);
}

fn branch2_eq_50p_1() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int1  = mu_int(1));
    typedef!    ((vm) int8  = mu_int(8));
    typedef!    ((vm) int64 = mu_int(64));

    constdef!   ((vm) <int8>  int8_1  = Constant::Int(1));
    constdef!   ((vm) <int64> int64_0 = Constant::Int(0));
    constdef!   ((vm) <int64> int64_1 = Constant::Int(1));

    funcsig!    ((vm) sig = (int8) -> (int64));
    funcdecl!   ((vm) <sig> branch2_eq_50p_1);
    funcdef!    ((vm) <sig> branch2_eq_50p_1 VERSION branch2_eq_50p_1_v1);

    // blk entry
    block!      ((vm, branch2_eq_50p_1_v1) blk_entry);
    ssa!        ((vm, branch2_eq_50p_1_v1) <int8> cond);

    block!      ((vm, branch2_eq_50p_1_v1) blk_true);
    block!      ((vm, branch2_eq_50p_1_v1) blk_false);

    consta!     ((vm, branch2_eq_50p_1_v1) int8_1_local = int8_1);
    ssa!        ((vm, branch2_eq_50p_1_v1) <int1> cmp_res);
    inst!       ((vm, branch2_eq_50p_1_v1) blk_entry_cmp:
        cmp_res = CMPOP (CmpOp::EQ) cond int8_1_local
    );

    inst!       ((vm, branch2_eq_50p_1_v1) blk_entry_branch2:
        BRANCH2 (cmp_res)
            IF (OP 0)
            THEN blk_true  (vec![]) WITH 0.5f32,
            ELSE blk_false (vec![])
    );

    define_block!((vm, branch2_eq_50p_1_v1) blk_entry(cond) {
        blk_entry_cmp,
        blk_entry_branch2
    });

    // blk_true
    consta!     ((vm, branch2_eq_50p_1_v1) int64_1_local = int64_1);
    consta!     ((vm, branch2_eq_50p_1_v1) int64_0_local = int64_0);

    inst!       ((vm, branch2_eq_50p_1_v1) blk_true_ret:
        RET (int64_1_local)
    );

    define_block!((vm, branch2_eq_50p_1_v1) blk_true() {
        blk_true_ret
    });

    // blk_false
    inst!       ((vm, branch2_eq_50p_1_v1) blk_false_ret:
        RET (int64_0_local)
    );

    define_block!((vm, branch2_eq_50p_1_v1) blk_false() {
        blk_false_ret
    });

    define_func_ver!((vm) branch2_eq_50p_1_v1 (entry: blk_entry) {
        blk_entry, blk_true, blk_false
    });
    
    emit_test! ((vm)
        branch2_eq_50p_1, branch2_eq_50p_1_test1, branch2_eq_50p_1_test1_v1,
        Int RET Int,
        EQ,
        sig,
        int8(1u64) RET int64(1u64),
    );
    emit_test! ((vm)
        branch2_eq_50p_1, branch2_eq_50p_1_test2, branch2_eq_50p_1_test2_v1,
        Int RET Int,
        EQ,
        sig,
        int8(0u64) RET int64(0u64),
    );
    
    vm
}

#[test]
fn test_branch2_eq_50p_2() {
    build_and_run_test!(branch2_eq_50p_2, branch2_eq_50p_2_test1);
    build_and_run_test!(branch2_eq_50p_2, branch2_eq_50p_2_test2);
}

fn branch2_eq_50p_2() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int1  = mu_int(1));
    typedef!    ((vm) int8  = mu_int(8));
    typedef!    ((vm) int64 = mu_int(64));

    constdef!   ((vm) <int8>  int8_1  = Constant::Int(1));
    constdef!   ((vm) <int64> int64_0 = Constant::Int(0));
    constdef!   ((vm) <int64> int64_1 = Constant::Int(1));

    funcsig!    ((vm) sig = (int8) -> (int64));
    funcdecl!   ((vm) <sig> branch2_eq_50p_2);
    funcdef!    ((vm) <sig> branch2_eq_50p_2 VERSION branch2_eq_50p_2_v1);

    // blk entry
    block!      ((vm, branch2_eq_50p_2_v1) blk_entry);
    ssa!        ((vm, branch2_eq_50p_2_v1) <int8> cond);

    block!      ((vm, branch2_eq_50p_2_v1) blk_true);
    block!      ((vm, branch2_eq_50p_2_v1) blk_false);

    consta!     ((vm, branch2_eq_50p_2_v1) int8_1_local = int8_1);
    ssa!        ((vm, branch2_eq_50p_2_v1) <int1> cmp_res);
    inst!       ((vm, branch2_eq_50p_2_v1) blk_entry_cmp:
        cmp_res = CMPOP (CmpOp::EQ) cond int8_1_local
    );

    inst!       ((vm, branch2_eq_50p_2_v1) blk_entry_branch2:
        BRANCH2 (cmp_res)
            IF (OP 0)
            THEN blk_true  (vec![]) WITH 0.5f32,
            ELSE blk_false (vec![])
    );

    define_block!((vm, branch2_eq_50p_2_v1) blk_entry(cond) {
        blk_entry_cmp,
        blk_entry_branch2
    });

    // blk_true
    consta!     ((vm, branch2_eq_50p_2_v1) int64_1_local = int64_1);
    consta!     ((vm, branch2_eq_50p_2_v1) int64_0_local = int64_0);

    inst!       ((vm, branch2_eq_50p_2_v1) blk_true_ret:
        RET (int64_1_local)
    );

    define_block!((vm, branch2_eq_50p_2_v1) blk_true() {
        blk_true_ret
    });

    // blk_false
    inst!       ((vm, branch2_eq_50p_2_v1) blk_false_ret:
        RET (int64_0_local)
    );

    define_block!((vm, branch2_eq_50p_2_v1) blk_false() {
        blk_false_ret
    });

    define_func_ver!((vm) branch2_eq_50p_2_v1 (entry: blk_entry) {
        blk_entry, blk_false, blk_true
    });
    
    emit_test! ((vm)
        branch2_eq_50p_2, branch2_eq_50p_2_test1, branch2_eq_50p_2_test1_v1,
        Int RET Int,
        EQ,
        sig,
        int8(1u64) RET int64(1u64),
    );
    emit_test! ((vm)
        branch2_eq_50p_2, branch2_eq_50p_2_test2, branch2_eq_50p_2_test2_v1,
        Int RET Int,
        EQ,
        sig,
        int8(0u64) RET int64(0u64),
    );
    
    vm
}

#[test]
fn test_branch2_high_prob_branch_cannot_fallthrough() {
    build_and_run_test!(branch2, branch2_test1, branch2_high_prob_branch_cannot_fallthrough);
    build_and_run_test!(branch2, branch2_test2, branch2_high_prob_branch_cannot_fallthrough);
}

fn branch2_high_prob_branch_cannot_fallthrough() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int1  = mu_int(1));
    typedef!    ((vm) int8  = mu_int(8));
    typedef!    ((vm) int64 = mu_int(64));

    constdef!   ((vm) <int8>  int8_1  = Constant::Int(1));
    constdef!   ((vm) <int64> int64_0 = Constant::Int(0));
    constdef!   ((vm) <int64> int64_1 = Constant::Int(1));

    funcsig!    ((vm) sig = (int8) -> (int64));
    funcdecl!   ((vm) <sig> branch2);
    funcdef!    ((vm) <sig> branch2 VERSION branch2_v1);

    // blk entry
    block!      ((vm, branch2_v1) blk_entry);
    ssa!        ((vm, branch2_v1) <int8> blk_entry_arg);

    consta!     ((vm, branch2_v1) int64_0_local = int64_0);
    consta!     ((vm, branch2_v1) int64_1_local = int64_1);

    // cmp1_res = EQ 0 1
    ssa!        ((vm, branch2_v1) <int1> cmp1_res);
    inst!       ((vm, branch2_v1) blk_entry_cmp:
        cmp1_res = CMPOP (CmpOp::EQ) int64_0_local int64_1_local
    );

    // branch2 cmp1_res blk_true blk_check(blk_entry_arg)
    block!      ((vm, branch2_v1) blk_true);
    block!      ((vm, branch2_v1) blk_check);
    inst!       ((vm, branch2_v1) blk_entry_branch2:
        BRANCH2 (cmp1_res, blk_entry_arg)
            IF (OP 0)
            THEN blk_true  (vec![]) WITH 0.6f32,
            ELSE blk_check (vec![1])
    );

    define_block!((vm, branch2_v1) blk_entry (blk_entry_arg) {
        blk_entry_cmp, blk_entry_branch2
    });

    // blk_check
    ssa!        ((vm, branch2_v1) <int8> blk_check_arg);

    // cmp2_res = EQ blk_check_arg 1
    ssa!        ((vm, branch2_v1) <int1> cmp2_res);
    consta!     ((vm, branch2_v1) int8_1_local = int8_1);
    inst!       ((vm, branch2_v1) blk_check_cmp:
        cmp2_res = CMPOP (CmpOp::EQ) blk_check_arg int8_1_local
    );

    // branch2 cmp2_res blk_true blk_false
    block!      ((vm, branch2_v1) blk_false);
    inst!       ((vm, branch2_v1) blk_check_branch2:
        BRANCH2 (cmp2_res)
            IF (OP 0)
            THEN blk_true  (vec![]) WITH 0.6f32,
            ELSE blk_false (vec![])
    );

    define_block!((vm, branch2_v1) blk_check (blk_check_arg) {
        blk_check_cmp, blk_check_branch2
    });

    // blk_true
    consta!     ((vm, branch2_v1) int64_1_local = int64_1);
    consta!     ((vm, branch2_v1) int64_0_local = int64_0);

    inst!       ((vm, branch2_v1) blk_true_ret:
        RET (int64_1_local)
    );

    define_block!((vm, branch2_v1) blk_true() {
        blk_true_ret
    });

    // blk_false
    inst!       ((vm, branch2_v1) blk_false_ret:
        RET (int64_0_local)
    );

    define_block!((vm, branch2_v1) blk_false() {
        blk_false_ret
    });

    define_func_ver!((vm) branch2_v1 (entry: blk_entry) {
        blk_entry, blk_check, blk_true, blk_false
    });
    
    emit_test! ((vm)
        branch2, branch2_test1, branch2_test1_v1,
        Int RET Int,
        EQ,
        sig,
        int8(1u64) RET int64(1u64),
    );
    emit_test! ((vm)
        branch2, branch2_test2, branch2_test2_v1,
        Int RET Int,
        EQ,
        sig,
        int8(0u64) RET int64(0u64),
    );
    
    vm
}
