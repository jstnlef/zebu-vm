extern crate mu;
extern crate log;
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
fn test_fp_add() {
    VM::start_logging_trace();

    let vm = Arc::new(fp_add());

    let compiler = Compiler::new(CompilerPolicy::default(), vm.clone());

    let func_id = vm.id_of("fp_add");

    {
        let funcs = vm.funcs().read().unwrap();
        let func = funcs.get(&func_id).unwrap().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();
        let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();

        compiler.compile(&mut func_ver);
    }

    backend::emit_context(&vm);

    let dylib = aot::link_dylib(vec![Mu("fp_add")], "libfp_add.dylib");

    let lib = libloading::Library::new(dylib.as_os_str()).unwrap();
    unsafe {
        let fp_add : libloading::Symbol<unsafe extern fn(f64, f64) -> f64> = lib.get(b"fp_add").unwrap();

        let fp_add_1_1 = fp_add(1f64, 1f64);
        println!("fp_add(1, 1) = {}", fp_add_1_1);
        assert!(fp_add_1_1 == 2f64);
    }
}

fn fp_add() -> VM {
    let vm = VM::new();

    // .typedef @double = double
    let type_def_double = vm.declare_type(vm.next_id(), MuType_::double());
    vm.set_name(type_def_double.as_entity(), Mu("double"));

    // .funcsig @fp_add_sig = (@double @double) -> (@double)
    let fp_add_sig = vm.declare_func_sig(vm.next_id(), vec![type_def_double.clone()], vec![type_def_double.clone(), type_def_double.clone()]);
    vm.set_name(fp_add_sig.as_entity(), Mu("fp_add_sig"));

    // .funcdecl @fp_add <@fp_add_sig>
    let func_id = vm.next_id();
    let func = MuFunction::new(func_id, fp_add_sig.clone());
    vm.set_name(func.as_entity(), Mu("fp_add"));
    vm.declare_func(func);

    // .funcdef @fp_add VERSION @fp_add_v1 <@fp_add_sig>
    let mut func_ver = MuFunctionVersion::new(vm.next_id(), func_id, fp_add_sig.clone());
    vm.set_name(func_ver.as_entity(), Mu("fp_add_v1"));

    // %entry(<@double> %a, <@double> %b):
    let mut blk_entry = Block::new(vm.next_id());
    vm.set_name(blk_entry.as_entity(), Mu("entry"));

    let blk_entry_a = func_ver.new_ssa(vm.next_id(), type_def_double.clone());
    vm.set_name(blk_entry_a.as_entity(), Mu("blk_entry_a"));
    let blk_entry_b = func_ver.new_ssa(vm.next_id(), type_def_double.clone());
    vm.set_name(blk_entry_b.as_entity(), Mu("blk_entry_b"));

    // %r = FADD %a %b
    let blk_entry_r = func_ver.new_ssa(vm.next_id(), type_def_double.clone());
    vm.set_name(blk_entry_r.as_entity(), Mu("blk_entry_r"));
    let blk_entry_add = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_entry_r.clone_value()]),
        ops: RwLock::new(vec![blk_entry_a.clone(), blk_entry_b.clone()]),
        v: Instruction_::BinOp(BinOp::FAdd, 0, 1)
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