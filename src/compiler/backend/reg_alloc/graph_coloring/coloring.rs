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

use ast::ptr::*;
use ast::ir::*;
use compiler::backend;
use compiler::backend::reg_alloc::graph_coloring;
use compiler::backend::reg_alloc::graph_coloring::liveness::InterferenceGraph;
use compiler::machine_code::CompiledFunction;
use vm::VM;

use utils::vec_utils;
use utils::LinkedHashSet;
use utils::LinkedHashMap;

use std::cell::RefCell;

use compiler::backend::reg_alloc::graph_coloring::liveness::Move;
use compiler::backend::reg_alloc::graph_coloring::petgraph::graph::NodeIndex;

const COALESCING: bool = true;
const MAX_REWRITE_ITERATIONS_ALLOWED: usize = 10;

/// GraphColoring algorithm
/// based on Appel's book section 11.4
pub struct GraphColoring<'a> {
    // context
    pub func: &'a mut MuFunctionVersion,
    pub cf: &'a mut CompiledFunction,
    pub vm: &'a VM,
    pub ig: InterferenceGraph,

    /// how many coloring iteration have we done?
    /// In case that a bug may trigger the coloring iterate endlessly, we use this count to stop
    iteration_count: usize,

    /// machine registers, preassigned a color
    precolored: LinkedHashSet<NodeIndex>,
    /// all colors available
    colors: LinkedHashMap<backend::RegGroup, LinkedHashSet<MuID>>,
    /// temporaries, not precolored and not yet processed
    initial: Vec<NodeIndex>,

    /// list of low-degree non-move-related nodes
    worklist_simplify: LinkedHashSet<NodeIndex>,
    /// low-degree move related nodes
    worklist_freeze: LinkedHashSet<NodeIndex>,
    /// nodes marked for spilling during this round
    worklist_spill: Vec<NodeIndex>,
    /// nodes marked for spilling during this round
    spilled_nodes: Vec<NodeIndex>,
    /// temps that have been coalesced
    /// when u <- v is coalesced, v is added to this set and u put back on some work list
    coalesced_nodes: LinkedHashSet<NodeIndex>,
    /// nodes successfully colored
    colored_nodes: Vec<NodeIndex>,
    /// stack containing temporaries removed from the graph
    select_stack: Vec<NodeIndex>,

    /// moves that have been coalesced
    coalesced_moves: LinkedHashSet<Move>,
    /// moves whose source and target interfere
    constrained_moves: LinkedHashSet<Move>,
    /// moves that will no longer be considered for coalescing
    frozen_moves: LinkedHashSet<Move>,
    /// moves enabled for possible coalescing
    worklist_moves: Vec<Move>,
    /// moves not yet ready for coalescing
    active_moves: LinkedHashSet<Move>,

    /// degree of nodes
    degree: LinkedHashMap<NodeIndex, usize>,
    /// a mapping from a node to the list of moves it is associated with
    movelist: LinkedHashMap<NodeIndex, RefCell<Vec<Move>>>,
    /// when a move (u, v) has been coalesced, and v put in coalescedNodes, then alias(v) = u
    alias: LinkedHashMap<NodeIndex, NodeIndex>,

    // for validation use
    /// we need to log all registers get spilled with their spill location
    spill_history: LinkedHashMap<MuID, P<Value>>,
    /// we need to know the mapping between scratch temp -> original temp
    spill_scratch_temps: LinkedHashMap<MuID, MuID>
}

