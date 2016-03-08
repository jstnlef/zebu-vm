use ast::ir::*;
use compiler::CompilerPass;

pub struct TreeGenerationPass;

impl TreeGenerationPass {
    pub fn new() -> TreeGenerationPass {
        TreeGenerationPass
    }
}

impl CompilerPass for TreeGenerationPass {
    fn execute(&mut self, func: &mut MuFunction) {
        
    }
}