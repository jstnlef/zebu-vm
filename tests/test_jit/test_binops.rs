extern crate mu;
extern crate libloading as ll;

use test_ir::test_ir::sum;
use test_ir::test_ir::factorial;
use mu::ast::ir::*;
use mu::ast::op::*;
use mu::ast::inst::*;
use mu::ast::types::*;
use mu::vm::*;

use std::sync::RwLock;
use std::collections::HashMap;

use mu::testutil::compile_fnc;


#[test]
fn test_add_8bit_wraparound() {
    fn build_fn() -> VM {
        /* Build the following bundle:

            .typedef @i8 = int<8>
            .const @0xff_i8 <@i8> = 0xff
            .const @0x0a_i8 <@i8> = 0x0a
            .funcsig @sig__i8 = () -> (@i8)
            .funcdecl @fnc <@fnrsig__i8>
            .funcdef @fnc VERSION @fnc_v1 <@sig__i8> {
                @fnc_v1.blk0():
                    @fnc_v1.blk0.res = ADD <@i8> @0xff_i8 @0x0a_i8
                    RET @fnc_v1.blk0.res
            }
        */
        let vm = VM::new();
        let i8 = vm.declare_type(vm.next_id(), MuType_::int(8));
        vm.set_name(i8.as_entity(), "i8".to_string());
        let c_0xff_i8 = vm.declare_const(vm.next_id(), i8.clone(), Constant::Int(0xff));
        vm.set_name(c_0xff_i8.as_entity(), "0xff_i8".to_string());
        let c_0x0a_i8 = vm.declare_const(vm.next_id(), i8.clone(), Constant::Int(0x0a));
        vm.set_name(c_0x0a_i8.as_entity(), "0x0a_i8".to_string());
        let sig__i8 = vm.declare_func_sig(vm.next_id(), vec![], vec![i8.clone()]);
        vm.set_name(sig__i8.as_entity(), "sig__i8".to_string());
        // .funcdecl @fnc <@fnrsig__i8>
        let fnc = MuFunction::new(vm.next_id(), sig__i8.clone());
        vm.set_name(fnc.as_entity(), "fnc".to_string());
        let id_fnc = fnc.id();
        vm.declare_func(fnc);

        // .funcdef @fnc VERSION @v1 <@sig__i8> {
        let mut fnc_ver = MuFunctionVersion::new(vm.next_id(), id_fnc, sig__i8.clone());
        vm.set_name(fnc_ver.as_entity(), "fnc_v1".to_string());
        // blk0
        let mut blk0 = Block::new(vm.next_id());
        vm.set_name(blk0.as_entity(), "fnc_v1.blk0".to_string());

        // @fnc_v1.blk0.res = ADD <@i8> @0xff_i8 @0x0a_i8
        let lc_0xff_i8 = fnc_ver.new_constant(c_0xff_i8.clone());
        let lc_0x0a_i8 = fnc_ver.new_constant(c_0x0a_i8.clone());
        let res = fnc_ver.new_ssa(vm.next_id(), i8.clone());
        vm.set_name(res.as_entity(), "fnc_v1.blk0.res".to_string());
        let op_add = fnc_ver.new_inst(Instruction{
            hdr: MuEntityHeader::unnamed(vm.next_id()),
            value: Some(vec![res.clone_value()]),
            ops: RwLock::new(vec![lc_0xff_i8.clone(), lc_0x0a_i8.clone()]),
            v: Instruction_::BinOp(BinOp::Add, 0, 1)
        });

        let op_ret = fnc_ver.new_inst(Instruction{
            hdr: MuEntityHeader::unnamed(vm.next_id()),
            value: None,
            ops: RwLock::new(vec![res.clone()]),
            v: Instruction_::Return(vec![0])
        });
        blk0.content = Some(BlockContent{
            args: vec![],
            exn_arg: None,
            body: vec![op_add, op_ret],
            keepalives: None
        });
        fnc_ver.define(FunctionContent{
            entry: blk0.id(),
            blocks: {
                let mut blocks = HashMap::new();
                blocks.insert(blk0.id(), blk0);
                blocks
            }
        });
        vm.define_func_version(fnc_ver);

        vm
    }
    let lib = compile_fnc("fnc", &build_fn);
    unsafe {
        let fncptr: ll::Symbol<unsafe extern fn () -> u8> = lib.get(b"fnc").unwrap();
        assert!(fncptr() == 9);
    }
}

