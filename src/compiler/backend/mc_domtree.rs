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

use compiler::CompilerPass;
use ast::ir::*;
use vm::VM;
use compiler::machine_code::*;
use compiler::backend;
use std::any::Any;
use utils::LinkedHashMap;
use utils::LinkedHashSet;
use utils::LinkedMultiMap;

const TRACE_DOMTREE: bool = true;

pub struct MCDomTree {
    name: &'static str
}

impl CompilerPass for MCDomTree {
    fn name(&self) -> &'static str {
        self.name
    }

    fn as_any(&self) -> &Any {
        self
    }

    fn visit_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        let compiled_funcs = vm.compiled_funcs().read().unwrap();
        let mut cf = compiled_funcs.get(&func.id()).unwrap().write().unwrap();
        let cfg = cf.mc().build_cfg();

        let dominators = compute_dominators(&cf, &cfg);
        trace!("dominators:");
        trace!("{:?}", dominators);

        let idoms = compute_immediate_dominators(&dominators);
        trace!("immediate dominators:");
        trace!("{:?}", idoms);
    }
}

impl MCDomTree {
    pub fn new() -> MCDomTree {
        MCDomTree {
            name: "Compute Machine Code Dom Tree"
        }
    }
}

fn compute_dominators(func: &CompiledFunction, cfg: &MachineCFG) -> LinkedMultiMap<MuName, MuName> {
    let mc = func.mc();
    // D[s0] = s0
    // D[n] = {n} \/ ( /\ D[p] for all p in pred[n])

    // use iterative algorithm to compute dominators

    // init dominators:
    let mut dominators = LinkedMultiMap::new();
    // D[s0] = s0;
    let entry = mc.get_prologue_block();
    dominators.insert(entry.clone(), entry.clone());
    // D[n] (n!=s0) = all the blocks
    let all_blocks = LinkedHashSet::from_vec(mc.get_all_blocks());
    for block in mc.get_all_blocks() {
        if block != entry {
            dominators.insert_set(block.clone(), all_blocks.clone());
        }
    }

    // iteration - start with a work queue of all successors of entry block
    let mut work_queue: LinkedHashSet<MuName> =
        LinkedHashSet::from_vec(cfg.get_succs(&entry).clone());
    while let Some(cur) = work_queue.pop_front() {
        let preds = cfg.get_preds(&cur);
        let new_set = {
            // ( /\ D[p] for all p in pred[n])

            let mut intersect: LinkedHashSet<MuName> = LinkedHashSet::new();
            // add the first predecessor's dominators
            for dp in dominators.get(&preds[0]).unwrap().iter() {
                intersect.insert(dp.clone());
            }
            // retain the first predecessors's dominators with the rest
            for p in preds.iter() {
                let dp_set = dominators.get(p).unwrap();
                intersect.retain(|x| dp_set.contains(x));
            }

            // union {n} with intersect

            let mut union: LinkedHashSet<MuName> = LinkedHashSet::new();
            union.insert(cur.clone());
            union.add_all(intersect);

            union
        };

        if new_set.equals(dominators.get(&cur).unwrap()) {
            // no change, nothing
        } else {
            // otherwise set dominator as current set
            dominators.replace_set(cur.clone(), new_set);
            // add successors to work queue
            work_queue.add_from_vec(cfg.get_succs(&cur).clone());
        }
    }

    dominators
}

fn compute_immediate_dominators(dominators: &LinkedMultiMap<MuName, MuName>)
    -> LinkedHashMap<MuName, MuName> {
    let mut immediate_doms: LinkedHashMap<MuName, MuName> = LinkedHashMap::new();

    for (n, doms) in dominators.iter() {
        trace_if!(TRACE_DOMTREE, "compute idom(n={:?})", n);
        // check which dominator idom from doms is the immediate dominator
        // 1. idom is not n
        // 2. idom dominates n (for sure if we find idom from n's dominators)
        // 3. idom does not dominate any other dominator of n
        for candidate in doms.iter() {
            trace_if!(TRACE_DOMTREE, "  check candidate {:?}", candidate);
            // idom is not n
            if candidate != n {
                let mut candidate_is_idom = true;

                // candidate does not dominate any other dominator of n
                for d in doms.iter() {
                    trace_if!(TRACE_DOMTREE, "    check if {:?} doms d={:?}", candidate, d);
                    if d != candidate && d != n {
                        if is_dom(candidate, d, &dominators) {
                            trace_if!(
                                TRACE_DOMTREE,
                                "    failed, as {:?} dominates other dominator {:?}",
                                candidate,
                                d
                            );
                            candidate_is_idom = false;
                        }
                    } else {
                        trace_if!(TRACE_DOMTREE, "    skip, as d==candidate or d==n");
                    }
                }

                if candidate_is_idom {
                    assert!(!immediate_doms.contains_key(n));
                    trace_if!(TRACE_DOMTREE, "    add idom({:?}) = {:?}", n, candidate);
                    immediate_doms.insert(n.clone(), candidate.clone());
                    break;
                }
            } else {
                trace_if!(TRACE_DOMTREE, "  skip, candidate is n");
            }
        }
    }

    assert_eq!(immediate_doms.len(), dominators.len() - 1); // entry block does not have idom
    immediate_doms
}

/// whether a dominates b (i.e. b is one of the dominators of a
fn is_dom(a: &MuName, b: &MuName, dominators: &LinkedMultiMap<MuName, MuName>) -> bool {
    dominators.contains_key_val(b, a)
}
