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

extern crate hprof;

use compiler::machine_code::CompiledFunction;
use ast::ir::*;
use compiler::backend;
use utils::LinkedHashSet;
use utils::LinkedHashMap;

use compiler::backend::reg_alloc::graph_coloring::petgraph;
use compiler::backend::reg_alloc::graph_coloring::petgraph::Graph;
use compiler::backend::reg_alloc::graph_coloring::petgraph::graph::NodeIndex;

/// GraphNode represents a node in the interference graph.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GraphNode {
    /// temp ID (could be register)
    temp: MuID,
    /// assigned color
    color: Option<MuID>,
    /// temp register group (which machine register class we should assign)
    group: backend::RegGroup,
    /// cost to spill this temp
    spill_cost: f32
}

/// Move represents a move between two nodes (referred by index)
/// We need to know the moves so that we can coalesce.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Move{pub from: NodeIndex, pub to: NodeIndex}

/// InterferenceGraph represents the interference graph, including
/// * the graph
/// * all the nodes and its NodeIndex (a node is referred to by NodeIndex)
/// * all the moves
pub struct InterferenceGraph {
    /// the internal graph
    graph: Graph<GraphNode, (), petgraph::Undirected>,
    /// a map of all nodes (from temp ID to node index)
    /// node index is how nodes are referred to with pet_graph
    nodes: LinkedHashMap<MuID, NodeIndex>,
    /// a set of all moves
    moves: LinkedHashSet<Move>,
}

impl InterferenceGraph {
    /// creates a new graph
    fn new() -> InterferenceGraph {
        InterferenceGraph {
            graph: Graph::new_undirected(),
            nodes: LinkedHashMap::new(),
            moves: LinkedHashSet::new()
        }
    }

    /// creates a new node for a temp (if we already created a temp for the temp, returns the node)
    /// This function will increase spill cost for the node by 1 each tiem it is called for the temp
    fn new_node(&mut self, reg_id: MuID, context: &FunctionContext) -> NodeIndex {
        let entry = context.get_value(reg_id).unwrap();

        // if it is the first time, create the node
        if !self.nodes.contains_key(&reg_id) {
            let node = GraphNode {
                temp: reg_id,
                color: None,
                group: backend::RegGroup::get_from_ty(entry.ty()),
                spill_cost: 0.0f32
            };

            // add to the graph
            let index = self.graph.add_node(node);
            // save index
            self.nodes.insert(reg_id, index);
        }

        // get the node index
        let node_index = *self.nodes.get(&reg_id).unwrap();
        // get node
        let node_mut = self.graph.node_weight_mut(node_index).unwrap();
        // increase node spill cost
        node_mut.spill_cost += 1.0f32;
        
        node_index
    }

    /// returns the node index for a temp
    pub fn get_node(&self, reg: MuID) -> NodeIndex {
        match self.nodes.get(&reg) {
            Some(index) => *index,
            None => panic!("do not have a node for {}", reg)
        }
    }

    /// returns all the nodes in the graph
    pub fn nodes(&self) -> Vec<NodeIndex> {
        let mut ret = vec![];
        for index in self.nodes.values() {
            ret.push(*index);
        }
        ret
    }

    /// returns all the moves in the graph
    pub fn moves(&self) -> &LinkedHashSet<Move> {
        &self.moves
    }

    /// adds a move edge between two nodes
    fn add_move(&mut self, src: NodeIndex, dst: NodeIndex) {
        let src = {
            let temp_src = self.get_temp_of(src);
            if temp_src < MACHINE_ID_END {
                // get the color for the machine register, e.g. rax for eax/ax/al/ah
                let alias = backend::get_color_for_precolored(temp_src);
                self.get_node(alias)
            } else {
                src
            }
        };

        let dst = {
            let temp_dst = self.get_temp_of(dst);
            if temp_dst < MACHINE_ID_END {
                let alias = backend::get_color_for_precolored(temp_dst);
                self.get_node(alias)
            } else {
                dst
            }
        };

        self.moves.insert(Move{from: src, to: dst});
    }