#[test]
fn test_sub_8bit_wraparound() {
    fn build_fn() -> VM {
        /* Build the following bundle:

            .typedef @i8 = int<8>
            .const @0xff_i8 <@i8> = 0xff
            .const @0x0a_i8 <@i8> = 0x0a
            .funcsig @sig__i8 = () -> (@i8)
            .funcdecl @fnc <@fnrsig__i8>
            .funcdef @fnc VERSION @fnc_v1 <@sig__i8> {
                @fnc_v1.blk0():
                    @fnc_v1.blk0.res = SUB <@i8> @0x0a_i8 @0xff_i8
                    RET @fnc_v1.blk0.res
            }
        */
        let vm = VM::new();
        let i8 = vm.declare_type(vm.next_id(), MuType_::int(8));
        vm.set_name(i8.as_entity(), "i8".to_string());
        let c_0xff_i8 = vm.declare_const(vm.next_id(), i8.clone(), Constant::Int(0xff));
        vm.set_name(c_0xff_i8.as_entity(), "0xff_i8".to_string());
        let c_0x0a_i8 = vm.declare_const(vm.next_id(), i8.clone(), Constant::Int(0x0a));
        vm.set_name(c_0x0a_i8.as_entity(), "0x0a_i8".to_string());
        let sig__i8 = vm.declare_func_sig(vm.next_id(), vec![], vec![i8.clone()]);
        vm.set_name(sig__i8.as_entity(), "sig__i8".to_string());
        // .funcdecl @fnc <@fnrsig__i8>
        let fnc = MuFunction::new(vm.next_id(), sig__i8.clone());
        vm.set_name(fnc.as_entity(), "fnc".to_string());
        let id_fnc = fnc.id();
        vm.declare_func(fnc);

        // .funcdef @fnc VERSION @v1 <@sig__i8> {
        let mut fnc_ver = MuFunctionVersion::new(vm.next_id(), id_fnc, sig__i8.clone());
        vm.set_name(fnc_ver.as_entity(), "fnc_v1".to_string());
        // blk0
        let mut blk0 = Block::new(vm.next_id());
        vm.set_name(blk0.as_entity(), "fnc_v1.blk0".to_string());

        // @fnc_v1.blk0.res = SUB <@i8> @0x0a_i8 @0xff_i8
        let lc_0xff_i8 = fnc_ver.new_constant(c_0xff_i8.clone());
        let lc_0x0a_i8 = fnc_ver.new_constant(c_0x0a_i8.clone());
        let res = fnc_ver.new_ssa(vm.next_id(), i8.clone());
        vm.set_name(res.as_entity(), "fnc_v1.blk0.res".to_string());
        let op_add = fnc_ver.new_inst(Instruction{
            hdr: MuEntityHeader::unnamed(vm.next_id()),
            value: Some(vec![res.clone_value()]),
            ops: RwLock::new(vec![lc_0x0a_i8.clone(), lc_0xff_i8.clone()]),
            v: Instruction_::BinOp(BinOp::Sub, 0, 1)
        });

        let op_ret = fnc_ver.new_inst(Instruction{
            hdr: MuEntityHeader::unnamed(vm.next_id()),
            value: None,
            ops: RwLock::new(vec![res.clone()]),
            v: Instruction_::Return(vec![0])
        });
        blk0.content = Some(BlockContent{
            args: vec![],
            exn_arg: None,
            body: vec![op_add, op_ret],
            keepalives: None
        });
        fnc_ver.define(FunctionContent{
            entry: blk0.id(),
            blocks: {
                let mut blocks = HashMap::new();
                blocks.insert(blk0.id(), blk0);
                blocks
            }
        });
        vm.define_func_version(fnc_ver);

        vm
    }
    let lib = compile_fnc("fnc", &build_fn);
    unsafe {
        let fncptr: ll::Symbol<unsafe extern fn () -> u8> = lib.get(b"fnc").unwrap();
        assert!(fncptr() == 0xf5);
    }
}

