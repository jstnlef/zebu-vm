use mu::ast::types::*;
use mu::ast::ir::*;
use mu::ast::ptr::*;
use mu::ast::inst::*;
use mu::vm::*;
use mu::compiler::*;

use std::sync::RwLock;
use std::sync::Arc;
use mu::testutil::aot;
use mu::utils::LinkedHashMap;

#[test]
fn test_ccall_exit() {
    VM::start_logging_trace();

    let vm = Arc::new(ccall_exit());

    let compiler = Compiler::new(CompilerPolicy::default(), &vm);

    let func_id = vm.id_of("ccall_exit");
    {
        let funcs = vm.funcs().read().unwrap();
        let func = funcs.get(&func_id).unwrap().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();
        let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();

        compiler.compile(&mut func_ver);
    }

    vm.make_primordial_thread(func_id, true, vec![]);
    backend::emit_context(&vm);

    let executable = aot::link_primordial(vec!["ccall_exit".to_string()], "ccall_exit_test", &vm);
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

    func_ver.define(FunctionContent::new(
        blk_entry.id(),
        {
            let mut map = LinkedHashMap::new();
            map.insert(blk_entry.id(), blk_entry);
            map
        }
    ));

    vm.define_func_version(func_ver);

    vm
}

#[test]
fn test_pass_1arg_by_stack() {
    VM::start_logging_trace();
    let vm = Arc::new(pass_1arg_by_stack());

    let compiler = Compiler::new(CompilerPolicy::default(), &vm);

    let func_foo = vm.id_of("foo7");
    let func_main = vm.id_of("pass_1arg_by_stack");
    {
        let funcs = vm.funcs().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();

        {
            let func = funcs.get(&func_foo).unwrap().read().unwrap();
            let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();

            compiler.compile(&mut func_ver);
        }
        {
            let func = funcs.get(&func_main).unwrap().read().unwrap();
            let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();

            compiler.compile(&mut func_ver);
        }
    }

    vm.make_primordial_thread(func_main, true, vec![]);
    backend::emit_context(&vm);

    let executable = aot::link_primordial(vec![Mu("foo7"), Mu("pass_1arg_by_stack")], "test_pass_1arg_by_stack", &vm);
    let output = aot::execute_nocheck(executable);

    // exit with (1)
    assert!(output.status.code().is_some());
    assert_eq!(output.status.code().unwrap(), 1);
}

