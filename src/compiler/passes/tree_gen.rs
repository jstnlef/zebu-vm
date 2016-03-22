use ast::ir::*;
use vm::context::VMContext;

use compiler::CompilerPass;

pub struct TreeGen {
    name: &'static str,
} 

impl TreeGen {
    pub fn new() -> TreeGen {
        TreeGen{name: "Tree Geenration"}
    }
}

impl CompilerPass for TreeGen {
    fn name(&self) -> &'static str {
        self.name
    }
}