#[test]
fn test_mul_8bit_wraparound() {
    fn build_fn() -> VM {
        /* Build the following bundle:

            .typedef @i8 = int<8>
            .const @0xff_i8 <@i8> = 0xff
            .const @0x0a_i8 <@i8> = 0x0a
            .funcsig @sig__i8 = () -> (@i8)
            .funcdecl @fnc <@fnrsig__i8>
            .funcdef @fnc VERSION @fnc_v1 <@sig__i8> {
                @fnc_v1.blk0():
                    @fnc_v1.blk0.res = MUL <@i8> @0x0a_i8 @0xff_i8
                    RET @fnc_v1.blk0.res
            }
        */
        let vm = VM::new();
        let i8 = vm.declare_type(vm.next_id(), MuType_::int(8));
        vm.set_name(i8.as_entity(), "i8".to_string());
        let c_0xff_i8 = vm.declare_const(vm.next_id(), i8.clone(), Constant::Int(0xff));
        vm.set_name(c_0xff_i8.as_entity(), "0xff_i8".to_string());
        let c_0x0a_i8 = vm.declare_const(vm.next_id(), i8.clone(), Constant::Int(0x0a));
        vm.set_name(c_0x0a_i8.as_entity(), "0x0a_i8".to_string());
        let sig__i8 = vm.declare_func_sig(vm.next_id(), vec![], vec![i8.clone()]);
        vm.set_name(sig__i8.as_entity(), "sig__i8".to_string());
        // .funcdecl @fnc <@fnrsig__i8>
        let fnc = MuFunction::new(vm.next_id(), sig__i8.clone());
        vm.set_name(fnc.as_entity(), "fnc".to_string());
        let id_fnc = fnc.id();
        vm.declare_func(fnc);

        // .funcdef @fnc VERSION @v1 <@sig__i8> {
        let mut fnc_ver = MuFunctionVersion::new(vm.next_id(), id_fnc, sig__i8.clone());
        vm.set_name(fnc_ver.as_entity(), "fnc_v1".to_string());
        // blk0
        let mut blk0 = Block::new(vm.next_id());
        vm.set_name(blk0.as_entity(), "fnc_v1.blk0".to_string());

        // @fnc_v1.blk0.res = MUL <@i8> @0x0a_i8 @0xff_i8
        let lc_0xff_i8 = fnc_ver.new_constant(c_0xff_i8.clone());
        let lc_0x0a_i8 = fnc_ver.new_constant(c_0x0a_i8.clone());
        let res = fnc_ver.new_ssa(vm.next_id(), i8.clone());
        vm.set_name(res.as_entity(), "fnc_v1.blk0.res".to_string());
        let op_add = fnc_ver.new_inst(Instruction{
            hdr: MuEntityHeader::unnamed(vm.next_id()),
            value: Some(vec![res.clone_value()]),
            ops: RwLock::new(vec![lc_0x0a_i8.clone(), lc_0xff_i8.clone()]),
            v: Instruction_::BinOp(BinOp::Mul, 0, 1)
        });

        let op_ret = fnc_ver.new_inst(Instruction{
            hdr: MuEntityHeader::unnamed(vm.next_id()),
            value: None,
            ops: RwLock::new(vec![res.clone()]),
            v: Instruction_::Return(vec![0])
        });
        blk0.content = Some(BlockContent{
            args: vec![],
            exn_arg: None,
            body: vec![op_add, op_ret],
            keepalives: None
        });
        fnc_ver.define(FunctionContent{
            entry: blk0.id(),
            blocks: {
                let mut blocks = HashMap::new();
                blocks.insert(blk0.id(), blk0);
                blocks
            }
        });
        vm.define_func_version(fnc_ver);

        vm
    }
    let lib = compile_fnc("fnc", &build_fn);
    unsafe {
        let fncptr: ll::Symbol<unsafe extern fn () -> u8> = lib.get(b"fnc").unwrap();
        assert!(fncptr() == 0xf6);
    }
}

