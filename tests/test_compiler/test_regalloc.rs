extern crate mu;
extern crate log;
extern crate simple_logger;

use test_ir::test_ir::factorial;
use self::mu::compiler::*;
use self::mu::utils::vec_utils;
use self::mu::ast::ir::*;

use std::sync::Arc;

#[test]
fn test_ir_liveness_fac() {
    simple_logger::init_with_level(log::LogLevel::Trace).ok();
    
    let vm_context = Arc::new(factorial());
    
    let compiler = Compiler::new(CompilerPolicy::new(vec![
            Box::new(passes::DefUse::new()),
            Box::new(passes::TreeGen::new()),
            Box::new(passes::ControlFlowAnalysis::new()),
            Box::new(passes::TraceGen::new()),
            Box::new(backend::inst_sel::InstructionSelection::new()),
    ]), vm_context.clone());
    
    let funcs = vm_context.funcs().read().unwrap();
    let mut factorial_func = funcs.get("fac").unwrap().borrow_mut();
    
    compiler.compile(&mut factorial_func);
    
    let cf_lock = vm_context.compiled_funcs().read().unwrap();
    let cf = cf_lock.get("fac").unwrap().borrow();
    
    // block 0
    
    let block_0_livein = cf.mc.get_ir_block_livein("blk_0").unwrap();
    let blk_0_n_3 = factorial_func.context.get_value_by_tag("blk_0_n_3").unwrap().id;
    assert!(vec_utils::is_identical_to_str_ignore_order(block_0_livein, vec![blk_0_n_3]));
    
    let block_0_liveout = cf.mc.get_ir_block_liveout("blk_0").unwrap();
    assert!(vec_utils::is_identical_to_str_ignore_order(block_0_liveout, vec![blk_0_n_3]));
    
    // block 1
    
    let block_1_livein = cf.mc.get_ir_block_livein("blk_1").unwrap();
    let blk_1_n_3 = factorial_func.context.get_value_by_tag("blk_1_n_3").unwrap().id;
    assert!(vec_utils::is_identical_to_str_ignore_order(block_1_livein, vec![blk_1_n_3]));
    
    let block_1_liveout = cf.mc.get_ir_block_liveout("blk_1").unwrap();
    let blk_1_v52 = factorial_func.context.get_value_by_tag("blk_1_v52").unwrap().id;
    assert!(vec_utils::is_identical_to_str_ignore_order(block_1_liveout, vec![blk_1_v52]));
    
    // block 2
    
    let block_2_livein = cf.mc.get_ir_block_livein("blk_2").unwrap();
    let blk_2_v53 = factorial_func.context.get_value_by_tag("blk_2_v53").unwrap().id;
    assert!(vec_utils::is_identical_to_str_ignore_order(block_2_livein, vec![blk_2_v53]));
    
    let block_2_liveout = cf.mc.get_ir_block_liveout("blk_2").unwrap();
    let expect : Vec<MuID> = vec![];
    assert!(vec_utils::is_identical_to_str_ignore_order(block_2_liveout, expect));
}

#[test]
fn test_regalloc_fac() {
    simple_logger::init_with_level(log::LogLevel::Trace).ok();
    
    let vm_context = Arc::new(factorial());
    
    let compiler = Compiler::new(CompilerPolicy::new(vec![
            Box::new(passes::DefUse::new()),
            Box::new(passes::TreeGen::new()),
            Box::new(passes::ControlFlowAnalysis::new()),
            Box::new(passes::TraceGen::new()),
            Box::new(backend::inst_sel::InstructionSelection::new()),
            Box::new(backend::reg_alloc::RegisterAllocation::new()),
            Box::new(backend::peephole_opt::PeepholeOptimization::new()),
            Box::new(backend::code_emission::CodeEmission::new())
    ]), vm_context.clone());
    
    let funcs = vm_context.funcs().read().unwrap();
    let mut factorial_func = funcs.get("fac").unwrap().borrow_mut();
    
    compiler.compile(&mut factorial_func);    
}
