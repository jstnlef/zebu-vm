use ast::ir::MuID;
use compiler::backend::reg_alloc::liveness::InterferenceGraph;
use compiler::backend::reg_alloc::liveness::{Node, Move};
use vm::machine_code::CompiledFunction;

use compiler::backend;
use utils::vec_utils;
use utils::hashset_utils;

use std::cell::RefCell;
use std::collections::HashSet;
use std::collections::HashMap;

const COALESCING : bool = true;

pub struct GraphColoring <'a> {
    ig: InterferenceGraph,
    cur_cf: &'a CompiledFunction,
    
    precolored: HashSet<Node>,
    colors: HashMap<backend::RegGroup, HashSet<MuID>>,
    colored_nodes: Vec<Node>,
    
    initial: Vec<Node>,
    degree: HashMap<Node, isize>,
    
    worklist_moves: Vec<Move>,
    movelist: HashMap<Node, RefCell<Vec<Move>>>,
    active_moves: HashSet<Move>,
    coalesced_nodes: HashSet<Node>,
    coalesced_moves: HashSet<Move>,
    constrained_moves: HashSet<Move>,
    alias: HashMap<Node, Node>,
    
    worklist_spill: Vec<Node>,
    spillable: HashMap<MuID, bool>,
    spilled_nodes: Vec<Node>,
    
    worklist_freeze: HashSet<Node>,
    frozen_moves: HashSet<Move>,
    
    worklist_simplify: HashSet<Node>,
    select_stack: Vec<Node>
}

impl <'a> GraphColoring <'a> {
    pub fn start (cf: &CompiledFunction, ig: InterferenceGraph) -> GraphColoring {
        let mut coloring = GraphColoring {
            ig: ig,
            cur_cf: cf, 
            
            precolored: HashSet::new(),
            colors: {
                let mut map = HashMap::new();
                map.insert(backend::RegGroup::GPR, HashSet::new());
                map.insert(backend::RegGroup::FPR, HashSet::new());
                map
            },
            colored_nodes: Vec::new(),
            
            initial: Vec::new(),
            degree: HashMap::new(),
            
            worklist_moves: Vec::new(),
            movelist: HashMap::new(),
            active_moves: HashSet::new(),
            coalesced_nodes: HashSet::new(),
            coalesced_moves: HashSet::new(),
            constrained_moves: HashSet::new(),
            alias: HashMap::new(),
            
            worklist_spill: Vec::new(),
            spillable: HashMap::new(),
            spilled_nodes: Vec::new(),
            
            worklist_freeze: HashSet::new(),
            frozen_moves: HashSet::new(),
            
            worklist_simplify: HashSet::new(),
            select_stack: Vec::new()
        };
        
        coloring.init();
        
        coloring
    }
    