#[test]
fn test_sdiv_i8() {
    fn build_fn() -> VM {
        /* Build the following bundle:

            .typedef @i8 = int<8>
            .const @0xff_i8 <@i8> = 0xff
            .const @0x0a_i8 <@i8> = 0x0a
            .funcsig @sig__i8 = () -> (@i8)
            .funcdecl @fnc <@fnrsig__i8>
            .funcdef @fnc VERSION @fnc_v1 <@sig__i8> {
                @fnc_v1.blk0():
                    @fnc_v1.blk0.res = SDIV <@i8> @0xff_i8 @0x0a_i8
                    RET @fnc_v1.blk0.res
            }
        */
        let vm = VM::new();
        let i8 = vm.declare_type(vm.next_id(), MuType_::int(8));
        vm.set_name(i8.as_entity(), "i8".to_string());
        let c_0xff_i8 = vm.declare_const(vm.next_id(), i8.clone(), Constant::Int(0xff));
        vm.set_name(c_0xff_i8.as_entity(), "0xff_i8".to_string());
        let c_0x0a_i8 = vm.declare_const(vm.next_id(), i8.clone(), Constant::Int(0x0a));
        vm.set_name(c_0x0a_i8.as_entity(), "0x0a_i8".to_string());
        let sig__i8 = vm.declare_func_sig(vm.next_id(), vec![], vec![i8.clone()]);
        vm.set_name(sig__i8.as_entity(), "sig__i8".to_string());
        // .funcdecl @fnc <@fnrsig__i8>
        let fnc = MuFunction::new(vm.next_id(), sig__i8.clone());
        vm.set_name(fnc.as_entity(), "fnc".to_string());
        let id_fnc = fnc.id();
        vm.declare_func(fnc);

        // .funcdef @fnc VERSION @v1 <@sig__i8> {
        let mut fnc_ver = MuFunctionVersion::new(vm.next_id(), id_fnc, sig__i8.clone());
        vm.set_name(fnc_ver.as_entity(), "fnc_v1".to_string());
        // blk0
        let mut blk0 = Block::new(vm.next_id());
        vm.set_name(blk0.as_entity(), "fnc_v1.blk0".to_string());

        // @fnc_v1.blk0.res = SDIV <@i8> @0xff_i8 @0x0a_i8
        let lc_0xff_i8 = fnc_ver.new_constant(c_0xff_i8.clone());
        let lc_0x0a_i8 = fnc_ver.new_constant(c_0x0a_i8.clone());
        let res = fnc_ver.new_ssa(vm.next_id(), i8.clone());
        vm.set_name(res.as_entity(), "fnc_v1.blk0.res".to_string());
        let op_add = fnc_ver.new_inst(Instruction{
            hdr: MuEntityHeader::unnamed(vm.next_id()),
            value: Some(vec![res.clone_value()]),
            ops: RwLock::new(vec![lc_0xff_i8.clone(), lc_0x0a_i8.clone()]),
            v: Instruction_::BinOp(BinOp::Sdiv, 0, 1)
        });

        let op_ret = fnc_ver.new_inst(Instruction{
            hdr: MuEntityHeader::unnamed(vm.next_id()),
            value: None,
            ops: RwLock::new(vec![res.clone()]),
            v: Instruction_::Return(vec![0])
        });
        blk0.content = Some(BlockContent{
            args: vec![],
            exn_arg: None,
            body: vec![op_add, op_ret],
            keepalives: None
        });
        fnc_ver.define(FunctionContent{
            entry: blk0.id(),
            blocks: {
                let mut blocks = HashMap::new();
                blocks.insert(blk0.id(), blk0);
                blocks
            }
        });
        vm.define_func_version(fnc_ver);

        vm
    }
    let lib = compile_fnc("fnc", &build_fn);
    unsafe {
        let fncptr: ll::Symbol<unsafe extern fn () -> u8> = lib.get(b"fnc").unwrap();
        assert!(fncptr() == 0xf4);    // -12
    }
}

