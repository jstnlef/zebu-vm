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
extern crate log;
extern crate mu;

use self::mu::ast::inst::*;
use self::mu::ast::ir::*;
use self::mu::ast::op::*;
use self::mu::ast::types::*;
use self::mu::compiler::*;
use self::mu::runtime::thread::MuThread;
use self::mu::vm::VM;
use mu::linkutils;
use mu::linkutils::aot;
use mu::vm::handle;
use test_compiler::test_call::gen_ccall_exit;
use test_ir::test_ir::global_access;
use utils::Address;
use utils::LinkedHashMap;

use std::sync::Arc;

#[test]
fn test_global_access() {
    VM::start_logging_trace();

    let vm = Arc::new(VM::new());
    unsafe {
        MuThread::current_thread_as_mu_thread(Address::zero(), vm.clone());
    }
    global_access(&vm);

    let compiler = Compiler::new(CompilerPolicy::default(), &vm);

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

    let compiler = Compiler::new(CompilerPolicy::default(), &vm);
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
        vm.handle_store(MemoryOrder::Relaxed, &global_handle, &uint64_10_handle);
    }

    // then emit context (global will be put into context.s
    vm.set_primordial_thread(func_id, true, vec![]);
    backend::emit_context(&vm);

    // link
    let executable = aot::link_primordial(vec![Mu("set_global_by_api")], "set_global_by_api_test", &vm);
    let output = linkutils::exec_path_nocheck(executable);

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

#[test]
fn test_get_global_in_dylib() {
    VM::start_logging_trace();

    let vm = Arc::new(VM::new());
    unsafe {
        MuThread::current_thread_as_mu_thread(Address::zero(), vm.clone());
    }
    get_global_in_dylib(&vm);

    let compiler = Compiler::new(CompilerPolicy::default(), &vm);
    let func_id = vm.id_of("get_global_in_dylib");

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
        vm.handle_store(MemoryOrder::Relaxed, &global_handle, &uint64_10_handle);
    }

    // then emit context (global will be put into context.s
    backend::emit_context(&vm);

    // link
    let libname = &linkutils::get_dylib_name("get_global_in_dylib");
    let libpath = linkutils::aot::link_dylib(vec![Mu("get_global_in_dylib")], libname, &vm);
    let lib = libloading::Library::new(libpath.as_os_str()).unwrap();

    unsafe {
        let get_global_in_dylib: libloading::Symbol<unsafe extern "C" fn() -> u64> =
            lib.get(b"get_global_in_dylib").unwrap();
        let res = get_global_in_dylib();

        println!("my_global = {}", res);
        assert_eq!(res, 10);
    }
}

fn get_global_in_dylib(vm: &VM) {
    typedef!    ((vm) int64      = mu_int(64));
    typedef!    ((vm) iref_int64 = mu_iref(int64));

    globaldef!  ((vm) <int64> my_global);

    funcsig!    ((vm) sig = () -> (int64));
    funcdecl!   ((vm) <sig> get_global_in_dylib);
    funcdef!    ((vm) <sig> get_global_in_dylib VERSION get_global_in_dylib_v1);

    // blk entry
    block!      ((vm, get_global_in_dylib_v1) blk_entry);
    ssa!        ((vm, get_global_in_dylib_v1) <int64> val);
    global!     ((vm, get_global_in_dylib_v1) blk_entry_my_global = my_global);
    inst!       ((vm, get_global_in_dylib_v1) blk_entry_load:
        val = LOAD blk_entry_my_global (is_ptr: false, order: MemoryOrder::SeqCst)
    );

    // won't execute this inst
    inst!       ((vm, get_global_in_dylib_v1) blk_entry_ret:
        RET (val)
    );

    define_block!   ((vm, get_global_in_dylib_v1) blk_entry() {
        blk_entry_load, blk_entry_ret
    });

    define_func_ver!((vm) get_global_in_dylib_v1 (entry: blk_entry) {
        blk_entry
    });
}

