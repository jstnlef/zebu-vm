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
use ast::types::*;
use ast::inst::*;
use ast::ptr::*;
use ast::op::CmpOp;
use vm::VM;
use compiler::CompilerPass;
use utils::LinkedHashSet;
use std::any::Any;

pub struct TraceGen {
    name: &'static str
}

impl TraceGen {
    pub fn new() -> TraceGen {
        TraceGen {
            name: "Trace Generation"
        }
    }
}

const LOG_TRACE_SCHEDULE: bool = true;

impl CompilerPass for TraceGen {
    fn name(&self) -> &'static str {
        self.name
    }

    fn as_any(&self) -> &Any {
        self
    }

    #[allow(unused_variables)] // vm is not used here
    fn visit_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        // we put the high probability edge into a hot trace, and others into cold paths
        // and traverse cold_path later
        let trace = {
            let mut trace: Vec<MuID> = vec![];

            // main work stack
            let mut work_stack: LinkedHashSet<MuID> = LinkedHashSet::new();
            // slow path queue (they are scheduled after main work stack is finished)
            let mut slowpath_queue: LinkedHashSet<MuID> = LinkedHashSet::new();
            // return sink (always schedule this after all blocks)
            let mut ret_sink: Option<MuID> = None;

            let f_content = func.content.as_ref().unwrap();
            let entry = f_content.entry;
            work_stack.insert(entry);

            while !work_stack.is_empty() || !slowpath_queue.is_empty() {
                let cur_block: &Block = {
                    let ret = if let Some(b) = work_stack.pop_back() {
                        b
                    } else if let Some(b) = slowpath_queue.pop_front() {
                        b
                    } else {
                        unreachable!()
                    };
                    f_content.get_block(ret)
                };
                trace_if!(
                    LOG_TRACE_SCHEDULE,
                    "---check block {} #{}---",
                    cur_block,
                    cur_block.id()
                );

                // append current block to the trace
                if !trace.contains(&cur_block.id()) {
                    trace_if!(
                        LOG_TRACE_SCHEDULE,
                        "add {} #{} to trace",
                        cur_block,
                        cur_block.id()
                    );
                    trace.push(cur_block.id());

                    // trying to find next block
                    let next_block: Option<MuID> = match find_next_block(cur_block, func) {
                        Some(id) => Some(id),
                        None => None
                    };
                    trace_if!(
                        LOG_TRACE_SCHEDULE && next_block.is_some(),
                        "find next block as {} #{}",
                        f_content.get_block(next_block.unwrap()),
                        next_block.unwrap()
                    );

                    // put other succeeding blocks to different work stacks
                    let mut all_successors: LinkedHashSet<MuID> = LinkedHashSet::from_vec(
                        cur_block
                            .control_flow
                            .succs
                            .iter()
                            .map(|x| x.target)
                            .collect()
                    );
                    // remove next block from it
                    if next_block.is_some() {
                        all_successors.remove(&next_block.unwrap());
                    }

                    // push other successors to different work queues
                    for succ_id in all_successors.iter() {
                        let succ = f_content.get_block(*succ_id);
                        match succ.trace_hint {
                            TraceHint::None => {
                                trace_if!(
                                    LOG_TRACE_SCHEDULE,
                                    "push {} #{} to work stack",
                                    succ,
                                    succ_id
                                );
                                work_stack.insert(*succ_id);
                            }
                            TraceHint::SlowPath => {
                                trace_if!(
                                    LOG_TRACE_SCHEDULE,
                                    "push {} #{} to slow path",
                                    succ,
                                    succ_id
                                );
                                slowpath_queue.insert(*succ_id);
                            }
                            TraceHint::ReturnSink => {
                                assert!(
                                    ret_sink.is_none() ||
                                        (ret_sink.is_some() && ret_sink.unwrap() == *succ_id),
                                    "cannot have more than one return sink"
                                );
                                trace_if!(
                                    LOG_TRACE_SCHEDULE,
                                    "set {} #{} as return sink",
                                    succ,
                                    succ_id
                                );
                                ret_sink = Some(*succ_id);
                            }
                            TraceHint::FastPath => {
                                panic!(
                                    "trying to delay the insertion of a block with fastpath hint: \
                                     {} #{}. Either we missed to pick it as next block, or the \
                                     current checking block has several succeeding blocks with \
                                     fastpath hint which is \
                                     not reasonable",
                                    succ,
                                    succ_id
                                );
                            }
                        }
                    }

                    // push next block to work stack - this comes after the pushes above, so it gets
                    // popped earlier (and scheduled before those)
                    if next_block.is_some() {
                        let next_block = next_block.unwrap();
                        trace_if!(
                            LOG_TRACE_SCHEDULE,
                            "push hot edge {} #{} to work stack",
                            f_content.get_block(next_block),
                            next_block
                        );
                        work_stack.insert(next_block);
                    }

                    trace_if!(LOG_TRACE_SCHEDULE, "");
                } else {
                    trace_if!(LOG_TRACE_SCHEDULE, "block already in trace, ignore");
                    continue;
                }
            }

            // add return sink
            if let Some(ret_sink) = ret_sink {
                assert!(
                    !trace.contains(&ret_sink),
                    "return sink should not already be scheduled"
                );
                trace_if!(
                    LOG_TRACE_SCHEDULE,
                    "push return sink {} #{} to the trace",
                    f_content.get_block(ret_sink),
                    ret_sink
                );
                trace.push(ret_sink);
            }

            trace
        };

        func.block_trace = Some(trace);
    }

    #[allow(unused_variables)] // vm is not used here
    fn finish_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        debug!("trace for {}", func);
        debug!("{:?}", func.block_trace.as_ref().unwrap());

        info!("doing branch adjustment...");
        branch_adjustment(func, vm);

        debug!("trace for {}", func);
        debug!("{:?}", func.block_trace.as_ref().unwrap());
    }
}