#[test]
fn test_urem_i8() {
    fn build_fn() -> VM {
        /* Build the following bundle:

            .typedef @i8 = int<8>
            .const @0xff_i8 <@i8> = 0xff
            .const @0x0a_i8 <@i8> = 0x0a
            .funcsig @sig__i8 = () -> (@i8)
            .funcdecl @fnc <@fnrsig__i8>
            .funcdef @fnc VERSION @fnc_v1 <@sig__i8> {
                @fnc_v1.blk0():
                    @fnc_v1.blk0.res = UREM <@i8> @0xff_i8 @0x0a_i8
                    RET @fnc_v1.blk0.res
            }
        */
        let vm = VM::new();
        let i8 = vm.declare_type(vm.next_id(), MuType_::int(8));
        vm.set_name(i8.as_entity(), "i8".to_string());
        let c_0xff_i8 = vm.declare_const(vm.next_id(), i8.clone(), Constant::Int(0xff));
        vm.set_name(c_0xff_i8.as_entity(), "0xff_i8".to_string());
        let c_0x0a_i8 = vm.declare_const(vm.next_id(), i8.clone(), Constant::Int(0x0a));
        vm.set_name(c_0x0a_i8.as_entity(), "0x0a_i8".to_string());
        let sig__i8 = vm.declare_func_sig(vm.next_id(), vec![], vec![i8.clone()]);
        vm.set_name(sig__i8.as_entity(), "sig__i8".to_string());
        // .funcdecl @fnc <@fnrsig__i8>
        let fnc = MuFunction::new(vm.next_id(), sig__i8.clone());
        vm.set_name(fnc.as_entity(), "fnc".to_string());
        let id_fnc = fnc.id();
        vm.declare_func(fnc);

        // .funcdef @fnc VERSION @v1 <@sig__i8> {
        let mut fnc_ver = MuFunctionVersion::new(vm.next_id(), id_fnc, sig__i8.clone());
        vm.set_name(fnc_ver.as_entity(), "fnc_v1".to_string());
        // blk0
        let mut blk0 = Block::new(vm.next_id());
        vm.set_name(blk0.as_entity(), "fnc_v1.blk0".to_string());

        // @fnc_v1.blk0.res = UREM <@i8> @0xff_i8 @0x0a_i8
        let lc_0xff_i8 = fnc_ver.new_constant(c_0xff_i8.clone());
        let lc_0x0a_i8 = fnc_ver.new_constant(c_0x0a_i8.clone());
        let res = fnc_ver.new_ssa(vm.next_id(), i8.clone());
        vm.set_name(res.as_entity(), "fnc_v1.blk0.res".to_string());
        let op_add = fnc_ver.new_inst(Instruction{
            hdr: MuEntityHeader::unnamed(vm.next_id()),
            value: Some(vec![res.clone_value()]),
            ops: RwLock::new(vec![lc_0xff_i8.clone(), lc_0x0a_i8.clone()]),
            v: Instruction_::BinOp(BinOp::Urem, 0, 1)
        });

        let op_ret = fnc_ver.new_inst(Instruction{
            hdr: MuEntityHeader::unnamed(vm.next_id()),
            value: None,
            ops: RwLock::new(vec![res.clone()]),
            v: Instruction_::Return(vec![0])
        });
        blk0.content = Some(BlockContent{
            args: vec![],
            exn_arg: None,
            body: vec![op_add, op_ret],
            keepalives: None
        });
        fnc_ver.define(FunctionContent{
            entry: blk0.id(),
            blocks: {
                let mut blocks = HashMap::new();
                blocks.insert(blk0.id(), blk0);
                blocks
            }
        });
        vm.define_func_version(fnc_ver);

        vm
    }
    let lib = compile_fnc("fnc", &build_fn);
    unsafe {
        let fncptr: ll::Symbol<unsafe extern fn () -> u8> = lib.get(b"fnc").unwrap();
        assert!(fncptr() == 5);
    }
}

