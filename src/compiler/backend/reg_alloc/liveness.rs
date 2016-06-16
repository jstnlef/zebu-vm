extern crate nalgebra;

use vm::machine_code::CompiledFunction;
use vm::machine_code::MachineCode;
use ast::ir::*;
use compiler::backend::get_name_for_value as get_tag;

use std::collections::LinkedList;
use std::collections::{HashMap, HashSet};

use self::nalgebra::DMatrix;

type MatrixIndex = usize;

pub struct InterferenceGraph {
    nodes: HashMap<MuID, MatrixIndex>,
    
    matrix: Option<DMatrix<bool>>,
    color: HashMap<MuID, MuID>,
    
    moves: HashSet<(MuID, MuID)>
}

impl InterferenceGraph {
    fn new() -> InterferenceGraph {
        InterferenceGraph {
            nodes: HashMap::new(),
            matrix: None,
            color: HashMap::new(),
            moves: HashSet::new()
        }
    }
    
    fn new_node(&mut self, node: MuID) -> MatrixIndex {
        if !self.nodes.contains_key(&node) {
            let index = self.nodes.len();
            self.nodes.insert(node, index);
            
            index
        } else {
            * self.nodes.get(&node).unwrap()
        }
    }
    
    fn get_node(&self, node: MuID) -> MatrixIndex {
        match self.nodes.get(&node) {
            Some(index) => *index,
            None => panic!("do not have a node for {}", node)
        }
    }
    
    fn init_graph(&mut self) {
        let len = self.nodes.len();
        self.matrix = Some(DMatrix::from_element(len, len, false));
    }
    
    fn add_move(&mut self, src: MuID, dst: MuID) {
        self.moves.insert((src, dst));
    }
    
    fn add_interference_edge(&mut self, from: MuID, to: MuID) {
        let from_ix = self.get_node(from);
        let to_ix = self.get_node(to);
        
        self.matrix.as_mut().unwrap()[(from_ix, to_ix)] = true;
    }
    
    fn color_node(&mut self, node: MuID, color: MuID) {
        self.color.insert(node, color);
    }
    
    fn node_has_color(&self, node: MuID) -> bool {
        self.color.contains_key(&node)
    }
    
    fn is_same_node(&self, node1: MuID, node2: MuID) -> bool {
        let ix1 = self.get_node(node1);
        let ix2 = self.get_node(node2);
        
        ix1 == ix2
    }
    
    fn is_adj(&self, from: MuID, to: MuID) -> bool {
        let from_ix = self.get_node(from);
        let to_ix = self.get_node(to);
        
        let ref matrix = self.matrix.as_ref().unwrap();
        
        matrix[(from_ix, to_ix)] || matrix[(to_ix, from_ix)]
    }
    
    pub fn print(&self) {
        println!("");
        println!("Interference Graph");

        println!("color:");
        for (n, c) in self.color.iter() {
            println!("Node {} -> Color {}", n, c);
        }
        println!("moves:");
        for mov in self.moves.iter() {
            println!("Move {} -> {}", mov.0, mov.1);
        }
        println!("graph:");
        {
            let idx_to_node_id = {
                let mut ret : HashMap<MatrixIndex, MuID> = HashMap::new();
                
                for node_id in self.nodes.keys() {
                    ret.insert(*self.nodes.get(node_id).unwrap(), *node_id);
                }
                
                ret 
            };
            
            let matrix = self.matrix.as_ref().unwrap();
            for i in 0..matrix.ncols() {
                for j in 0..matrix.nrows() {
                    if matrix[(i, j)] {
                        let from_node = idx_to_node_id.get(&i).unwrap();
                        let to_node = idx_to_node_id.get(&j).unwrap();
                        
                        println!("Node {} -> Node {}", from_node, to_node);
                    }
                }
            }
        }
        println!("");
    }
    
