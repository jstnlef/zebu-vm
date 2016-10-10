extern crate mu;
extern crate log;
extern crate simple_logger;

use test_ir::test_ir::global_access;
use self::mu::compiler::*;

use std::sync::Arc;

#[test]
fn test_global_access() {
    simple_logger::init_with_level(log::LogLevel::Trace).ok();
    
    let vm = Arc::new(global_access());
    
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
