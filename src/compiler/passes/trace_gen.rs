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

use std::any::Any;

pub struct TraceGen {
    name: &'static str,
}

impl TraceGen {
    pub fn new() -> TraceGen {
        TraceGen {
            name: "Trace Generation",
        }
    }
}

impl CompilerPass for TraceGen {
    fn name(&self) -> &'static str {
        self.name
    }

    fn as_any(&self) -> &Any {
        self
    }

    #[allow(unused_variables)]
    fn visit_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        // we put the high probability edge into a hot trace, and others into cold paths
        // and traverse cold_path later
        let trace = {
            let mut trace: Vec<MuID> = vec![];
            let mut work_stack: Vec<MuID> = vec![];

            let entry = func.content.as_ref().unwrap().entry;
            work_stack.push(entry);

            while !work_stack.is_empty() {
                let cur = work_stack.pop().unwrap();
                let cur_block = func.content.as_ref().unwrap().get_block(cur);
                trace!("check block {}", cur);

                trace!("add {:?} to trace", cur);
                trace.push(cur);

                // get hot path
                let hot_edge = {
                    match cur_block.control_flow.get_hottest_succ() {
                        Some(tag) => tag,
                        None => continue,
                    }
                };

                // push cold paths (that are not in the trace and not in the work_stack) to work_stack
                let mut cold_edges = cur_block.control_flow.succs.clone();
                cold_edges.retain(|x| {
                    !x.target.eq(&hot_edge) && !trace.contains(&x.target) &&
                        !work_stack.contains(&x.target)
                });
                let mut cold_edge_tags = cold_edges.iter().map(|x| x.target).collect::<Vec<MuID>>();
                trace!("push cold edges {:?} to work stack", cold_edge_tags);
                work_stack.append(&mut cold_edge_tags);

                // if hot edge is not in the trace, push it to the trace
                if !trace.contains(&hot_edge) && !work_stack.contains(&hot_edge) {
                    trace!("push hot edge {:?} to work stack", hot_edge);
                    work_stack.push(hot_edge);
                } else {
                    trace!("hot edge {:?} already in trace, ignore", hot_edge);
                }

                trace!("");
            }

            trace
        };

        func.block_trace = Some(trace);
    }

    #[allow(unused_variables)]
    fn finish_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        debug!("trace for {}", func);
        debug!("{:?}", func.block_trace.as_ref().unwrap());
    }
}
