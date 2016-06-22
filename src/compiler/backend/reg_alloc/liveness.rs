extern crate nalgebra;

use vm::machine_code::CompiledFunction;
use vm::machine_code::MachineCode;
use ast::ir::*;
use ast::types;
use compiler::backend;
use utils::vec_utils;

use std::collections::LinkedList;
use std::collections::{HashMap, HashSet};

use self::nalgebra::DMatrix;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Node(usize);
#[derive(Clone, Debug, PartialEq)]
pub struct NodeProperty {
    color: Option<MuID>,
    group: backend::RegGroup,
    temp: MuID,
    spill_cost: f32
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Move{pub from: Node, pub to: Node}

pub struct InterferenceGraph {
    nodes: HashMap<MuID, Node>,
    nodes_property: HashMap<Node, NodeProperty>,
    
    matrix: Option<DMatrix<bool>>,
    
    moves: HashSet<Move>,
}

impl InterferenceGraph {
    fn new() -> InterferenceGraph {
        InterferenceGraph {
            nodes: HashMap::new(),
            nodes_property: HashMap::new(),
            matrix: None,
            moves: HashSet::new()
        }
    }
    
    fn new_node(&mut self, reg_id: MuID, context: &FunctionContext) -> Node {
        let entry = context.get_value(reg_id).unwrap();
        
        if !self.nodes.contains_key(&reg_id) {
            let index = self.nodes.len();
            let node = Node(index);
            
            // add the node
            self.nodes.insert(reg_id, node.clone());
            
            // add node property
            let group = {
                let ref ty = entry.ty;
                if types::is_scalar(ty) {
                    if types::is_fp(ty) {
                        backend::RegGroup::FPR
                    } else {
                        backend::RegGroup::GPR
                    }
                } else {
                    unimplemented!()
                }
            };
            let property = NodeProperty {
                color: None,
                group: group,
                temp: reg_id,
                spill_cost: 0.0f32
            };
            self.nodes_property.insert(node, property);
        } 
        
        
        let node = * self.nodes.get(&reg_id).unwrap();
        
        // increase node spill cost
        let property = self.nodes_property.get_mut(&node).unwrap();
        property.spill_cost += 1.0f32;
        
        node
    }
    
