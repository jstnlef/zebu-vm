extern crate mu;
extern crate log;
extern crate simple_logger;
extern crate libloading;

use self::mu::ast::types::*;
use self::mu::ast::ir::*;
use self::mu::ast::inst::*;
use self::mu::ast::op::*;
use self::mu::vm::*;
use self::mu::compiler::*;

use std::sync::RwLock;
use std::sync::Arc;
use mu::testutil;
use mu::testutil::aot;

#[test]
fn test_u8_add() {
    let lib = testutil::compile_fnc("u8_add", &u8_add);

    unsafe {
        let u8_add : libloading::Symbol<unsafe extern fn(u8, u8) -> u8> = lib.get(b"u8_add").unwrap();

        let u8_add_1_1 = u8_add(1, 1);
        println!("u8_add(1, 1) = {}", u8_add_1_1);
        assert!(u8_add_1_1 == 2);

        let u8_add_255_1 = u8_add(255u8, 1u8);
        println!("u8_add(255, 1) = {}", u8_add_255_1);
        assert!(u8_add_255_1 == 0);
    }
}

fn u8_add() -> VM {
    let vm = VM::new();

    // .typedef @u8 = int<8>
    let type_def_u8 = vm.declare_type(vm.next_id(), MuType_::int(8));
    vm.set_name(type_def_u8.as_entity(), Mu("u8"));

    // .funcsig @u8_add_sig = (@u8 @u8) -> (@u8)
    let u8_add_sig = vm.declare_func_sig(vm.next_id(), vec![type_def_u8.clone()], vec![type_def_u8.clone(), type_def_u8.clone()]);
    vm.set_name(u8_add_sig.as_entity(), Mu("u8_add_sig"));

    // .funcdecl @u8_add <@u8_add_sig>
    let func_id = vm.next_id();
    let func = MuFunction::new(func_id, u8_add_sig.clone());
    vm.set_name(func.as_entity(), Mu("u8_add"));
    vm.declare_func(func);

    // .funcdef @u8_add VERSION @u8_add_v1 <@u8_add_sig>
    let mut func_ver = MuFunctionVersion::new(vm.next_id(), func_id, u8_add_sig.clone());
    vm.set_name(func_ver.as_entity(), Mu("u8_add_v1"));

    // %entry(<@u8> %a, <@u8> %b):
    let mut blk_entry = Block::new(vm.next_id());
    vm.set_name(blk_entry.as_entity(), Mu("entry"));

    let blk_entry_a = func_ver.new_ssa(vm.next_id(), type_def_u8.clone());
    vm.set_name(blk_entry_a.as_entity(), Mu("blk_entry_a"));
    let blk_entry_b = func_ver.new_ssa(vm.next_id(), type_def_u8.clone());
    vm.set_name(blk_entry_b.as_entity(), Mu("blk_entry_b"));

    // %r = ADD %a %b
    let blk_entry_r = func_ver.new_ssa(vm.next_id(), type_def_u8.clone());
    vm.set_name(blk_entry_r.as_entity(), Mu("blk_entry_r"));
    let blk_entry_add = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_entry_r.clone_value()]),
        ops: RwLock::new(vec![blk_entry_a.clone(), blk_entry_b.clone()]),
        v: Instruction_::BinOp(BinOp::Add, 0, 1)
    });

    // RET %r
    let blk_entry_term = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: None,
        ops: RwLock::new(vec![blk_entry_r.clone()]),
        v: Instruction_::Return(vec![0])
    });

    blk_entry.content = Some(BlockContent{
        args: vec![blk_entry_a.clone_value(), blk_entry_b.clone_value()],
        exn_arg: None,
        body: vec![blk_entry_add, blk_entry_term],
        keepalives: None
    });

    func_ver.define(FunctionContent{
        entry: blk_entry.id(),
        blocks: hashmap!{
            blk_entry.id() => blk_entry
        }
    });

    vm.define_func_version(func_ver);

    vm
}

#[test]
fn test_truncate() {
    let lib = testutil::compile_fnc("truncate", &truncate);

    unsafe {
        let truncate : libloading::Symbol<unsafe extern fn(u64) -> u64> = lib.get(b"truncate").unwrap();

        let res = truncate(0xF01u64);
        println!("truncate(0xF01) = {}", res);
        assert!(res == 1);
    }
}