/// returns the successor of current block.
/// We first look at trace hint, if there is no trace hint to indicate next block,
/// we layout the block with the highest probability as next block (in case of a tie,
/// returns the first met successor). If current block does not have any successor,
/// returns None.
fn find_next_block(cur_block: &Block, func: &MuFunctionVersion) -> Option<MuID> {
    let f_content = func.content.as_ref().unwrap();
    let ref succs = cur_block.control_flow.succs;
    let has_fastpath = succs.iter().find(|edge| {
        f_content.get_block(edge.target).trace_hint == TraceHint::FastPath
    });

    if has_fastpath.is_some() {
        let target = has_fastpath.unwrap().target;
        trace_if!(
            LOG_TRACE_SCHEDULE,
            "found fastpath successor {} for block {}",
            target,
            cur_block
        );
        Some(target)
    } else {
        // we need to find next path by examining probability
        if succs.len() == 0 {
            trace_if!(
                LOG_TRACE_SCHEDULE,
                "cannot find successors of block {}",
                cur_block
            );
            None
        } else {
            trace_if!(LOG_TRACE_SCHEDULE, "successors: {:?}", succs);
            let ideal_successors: Vec<&BlockEdge> = succs
                .iter()
                .filter(|b| match f_content.get_block(b.target).trace_hint {
                    TraceHint::SlowPath | TraceHint::ReturnSink => false,
                    _ => true
                })
                .collect();
            trace_if!(
                LOG_TRACE_SCHEDULE,
                "after filtering out slowpath/retsink, we have: {:?}",
                ideal_successors
            );

            if ideal_successors.len() == 0 {
                None
            } else {
                let mut hot_blk = ideal_successors[0].target;
                let mut hot_prob = ideal_successors[0].probability;

                for edge in ideal_successors.iter() {
                    trace_if!(
                        LOG_TRACE_SCHEDULE,
                        "succ: {}/{}",
                        edge.target,
                        edge.probability
                    );
                    if edge.probability >= hot_prob {
                        hot_blk = edge.target;
                        hot_prob = edge.probability;
                    }
                }

                Some(hot_blk)
            }
        }
    }
}