#[test]
fn test_shl() {
    fn build_fn() -> VM {
        /* Build the following bundle:

            .typedef @i64 = int<64>
            .const @0x6d9f9c1d58324b55_i64 <@i64> = 0x6d9f9c1d58324b55
            .const @0x0a_i64 <@i64> = 0x0a
            .funcsig @sig__i64 = () -> (@i64)
            .funcdecl @fnc <@fnrsig__i64>
            .funcdef @fnc VERSION @fnc_v1 <@sig__i64> {
                @fnc_v1.blk0():
                    @fnc_v1.blk0.res = SHL <@i64> @0x6d9f9c1d58324b55 @0x0a_i64
                    RET @fnc_v1.blk0.res
            }
        */
        let vm = VM::new();
        let i64 = vm.declare_type(vm.next_id(), MuType_::int(64));
        vm.set_name(i64.as_entity(), "i64".to_string());
        let c_0x6d9f9c1d58324b55_i64 = vm.declare_const(vm.next_id(), i64.clone(),
                                                        Constant::Int(0x6d9f9c1d58324b55));
        vm.set_name(c_0x6d9f9c1d58324b55_i64.as_entity(), "0x6d9f9c1d58324b55_i64".to_string());
        let c_0x0a_i64 = vm.declare_const(vm.next_id(), i64.clone(), Constant::Int(0x0a));
        vm.set_name(c_0x0a_i64.as_entity(), "0x0a_i64".to_string());
        let sig__i64 = vm.declare_func_sig(vm.next_id(), vec![], vec![i64.clone()]);
        vm.set_name(sig__i64.as_entity(), "sig__i64".to_string());
        // .funcdecl @fnc <@fnrsig__i64>
        let fnc = MuFunction::new(vm.next_id(), sig__i64.clone());
        vm.set_name(fnc.as_entity(), "fnc".to_string());
        let id_fnc = fnc.id();
        vm.declare_func(fnc);

        // .funcdef @fnc VERSION @v1 <@sig__i64> {
        let mut fnc_ver = MuFunctionVersion::new(vm.next_id(), id_fnc, sig__i64.clone());
        vm.set_name(fnc_ver.as_entity(), "fnc_v1".to_string());
        // blk0
        let mut blk0 = Block::new(vm.next_id());
        vm.set_name(blk0.as_entity(), "fnc_v1.blk0".to_string());

        // @fnc_v1.blk0.res = SHL <@i64> @0x6d9f9c1d58324b55 @0x0a_i64
        let lc_0x6d9f9c1d58324b55_i64 = fnc_ver.new_constant(c_0x6d9f9c1d58324b55_i64.clone());
        let lc_0x0a_i64 = fnc_ver.new_constant(c_0x0a_i64.clone());
        let res = fnc_ver.new_ssa(vm.next_id(), i64.clone());
        vm.set_name(res.as_entity(), "fnc_v1.blk0.res".to_string());
        let op_add = fnc_ver.new_inst(Instruction{
            hdr: MuEntityHeader::unnamed(vm.next_id()),
            value: Some(vec![res.clone_value()]),
            ops: RwLock::new(vec![lc_0x6d9f9c1d58324b55_i64.clone(), lc_0x0a_i64.clone()]),
            v: Instruction_::BinOp(BinOp::Shl, 0, 1)
        });

        let op_ret = fnc_ver.new_inst(Instruction{
            hdr: MuEntityHeader::unnamed(vm.next_id()),
            value: None,
            ops: RwLock::new(vec![res.clone()]),
            v: Instruction_::Return(vec![0])
        });
        blk0.content = Some(BlockContent{
            args: vec![],
            exn_arg: None,
            body: vec![op_add, op_ret],
            keepalives: None
        });
        fnc_ver.define(FunctionContent{
            entry: blk0.id(),
            blocks: {
                let mut blocks = HashMap::new();
                blocks.insert(blk0.id(), blk0);
                blocks
            }
        });
        vm.define_func_version(fnc_ver);

        vm
    }
    let lib = compile_fnc("fnc", &build_fn);
    unsafe {
        let fncptr: ll::Symbol<unsafe extern fn () -> u64> = lib.get(b"fnc").unwrap();
        assert!(fncptr() == 0x7e707560c92d5400);
    }
}