fn pass_1arg_by_stack() -> VM {
    let vm = VM::new_with_opts("init_mu --disable-inline");

    typedef!    ((vm) int64 = mu_int(64));

    constdef!   ((vm) <int64> int64_0 = Constant::Int(0));
    constdef!   ((vm) <int64> int64_1 = Constant::Int(1));
    constdef!   ((vm) <int64> int64_2 = Constant::Int(2));

    // foo7
    funcsig!    ((vm) foo7_sig = (int64, int64, int64, int64, int64, int64, int64) -> (int64));
    funcdecl!   ((vm) <foo7_sig> foo7);
    funcdef!    ((vm) <foo7_sig> foo7 VERSION foo7_v1);

    // blk_entry
    ssa!        ((vm, foo7_v1) <int64> v0);
    ssa!        ((vm, foo7_v1) <int64> v1);
    ssa!        ((vm, foo7_v1) <int64> v2);
    ssa!        ((vm, foo7_v1) <int64> v3);
    ssa!        ((vm, foo7_v1) <int64> v4);
    ssa!        ((vm, foo7_v1) <int64> v5);
    ssa!        ((vm, foo7_v1) <int64> v6);
    block!      ((vm, foo7_v1) blk_entry);

    inst!       ((vm, foo7_v1) blk_entry_ret:
        RET (v6)
    );

    define_block!((vm, foo7_v1) blk_entry(v0, v1, v2, v3, v4, v5, v6) {
        blk_entry_ret
    });

    define_func_ver!((vm) foo7_v1 (entry: blk_entry) {blk_entry});

    // pass_1arg_by_stack
    funcsig!    ((vm) sig = () -> ());
    funcdecl!   ((vm) <sig> pass_1arg_by_stack);
    funcdef!    ((vm) <sig> pass_1arg_by_stack VERSION pass_1arg_by_stack_v1);

    typedef!    ((vm) type_funcref_foo7 = mu_funcref(foo7_sig));
    constdef!   ((vm) <type_funcref_foo7> const_funcref_foo7 = Constant::FuncRef(vm.id_of("foo7")));

    // blk_entry
    consta!     ((vm, pass_1arg_by_stack_v1) int64_0_local = int64_0);
    consta!     ((vm, pass_1arg_by_stack_v1) int64_1_local = int64_1);

    block!      ((vm, pass_1arg_by_stack_v1) blk_entry);
    block!      ((vm, pass_1arg_by_stack_v1) blk_main);
    inst!       ((vm, pass_1arg_by_stack_v1) blk_entry_branch:
        BRANCH blk_main (
            int64_0_local,
            int64_0_local,
            int64_0_local,
            int64_0_local,
            int64_0_local,
            int64_0_local,

            int64_1_local
        )
    );

    define_block!((vm, pass_1arg_by_stack_v1) blk_entry() {blk_entry_branch});

    // blk_main
    ssa!        ((vm, pass_1arg_by_stack_v1) <int64> a0);
    ssa!        ((vm, pass_1arg_by_stack_v1) <int64> a1);
    ssa!        ((vm, pass_1arg_by_stack_v1) <int64> a2);
    ssa!        ((vm, pass_1arg_by_stack_v1) <int64> a3);
    ssa!        ((vm, pass_1arg_by_stack_v1) <int64> a4);
    ssa!        ((vm, pass_1arg_by_stack_v1) <int64> a5);
    ssa!        ((vm, pass_1arg_by_stack_v1) <int64> a6);

    consta!     ((vm, pass_1arg_by_stack_v1) const_funcref_foo7_local = const_funcref_foo7);
    ssa!        ((vm, pass_1arg_by_stack_v1) <int64> retval);
    inst!       ((vm, pass_1arg_by_stack_v1) blk_main_call:
        retval = EXPRCALL (CallConvention::Mu, is_abort: false) const_funcref_foo7_local (a0, a1, a2, a3, a4, a5, a6)
    );

    let blk_main_exit = gen_ccall_exit(retval.clone(), &mut pass_1arg_by_stack_v1, &vm);

    inst!       ((vm, pass_1arg_by_stack_v1) blk_main_ret:
        RET
    );

    define_block!((vm, pass_1arg_by_stack_v1) blk_main(a0, a1, a2, a3, a4, a5, a6) {
        blk_main_call,
        blk_main_exit,
        blk_main_ret
    });

    define_func_ver!((vm) pass_1arg_by_stack_v1 (entry: blk_entry) {
        blk_entry,
        blk_main
    });

    vm
}

#[test]
fn test_pass_2args_by_stack() {
    VM::start_logging_trace();
    let vm = Arc::new(pass_2args_by_stack());

    let compiler = Compiler::new(CompilerPolicy::default(), &vm);

    let func_foo = vm.id_of("foo8");
    let func_main = vm.id_of("pass_2args_by_stack");
    {
        let funcs = vm.funcs().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();

        {
            let func = funcs.get(&func_foo).unwrap().read().unwrap();
            let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();

            compiler.compile(&mut func_ver);
        }
        {
            let func = funcs.get(&func_main).unwrap().read().unwrap();
            let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();

            compiler.compile(&mut func_ver);
        }
    }

    vm.make_primordial_thread(func_main, true, vec![]);
    backend::emit_context(&vm);

    let executable = aot::link_primordial(vec![Mu("foo8"), Mu("pass_2args_by_stack")], "test_pass_2args_by_stack", &vm);
    let output = aot::execute_nocheck(executable);

    // exit with (2)
    assert!(output.status.code().is_some());
    assert_eq!(output.status.code().unwrap(), 2);
}