/// a conditional branch should always be followed by its false label.
/// The adjustment should follow the rules:
/// * any conditional branch followed by its false label stays unchanged
/// * for conditional branch followed by its true label,
///   we switch the true and false label, and negate the condition
/// * for conditional branch followed by neither label,
///   we invent a new false label, and rewrite the conditional branch so that
///   the new cond branch will be followed by the new false label.
fn branch_adjustment(func: &mut MuFunctionVersion, vm: &VM) {
    let mut trace = func.block_trace.take().unwrap();
    let mut f_content = func.content.take().unwrap();
    let mut new_blocks: Vec<Block> = vec![];

    for (blk_id, block) in f_content.blocks.iter_mut() {
        trace_if!(LOG_TRACE_SCHEDULE, "block: {} #{}", block, blk_id);

        let next_block_in_trace: Option<usize> = {
            if let Some(index) = trace.iter().position(|x| x == blk_id) {
                if index >= trace.len() - 1 {
                    // we do not have next block in the trace
                    None
                } else {
                    Some(trace[index + 1])
                }
            } else {
                warn!("find an unreachable block (a block exists in IR, but is not in trace");
                continue;
            }
        };

        // we only need to deal with blocks that ends with a branch
        if block.ends_with_cond_branch() {
            // old block content
            let block_content = block.content.as_ref().unwrap().clone();

            let mut new_body = vec![];

            for node in block_content.body.iter() {
                match node.v {
                    TreeNode_::Instruction(Instruction {
                        ref ops,
                        v: Instruction_::Branch2 {
                            cond,
                            ref true_dest,
                            ref false_dest,
                            true_prob
                        },
                        ..
                    }) => {
                        trace_if!(LOG_TRACE_SCHEDULE, "rewrite cond branch: {}", node);

                        let true_label_id = true_dest.target.id();
                        let false_label_id = false_dest.target.id();

                        trace_if!(LOG_TRACE_SCHEDULE, "true_label = {}", true_label_id);
                        trace_if!(LOG_TRACE_SCHEDULE, "false_label = {}", false_label_id);
                        trace_if!(
                            LOG_TRACE_SCHEDULE,
                            "next_block_in_trace = {:?}",
                            next_block_in_trace
                        );

                        if next_block_in_trace.is_some() &&
                            next_block_in_trace.unwrap() == false_label_id
                        {
                            // any conditional branch followed by its false label stays unchanged
                            trace_if!(LOG_TRACE_SCHEDULE, ">>stays unchanged");
                            new_body.push(node.clone());
                        } else if next_block_in_trace.is_some() &&
                                   next_block_in_trace.unwrap() == true_label_id
                        {
                            // for conditional branch followed by its true label
                            // we switch the true and false label, and negate the condition
                            let new_true_dest = false_dest.clone();
                            let new_false_dest = true_dest.clone();

                            let new_cond_node = {
                                let old_cond_node_clone = ops[cond].clone();
                                match ops[cond].v {
                                    // cond is a comparison, we recreate a comparison node
                                    // with inverted operator
                                    // orig: if a  OP b then L1 else L2
                                    // new : if a ~OP b then L2 else L1
                                    TreeNode_::Instruction(Instruction {
                                        ref value,
                                        ref ops,
                                        v: Instruction_::CmpOp(optr, op1, op2),
                                        ..
                                    }) => {
                                        TreeNode::new_inst(Instruction {
                                            hdr: MuEntityHeader::unnamed(vm.next_id()),
                                            value: value.clone(),
                                            ops: ops.clone(),
                                            v: Instruction_::CmpOp(optr.invert(), op1, op2)
                                        })
                                    }
                                    // cond is computed form other instruction or is a value
                                    // we add an instruction for cond EQ 0 (negate of cond EQ 1)
                                    // orig: if (cond)        then L1 else L2
                                    // new : if ((cond) EQ 0) then L2 else L1
                                    _ => {
                                        let temp_res: P<TreeNode> = func.new_ssa(
                                            MuEntityHeader::unnamed(vm.next_id()),
                                            UINT1_TYPE.clone()
                                        );
                                        let const_0 = func.new_constant(Value::make_int_const_ty(
                                            vm.next_id(),
                                            UINT1_TYPE.clone(),
                                            0
                                        ));
                                        TreeNode::new_inst(Instruction {
                                            hdr: MuEntityHeader::unnamed(vm.next_id()),
                                            value: Some(vec![temp_res.clone_value()]),
                                            ops: vec![old_cond_node_clone, const_0],
                                            v: Instruction_::CmpOp(CmpOp::EQ, 0, 1)
                                        })
                                    }
                                }
                            };

                            // old ops is something like: a b cond c d e
                            // we want: a b new_cond c d e
                            let new_ops = {
                                let mut ret = ops.clone();
                                ret.push(new_cond_node);
                                let last = ret.len() - 1;
                                ret.swap(cond, last);
                                ret.remove(last);
                                ret
                            };

                            let new_cond_branch = func.new_inst(Instruction {
                                hdr: MuEntityHeader::unnamed(vm.next_id()),
                                value: None,
                                ops: new_ops,
                                v: Instruction_::Branch2 {
                                    cond: cond,
                                    true_dest: new_true_dest,
                                    false_dest: new_false_dest,
                                    true_prob: 1f32 - true_prob
                                }
                            });

                            trace_if!(LOG_TRACE_SCHEDULE, ">>T/F labels switched, op negated");
                            trace_if!(LOG_TRACE_SCHEDULE, ">>{}", new_cond_branch);
                            new_body.push(new_cond_branch);
                        } else {
                            // for conditional branch followed by neither label,
                            // we invent a new false label, and rewrite the conditional branch
                            // so that the new cond branch will be followed by the new false label

                            // create a false block
                            // Lnew_false (arg list):
                            //   BRANCH Lfalse (arg list)
                            let new_false_block = {
                                let block_name =
                                    Arc::new(format!("{}:#{}:false", func.name(), node.id()));
                                let mut block =
                                    Block::new(MuEntityHeader::named(vm.next_id(), block_name));

                                let block_args: Vec<P<TreeNode>> = false_dest
                                    .args
                                    .iter()
                                    .map(|x| match x {
                                        &DestArg::Normal(i) => {
                                            func.new_ssa(
                                                MuEntityHeader::unnamed(vm.next_id()),
                                                ops[i].as_value().ty.clone()
                                            )
                                        }
                                        _ => unimplemented!()
                                    })
                                    .collect();
                                let block_args_len = block_args.len();
                                block.content = Some(BlockContent {
                                    args: block_args.iter().map(|x| x.clone_value()).collect(),
                                    exn_arg: None,
                                    body: vec![
                                        func.new_inst(Instruction {
                                            hdr: MuEntityHeader::unnamed(vm.next_id()),
                                            value: None,
                                            ops: block_args,
                                            v: Instruction_::Branch1(Destination {
                                                target: false_dest.target.clone(),
                                                args: (0..block_args_len)
                                                    .map(|x| DestArg::Normal(x))
                                                    .collect()
                                            })
                                        }),
                                    ],
                                    keepalives: None
                                });

                                block
                            };

                            // for BRANCH2 cond Ltrue Lfalse
                            // rewrite it as BRANCH2 cond Ltrue (...) Lnew_false (...)
                            let new_cond_branch = func.new_inst(Instruction {
                                hdr: MuEntityHeader::unnamed(vm.next_id()),
                                value: None,
                                ops: ops.clone(),
                                v: Instruction_::Branch2 {
                                    cond: cond,
                                    true_dest: true_dest.clone(),
                                    false_dest: Destination {
                                        target: new_false_block.hdr.clone(),
                                        args: false_dest.args.clone()
                                    },
                                    true_prob: true_prob
                                }
                            });

                            trace_if!(LOG_TRACE_SCHEDULE, ">>new F label created");
                            trace_if!(LOG_TRACE_SCHEDULE, ">>{}", new_cond_branch);
                            new_body.push(new_cond_branch);

                            // add new false block to trace (immediate after this block)
                            if let Some(next_block) = next_block_in_trace {
                                let next_block_index =
                                    trace.iter().position(|x| *x == next_block).unwrap();
                                trace.insert(next_block_index, new_false_block.id());
                            } else {
                                trace.push(new_false_block.id());
                            }
                            // add new false block to new_blocks (insert them to function later)
                            new_blocks.push(new_false_block);
                        }
                    }

                    _ => new_body.push(node.clone())
                }
            }

            block.content = Some(BlockContent {
                args: block_content.args.to_vec(),
                exn_arg: block_content.exn_arg.clone(),
                body: new_body,
                keepalives: block_content.keepalives.clone()
            });
        }
    }

    for blk in new_blocks {
        f_content.blocks.insert(blk.id(), blk);
    }

    func.content = Some(f_content);
    func.block_trace = Some(trace);
}