    /// adds an interference edge between two nodes
    pub fn add_interference_edge(&mut self, from: NodeIndex, to: NodeIndex) {
        // adds edge to the internal graph
        self.graph.update_edge(from, to, ());

        // if one of the node is machine register, we also add
        // interference edge to its alias
        // e.g. if we have %a - %edi interfered,
        // we also add %a - %rdi interference

        let from_tmp = self.graph.node_weight(from).unwrap().temp;
        let to_tmp   = self.graph.node_weight(to).unwrap().temp;

        if from_tmp < MACHINE_ID_END || to_tmp < MACHINE_ID_END {
            let from_tmp = if from_tmp < MACHINE_ID_END {
                backend::get_color_for_precolored(from_tmp)
            } else {
                from_tmp
            };

            let to_tmp = if to_tmp < MACHINE_ID_END {
                backend::get_color_for_precolored(to_tmp)
            } else {
                to_tmp
            };

            let from_tmp_node = self.get_node(from_tmp);
            let to_tmp_node   = self.get_node(to_tmp);
            self.graph.update_edge(from_tmp_node, to_tmp_node, ());
        }
    }

    /// is two nodes interfered?
    pub fn is_interfered_with(&self, node1: NodeIndex, node2: NodeIndex) -> bool {
        let edge = self.graph.find_edge(node1, node2);
        edge.is_some()
    }

    /// set color for a node
    pub fn color_node(&mut self, node: NodeIndex, color: MuID) {
        self.graph.node_weight_mut(node).unwrap().color = Some(color);
    }

    /// is a node colored yet?
    pub fn is_colored(&self, node: NodeIndex) -> bool {
        self.graph.node_weight(node).unwrap().color.is_some()
    }

    /// gets the color of a node
    pub fn get_color_of(&self, node: NodeIndex) -> Option<MuID> {
        self.graph.node_weight(node).unwrap().color
    }

    /// gets the reg group of a node
    pub fn get_group_of(&self, node: NodeIndex) -> backend::RegGroup {
        self.graph.node_weight(node).unwrap().group
    }

    /// gets the temporary of a node
    pub fn get_temp_of(&self, node: NodeIndex) -> MuID {
        self.graph.node_weight(node).unwrap().temp
    }

    /// gets the spill cost of a node
    pub fn get_spill_cost(&self, node: NodeIndex) -> f32 {
        self.graph.node_weight(node).unwrap().spill_cost
    }

    /// are two nodes the same node?
    fn is_same_node(&self, node1: NodeIndex, node2: NodeIndex) -> bool {
        node1 == node2
    }

    /// are two nodes from the same reg group?
    fn is_same_group(&self, node1: NodeIndex, node2: NodeIndex) -> bool {
        let node1 = self.graph.node_weight(node1).unwrap();
        let node2 = self.graph.node_weight(node2).unwrap();

        node1.group == node2.group
    }

    /// are two nodes adjacent?
    pub fn is_adj(&self, from: NodeIndex, to: NodeIndex) -> bool {
        self.is_interfered_with(from, to)
    }

    /// gets edges from a node
    pub fn get_edges_of(&self, node: NodeIndex) -> Vec<NodeIndex> {
        self.graph.neighbors(node).collect()
    }

    /// gets degree of a node (number of edges from the node)
    pub fn get_degree_of(&self, node: NodeIndex) -> usize {
        self.get_edges_of(node).len()
    }

    /// prints current graph for debugging (via trace log)
    pub fn print(&self, context: &FunctionContext) {
        use compiler::backend::reg_alloc::graph_coloring::petgraph::dot::Dot;
        use compiler::backend::reg_alloc::graph_coloring::petgraph::dot::Config;

        trace!("");
        trace!("Interference Graph");

        trace!("nodes:");
        for id in self.nodes.keys() {
            let val = context.get_value(*id).unwrap().value();
            trace!("Reg {} -> {:?}", val, self.nodes.get(&id).unwrap());
        }

        trace!("moves:");
        for mov in self.moves.iter() {
            trace!("Move {:?} -> {:?}", mov.from, mov.to);
        }

        trace!("graph:");
        trace!("\n\n{:?}\n", Dot::with_config(&self.graph, &[Config::EdgeNoLabel]));
        trace!("");
    }
}

/// prints trace during building liveness for debugging?
const TRACE_LIVENESS: bool = false;