#[test]
fn test_persist_linked_list() {
    VM::start_logging_trace();

    let vm = Arc::new(VM::new());
    unsafe {
        MuThread::current_thread_as_mu_thread(Address::zero(), vm.clone());
    }
    persist_linked_list(&vm);

    let compiler = Compiler::new(CompilerPolicy::default(), &vm);
    let func_id = vm.id_of("persist_linked_list");

    {
        let funcs = vm.funcs().read().unwrap();
        let func = funcs.get(&func_id).unwrap().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();
        let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();

        compiler.compile(&mut func_ver);
    }

    // create a linked list by api
    const LINKED_LIST_SIZE: usize = 5;
    {
        let mut i = 0;
        let mut last_node: Option<Box<handle::APIHandle>> = None;

        let node_tyid = vm.id_of("node");

        while i < LINKED_LIST_SIZE {
            // new node
            let node_ref = vm.new_fixed(node_tyid);
            let node_iref = vm.handle_get_iref(&node_ref);

            // store i as payload
            let payload_iref = vm.handle_get_field_iref(&node_iref, 1); // payload is the 2nd field
            let int_handle = vm.handle_from_uint64(i as u64, 64);
            vm.handle_store(MemoryOrder::Relaxed, &payload_iref, &int_handle);

            // store last_node as next
            let next_iref = vm.handle_get_field_iref(&node_iref, 0);
            if last_node.is_some() {
                let handle = last_node.take().unwrap();
                vm.handle_store(MemoryOrder::Relaxed, &next_iref, &handle);
            }

            last_node = Some(node_ref);
            i += 1;
        }

        // store last_node in global
        let global_id = vm.id_of("my_global");
        let global_handle = vm.handle_from_global(global_id);

        vm.handle_store(MemoryOrder::Relaxed, &global_handle, last_node.as_ref().unwrap());
    }

    // then emit context (global will be put into context.s
    vm.set_primordial_thread(func_id, true, vec![]);
    backend::emit_context(&vm);

    // link
    let executable = aot::link_primordial(vec![Mu("persist_linked_list")], "persist_linked_list_test", &vm);
    let output = linkutils::exec_path_nocheck(executable);

    assert!(output.status.code().is_some());

    let ret_code = output.status.code().unwrap();
    println!("return code: {}", ret_code);
    assert!(ret_code == 10);
}

