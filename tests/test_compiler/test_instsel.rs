extern crate mu;
extern crate log;
extern crate simple_logger;

use test_ir::test_ir::factorial;
use self::mu::compiler::*;
use self::mu::vm::context::VMContext;

#[test]
fn test_instsel_fac() {
    simple_logger::init_with_level(log::LogLevel::Trace).ok();
    
    let vm_context : VMContext = factorial();
    let compiler = Compiler::new(CompilerPolicy::new(vec![
            Box::new(passes::DefUse::new()),
            Box::new(passes::TreeGen::new()),
            Box::new(passes::ControlFlowAnalysis::new()),
            Box::new(passes::TraceGen::new()),
            Box::new(backend::inst_sel::InstructionSelection::new())
    ]));
    
    let mut factorial_func = {
        vm_context.get_func("fac").unwrap().borrow_mut()
    };
    
    compiler.compile(&vm_context, &mut factorial_func);    
}