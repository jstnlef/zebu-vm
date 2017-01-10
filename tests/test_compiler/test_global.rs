extern crate mu;
extern crate log;
extern crate libloading;

use test_ir::test_ir::global_access;
use test_compiler::test_call::gen_ccall_exit;
use self::mu::compiler::*;
use self::mu::vm::VM;
use self::mu::runtime::thread::MuThread;
use self::mu::ast::types::*;
use self::mu::ast::ir::*;
use self::mu::ast::inst::*;
use self::mu::ast::op::*;
use utils::Address;
use utils::LinkedHashMap;
use mu::testutil;
use mu::testutil::aot;
use mu::vm::handle;

use std::sync::RwLock;
use std::sync::Arc;

#[test]
fn test_global_access() {
    VM::start_logging_trace();

    let vm = Arc::new(VM::new());
    unsafe {
        MuThread::current_thread_as_mu_thread(Address::zero(), vm.clone());
    }
    global_access(&vm);

    let compiler = Compiler::new(CompilerPolicy::default(), vm.clone());

    {
        let func_id = vm.id_of("global_access");
        let funcs = vm.funcs().read().unwrap();
        let func = funcs.get(&func_id).unwrap().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();
        let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();

        compiler.compile(&mut func_ver);
    }
    backend::emit_context(&vm);
}

#[test]
fn test_set_global_by_api() {
    VM::start_logging_trace();

    let vm = Arc::new(VM::new());
    unsafe {
        MuThread::current_thread_as_mu_thread(Address::zero(), vm.clone());
    }
    set_global_by_api(&vm);

    let compiler = Compiler::new(CompilerPolicy::default(), vm.clone());
    let func_id = vm.id_of("set_global_by_api");

    {
        let funcs = vm.funcs().read().unwrap();
        let func = funcs.get(&func_id).unwrap().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();
        let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();

        compiler.compile(&mut func_ver);
    }

    // set global by api here
    {
        let global_id = vm.id_of("my_global");
        let global_handle = vm.handle_from_global(global_id);

        let uint64_10_handle = vm.handle_from_uint64(10, 64);

        debug!("write {:?} to location {:?}", uint64_10_handle, global_handle);
        handle::store(MemoryOrder::Relaxed, global_handle, uint64_10_handle);
    }

    // then emit context (global will be put into context.s
    vm.make_primordial_thread(func_id, vec![]);
    backend::emit_context(&vm);

    // link
    let executable = aot::link_primordial(vec![Mu("set_global_by_api")], "set_global_by_api_test", &vm);
    let output = aot::execute_nocheck(executable);

    assert!(output.status.code().is_some());

    let ret_code = output.status.code().unwrap();
    println!("return code: {} (i.e. the value set before)", ret_code);
    assert!(ret_code == 10);
}

fn set_global_by_api(vm: &VM) {
    typedef!    ((vm) int64      = mu_int(64));
    typedef!    ((vm) iref_int64 = mu_iref(int64));

    globaldef!  ((vm) <int64> my_global);

    funcsig!    ((vm) sig = () -> (int64));
    funcdecl!   ((vm) <sig> set_global_by_api);
    funcdef!    ((vm) <sig> set_global_by_api VERSION set_global_by_api_v1);

    // blk entry
    block!      ((vm, set_global_by_api_v1) blk_entry);
    ssa!        ((vm, set_global_by_api_v1) <int64> val);
    global!     ((vm, set_global_by_api_v1) blk_entry_my_global = my_global);
    inst!       ((vm, set_global_by_api_v1) blk_entry_load:
        val = LOAD blk_entry_my_global (is_ptr: false, order: MemoryOrder::SeqCst)
    );

    let blk_entry_exit = gen_ccall_exit(val.clone(), &mut set_global_by_api_v1, &vm);

    // won't execute this inst
    inst!       ((vm, set_global_by_api_v1) blk_entry_ret:
        RET (val)
    );

    define_block!   ((vm, set_global_by_api_v1) blk_entry() {
        blk_entry_load, blk_entry_exit, blk_entry_ret
    });

    define_func_ver!((vm) set_global_by_api_v1 (entry: blk_entry) {
        blk_entry
    });
}