/// builds interference graph based on chaitin briggs algorithms
/// (reference: Tailoring Graph-coloring Register Allocation For Runtime Compilation - CGO'06, Figure 4)
pub fn build_interference_graph_chaitin_briggs(cf: &mut CompiledFunction, func: &MuFunctionVersion) -> InterferenceGraph {
    let _p = hprof::enter("regalloc: build global liveness");
    build_global_liveness(cf, func);
    drop(_p);

    let _p = hprof::enter("regalloc: build interference graph");

    info!("---start building interference graph---");
    let mut ig = InterferenceGraph::new();

    // precolor machine register nodes
    for reg in backend::all_regs().values() {
        let reg_id = reg.extract_ssa_id().unwrap();
        let node = ig.new_node(reg_id, &func.context);
        let precolor = backend::get_color_for_precolored(reg_id);

        ig.color_node(node, precolor);
    }

    // initialize and creates nodes for all the involved temps/regs
    for i in 0..cf.mc().number_of_insts() {
        for reg_id in cf.mc().get_inst_reg_defines(i) {
            ig.new_node(reg_id, &func.context);
        }

        for reg_id in cf.mc().get_inst_reg_uses(i) {
            ig.new_node(reg_id, &func.context);
        }
    }

    // for each basic block, insert interference edge while reversely traversing instructions
    for block in cf.mc().get_all_blocks() {
        // Current_Live(B) = LiveOut(B)
        let mut current_live = LinkedHashSet::from_vec(match cf.mc().get_ir_block_liveout(&block) {
            Some(liveout) => liveout.to_vec(),
            None => panic!("cannot find liveout for block {}", block)
        });
        if TRACE_LIVENESS {
            trace!("Block{}: live out", block);
            for ele in current_live.iter() {
                trace!("{}", func.context.get_temp_display(*ele));
            }
        }

        let range = cf.mc().get_block_range(&block);
        if range.is_none() {
            warn!("Block{}: has no range (no instructions?)", block);
            continue;
        }
        trace_if!(TRACE_LIVENESS, "Block{}: range = {:?}", block, range.as_ref().unwrap());

        // for every inst I in reverse order
        for i in range.unwrap().rev() {
            if TRACE_LIVENESS {
                trace!("Block{}: Inst{}", block, i);
                cf.mc().trace_inst(i);
                trace!("current live: ");
                for ele in current_live.iter() {
                    trace!("{}", func.context.get_temp_display(*ele));
                }
            }

            let src : Option<MuID> = {
                if cf.mc().is_move(i) {
                    let src = cf.mc().get_inst_reg_uses(i);
                    let dst = cf.mc().get_inst_reg_defines(i);

                    // src:  reg/imm/mem
                    // dest: reg/mem
                    // we dont care if src/dest is mem
                    if cf.mc().is_using_mem_op(i) {
                        None
                    } else {
                        if src.len() == 1 {
                            let node1 = ig.get_node(src[0]);
                            let node2 = ig.get_node(dst[0]);
                            trace_if!(TRACE_LIVENESS, "add move between {} and {}",
                            func.context.get_temp_display(src[0]),
                            func.context.get_temp_display(dst[0]));
                            ig.add_move(node1, node2);

                            Some(src[0])
                        } else {
                            None
                        }
                    }
                } else {
                    None
                }
            };
            trace_if!(TRACE_LIVENESS, "Block{}: Inst{}: src={:?}", block, i, src);

            let defines = cf.mc().get_inst_reg_defines(i);
            for d in defines.iter() {
                current_live.insert(*d);
            }

            // for every definition D in I
            for d in defines {
                trace_if!(TRACE_LIVENESS, "Block{}: Inst{}: for definition {}",
                    block, i, func.context.get_temp_display(d));
                // add an interference from D to every element E in Current_Live - {D}
                // creating nodes if necessary
                for e in current_live.iter() {
                    trace_if!(TRACE_LIVENESS, "Block{}: Inst{}: for each live {}",
                        block, i, func.context.get_temp_display(*e));
                    if src.is_none() || (src.is_some() && *e != src.unwrap()) {
                        let from = ig.get_node(d);
                        let to = ig.get_node(*e);

                        if !ig.is_same_node(from, to) &&ig.is_same_group(from, to) && !ig.is_adj(from, to) {
                            if !ig.is_colored(from) {
                                trace_if!(TRACE_LIVENESS, "Block{}: Inst{}: add interference between {} and {}",
                                    block, i,
                                    func.context.get_temp_display(d),
                                    func.context.get_temp_display(*e));
                                ig.add_interference_edge(from, to);
                            }
                            if !ig.is_colored(to) {
                                trace_if!(TRACE_LIVENESS, "Block{}: Inst{}: add interference between {} and {}",
                                    block, i,
                                    func.context.get_temp_display(*e),
                                    func.context.get_temp_display(d));
                                ig.add_interference_edge(to, from);
                            }
                        }
                    }
                }
            }

            // for every definition D in I
            for d in cf.mc().get_inst_reg_defines(i) {
                trace_if!(TRACE_LIVENESS, "Block{}: Inst{}: remove define {} from current_live",
                    block, i,
                    func.context.get_temp_display(d));
                // remove D from Current_Live
                current_live.remove(&d);
            }

            // for every use U in I
            for u in cf.mc().get_inst_reg_uses(i) {
                trace_if!(TRACE_LIVENESS, "Block{}: Inst{}: add use {} to current_live",
                    block, i,
                    func.context.get_temp_display(u));
                // add U to Current_live
                current_live.insert(u);
            }

            if TRACE_LIVENESS {
                trace!("Block{}: Inst{}: done. current_live:", block, i);
                for ele in current_live.iter() {
                    trace!("{}", func.context.get_temp_display(*ele));
                }
            }
        }
    }

    drop(_p);
    info!("---finish building interference graph---");
    ig
}

