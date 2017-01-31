extern crate hprof;

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
        let src = {
            let temp_src = self.get_temp_of(src);
            if temp_src < MACHINE_ID_END {
                let alias = backend::get_color_for_precolored(self.get_temp_of(src));
                self.get_node(alias)
            } else {
                src
            }
        };


        let dst = {
            let temp_dst = self.get_temp_of(dst);
            if temp_dst < MACHINE_ID_END {
                let alias = backend::get_color_for_precolored(self.get_temp_of(dst));
                self.get_node(alias)
            } else {
                dst
            }
        };

        self.moves.insert(Move{from: src, to: dst});
    }
    
    pub fn add_interference_edge(&mut self, from: NodeIndex, to: NodeIndex) {
        self.graph.update_edge(from, to, ());

        // if one of the node is machine register, we also add
        // interference edge to its alias
        // e.g. if we have %a, %edi interferenced,
        // we also add %a, %rdi interference

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

    pub fn is_interferenced_with(&self, node1: NodeIndex, node2: NodeIndex) -> bool {
        let edge = self.graph.find_edge(node1, node2);

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

const TRACE_BUILD_LIVE_SET : bool = false;

fn build_live_set (cf: &mut CompiledFunction, func: &MuFunctionVersion) {
    info!("---start building live set---");

    let cfg = local_liveness_analysis(cf, func);
    global_liveness_analysis(cfg, cf, func);

    info!("---finish building live set---");
}

#[derive(Clone, Debug)]
struct CFGBlockNode {
    block: String,

    pred: Vec<String>,
    succ: Vec<String>,

    uses:  Vec<MuID>,
    defs: Vec<MuID>
}

fn local_liveness_analysis (cf: &mut CompiledFunction, func: &MuFunctionVersion) -> HashMap<String, CFGBlockNode> {
    info!("---local liveness analysis---");
    let mc = cf.mc();

    let mut ret = hashmap!{};

    let all_blocks = mc.get_all_blocks();

    // create maps (start_inst -> name) and (end_inst -> name)
    let mut start_inst_map : HashMap<usize, &str> = hashmap!{};
    let mut end_inst_map   : HashMap<usize, &str> = hashmap!{};
    for block in all_blocks.iter() {
        let range = match mc.get_block_range(block) {
            Some(range) => range,
            None => panic!("cannot find range for block {}", block)
        };

        debug!("Block {}: start_inst={}, end_inst(inclusive)={}", block, range.start, range.end-1);
        start_inst_map.insert(range.start, block);
        end_inst_map.insert(range.end - 1, block);
    }

    // local liveness analysis
    for block in mc.get_all_blocks().iter() {
        trace!("---block {}---", block);
        let range = mc.get_block_range(block).unwrap();

        let start_inst = range.start;
        let end        = range.end;

        let mut livein = vec![];
        let mut all_defined : HashSet<MuID> = HashSet::new();

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

        let defs : Vec<MuID> = all_defined.into_iter().collect();

        let preds : Vec<String> = {
            let mut ret = vec![];

            // start_inst is the first instruction
            // start_inst-1 is the label for the block
            // FIXME: this is confusing! label should be an attribute to an instruction
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

            for succ in mc.get_succs(end - 1).into_iter() {
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

        trace!("as CFGNode {:?}", node);

        ret.insert(block.clone(), node);
    }

    ret
}

//fn topological_sort_cfg(entry: String, cfg: HashMap<String, CFGBlockNode>) -> Vec<CFGBlockNode> {
//    let mut ret = vec![];
//    // for all nodes i
//    //   mark[i] <- false
//    let mut mark = {
//        let mut ret = hashmap!{};
//        for str in cfg.keys() {
//            ret.insert(str.clone(), false);
//        }
//
//        ret
//    };
//
//    // dfs(start-node)
//    dfs(entry, &cfg, &mut mark, &mut ret);
//
//    ret.reverse();
//
//    ret
//}
//
//fn dfs(node: String, cfg: &HashMap<String, CFGBlockNode>, mark: &mut HashMap<String, bool>, sorted: &mut Vec<CFGBlockNode>) {
//    // if mark[i] = false
//    if !mark.get(&node).unwrap() {
//        mark.insert(node.clone(), true);
//
//        let cfg_node = cfg.get(&node).unwrap().clone();
//        for succ in cfg_node.succ.iter() {
//            dfs(succ.clone(), cfg, mark, sorted);
//        }
//
//        sorted.push(cfg_node);
//    }
//}

fn global_liveness_analysis(blocks: HashMap<String, CFGBlockNode>, cf: &mut CompiledFunction, func: &MuFunctionVersion) {
    let n_nodes = blocks.len();

    // init live in and live out
    let mut livein  : HashMap<String, LinkedHashSet<MuID>> = {
        let mut ret = hashmap!{};
        for name in blocks.keys() {
            ret.insert(name.clone(), LinkedHashSet::new());
        }
        ret
    };
    let mut liveout  : HashMap<String, LinkedHashSet<MuID>> = {
        let mut ret = hashmap!{};
        for name in blocks.keys() {
            ret.insert(name.clone(), LinkedHashSet::new());
        }
        ret
    };

    let mut is_changed = true;
    let mut i = 0;

    while is_changed {
        trace!("---iteration {}---", i);
        i += 1;

        // reset
        is_changed = false;

        for node in blocks.keys() {
            let cfg_node = blocks.get(node).unwrap();

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

            if TRACE_BUILD_LIVE_SET {
                trace!("block {}", node);
                trace!("in(old)  = {:?}", in_set_old);
                trace!("in(new)  = {:?}", livein.get(node).unwrap());
                trace!("out(old) = {:?}", out_set_old);
                trace!("out(new) = {:?}", liveout.get(node).unwrap());
            }

            is_changed = is_changed || n_changed;
        }
    }

    for block in blocks.keys() {
        let livein : Vec<MuID> = livein.get(block).unwrap().clone().iter().map(|x| *x).collect();
        {
            let display_array : Vec<String> =  livein.iter().map(|x| func.context.get_temp_display(*x)).collect();
            trace!("livein  for block {}: {:?}", block, display_array);
        }
        cf.mc_mut().set_ir_block_livein(block, livein);

        let liveout : Vec<MuID> = liveout.get(block).unwrap().clone().iter().map(|x| *x).collect();
        {
            let display_array : Vec<String> = liveout.iter().map(|x| func.context.get_temp_display(*x)).collect();
            trace!("liveout for block {}: {:?}", block, display_array);
        }
        cf.mc_mut().set_ir_block_liveout(block, liveout);
    }
}

#[allow(dead_code)]
fn naive_global_liveness_analysis(cf: &mut CompiledFunction, func: &MuFunctionVersion) {
    let n_insts = cf.mc().number_of_insts();

    let mut livein  : Vec<LinkedHashSet<MuID>> = vec![LinkedHashSet::new(); n_insts];
    let mut liveout : Vec<LinkedHashSet<MuID>> = vec![LinkedHashSet::new(); n_insts];

    let mut is_changed = true;

    let mut i = 0;
    while is_changed {
        trace!("---iteration {}---", i);
        i += 1;

        // reset
        is_changed = false;

        for n in 0..n_insts {
            let in_set_old = livein[n].clone();
            let out_set_old = liveout[n].clone();

            // in[n] <- use[n] + (out[n] - def[n]);
            {
                let ref mut inset = livein[n];

                inset.clear();

                // (1) out[n] - def[n]
                inset.add_all(liveout[n].clone());
                for def in cf.mc().get_inst_reg_defines(n) {
                    inset.remove(&def);
                }
                // (2) in[n] + (out[n] - def[n])
                inset.add_from_vec(cf.mc().get_inst_reg_uses(n));
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

            if TRACE_BUILD_LIVE_SET {
                trace!("inst {}", n);
                trace!("in(old)  = {:?}", in_set_old);
                trace!("in(new)  = {:?}", livein[n]);
                trace!("out(old) = {:?}", out_set_old);
                trace!("out(new) = {:?}", liveout[n]);
            }

            is_changed = is_changed || n_changed;
        }
    }
}

// from Tailoring Graph-coloring Register Allocation For Runtime Compilation, Figure 4
pub fn build_chaitin_briggs (cf: &mut CompiledFunction, func: &MuFunctionVersion) -> InterferenceGraph {
    let _p = hprof::enter("regalloc: build live set");
    build_live_set(cf, func);
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

    drop(_p);
    info!("---finish building interference graph---");
    ig
}