fn persist_linked_list(vm: &VM) {
    typedef!    ((vm) int1       = mu_int(1));
    typedef!    ((vm) int64      = mu_int(64));
    typedef!    ((vm) node       = mu_struct_placeholder());
    typedef!    ((vm) iref_node  = mu_iref(node));
    typedef!    ((vm) iref_int64 = mu_iref(int64));
    typedef!    ((vm) ref_node   = mu_ref(node));
    typedef!    ((vm) iref_ref_node = mu_iref(ref_node));
    typedef!    ((vm) mu_struct_put(node, ref_node, int64));

    globaldef!  ((vm) <ref_node> my_global);

    constdef!   ((vm) <int64>    int64_0       = Constant::Int(0));
    constdef!   ((vm) <ref_node> ref_node_null = Constant::NullRef);

    funcsig!    ((vm) sig = () -> (int64));
    funcdecl!   ((vm) <sig> persist_linked_list);
    funcdef!    ((vm) <sig> persist_linked_list VERSION persist_linked_list_v1);

    // --- blk entry ---
    block!      ((vm, persist_linked_list_v1) blk_entry);
    consta!     ((vm, persist_linked_list_v1) int64_0_local = int64_0);
    global!     ((vm, persist_linked_list_v1) blk_entry_my_global = my_global);

    // %head = LOAD %blk_entry_my_global
    ssa!        ((vm, persist_linked_list_v1) <ref_node> head);
    inst!       ((vm, persist_linked_list_v1) blk_entry_load:
        head = LOAD blk_entry_my_global (is_ptr: false, order: MemoryOrder::SeqCst)
    );

    // branch blk_loop_head (%head, 0)
    block!      ((vm, persist_linked_list_v1) blk_loop_head);
    inst!       ((vm, persist_linked_list_v1) blk_entry_branch:
        BRANCH blk_loop_head (head, int64_0_local)
    );

    define_block!   ((vm, persist_linked_list_v1) blk_entry() {
        blk_entry_load, blk_entry_branch
    });

    // --- blk loop_head ---
    ssa!        ((vm, persist_linked_list_v1) <ref_node> cursor);
    ssa!        ((vm, persist_linked_list_v1) <int64> sum);

    // %cond = CMP EQ cursor NULLREF
    ssa!        ((vm, persist_linked_list_v1) <int1> cond);
    consta!     ((vm, persist_linked_list_v1) ref_node_null_local = ref_node_null);
    inst!       ((vm, persist_linked_list_v1) blk_loop_head_cmp:
        cond = CMPOP (CmpOp::EQ) cursor ref_node_null_local
    );

    // BRANCH2 cond exit[sum] loop_body[cursor, sum]
    block!      ((vm, persist_linked_list_v1) blk_exit);
    block!      ((vm, persist_linked_list_v1) blk_loop_body);
    inst!       ((vm, persist_linked_list_v1) blk_loop_head_branch2:
        BRANCH2 (cond, sum, cursor)
            IF (OP 0)
            THEN blk_exit (vec![1]) WITH 0.1f32,
            ELSE blk_loop_body (vec![2, 1])
    );

    define_block!   ((vm, persist_linked_list_v1) blk_loop_head(cursor, sum) {
        blk_loop_head_cmp, blk_loop_head_branch2
    });

    // --- blk loop_body ---
    ssa!        ((vm, persist_linked_list_v1) <ref_node> body_cursor);
    ssa!        ((vm, persist_linked_list_v1) <int64>    body_sum);

    // %iref_cursor = GETIREF %body_cursor
    ssa!        ((vm, persist_linked_list_v1) <iref_node> iref_cursor);
    inst!       ((vm, persist_linked_list_v1) blk_loop_body_getiref:
        iref_cursor = GETIREF body_cursor
    );

    // %iref_payload = GETFIELDIREF iref_cursor 1
    ssa!        ((vm, persist_linked_list_v1) <iref_int64> iref_payload);
    inst!       ((vm, persist_linked_list_v1) blk_loop_body_getfieldiref:
        iref_payload = GETFIELDIREF iref_cursor (is_ptr: false, index: 1)
    );

    // %payload = LOAD %iref_payload
    ssa!        ((vm, persist_linked_list_v1) <int64> payload);
    inst!       ((vm, persist_linked_list_v1) blk_loop_body_load:
        payload = LOAD iref_payload (is_ptr: false, order: MemoryOrder::SeqCst)
    );

    // %body_sum2 = BINOP ADD body_sum payload
    ssa!        ((vm, persist_linked_list_v1) <int64> body_sum2);
    inst!       ((vm, persist_linked_list_v1) blk_loop_body_add:
        body_sum2 = BINOP (BinOp::Add) body_sum payload
    );

    // %iref_next = GETFIELDIREF iref_cursor 0
    ssa!        ((vm, persist_linked_list_v1) <iref_ref_node> iref_next);
    inst!       ((vm, persist_linked_list_v1) blk_loop_body_getfieldiref2:
        iref_next = GETFIELDIREF iref_cursor (is_ptr: false, index: 0)
    );

    // %next = LOAD %iref_next
    ssa!        ((vm, persist_linked_list_v1) <ref_node> next);
    inst!       ((vm, persist_linked_list_v1) blk_loop_body_load2:
        next = LOAD iref_next (is_ptr: false, order: MemoryOrder::SeqCst)
    );

    // BRANCH blk_loop_head (next, body_sum2)
    inst!       ((vm, persist_linked_list_v1) blk_loop_body_branch:
        BRANCH blk_loop_head (next, body_sum2)
    );

    define_block!   ((vm, persist_linked_list_v1) blk_loop_body(body_cursor, body_sum){
        blk_loop_body_getiref,
        blk_loop_body_getfieldiref,
        blk_loop_body_load,
        blk_loop_body_add,
        blk_loop_body_getfieldiref2,
        blk_loop_body_load2,
        blk_loop_body_branch
    });

    // --- blk exit ---
    ssa!       ((vm, persist_linked_list_v1) <int64> res);

    let blk_exit_exit = gen_ccall_exit(res.clone(), &mut persist_linked_list_v1, &vm);

    inst!      ((vm, persist_linked_list_v1) blk_exit_ret:
        RET (res)
    );

    define_block!   ((vm, persist_linked_list_v1) blk_exit(res) {
        blk_exit_exit, blk_exit_ret
    });

    define_func_ver!((vm) persist_linked_list_v1 (entry: blk_entry) {
        blk_entry, blk_loop_head, blk_loop_body, blk_exit
    });
}

