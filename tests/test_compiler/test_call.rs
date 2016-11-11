use mu::ast::types::*;
use mu::ast::ir::*;
use mu::ast::ptr::*;
use mu::ast::inst::*;
use mu::vm::*;
use mu::compiler::*;

use std::sync::RwLock;
use std::sync::Arc;
use mu::testutil::aot;

#[test]
fn test_ccall_exit() {
    VM::start_logging_trace();

    let vm = Arc::new(ccall_exit());

    let compiler = Compiler::new(CompilerPolicy::default(), vm.clone());

    let func_id = vm.id_of("ccall_exit");
    {
        let funcs = vm.funcs().read().unwrap();
        let func = funcs.get(&func_id).unwrap().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();
        let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();

        compiler.compile(&mut func_ver);
    }

    vm.make_primordial_thread(func_id, vec![]);
    backend::emit_context(&vm);

    let executable = aot::link_primordial(vec!["ccall_exit".to_string()], "ccall_exit_test");
    let output = aot::execute_nocheck(executable);

    assert!(output.status.code().is_some());

    let ret_code = output.status.code().unwrap();
    println!("return code: {}", ret_code);
    assert!(ret_code == 10);
}

pub fn gen_ccall_exit(arg: P<TreeNode>, func_ver: &mut MuFunctionVersion, vm: &VM) -> Box<TreeNode> {
    // .typedef @int32 = int<32>
    let type_def_int32 = vm.declare_type(vm.next_id(), MuType_::int(32));
    vm.set_name(type_def_int32.as_entity(), Mu("exit_int32"));

    // .typedef @exit_sig = (@int32) -> !
    let exit_sig = vm.declare_func_sig(vm.next_id(), vec![], vec![type_def_int32.clone()]);
    vm.set_name(exit_sig.as_entity(), Mu("exit_sig"));

    // .typedef @ufp_exit = ufuncptr(@exit_sig)
    let type_def_ufp_exit = vm.declare_type(vm.next_id(), MuType_::UFuncPtr(exit_sig.clone()));
    vm.set_name(type_def_ufp_exit.as_entity(), Mu("ufp_exit"));

    // .const @exit = EXTERN SYMBOL "exit"
    let const_exit = vm.declare_const(vm.next_id(), type_def_ufp_exit.clone(), Constant::ExternSym(C("exit")));
    vm.set_name(const_exit.as_entity(), Mu("exit"));

    // exprCCALL %const_exit (%const_int32_10)
    let const_exit_local = func_ver.new_constant(const_exit.clone());

    func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: None,
        ops: RwLock::new(vec![const_exit_local, arg]),
        v: Instruction_::ExprCCall {
            data: CallData {
                func: 0,
                args: vec![1],
                convention: CallConvention::Foreign(ForeignFFI::C)
            },
            is_abort: false
        }
    })
}

fn ccall_exit() -> VM {
    let vm = VM::new();

    // .typedef @int32 = int<32>
    let type_def_int32 = vm.declare_type(vm.next_id(), MuType_::int(32));
    vm.set_name(type_def_int32.as_entity(), Mu("int32"));

    // .const @int32_10 = 10
    let const_int32_10 = vm.declare_const(vm.next_id(), type_def_int32.clone(), Constant::Int(10));
    vm.set_name(const_int32_10.as_entity(), Mu("const_int32_10"));

    // .const @int32_0 = 0
    let const_int32_0 = vm.declare_const(vm.next_id(), type_def_int32.clone(), Constant::Int(0));
    vm.set_name(const_int32_0.as_entity(), Mu("const_int32_0"));

    // .funcsig @ccall_exit_sig = () -> !
    let ccall_exit_sig = vm.declare_func_sig(vm.next_id(), vec![], vec![]);
    vm.set_name(ccall_exit_sig.as_entity(), Mu("ccall_exit_sig"));

    // .funcdecl @ccall_exit <@ccall_exit_sig>
    let func_id = vm.next_id();
    let func = MuFunction::new(func_id, ccall_exit_sig.clone());
    vm.set_name(func.as_entity(), Mu("ccall_exit"));
    vm.declare_func(func);

    // .funcdef @ccall_exit VERSION @ccall_exit_v1 <@ccall_exit_sig>
    let mut func_ver = MuFunctionVersion::new(vm.next_id(), func_id, ccall_exit_sig.clone());
    vm.set_name(func_ver.as_entity(), Mu("ccall_exit_v1"));

    // %entry():
    let mut blk_entry = Block::new(vm.next_id());
    vm.set_name(blk_entry.as_entity(), Mu("entry"));

    // exprCCALL %const_exit (%const_int32_10)
    let const_int32_10_local = func_ver.new_constant(const_int32_10.clone());
    let blk_entry_ccall = gen_ccall_exit(const_int32_10_local.clone(), &mut func_ver, &vm);

    // RET %const_int32_0
    let const_int32_0_local = func_ver.new_constant(const_int32_0.clone());

    let blk_entry_ret = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: None,
        ops: RwLock::new(vec![const_int32_0_local]),
        v: Instruction_::Return(vec![0])
    });

    blk_entry.content = Some(BlockContent {
        args: vec![],
        exn_arg: None,
        body: vec![blk_entry_ccall, blk_entry_ret],
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