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
use vm::VM;
use compiler::CompilerPass;
use utils::LinkedHashSet;
use std::any::Any;

pub struct TraceGen {
    name: &'static str
}

impl TraceGen {
    pub fn new() -> TraceGen {
        TraceGen{name: "Trace Generation"}
    }
}

const LOG_TRACE_SCHEDULE : bool = true;

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
            let mut trace : Vec<MuID> = vec![];

            // main work stack
            let mut work_stack : LinkedHashSet<MuID> = LinkedHashSet::new();
            // slow path queue (they are scheduled after main work stack is finished)
            let mut slowpath_queue : LinkedHashSet<MuID> = LinkedHashSet::new();
            // return sink (always schedule this after all blocks)
            let mut ret_sink : Option<MuID> = None;

            let f_content = func.content.as_ref().unwrap();
            let entry = f_content.entry;
            work_stack.insert(entry);
            
            while !work_stack.is_empty() || !slowpath_queue.is_empty() {
                let cur_block : &Block = {
                    let ret = if let Some(b) = work_stack.pop_back() {
                        b
                    } else if let Some(b) = slowpath_queue.pop_front() {
                        b
                    } else {
                        unreachable!()
                    };
                    f_content.get_block(ret)
                };
                trace_if!(LOG_TRACE_SCHEDULE, "---check block {}---", cur_block);

                // append current block to the trace
                trace_if!(LOG_TRACE_SCHEDULE, "add {} to trace", cur_block);
                trace.push(cur_block.id());

                // trying to find next block
                let next_block : MuID = match find_next_block(cur_block, func) {
                    Some(id) => id,
                    None => continue
                };
                trace_if!(LOG_TRACE_SCHEDULE, "find next block as {}", f_content.get_block(next_block));

                // put other succeeding blocks to different work stacks
                let mut all_successors : LinkedHashSet<MuID> =
                    LinkedHashSet::from_vec(cur_block.control_flow.succs.iter().map(|x| x.target).collect());
                // remove next block from it
                all_successors.remove(&next_block);

                // push other successors to different work queues
                for succ_id in all_successors.iter() {
                    let succ = f_content.get_block(*succ_id);
                    match succ.trace_hint {
                        TraceHint::None => {
                            trace_if!(LOG_TRACE_SCHEDULE, "push {} to work stack", succ);
                            work_stack.insert(*succ_id);
                        }
                        TraceHint::SlowPath => {
                            trace_if!(LOG_TRACE_SCHEDULE, "push {} to slow path", succ);
                            slowpath_queue.insert(*succ_id);
                        }
                        TraceHint::ReturnSink => {
                            assert!(ret_sink.is_none(), "cannot have more than one return sink");
                            trace_if!(LOG_TRACE_SCHEDULE, "set {} as return sink", succ);
                            ret_sink = Some(*succ_id);
                        }
                        TraceHint::FastPath => {
                            panic!("trying to delay the insertion of a block with fastpath hint: {}. \
                                Either we missed to pick it as next block, or the current checking \
                                block has several succeeding blocks with fastpath hint which is \
                                not reasonable", succ);
                        }
                    }
                }

                // if hot edge is not in the trace, push it to the trace
                if !trace.contains(&next_block) && !work_stack.contains(&next_block) {
                    trace_if!(LOG_TRACE_SCHEDULE, "push hot edge {:?} to work stack", next_block);
                    work_stack.insert(next_block);
                } else {
                    trace_if!(LOG_TRACE_SCHEDULE, "hot edge {:?} already in trace, ignore", next_block);
                }

                trace_if!(LOG_TRACE_SCHEDULE, "");
            }

            // add return sink
            if let Some(ret_sink) = ret_sink {
                assert!(!trace.contains(&ret_sink), "return sink should not already be scheduled");
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
    let has_fastpath = succs.iter()
        .find(|edge| f_content.get_block(edge.target).trace_hint == TraceHint::FastPath);

    if has_fastpath.is_some() {
        Some(has_fastpath.unwrap().target)
    } else {
        // we need to find next path by examining probability
        if succs.len() == 0 {
            None
        } else {
            let mut hot_blk = succs[0].target;
            let mut hot_prob = succs[0].probability;

            for edge in succs.iter() {
                if edge.probability > hot_prob {
                    hot_blk = edge.target;
                    hot_prob = edge.probability;
                }
            }

            Some(hot_blk)
        }
    }
}