#[test]
fn test_persist_hybrid() {
    VM::start_logging_trace();

    let vm = Arc::new(VM::new());
    unsafe {
        MuThread::current_thread_as_mu_thread(Address::zero(), vm.clone());
    }
    persist_hybrid(&vm);

    let compiler = Compiler::new(CompilerPolicy::default(), &vm);
    let func_id = vm.id_of("persist_hybrid");

    {
        let funcs = vm.funcs().read().unwrap();
        let func = funcs.get(&func_id).unwrap().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();
        let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();

        compiler.compile(&mut func_ver);
    }

    // create hybrid by api
    const HYBRID_LENGTH: usize = 5;
    {
        let hybrid_tyid = vm.id_of("hybrid");
        let hybrid_len = vm.handle_from_uint64(HYBRID_LENGTH as u64, 64);
        let hybrid = vm.new_hybrid(hybrid_tyid, &hybrid_len);

        let hybrid_iref = vm.handle_get_iref(&hybrid);
        let hybrid_varpart = vm.handle_get_var_part_iref(&hybrid_iref);

        // create int64 objects to fill var part
        let int64_tyid = vm.id_of("int64");

        for i in 0..HYBRID_LENGTH {
            // new node
            let node_ref = vm.new_fixed(int64_tyid);
            let node_iref = vm.handle_get_iref(&node_ref);

            // store i into node
            let int_handle = vm.handle_from_uint64(i as u64, 64);
            vm.handle_store(MemoryOrder::Relaxed, &node_iref, &int_handle);

            // store node to hybrid
            let hybrid_cell = vm.handle_shift_iref(&hybrid_varpart, &int_handle);
            vm.handle_store(MemoryOrder::Relaxed, &hybrid_cell, &node_ref);
        }

        // store last_node in global
        let global_id = vm.id_of("my_global");
        let global_handle = vm.handle_from_global(global_id);

        vm.handle_store(MemoryOrder::Relaxed, &global_handle, &hybrid);
    }

    // then emit context (global will be put into context.s
    vm.set_primordial_thread(func_id, true, vec![Constant::Int(HYBRID_LENGTH as u64)]);
    backend::emit_context(&vm);

    // link
    let executable = aot::link_primordial(vec![Mu("persist_hybrid")], "persist_hybrid_test", &vm);
    let output = linkutils::exec_path_nocheck(executable);

    assert!(output.status.code().is_some());

    let ret_code = output.status.code().unwrap();
    println!("return code: {}", ret_code);
    assert!(ret_code == 10);
}