    #[allow(dead_code)]
    pub fn print_symbols(&self, func: &MuFunction) {
        let ref context = func.context;
        
        println!("");
        println!("Interference Graph");
        
        println!("color:");
        for (n, c) in self.color.iter() {
            println!("Node {} -> Color {}", get_tag(*n, context), get_tag(*c, context));
        }
        println!("moves:");
        for mov in self.moves.iter() {
            println!("Move {} -> {}", get_tag(mov.0, context), get_tag(mov.1, context));
        }
        println!("graph:");
        {
            let idx_to_node_id = {
                let mut ret : HashMap<MatrixIndex, MuID> = HashMap::new();
                
                for node_id in self.nodes.keys() {
                    ret.insert(*self.nodes.get(node_id).unwrap(), *node_id);
                }
                
                ret 
            };
            
            let matrix = self.matrix.as_ref().unwrap();
            for i in 0..matrix.ncols() {
                for j in 0..matrix.nrows() {
                    if matrix[(i, j)] {
                        let from_node = idx_to_node_id.get(&i).unwrap();
                        let to_node = idx_to_node_id.get(&j).unwrap();
                        
                        println!("Node {} -> Node {}", get_tag(*from_node, context), get_tag(*to_node, context));
                    }
                }
            }
        }
        println!("");
    }
}

fn is_machine_reg(node: MuID) -> bool {
    if node < RESERVED_NODE_IDS_FOR_MACHINE {
        true
    } else {
        false
    }
}

// from tony's code src/RegAlloc/Liveness.java
pub fn build (cf: &CompiledFunction) -> InterferenceGraph {
    let mut ig = InterferenceGraph::new();
    
    // move precolor nodes to later iteration of registers
    
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
            ig.new_node(reg_id);
            
            // precolor
            if is_machine_reg(reg_id) {
                ig.color_node(reg_id, reg_id);
            }
        }
        
        for reg_id in cf.mc.get_inst_reg_uses(i) {
            let reg_id = *reg_id;
            
            ig.new_node(reg_id);
            in_set.push(reg_id);
            
            // precolor
            if is_machine_reg(reg_id) {
                ig.color_node(reg_id, reg_id);
            }
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
            add_all(out_set, &live_in[*succ]);
        }
        
        // in = use(i.e. live_in) + (out - def) 
        let mut diff = out_set.clone();
        for def in cf.mc.get_inst_reg_defines(n) {
            remove_value(&mut diff, *def);
//            trace!("removing def: {}", *def);
//            trace!("diff = {:?}", diff);
        }
//        trace!("out - def = {:?}", diff);
        
        if !diff.is_empty() {
            let ref mut in_set = live_in[n];
            
            if add_all(in_set, &diff) {
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
                    ig.add_move(src[0], dst[0]);
                    
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
                    let from = *d;
                    let to = *t;
                    
                    if !ig.is_same_node(from, to) && !ig.is_adj(from, to) {
                        if !ig.node_has_color(from) {
                            ig.add_interference_edge(from, to);
                        }
                        if !ig.node_has_color(to) {
                            ig.add_interference_edge(to, from);
                        }
                    }
                }
            }
        }
        
        for d in cf.mc.get_inst_reg_defines(n) {
            remove_value(live, *d);
        }
        
        for u in cf.mc.get_inst_reg_uses(n) {
            live.push(*u);
        }
    }
    
    ig
}

use std::fmt;

fn add_all<T: Copy + PartialEq> (vec: &mut Vec<T>, vec2: &Vec<T>) -> bool {
    let mut is_changed = false;
    
    for i in vec2.iter() {
        if !vec.contains(i) {
            vec.push(*i);
            is_changed = true;
        }
    }
    
    is_changed
}

fn find_value<T: Ord + fmt::Debug + fmt::Display> (vec: &mut Vec<T>, val: T) -> Option<usize> {
    for i in 0..vec.len() {
        if vec[i] == val {
            return Some(i);
        }
    }
    
    None
}

fn remove_value<T: Ord + fmt::Debug + fmt::Display> (vec: &mut Vec<T>, val: T) {
    match find_value(vec, val) {
        Some(index) => {vec.remove(index);},
        None => {} // do nothing
    }
}