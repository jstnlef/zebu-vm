use ast::ir::*;
use vm::context::VMContext;

use compiler::CompilerPass;

pub struct TreeGenerationPass;

impl TreeGenerationPass {
    pub fn new() -> TreeGenerationPass {
        TreeGenerationPass
    }
}

impl CompilerPass for TreeGenerationPass {
    fn execute(&mut self, vm: &VMContext, func: &mut MuFunction) {
        debug!("Generating Tree for {:?}", func.fn_name);
        
        for entry in func.blocks.iter_mut() {
            let label : MuTag = entry.0;
            let ref mut block : &mut Block = &mut entry.1;
            
            debug!("  block: {:?}", label);

            for inst in block.content.take().unwrap().body {
                
            }
        }
    }
}