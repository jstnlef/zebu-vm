extern crate mu;
extern crate log;
extern crate simple_logger;
extern crate libloading as ll;

use test_ir::test_ir::sum;
use test_ir::test_ir::factorial;
use self::mu::ast::ir::*;
use self::mu::ast::op::*;
use self::mu::ast::inst::*;
use self::mu::ast::types::*;
use self::mu::vm::*;

use std::sync::RwLock;
use std::collections::HashMap;

use testutil;

#[test]
fn test_factorial() {
    let lib = testutil::compile_fnc("fac", &factorial);
    unsafe {
        let fac: ll::Symbol<unsafe extern fn (u64) -> u64> = lib.get(b"fac").unwrap();
        println!("fac(10) = {}", fac(10));
        assert!(fac(10) == 3628800);
    }
}

#[test]
fn test_sum() {
    let lib = testutil::compile_fnc("sum", &sum);
    unsafe {
        let sumptr: ll::Symbol<unsafe extern fn (u64) -> u64> = lib.get(b"sum").unwrap();
        assert!(sumptr(5) == 10);
        assert!(sumptr(10) == 45);
    }
}

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
        let lc_0xff_i8 = fnc_ver.new_constant(vm.next_id(), c_0xff_i8.clone());
        let lc_0x0a_i8 = fnc_ver.new_constant(vm.next_id(), c_0x0a_i8.clone());
        let res = fnc_ver.new_ssa(vm.next_id(), i8.clone());
        vm.set_name(res.as_entity(), "fnc_v1.blk0.res".to_string());
        let op_add = fnc_ver.new_inst(vm.next_id(), Instruction{
            value: Some(vec![res.clone_value()]),
            ops: RwLock::new(vec![lc_0xff_i8.clone(), lc_0x0a_i8.clone()]),
            v: Instruction_::BinOp(BinOp::Add, 0, 1)
        });

        let op_ret = fnc_ver.new_inst(vm.next_id(), Instruction{
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
    let lib = testutil::compile_fnc("fnc", &build_fn);
    unsafe {
        let fncptr: ll::Symbol<unsafe extern fn () -> u8> = lib.get(b"fnc").unwrap();
        assert!(fncptr() == 9);
    }
}