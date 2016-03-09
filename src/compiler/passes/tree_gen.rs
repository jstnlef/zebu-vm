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
        
    }
}