    fn init (&mut self) {
        trace!("Initializing coloring allocator...");
        
        // for all machine registers
        for reg in backend::all_regs().iter() {
            let reg_id = reg.extract_ssa_id().unwrap();
            let node = self.ig.get_node(reg_id);
            
            self.precolored.insert(node);
            {
                let group = backend::pick_group_for_reg(reg_id);
                self.colors.get_mut(&group).unwrap().insert(reg_id);
            }
        }
        
        for node in self.ig.nodes() {
            if !self.ig.is_colored(node) {
                self.initial.push(node);
                let outdegree = self.ig.outdegree_of(node);
                self.degree.insert(node, outdegree as isize);
                trace!("{} has outdegree of {}", self.node_info(node), outdegree);
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
    
    fn build(&mut self) {
        if COALESCING {
            trace!("coalescing enabled, build move list");
            let ref ig = self.ig;
            let ref mut movelist = self.movelist;
            for m in ig.moves() {
                trace!("add {:?} to movelist", m);
                self.worklist_moves.push(m.clone());
                GraphColoring::movelist_mut(movelist, m.from).borrow_mut().push(m.clone());
                GraphColoring::movelist_mut(movelist, m.to).borrow_mut().push(m.clone());
            }
        } else {
            trace!("coalescing disabled");
        }
    }
    
    fn make_work_list(&mut self) {
        trace!("Making work list from initials...");
        while !self.initial.is_empty() {
            let node = self.initial.pop().unwrap();
            
            if {
                // condition: degree >= K
                let degree = self.ig.degree_of(node); 
                let n_regs = self.n_regs_for_node(node);
                
                degree >= n_regs
            } {
                trace!("{} 's degree >= reg number limit (K), push to spill list", self.node_info(node));
                self.worklist_spill.push(node);
            } else if self.is_move_related(node) {
                trace!("{} is move related, push to freeze list", self.node_info(node));
                self.worklist_freeze.insert(node);
            } else {
                trace!("{} has small degree and not move related, push to simplify list", self.node_info(node));
                self.worklist_simplify.insert(node);
            }
        }
    }
    
    fn n_regs_for_node(&self, node: Node) -> usize {
        backend::number_of_regs_in_group(self.ig.get_group_of(node))
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
        let movelist = &GraphColoring::movelist_mut(&mut self.movelist, node).borrow();
        for m in moves.iter() {
            if vec_utils::find_value(movelist, *m).is_some() {
                retained.insert(*m);
            }
        }
        
        retained
    }
    
    // avoid using &mut self as argument
    // in build(), we will need to mutate on self.movelist while
    // holding an immmutable reference of self(self.ig)
    fn movelist_mut(list: &mut HashMap<Node, RefCell<Vec<Move>>>, node: Node) -> &RefCell<Vec<Move>> {
        GraphColoring::movelist_check(list, node);
        unsafe {GraphColoring::movelist_nocheck(list, node)}
    }
    
    fn movelist_check(list: &mut HashMap<Node, RefCell<Vec<Move>>>, node: Node) {
        if !list.contains_key(&node) {
            list.insert(node, RefCell::new(Vec::new()));
        }
    }
    
    // allows getting the Vec<Move> without a mutable reference of the hashmap
    unsafe fn movelist_nocheck(list: &HashMap<Node, RefCell<Vec<Move>>>, node: Node) -> &RefCell<Vec<Move>> {
        list.get(&node).unwrap()
    }
    
    fn simplify(&mut self) {
        // remove next element from worklist_simplify, we know its not empty
        let node = hashset_utils::pop_first(&mut self.worklist_simplify).unwrap();
        
        trace!("Simplifying {}", self.node_info(node));
        
        self.select_stack.push(node);
        
        for m in self.adjacent(node) {
            trace!("decrement degree of its adjacent node {}", self.node_info(m));
            self.decrement_degree(m);
        }
    }
    
    fn adjacent(&self, n: Node) -> HashSet<Node> {
        let mut adj = HashSet::new();
        
        // add n's successors
        for s in self.ig.outedges_of(n) {
            adj.insert(s);
        }
        
        // removeAll(select_stack)
        for s in self.select_stack.iter() {
            adj.remove(s);
        }
        
        // removeAll(coalesced_nodes)
        for s in self.coalesced_nodes.iter() {
            adj.remove(s);
        }
        
        adj
    }
    
    fn degree(&self, n: Node) -> isize {
        match self.degree.get(&n) {
            Some(d) => *d,
            None => 0
        }
    }
    
    fn decrement_degree(&mut self, n: Node) {
        let d = self.degree(n);
        self.degree.insert(n, d - 1);
        
        if d == self.n_regs_for_node(n) as isize {
            trace!("{}'s degree is K, no longer need to spill it", self.node_info(n));
            let mut nodes = self.adjacent(n);
            nodes.insert(n);
            trace!("enable moves of {:?}", nodes);
            self.enable_moves(nodes);
            
            vec_utils::remove_value(&mut self.worklist_spill, n);
            
            if self.is_move_related(n) {
                trace!("{} is move related, push to freeze list", self.node_info(n));
                self.worklist_freeze.insert(n);
            } else {
                trace!("{} is not move related, push to simplify list", self.node_info(n));
                self.worklist_simplify.insert(n);
            }
        }
    }
    
    fn enable_moves(&mut self, nodes: HashSet<Node>) {
        for n in nodes {
            for mov in self.node_moves(n) {
                if self.active_moves.contains(&mov) {
                    self.active_moves.insert(mov);
                    self.worklist_moves.push(mov);
                }
            }
        }
    }
    
    fn coalesce(&mut self) {
        let m = self.worklist_moves.pop().unwrap();
        
        trace!("Coalescing on {}", self.move_info(m));
        
        let x = self.get_alias(m.from);
        let y = self.get_alias(m.to);
        trace!("resolve alias: from {} to {}", self.node_info(x), self.node_info(y));
        
        let (u, v, precolored_u, precolored_v) = {
            if self.precolored.contains(&y) {
                let u = y;
                let v = x;
                let precolored_u = true;
                let precolored_v = self.precolored.contains(&v);
                
                (u, v, precolored_u, precolored_v)
            } else {
                let u = x;
                let v = y;
                let precolored_u = self.precolored.contains(&u);
                let precolored_v = self.precolored.contains(&v);
                
                (u, v, precolored_u, precolored_v)
            }
        };
        trace!("u={}, v={}, precolored_u={}, precolroed_v={}", 
            self.node_info(u),
            self.node_info(v),
            precolored_u, precolored_v);
        
        if u == v {
            trace!("u == v, coalesce the move");
            self.coalesced_moves.insert(m);
            if !precolored_u {
                self.add_worklist(u);
            }
        } else if precolored_v || self.ig.is_adj(u, v) {
            trace!("v is precolored or u,v is adjacent, the move is constrained");
            self.constrained_moves.insert(m);
            if !precolored_u {
                self.add_worklist(u);
            }
            if !precolored_v {
                self.add_worklist(v);
            }
        } else if (precolored_u && self.ok(u, v)) 
          || (!precolored_u && self.conservative(u, v)) {
            trace!("precolored_u&&ok(u,v) || !precolored_u&&conserv(u,v), coalesce and combine the move");  
            self.coalesced_moves.insert(m);
            self.combine(u, v);
            if !precolored_u {
                self.add_worklist(u);
            }
        } else {
            trace!("cannot coalesce the move");
            self.active_moves.insert(m);
        }
    }
    
    fn get_alias(&self, node: Node) -> Node {
        if self.coalesced_nodes.contains(&node) {
            self.get_alias(*self.alias.get(&node).unwrap())
        } else {
            node
        }
    }
    
    fn add_worklist(&mut self, node: Node) {
        if !self.is_move_related(node) && self.degree(node) < self.n_regs_for_node(node) as isize {
            self.worklist_freeze.remove(&node);
            self.worklist_simplify.insert(node);
        }
    }
    
    fn ok(&self, u: Node, v: Node) -> bool {
        for t in self.adjacent(v) {
            if !self.precolored.contains(&t) 
              || self.degree(t) < self.n_regs_for_node(t) as isize
              || self.ig.is_adj(t, u) {
                return false;
            } 
        }
        
        true
    }
    
    fn conservative(&self, u: Node, v: Node) -> bool {
        debug_assert!(self.ig.get_group_of(u) == self.ig.get_group_of(v));
        
        let adj_u = self.adjacent(u);
        let adj_v = self.adjacent(v);
        let nodes = adj_u.union(&adj_v).collect::<HashSet<_>>();
        
        let mut k = 0;
        for n in nodes {
            if self.precolored.contains(n) || self.degree(*n) >= self.n_regs_for_node(*n) as isize {
                k += 1;
            }
        }
        
        k < self.n_regs_for_node(u)
    }
    
    fn combine(&mut self, u: Node, v: Node) {
        if self.worklist_freeze.contains(&v) {
            self.worklist_freeze.remove(&v);
            self.coalesced_nodes.insert(v);
        } else {
            vec_utils::remove_value(&mut self.worklist_spill, v);
            self.coalesced_nodes.insert(v);
        }
        
        self.alias.insert(v, u); 
        
        {
            let ref mut movelist = self.movelist;
            GraphColoring::movelist_check(movelist, u);
            GraphColoring::movelist_check(movelist, v);
            // we checked before getting the movelist, its safe
            // use nocheck version which requires only immutable references of movelist
            // avoid maintaining a mutable reference of movelist alive
            let movelist_u = &mut unsafe {GraphColoring::movelist_nocheck(movelist, u)}.borrow_mut();
            let movelist_v = &mut unsafe {GraphColoring::movelist_nocheck(movelist, v)}.borrow_mut();
            
            // addAll()
            movelist_u.extend_from_slice(movelist_v.as_slice());
        }
        
        let mut nodes = HashSet::new();
        nodes.insert(v);
        self.enable_moves(nodes);
        
        for t in self.adjacent(v) {
            self.add_edge(t, u);
            self.decrement_degree(t);
        }
        
        if self.worklist_freeze.contains(&u)
          && self.degree(u) >= self.n_regs_for_node(u) as isize {
            self.worklist_freeze.remove(&u);
            self.worklist_spill.push(u);
        }
    }
    
    fn add_edge(&mut self, u: Node, v: Node) {
        if u != v && !self.ig.is_adj(u, v) {
            if !self.precolored.contains(&u) {
                self.ig.add_interference_edge(u, v);
                let degree_u = self.degree(u);
                self.degree.insert(u, degree_u + 1);
            }
            if !self.precolored.contains(&v) {
                self.ig.add_interference_edge(v, u);
                let degree_v = self.degree(v);
                self.degree.insert(v, degree_v + 1);
            }
        }
    }
    
    fn freeze(&mut self) {
        // it is not empty (checked before)
        let node = hashset_utils::pop_first(&mut self.worklist_freeze).unwrap();
        trace!("Freezing {}...", self.node_info(node));
        
        self.worklist_simplify.insert(node);
        self.freeze_moves(node);
    }
    
    fn freeze_moves(&mut self, u: Node) {
        for m in self.node_moves(u) {
            let mut v = self.get_alias(m.from);
            if v == self.get_alias(u) {
                v = self.get_alias(m.to);
            }
            
            self.active_moves.remove(&m);
            self.frozen_moves.insert(m);
            
            if !self.precolored.contains(&v) 
               && self.node_moves(v).is_empty()
               && self.degree(v) < self.n_regs_for_node(v) as isize {
                self.worklist_freeze.remove(&v);
                self.worklist_simplify.insert(v);
            }
        }
    }
    
    fn select_spill(&mut self) {
        trace!("Selecting a node to spill...");
        let mut m : Option<Node> = None;
        
        for n in self.worklist_spill.iter() {
            let n = *n;
            if m.is_none() {
                m = Some(n);
            } else if {
                // m is not none
                let temp = self.ig.get_temp_of(m.unwrap());
                let spillable = {match self.spillable.get(&temp) {
                    None => {
                        //by default, its spillable
                        true
                    },
                    Some(b) => *b
                }};
                
                !spillable
            } {
                m = Some(n);
            } else if (self.ig.get_spill_cost(n) / (self.degree(n) as f32)) 
              < (self.ig.get_spill_cost(m.unwrap()) / (self.degree(m.unwrap()) as f32)) {
                m = Some(n);
            }
        }
        
        // m is not none
        let m = m.unwrap();
        trace!("Spilling {}...", self.node_info(m));
        
        vec_utils::remove_value(&mut self.worklist_spill, m);
        self.worklist_simplify.insert(m);
        self.freeze_moves(m);
    }
    
    fn assign_colors(&mut self) {
        trace!("---coloring done---");
        while !self.select_stack.is_empty() {
            let n = self.select_stack.pop().unwrap();
            trace!("Assigning color to {}", self.node_info(n));
            
            let mut ok_colors = self.colors.get(&self.ig.get_group_of(n)).unwrap().clone();
            for w in self.ig.outedges_of(n) {
                let w = self.get_alias(w);
                match self.ig.get_color_of(w) {
                    None => {}, // do nothing
                    Some(color) => {ok_colors.remove(&color);}
                }
            }
            trace!("available colors: {:?}", ok_colors);
            
            if ok_colors.is_empty() {
                trace!("{} is a spilled node", self.node_info(n));
                self.spilled_nodes.push(n);
            } else {
                let first_available_color = hashset_utils::pop_first(&mut ok_colors).unwrap();
                trace!("Color {} as {}", self.node_info(n), first_available_color);
                self.colored_nodes.push(n);
                self.ig.color_node(n, first_available_color);
            }
        }
        
        for n in self.coalesced_nodes.iter() {
            let n = *n;
            let alias = self.get_alias(n);
            let alias_color = self.ig.get_color_of(alias).unwrap();
            
            trace!("Assign color to {} based on aliased {}", self.node_info(n), self.node_info(alias));
            trace!("Color {} as {}", self.node_info(n), alias_color);
            self.ig.color_node(n, alias_color);
        }
    }
    
    pub fn spills(&self) -> Vec<MuID> {
        let mut spills = vec![];
        
        let spill_count = self.spilled_nodes.len();
        if spill_count > 0 {
            for n in self.spilled_nodes.iter() {
                spills.push(self.ig.get_temp_of(*n));
            }
        }
        
        spills
    }
    
    fn node_info(&self, node: Node) -> String {
        let reg = self.ig.get_temp_of(node);
        format!("{:?}/Reg {}", node, reg)
    }
    
    fn move_info(&self, m: Move) -> String {
        format!("Move: {} -> {}", self.node_info(m.from), self.node_info(m.to))
    }
}