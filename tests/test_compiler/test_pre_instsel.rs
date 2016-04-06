extern crate mu;
extern crate log;
extern crate simple_logger;

use test_ir::test_ir::factorial;
use self::mu::compiler::*;
use self::mu::vm::context::VMContext;

#[test]
fn test_use_count() {
    simple_logger::init_with_level(log::LogLevel::Trace).ok();
    
    let vm_context : VMContext = factorial();
    let compiler = Compiler::new(CompilerPolicy::new(
        vec![Box::new(passes::DefUse::new())]
    ));
    
    let mut factorial_func = {
        vm_context.get_func("fac").unwrap().borrow_mut()
    };
    
    compiler.compile(&vm_context, &mut factorial_func);
    
    assert!(factorial_func.context.get_value(0).unwrap().use_count.get() == 2, "blk_0_n_3 use should be 2");
    assert!(factorial_func.context.get_value(1).unwrap().use_count.get() == 1, "blk_0_v48 use should be 1");
    assert!(factorial_func.context.get_value(2).unwrap().use_count.get() == 1, "blk_2_v53 use should be 1");
    assert!(factorial_func.context.get_value(3).unwrap().use_count.get() == 2, "blk_1_n_3 use should be 2");
    assert!(factorial_func.context.get_value(4).unwrap().use_count.get() == 1, "blk_1_v50 use should be 1");
    assert!(factorial_func.context.get_value(5).unwrap().use_count.get() == 1, "blk_1_v51 use should be 1");
    assert!(factorial_func.context.get_value(6).unwrap().use_count.get() == 1, "blk_1_fac use should be 1");
    assert!(factorial_func.context.get_value(7).unwrap().use_count.get() == 1, "blk_1_v52 use should be 1");
}

#[test]
fn test_build_tree() {
    simple_logger::init_with_level(log::LogLevel::Trace).ok();
    
    let vm_context : VMContext = factorial();
    let compiler = Compiler::new(CompilerPolicy::new(
        vec![Box::new(passes::DefUse::new()),
             Box::new(passes::TreeGen::new())]
    ));
    
    let mut factorial_func = {
        vm_context.get_func("fac").unwrap().borrow_mut()
    };
    
    compiler.compile(&vm_context, &mut factorial_func);
}

#[test]
fn test_cfa() {
    simple_logger::init_with_level(log::LogLevel::Trace).ok();
    
    let vm_context : VMContext = factorial();
    let compiler = Compiler::new(CompilerPolicy::new(vec![
            Box::new(passes::DefUse::new()),
            Box::new(passes::TreeGen::new()),
            Box::new(passes::ControlFlowAnalysis::new())
    ]));
    
    let mut factorial_func = {
        vm_context.get_func("fac").unwrap().borrow_mut()
    };
    
    compiler.compile(&vm_context, &mut factorial_func);    
}