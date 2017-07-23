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

extern crate mu;

use test_ir::test_ir::factorial;
use test_ir::test_ir::sum;
use self::mu::ast::ir::*;
use self::mu::compiler::*;
use self::mu::vm::VM;

use std::sync::Arc;

#[test]
fn test_use_count() {
    VM::start_logging_trace();

    let vm = Arc::new(factorial());
    let compiler = Compiler::new(
        CompilerPolicy::new(vec![Box::new(passes::DefUse::new())]),
        &vm,
    );

    let func_id = vm.id_of("fac");
    let funcs = vm.funcs().read().unwrap();
    let func = funcs.get(&func_id).unwrap().read().unwrap();
    let func_vers = vm.func_vers().read().unwrap();
    let mut func_ver = func_vers
        .get(&func.cur_ver.unwrap())
        .unwrap()
        .write()
        .unwrap();

    compiler.compile(&mut func_ver);

    assert!(
        func_ver
            .context
            .get_value(vm.id_of("blk_0_n_3"))
            .unwrap()
            .use_count() == 2,
        "blk_0_n_3 use should be 2"
    );
    assert!(
        func_ver
            .context
            .get_value(vm.id_of("blk_0_v48"))
            .unwrap()
            .use_count() == 1,
        "blk_0_v48 use should be 1"
    );
    assert!(
        func_ver
            .context
            .get_value(vm.id_of("blk_2_v53"))
            .unwrap()
            .use_count() == 1,
        "blk_2_v53 use should be 1"
    );
    assert!(
        func_ver
            .context
            .get_value(vm.id_of("blk_1_n_3"))
            .unwrap()
            .use_count() == 2,
        "blk_1_n_3 use should be 2"
    );
    assert!(
        func_ver
            .context
            .get_value(vm.id_of("blk_1_v50"))
            .unwrap()
            .use_count() == 1,
        "blk_1_v50 use should be 1"
    );
    assert!(
        func_ver
            .context
            .get_value(vm.id_of("blk_1_v51"))
            .unwrap()
            .use_count() == 1,
        "blk_1_v51 use should be 1"
    );
    assert!(
        func_ver
            .context
            .get_value(vm.id_of("blk_1_v52"))
            .unwrap()
            .use_count() == 1,
        "blk_1_v52 use should be 1"
    );
}

#[test]
fn test_build_tree() {
    VM::start_logging_trace();

    let vm = Arc::new(factorial());
    let compiler = Compiler::new(
        CompilerPolicy::new(vec![
            Box::new(passes::DefUse::new()),
            Box::new(passes::TreeGen::new()),
        ]),
        &vm,
    );

    let func_id = vm.id_of("fac");
    let funcs = vm.funcs().read().unwrap();
    let func = funcs.get(&func_id).unwrap().read().unwrap();
    let func_vers = vm.func_vers().read().unwrap();
    let mut func_ver = func_vers
        .get(&func.cur_ver.unwrap())
        .unwrap()
        .write()
        .unwrap();

    compiler.compile(&mut func_ver);
}

// consider one intermediate block
fn is_successor(from_id: MuID, to_id: MuID, content: &FunctionContent) -> bool {
    let blk_from = content.get_block(from_id);

    for outedge in blk_from.control_flow.succs.iter() {
        if outedge.target == to_id {
            return true;
        }

        let intermediate_block = content.get_block(outedge.target);

        for int_outedge in intermediate_block.control_flow.succs.iter() {
            if int_outedge.target == to_id {
                return true;
            }
        }
    }

    return false;
}

fn has_successor(id: MuID, content: &FunctionContent) -> bool {
    let blk = content.get_block(id);

    !blk.control_flow.succs.is_empty()
}

fn is_predecessor(from_id: MuID, to_id: MuID, content: &FunctionContent) -> bool {
    let blk_from = content.get_block(from_id);

    for pred in blk_from.control_flow.preds.iter() {
        if *pred == to_id {
            return true;
        }

        let intermediate_block = content.get_block(*pred);

        for int_pred in intermediate_block.control_flow.preds.iter() {
            if *int_pred == to_id {
                return true;
            }
        }
    }

    return false;
}

fn has_predecessor(id: MuID, content: &FunctionContent) -> bool {
    let blk = content.get_block(id);

    !blk.control_flow.preds.is_empty()
}

#[test]
fn test_cfa_factorial() {
    VM::start_logging_trace();

    let vm = Arc::new(factorial());
    let compiler = Compiler::new(
        CompilerPolicy::new(vec![
            Box::new(passes::DefUse::new()),
            Box::new(passes::TreeGen::new()),
            Box::new(passes::GenMovPhi::new()),
            Box::new(passes::ControlFlowAnalysis::new()),
        ]),
        &vm,
    );

    let func_id = vm.id_of("fac");
    let funcs = vm.funcs().read().unwrap();
    let func = funcs.get(&func_id).unwrap().read().unwrap();
    let func_vers = vm.func_vers().read().unwrap();
    let mut func_ver = func_vers
        .get(&func.cur_ver.unwrap())
        .unwrap()
        .write()
        .unwrap();

    compiler.compile(&mut func_ver);

    // assert cfa
    let content = func_ver.content.as_ref().unwrap();

    let (blk_0_id, blk_1_id, blk_2_id) = (vm.id_of("blk_0"), vm.id_of("blk_1"), vm.id_of("blk_2"));

    // blk_0: preds=[], succs=[blk_2, blk_1] - however there will be intermediate block
    // check blk_0 predecessor
    assert!(!has_predecessor(blk_0_id, content));
    // check blk_0 successor
    assert!(is_successor(blk_0_id, blk_1_id, content));
    assert!(is_successor(blk_0_id, blk_2_id, content));

    // blk_2: preds=[blk_0, blk_1], succs=[]
    // check blk_2 predecessor
    assert!(is_predecessor(blk_2_id, blk_0_id, content));
    assert!(is_predecessor(blk_2_id, blk_1_id, content));
    // check blk_2 successor
    assert!(!has_successor(blk_2_id, content));

    // blk_1: preds=[blk_0], succs=[blk_2]
    // check blk_1 predecessor
    assert!(is_predecessor(blk_1_id, blk_0_id, content));
    // check blk_1 successor
    assert!(is_successor(blk_1_id, blk_2_id, content));
}