    pub fn get_node(&self, reg: MuID) -> Node {
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
    
    pub fn nodes(&self) -> Vec<Node> {
        let mut ret = vec![];
        for node in self.nodes.values() {
            ret.push(node.clone());
        }
        ret
    }
    
    pub fn moves(&self) -> &HashSet<Move> {
        &self.moves
    }
    
    pub fn n_nodes(&self) -> usize {
        self.nodes.len()
    }
    
    fn init_graph(&mut self) {
        let len = self.nodes.len();
        self.matrix = Some(DMatrix::from_element(len, len, false));
    }
    
    fn add_move(&mut self, src: Node, dst: Node) {
        self.moves.insert(Move{from: src, to: dst});
    }
    
    pub fn add_interference_edge(&mut self, from: Node, to: Node) {
        // only if two nodes are from the same RegGroup,
        // they may interefere
        if self.nodes_property.get(&from).unwrap().group 
           == self.nodes_property.get(&to).unwrap().group {
            self.matrix.as_mut().unwrap()[(from.0, to.0)] = true;
        }
    }
    
    pub fn color_node(&mut self, node: Node, color: MuID) {
        self.nodes_property.get_mut(&node).unwrap().color = Some(color);
    }
    
    pub fn is_colored(&self, node: Node) -> bool {
        self.nodes_property.get(&node).unwrap().color.is_some()
    }
    
    pub fn get_color_of(&self, node: Node) -> Option<MuID> {
        self.nodes_property.get(&node).unwrap().color
    }
    
    pub fn get_group_of(&self, node: Node) -> backend::RegGroup {
        self.nodes_property.get(&node).unwrap().group
    }
    
    pub fn get_temp_of(&self, node: Node) -> MuID {
        self.nodes_property.get(&node).unwrap().temp
    }
    
    pub fn get_spill_cost(&self, node: Node) -> f32 {
        self.nodes_property.get(&node).unwrap().spill_cost
    }
    
    fn is_same_node(&self, node1: Node, node2: Node) -> bool {
        node1 == node2
    }
    
    pub fn is_adj(&self, from: Node, to: Node) -> bool {
        let ref matrix = self.matrix.as_ref().unwrap();
        
        matrix[(from.0, to.0)] || matrix[(to.0, from.0)]
    }
    
    pub fn outedges_of(&self, node: Node) -> Vec<Node> {
        let mut ret = vec![];
        let matrix = self.matrix.as_ref().unwrap();
        
        for i in 0..self.nodes.len() {
            if matrix[(node.0, i)] {
                ret.push(Node(i));
            }
        }
        
        ret
    }
    
    pub fn outdegree_of(&self, node: Node) -> usize {
        let mut count = 0;
        for i in 0..self.nodes.len() {
            if self.matrix.as_ref().unwrap()[(node.0, i)] {
                count += 1;
            }
        }
        
        count
    }
    
    pub fn indegree_of(&self, node: Node) -> usize {
        let mut count = 0;
        for i in 0..self.nodes.len() {
            if self.matrix.as_ref().unwrap()[(i, node.0)] {
                count += 1;
            }
        }
        
        count
    }
    
    pub fn degree_of(&self, node: Node) -> usize {
        self.outdegree_of(node) + self.indegree_of(node)
    }
    
    pub fn print(&self) {
        println!("");
        println!("Interference Graph");

        println!("nodes:");
        for id in self.nodes.keys() {
            println!("Reg {} -> {:?}", id, self.nodes.get(&id).unwrap());
        }

        println!("color:");
        for (n, c) in self.nodes_property.iter() {
            println!("{:?} -> Color/Reg {:?}", n, c);
        }
        println!("moves:");
        for mov in self.moves.iter() {
            println!("Move {:?} -> {:?}", mov.from, mov.to);
        }
        println!("graph:");
        {
            let node_to_reg_id = {
                let mut ret : HashMap<Node, MuID> = HashMap::new();
                
                for reg in self.nodes.keys() {
                    ret.insert(*self.nodes.get(reg).unwrap(), *reg);
                }
                
                ret 
            };
            
            let matrix = self.matrix.as_ref().unwrap();
            for i in 0..matrix.ncols() {
                for j in 0..matrix.nrows() {
                    if matrix[(i, j)] {
                        let from_node = node_to_reg_id.get(&Node(i)).unwrap();
                        let to_node = node_to_reg_id.get(&Node(j)).unwrap();
                        
                        println!("Reg {} -> Reg {}", from_node, to_node);
                    }
                }
            }
        }
        println!("");
    }
}

pub fn is_machine_reg(reg: MuID) -> bool {
    if reg < RESERVED_NODE_IDS_FOR_MACHINE {
        true
    } else {
        false
    }
}

// from tony's code src/RegAlloc/Liveness.java
pub fn build (cf: &CompiledFunction, func: &MuFunction) -> InterferenceGraph {
    let mut ig = InterferenceGraph::new();
    
    // precolor machine register nodes
    for reg in backend::all_regs().iter() {
        let reg_id = reg.extract_ssa_id().unwrap();
        let node = ig.new_node(reg_id, &func.context);
        ig.color_node(node, reg_id);
    }
    
    // Liveness Analysis
    let n_insts = cf.mc.number_of_insts();
    let mut live_in : Vec<Vec<MuID>> = vec![vec![]; n_insts];
    let mut live_out : Vec<Vec<MuID>> = vec![vec![]; n_insts];
    let mut work_list : LinkedList<usize> = LinkedList::new();
    
    // Initialize 'in' sets for each node in the flow graph
    // and creates nodes for all the involved temps/regs
    for i in 0..n_insts {
        let ref mut in_set = live_in[i];
        
        for reg_id in cf.mc.get_inst_reg_defines(i) {
            let reg_id = *reg_id;
            ig.new_node(reg_id, &func.context);
        }
        
        for reg_id in cf.mc.get_inst_reg_uses(i) {
            let reg_id = *reg_id;
            ig.new_node(reg_id, &func.context);
            
            in_set.push(reg_id);
        }
        
        work_list.push_front(i);
    }
    
    // all nodes has been added, we init graph (create adjacency matrix)
    ig.init_graph();
    
    // compute liveIn and liveOut iteratively
    trace!("build live outs");
    while !work_list.is_empty() {
        let n = work_list.pop_front().unwrap();
//        trace!("build liveout for #{}", n);
        let ref mut out_set = live_out[n];
        
        // out = union(in[succ]) for all succs
        for succ in cf.mc.get_succs(n) {
//            trace!("add successor's livein {:?} to #{}", &live_in[*succ], n); 
            vec_utils::add_all(out_set, &live_in[*succ]);
        }
        
        // in = use(i.e. live_in) + (out - def) 
        let mut diff = out_set.clone();
        for def in cf.mc.get_inst_reg_defines(n) {
            vec_utils::remove_value(&mut diff, *def);
//            trace!("removing def: {}", *def);
//            trace!("diff = {:?}", diff);
        }
//        trace!("out - def = {:?}", diff);
        
        if !diff.is_empty() {
            let ref mut in_set = live_in[n];
            
            if vec_utils::add_all(in_set, &diff) {
                for p in cf.mc.get_preds(n) {
                    work_list.push_front(*p);
                }
            }
        }
//        trace!("in = use + (out - def) = {:?}", live_in[n]);
    }
    
    // debug live-outs
    if cfg!(debug_assertions) {
        trace!("check live-outs");
        for n in 0..n_insts {
            let ref mut live = live_out[n];
            trace!("#{}\t{:?}", n, live);
        }
    }
    
    // build interference graph
    for n in 0..n_insts {
        let ref mut live = live_out[n];
        
        let src : Option<MuID> = {
            if cf.mc.is_move(n) {
                let src = cf.mc.get_inst_reg_uses(n);
                let dst = cf.mc.get_inst_reg_defines(n);
                
                // src may be an immediate number
                // but dest is definitly a register
                debug_assert!(dst.len() == 1);
                
                if src.len() == 1 {
                    let node1 = ig.get_node(src[0]);
                    let node2 = ig.get_node(dst[0]);
                    ig.add_move(node1, node2);
                    
                    Some(src[0])
                } else {
                    None
                }
            } else {
                None
            }
        };
        
        for d in cf.mc.get_inst_reg_defines(n) {
            for t in live.iter() {
                if src.is_none() || (src.is_some() && *t != src.unwrap()) {
                    let from = ig.get_node(*d);
                    let to = ig.get_node(*t);
                    
                    if !ig.is_same_node(from, to) && !ig.is_adj(from, to) {
                        if !ig.is_colored(from) {
                            ig.add_interference_edge(from, to);
                        }
                        if !ig.is_colored(to) {
                            ig.add_interference_edge(to, from);
                        }
                    }
                }
            }
        }
        
        for d in cf.mc.get_inst_reg_defines(n) {
            vec_utils::remove_value(live, *d);
        }
        
        for u in cf.mc.get_inst_reg_uses(n) {
            live.push(*u);
        }
    }
    
    ig
}