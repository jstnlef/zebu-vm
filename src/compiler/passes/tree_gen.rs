use ast::ir::*;
use vm::context::VMContext;

use compiler::CompilerPass;

pub struct TreeGenerationPass {
    name: &'static str,
} 

impl TreeGenerationPass {
    pub fn new(name: &'static str) -> TreeGenerationPass {
        TreeGenerationPass{name: name}
    }
}

impl CompilerPass for TreeGenerationPass {
    fn name(&self) -> &'static str {
        self.name
    }
}