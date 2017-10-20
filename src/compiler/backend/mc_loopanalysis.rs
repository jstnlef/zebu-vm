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
use utils::LinkedHashMap;
use utils::LinkedHashSet;
use utils::LinkedMultiMap;
use utils::LinkedRepeatableMultiMap;
use utils::Tree;

use std::any::Any;

const TRACE_LOOPANALYSIS: bool = true;

pub struct MCLoopAnalysis {
    name: &'static str
}

impl CompilerPass for MCLoopAnalysis {
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
        let prologue = cf.mc().get_prologue_block();

        let dominators = compute_dominators(&cf, &cfg);
        trace!("---dominators---");
        trace!("{:?}", dominators);

        let idoms = compute_immediate_dominators(&dominators);
        trace!("---immediate dominators---");
        trace!("{:?}", idoms);

        let domtree = compute_domtree(prologue.clone(), &idoms);
        trace!("---domtree---");
        trace!("{:?}", domtree);

        let loops = compute_loops(&domtree, &cfg);
        trace!("---loops---");
        trace!("{:?}", loops);

        let merged_loops = compute_merged_loop(&loops);
        trace!("---merged loops---");
        trace!("{:?}", merged_loops);

        let loop_nest_tree = compute_loop_nest_tree(prologue.clone(), &merged_loops);
        trace!("---loop-nest tree---");
        trace!("{:?}", loop_nest_tree);

        let loop_depth = compute_loop_depth(&loop_nest_tree, &merged_loops);
        trace!("---loop depth---");
        trace!("{:?}", loop_depth);

        let result = Box::new(MCLoopAnalysisResult {
            domtree,
            loops,
            loop_nest_tree,
            loop_depth
        });

        cf.loop_analysis = Some(result);
    }
}

#[allow(dead_code)]
pub struct MCLoopAnalysisResult {
    pub domtree: MCDomTree,
    pub loops: LinkedRepeatableMultiMap<MuName, MCNaturalLoop>,
    pub loop_nest_tree: MCLoopNestTree,
    pub loop_depth: LinkedHashMap<MuName, usize>
}

