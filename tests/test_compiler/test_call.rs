// Copyright 2017 The Australian National University
// 
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// 
//     http://www.apache.org/licenses/LICENSE-2.0
// 
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate libloading;

use mu::ast::types::*;
use mu::ast::ir::*;
use mu::ast::ptr::*;
use mu::ast::inst::*;
use mu::ast::op::*;
use mu::vm::*;
use mu::compiler::*;

use std::sync::Arc;
use mu::testutil;
use mu::testutil::aot;
use mu::utils::LinkedHashMap;

// NOTE: aarch64 has 2 more parameter registers than x86-64 so it needs different test cases for stack arguments

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
    typedef!    ((vm) int64 = mu_int(64));
    funcsig!    ((vm) exit_sig = (int64) -> ());
    typedef!    ((vm) ufp_exit = mu_ufuncptr(exit_sig));

    // .const @exit = EXTERN SYMBOL "exit"
    constdef!   ((vm) <ufp_exit> const_exit = Constant::ExternSym(C ("exit")));
    consta!     ((vm, func_ver) const_exit_local = const_exit);

    inst!       ((vm, func_ver) ret:
        EXPRCCALL (CallConvention::Foreign(ForeignFFI::C), is_abort: false) const_exit_local (arg)
    );

    ret
}

fn ccall_exit() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int32 = mu_int(32));

    constdef!   ((vm) <int32> int32_10 = Constant::Int(10));
    constdef!   ((vm) <int32> int32_0  = Constant::Int(0));

    funcsig!    ((vm) ccall_exit_sig = () -> ());
    funcdecl!   ((vm) <ccall_exit_sig> ccall_exit);
    funcdef!    ((vm) <ccall_exit_sig> ccall_exit VERSION ccall_exit_v1);

    // %entry():
    block!      ((vm, ccall_exit_v1) blk_entry);

    // exprCCALL %const_exit (%const_int32_10)
    consta!     ((vm, ccall_exit_v1) int32_10_local = int32_10);
    let blk_entry_ccall = gen_ccall_exit(int32_10_local.clone(), &mut ccall_exit_v1, &vm);

    // RET %const_int32_0
    consta!     ((vm, ccall_exit_v1) int32_0_local = int32_0);
    inst!       ((vm, ccall_exit_v1) blk_entry_ret:
        RET (int32_0_local)
    );

    define_block!((vm, ccall_exit_v1) blk_entry() {
        blk_entry_ccall,
        blk_entry_ret
    });

    define_func_ver!((vm) ccall_exit_v1 (entry: blk_entry) {
        blk_entry
    });

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

