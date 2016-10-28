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
use mu::testutil::aot;

#[test]
fn test_u8_add() {
    simple_logger::init_with_level(log::LogLevel::Trace).ok();

    let vm = Arc::new(u8_add());

    let compiler = Compiler::new(CompilerPolicy::default(), vm.clone());

    let func_id = vm.id_of("u8_add");

    {
        let funcs = vm.funcs().read().unwrap();
        let func = funcs.get(&func_id).unwrap().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();
        let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();

        compiler.compile(&mut func_ver);
    }

    backend::emit_context(&vm);

    let dylib = aot::link_dylib(vec![Mu("u8_add")], "libu8_add.dylib");

    let lib = libloading::Library::new(dylib.as_os_str()).unwrap();
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