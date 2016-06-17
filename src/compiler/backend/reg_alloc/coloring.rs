use ast::ir::MuID;
use compiler::backend::reg_alloc::liveness::InterferenceGraph;
use compiler::backend::reg_alloc::liveness::{Node, Move};
use vm::machine_code::CompiledFunction;

use compiler::backend;
use compiler::backend::reg_alloc::liveness::find_value;

use std::collections::LinkedList;
use std::collections::HashSet;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, ATOMIC_BOOL_INIT, Ordering}; 

const COALESCING : AtomicBool = ATOMIC_BOOL_INIT;

pub struct GraphColoring <'a> {
    ig: InterferenceGraph,
    cur_cf: &'a CompiledFunction,
    
    precolored: HashSet<Node>,
    colors: HashSet<MuID>,
    
    initial: LinkedList<Node>,
    degree: HashMap<Node, usize>,
    
    worklist_moves: LinkedList<Move>,
    movelist: HashMap<Node, Vec<Move>>,
    active_moves: HashSet<Move>,
    
    worklist_spill: LinkedList<Node>,
    worklist_freeze: HashSet<Node>,
    worklist_simplify: HashSet<Node>
}

impl <'a> GraphColoring <'a> {
    pub fn start (cf: &CompiledFunction, ig: InterferenceGraph) {
        let mut coloring = GraphColoring {
            ig: ig,
            cur_cf: cf, 
            
            precolored: HashSet::new(),
            colors: HashSet::new(),
            
            initial: LinkedList::new(),
            degree: HashMap::new(),
            
            worklist_moves: LinkedList::new(),
            movelist: HashMap::new(),
            active_moves: HashSet::new(),
            
            worklist_spill: LinkedList::new(),
            worklist_freeze: HashSet::new(),
            worklist_simplify: HashSet::new()
        };
        
        coloring.init();
    }
    
    fn init (&mut self) {
        COALESCING.store(true, Ordering::Relaxed);
        
        for reg in backend::all_regs().iter() {
            let reg_id = reg.extract_ssa_id().unwrap();
            let node = self.ig.get_node(reg_id);
            
            self.precolored.insert(node);
            self.colors.insert(reg_id);
        }
        
        for node in self.ig.nodes() {
            if !self.ig.is_colored(node) {
                self.initial.push_back(node);
                self.degree.insert(node, self.ig.outdegree_of(node));
            }
        }
        
        self.build();
        self.make_work_list();
        
        while {
            if !self.worklist_simplify.is_empty() {
                self.simplify();
            } else if !self.worklist_moves.is_empty() {
                self.coalesce();
            } else if !self.worklist_freeze.is_empty() {
                self.freeze();
            } else if !self.worklist_spill.is_empty() {
                self.select_spill();
            }
            
            ! (self.worklist_simplify.is_empty()
            && self.worklist_moves.is_empty()
            && self.worklist_freeze.is_empty()
            && self.worklist_spill.is_empty())
        } {}
        
        self.assign_colors();
    }
    
    fn simplify(&mut self) {
        unimplemented!()
    }
    
    fn coalesce(&mut self) {
        unimplemented!()
    }
    
    fn freeze(&mut self) {
        unimplemented!()
    }
    
    fn select_spill(&mut self) {
        unimplemented!()
    }
    
    fn assign_colors(&mut self) {
        unimplemented!()
    }
    
    fn build(&mut self) {
        if COALESCING.load(Ordering::Relaxed) {
            let ref ig = self.ig;
            let ref mut movelist = self.movelist;
            for m in ig.moves() {
                self.worklist_moves.push_back(m.clone());
                GraphColoring::movelist_mut(movelist, m.from).push(m.clone());
                GraphColoring::movelist_mut(movelist, m.to).push(m.clone());
            }
        }
    }
    
    fn make_work_list(&mut self) {
        while !self.initial.is_empty() {
            let node = self.initial.pop_front().unwrap();
            
            if {
                // condition: degree >= K
                let degree = self.ig.degree_of(node); 
                let n_regs = backend::number_of_regs_in_group(self.ig.get_group_of(node));
                
                degree >= n_regs
            } {
                self.worklist_spill.push_back(node);
            } else if self.is_move_related(node) {
                self.worklist_freeze.insert(node);
            } else {
                self.worklist_simplify.insert(node);
            }
        }
    }
    
    fn is_move_related(&mut self, node: Node) -> bool {
        !self.node_moves(node).is_empty()
    }
    
    fn node_moves(&mut self, node: Node) -> HashSet<Move> {
        let mut moves = HashSet::new();
        
        // addAll(active_moves)
        for m in self.active_moves.iter() {
            moves.insert(m.clone());
        }
        
        // addAll(worklist_moves)
        for m in self.worklist_moves.iter() {
            moves.insert(m.clone());
        }
        
        let mut retained = HashSet::new();
        let movelist = GraphColoring::movelist_mut(&mut self.movelist, node);
        for m in moves.iter() {
            if find_value(movelist, *m).is_some() {
                retained.insert(*m);
            }
        }
        
        retained
    }
    
    fn movelist_mut(list: &mut HashMap<Node, Vec<Move>>, node: Node) -> &mut Vec<Move> {
        if !list.contains_key(&node) {
            list.insert(node, Vec::new());
        }
        
        list.get_mut(&node).unwrap()
    }
}