#[cfg(target_arch = "aarch64")]
// aarch64 has 2 more parameter registers than x86-64 so we need to modify the test
fn pass_1arg_by_stack() -> VM {
    let vm = VM::new_with_opts("init_mu --disable-inline");

    typedef!    ((vm) int64 = mu_int(64));

    constdef!   ((vm) <int64> int64_0 = Constant::Int(0));
    constdef!   ((vm) <int64> int64_1 = Constant::Int(1));
    constdef!   ((vm) <int64> int64_2 = Constant::Int(2));

    // foo7
    funcsig!    ((vm) foo7_sig = (int64, int64, int64, int64, int64, int64, int64, int64, int64) -> (int64));
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
    ssa!        ((vm, foo7_v1) <int64> v7);
    ssa!        ((vm, foo7_v1) <int64> v8);
    block!      ((vm, foo7_v1) blk_entry);

    inst!       ((vm, foo7_v1) blk_entry_ret:
        RET (v8)
    );

    define_block!((vm, foo7_v1) blk_entry(v0, v1, v2, v3, v4, v5, v6, v7, v8) {
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
    ssa!        ((vm, pass_1arg_by_stack_v1) <int64> a7);
    ssa!        ((vm, pass_1arg_by_stack_v1) <int64> a8);

    consta!     ((vm, pass_1arg_by_stack_v1) const_funcref_foo7_local = const_funcref_foo7);
    ssa!        ((vm, pass_1arg_by_stack_v1) <int64> retval);
    inst!       ((vm, pass_1arg_by_stack_v1) blk_main_call:
        retval = EXPRCALL (CallConvention::Mu, is_abort: false) const_funcref_foo7_local (a0, a1, a2, a3, a4, a5, a6, a7, a8)
    );

    let blk_main_exit = gen_ccall_exit(retval.clone(), &mut pass_1arg_by_stack_v1, &vm);

    inst!       ((vm, pass_1arg_by_stack_v1) blk_main_ret:
        RET
    );

    define_block!((vm, pass_1arg_by_stack_v1) blk_main(a0, a1, a2, a3, a4, a5, a6, a7, a8) {
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

#[cfg(target_arch = "x86_64")]
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

#[cfg(target_arch = "aarch64")]
// aarch64 has 2 more parameter registers than x86-64 so we need to modify the test
fn pass_2args_by_stack() -> VM {
    let vm = VM::new_with_opts("init_mu --disable-inline");

    typedef!    ((vm) int64 = mu_int(64));

    constdef!   ((vm) <int64> int64_0 = Constant::Int(0));
    constdef!   ((vm) <int64> int64_1 = Constant::Int(1));
    constdef!   ((vm) <int64> int64_2 = Constant::Int(2));

    // foo8
    funcsig!    ((vm) foo8_sig = (int64, int64, int64, int64, int64, int64, int64, int64, int64, int64) -> (int64));
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
    ssa!        ((vm, foo8_v1) <int64> v8);
    ssa!        ((vm, foo8_v1) <int64> v9);
    block!      ((vm, foo8_v1) blk_entry);

    inst!       ((vm, foo8_v1) blk_entry_ret:
        RET (v9)
    );

    define_block!((vm, foo8_v1) blk_entry(v0, v1, v2, v3, v4, v5, v6, v7, v8, v9) {
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
    ssa!        ((vm, pass_2args_by_stack_v1) <int64> a8);
    ssa!        ((vm, pass_2args_by_stack_v1) <int64> a9);

    consta!     ((vm, pass_2args_by_stack_v1) const_funcref_foo8_local = const_funcref_foo8);
    ssa!        ((vm, pass_2args_by_stack_v1) <int64> retval);
    inst!       ((vm, pass_2args_by_stack_v1) blk_main_call:
        retval = EXPRCALL (CallConvention::Mu, is_abort: false) const_funcref_foo8_local (a0, a1, a2, a3, a4, a5, a6, a7, a8, a9)
    );

    let blk_main_exit = gen_ccall_exit(retval.clone(), &mut pass_2args_by_stack_v1, &vm);

    inst!       ((vm, pass_2args_by_stack_v1) blk_main_ret:
        RET
    );

    define_block!((vm, pass_2args_by_stack_v1) blk_main(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9) {
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

#[cfg(target_arch = "x86_64")]
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

#[test]
fn test_pass_2_int8_args_by_stack() {
    VM::start_logging_trace();
    let vm = Arc::new(pass_2_int8_args_by_stack());

    let compiler = Compiler::new(CompilerPolicy::default(), &vm);

    let func_foo = vm.id_of("foo8");
    let func_main = vm.id_of("pass_2_int8_args_by_stack");
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

    let executable = aot::link_primordial(vec![Mu("foo8"), Mu("pass_2_int8_args_by_stack")], "test_pass_2_int8_args_by_stack", &vm);
    let output = aot::execute_nocheck(executable);

    // exit with (2)
    assert!(output.status.code().is_some());
    assert_eq!(output.status.code().unwrap(), 2);
}

#[cfg(target_arch = "aarch64")]
fn pass_2_int8_args_by_stack() -> VM {
    let vm = VM::new_with_opts("init_mu --disable-inline");

    typedef!    ((vm) int64 = mu_int(64));
    typedef!    ((vm) int8  = mu_int(8));

    constdef!   ((vm) <int64> int64_0 = Constant::Int(0));
    constdef!   ((vm) <int64> int64_1 = Constant::Int(1));
    constdef!   ((vm) <int64> int64_2 = Constant::Int(2));
    constdef!   ((vm) <int8>  int8_1 = Constant::Int(1));
    constdef!   ((vm) <int8>  int8_2 = Constant::Int(2));

    // foo8
    funcsig!    ((vm) foo8_sig = (int64, int64, int64, int64, int64, int64, int64, int64, int8, int8) -> (int64));
    funcdecl!   ((vm) <foo8_sig> foo8);
    funcdef!    ((vm) <foo8_sig> foo8 VERSION foo8_v1);

    // blk_entry
    ssa!        ((vm, foo8_v1) <int64> v0);
    ssa!        ((vm, foo8_v1) <int64> v1);
    ssa!        ((vm, foo8_v1) <int64> v2);
    ssa!        ((vm, foo8_v1) <int64> v3);
    ssa!        ((vm, foo8_v1) <int64> v4);
    ssa!        ((vm, foo8_v1) <int64> v5);
    ssa!        ((vm, foo8_v1) <int8> v6);
    ssa!        ((vm, foo8_v1) <int8> v7);
    ssa!        ((vm, foo8_v1) <int8> v8);
    ssa!        ((vm, foo8_v1) <int8> v9);
    block!      ((vm, foo8_v1) blk_entry);

    ssa!        ((vm, foo8_v1) <int64> res);
    inst!       ((vm, foo8_v1) blk_entry_zext:
        res = CONVOP (ConvOp::ZEXT) <int8 int64> v9
    );

    inst!       ((vm, foo8_v1) blk_entry_ret:
        RET (res)
    );

    define_block!((vm, foo8_v1) blk_entry(v0, v1, v2, v3, v4, v5, v6, v7, v8, v9) {
        blk_entry_zext,
        blk_entry_ret
    });

    define_func_ver!((vm) foo8_v1 (entry: blk_entry) {blk_entry});

    // pass_2_int8_args_by_stack
    funcsig!    ((vm) sig = () -> ());
    funcdecl!   ((vm) <sig> pass_2_int8_args_by_stack);
    funcdef!    ((vm) <sig> pass_2_int8_args_by_stack VERSION pass_2_int8_args_by_stack_v1);

    typedef!    ((vm) type_funcref_foo8 = mu_funcref(foo8_sig));
    constdef!   ((vm) <type_funcref_foo8> const_funcref_foo8 = Constant::FuncRef(vm.id_of("foo8")));

    // blk_entry
    consta!     ((vm, pass_2_int8_args_by_stack_v1) int64_0_local = int64_0);
    consta!     ((vm, pass_2_int8_args_by_stack_v1) int8_1_local  = int8_1);
    consta!     ((vm, pass_2_int8_args_by_stack_v1) int8_2_local  = int8_2);

    block!      ((vm, pass_2_int8_args_by_stack_v1) blk_entry);
    block!      ((vm, pass_2_int8_args_by_stack_v1) blk_main);
    inst!       ((vm, pass_2_int8_args_by_stack_v1) blk_entry_branch:
        BRANCH blk_main (
            int64_0_local,
            int64_0_local,
            int64_0_local,
            int64_0_local,
            int64_0_local,
            int64_0_local,
            int64_0_local,
            int64_0_local,

            int8_1_local,
            int8_2_local
        )
    );

    define_block!((vm, pass_2_int8_args_by_stack_v1) blk_entry() {blk_entry_branch});

    // blk_main
    ssa!        ((vm, pass_2_int8_args_by_stack_v1) <int64> a0);
    ssa!        ((vm, pass_2_int8_args_by_stack_v1) <int64> a1);
    ssa!        ((vm, pass_2_int8_args_by_stack_v1) <int64> a2);
    ssa!        ((vm, pass_2_int8_args_by_stack_v1) <int64> a3);
    ssa!        ((vm, pass_2_int8_args_by_stack_v1) <int64> a4);
    ssa!        ((vm, pass_2_int8_args_by_stack_v1) <int64> a5);
    ssa!        ((vm, pass_2_int8_args_by_stack_v1) <int64> a6);
    ssa!        ((vm, pass_2_int8_args_by_stack_v1) <int64> a7);
    ssa!        ((vm, pass_2_int8_args_by_stack_v1) <int8> a8);
    ssa!        ((vm, pass_2_int8_args_by_stack_v1) <int8> a9);

    consta!     ((vm, pass_2_int8_args_by_stack_v1) const_funcref_foo8_local = const_funcref_foo8);
    ssa!        ((vm, pass_2_int8_args_by_stack_v1) <int64> retval);
    inst!       ((vm, pass_2_int8_args_by_stack_v1) blk_main_call:
        retval = EXPRCALL (CallConvention::Mu, is_abort: false) const_funcref_foo8_local (a0, a1, a2, a3, a4, a5, a6, a7, a8, a9)
    );

    let blk_main_exit = gen_ccall_exit(retval.clone(), &mut pass_2_int8_args_by_stack_v1, &vm);

    inst!       ((vm, pass_2_int8_args_by_stack_v1) blk_main_ret:
        RET
    );

    define_block!((vm, pass_2_int8_args_by_stack_v1) blk_main(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9) {
        blk_main_call,
        blk_main_exit,
        blk_main_ret
    });

    define_func_ver!((vm) pass_2_int8_args_by_stack_v1 (entry: blk_entry) {
        blk_entry,
        blk_main
    });

    vm
}

#[cfg(target_arch = "x86_64")]
fn pass_2_int8_args_by_stack() -> VM {
    let vm = VM::new_with_opts("init_mu --disable-inline");

    typedef!    ((vm) int64 = mu_int(64));
    typedef!    ((vm) int8  = mu_int(8));

    constdef!   ((vm) <int64> int64_0 = Constant::Int(0));
    constdef!   ((vm) <int64> int64_1 = Constant::Int(1));
    constdef!   ((vm) <int64> int64_2 = Constant::Int(2));
    constdef!   ((vm) <int8>  int8_1 = Constant::Int(1));
    constdef!   ((vm) <int8>  int8_2 = Constant::Int(2));

    // foo8
    funcsig!    ((vm) foo8_sig = (int64, int64, int64, int64, int64, int64, int8, int8) -> (int64));
    funcdecl!   ((vm) <foo8_sig> foo8);
    funcdef!    ((vm) <foo8_sig> foo8 VERSION foo8_v1);

    // blk_entry
    ssa!        ((vm, foo8_v1) <int64> v0);
    ssa!        ((vm, foo8_v1) <int64> v1);
    ssa!        ((vm, foo8_v1) <int64> v2);
    ssa!        ((vm, foo8_v1) <int64> v3);
    ssa!        ((vm, foo8_v1) <int64> v4);
    ssa!        ((vm, foo8_v1) <int64> v5);
    ssa!        ((vm, foo8_v1) <int8> v6);
    ssa!        ((vm, foo8_v1) <int8> v7);
    block!      ((vm, foo8_v1) blk_entry);

    ssa!        ((vm, foo8_v1) <int64> res);
    inst!       ((vm, foo8_v1) blk_entry_zext:
        res = CONVOP (ConvOp::ZEXT) <int8 int64> v7
    );

    inst!       ((vm, foo8_v1) blk_entry_ret:
        RET (res)
    );

    define_block!((vm, foo8_v1) blk_entry(v0, v1, v2, v3, v4, v5, v6, v7) {
        blk_entry_zext,
        blk_entry_ret
    });

    define_func_ver!((vm) foo8_v1 (entry: blk_entry) {blk_entry});

    // pass_2_int8_args_by_stack
    funcsig!    ((vm) sig = () -> ());
    funcdecl!   ((vm) <sig> pass_2_int8_args_by_stack);
    funcdef!    ((vm) <sig> pass_2_int8_args_by_stack VERSION pass_2_int8_args_by_stack_v1);

    typedef!    ((vm) type_funcref_foo8 = mu_funcref(foo8_sig));
    constdef!   ((vm) <type_funcref_foo8> const_funcref_foo8 = Constant::FuncRef(vm.id_of("foo8")));

    // blk_entry
    consta!     ((vm, pass_2_int8_args_by_stack_v1) int64_0_local = int64_0);
    consta!     ((vm, pass_2_int8_args_by_stack_v1) int8_1_local  = int8_1);
    consta!     ((vm, pass_2_int8_args_by_stack_v1) int8_2_local  = int8_2);

    block!      ((vm, pass_2_int8_args_by_stack_v1) blk_entry);
    block!      ((vm, pass_2_int8_args_by_stack_v1) blk_main);
    inst!       ((vm, pass_2_int8_args_by_stack_v1) blk_entry_branch:
        BRANCH blk_main (
            int64_0_local,
            int64_0_local,
            int64_0_local,
            int64_0_local,
            int64_0_local,
            int64_0_local,

            int8_1_local,
            int8_2_local
        )
    );

    define_block!((vm, pass_2_int8_args_by_stack_v1) blk_entry() {blk_entry_branch});

    // blk_main
    ssa!        ((vm, pass_2_int8_args_by_stack_v1) <int64> a0);
    ssa!        ((vm, pass_2_int8_args_by_stack_v1) <int64> a1);
    ssa!        ((vm, pass_2_int8_args_by_stack_v1) <int64> a2);
    ssa!        ((vm, pass_2_int8_args_by_stack_v1) <int64> a3);
    ssa!        ((vm, pass_2_int8_args_by_stack_v1) <int64> a4);
    ssa!        ((vm, pass_2_int8_args_by_stack_v1) <int64> a5);
    ssa!        ((vm, pass_2_int8_args_by_stack_v1) <int8> a6);
    ssa!        ((vm, pass_2_int8_args_by_stack_v1) <int8> a7);

    consta!     ((vm, pass_2_int8_args_by_stack_v1) const_funcref_foo8_local = const_funcref_foo8);
    ssa!        ((vm, pass_2_int8_args_by_stack_v1) <int64> retval);
    inst!       ((vm, pass_2_int8_args_by_stack_v1) blk_main_call:
        retval = EXPRCALL (CallConvention::Mu, is_abort: false) const_funcref_foo8_local (a0, a1, a2, a3, a4, a5, a6, a7)
    );

    let blk_main_exit = gen_ccall_exit(retval.clone(), &mut pass_2_int8_args_by_stack_v1, &vm);

    inst!       ((vm, pass_2_int8_args_by_stack_v1) blk_main_ret:
        RET
    );

    define_block!((vm, pass_2_int8_args_by_stack_v1) blk_main(a0, a1, a2, a3, a4, a5, a6, a7) {
        blk_main_call,
        blk_main_exit,
        blk_main_ret
    });

    define_func_ver!((vm) pass_2_int8_args_by_stack_v1 (entry: blk_entry) {
        blk_entry,
        blk_main
    });

    vm
}

#[test]
fn test_pass_mixed_args_by_stack() {
    VM::start_logging_trace();
    let vm = Arc::new(pass_mixed_args_by_stack());

    let compiler = Compiler::new(CompilerPolicy::default(), &vm);

    let func_foo = vm.id_of("foo8");
    let func_main = vm.id_of("pass_mixed_args_by_stack");
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

    let executable = aot::link_primordial(vec![Mu("foo8"), Mu("pass_mixed_args_by_stack")], "test_pass_mixed_args_by_stack", &vm);
    let output = aot::execute_nocheck(executable);

    // exit with (2)
    assert!(output.status.code().is_some());
    assert_eq!(output.status.code().unwrap(), 2);
}

#[cfg(target_arch = "aarch64")]
fn pass_mixed_args_by_stack() -> VM {
    let vm = VM::new_with_opts("init_mu --disable-inline");

    typedef!    ((vm) int64 = mu_int(64));
    typedef!    ((vm) int8  = mu_int(8));

    constdef!   ((vm) <int64> int64_0 = Constant::Int(0));
    constdef!   ((vm) <int64> int64_1 = Constant::Int(1));
    constdef!   ((vm) <int64> int64_2 = Constant::Int(2));
    constdef!   ((vm) <int8>  int8_1 = Constant::Int(1));
    constdef!   ((vm) <int8>  int8_2 = Constant::Int(2));

    // foo8
    funcsig!    ((vm) foo8_sig = (int64, int64, int64, int64, int64, int64, int64, int64, int8, int64) -> (int64));
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
    ssa!        ((vm, foo8_v1) <int8> v8);
    ssa!        ((vm, foo8_v1) <int64> v9);
    block!      ((vm, foo8_v1) blk_entry);

    ssa!        ((vm, foo8_v1) <int64> res);

    inst!       ((vm, foo8_v1) blk_entry_ret:
        RET (v9)
    );

    define_block!((vm, foo8_v1) blk_entry(v0, v1, v2, v3, v4, v5, v6, v7, v8, v9) {
        blk_entry_ret
    });

    define_func_ver!((vm) foo8_v1 (entry: blk_entry) {blk_entry});

    // pass_mixed_args_by_stack
    funcsig!    ((vm) sig = () -> ());
    funcdecl!   ((vm) <sig> pass_mixed_args_by_stack);
    funcdef!    ((vm) <sig> pass_mixed_args_by_stack VERSION pass_mixed_args_by_stack_v1);

    typedef!    ((vm) type_funcref_foo8 = mu_funcref(foo8_sig));
    constdef!   ((vm) <type_funcref_foo8> const_funcref_foo8 = Constant::FuncRef(vm.id_of("foo8")));

    // blk_entry
    consta!     ((vm, pass_mixed_args_by_stack_v1) int64_0_local = int64_0);
    consta!     ((vm, pass_mixed_args_by_stack_v1) int64_2_local = int64_2);
    consta!     ((vm, pass_mixed_args_by_stack_v1) int8_1_local  = int8_1);

    block!      ((vm, pass_mixed_args_by_stack_v1) blk_entry);
    block!      ((vm, pass_mixed_args_by_stack_v1) blk_main);
    inst!       ((vm, pass_mixed_args_by_stack_v1) blk_entry_branch:
        BRANCH blk_main (
            int64_0_local,
            int64_0_local,
            int64_0_local,
            int64_0_local,
            int64_0_local,
            int64_0_local,
            int64_0_local,
            int64_0_local,

            int8_1_local,
            int64_2_local
        )
    );

    define_block!((vm, pass_mixed_args_by_stack_v1) blk_entry() {blk_entry_branch});

    // blk_main
    ssa!        ((vm, pass_mixed_args_by_stack_v1) <int64> a0);
    ssa!        ((vm, pass_mixed_args_by_stack_v1) <int64> a1);
    ssa!        ((vm, pass_mixed_args_by_stack_v1) <int64> a2);
    ssa!        ((vm, pass_mixed_args_by_stack_v1) <int64> a3);
    ssa!        ((vm, pass_mixed_args_by_stack_v1) <int64> a4);
    ssa!        ((vm, pass_mixed_args_by_stack_v1) <int64> a5);
    ssa!        ((vm, pass_mixed_args_by_stack_v1) <int64> a6);
    ssa!        ((vm, pass_mixed_args_by_stack_v1) <int64> a7);
    ssa!        ((vm, pass_mixed_args_by_stack_v1) <int8> a8);
    ssa!        ((vm, pass_mixed_args_by_stack_v1) <int64> a9);

    consta!     ((vm, pass_mixed_args_by_stack_v1) const_funcref_foo8_local = const_funcref_foo8);
    ssa!        ((vm, pass_mixed_args_by_stack_v1) <int64> retval);
    inst!       ((vm, pass_mixed_args_by_stack_v1) blk_main_call:
        retval = EXPRCALL (CallConvention::Mu, is_abort: false) const_funcref_foo8_local (a0, a1, a2, a3, a4, a5, a6, a7, a8, a9)
    );

    let blk_main_exit = gen_ccall_exit(retval.clone(), &mut pass_mixed_args_by_stack_v1, &vm);

    inst!       ((vm, pass_mixed_args_by_stack_v1) blk_main_ret:
        RET
    );

    define_block!((vm, pass_mixed_args_by_stack_v1) blk_main(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9) {
        blk_main_call,
        blk_main_exit,
        blk_main_ret
    });

    define_func_ver!((vm) pass_mixed_args_by_stack_v1 (entry: blk_entry) {
        blk_entry,
        blk_main
    });

    vm
}

#[cfg(target_arch = "x86_64")]
fn pass_mixed_args_by_stack() -> VM {
    let vm = VM::new_with_opts("init_mu --disable-inline");

    typedef!    ((vm) int64 = mu_int(64));
    typedef!    ((vm) int8  = mu_int(8));

    constdef!   ((vm) <int64> int64_0 = Constant::Int(0));
    constdef!   ((vm) <int64> int64_1 = Constant::Int(1));
    constdef!   ((vm) <int64> int64_2 = Constant::Int(2));
    constdef!   ((vm) <int8>  int8_1 = Constant::Int(1));
    constdef!   ((vm) <int8>  int8_2 = Constant::Int(2));

    // foo8
    funcsig!    ((vm) foo8_sig = (int64, int64, int64, int64, int64, int64, int8, int64) -> (int64));
    funcdecl!   ((vm) <foo8_sig> foo8);
    funcdef!    ((vm) <foo8_sig> foo8 VERSION foo8_v1);

    // blk_entry
    ssa!        ((vm, foo8_v1) <int64> v0);
    ssa!        ((vm, foo8_v1) <int64> v1);
    ssa!        ((vm, foo8_v1) <int64> v2);
    ssa!        ((vm, foo8_v1) <int64> v3);
    ssa!        ((vm, foo8_v1) <int64> v4);
    ssa!        ((vm, foo8_v1) <int64> v5);
    ssa!        ((vm, foo8_v1) <int8> v6);
    ssa!        ((vm, foo8_v1) <int64> v7);
    block!      ((vm, foo8_v1) blk_entry);

    ssa!        ((vm, foo8_v1) <int64> res);

    inst!       ((vm, foo8_v1) blk_entry_ret:
        RET (v7)
    );

    define_block!((vm, foo8_v1) blk_entry(v0, v1, v2, v3, v4, v5, v6, v7) {
        blk_entry_ret
    });

    define_func_ver!((vm) foo8_v1 (entry: blk_entry) {blk_entry});

    // pass_mixed_args_by_stack
    funcsig!    ((vm) sig = () -> ());
    funcdecl!   ((vm) <sig> pass_mixed_args_by_stack);
    funcdef!    ((vm) <sig> pass_mixed_args_by_stack VERSION pass_mixed_args_by_stack_v1);

    typedef!    ((vm) type_funcref_foo8 = mu_funcref(foo8_sig));
    constdef!   ((vm) <type_funcref_foo8> const_funcref_foo8 = Constant::FuncRef(vm.id_of("foo8")));

    // blk_entry
    consta!     ((vm, pass_mixed_args_by_stack_v1) int64_0_local = int64_0);
    consta!     ((vm, pass_mixed_args_by_stack_v1) int64_2_local = int64_2);
    consta!     ((vm, pass_mixed_args_by_stack_v1) int8_1_local  = int8_1);

    block!      ((vm, pass_mixed_args_by_stack_v1) blk_entry);
    block!      ((vm, pass_mixed_args_by_stack_v1) blk_main);
    inst!       ((vm, pass_mixed_args_by_stack_v1) blk_entry_branch:
        BRANCH blk_main (
            int64_0_local,
            int64_0_local,
            int64_0_local,
            int64_0_local,
            int64_0_local,
            int64_0_local,

            int8_1_local,
            int64_2_local
        )
    );

    define_block!((vm, pass_mixed_args_by_stack_v1) blk_entry() {blk_entry_branch});

    // blk_main
    ssa!        ((vm, pass_mixed_args_by_stack_v1) <int64> a0);
    ssa!        ((vm, pass_mixed_args_by_stack_v1) <int64> a1);
    ssa!        ((vm, pass_mixed_args_by_stack_v1) <int64> a2);
    ssa!        ((vm, pass_mixed_args_by_stack_v1) <int64> a3);
    ssa!        ((vm, pass_mixed_args_by_stack_v1) <int64> a4);
    ssa!        ((vm, pass_mixed_args_by_stack_v1) <int64> a5);
    ssa!        ((vm, pass_mixed_args_by_stack_v1) <int8> a6);
    ssa!        ((vm, pass_mixed_args_by_stack_v1) <int64> a7);

    consta!     ((vm, pass_mixed_args_by_stack_v1) const_funcref_foo8_local = const_funcref_foo8);
    ssa!        ((vm, pass_mixed_args_by_stack_v1) <int64> retval);
    inst!       ((vm, pass_mixed_args_by_stack_v1) blk_main_call:
        retval = EXPRCALL (CallConvention::Mu, is_abort: false) const_funcref_foo8_local (a0, a1, a2, a3, a4, a5, a6, a7)
    );

    let blk_main_exit = gen_ccall_exit(retval.clone(), &mut pass_mixed_args_by_stack_v1, &vm);

    inst!       ((vm, pass_mixed_args_by_stack_v1) blk_main_ret:
        RET
    );

    define_block!((vm, pass_mixed_args_by_stack_v1) blk_main(a0, a1, a2, a3, a4, a5, a6, a7) {
        blk_main_call,
        blk_main_exit,
        blk_main_ret
    });

    define_func_ver!((vm) pass_mixed_args_by_stack_v1 (entry: blk_entry) {
        blk_entry,
        blk_main
    });

    vm
}

#[test]
fn test_pass_fp_arg() {
    let lib = testutil::compile_fncs("pass_fp_arg", vec!["pass_fp_arg", "foo"], &pass_fp_arg);

    unsafe {
        let pass_fp_arg : libloading::Symbol<unsafe extern fn (f64) -> f64> = lib.get(b"pass_fp_arg").unwrap();

        let res1 = pass_fp_arg(0f64);
        println!("pass_fp_arg(0.0) = {}", res1);
        assert!(res1 == 0f64);

        let res2 = pass_fp_arg(3.14f64);
        println!("pass_fp_arg(3.14) = {}", res2);
        assert!(res2 == 3.14f64);
    }
}

fn pass_fp_arg() -> VM {
    let vm = VM::new_with_opts("init_mu --disable-inline");

    typedef!    ((vm) double = mu_double);

    // foo
    funcsig!    ((vm) foo_sig = (double) -> (double));
    funcdecl!   ((vm) <foo_sig> foo);
    funcdef!    ((vm) <foo_sig> foo VERSION foo_v1);

    // blk_entry
    ssa!        ((vm, foo_v1) <double> x);
    block!      ((vm, foo_v1) blk_entry);

    inst!       ((vm, foo_v1) blk_entry_ret:
        RET (x)
    );

    define_block!   ((vm, foo_v1) blk_entry(x) {
        blk_entry_ret
    });

    define_func_ver!((vm) foo_v1 (entry: blk_entry) {
        blk_entry
    });

    // pass_fp_arg
    funcsig!    ((vm) sig = (double) -> (double));
    funcdecl!   ((vm) <sig> pass_fp_arg);
    funcdef!    ((vm) <sig> pass_fp_arg VERSION pass_fp_arg_v1);

    typedef!    ((vm) type_funcref_foo = mu_funcref(foo_sig));
    constdef!   ((vm) <type_funcref_foo> const_funcref_foo = Constant::FuncRef(vm.id_of("foo")));

    // blk_entry
    ssa!        ((vm, pass_fp_arg_v1) <double> x);
    block!      ((vm, pass_fp_arg_v1) blk_entry);

    ssa!        ((vm, pass_fp_arg_v1) <double> res);
    consta!     ((vm, pass_fp_arg_v1) const_funcref_foo_local = const_funcref_foo);
    inst!       ((vm, pass_fp_arg_v1) blk_entry_call:
        res = EXPRCALL (CallConvention::Mu, is_abort: false) const_funcref_foo_local (x)
    );

    inst!       ((vm, pass_fp_arg_v1) blk_entry_ret:
        RET (res)
    );

    define_block!((vm, pass_fp_arg_v1) blk_entry(x) {
        blk_entry_call,
        blk_entry_ret
    });

    define_func_ver!((vm) pass_fp_arg_v1 (entry: blk_entry) {
        blk_entry
    });

    vm
}

#[test]
fn test_store_funcref() {
    let lib = testutil::compile_fncs("store_funcref", vec!["store_funcref", "foo"], &store_funcref);

    unsafe {
        use mu::utils::mem::memsec::malloc;
        let ptr = match malloc::<u64>(8) {
            Some(ptr) => ptr,
            None => panic!("failed to alloc memory for testing")
        };

        let store_funcref : libloading::Symbol<unsafe extern fn (*mut u64) -> (u64)> = lib.get(b"store_funcref").unwrap();

        let res = store_funcref(ptr);
        println!("store_funcref() = {}", res);
        assert!(res == 1);
    }
}

fn store_funcref() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int64 = mu_int(64));
    constdef!   ((vm) <int64> int64_1 = Constant::Int(1));

    // foo
    funcsig!    ((vm) foo_sig = () -> (int64));
    funcdecl!   ((vm) <foo_sig> foo);
    funcdef!    ((vm) <foo_sig> foo VERSION foo_v1);

    block!      ((vm, foo_v1) blk_entry);
    consta!     ((vm, foo_v1) int64_1_local = int64_1);
    inst!       ((vm, foo_v1) blk_entry_ret:
        RET (int64_1_local)
    );

    define_block!((vm, foo_v1) blk_entry() {
        blk_entry_ret
    });

    define_func_ver!((vm) foo_v1(entry: blk_entry) {
        blk_entry
    });

    // store_funcref
    typedef!    ((vm) type_funcref_foo = mu_funcref(foo_sig));
    constdef!   ((vm) <type_funcref_foo> const_funcref_foo = Constant::FuncRef(vm.id_of("foo")));

    typedef!    ((vm) uptr_funcref_foo = mu_uptr(type_funcref_foo));

    funcsig!    ((vm) store_funcref_sig = (uptr_funcref_foo) -> (int64));
    funcdecl!   ((vm) <store_funcref_sig> store_funcref);
    funcdef!    ((vm) <store_funcref_sig> store_funcref VERSION store_funcref_v1);

    // blk_entry(loc):
    block!      ((vm, store_funcref_v1) blk_entry);
    ssa!        ((vm, store_funcref_v1) <uptr_funcref_foo> loc);

    // STORE funcref_foo loc
    consta!     ((vm, store_funcref_v1) const_funcref_foo_local = const_funcref_foo);
    inst!       ((vm, store_funcref_v1) blk_entry_store:
        STORE loc const_funcref_foo_local (is_ptr: true, order: MemoryOrder::Relaxed)
    );

    // f = LOAD loc
    ssa!        ((vm, store_funcref_v1) <type_funcref_foo> f);
    inst!       ((vm, store_funcref_v1) blk_entry_load:
        f = LOAD loc (is_ptr: true, order: MemoryOrder::Relaxed)
    );

    // res = EXPRCALL f ()
    ssa!        ((vm, store_funcref_v1) <int64> res);
    inst!       ((vm, store_funcref_v1) blk_entry_call:
        res = EXPRCALL (CallConvention::Mu, is_abort: false) f ()
    );

    // RET res
    inst!       ((vm, store_funcref_v1) blk_entry_ret:
        RET (res)
    );

    define_block!((vm, store_funcref_v1) blk_entry(loc) {
        blk_entry_store,
        blk_entry_load,
        blk_entry_call,
        blk_entry_ret
    });

    define_func_ver!((vm) store_funcref_v1 (entry: blk_entry) {
        blk_entry
    });

    vm
}

use test_compiler::test_int128::add_u128;

#[test]
fn test_call_int128_arg() {
    VM::start_logging_trace();

    let vm = Arc::new(add_u128());
    call_add_u128(&vm);

    let func_add  = vm.id_of("add_u128");
    let func_call = vm.id_of("call_add_u128");

    let compiler = Compiler::new(CompilerPolicy::default(), &vm);
    {
        let funcs = vm.funcs().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();

        {
            let func = funcs.get(&func_add).unwrap().read().unwrap();
            let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();

            compiler.compile(&mut func_ver);
        }
        {
            let func = funcs.get(&func_call).unwrap().read().unwrap();
            let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();

            compiler.compile(&mut func_ver);
        }
    }

    vm.make_primordial_thread(func_call, true, vec![]);
    backend::emit_context(&vm);

    let executable = aot::link_primordial(vec![Mu("add_u128"), Mu("call_add_u128")], "test_call_int128_arg", &vm);
    let output = aot::execute_nocheck(executable);

    // exit with (84)
    assert!(output.status.code().is_some());
    assert_eq!(output.status.code().unwrap(), 84);
}

fn call_add_u128(vm: &VM) {
    let add_u128_sig = vm.get_func_sig(vm.id_of("sig"));
    let add_u128_id  = vm.id_of("add_u128");

    typedef!    ((vm) int64  = mu_int(64));
    typedef!    ((vm) int128 = mu_int(128));

    constdef!   ((vm) <int128> int128_42 = Constant::IntEx(vec![42, 0]));

    typedef!    ((vm) funcref_add_u128 = mu_funcref(add_u128_sig));
    constdef!   ((vm) <funcref_add_u128> const_funcref_add_u128 = Constant::FuncRef(add_u128_id));

    funcsig!    ((vm) call_add_u128_sig = () -> ());
    funcdecl!   ((vm) <call_add_u128_sig> call_add_u128);
    funcdef!    ((vm) <call_add_u128_sig> call_add_u128 VERSION call_add_u128_v1);

    // blk_entry
    block!      ((vm, call_add_u128_v1) blk_entry);

    // EXPRCALL add_u128 (42, 42)
    ssa!        ((vm, call_add_u128_v1) <int128> res);
    consta!     ((vm, call_add_u128_v1) int128_42_local = int128_42);
    consta!     ((vm, call_add_u128_v1) const_funcref_add_u128_local = const_funcref_add_u128);
    inst!       ((vm, call_add_u128_v1) blk_entry_call:
        res = EXPRCALL (CallConvention::Mu, is_abort: false) const_funcref_add_u128_local (int128_42_local, int128_42_local)
    );

    ssa!        ((vm, call_add_u128_v1) <int64> trunc_res);
    inst!       ((vm, call_add_u128_v1) blk_entry_trunc:
        trunc_res = CONVOP (ConvOp::TRUNC) <int128 int64> res
    );

    let blk_entry_exit = gen_ccall_exit(trunc_res.clone(), &mut call_add_u128_v1, &vm);

    inst!       ((vm, call_add_u128_v1) blk_entry_ret:
        RET
    );

    define_block!((vm, call_add_u128_v1) blk_entry() {
        blk_entry_call,
        blk_entry_trunc,
        blk_entry_exit,
        blk_entry_ret
    });

    define_func_ver!((vm) call_add_u128_v1 (entry: blk_entry) {
        blk_entry
    });
}