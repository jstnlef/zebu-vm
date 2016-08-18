extern crate mu;
extern crate log;
extern crate simple_logger;

use common::*;
use test_ir::test_ir::factorial;
use test_ir::test_ir::sum;
use self::mu::ast::ir::*;
use self::mu::compiler::*;

use std::sync::Arc;

#[test]
fn test_use_count() {
    simple_logger::init_with_level(log::LogLevel::Trace).ok();
    
    let vm = Arc::new(factorial());
    let compiler = Compiler::new(CompilerPolicy::new(
        vec![Box::new(passes::DefUse::new())]
    ), vm.clone());
    
    let func_id = vm.id_of("fac");
    let funcs = vm.funcs().read().unwrap();
    let func = funcs.get(&func_id).unwrap().read().unwrap();
    let func_vers = vm.func_vers().read().unwrap();
    let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();
    
    compiler.compile(&mut func_ver);
    
    assert!(func_ver.context.get_value_by_tag("blk_0_n_3").unwrap().use_count.get() == 2, "blk_0_n_3 use should be 2");
    assert!(func_ver.context.get_value_by_tag("blk_0_v48").unwrap().use_count.get() == 1, "blk_0_v48 use should be 1");
    assert!(func_ver.context.get_value_by_tag("blk_2_v53").unwrap().use_count.get() == 1, "blk_2_v53 use should be 1");
    assert!(func_ver.context.get_value_by_tag("blk_1_n_3").unwrap().use_count.get() == 2, "blk_1_n_3 use should be 2");
    assert!(func_ver.context.get_value_by_tag("blk_1_v50").unwrap().use_count.get() == 1, "blk_1_v50 use should be 1");
    assert!(func_ver.context.get_value_by_tag("blk_1_v51").unwrap().use_count.get() == 1, "blk_1_v51 use should be 1");
    assert!(func_ver.context.get_value_by_tag("blk_1_v52").unwrap().use_count.get() == 1, "blk_1_v52 use should be 1");
}

#[test]
fn test_build_tree() {
    simple_logger::init_with_level(log::LogLevel::Trace).ok();
    
    let vm = Arc::new(factorial());
    let compiler = Compiler::new(CompilerPolicy::new(
        vec![Box::new(passes::DefUse::new()),
             Box::new(passes::TreeGen::new())]
    ), vm.clone());
    
    let func_id = vm.id_of("fac");
    let funcs = vm.funcs().read().unwrap();
    let func = funcs.get(&func_id).unwrap().read().unwrap();
    let func_vers = vm.func_vers().read().unwrap();
    let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();
    
    compiler.compile(&mut func_ver);
}

#[test]
fn test_cfa_factorial() {
    simple_logger::init_with_level(log::LogLevel::Trace).ok();
    
    let vm = Arc::new(factorial());
    let compiler = Compiler::new(CompilerPolicy::new(vec![
            Box::new(passes::DefUse::new()),
            Box::new(passes::TreeGen::new()),
            Box::new(passes::ControlFlowAnalysis::new())
    ]), vm.clone());
    
    let func_id = vm.id_of("fac");
    let funcs = vm.funcs().read().unwrap();
    let func = funcs.get(&func_id).unwrap().read().unwrap();
    let func_vers = vm.func_vers().read().unwrap();
    let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();
    
    compiler.compile(&mut func_ver);
    
    // assert cfa
    let content = func_ver.content.as_ref().unwrap();
    
    // blk_0: preds=[], succs=[blk_2, blk_1]
    let (blk_0_id, blk_1_id, blk_2_id) = (vm.id_of("blk_0"), vm.id_of("blk_1"), vm.id_of("blk_2"));
    
    let blk_0 = content.get_block(blk_0_id);
    assert_vector_no_order(&blk_0.control_flow.preds, &vec![]);
    assert_vector_no_order(&block_edges_into_vec(&blk_0.control_flow.succs), &vec![blk_2_id, blk_1_id]);
    
    // blk_2: preds=[blk_0, blk_1], succs=[]
    let blk_2 = content.get_block(blk_2_id);
    assert_vector_no_order(&blk_2.control_flow.preds, &vec![blk_0_id, blk_1_id]);
    assert_vector_no_order(&block_edges_into_vec(&blk_2.control_flow.succs), &vec![]);
    
    // blk_1: preds=[blk_0], succs=[blk_2]
    let blk_1 = content.get_block(blk_1_id);
    assert_vector_no_order(&blk_1.control_flow.preds, &vec![blk_0_id]);
    assert_vector_no_order(&block_edges_into_vec(&blk_1.control_flow.succs), &vec![blk_2_id]);
}

