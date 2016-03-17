use ast::ir::*;
use vm::context::VMContext;

use compiler::CompilerPass;

pub struct DefUsePass {
    name: &'static str,
}

impl DefUsePass {
    pub fn name(name: &'static str) -> DefUsePass {
        DefUsePass{name: name}
    }
}

impl CompilerPass for DefUsePass {
    fn name(&self) -> &'static str {
        self.name
    }
    
    fn visit_node(&mut self, vm_context: &VMContext, node: &mut TreeNode) {
        
    }
}