fn persist_hybrid(vm: &VM) {
    typedef!    ((vm) int1            = mu_int(1));
    typedef!    ((vm) int64           = mu_int(64));
    typedef!    ((vm) ref_int64       = mu_ref(int64));
    typedef!    ((vm) iref_int64      = mu_iref(int64));
    typedef!    ((vm) hybrid          = mu_hybrid()(ref_int64));
    typedef!    ((vm) ref_hybrid      = mu_ref(hybrid));
    typedef!    ((vm) iref_ref_hybrid = mu_iref(ref_hybrid));
    typedef!    ((vm) iref_ref_int64  = mu_iref(ref_int64));

    globaldef!  ((vm) <ref_hybrid> my_global);

    constdef!   ((vm) <int64> int64_0 = Constant::Int(0));
    constdef!   ((vm) <int64> int64_1 = Constant::Int(1));

    funcsig!    ((vm) sig = (int64) -> (int64));
    funcdecl!   ((vm) <sig> persist_hybrid);
    funcdef!    ((vm) <sig> persist_hybrid VERSION persist_hybrid_v1);

    // --- blk entry ---
    block!      ((vm, persist_hybrid_v1) blk_entry);
    ssa!        ((vm, persist_hybrid_v1) <int64> hybrid_len);
    global!     ((vm, persist_hybrid_v1) blk_entry_my_global = my_global);

    // %h = LOAD %blk_entry_my_global
    ssa!        ((vm, persist_hybrid_v1) <ref_hybrid> h);
    inst!       ((vm, persist_hybrid_v1) blk_entry_load:
        h = LOAD blk_entry_my_global (is_ptr: false, order: MemoryOrder::SeqCst)
    );

    // %var_h = GETVARPARTIREF %h
    ssa!        ((vm, persist_hybrid_v1) <iref_ref_int64> var_h);
    inst!       ((vm, persist_hybrid_v1) blk_entry_getvarpart:
        var_h = GETVARPARTIREF h (is_ptr: false)
    );

    // BRANCH loop_head (varpart: %var_h, sum: 0, i: 0, n: %hybrid_len)
    block!      ((vm, persist_hybrid_v1) blk_loop_head);
    consta!     ((vm, persist_hybrid_v1) int64_0_local = int64_0);
    inst!       ((vm, persist_hybrid_v1) blk_entry_branch:
        BRANCH blk_loop_head (var_h, int64_0_local, int64_0_local, hybrid_len)
    );

    define_block!   ((vm, persist_hybrid_v1) blk_entry(hybrid_len) {
        blk_entry_load, blk_entry_getvarpart, blk_entry_branch
    });

    // --- blk loop_head ---
    ssa!        ((vm, persist_hybrid_v1) <iref_ref_int64> head_varpart);
    ssa!        ((vm, persist_hybrid_v1) <int64> head_sum);
    ssa!        ((vm, persist_hybrid_v1) <int64> head_i);
    ssa!        ((vm, persist_hybrid_v1) <int64> head_n);

    // %cond = CMP ULT %i %n
    ssa!        ((vm, persist_hybrid_v1) <int1> cond);
    inst!       ((vm, persist_hybrid_v1) blk_loop_head_cmp:
        cond = CMPOP (CmpOp::ULT) head_i head_n
    );

    // BRANCH2 cond loop_body[varpart, sum, i, n] exit[sum]
    block!      ((vm, persist_hybrid_v1) blk_loop_body);
    block!      ((vm, persist_hybrid_v1) blk_exit);
    inst!       ((vm, persist_hybrid_v1) blk_loop_head_branch2:
        BRANCH2 (cond, head_varpart, head_sum, head_i, head_n)
            IF (OP 0)
            THEN blk_loop_body (vec![1, 2, 3, 4]) WITH 0.9f32,
            ELSE blk_exit (vec![2])
    );

    define_block!   ((vm, persist_hybrid_v1) blk_loop_head(head_varpart, head_sum, head_i, head_n) {
        blk_loop_head_cmp, blk_loop_head_branch2
    });

    // --- blk loop_body ---
    ssa!        ((vm, persist_hybrid_v1) <iref_ref_int64> varpart);
    ssa!        ((vm, persist_hybrid_v1) <int64> sum);
    ssa!        ((vm, persist_hybrid_v1) <int64> i);
    ssa!        ((vm, persist_hybrid_v1) <int64> n);

    // %cell = SHIFTIREF %varpart %i
    ssa!        ((vm, persist_hybrid_v1) <iref_ref_int64> cell);
    inst!       ((vm, persist_hybrid_v1) blk_loop_body_shiftiref:
        cell = SHIFTIREF varpart i (is_ptr: false)
    );

    // %int_obj = LOAD %cell
    ssa!        ((vm, persist_hybrid_v1) <ref_int64> int_obj);
    inst!       ((vm, persist_hybrid_v1) blk_loop_body_load_obj:
        int_obj = LOAD cell (is_ptr: false, order: MemoryOrder::SeqCst)
    );

    // %iref_int_obj = GETIREF %int_obj
    ssa!        ((vm, persist_hybrid_v1) <iref_int64> iref_int_obj);
    inst!       ((vm, persist_hybrid_v1) blk_loop_body_getiref:
        iref_int_obj = GETIREF int_obj
    );

    // %val = LOAD %iref_int_obj
    ssa!        ((vm, persist_hybrid_v1) <int64> val);
    inst!       ((vm, persist_hybrid_v1) blk_loop_body_load_val:
        val = LOAD iref_int_obj (is_ptr: false, order: MemoryOrder::SeqCst)
    );

    // %sum2 = BINOP ADD sum val
    ssa!        ((vm, persist_hybrid_v1) <int64> sum2);
    inst!       ((vm, persist_hybrid_v1) blk_loop_body_add:
        sum2 = BINOP (BinOp::Add) sum val
    );

    // %i2 = BINOP ADD %i 1
    ssa!        ((vm, persist_hybrid_v1) <int64> i2);
    consta!     ((vm, persist_hybrid_v1) int64_1_local = int64_1);
    inst!       ((vm, persist_hybrid_v1) blk_loop_body_add2:
        i2 = BINOP (BinOp::Add) i int64_1_local
    );

    // BRANCH loop_head (%varpart, %sum2, %i2, %n)
    inst!       ((vm, persist_hybrid_v1) blk_loop_body_branch:
        BRANCH blk_loop_head (varpart, sum2, i2, n)
    );

    define_block!   ((vm, persist_hybrid_v1) blk_loop_body(varpart, sum, i, n) {
        blk_loop_body_shiftiref,
        blk_loop_body_load_obj,
        blk_loop_body_getiref,
        blk_loop_body_load_val,
        blk_loop_body_add,
        blk_loop_body_add2,
        blk_loop_body_branch
    });

    // --- blk exit ---
    ssa!        ((vm, persist_hybrid_v1) <int64> res);

    let blk_exit_exit = gen_ccall_exit(res.clone(), &mut persist_hybrid_v1, &vm);

    inst!       ((vm, persist_hybrid_v1) blk_exit_ret:
        RET (res)
    );

    define_block!   ((vm, persist_hybrid_v1) blk_exit(res) {
        blk_exit_exit, blk_exit_ret
    });

    define_func_ver! ((vm) persist_hybrid_v1 (entry: blk_entry) {
        blk_entry, blk_loop_head, blk_loop_body, blk_exit
    });
}

