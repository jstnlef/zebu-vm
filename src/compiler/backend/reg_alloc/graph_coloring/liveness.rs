use compiler::machine_code::CompiledFunction;
use ast::ir::*;
use compiler::backend;
use utils::LinkedHashSet;

use std::collections::{HashMap, HashSet};

use compiler::backend::reg_alloc::graph_coloring::petgraph;
use compiler::backend::reg_alloc::graph_coloring::petgraph::Graph;
use compiler::backend::reg_alloc::graph_coloring::petgraph::graph::NodeIndex;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GraphNode {
    temp: MuID,
    color: Option<MuID>,
    group: backend::RegGroup,
    spill_cost: f32
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Move{pub from: NodeIndex, pub to: NodeIndex}

pub struct InterferenceGraph {
    graph: Graph<GraphNode, (), petgraph::Undirected>,
    nodes: HashMap<MuID, NodeIndex>,
    moves: HashSet<Move>,
}

impl InterferenceGraph {
    fn new() -> InterferenceGraph {
        InterferenceGraph {
            graph: Graph::new_undirected(),
            nodes: HashMap::new(),
            moves: HashSet::new()
        }
    }
    
    fn new_node(&mut self, reg_id: MuID, context: &FunctionContext) -> NodeIndex {
        let entry = context.get_value(reg_id).unwrap();
        
        if !self.nodes.contains_key(&reg_id) {
            let node = GraphNode {
                temp: reg_id,
                color: None,
                group: backend::RegGroup::get(entry.ty()),
                spill_cost: 0.0f32
            };

            let index = self.graph.add_node(node);

            self.nodes.insert(reg_id, index);
        }
        
        let node_index = *self.nodes.get(&reg_id).unwrap();
        let node_mut = self.graph.node_weight_mut(node_index).unwrap();
        
        // increase node spill cost
        node_mut.spill_cost += 1.0f32;
        
        node_index
    }
    
    pub fn get_node(&self, reg: MuID) -> NodeIndex {
        match self.nodes.get(&reg) {
            Some(index) => *index,
            None => panic!("do not have a node for {}", reg)
        }
    }
    
    pub fn temps(&self) -> Vec<MuID>{
        let mut ret = vec![];
        for reg in self.nodes.keys() {
            ret.push(*reg);
        }
        ret
    }
    
    pub fn nodes(&self) -> Vec<NodeIndex> {
        let mut ret = vec![];
        for index in self.nodes.values() {
            ret.push(*index);
        }
        ret
    }
    
    pub fn moves(&self) -> &HashSet<Move> {
        &self.moves
    }
    
    pub fn n_nodes(&self) -> usize {
        self.nodes.len()
    }
    
    fn add_move(&mut self, src: NodeIndex, dst: NodeIndex) {
        self.moves.insert(Move{from: src, to: dst});
    }
    
    pub fn add_interference_edge(&mut self, from: NodeIndex, to: NodeIndex) {
        self.graph.update_edge(from, to, ());
    }

    pub fn is_interferenced_with(&self, node1: NodeIndex, node2: NodeIndex) -> bool {
        trace!("trying to find edge between {:?} and {:?}", node1, node2);
        let edge = self.graph.find_edge(node1, node2);

        trace!("edge: {:?}", edge);

        edge.is_some()
    }
    
    pub fn color_node(&mut self, node: NodeIndex, color: MuID) {
        self.graph.node_weight_mut(node).unwrap().color = Some(color);
    }
    
    pub fn is_colored(&self, node: NodeIndex) -> bool {
        self.graph.node_weight(node).unwrap().color.is_some()
    }
    
    pub fn get_color_of(&self, node: NodeIndex) -> Option<MuID> {
        self.graph.node_weight(node).unwrap().color
    }
    
    pub fn get_group_of(&self, node: NodeIndex) -> backend::RegGroup {
        self.graph.node_weight(node).unwrap().group
    }
    
    pub fn get_temp_of(&self, node: NodeIndex) -> MuID {
        self.graph.node_weight(node).unwrap().temp
    }
    
    pub fn get_spill_cost(&self, node: NodeIndex) -> f32 {
        self.graph.node_weight(node).unwrap().spill_cost
    }
    
    fn is_same_node(&self, node1: NodeIndex, node2: NodeIndex) -> bool {
        node1 == node2
    }

    fn is_same_group(&self, node1: NodeIndex, node2: NodeIndex) -> bool {
        let node1 = self.graph.node_weight(node1).unwrap();
        let node2 = self.graph.node_weight(node2).unwrap();

        node1.group == node2.group
    }
    
    pub fn is_adj(&self, from: NodeIndex, to: NodeIndex) -> bool {
        self.is_interferenced_with(from, to)
    }
    
    pub fn outedges_of(&self, node: NodeIndex) -> Vec<NodeIndex> {
        self.graph.neighbors(node).collect()
    }
    
    pub fn outdegree_of(&self, node: NodeIndex) -> usize {
        self.outedges_of(node).len()
    }
    
    pub fn indegree_of(&self, node: NodeIndex) -> usize {
        self.outdegree_of(node)
    }
    
    pub fn degree_of(&self, node: NodeIndex) -> usize {
        self.outdegree_of(node)
    }
    
    pub fn print(&self, context: &FunctionContext) {
        use compiler::backend::reg_alloc::graph_coloring::petgraph::dot::Dot;
        use compiler::backend::reg_alloc::graph_coloring::petgraph::dot::Config;

        debug!("");
        debug!("Interference Graph");

        debug!("nodes:");
        for id in self.nodes.keys() {
            let val = context.get_value(*id).unwrap().value();
            debug!("Reg {} -> {:?}", val, self.nodes.get(&id).unwrap());
        }

        debug!("moves:");
        for mov in self.moves.iter() {
            debug!("Move {:?} -> {:?}", mov.from, mov.to);
        }

        debug!("graph:");
        debug!("\n\n{:?}\n", Dot::with_config(&self.graph, &[Config::EdgeNoLabel]));
        debug!("");
    }
}

fn build_live_set (cf: &mut CompiledFunction) {
    info!("start building live set");

    let n_insts = cf.mc().number_of_insts();

    let mut livein  : Vec<LinkedHashSet<MuID>> = vec![LinkedHashSet::new(); n_insts];
    let mut liveout : Vec<LinkedHashSet<MuID>> = vec![LinkedHashSet::new(); n_insts];

    let mut is_changed = true;

    while is_changed {
        // reset
        is_changed = false;

        for n in 0..n_insts {
            let in_set_old = livein[n].clone();
            let out_set_old = liveout[n].clone();

            // in[n] <- use[n] + (out[n] - def[n]);
            {
                let ref mut inset = livein[n];

                inset.clear();

                // (1) in[n] = use[n]
                inset.add_from_vec(cf.mc().get_inst_reg_uses(n));
                // (2) + out[n]
                inset.add_all(liveout[n].clone());
                // (3) - def[n]
                for def in cf.mc().get_inst_reg_defines(n) {
                    inset.remove(&def);
                }
            }

            // out[n] <- union(in[s] for every successor s of n)
            {
                let ref mut outset = liveout[n];
                outset.clear();

                for s in cf.mc().get_succs(n) {
                    outset.add_all(livein[*s].clone());
                }
            }

            // is in/out changed in this iteration?
            let n_changed = !in_set_old.equals(&livein[n]) || !out_set_old.equals(&liveout[n]);

            is_changed = is_changed || n_changed;
        }
    }

    for block in cf.mc().get_all_blocks().to_vec() {
        let start_inst = cf.mc().get_block_range(&block).unwrap().start;
        cf.mc_mut().set_ir_block_livein(&block, livein[start_inst].clone().to_vec());

        let end_inst = cf.mc().get_block_range(&block).unwrap().end;
        cf.mc_mut().set_ir_block_liveout(&block, liveout[end_inst].clone().to_vec());
    }
}

// from Tailoring Graph-coloring Register Allocation For Runtime Compilation, Figure 4
pub fn build_chaitin_briggs (cf: &mut CompiledFunction, func: &MuFunctionVersion) -> InterferenceGraph {
    build_live_set(cf);
    
    let mut ig = InterferenceGraph::new();
    
    // precolor machine register nodes
    for reg in backend::all_regs().values() {
        let reg_id = reg.extract_ssa_id().unwrap();
        let node = ig.new_node(reg_id, &func.context);

        let precolor = backend::get_color_for_precolroed(reg_id);

        ig.color_node(node, precolor);
    }
    
    // Initialize and creates nodes for all the involved temps/regs
    for i in 0..cf.mc().number_of_insts() {
        for reg_id in cf.mc().get_inst_reg_defines(i) {
            ig.new_node(reg_id, &func.context);
        }
        
        for reg_id in cf.mc().get_inst_reg_uses(i) {
            ig.new_node(reg_id, &func.context);
        }
    }
    
    for block in cf.mc().get_all_blocks() {
        // Current_Live(B) = LiveOut(B)
        let mut current_live = LinkedHashSet::from_vec(match cf.mc().get_ir_block_liveout(&block) {
            Some(liveout) => liveout.to_vec(),
            None => panic!("cannot find liveout for block {}", block)
        });
        if cfg!(debug_assertions) {
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
        trace!("Block{}: range = {:?}", block, range.as_ref().unwrap());
        
        // for every inst I in reverse order
        for i in range.unwrap().rev() {
            if cfg!(debug_assertions) {
                trace!("Block{}: Inst{}: start. current_live:", block, i);
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
                            trace!("add move between {} and {}",
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
            trace!("Block{}: Inst{}: src={:?}", block, i, src);
            
            // for every definition D in I
            for d in cf.mc().get_inst_reg_defines(i) {
                trace!("Block{}: Inst{}: for definition {}", block, i, func.context.get_temp_display(d));
                // add an interference from D to every element E in Current_Live - {D}
                // creating nodes if necessary
                for e in current_live.iter() {
                    trace!("Block{}: Inst{}: for each live {}",
                           block, i,
                           func.context.get_temp_display(*e));
                    if src.is_none() || (src.is_some() && *e != src.unwrap()) {
                        let from = ig.get_node(d);
                        let to = ig.get_node(*e);
                        
                        if !ig.is_same_node(from, to) &&ig.is_same_group(from, to) && !ig.is_adj(from, to) {
                            if !ig.is_colored(from) {
                                trace!("Block{}: Inst{}: add interference between {} and {}",
                                       block, i,
                                       func.context.get_temp_display(d),
                                       func.context.get_temp_display(*e));
                                ig.add_interference_edge(from, to);
                            }
                            if !ig.is_colored(to) {
                                trace!("Block{}: Inst{}: add interference between {} and {}",
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
                trace!("Block{}: Inst{}: remove define {} from current_live",
                       block, i,
                       func.context.get_temp_display(d));
                // remove D from Current_Live
                current_live.remove(&d);
            }
            
            // for every use U in I
            for u in cf.mc().get_inst_reg_uses(i) {
                trace!("Block{}: Inst{}: add use {} to current_live",
                       block, i,
                       func.context.get_temp_display(u));
                // add U to Current_live
                current_live.insert(u);
            }

            if cfg!(debug_assertions) {
                trace!("Block{}: Inst{}: done. current_live:", block, i);
                for ele in current_live.iter() {
                    trace!("{}", func.context.get_temp_display(*ele));
                }
            }
        }
    }
    
    ig
}

// from tony's code src/RegAlloc/Liveness.java
// this function is no longer used
//#[allow(dead_code)]
//pub fn build (cf: &CompiledFunction, func: &MuFunctionVersion) -> InterferenceGraph {
//    let mut ig = InterferenceGraph::new();
//
//    // precolor machine register nodes
//    for reg in backend::all_regs().values() {
//        let reg_id = reg.extract_ssa_id().unwrap();
//        let node = ig.new_node(reg_id, &func.context);
//        ig.color_node(node, reg_id);
//    }
//
//    // Liveness Analysis
//    let n_insts = cf.mc().number_of_insts();
//    let mut live_in : Vec<Vec<MuID>> = vec![vec![]; n_insts];
//    let mut live_out : Vec<Vec<MuID>> = vec![vec![]; n_insts];
//    let mut work_list : LinkedList<usize> = LinkedList::new();
//
//    // Initialize 'in' sets for each node in the flow graph
//    // and creates nodes for all the involved temps/regs
//    for i in 0..n_insts {
//        let ref mut in_set = live_in[i];
//
//        for reg_id in cf.mc().get_inst_reg_defines(i) {
//            ig.new_node(reg_id, &func.context);
//        }
//
//        for reg_id in cf.mc().get_inst_reg_uses(i) {
//            ig.new_node(reg_id, &func.context);
//
//            in_set.push(reg_id);
//        }
//
//        work_list.push_front(i);
//    }
//
//    // all nodes has been added, we init graph (create adjacency matrix)
//    ig.init_graph();
//
//    // compute liveIn and liveOut iteratively
//    trace!("build live outs");
//    while !work_list.is_empty() {
//        let n = work_list.pop_front().unwrap();
//        trace!("build liveout for #{}", n);
//        let ref mut out_set = live_out[n];
//
//        // out = union(in[succ]) for all succs
//        for succ in cf.mc().get_succs(n) {
//            trace!("add successor's livein {:?} to #{}", &live_in[*succ], n);
//            vec_utils::add_all(out_set, &live_in[*succ]);
//        }
//
//        // in = use(i.e. live_in) + (out - def)
//        let mut diff = out_set.clone();
//        for def in cf.mc().get_inst_reg_defines(n) {
//            vec_utils::remove_value(&mut diff, def);
//            trace!("removing def: {}", def);
//            trace!("diff = {:?}", diff);
//        }
//        trace!("out - def = {:?}", diff);
//
//        if !diff.is_empty() {
//            let ref mut in_set = live_in[n];
//            trace!("in = (use) {:?}", in_set);
//
//            if vec_utils::add_all(in_set, &diff) {
//                for p in cf.mc().get_preds(n) {
//                    work_list.push_front(*p);
//                }
//            }
//        }
//        trace!("in = use + (out - def) = {:?}", live_in[n]);
//    }
//
//    // debug live-outs
//    if cfg!(debug_assertions) {
//        trace!("check live-outs");
//        for n in 0..n_insts {
//            let ref mut live = live_out[n];
//            trace!("#{}\t{:?}", n, live);
//        }
//    }
//
//    // build interference graph
//    for n in 0..n_insts {
//        let ref mut live = live_out[n];
//
//        let src : Option<MuID> = {
//            if cf.mc().is_move(n) {
//                let src = cf.mc().get_inst_reg_uses(n);
//                let dst = cf.mc().get_inst_reg_defines(n);
//
//                // src may be an immediate number
//                // but dest is definitly a register
//                debug_assert!(dst.len() == 1);
//
//                if src.len() == 1 {
//                    let node1 = ig.get_node(src[0]);
//                    let node2 = ig.get_node(dst[0]);
//                    ig.add_move(node1, node2);
//
//                    Some(src[0])
//                } else {
//                    None
//                }
//            } else {
//                None
//            }
//        };
//
//        for d in cf.mc().get_inst_reg_defines(n) {
//            for t in live.iter() {
//                if src.is_none() || (src.is_some() && *t != src.unwrap()) {
//                    let from = ig.get_node(d);
//                    let to = ig.get_node(*t);
//
//                    if !ig.is_same_node(from, to) && !ig.is_adj(from, to) {
//                        if !ig.is_colored(from) {
//                            ig.add_interference_edge(from, to);
//                        }
//                        if !ig.is_colored(to) {
//                            ig.add_interference_edge(to, from);
//                        }
//                    }
//                }
//            }
//        }
//
//        for d in cf.mc().get_inst_reg_defines(n) {
//            vec_utils::remove_value(live, d);
//        }
//
//        for u in cf.mc().get_inst_reg_uses(n) {
//            live.push(u);
//        }
//    }
//
//    ig
//}
