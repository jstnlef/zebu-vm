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
    
    let compiler = Compiler::new(CompilerPolicy::new(vec![
        Box::new(passes::DefUse::new()),
        Box::new(passes::TreeGen::new()),
        Box::new(passes::ControlFlowAnalysis::new()),
        Box::new(passes::TraceGen::new()),
        Box::new(backend::inst_sel::InstructionSelection::new()),
        Box::new(backend::reg_alloc::RegisterAllocation::new()),
        Box::new(backend::peephole_opt::PeepholeOptimization::new()),
        Box::new(backend::code_emission::CodeEmission::new())
    ]), vm.clone());
    
    let funcs = vm.funcs().read().unwrap();
    let func = funcs.get("global_access").unwrap().borrow();
    let func_vers = vm.func_vers().read().unwrap();
    let mut func_ver = func_vers.get(&(func.fn_name, func.cur_ver.unwrap())).unwrap().borrow_mut();
    
    compiler.compile(&mut func_ver); 
    backend::emit_context(&vm);
}
