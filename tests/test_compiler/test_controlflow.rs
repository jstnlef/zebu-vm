extern crate libloading;

use mu::ast::types::*;
use mu::ast::ir::*;
use mu::ast::inst::*;
use mu::ast::op::*;
use mu::vm::*;
use mu::testutil;
use mu::utils::LinkedHashMap;

use std::sync::RwLock;

#[test]
fn test_switch() {
    let lib = testutil::compile_fnc("switch", &switch);

    unsafe {
        let switch : libloading::Symbol<unsafe extern fn(u64) -> u64> = lib.get(b"switch").unwrap();

        let res = switch(0);
        println!("switch(0) = {}", res);
        assert!(res == 0);

        let res = switch(1);
        println!("switch(1) = {}", res);
        assert!(res == 1);

        let res = switch(2);
        println!("switch(2) = {}", res);
        assert!(res == 2);

        let res = switch(3);
        println!("switch(3) = {}", res);
        assert!(res == 99);
    }
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
        ops: RwLock::new(vec![
            a.clone(), // 0
            int64_0_local.clone(), // 1
            int64_1_local.clone(), // 2
            int64_2_local.clone(), // 3
        ]),
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

    vm
}

#[test]
fn test_select_eq_zero() {
    let lib = testutil::compile_fnc("select_eq_zero", &select_eq_zero);

    unsafe {
        let select_eq_zero : libloading::Symbol<unsafe extern fn(u64) -> u64> = lib.get(b"select_eq_zero").unwrap();

        let res = select_eq_zero(0);
        println!("select_eq_zero(0) = {}", res);
        assert!(res == 1);

        let res = select_eq_zero(1);
        println!("select_eq_zero(1) = {}", res);
        assert!(res == 0);
    }
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

    vm
}

#[test]
fn test_select_u8_eq_zero() {
    let lib = testutil::compile_fnc("select_u8_eq_zero", &select_u8_eq_zero);

    unsafe {
        let select_eq_zero : libloading::Symbol<unsafe extern fn(u8) -> u8> = lib.get(b"select_u8_eq_zero").unwrap();

        let res = select_eq_zero(0);
        println!("select_u8_eq_zero(0) = {}", res);
        assert!(res == 1);

        let res = select_eq_zero(1);
        println!("select_u8_eq_zero(1) = {}", res);
        assert!(res == 0);
    }
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

    vm
}

#[test]
fn test_select_sge_zero() {
    let lib = testutil::compile_fnc("select_sge_zero", &select_sge_zero);

    unsafe {
        let select_sge_zero : libloading::Symbol<unsafe extern fn(i64) -> u64> = lib.get(b"select_sge_zero").unwrap();

        let res = select_sge_zero(0);
        println!("select_sge_zero(0) = {}", res);
        assert!(res == 1);

        let res = select_sge_zero(1);
        println!("select_sge_zero(1) = {}", res);
        assert!(res == 1);

        let res = select_sge_zero(-1);
        println!("select_sge_zero(-1) = {}", res);
        assert!(res == 0);
    }
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

    vm
}

#[test]
fn test_sgt_value() {
    let lib = testutil::compile_fnc("sgt_value", &sgt_value);

    unsafe {
        let sgt_value : libloading::Symbol<unsafe extern fn(i64, i64) -> u8> = lib.get(b"sgt_value").unwrap();

        let res = sgt_value(255, 0);
        println!("sgt_value(255, 0) = {}", res);
        assert!(res == 1);

        let res = sgt_value(255, 255);
        println!("sgt_value(255, 255) = {}", res);
        assert!(res == 0);

        let res = sgt_value(0, 255);
        println!("sgt_value(0, 255) = {}", res);
        assert!(res == 0);
    }
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

    vm
}

