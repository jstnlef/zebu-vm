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

use ast::ir::*;
use ast::inst::Instruction_::*;
use utils::vec_utils::as_str as vector_as_str;
use vm::VM;
use compiler::CompilerPass;
use utils::LinkedHashMap;
use utils::LinkedHashSet;
use std::any::Any;

pub struct ControlFlowAnalysis {
    name: &'static str
}

impl ControlFlowAnalysis {
    pub fn new() -> ControlFlowAnalysis {
        ControlFlowAnalysis{name: "Control Flow Analysis"}
    }
}

fn check_edge_kind(target: MuID, stack: &Vec<MuID>) -> EdgeKind {
    if stack.contains(&target) {
        EdgeKind::Backward
    } else {
        EdgeKind::Forward
    }
}

fn new_edge(cur: MuID, edge: BlockEdge, stack: &mut Vec<MuID>, visited: &mut Vec<MuID>, func: &mut MuFunctionVersion) {
    // add current block to target's predecessors
    {
        let target = func.content.as_mut().unwrap().get_block_mut(edge.target);
        target.control_flow.preds.push(cur);
    }

    // add target as current block's successors and start dfs
    let succ = edge.target;
    {
        let cur = func.content.as_mut().unwrap().get_block_mut(cur);
        cur.control_flow.succs.push(edge);
    }
    if !visited.contains(&succ) {
        dfs(succ, stack, visited, func);
    }

}

const WATCHPOINT_DISABLED_CHANCE : f32 = 0.9f32;

const NORMAL_RESUME_CHANCE       : f32 = 0.6f32;
const EXN_RESUME_CHANCE          : f32 = 1f32 - NORMAL_RESUME_CHANCE;