#[test]
fn test_persist_funcref() {
    VM::start_logging_trace();

    let vm = Arc::new(VM::new_with_opts("init_mu --disable-inline"));
    unsafe {
        MuThread::current_thread_as_mu_thread(Address::zero(), vm.clone());
    }
    persist_funcref(&vm);

    let compiler = Compiler::new(CompilerPolicy::default(), &vm);

    let func_ret42_id = vm.id_of("ret42");
    {
        let funcs = vm.funcs().read().unwrap();
        let func = funcs.get(&func_ret42_id).unwrap().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();
        let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();

        compiler.compile(&mut func_ver);
    }

    let func_my_main_id = vm.id_of("my_main");
    {
        let funcs = vm.funcs().read().unwrap();
        let func = funcs.get(&func_my_main_id).unwrap().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();
        let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();

        compiler.compile(&mut func_ver);
    }

    // store funcref to ret42 in the global
    {
        let global_id = vm.id_of("my_global");
        let global_handle = vm.handle_from_global(global_id);

        let func_ret42_handle = vm.handle_from_func(func_ret42_id);

        debug!("write {:?} to location {:?}", func_ret42_handle, global_handle);
        vm.handle_store(MemoryOrder::Relaxed, &global_handle, &func_ret42_handle);
    }

    let my_main_handle = vm.handle_from_func(func_my_main_id);

    // make boot image
    vm.make_boot_image(
        vec![func_ret42_id, func_my_main_id], // whitelist
        Some(&my_main_handle),
        None, // primoridal func, stack
        None, // threadlocal
        vec![],
        vec![], // sym fields/strings
        vec![],
        vec![], // reloc fields/strings
        "test_persist_funcref".to_string()
    );

    // link
    let executable = {
        use std::path;
        let mut path = path::PathBuf::new();
        path.push(&vm.options.flag_aot_emit_dir);
        path.push("test_persist_funcref");
        path
    };
    let output = linkutils::exec_path_nocheck(executable);

    assert!(output.status.code().is_some());

    let ret_code = output.status.code().unwrap();
    println!("return code: {}", ret_code);
    assert!(ret_code == 42);
}