fn pass_2args_by_stack() -> VM {
    let vm = VM::new_with_opts("init_mu --disable-inline");

    typedef!    ((vm) int64 = mu_int(64));

    constdef!   ((vm) <int64> int64_0 = Constant::Int(0));
    constdef!   ((vm) <int64> int64_1 = Constant::Int(1));
    constdef!   ((vm) <int64> int64_2 = Constant::Int(2));

    // foo8
    funcsig!    ((vm) foo8_sig = (int64, int64, int64, int64, int64, int64, int64, int64) -> (int64));
    funcdecl!   ((vm) <foo8_sig> foo8);
    funcdef!    ((vm) <foo8_sig> foo8 VERSION foo8_v1);

    // blk_entry
    ssa!        ((vm, foo8_v1) <int64> v0);
    ssa!        ((vm, foo8_v1) <int64> v1);
    ssa!        ((vm, foo8_v1) <int64> v2);
    ssa!        ((vm, foo8_v1) <int64> v3);
    ssa!        ((vm, foo8_v1) <int64> v4);
    ssa!        ((vm, foo8_v1) <int64> v5);
    ssa!        ((vm, foo8_v1) <int64> v6);
    ssa!        ((vm, foo8_v1) <int64> v7);
    block!      ((vm, foo8_v1) blk_entry);

    inst!       ((vm, foo8_v1) blk_entry_ret:
        RET (v7)
    );

    define_block!((vm, foo8_v1) blk_entry(v0, v1, v2, v3, v4, v5, v6, v7) {
        blk_entry_ret
    });

    define_func_ver!((vm) foo8_v1 (entry: blk_entry) {blk_entry});

    // pass_2args_by_stack
    funcsig!    ((vm) sig = () -> ());
    funcdecl!   ((vm) <sig> pass_2args_by_stack);
    funcdef!    ((vm) <sig> pass_2args_by_stack VERSION pass_2args_by_stack_v1);

    typedef!    ((vm) type_funcref_foo8 = mu_funcref(foo8_sig));
    constdef!   ((vm) <type_funcref_foo8> const_funcref_foo8 = Constant::FuncRef(vm.id_of("foo8")));

    // blk_entry
    consta!     ((vm, pass_2args_by_stack_v1) int64_0_local = int64_0);
    consta!     ((vm, pass_2args_by_stack_v1) int64_1_local = int64_1);
    consta!     ((vm, pass_2args_by_stack_v1) int64_2_local = int64_2);

    block!      ((vm, pass_2args_by_stack_v1) blk_entry);
    block!      ((vm, pass_2args_by_stack_v1) blk_main);
    inst!       ((vm, pass_2args_by_stack_v1) blk_entry_branch:
        BRANCH blk_main (
            int64_0_local,
            int64_0_local,
            int64_0_local,
            int64_0_local,
            int64_0_local,
            int64_0_local,

            int64_1_local,
            int64_2_local
        )
    );

    define_block!((vm, pass_2args_by_stack_v1) blk_entry() {blk_entry_branch});

    // blk_main
    ssa!        ((vm, pass_2args_by_stack_v1) <int64> a0);
    ssa!        ((vm, pass_2args_by_stack_v1) <int64> a1);
    ssa!        ((vm, pass_2args_by_stack_v1) <int64> a2);
    ssa!        ((vm, pass_2args_by_stack_v1) <int64> a3);
    ssa!        ((vm, pass_2args_by_stack_v1) <int64> a4);
    ssa!        ((vm, pass_2args_by_stack_v1) <int64> a5);
    ssa!        ((vm, pass_2args_by_stack_v1) <int64> a6);
    ssa!        ((vm, pass_2args_by_stack_v1) <int64> a7);

    consta!     ((vm, pass_2args_by_stack_v1) const_funcref_foo8_local = const_funcref_foo8);
    ssa!        ((vm, pass_2args_by_stack_v1) <int64> retval);
    inst!       ((vm, pass_2args_by_stack_v1) blk_main_call:
        retval = EXPRCALL (CallConvention::Mu, is_abort: false) const_funcref_foo8_local (a0, a1, a2, a3, a4, a5, a6, a7)
    );

    let blk_main_exit = gen_ccall_exit(retval.clone(), &mut pass_2args_by_stack_v1, &vm);

    inst!       ((vm, pass_2args_by_stack_v1) blk_main_ret:
        RET
    );

    define_block!((vm, pass_2args_by_stack_v1) blk_main(a0, a1, a2, a3, a4, a5, a6, a7) {
        blk_main_call,
        blk_main_exit,
        blk_main_ret
    });

    define_func_ver!((vm) pass_2args_by_stack_v1 (entry: blk_entry) {
        blk_entry,
        blk_main
    });

    vm
}