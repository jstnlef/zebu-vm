extern crate libloading;

use mu::ast::types::*;
use mu::ast::ir::*;
use mu::ast::inst::*;
use mu::ast::op::*;
use mu::vm::*;
use mu::testutil;

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

    // .typedef @int64 = int<64>
    let type_def_int64 = vm.declare_type(vm.next_id(), MuType_::int(64));
    vm.set_name(type_def_int64.as_entity(), Mu("int64"));
    
    // .const @int64_0 <@int64> = 0
    let const_int64_0 = vm.declare_const(vm.next_id(), type_def_int64.clone(), Constant::Int(0));
    // .const @int64_1 <@int64> = 1
    let const_int64_1 = vm.declare_const(vm.next_id(), type_def_int64.clone(), Constant::Int(1));
    // .const @int64_2 <@int64> = 2
    let const_int64_2 = vm.declare_const(vm.next_id(), type_def_int64.clone(), Constant::Int(2));
    // .const @int64_99 <@int64> = 99
    let const_int64_99 = vm.declare_const(vm.next_id(), type_def_int64.clone(), Constant::Int(99));

    // .funcsig @switch_sig = (@int64) -> (@int64)
    let switch_sig = vm.declare_func_sig(vm.next_id(), vec![type_def_int64.clone()], vec![type_def_int64.clone()]);
    vm.set_name(switch_sig.as_entity(), Mu("switch_sig"));

    // .funcdecl @switch <@switch_sig>
    let func_id = vm.next_id();
    let func = MuFunction::new(func_id, switch_sig.clone());
    vm.set_name(func.as_entity(), Mu("switch"));
    vm.declare_func(func);

    // .funcdef @switch VERSION @switch_v1 <@switch_sig>
    let mut func_ver = MuFunctionVersion::new(vm.next_id(), func_id, switch_sig.clone());
    vm.set_name(func_ver.as_entity(), Mu("switch_v1"));

    // %entry(<@int64> %a):
    let mut blk_entry = Block::new(vm.next_id());
    vm.set_name(blk_entry.as_entity(), Mu("entry"));

    let blk_entry_a = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_a.as_entity(), Mu("blk_entry_a"));

    // SWITCH %a %blk_default (0 -> %blk_ret0, 1 -> %blk_ret1, 2 -> %blk_ret2)
    let const0 = func_ver.new_constant(const_int64_0.clone());
    let const1 = func_ver.new_constant(const_int64_1.clone());
    let const2 = func_ver.new_constant(const_int64_2.clone());

    let blk_default_id = vm.next_id();
    let blk_ret0_id = vm.next_id();
    let blk_ret1_id = vm.next_id();
    let blk_ret2_id = vm.next_id();

    let blk_entry_switch = func_ver.new_inst(Instruction {
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: None,
        ops: RwLock::new(vec![
            blk_entry_a.clone(), // 0
            const0.clone(), // 1
            const1.clone(), // 2
            const2.clone(), // 3
        ]),
        v: Instruction_::Switch {
            cond: 0,
            default: Destination {
                target: blk_default_id,
                args: vec![]
            },
            branches: vec![
                (1, Destination{target: blk_ret0_id, args: vec![]}),
                (2, Destination{target: blk_ret1_id, args: vec![]}),
                (3, Destination{target: blk_ret2_id, args: vec![]})
            ]
        }
    });

    blk_entry.content = Some(BlockContent{
        args: vec![blk_entry_a.clone_value()],
        exn_arg: None,
        body: vec![blk_entry_switch],
        keepalives: None
    });

    // blk_default

    let mut blk_default = Block::new(blk_default_id);
    vm.set_name(blk_default.as_entity(), Mu("default"));

    let const99 = func_ver.new_constant(const_int64_99.clone());
    
    let blk_default_ret = func_ver.new_inst(Instruction {
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: None,
        ops: RwLock::new(vec![const99]),
        v: Instruction_::Return(vec![0])
    });

    blk_default.content = Some(BlockContent{
        args: vec![],
        exn_arg: None,
        body: vec![blk_default_ret],
        keepalives: None
    });

    // blk_ret0

    let mut blk_ret0 = Block::new(blk_ret0_id);
    vm.set_name(blk_ret0.as_entity(), Mu("ret0"));

    let blk_ret0_ret = func_ver.new_inst(Instruction {
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: None,
        ops: RwLock::new(vec![const0.clone()]),
        v: Instruction_::Return(vec![0])
    });

    blk_ret0.content = Some(BlockContent{
        args: vec![],
        exn_arg: None,
        body: vec![blk_ret0_ret],
        keepalives: None
    });

    // blk_ret1

    let mut blk_ret1 = Block::new(blk_ret1_id);
    vm.set_name(blk_ret1.as_entity(), Mu("ret1"));

    let blk_ret1_ret = func_ver.new_inst(Instruction {
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: None,
        ops: RwLock::new(vec![const1.clone()]),
        v: Instruction_::Return(vec![0])
    });

    blk_ret1.content = Some(BlockContent{
        args: vec![],
        exn_arg: None,
        body: vec![blk_ret1_ret],
        keepalives: None
    });

    // blk_ret2

    let mut blk_ret2 = Block::new(blk_ret2_id);
    vm.set_name(blk_ret2.as_entity(), Mu("ret2"));

    let blk_ret2_ret = func_ver.new_inst(Instruction {
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: None,
        ops: RwLock::new(vec![const2.clone()]),
        v: Instruction_::Return(vec![0])
    });

    blk_ret2.content = Some(BlockContent{
        args: vec![],
        exn_arg: None,
        body: vec![blk_ret2_ret],
        keepalives: None
    });

    func_ver.define(FunctionContent{
        entry: blk_entry.id(),
        blocks: hashmap!{
            blk_entry.id() => blk_entry,
            blk_default_id => blk_default,
            blk_ret0_id => blk_ret0,
            blk_ret1_id => blk_ret1,
            blk_ret2_id => blk_ret2
        }
    });

    vm.define_func_version(func_ver);

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
        let sgt_value : libloading::Symbol<unsafe extern fn(i64, i64) -> u64> = lib.get(b"sgt_value").unwrap();

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
        let sgt_u8_value : libloading::Symbol<unsafe extern fn(i8, i8) -> u64> = lib.get(b"sgt_u8_value").unwrap();

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