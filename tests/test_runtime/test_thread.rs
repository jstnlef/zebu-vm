#![allow(unused_imports)]

extern crate mu;
extern crate log;
extern crate simple_logger;

use self::mu::ast::types::*;
use self::mu::ast::ir::*;
use self::mu::ast::inst::*;
use self::mu::ast::op::*;
use self::mu::vm::*;
use self::mu::compiler::*;

use test_ir::test_ir::factorial;

use std::sync::Arc;

#[test]
fn test_thread_create() {
    let vm = Arc::new(factorial());
    
    let compiler = Compiler::new(CompilerPolicy::default(), vm.clone());
    
    let func_id = vm.id_of("fac");    
    {
        let funcs = vm.funcs().read().unwrap();
        let func = funcs.get(&func_id).unwrap().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();
        let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();
        
        compiler.compile(&mut func_ver);
    }
    
    vm.make_primordial_thread(func_id, vec![Constant::Int(5)]);
    backend::emit_context(&vm);
}