/// builds global liveness for a compiled function
fn build_global_liveness(cf: &mut CompiledFunction, func: &MuFunctionVersion) {
    info!("---start building live set---");

    // build control flow graphs, treat a whole block as one node in the graph
    let cfg = build_cfg_nodes(cf);
    // do liveness analysis
    global_liveness_analysis(cfg, cf, func);

    info!("---finish building live set---");
}

/// CFGBlockNode represents a basic block as a whole for global liveness analysis
#[derive(Clone, Debug)]
struct CFGBlockNode {
    block: String,
    pred: Vec<String>,
    succ: Vec<String>,
    uses:  Vec<MuID>,
    defs: Vec<MuID>
}

/// builds a LinkedHashMap from basic block names to CFGBlockNode
/// We need to collect for each basic block:
/// * predecessors
/// * successors
/// * uses
/// * defs
fn build_cfg_nodes(cf: &mut CompiledFunction) -> LinkedHashMap<String, CFGBlockNode> {
    info!("---local liveness analysis---");
    let mc = cf.mc();
    let mut ret = LinkedHashMap::new();
    let all_blocks = mc.get_all_blocks();

    // create maps (start_inst -> name) and (end_inst -> name)
    // we will use it to find basic blocks when given a inst index
    let (start_inst_map, end_inst_map) = {
        let mut start_inst_map : LinkedHashMap<usize, &str> = LinkedHashMap::new();
        let mut end_inst_map   : LinkedHashMap<usize, &str> = LinkedHashMap::new();
        for block in all_blocks.iter() {
            let range = match mc.get_block_range(block) {
                Some(range) => range,
                None => panic!("cannot find range for block {}", block)
            };

            // start inst
            let first_inst = range.start;
            // last inst (we need to skip symbols)
            let last_inst = match mc.get_last_inst(range.end) {
                Some(last) => last,
                None => panic!("cannot find last instruction in block {}, this block contains no instruction?", block)
            };
            trace_if!(TRACE_LIVENESS, "Block {}: start_inst={}, end_inst(inclusive)={}", block, first_inst, last_inst);

            start_inst_map.insert(first_inst, block);
            end_inst_map.insert(last_inst, block);
        }

        (start_inst_map, end_inst_map)
    };

    // collect info for each basic block
    for block in mc.get_all_blocks().iter() {
        trace_if!(TRACE_LIVENESS, "---block {}---", block);
        let range = mc.get_block_range(block).unwrap();
        let start_inst = range.start;
        let end        = range.end;

        // livein set of this block is what temps this block uses from other blocks
        // defs is what temps this block defines in the block
        let (livein, defs) = {
            // we gradually build livein
            let mut livein = vec![];
            // we need to know all temporaries defined in the block
            // if a temporary is not defined in this block, it is a livein for this block
            let mut all_defined : LinkedHashSet<MuID> = LinkedHashSet::new();

            for i in start_inst..end {
                let reg_uses = mc.get_inst_reg_uses(i);

                // if a reg is used but not defined before, it is a live-in
                for reg in reg_uses {
                    if !all_defined.contains(&reg) {
                        livein.push(reg);
                    }
                }

                let reg_defs = mc.get_inst_reg_defines(i);
                for reg in reg_defs {
                    all_defined.insert(reg);
                }
            }

            let defs : Vec<MuID> = all_defined.iter().map(|x| *x).collect();

            (livein, defs)
        };

        let preds : Vec<String> = {
            let mut ret = vec![];

            // predecessors of the first instruction is the predecessors of this block
            for pred in mc.get_preds(start_inst).into_iter() {
                match end_inst_map.get(pred) {
                    Some(str) => ret.push(String::from(*str)),
                    None => {}
                }
            }

            ret
        };

        let succs : Vec<String> = {
            let mut ret = vec![];

            // successors of the last instruction is the successors of this block
            for succ in mc.get_succs(mc.get_last_inst(end).unwrap()).into_iter() {
                match start_inst_map.get(succ) {
                    Some(str) => ret.push(String::from(*str)),
                    None => {}
                }
            }

            ret
        };

        let node = CFGBlockNode {
            block: block.clone(),
            pred: preds,
            succ: succs,
            uses: livein,
            defs: defs
        };

        trace_if!(TRACE_LIVENESS, "as CFGNode {:?}", node);
        ret.insert(block.clone(), node);
    }

    ret
}

