use compiler::backend::reg_alloc::liveness::InterferenceGraph; 
use vm::machine_code::CompiledFunction;

use compiler::backend::GPR_COUNT;

pub struct GraphColoring <'a> {
    ig: InterferenceGraph,
    cur_cf: &'a CompiledFunction,
    K: usize
}

impl <'a> GraphColoring <'a> {
    pub fn start (cf: &CompiledFunction, ig: InterferenceGraph) {
        let mut coloring = GraphColoring {
            ig: ig,
            cur_cf: cf, 
            K: 0
        };
        
        coloring.init();
    }
    
    fn init (&mut self) {
        self.K = GPR_COUNT;
    }
}