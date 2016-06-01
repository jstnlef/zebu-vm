use vm::machine_code::CompiledFunction;
use vm::machine_code::MachineCode;
use ast::ir::*;

use std::collections::LinkedList;

pub struct InterferenceGraph {
    foo: usize
}

impl InterferenceGraph {
    fn new() -> InterferenceGraph {
        InterferenceGraph {foo: 0}
    }
    
    fn new_node(&mut self, node: MuID) {
        
    }
}

// from tony's code src/RegAlloc/Liveness.java
pub fn build (cf: &CompiledFunction, f: &MuFunction) {
    let mut ig = InterferenceGraph::new();
    
    // FIXME: precolor nodes
    
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
            ig.new_node(*reg_id);
        }
        
        for reg_id in cf.mc.get_inst_reg_uses(i) {
            ig.new_node(*reg_id);
            in_set.push(*reg_id);
        }
        
        work_list.push_front(i);
    }
    
    while !work_list.is_empty() {
        let n = work_list.pop_front().unwrap();
        let ref in_set = live_in[n];
        let ref mut out_set = live_out[n];
    }
} 