impl MCLoopAnalysis {
    pub fn new() -> MCLoopAnalysis {
        MCLoopAnalysis {
            name: "Machine Code Loop Analysis"
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
        trace_if!(TRACE_LOOPANALYSIS, "compute idom(n={:?})", n);
        // check which dominator idom from doms is the immediate dominator
        // 1. idom is not n
        // 2. idom dominates n (for sure if we find idom from n's dominators)
        // 3. idom does not dominate any other dominator of n
        for candidate in doms.iter() {
            trace_if!(TRACE_LOOPANALYSIS, "  check candidate {:?}", candidate);
            // idom is not n
            if candidate != n {
                let mut candidate_is_idom = true;

                // candidate does not dominate any other dominator of n
                for d in doms.iter() {
                    trace_if!(
                        TRACE_LOOPANALYSIS,
                        "    check if {:?} doms d={:?}",
                        candidate,
                        d
                    );
                    if d != candidate && d != n {
                        if is_dom(candidate, d, &dominators) {
                            trace_if!(
                                TRACE_LOOPANALYSIS,
                                "    failed, as {:?} dominates other dominator {:?}",
                                candidate,
                                d
                            );
                            candidate_is_idom = false;
                        }
                    } else {
                        trace_if!(TRACE_LOOPANALYSIS, "    skip, as d==candidate or d==n");
                    }
                }

                if candidate_is_idom {
                    assert!(!immediate_doms.contains_key(n));
                    trace_if!(
                        TRACE_LOOPANALYSIS,
                        "    add idom({:?}) = {:?}",
                        n,
                        candidate
                    );
                    immediate_doms.insert(n.clone(), candidate.clone());
                    break;
                }
            } else {
                trace_if!(TRACE_LOOPANALYSIS, "  skip, candidate is n");
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

pub type MCDomTree = Tree<MuName>;

fn compute_domtree(entry: MuName, idoms: &LinkedHashMap<MuName, MuName>) -> MCDomTree {
    let mut domtree = MCDomTree::new(entry);
    // for every other node, there is an edge from
    for (x, idom_x) in idoms.iter() {
        domtree.insert(idom_x.clone(), x.clone());
    }
    domtree
}

#[derive(Debug)]
pub struct MCNaturalLoop {
    header: MuName,
    backedge: MuName,
    blocks: LinkedHashSet<MuName>
}

/// returns a set of lists, which contains blocks in the loop
/// the first element in the list is the block header
fn compute_loops(
    domtree: &MCDomTree,
    cfg: &MachineCFG
) -> LinkedRepeatableMultiMap<MuName, MCNaturalLoop> {
    let mut ret = LinkedRepeatableMultiMap::new();
    let mut work_list = vec![domtree.root()];
    while !work_list.is_empty() {
        let cur = work_list.pop().unwrap();
        if let Some(loops) = identify_loop(cur, domtree, cfg) {
            ret.insert_vec(cur.clone(), loops);
        }
        if domtree.has_children(cur) {
            for child in domtree.get_children(cur).iter() {
                work_list.push(child);
            }
        }
    }
    ret
}

fn identify_loop(
    header: &MuName,
    domtree: &MCDomTree,
    cfg: &MachineCFG
) -> Option<Vec<MCNaturalLoop>> {
    trace_if!(TRACE_LOOPANALYSIS, "find loop with header {}", header);
    let descendants = domtree.get_all_descendants(header);
    trace_if!(TRACE_LOOPANALYSIS, "descendants: {:?}", descendants);
    let mut ret = None;
    for n in descendants.iter() {
        if cfg.has_edge(n, header) {
            // n -> header is a backedge
            let lp = identify_single_loop(header, n, &descendants, cfg);
            if ret.is_none() {
                ret = Some(vec![lp]);
            } else {
                ret.as_mut().unwrap().push(lp);
            }
        }
    }
    ret
}

fn identify_single_loop(
    header: &MuName,
    backedge: &MuName,
    nodes: &LinkedHashSet<MuName>,
    cfg: &MachineCFG
) -> MCNaturalLoop {
    trace_if!(
        TRACE_LOOPANALYSIS,
        "find loop with header {} and backedge {}",
        header,
        backedge
    );
    // we want to find all nodes x in 'nodes'
    // where there is a path from x to backedge not containing header
    let mut loop_blocks = LinkedHashSet::new();
    for x in nodes.iter() {
        if x == header || x == backedge {
            loop_blocks.insert(x.clone());
        } else if cfg.has_path_with_node_excluded(x, backedge, header) {
            loop_blocks.insert(x.clone());
        }
    }
    MCNaturalLoop {
        header: header.clone(),
        backedge: backedge.clone(),
        blocks: loop_blocks
    }
}

#[derive(Debug)]
struct MCMergedLoop {
    header: MuName,
    backedges: LinkedHashSet<MuName>,
    blocks: LinkedHashSet<MuName>
}

fn compute_merged_loop(loops: &LinkedRepeatableMultiMap<MuName, MCNaturalLoop>)
    -> LinkedHashMap<MuName, MCMergedLoop> {
    let mut merged_loops = LinkedHashMap::new();
    for (header, natural_loops) in loops.iter() {
        let mut merged_loop = MCMergedLoop {
            header: header.clone(),
            backedges: LinkedHashSet::new(),
            blocks: LinkedHashSet::new()
        };
        for l in natural_loops.iter() {
            merged_loop.backedges.insert(l.backedge.clone());
            merged_loop.blocks.add_all(l.blocks.clone());
        }
        merged_loops.insert(header.clone(), merged_loop);
    }
    merged_loops
}

type MCLoopNestTree = Tree<MuName>;

fn compute_loop_nest_tree(
    root: MuName,
    merged_loops: &LinkedHashMap<MuName, MCMergedLoop>
) -> MCLoopNestTree {
    trace_if!(TRACE_LOOPANALYSIS, "compute loop-nest tree");
    let mut loop_nest_tree = Tree::new(root.clone());

    for header in merged_loops.keys() {
        trace_if!(TRACE_LOOPANALYSIS, "check loop: {}", header);

        // if header appears in other merged loop, then it is a nested loop
        // we need to find the most recent outer loop (determined by loop size - number of blocks)
        let mut outer_loop_candidate = None;
        let mut outer_loop_size = {
            use std::usize;
            usize::MAX
        };
        for (outer_header, outer_merged_loop) in merged_loops.iter() {
            // nested loop - add an edge from outer loop header to this loop header
            if header != outer_header && outer_merged_loop.blocks.contains(header) {
                let loop_size = outer_merged_loop.blocks.len();
                if loop_size < outer_loop_size {
                    outer_loop_candidate = Some(outer_header);
                    outer_loop_size = loop_size;
                }
            }
        }
        if let Some(outer_loop) = outer_loop_candidate {
            loop_nest_tree.insert(outer_loop.clone(), header.clone());
        } else {
            // this header is not a nested loop - add an edge from root to this loop header
            loop_nest_tree.insert(root.clone(), header.clone());
        }
    }

    loop_nest_tree
}

fn compute_loop_depth(
    tree: &MCLoopNestTree,
    merged_loops: &LinkedHashMap<MuName, MCMergedLoop>
) -> LinkedHashMap<MuName, usize> {
    trace_if!(TRACE_LOOPANALYSIS, "compute loop depth");
    let mut ret = LinkedHashMap::new();
    record_depth(0, tree.root(), tree, merged_loops, &mut ret);
    ret
}

fn record_depth(
    depth: usize,
    node: &MuName,
    tree: &MCLoopNestTree,
    merged_loops: &LinkedHashMap<MuName, MCMergedLoop>,
    map: &mut LinkedHashMap<MuName, usize>
) {
    // insert the header with the deapth
    trace_if!(TRACE_LOOPANALYSIS, "Header {} = Depth {}", node, depth);
    map.insert(node.clone(), depth);
    // also find all the blocks that belong to the header and are not inner loop header
    // and insert them with the same depth
    if let Some(merged_loop) = merged_loops.get(node) {
        for b in merged_loop.blocks.iter() {
            if !merged_loops.contains_key(b) {
                map.insert(b.clone(), depth);
                trace_if!(TRACE_LOOPANALYSIS, "{} = Depth {}", b, depth);
            }
        }
    }
    if tree.has_children(node) {
        for c in tree.get_children(node).iter() {
            record_depth(depth + 1, c, tree, merged_loops, map);
        }
    }
}
