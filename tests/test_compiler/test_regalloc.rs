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
    
    let vm = Arc::new(factorial());
    
    let compiler = Compiler::new(CompilerPolicy::new(vec![
            Box::new(passes::DefUse::new()),
            Box::new(passes::TreeGen::new()),
            Box::new(passes::ControlFlowAnalysis::new()),
            Box::new(passes::TraceGen::new()),
            Box::new(backend::inst_sel::InstructionSelection::new()),
    ]), vm.clone());
    
    let func_id = vm.id_of("fac");
    let funcs = vm.funcs().read().unwrap();
    let func = funcs.get(&func_id).unwrap().read().unwrap();
    let func_vers = vm.func_vers().read().unwrap();
    let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();
    
    compiler.compile(&mut func_ver);
    
    let cf_lock = vm.compiled_funcs().read().unwrap();
    let cf = cf_lock.get(&func_ver.id()).unwrap().read().unwrap();
    
    // block 0
    
    let block_0_livein = cf.mc.get_ir_block_livein("blk_0").unwrap();
    let blk_0_n_3 = vm.id_of("blk_0_n_3");;
    assert!(vec_utils::is_identical_to_str_ignore_order(block_0_livein, vec![blk_0_n_3]));
    
    let block_0_liveout = cf.mc.get_ir_block_liveout("blk_0").unwrap();
    assert!(vec_utils::is_identical_to_str_ignore_order(block_0_liveout, vec![blk_0_n_3]));
    
    // block 1
    
    let block_1_livein = cf.mc.get_ir_block_livein("blk_1").unwrap();
    let blk_1_n_3 = vm.id_of("blk_1_n_3");
    assert!(vec_utils::is_identical_to_str_ignore_order(block_1_livein, vec![blk_1_n_3]));
    
    let block_1_liveout = cf.mc.get_ir_block_liveout("blk_1").unwrap();
    let blk_1_v52 = vm.id_of("blk_1_v52");
    assert!(vec_utils::is_identical_to_str_ignore_order(block_1_liveout, vec![blk_1_v52]));
    
    // block 2
    
    let block_2_livein = cf.mc.get_ir_block_livein("blk_2").unwrap();
    let blk_2_v53 = vm.id_of("blk_2_v53");
    assert!(vec_utils::is_identical_to_str_ignore_order(block_2_livein, vec![blk_2_v53]));
    
    let block_2_liveout = cf.mc.get_ir_block_liveout("blk_2").unwrap();
    let expect : Vec<MuID> = vec![];
    assert!(vec_utils::is_identical_to_str_ignore_order(block_2_liveout, expect));
}

#[test]
fn test_regalloc_fac() {
    simple_logger::init_with_level(log::LogLevel::Trace).ok();
    
    let vm = Arc::new(factorial());
    
    let compiler = Compiler::new(CompilerPolicy::default(), vm.clone());
    
    let func_id = vm.id_of("fac");
    let funcs = vm.funcs().read().unwrap();
    let func = funcs.get(&func_id).unwrap().read().unwrap();
    let func_vers = vm.func_vers().read().unwrap();
    let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();
    
    compiler.compile(&mut func_ver);
}