/// global analysis, the iterative algorithm to compute livenss until livein/out reaches a fix point
fn global_liveness_analysis(blocks: LinkedHashMap<String, CFGBlockNode>, cf: &mut CompiledFunction, func: &MuFunctionVersion) {
    info!("---global liveness analysis---");
    info!("{} blocks", blocks.len());

    // init live in and live out
    let mut livein  : LinkedHashMap<String, LinkedHashSet<MuID>> = {
        let mut ret = LinkedHashMap::new();
        for name in blocks.keys() {
            ret.insert(name.clone(), LinkedHashSet::new());
        }
        ret
    };
    let mut liveout  : LinkedHashMap<String, LinkedHashSet<MuID>> = {
        let mut ret = LinkedHashMap::new();
        for name in blocks.keys() {
            ret.insert(name.clone(), LinkedHashSet::new());
        }
        ret
    };

    // is the result changed in this iteration?
    let mut is_changed = true;
    // record iteration count
    let mut i = 0;

    while is_changed {
        trace_if!(TRACE_LIVENESS, "---iteration {}---", i);
        i += 1;

        // reset
        is_changed = false;

        for node in blocks.keys() {
            let cfg_node = blocks.get(node).unwrap();

            // old livein/out
            let in_set_old  = livein.get(node).unwrap().clone();
            let out_set_old = liveout.get(node).unwrap().clone();

            // in <- use + (out - def)
            {
                let mut inset = livein.get_mut(node).unwrap();

                inset.clear();

                // (1) out - def
                inset.add_all(liveout.get(node).unwrap().clone());
                for def in cfg_node.defs.iter() {
                    inset.remove(def);
                }

                // (2) in + (out - def)
                for in_reg in cfg_node.uses.iter() {
                    inset.insert(*in_reg);
                }
            }

            // out[n] <- union(in[s] for every successor s of n)
            {
                let mut outset = liveout.get_mut(node).unwrap();
                outset.clear();

                for s in cfg_node.succ.iter() {
                    outset.add_all(livein.get(s).unwrap().clone());
                }
            }

            // is in/out changed in this iteration?
            let n_changed = !in_set_old.equals(livein.get(node).unwrap()) || !out_set_old.equals(liveout.get(node).unwrap());

            if TRACE_LIVENESS {
                trace!("block {}", node);
                trace!("in(old)  = {:?}", in_set_old);
                trace!("in(new)  = {:?}", livein.get(node).unwrap());
                trace!("out(old) = {:?}", out_set_old);
                trace!("out(new) = {:?}", liveout.get(node).unwrap());
            }

            is_changed = is_changed || n_changed;
        }
    }

    info!("finished in {} iterations", i);

    // set live in and live out
    for block in blocks.keys() {
        let livein : Vec<MuID> = livein.get(block).unwrap().clone().iter().map(|x| *x).collect();
        if TRACE_LIVENESS {
            let display_array : Vec<String> =  livein.iter().map(|x| func.context.get_temp_display(*x)).collect();
            trace!("livein  for block {}: {:?}", block, display_array);
        }
        cf.mc_mut().set_ir_block_livein(block, livein);

        let liveout : Vec<MuID> = liveout.get(block).unwrap().clone().iter().map(|x| *x).collect();
        if TRACE_LIVENESS {
            let display_array : Vec<String> = liveout.iter().map(|x| func.context.get_temp_display(*x)).collect();
            trace!("liveout for block {}: {:?}", block, display_array);
        }
        cf.mc_mut().set_ir_block_liveout(block, liveout);
    }
}