#[test]
fn test_cfa_sum() {
    simple_logger::init_with_level(log::LogLevel::Trace).ok();
    
    let vm = Arc::new(sum());
    let compiler = Compiler::new(CompilerPolicy::new(vec![
            Box::new(passes::DefUse::new()),
            Box::new(passes::TreeGen::new()),
            Box::new(passes::ControlFlowAnalysis::new())
    ]), vm.clone());
    
    let func_id = vm.id_of("sum");
    let funcs = vm.funcs().read().unwrap();
    let func = funcs.get(&func_id).unwrap().read().unwrap();
    let func_vers = vm.func_vers().read().unwrap();
    let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();
    
    compiler.compile(&mut func_ver);
    
    // assert cfa
    let content = func_ver.content.as_ref().unwrap();
    
    let entry_id = vm.id_of("entry");
    let head_id  = vm.id_of("head");
    let ret_id   = vm.id_of("ret");
    
    // entry: preds=[], succs=[head]
    let entry = content.get_block(entry_id);
    assert_vector_no_order(&entry.control_flow.preds, &vec![]);
    assert_vector_no_order(&block_edges_into_vec(&entry.control_flow.succs), &vec![head_id]);
    
    // head: preds=[entry, head], succs=[head, ret]
    let head = content.get_block(head_id);
    assert_vector_no_order(&head.control_flow.preds, &vec![entry_id, head_id]);
    assert_vector_no_order(&block_edges_into_vec(&head.control_flow.succs), &vec![ret_id, head_id]);
    
    // ret: preds=[head], succs=[]
    let ret = content.get_block(ret_id);
    assert_vector_no_order(&ret.control_flow.preds, &vec![head_id]);
    assert_vector_no_order(&block_edges_into_vec(&ret.control_flow.succs), &vec![]);
}

fn block_edges_into_vec(edges: &Vec<BlockEdge>) -> Vec<MuID> {
    let mut ret = vec![];
    for edge in edges {
        ret.push(edge.target);
    }
    ret
}

#[test]
fn test_trace_factorial() {
    simple_logger::init_with_level(log::LogLevel::Trace).ok();
    
    let vm = Arc::new(factorial());
    let compiler = Compiler::new(CompilerPolicy::new(vec![
            Box::new(passes::DefUse::new()),
            Box::new(passes::TreeGen::new()),
            Box::new(passes::ControlFlowAnalysis::new()),
            Box::new(passes::TraceGen::new())
    ]), vm.clone());
    
    let func_id = vm.id_of("fac");
    let funcs = vm.funcs().read().unwrap();
    let func = funcs.get(&func_id).unwrap().read().unwrap();
    let func_vers = vm.func_vers().read().unwrap();
    let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();
    
    compiler.compile(&mut func_ver);
    
    assert_vector_ordered(func_ver.block_trace.as_ref().unwrap(), &vec![vm.id_of("blk_0"), vm.id_of("blk_1"), vm.id_of("blk_2")]);
}

#[test]
fn test_trace_sum() {
    simple_logger::init_with_level(log::LogLevel::Trace).ok();
    
    let vm = Arc::new(sum());
    let compiler = Compiler::new(CompilerPolicy::new(vec![
            Box::new(passes::DefUse::new()),
            Box::new(passes::TreeGen::new()),
            Box::new(passes::ControlFlowAnalysis::new()),
            Box::new(passes::TraceGen::new())
    ]), vm.clone());
    
    let func_id = vm.id_of("sum");
    let funcs = vm.funcs().read().unwrap();
    let func = funcs.get(&func_id).unwrap().read().unwrap();
    let func_vers = vm.func_vers().read().unwrap();
    let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();
    
    compiler.compile(&mut func_ver);
    
    assert_vector_ordered(func_ver.block_trace.as_ref().unwrap(), &vec![vm.id_of("entry"), vm.id_of("head"), vm.id_of("ret")]);
}
