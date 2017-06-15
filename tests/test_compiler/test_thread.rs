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

#![allow(unused_imports)]

extern crate mu;

use self::mu::ast::types::*;
use self::mu::ast::ir::*;
use self::mu::ast::inst::*;
use self::mu::ast::op::*;
use self::mu::vm::*;
use self::mu::compiler::*;
use self::mu::utils::LinkedHashMap;
use self::mu::testutil::aot;

use std::sync::Arc;
use std::sync::RwLock;

#[test]
fn test_thread_create() {
    VM::start_logging_trace();
    
    let vm = Arc::new(primordial_main());
    
    let compiler = Compiler::new(CompilerPolicy::default(), &vm);
    
    let func_id = vm.id_of("primordial_main");    
    {
        let funcs = vm.funcs().read().unwrap();
        let func = funcs.get(&func_id).unwrap().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();
        let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();
        
        compiler.compile(&mut func_ver);
    }
    
    vm.make_primordial_thread(func_id, true, vec![]);
    backend::emit_context(&vm);
    
    let executable = aot::link_primordial(vec!["primordial_main".to_string()], "primordial_main_test", &vm);
    aot::execute(executable);
}

fn primordial_main() -> VM {
    let vm = VM::new();

    funcsig!    ((vm) sig = () -> ());
    funcdecl!   ((vm) <sig> primordial_main);
    funcdef!    ((vm) <sig> primordial_main VERSION primordial_main_v1);
    
    block!      ((vm, primordial_main_v1) blk_entry);
    inst!       ((vm, primordial_main_v1) blk_entry_threadexit:
        THREADEXIT
    );

    define_block!((vm, primordial_main_v1) blk_entry() {
        blk_entry_threadexit
    });

    define_func_ver!((vm) primordial_main_v1(entry: blk_entry) {
        blk_entry
    });
    
    vm
}