fn truncate() -> VM {
    let vm = VM::new();

    // .typedef @u64 = int<64>
    let type_def_u64 = vm.declare_type(vm.next_id(), MuType_::int(64));
    vm.set_name(type_def_u64.as_entity(), Mu("u64"));
    // .typedef @u8 = int<8>
    let type_def_u8 = vm.declare_type(vm.next_id(), MuType_::int(8));
    vm.set_name(type_def_u8.as_entity(), Mu("u8"));

    // .funcsig @truncate_sig = (@u64) -> (@u64)
    let truncate_sig = vm.declare_func_sig(vm.next_id(), vec![type_def_u64.clone()], vec![type_def_u64.clone()]);
    vm.set_name(truncate_sig.as_entity(), Mu("truncate_sig"));

    // .funcdecl @truncate <@truncate_sig>
    let func_id = vm.next_id();
    let func = MuFunction::new(func_id, truncate_sig.clone());
    vm.set_name(func.as_entity(), Mu("truncate"));
    vm.declare_func(func);

    // .funcdef @truncate VERSION @truncate_v1 <@truncate_sig>
    let mut func_ver = MuFunctionVersion::new(vm.next_id(), func_id, truncate_sig.clone());
    vm.set_name(func_ver.as_entity(), Mu("truncate_v1"));

    // %entry(<@u64> %a):
    let mut blk_entry = Block::new(vm.next_id());
    vm.set_name(blk_entry.as_entity(), Mu("entry"));

    let blk_entry_a = func_ver.new_ssa(vm.next_id(), type_def_u8.clone());
    vm.set_name(blk_entry_a.as_entity(), Mu("blk_entry_a"));

    // %r = TRUNC @u64->@u8 %a
    let blk_entry_r = func_ver.new_ssa(vm.next_id(), type_def_u8.clone());
    vm.set_name(blk_entry_r.as_entity(), Mu("blk_entry_r"));

    let blk_entry_truncate = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_entry_r.clone_value()]),
        ops: RwLock::new(vec![blk_entry_a.clone()]),
        v: Instruction_::ConvOp{
            operation: ConvOp::TRUNC,
            from_ty: type_def_u64.clone(),
            to_ty: type_def_u8.clone(),
            operand: 0
        }
    });

    // %r2 = ZEXT @u8->@u64 %r
    let blk_entry_r2 = func_ver.new_ssa(vm.next_id(), type_def_u64.clone());
    vm.set_name(blk_entry_r2.as_entity(), Mu("blk_entry_r2"));

    let blk_entry_zext = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_entry_r2.clone_value()]),
        ops: RwLock::new(vec![blk_entry_r.clone()]),
        v: Instruction_::ConvOp {
            operation: ConvOp::ZEXT,
            from_ty: type_def_u8.clone(),
            to_ty: type_def_u64.clone(),
            operand: 0
        }
    });

    // RET %r2
    let blk_entry_term = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: None,
        ops: RwLock::new(vec![blk_entry_r2.clone()]),
        v: Instruction_::Return(vec![0])
    });

    blk_entry.content = Some(BlockContent{
        args: vec![blk_entry_a.clone_value()],
        exn_arg: None,
        body: vec![blk_entry_truncate, blk_entry_zext, blk_entry_term],
        keepalives: None
    });

    func_ver.define(FunctionContent{
        entry: blk_entry.id(),
        blocks: hashmap!{
            blk_entry.id() => blk_entry
        }
    });

    vm.define_func_version(func_ver);

    vm
}

#[test]
fn test_sext() {
    let lib = testutil::compile_fnc("sext", &sext);

    unsafe {
        let sext : libloading::Symbol<unsafe extern fn(i8) -> i64> = lib.get(b"sext").unwrap();

        let res = sext(-1);
        println!("truncate(-1) = {}", res);
        assert!(res == -1);
    }
}

fn sext() -> VM {
    let vm = VM::new();

    // .typedef @i8 = int<8>
    let type_def_i8 = vm.declare_type(vm.next_id(), MuType_::int(8));
    vm.set_name(type_def_i8.as_entity(), Mu("i8"));
    // .typedef @i64 = int<64>
    let type_def_i64 = vm.declare_type(vm.next_id(), MuType_::int(64));
    vm.set_name(type_def_i64.as_entity(), Mu("i64"));

    // .funcsig @sext_sig = (@i8) -> (@i64)
    let sext_sig = vm.declare_func_sig(vm.next_id(), vec![type_def_i64.clone()], vec![type_def_i8.clone()]);
    vm.set_name(sext_sig.as_entity(), Mu("sext_sig"));

    // .funcdecl @sext <@sext_sig>
    let func_id = vm.next_id();
    let func = MuFunction::new(func_id, sext_sig.clone());
    vm.set_name(func.as_entity(), Mu("sext"));
    vm.declare_func(func);

    // .funcdef @sext VERSION @sext_v1 <@sext_sig>
    let mut func_ver = MuFunctionVersion::new(vm.next_id(), func_id, sext_sig.clone());
    vm.set_name(func_ver.as_entity(), Mu("sext_v1"));

    // %entry(<@i8> %a):
    let mut blk_entry = Block::new(vm.next_id());
    vm.set_name(blk_entry.as_entity(), Mu("entry"));

    let blk_entry_a = func_ver.new_ssa(vm.next_id(), type_def_i8.clone());
    vm.set_name(blk_entry_a.as_entity(), Mu("blk_entry_a"));

    // %r = SEXT @i8->@i64 %a
    let blk_entry_r = func_ver.new_ssa(vm.next_id(), type_def_i64.clone());
    vm.set_name(blk_entry_r.as_entity(), Mu("blk_entry_r"));
    let blk_entry_add = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_entry_r.clone_value()]),
        ops: RwLock::new(vec![blk_entry_a.clone()]),
        v: Instruction_::ConvOp{
            operation: ConvOp::SEXT,
            from_ty: type_def_i8.clone(),
            to_ty: type_def_i64.clone(),
            operand: 0
        }
    });

    // RET %r
    let blk_entry_term = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: None,
        ops: RwLock::new(vec![blk_entry_r.clone()]),
        v: Instruction_::Return(vec![0])
    });

    blk_entry.content = Some(BlockContent{
        args: vec![blk_entry_a.clone_value()],
        exn_arg: None,
        body: vec![blk_entry_add, blk_entry_term],
        keepalives: None
    });

    func_ver.define(FunctionContent{
        entry: blk_entry.id(),
        blocks: hashmap!{
            blk_entry.id() => blk_entry
        }
    });

    vm.define_func_version(func_ver);

    vm
}