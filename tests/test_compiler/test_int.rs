extern crate mu;
extern crate libloading;

use self::mu::ast::types::*;
use self::mu::ast::ir::*;
use self::mu::ast::inst::*;
use self::mu::ast::op::*;
use self::mu::vm::*;

use std::sync::RwLock;
use mu::testutil;

#[test]
fn test_add_u8() {
    let lib = testutil::compile_fnc("add_u8", &add_u8);

    unsafe {
        let add_u8 : libloading::Symbol<unsafe extern fn(u8, u8) -> u8> = lib.get(b"add_u8").unwrap();

        let add_u8_1_1 = add_u8(1, 1);
        println!("add_u8(1, 1) = {}", add_u8_1_1);
        assert!(add_u8_1_1 == 2);

        let add_u8_255_1 = add_u8(255u8, 1u8);
        println!("add_u8(255, 1) = {}", add_u8_255_1);
        assert!(add_u8_255_1 == 0);
    }
}

fn add_u8() -> VM {
    let vm = VM::new();

    // .typedef @u8 = int<8>
    let type_def_u8 = vm.declare_type(vm.next_id(), MuType_::int(8));
    vm.set_name(type_def_u8.as_entity(), Mu("u8"));

    // .funcsig @add_u8_sig = (@u8 @u8) -> (@u8)
    let add_u8_sig = vm.declare_func_sig(vm.next_id(), vec![type_def_u8.clone()], vec![type_def_u8.clone(), type_def_u8.clone()]);
    vm.set_name(add_u8_sig.as_entity(), Mu("add_u8_sig"));

    // .funcdecl @add_u8 <@add_u8_sig>
    let func_id = vm.next_id();
    let func = MuFunction::new(func_id, add_u8_sig.clone());
    vm.set_name(func.as_entity(), Mu("add_u8"));
    vm.declare_func(func);

    // .funcdef @add_u8 VERSION @add_u8_v1 <@add_u8_sig>
    let mut func_ver = MuFunctionVersion::new(vm.next_id(), func_id, add_u8_sig.clone());
    vm.set_name(func_ver.as_entity(), Mu("add_u8_v1"));

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
        let truncate : libloading::Symbol<unsafe extern fn(u64) -> u8> = lib.get(b"truncate").unwrap();

        let res = truncate(0xF01u64);
        println!("truncate(0xF01) = {}", res);
        assert!(res == 1);
    }
}

fn truncate() -> VM {
    let vm = VM::new();

    typedef! ((vm) u64 = mu_int(64));
    typedef! ((vm) u8  = mu_int(8));

    funcsig! ((vm) sig = (u64) -> (u64));
    funcdecl!((vm) <sig> truncate);
    funcdef! ((vm) <sig> truncate VERSION truncate_v1);

    block!   ((vm, trucnate_v1) blk_entry);
    ssa!     ((vm, truncate_v1) <u64> blk_entry_a);

    ssa!     ((vm, truncate_v1) <u8>  blk_entry_r);
    inst!    ((vm, truncate_v1) blk_entry_truncate:
        blk_entry_r = CONVOP (ConvOp::TRUNC) <u64 u8> blk_entry_a
    );

    ssa!     ((vm, truncate_v1) <u64> blk_entry_r2);
    inst!    ((vm, truncate_v1) blk_entry_zext:
        blk_entry_r2 = CONVOP (ConvOp::ZEXT) <u8 u64> blk_entry_r
    );

    inst!    ((vm, truncate_v1) blk_entry_ret:
        RET (blk_entry_r2)
    );

    define_block! ((vm, truncate_v1) blk_entry(blk_entry_a) {
        blk_entry_truncate,
        blk_entry_zext,
        blk_entry_ret
    });

    define_func_ver! ((vm) truncate_v1 (entry: blk_entry) {blk_entry});

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

#[test]
fn test_add_9f() {
    let lib = testutil::compile_fnc("add_9f", &add_9f);

    unsafe {
        let add_9f : libloading::Symbol<unsafe extern fn(u64) -> u64> = lib.get(b"add_9f").unwrap();

        let add_9f_1 = add_9f(1);
        println!("add_9f(1) = {}", add_9f_1);
        assert!(add_9f_1 == 0x1000000000);
    }
}

fn add_9f() -> VM {
    let vm = VM::new();

    // .typedef @u64 = int<64>
    let type_def_u64 = vm.declare_type(vm.next_id(), MuType_::int(64));
    vm.set_name(type_def_u64.as_entity(), Mu("u64"));

    // .const @int_9f <@u64> = 0xfffffffff
    let const_def_int_9f = vm.declare_const(vm.next_id(), type_def_u64.clone(), Constant::Int(0xfffffffff));
    vm.set_name(const_def_int_9f.as_entity(), "int_9f".to_string());

    // .funcsig @add_9f_sig = (@u64) -> (@u64)
    let add_9f_sig = vm.declare_func_sig(vm.next_id(), vec![type_def_u64.clone()], vec![type_def_u64.clone()]);
    vm.set_name(add_9f_sig.as_entity(), Mu("add_9f_sig"));

    // .funcdecl @add_9f <@add_9f_sig>
    let func_id = vm.next_id();
    let func = MuFunction::new(func_id, add_9f_sig.clone());
    vm.set_name(func.as_entity(), Mu("add_9f"));
    vm.declare_func(func);

    // .funcdef @add_9f VERSION @add_9f_v1 <@add_9f_sig>
    let mut func_ver = MuFunctionVersion::new(vm.next_id(), func_id, add_9f_sig.clone());
    vm.set_name(func_ver.as_entity(), Mu("add_9f_v1"));

    // %entry(<@u64> %a):
    let mut blk_entry = Block::new(vm.next_id());
    vm.set_name(blk_entry.as_entity(), Mu("entry"));

    let blk_entry_a = func_ver.new_ssa(vm.next_id(), type_def_u64.clone());
    vm.set_name(blk_entry_a.as_entity(), Mu("blk_entry_a"));

    let const_int_9f_local = func_ver.new_constant(const_def_int_9f.clone());

    // %r = ADD %a %b
    let blk_entry_r = func_ver.new_ssa(vm.next_id(), type_def_u64.clone());
    vm.set_name(blk_entry_r.as_entity(), Mu("blk_entry_r"));
    let blk_entry_add = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_entry_r.clone_value()]),
        ops: RwLock::new(vec![blk_entry_a.clone(), const_int_9f_local.clone()]),
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