fn persist_funcref(vm: &VM) {
    typedef!    ((vm) int64 = mu_int(64));
    constdef!   ((vm) <int64> int64_42 = Constant::Int(42));

    funcsig!    ((vm) ret42_sig = () -> (int64));
    funcdecl!   ((vm) <ret42_sig> ret42);
    funcdef!    ((vm) <ret42_sig> ret42 VERSION ret42_v1);

    typedef!    ((vm) funcref_to_ret42 = mu_funcref(ret42_sig));
    globaldef!  ((vm) <funcref_to_ret42> my_global);

    // ---ret42---
    {
        // blk entry
        block!      ((vm, ret42_v1) blk_entry);
        consta!     ((vm, ret42_v1) int64_42_local = int64_42);
        inst!       ((vm, ret42_v1) blk_entry_ret:
            RET (int64_42_local)
        );
        define_block!((vm, ret42_v1) blk_entry() {
            blk_entry_ret
        });

        define_func_ver!((vm) ret42_v1 (entry: blk_entry) {blk_entry});
    }

    // ---my_main---
    {
        funcsig!    ((vm) my_main_sig = () -> ());
        funcdecl!   ((vm) <my_main_sig> my_main);
        funcdef!    ((vm) <my_main_sig> my_main VERSION my_main_v1);

        // blk entry
        block!      ((vm, my_main_v1) blk_entry);
        global!     ((vm, my_main_v1) blk_entry_global = my_global);
        ssa!        ((vm, my_main_v1) <funcref_to_ret42> func);

        inst!       ((vm, my_main_v1) blk_entry_load:
            func = LOAD blk_entry_global (is_ptr: false, order: MemoryOrder::SeqCst)
        );

        ssa!        ((vm, my_main_v1) <int64> blk_entry_res);
        inst!       ((vm, my_main_v1) blk_entry_call:
            blk_entry_res = EXPRCALL (CallConvention::Mu, is_abort: false) func ()
        );

        let blk_entry_exit = gen_ccall_exit(blk_entry_res.clone(), &mut my_main_v1, &vm);

        inst!       ((vm, my_main_v1) blk_entry_ret:
            RET
        );

        define_block!   ((vm, my_main_v1) blk_entry() {
            blk_entry_load,
            blk_entry_call,
            blk_entry_exit,
            blk_entry_ret
        });

        define_func_ver!((vm) my_main_v1 (entry: blk_entry) {
            blk_entry
        });
    }
}
