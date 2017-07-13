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

extern crate log;
extern crate libloading;
extern crate mu;

use self::mu::ast::types::*;
use self::mu::ast::ir::*;
use self::mu::ast::inst::*;
use self::mu::vm::*;
use self::mu::compiler::*;
use self::mu::runtime::thread::MuThread;
use self::mu::utils::Address;
use self::mu::utils::LinkedHashMap;

use std::sync::Arc;
use self::mu::linkutils;
use self::mu::linkutils::aot;

#[test]
fn test_allocation_fastpath() {
    VM::start_logging_trace();

    let vm = Arc::new(allocation_fastpath());

    let compiler = Compiler::new(CompilerPolicy::default(), &vm);

    let func_id = vm.id_of("allocation_fastpath");
    {
        let funcs = vm.funcs().read().unwrap();
        let func = funcs.get(&func_id).unwrap().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();
        let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();

        compiler.compile(&mut func_ver);
    }

    vm.set_primordial_thread(func_id, true, vec![]);
    backend::emit_context(&vm);

    let executable = aot::link_primordial(vec!["allocation_fastpath".to_string()], "allocation_fastpath_test", &vm);
    linkutils::exec_path(executable);
}

fn allocation_fastpath() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int1         = mu_int(1));
    typedef!    ((vm) int64        = mu_int(64));
    typedef!    ((vm) ref_int64    = mu_ref(int64));
    typedef!    ((vm) struct_t     = mu_struct(int64, int64, ref_int64));
    typedef!    ((vm) ref_struct_t = mu_ref(struct_t));

    funcsig!    ((vm) sig = () -> ());
    funcdecl!   ((vm) <sig> allocation_fastpath);
    funcdef!    ((vm) <sig> allocation_fastpath VERSION allocation_fastpath_v1);

    block!      ((vm, allocation_fastpath_v1) blk_entry);

    // a = NEW <struct_t>
    ssa!        ((vm, allocation_fastpath_v1) <ref_struct_t> a);
    inst!       ((vm, allocation_fastpath_v1) blk_entry_new1:
        a = NEW <struct_t>
    );

    inst!       ((vm, allocation_fastpath_v1) blk_entry_print1:
        PRINTHEX a
    );

    ssa!        ((vm, allocation_fastpath_v1) <ref_struct_t> b);
    inst!       ((vm, allocation_fastpath_v1) blk_entry_new2:
        b = NEW <struct_t>
    );

    inst!       ((vm, allocation_fastpath_v1) blk_entry_print2:
        PRINTHEX b
    );

    inst!       ((vm, allocation_fastpath_v1) blk_entry_threadexit:
        THREADEXIT
    );

    define_block!   ((vm, allocation_fastpath_v1) blk_entry() {
        blk_entry_new1, blk_entry_print1,
        blk_entry_new2, blk_entry_print2,
        blk_entry_threadexit
    });

    define_func_ver!((vm) allocation_fastpath_v1 (entry: blk_entry) {blk_entry});

    vm
}

#[test]
fn test_instruction_new() {
    VM::start_logging_trace();
    
    let vm = Arc::new(alloc_new());
    
    let compiler = Compiler::new(CompilerPolicy::default(), &vm);
    
    let func_id = vm.id_of("alloc_new");
    {
        let funcs = vm.funcs().read().unwrap();
        let func = funcs.get(&func_id).unwrap().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();
        let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();
        
        compiler.compile(&mut func_ver);
    }
    
    vm.set_primordial_thread(func_id, true, vec![]);
    backend::emit_context(&vm);
    
    let executable = aot::link_primordial(vec!["alloc_new".to_string()], "alloc_new_test", &vm);
    linkutils::exec_path(executable);
}

#[allow(dead_code)]
//#[test]
// The test won't work, since the generated dylib wants to use 'alloc_slow'.
// but in current process, there is no 'alloc_slow' (rust mangles it)
// The solution would be starting mu vm with libmu.so, then create IR from there.
// test_jit should contains a test for it. So I do not test it here
fn test_instruction_new_on_cur_thread() {
    VM::start_logging_trace();

    // compile
    let vm = Arc::new(alloc_new());
    let compiler = Compiler::new(CompilerPolicy::default(), &vm);
    let func_id = vm.id_of("alloc_new");
    {
        let funcs = vm.funcs().read().unwrap();
        let func = funcs.get(&func_id).unwrap().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();
        let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();

        compiler.compile(&mut func_ver);
    }

    backend::emit_context(&vm);

    // link
    let libname = &linkutils::get_dylib_name("alloc_new_on_cur_thread");
    let dylib = aot::link_dylib(vec![Mu("alloc_new")], libname, &vm);
    let lib = libloading::Library::new(dylib.as_os_str()).unwrap();

    unsafe {
        MuThread::current_thread_as_mu_thread(Address::zero(), vm.clone());
        let func : libloading::Symbol<unsafe extern fn() -> ()> = lib.get(b"alloc_new").unwrap();

        func();
    }
}

#[allow(unused_variables)]
pub fn alloc_new() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int64 = mu_int(64));
    typedef!    ((vm) ref_int64  = mu_ref(int64));
    typedef!    ((vm) iref_int64 = mu_iref(int64));
    
    constdef!   ((vm) <int64> int64_0 = Constant::Int(0));
    constdef!   ((vm) <int64> int64_1 = Constant::Int(1));

    funcsig!    ((vm) alloc_new_sig = () -> (int64));
    funcdecl!   ((vm) <alloc_new_sig> alloc_new);
    funcdef!    ((vm) <alloc_new_sig> alloc_new VERSION alloc_new_v1);
    
    // %blk_0():
    block!      ((vm, alloc_new_v1) blk_0);
    
    // %a = NEW <@int64_t>
    ssa!        ((vm, alloc_new_v1) <ref_int64> blk_0_a);
    inst!       ((vm, alloc_new_v1) blk_0_new:
        blk_0_a = NEW <int64>
    );
    
    // %a_iref = GETIREF <@int_64> @a
    ssa!        ((vm, alloc_new_v1) <iref_int64> blk_0_a_iref);
    inst!       ((vm, alloc_new_v1) blk_0_getiref:
        blk_0_a_iref = GETIREF blk_0_a
    );
    
    // STORE <@int_64> @a_iref @int_64_1
    consta!     ((vm, alloc_new_v1) int64_1_local = int64_1);
    inst!       ((vm, alloc_new_v1) blk_0_store:
        STORE blk_0_a_iref int64_1_local (is_ptr: false, order: MemoryOrder::Relaxed)
    );

    inst!       ((vm, alloc_new_v1) blk_0_term:
        THREADEXIT
    );

    define_block!((vm, alloc_new_v1) blk_0() {
        blk_0_new,
        blk_0_getiref,
        blk_0_store,
        blk_0_term
    });

    define_func_ver!((vm) alloc_new_v1 (entry: blk_0) {
        blk_0
    });
    
    vm
}