#[test]
fn test_sgt_u8_value() {
    let lib = testutil::compile_fnc("sgt_u8_value", &sgt_u8_value);

    unsafe {
        let sgt_u8_value : libloading::Symbol<unsafe extern fn(i8, i8) -> u8> = lib.get(b"sgt_u8_value").unwrap();

        let res = sgt_u8_value(-1, 0);
        println!("sgt_u8_value(-1, 0) = {}", res);
        assert!(res == 0);

        let res = sgt_u8_value(0, -1);
        println!("sgt_u8_value(0, -1) = {}", res);
        assert!(res == 1);

        let res = sgt_u8_value(2, 1);
        println!("sgt_u8_value(2, 1) = {}", res);
        assert!(res == 1);

        let res = sgt_u8_value(1, 2);
        println!("sgt_u8_value(1, 2) = {}", res);
        assert!(res == 0);

        let res = sgt_u8_value(-2, -1);
        println!("sgt_u8_value(-2, -1) = {}", res);
        assert!(res == 0);

        let res = sgt_u8_value(-1, -2);
        println!("sgt_u8_value(-1, -2) = {}", res);
        assert!(res == 1);
    }
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

    vm
}

#[test]
fn test_sgt_i32_branch() {
    let lib = testutil::compile_fnc("sgt_i32_branch", &sgt_i32_branch);

    unsafe {
        let sgt_i32 : libloading::Symbol<unsafe extern fn(i32, i32) -> u32> = lib.get(b"sgt_i32_branch").unwrap();

        let res = sgt_i32(-1, 0);
        println!("sgt_i32(-1, 0) = {}", res);
        assert!(res == 0);

        let res = sgt_i32(0, -1);
        println!("sgt_i32(0, -1) = {}", res);
        assert!(res == 1);

        let res = sgt_i32(-1, -1);
        println!("sgt_i32(-1, -1) = {}", res);
        assert!(res == 0);

        let res = sgt_i32(2, 1);
        println!("sgt_i32(2, 1) = {}", res);
        assert!(res == 1);

        let res = sgt_i32(1, 2);
        println!("sgt_i32(1, 2) = {}", res);
        assert!(res == 0);

        let res = sgt_i32(2, 2);
        println!("sgt_i32(2, 2) = {}", res);
        assert!(res == 0);

        let res = sgt_i32(-2, -1);
        println!("sgt_i32(-2, -1) = {}", res);
        assert!(res == 0);

        let res = sgt_i32(-1, -2);
        println!("sgt_i32(-1, -2) = {}", res);
        assert!(res == 1);

        let res = sgt_i32(0, 0);
        println!("sgt_i32(0, 0) = {}", res);
        assert!(res == 0);
    }
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

    vm
}

#[test]
fn test_sge_i32_branch() {
    let lib = testutil::compile_fnc("sge_i32_branch", &sge_i32_branch);

    unsafe {
        let sge_i32 : libloading::Symbol<unsafe extern fn(i32, i32) -> u32> = lib.get(b"sge_i32_branch").unwrap();

        let res = sge_i32(-1, 0);
        println!("sge_i32(-1, 0) = {}", res);
        assert!(res == 0);

        let res = sge_i32(0, -1);
        println!("sge_i32(0, -1) = {}", res);
        assert!(res == 1);

        let res = sge_i32(-1, -1);
        println!("sge_i32(-1, -1) = {}", res);
        assert!(res == 1);

        let res = sge_i32(2, 1);
        println!("sge_i32(2, 1) = {}", res);
        assert!(res == 1);

        let res = sge_i32(1, 2);
        println!("sge_i32(1, 2) = {}", res);
        assert!(res == 0);

        let res = sge_i32(2, 2);
        println!("sge_i32(2, 2) = {}", res);
        assert!(res == 1);

        let res = sge_i32(-2, -1);
        println!("sge_i32(-2, -1) = {}", res);
        assert!(res == 0);

        let res = sge_i32(-1, -2);
        println!("sge_i32(-1, -2) = {}", res);
        assert!(res == 1);

        let res = sge_i32(0, 0);
        println!("sge_i32(0, 0) = {}", res);
        assert!(res == 1);
    }
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

    vm
}