impl<'a> GraphColoring<'a> {
    /// starts coloring
    pub fn start(
        func: &'a mut MuFunctionVersion,
        cf: &'a mut CompiledFunction,
        vm: &'a VM
    ) -> GraphColoring<'a> {
        GraphColoring::start_with_spill_history(
            LinkedHashMap::new(),
            LinkedHashMap::new(),
            0,
            func,
            cf,
            vm
        )
    }

    /// restarts coloring with spill history
    fn start_with_spill_history(
        spill_history: LinkedHashMap<MuID, P<Value>>,
        spill_scratch_temps: LinkedHashMap<MuID, MuID>,
        iteration_count: usize,
        func: &'a mut MuFunctionVersion,
        cf: &'a mut CompiledFunction,
        vm: &'a VM
    ) -> GraphColoring<'a> {
        assert!(
            iteration_count < MAX_REWRITE_ITERATIONS_ALLOWED,
            "reach graph coloring max rewrite iterations ({}), probably something is going wrong",
            MAX_REWRITE_ITERATIONS_ALLOWED
        );
        let iteration_count = iteration_count + 1;

        trace!("Initializing coloring allocator...");
        cf.mc().trace_mc();

        let ig = graph_coloring::build_inteference_graph(cf, func);

        let coloring = GraphColoring {
            func: func,
            cf: cf,
            vm: vm,
            ig: ig,
            iteration_count: iteration_count,
            precolored: LinkedHashSet::new(),
            colors: {
                let mut map = LinkedHashMap::new();
                map.insert(backend::RegGroup::GPR, LinkedHashSet::new());
                map.insert(backend::RegGroup::FPR, LinkedHashSet::new());
                map
            },
            colored_nodes: Vec::new(),
            initial: Vec::new(),
            degree: LinkedHashMap::new(),
            worklist_moves: Vec::new(),
            movelist: LinkedHashMap::new(),
            active_moves: LinkedHashSet::new(),
            coalesced_nodes: LinkedHashSet::new(),
            coalesced_moves: LinkedHashSet::new(),
            constrained_moves: LinkedHashSet::new(),
            alias: LinkedHashMap::new(),
            worklist_spill: Vec::new(),
            spilled_nodes: Vec::new(),
            spill_history: spill_history,
            spill_scratch_temps: spill_scratch_temps,
            worklist_freeze: LinkedHashSet::new(),
            frozen_moves: LinkedHashSet::new(),
            worklist_simplify: LinkedHashSet::new(),
            select_stack: Vec::new()
        };

        coloring.regalloc()
    }

    /// returns formatted string for a node
    fn display_node(&self, node: NodeIndex) -> String {
        let id = self.ig.get_temp_of(node);
        self.display_id(id)
    }

    /// returns formatted string for an ID
    fn display_id(&self, id: MuID) -> String {
        self.func.context.get_temp_display(id)
    }

    /// returns formatted string for a move
    fn display_move(&self, m: Move) -> String {
        format!(
            "Move: {} -> {}",
            self.display_node(m.from),
            self.display_node(m.to)
        )
    }

    /// does coloring register allocation
    fn regalloc(mut self) -> GraphColoring<'a> {
        trace!("---InterenceGraph---");
        self.ig.print(&self.func.context);

        // start timing for graph coloring
        let _p = hprof::enter("regalloc: graph coloring");

        // precolor for all machine registers
        for reg in backend::all_regs().values() {
            let reg_id = reg.extract_ssa_id().unwrap();
            let node = self.ig.get_node(reg_id);
            self.precolored.insert(node);
        }

        // put usable registers as available colors
        for reg in backend::all_usable_regs().iter() {
            let reg_id = reg.extract_ssa_id().unwrap();
            let group = backend::pick_group_for_reg(reg_id);
            self.colors.get_mut(&group).unwrap().insert(reg_id);
        }

        // push uncolored nodes to initial work set
        for node in self.ig.nodes() {
            if !self.ig.is_colored(node) {
                self.initial.push(node);
                let degree = self.ig.get_degree_of(node);
                self.degree.insert(node, degree);
                trace!("{} has a degree of {}", self.display_node(node), degree);
            }
        }

        // initialize work
        self.build();
        self.make_work_list();

        // main loop
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

            !(self.worklist_simplify.is_empty() && self.worklist_moves.is_empty() &&
                  self.worklist_freeze.is_empty() && self.worklist_spill.is_empty())
        } {}

        // pick color for nodes
        self.assign_colors();

        // finish
        drop(_p);

        // if we need to spill
        if !self.spilled_nodes.is_empty() {
            trace!("spill required");
            if cfg!(debug_assertions) {
                trace!("nodes to be spilled:");
                for node in self.spilled_nodes.iter() {
                    trace!("{}", self.display_node(*node));
                }
            }

            // rewrite program to insert spilling code
            self.rewrite_program();

            // recursively redo graph coloring
            return GraphColoring::start_with_spill_history(
                self.spill_history.clone(),
                self.spill_scratch_temps.clone(),
                self.iteration_count,
                self.func,
                self.cf,
                self.vm
            );
        }

        self
    }

    fn build(&mut self) {
        if COALESCING {
            trace!("coalescing enabled, build move list");
            let ref ig = self.ig;
            let ref mut movelist = self.movelist;
            for m in ig.moves().iter() {
                trace!("add to movelist: {:?}", m);
                self.worklist_moves.push(m.clone());
                GraphColoring::movelist_mut(movelist, m.from)
                    .borrow_mut()
                    .push(m.clone());
                GraphColoring::movelist_mut(movelist, m.to)
                    .borrow_mut()
                    .push(m.clone());
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
                let degree = self.ig.get_degree_of(node);
                let n_regs = self.n_regs_for_node(node);

                degree >= n_regs
            } {
                trace!(
                    "{} 's degree >= reg number limit (K), push to spill list",
                    self.display_node(node)
                );
                self.worklist_spill.push(node);
            } else if self.is_move_related(node) {
                trace!(
                    "{} is move related, push to freeze list",
                    self.display_node(node)
                );
                self.worklist_freeze.insert(node);
            } else {
                trace!(
                    "{} has small degree and not move related, push to simplify list",
                    self.display_node(node)
                );
                self.worklist_simplify.insert(node);
            }
        }
    }

    fn n_regs_for_node(&self, node: NodeIndex) -> usize {
        backend::number_of_usable_regs_in_group(self.ig.get_group_of(node))
    }

    fn is_move_related(&mut self, node: NodeIndex) -> bool {
        !self.node_moves(node).is_empty()
    }

    fn is_spillable(&self, temp: MuID) -> bool {
        // if a temporary is created as scratch temp for a spilled temporary, we
        // should not spill it again (infinite loop otherwise)
        if self.spill_scratch_temps.contains_key(&temp) {
            false
        } else {
            true
        }
    }

    fn node_moves(&mut self, node: NodeIndex) -> LinkedHashSet<Move> {
        let mut moves = LinkedHashSet::new();

        // addAll(active_moves)
        for m in self.active_moves.iter() {
            moves.insert(m.clone());
        }

        // addAll(worklist_moves)
        for m in self.worklist_moves.iter() {
            moves.insert(m.clone());
        }

        let mut retained = LinkedHashSet::new();
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
    fn movelist_mut(
        list: &mut LinkedHashMap<NodeIndex, RefCell<Vec<Move>>>,
        node: NodeIndex
    ) -> &RefCell<Vec<Move>> {
        GraphColoring::movelist_check(list, node);
        unsafe { GraphColoring::movelist_nocheck(list, node) }
    }

    fn movelist_check(list: &mut LinkedHashMap<NodeIndex, RefCell<Vec<Move>>>, node: NodeIndex) {
        if !list.contains_key(&node) {
            list.insert(node, RefCell::new(Vec::new()));
        }
    }

    // allows getting the Vec<Move> without a mutable reference of the hashmap
    unsafe fn movelist_nocheck(
        list: &LinkedHashMap<NodeIndex, RefCell<Vec<Move>>>,
        node: NodeIndex
    ) -> &RefCell<Vec<Move>> {
        list.get(&node).unwrap()
    }

    fn simplify(&mut self) {
        // remove next element from worklist_simplify, we know its not empty
        let node = self.worklist_simplify.pop_front().unwrap();

        trace!("Simplifying {}", self.display_node(node));

        self.select_stack.push(node);
        for m in self.adjacent(node).iter() {
            self.decrement_degree(*m);
        }
    }

    fn adjacent(&self, n: NodeIndex) -> LinkedHashSet<NodeIndex> {
        let mut adj = LinkedHashSet::new();

        // add n's successors
        for s in self.ig.get_edges_of(n) {
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

    fn degree(&self, n: NodeIndex) -> usize {
        match self.degree.get(&n) {
            Some(d) => *d,
            None => 0
        }
    }

    fn decrement_degree(&mut self, n: NodeIndex) {
        if self.precolored.contains(&n) {
            return;
        }

        trace!("decrement degree of {}", self.display_node(n));

        let d = self.degree(n);
        debug_assert!(d != 0);
        self.degree.insert(n, d - 1);

        if d == self.n_regs_for_node(n) {
            trace!(
                "{}'s degree is K, no longer need to spill it",
                self.display_node(n)
            );
            let mut nodes = self.adjacent(n);
            nodes.insert(n);
            trace!("enable moves of {:?}", nodes);
            self.enable_moves(nodes);

            vec_utils::remove_value(&mut self.worklist_spill, n);

            if self.is_move_related(n) {
                trace!(
                    "{} is move related, push to freeze list",
                    self.display_node(n)
                );
                self.worklist_freeze.insert(n);
            } else {
                trace!(
                    "{} is not move related, push to simplify list",
                    self.display_node(n)
                );
                self.worklist_simplify.insert(n);
            }
        }
    }

    fn enable_moves(&mut self, nodes: LinkedHashSet<NodeIndex>) {
        for n in nodes.iter() {
            let n = *n;
            for mov in self.node_moves(n).iter() {
                let mov = *mov;
                if self.active_moves.contains(&mov) {
                    self.active_moves.insert(mov);
                    self.worklist_moves.push(mov);
                }
            }
        }
    }

    fn coalesce(&mut self) {
        let m = self.worklist_moves.pop().unwrap();

        trace!("Coalescing on {}", self.display_move(m));

        // if they are not from the same register group, we cannot coalesce them
        if self.ig.get_group_of(m.from) != self.ig.get_group_of(m.to) {
            info!("a move instruction of two temporaries of different register groups");
            info!("from: {:?}, to: {:?}", m.from, m.to);

            return;
        }

        let x = self.get_alias(m.from);
        let y = self.get_alias(m.to);
        trace!(
            "resolve alias: from {} to {}",
            self.display_node(x),
            self.display_node(y)
        );

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
        trace!(
            "u={}, v={}, precolored_u={}, precolroed_v={}",
            self.display_node(u),
            self.display_node(v),
            precolored_u,
            precolored_v
        );

        // if u or v is a machine register that is not usable/not a color, we cannot coalesce
        if self.precolored.contains(&u) {
            let reg_u = self.ig.get_temp_of(u);
            let group = backend::pick_group_for_reg(reg_u);
            if !self.colors.get(&group).unwrap().contains(&reg_u) {
                return;
            }
        }
        if self.precolored.contains(&v) {
            let reg_v = self.ig.get_temp_of(v);
            let group = backend::pick_group_for_reg(reg_v);
            if !self.colors.get(&group).unwrap().contains(&reg_v) {
                return;
            }
        }

        if u == v {
            trace!("u == v, coalesce the move");
            self.coalesced_moves.insert(m);
            if !precolored_u {
                self.add_worklist(u);
            }
        } else if precolored_v || self.ig.is_adj(u, v) {
            trace!("precolored_v: {}", precolored_v);
            trace!("is_adj(u, v): {}", self.ig.is_adj(u, v));
            trace!("v is precolored or u,v is adjacent, the move is constrained");
            self.constrained_moves.insert(m);
            if !precolored_u {
                self.add_worklist(u);
            }
            if !precolored_v {
                self.add_worklist(v);
            }
        } else if (precolored_u && self.ok(u, v)) || (!precolored_u && self.conservative(u, v)) {
            trace!("ok(u, v) = {}", self.ok(u, v));
            trace!("conservative(u, v) = {}", self.conservative(u, v));

            trace!(
                "precolored_u&&ok(u,v) || !precolored_u&&conserv(u,v), \
                 coalesce and combine the move"
            );
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

    pub fn get_alias(&self, node: NodeIndex) -> NodeIndex {
        if self.coalesced_nodes.contains(&node) {
            self.get_alias(*self.alias.get(&node).unwrap())
        } else {
            node
        }
    }

    fn add_worklist(&mut self, node: NodeIndex) {
        if !self.is_move_related(node) && self.degree(node) < self.n_regs_for_node(node) {
            self.worklist_freeze.remove(&node);
            self.worklist_simplify.insert(node);
        }
    }

    fn ok(&self, u: NodeIndex, v: NodeIndex) -> bool {
        for t in self.adjacent(v).iter() {
            let t = *t;
            if !(self.degree(t) < self.n_regs_for_node(t) || self.precolored.contains(&t) ||
                     self.ig.is_adj(t, u))
            {
                return false;
            }
        }

        true
    }

    fn conservative(&self, u: NodeIndex, v: NodeIndex) -> bool {
        let adj_u = self.adjacent(u);
        let adj_v = self.adjacent(v);
        let nodes = {
            let mut ret = adj_u;
            ret.add_all(adj_v);
            ret
        };

        let mut k = 0;
        for n in nodes.iter() {
            if self.precolored.contains(n) || self.degree(*n) >= self.n_regs_for_node(*n) {
                k += 1;
            }
        }

        k < self.n_regs_for_node(u) && k < self.n_regs_for_node(v)
    }

    fn combine(&mut self, u: NodeIndex, v: NodeIndex) {
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
            let movelist_u =
                &mut unsafe { GraphColoring::movelist_nocheck(movelist, u) }.borrow_mut();
            let movelist_v =
                &mut unsafe { GraphColoring::movelist_nocheck(movelist, v) }.borrow_mut();

            // addAll()
            movelist_u.extend_from_slice(movelist_v.as_slice());
        }

        let mut nodes = LinkedHashSet::new();
        nodes.insert(v);
        self.enable_moves(nodes);

        for t in self.adjacent(v).iter() {
            let t = *t;
            self.add_edge(t, u);
            self.decrement_degree(t);
        }

        if self.worklist_freeze.contains(&u) && self.degree(u) >= self.n_regs_for_node(u) {
            self.worklist_freeze.remove(&u);
            self.worklist_spill.push(u);
        }
    }

    fn add_edge(&mut self, u: NodeIndex, v: NodeIndex) {
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
        let node = self.worklist_freeze.pop_front().unwrap();
        trace!("Freezing {}...", self.display_node(node));

        self.worklist_simplify.insert(node);
        self.freeze_moves(node);
    }

    fn freeze_moves(&mut self, u: NodeIndex) {
        for m in self.node_moves(u).iter() {
            let m = *m;
            let mut v = self.get_alias(m.from);
            if v == self.get_alias(u) {
                v = self.get_alias(m.to);
            }

            self.active_moves.remove(&m);
            self.frozen_moves.insert(m);

            if !self.precolored.contains(&v) && self.node_moves(v).is_empty() &&
                self.degree(v) < self.n_regs_for_node(v)
            {
                self.worklist_freeze.remove(&v);
                self.worklist_simplify.insert(v);
            }
        }
    }

    fn select_spill(&mut self) {
        trace!("Selecting a node to spill...");
        let mut m: Option<NodeIndex> = None;

        for n in self.worklist_spill.iter() {
            let n = *n;
            if m.is_none() {
                m = Some(n);
            } else if {
                       // m is not none
                       let temp = self.ig.get_temp_of(m.unwrap());
                       !self.is_spillable(temp)
                   } {
                m = Some(n);
            } else if (self.ig.get_spill_cost(n) / (self.degree(n) as f32)) <
                       (self.ig.get_spill_cost(m.unwrap()) / (self.degree(m.unwrap()) as f32))
            {
                m = Some(n);
            }
        }

        // m is not none
        let m = m.unwrap();
        trace!("Spilling {}...", self.display_node(m));

        vec_utils::remove_value(&mut self.worklist_spill, m);
        self.worklist_simplify.insert(m);
        self.freeze_moves(m);
    }

    fn assign_colors(&mut self) {
        trace!("---coloring done---");
        while !self.select_stack.is_empty() {
            let n = self.select_stack.pop().unwrap();
            trace!("Assigning color to {}", self.display_node(n));

            let mut ok_colors: LinkedHashSet<MuID> =
                self.colors.get(&self.ig.get_group_of(n)).unwrap().clone();

            trace!("all the colors for this temp: {:?}", ok_colors);

            for w in self.ig.get_edges_of(n) {
                let w_alias = self.get_alias(w);
                match self.ig.get_color_of(w_alias) {
                    None => {} // do nothing
                    Some(color) => {
                        trace!(
                            "color {} is used for its neighbor {:?} (aliasing to {:?})",
                            color,
                            self.display_node(w),
                            self.display_node(w_alias)
                        );
                        ok_colors.remove(&color);
                    }
                }
            }
            trace!("available colors: {:?}", ok_colors);

            if ok_colors.is_empty() {
                trace!("{} is a spilled node", self.display_node(n));
                self.spilled_nodes.push(n);
            } else {
                let first_available_color = ok_colors.pop_front().unwrap();
                trace!(
                    "Color {} as {}",
                    self.display_node(n),
                    first_available_color
                );

                if !backend::is_callee_saved(first_available_color) {
                    trace!("Use caller saved register {}", first_available_color);
                }

                self.colored_nodes.push(n);
                self.ig.color_node(n, first_available_color);
            }
        }

        for n in self.colored_nodes.iter() {
            let n = *n;
            let alias = self.get_alias(n);
            let alias_color = self.ig.get_color_of(alias).unwrap();

            trace!(
                "Assign color to {} based on aliased {}",
                self.display_node(n),
                self.display_node(alias)
            );
            trace!("Color {} as {}", self.display_node(n), alias_color);
            self.ig.color_node(n, alias_color);
        }
    }

    fn rewrite_program(&mut self) {
        let spills = self.spills();

        let mut spilled_mem = LinkedHashMap::new();

        // allocating frame slots for every spilled temp
        for reg_id in spills.iter() {
            let ssa_entry = match self.func.context.get_value(*reg_id) {
                Some(entry) => entry,
                None => panic!("The spilled register {} is not in func context", reg_id)
            };
            let mem = self.cf
                .frame
                .alloc_slot_for_spilling(ssa_entry.value().clone(), self.vm);

            spilled_mem.insert(*reg_id, mem.clone());
            self.spill_history.insert(*reg_id, mem);
        }

        let scratch_temps = backend::spill_rewrite(&spilled_mem, self.func, self.cf, self.vm);
        for (k, v) in scratch_temps {
            self.spill_scratch_temps.insert(k, v);
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

    pub fn get_assignments(&self) -> LinkedHashMap<MuID, MuID> {
        let mut ret = LinkedHashMap::new();

        for node in self.ig.nodes() {
            let temp = self.ig.get_temp_of(node);

            if temp < MACHINE_ID_END {
                continue;
            } else {
                let alias = self.get_alias(node);
                let machine_reg = match self.ig.get_color_of(alias) {
                    Some(reg) => reg,
                    None => {
                        panic!(
                            "Reg{}/{:?} (aliased as Reg{}/{:?}) is not assigned with a color",
                            self.ig.get_temp_of(node),
                            node,
                            self.ig.get_temp_of(alias),
                            alias
                        )
                    }
                };

                ret.insert(temp, machine_reg);
            }
        }

        ret
    }

    pub fn get_spill_scratch_temps(&self) -> LinkedHashMap<MuID, MuID> {
        self.spill_scratch_temps.clone()
    }
}