#[test]
fn test_lshr() {
    fn build_fn() -> VM {
        /* Build the following bundle:

            .typedef @i64 = int<64>
            .const @0x8d9f9c1d58324b55_i64 <@i64> = 0x8d9f9c1d58324b55
            .const @0x0a_i64 <@i64> = 0x0a
            .funcsig @sig__i64 = () -> (@i64)
            .funcdecl @fnc <@fnrsig__i64>
            .funcdef @fnc VERSION @fnc_v1 <@sig__i64> {
                @fnc_v1.blk0():
                    @fnc_v1.blk0.res = LSHR <@i64> @0x8d9f9c1d58324b55 @0x0a_i64
                    RET @fnc_v1.blk0.res
            }
        */
        let vm = VM::new();
        let i64 = vm.declare_type(vm.next_id(), MuType_::int(64));
        vm.set_name(i64.as_entity(), "i64".to_string());
        let c_0x8d9f9c1d58324b55_i64 = vm.declare_const(vm.next_id(), i64.clone(),
                                                        Constant::Int(0x8d9f9c1d58324b55));
        vm.set_name(c_0x8d9f9c1d58324b55_i64.as_entity(), "0x8d9f9c1d58324b55_i64".to_string());
        let c_0x0a_i64 = vm.declare_const(vm.next_id(), i64.clone(), Constant::Int(0x0a));
        vm.set_name(c_0x0a_i64.as_entity(), "0x0a_i64".to_string());
        let sig__i64 = vm.declare_func_sig(vm.next_id(), vec![], vec![i64.clone()]);
        vm.set_name(sig__i64.as_entity(), "sig__i64".to_string());
        // .funcdecl @fnc <@fnrsig__i64>
        let fnc = MuFunction::new(vm.next_id(), sig__i64.clone());
        vm.set_name(fnc.as_entity(), "fnc".to_string());
        let id_fnc = fnc.id();
        vm.declare_func(fnc);

        // .funcdef @fnc VERSION @v1 <@sig__i64> {
        let mut fnc_ver = MuFunctionVersion::new(vm.next_id(), id_fnc, sig__i64.clone());
        vm.set_name(fnc_ver.as_entity(), "fnc_v1".to_string());
        // blk0
        let mut blk0 = Block::new(vm.next_id());
        vm.set_name(blk0.as_entity(), "fnc_v1.blk0".to_string());

        // @fnc_v1.blk0.res = LSHR <@i64> @0x8d9f9c1d58324b55 @0x0a_i64
        let lc_0x8d9f9c1d58324b55_i64 = fnc_ver.new_constant(c_0x8d9f9c1d58324b55_i64.clone());
        let lc_0x0a_i64 = fnc_ver.new_constant(c_0x0a_i64.clone());
        let res = fnc_ver.new_ssa(vm.next_id(), i64.clone());
        vm.set_name(res.as_entity(), "fnc_v1.blk0.res".to_string());
        let op_add = fnc_ver.new_inst(Instruction{
            hdr: MuEntityHeader::unnamed(vm.next_id()),
            value: Some(vec![res.clone_value()]),
            ops: RwLock::new(vec![lc_0x8d9f9c1d58324b55_i64.clone(), lc_0x0a_i64.clone()]),
            v: Instruction_::BinOp(BinOp::Lshr, 0, 1)
        });

        let op_ret = fnc_ver.new_inst(Instruction{
            hdr: MuEntityHeader::unnamed(vm.next_id()),
            value: None,
            ops: RwLock::new(vec![res.clone()]),
            v: Instruction_::Return(vec![0])
        });
        blk0.content = Some(BlockContent{
            args: vec![],
            exn_arg: None,
            body: vec![op_add, op_ret],
            keepalives: None
        });
        fnc_ver.define(FunctionContent{
            entry: blk0.id(),
            blocks: {
                let mut blocks = HashMap::new();
                blocks.insert(blk0.id(), blk0);
                blocks
            }
        });
        vm.define_func_version(fnc_ver);

        vm
    }
    let lib = compile_fnc("fnc", &build_fn);
    unsafe {
        let fncptr: ll::Symbol<unsafe extern fn () -> u64> = lib.get(b"fnc").unwrap();
        assert!(fncptr() == 0x2367E707560C92);
    }
}