#[test]
fn test_cfa_sum() {
    VM::start_logging_trace();

    let vm = Arc::new(sum());
    let compiler = Compiler::new(
        CompilerPolicy::new(vec![
            Box::new(passes::DefUse::new()),
            Box::new(passes::TreeGen::new()),
            Box::new(passes::GenMovPhi::new()),
            Box::new(passes::ControlFlowAnalysis::new()),
        ]),
        &vm,
    );

    let func_id = vm.id_of("sum");
    let funcs = vm.funcs().read().unwrap();
    let func = funcs.get(&func_id).unwrap().read().unwrap();
    let func_vers = vm.func_vers().read().unwrap();
    let mut func_ver = func_vers
        .get(&func.cur_ver.unwrap())
        .unwrap()
        .write()
        .unwrap();

    compiler.compile(&mut func_ver);

    // assert cfa
    let content = func_ver.content.as_ref().unwrap();

    let entry = vm.id_of("blk_entry");
    let head = vm.id_of("blk_head");
    let ret = vm.id_of("blk_ret");

    // entry: preds=[], succs=[head]
    assert!(!has_predecessor(entry, content));
    assert!(is_successor(entry, head, content));

    // head: preds=[entry, head], succs=[head, ret]
    assert!(is_predecessor(head, entry, content));
    assert!(is_predecessor(head, head, content));
    assert!(is_successor(head, head, content));
    assert!(is_successor(head, ret, content));

    // ret: preds=[head], succs=[]
    assert!(is_predecessor(ret, head, content));
    assert!(!has_successor(ret, content));
}

// as long as expected appears in correct order in actual, it is correct
fn match_trace(actual: &Vec<MuID>, expected: &Vec<MuID>) -> bool {
    assert!(actual.len() >= expected.len());

    debug!("matching trace:");
    debug!("actual: {:?}", actual);
    debug!("expected: {:?}", expected);

    let mut expected_cursor = 0;

    for i in actual {
        if *i == expected[expected_cursor] {
            expected_cursor += 1;

            if expected_cursor == expected.len() {
                return true;
            }
        }
    }

    return false;
}

#[test]
fn test_trace_factorial() {
    VM::start_logging_trace();

    let vm = Arc::new(factorial());
    let compiler = Compiler::new(
        CompilerPolicy::new(vec![
            Box::new(passes::DefUse::new()),
            Box::new(passes::TreeGen::new()),
            Box::new(passes::GenMovPhi::new()),
            Box::new(passes::ControlFlowAnalysis::new()),
            Box::new(passes::TraceGen::new()),
        ]),
        &vm,
    );

    let func_id = vm.id_of("fac");
    let funcs = vm.funcs().read().unwrap();
    let func = funcs.get(&func_id).unwrap().read().unwrap();
    let func_vers = vm.func_vers().read().unwrap();
    let mut func_ver = func_vers
        .get(&func.cur_ver.unwrap())
        .unwrap()
        .write()
        .unwrap();

    compiler.compile(&mut func_ver);

    assert!(match_trace(
        func_ver.block_trace.as_ref().unwrap(),
        &vec![vm.id_of("blk_0"), vm.id_of("blk_1"), vm.id_of("blk_2")]
    ));
}

#[test]
fn test_trace_sum() {
    VM::start_logging_trace();

    let vm = Arc::new(sum());
    let compiler = Compiler::new(
        CompilerPolicy::new(vec![
            Box::new(passes::DefUse::new()),
            Box::new(passes::TreeGen::new()),
            Box::new(passes::GenMovPhi::new()),
            Box::new(passes::ControlFlowAnalysis::new()),
            Box::new(passes::TraceGen::new()),
        ]),
        &vm,
    );

    let func_id = vm.id_of("sum");
    let funcs = vm.funcs().read().unwrap();
    let func = funcs.get(&func_id).unwrap().read().unwrap();
    let func_vers = vm.func_vers().read().unwrap();
    let mut func_ver = func_vers
        .get(&func.cur_ver.unwrap())
        .unwrap()
        .write()
        .unwrap();

    compiler.compile(&mut func_ver);

    assert!(match_trace(
        func_ver.block_trace.as_ref().unwrap(),
        &vec![
            vm.id_of("blk_entry"),
            vm.id_of("blk_head"),
            vm.id_of("blk_ret"),
        ]
    ));
}
