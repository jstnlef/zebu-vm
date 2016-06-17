use compiler::backend::reg_alloc::liveness::InterferenceGraph; 
use vm::machine_code::CompiledFunction;

pub struct GraphColoring <'a> {
    ig: InterferenceGraph,
    cur_cf: &'a CompiledFunction,
}

impl <'a> GraphColoring <'a> {
    pub fn start (cf: &CompiledFunction, ig: InterferenceGraph) {
        let mut coloring = GraphColoring {
            ig: ig,
            cur_cf: cf, 
        };
        
        coloring.init();
    }
    
    fn init (&mut self) {
        
    }
}