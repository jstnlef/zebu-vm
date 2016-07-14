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
    
    let vm_context = Arc::new(factorial());
    let compiler = Compiler::new(CompilerPolicy::new(
        vec![Box::new(passes::DefUse::new())]
    ), vm_context.clone());
    
    let funcs = vm_context.funcs().read().unwrap();
    let mut factorial_func = funcs.get("fac").unwrap().borrow_mut();
    
    compiler.compile(&mut factorial_func);
    
    assert!(factorial_func.context.get_value_by_tag("blk_0_n_3").unwrap().use_count.get() == 2, "blk_0_n_3 use should be 2");
    assert!(factorial_func.context.get_value_by_tag("blk_0_v48").unwrap().use_count.get() == 1, "blk_0_v48 use should be 1");
    assert!(factorial_func.context.get_value_by_tag("blk_2_v53").unwrap().use_count.get() == 1, "blk_2_v53 use should be 1");
    assert!(factorial_func.context.get_value_by_tag("blk_1_n_3").unwrap().use_count.get() == 2, "blk_1_n_3 use should be 2");
    assert!(factorial_func.context.get_value_by_tag("blk_1_v50").unwrap().use_count.get() == 1, "blk_1_v50 use should be 1");
    assert!(factorial_func.context.get_value_by_tag("blk_1_v51").unwrap().use_count.get() == 1, "blk_1_v51 use should be 1");
    assert!(factorial_func.context.get_value_by_tag("blk_1_v52").unwrap().use_count.get() == 1, "blk_1_v52 use should be 1");
}

#[test]
fn test_build_tree() {
    simple_logger::init_with_level(log::LogLevel::Trace).ok();
    
    let vm_context = Arc::new(factorial());
    let compiler = Compiler::new(CompilerPolicy::new(
        vec![Box::new(passes::DefUse::new()),
             Box::new(passes::TreeGen::new())]
    ), vm_context.clone());
    
    let funcs = vm_context.funcs().read().unwrap();
    let mut factorial_func = funcs.get("fac").unwrap().borrow_mut();
    
    compiler.compile(&mut factorial_func);
}

#[test]
fn test_cfa_factorial() {
    simple_logger::init_with_level(log::LogLevel::Trace).ok();
    
    let vm_context = Arc::new(factorial());
    let compiler = Compiler::new(CompilerPolicy::new(vec![
            Box::new(passes::DefUse::new()),
            Box::new(passes::TreeGen::new()),
            Box::new(passes::ControlFlowAnalysis::new())
    ]), vm_context.clone());
    
    let funcs = vm_context.funcs().read().unwrap();
    let mut factorial_func = funcs.get("fac").unwrap().borrow_mut();
    
    compiler.compile(&mut factorial_func);
    
    // assert cfa
    let content = factorial_func.content.as_ref().unwrap();
    
    // blk_0: preds=[], succs=[blk_2, blk_1]
    let blk_0 = content.get_block("blk_0");
    assert_vector_no_order(&blk_0.control_flow.preds, &vec![]);
    assert_vector_no_order(&block_edges_into_vec(&blk_0.control_flow.succs), &vec!["blk_2", "blk_1"]);
    
    // blk_2: preds=[blk_0, blk_1], succs=[]
    let blk_2 = content.get_block("blk_2");
    assert_vector_no_order(&blk_2.control_flow.preds, &vec!["blk_0", "blk_1"]);
    assert_vector_no_order(&block_edges_into_vec(&blk_2.control_flow.succs), &vec![]);
    
    // blk_1: preds=[blk_0], succs=[blk_2]
    let blk_1 = content.get_block("blk_1");
    assert_vector_no_order(&blk_1.control_flow.preds, &vec!["blk_0"]);
    assert_vector_no_order(&block_edges_into_vec(&blk_1.control_flow.succs), &vec!["blk_2"]);
}

#[test]
fn test_cfa_sum() {
    simple_logger::init_with_level(log::LogLevel::Trace).ok();
    
    let vm_context = Arc::new(sum());
    let compiler = Compiler::new(CompilerPolicy::new(vec![
            Box::new(passes::DefUse::new()),
            Box::new(passes::TreeGen::new()),
            Box::new(passes::ControlFlowAnalysis::new())
    ]), vm_context.clone());
    
    let funcs = vm_context.funcs().read().unwrap();
    let mut sum_func = funcs.get("sum").unwrap().borrow_mut();
    
    compiler.compile(&mut sum_func);
    
    // assert cfa
    let content = sum_func.content.as_ref().unwrap();
    
    // entry: preds=[], succs=[head]
    let entry = content.get_block("entry");
    assert_vector_no_order(&entry.control_flow.preds, &vec![]);
    assert_vector_no_order(&block_edges_into_vec(&entry.control_flow.succs), &vec!["head"]);
    
    // head: preds=[entry, head], succs=[head, ret]
    let head = content.get_block("head");
    assert_vector_no_order(&head.control_flow.preds, &vec!["entry", "head"]);
    assert_vector_no_order(&block_edges_into_vec(&head.control_flow.succs), &vec!["ret", "head"]);
    
    // ret: preds=[head], succs=[]
    let ret = content.get_block("ret");
    assert_vector_no_order(&ret.control_flow.preds, &vec!["head"]);
    assert_vector_no_order(&block_edges_into_vec(&ret.control_flow.succs), &vec![]);
}

fn block_edges_into_vec(edges: &Vec<BlockEdge>) -> Vec<&str> {
    let mut ret = vec![];
    for edge in edges {
        ret.push(edge.target);
    }
    ret
}

#[test]
fn test_trace_factorial() {
    simple_logger::init_with_level(log::LogLevel::Trace).ok();
    
    let vm_context = Arc::new(factorial());
    let compiler = Compiler::new(CompilerPolicy::new(vec![
            Box::new(passes::DefUse::new()),
            Box::new(passes::TreeGen::new()),
            Box::new(passes::ControlFlowAnalysis::new()),
            Box::new(passes::TraceGen::new())
    ]), vm_context.clone());
    
    let funcs = vm_context.funcs().read().unwrap();
    let mut factorial_func = funcs.get("fac").unwrap().borrow_mut();
    
    compiler.compile(&mut factorial_func);
    
    assert_vector_ordered(factorial_func.block_trace.as_ref().unwrap(), &vec!["blk_0", "blk_1", "blk_2"]);
}

#[test]
fn test_trace_sum() {
    simple_logger::init_with_level(log::LogLevel::Trace).ok();
    
    let vm_context = Arc::new(sum());
    let compiler = Compiler::new(CompilerPolicy::new(vec![
            Box::new(passes::DefUse::new()),
            Box::new(passes::TreeGen::new()),
            Box::new(passes::ControlFlowAnalysis::new()),
            Box::new(passes::TraceGen::new())
    ]), vm_context.clone());
    
    let funcs = vm_context.funcs().read().unwrap();
    let mut sum_func = funcs.get("sum").unwrap().borrow_mut();
    
    compiler.compile(&mut sum_func);
    
    assert_vector_ordered(sum_func.block_trace.as_ref().unwrap(), &vec!["entry", "head", "ret"]);
}