fn dfs(cur: MuID, stack: &mut Vec<MuID>, visited: &mut Vec<MuID>, func: &mut MuFunctionVersion) {
    trace!("dfs visiting block {}", cur);
    trace!("current stack: {:?}", stack);
    trace!("current visited: {:?}", visited);

    stack.push(cur);
    visited.push(cur);

    // find all the successors for current block, and push them to the stack
    let out_edges : Vec<BlockEdge> = {
        let cur = func.content.as_mut().unwrap().get_block_mut(cur);
        let ref body = cur.content.as_ref().unwrap().body;
        let last_inst = body.last().unwrap();

        match last_inst.v {
            TreeNode_::Instruction(ref inst) => {
                match inst.v {
                    // unconditional branch, definitely branch to the target
                    Branch1(ref dest) => vec![BlockEdge{
                        target: dest.target,
                        kind: check_edge_kind(dest.target, stack),
                        is_exception: false,
                        probability: 1.0f32
                    }],

                    // conditional branch
                    Branch2{ref true_dest, ref false_dest, true_prob, ..} => vec![
                        BlockEdge{
                            target: true_dest.target,
                            kind: check_edge_kind(true_dest.target, stack),
                            is_exception: false,
                            probability: true_prob
                        },
                        BlockEdge{
                            target: false_dest.target,
                            kind: check_edge_kind(false_dest.target, stack),
                            is_exception: false,
                            probability: 1.0f32 - true_prob
                        }
                    ],

                    // switch
                    Switch{ref default, ref branches, ..} => {
                        const BRANCH_DEFAULT_PROB : f32 = 0.1;
                        let switch_prob = (1.0f32 - BRANCH_DEFAULT_PROB) / (branches.len() as f32);

                        let map : LinkedHashMap<MuID, BlockEdge> = {
                            let mut ret = LinkedHashMap::new();

                            let check_add_edge = |map: &mut LinkedHashMap<MuID, BlockEdge>, target: MuID, prob: f32| {
                                if map.contains_key(&target) {
                                    let mut edge : &mut BlockEdge = map.get_mut(&target).unwrap();
                                    edge.probability += prob;
                                } else {
                                    map.insert(target, BlockEdge{
                                        target: target,
                                        kind: check_edge_kind(target, stack),
                                        is_exception: false,
                                        probability: prob
                                    });
                                }
                            };

                            for &(_, ref dest) in branches.iter() {
                                let target = dest.target;

                                check_add_edge(&mut ret, target, switch_prob);
                            }

                            check_add_edge(&mut ret, default.target, BRANCH_DEFAULT_PROB);

                            ret
                        };

                        let mut ret = vec![];

                        for edge in map.values() {
                            ret.push(*edge);
                        }

                        ret
                    }

                    // watchpoints
                    Watchpoint{ref id, ref disable_dest, ref resume} => {
                        let ref normal = resume.normal_dest;
                        let ref exn    = resume.exn_dest;

                        if id.is_none() {
                            // unconditional trap
                            vec![
                                BlockEdge{
                                    target: normal.target,
                                    kind: check_edge_kind(normal.target, stack),
                                    is_exception: false,
                                    probability: 1.0f32 * NORMAL_RESUME_CHANCE
                                },
                                BlockEdge{
                                    target: exn.target,
                                    kind: check_edge_kind(exn.target, stack),
                                    is_exception: true,
                                    probability: 1.0f32 * EXN_RESUME_CHANCE
                                }
                            ]
                        } else {
                            // watchpoint. jump to disable_dest when disabled. otherwise trap
                            vec![
                                BlockEdge{
                                    target: disable_dest.as_ref().unwrap().target,
                                    kind: check_edge_kind(disable_dest.as_ref().unwrap().target, stack),
                                    is_exception: false,
                                    probability: WATCHPOINT_DISABLED_CHANCE
                                },
                                BlockEdge{
                                    target: normal.target,
                                    kind: check_edge_kind(normal.target, stack),
                                    is_exception: false,
                                    probability: (1.0f32 - WATCHPOINT_DISABLED_CHANCE) * NORMAL_RESUME_CHANCE
                                },
                                BlockEdge{
                                    target: exn.target,
                                    kind: check_edge_kind(exn.target, stack),
                                    is_exception: true,
                                    probability: (1.0f32 - WATCHPOINT_DISABLED_CHANCE) * EXN_RESUME_CHANCE
                                }
                            ]
                        }
                    },

                    // wpbranch
                    WPBranch{ref disable_dest, ref enable_dest, ..} => vec![
                        BlockEdge{
                            target: disable_dest.target,
                            kind: check_edge_kind(disable_dest.target, stack),
                            is_exception: false,
                            probability: WATCHPOINT_DISABLED_CHANCE
                        },
                        BlockEdge{
                            target: enable_dest.target,
                            kind: check_edge_kind(enable_dest.target, stack),
                            is_exception: false,
                            probability: 1.0f32 - WATCHPOINT_DISABLED_CHANCE
                        }
                    ],

                    // call
                    Call{ref resume, ..}
                    | CCall{ref resume, ..}
                    | SwapStack{ref resume, ..}
                    | ExnInstruction{ref resume, ..} => {
                        let ref normal = resume.normal_dest;
                        let ref exn    = resume.exn_dest;

                        vec![
                                BlockEdge{
                                    target: normal.target,
                                    kind: check_edge_kind(normal.target, stack),
                                    is_exception: false,
                                    probability: 1.0f32 * NORMAL_RESUME_CHANCE
                                },
                                BlockEdge{
                                    target: exn.target,
                                    kind: check_edge_kind(exn.target, stack),
                                    is_exception: true,
                                    probability: 1.0f32 * EXN_RESUME_CHANCE
                                }

                        ]
                    },

                    _ => vec![]
                }
            },
            _ => panic!("expected an instruction")
        }
    };

    trace!("out edges for {}: {}", cur, vector_as_str(&out_edges));

    for edge in out_edges {
        new_edge(cur, edge, stack, visited, func);
    }

    stack.pop();
}

impl CompilerPass for ControlFlowAnalysis {
    fn name(&self) -> &'static str {
        self.name
    }

    fn as_any(&self) -> &Any {
        self
    }

    #[allow(unused_variables)]
    fn visit_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        let mut stack   : Vec<MuID> = vec![];
        let mut visited : Vec<MuID> = vec![];

        dfs(func.content.as_ref().unwrap().entry, &mut stack, &mut visited, func);
    }

    #[allow(unused_variables)]
    fn finish_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        {
            let mut exception_blocks = LinkedHashSet::new();

            for block in func.content.as_ref().unwrap().blocks.iter() {
                let ref control_flow = block.1.control_flow;

                for edge in control_flow.succs.iter() {
                    if edge.is_exception {
                        exception_blocks.insert(edge.target);
                    }
                }
            }

            func.content.as_mut().unwrap().exception_blocks.add_all(exception_blocks);
        }

        debug!("check control flow for {}", func);

        for entry in func.content.as_ref().unwrap().blocks.iter() {
            debug!("block {}", entry.0);

            debug!("{}", entry.1.control_flow);
        }
    }
}
