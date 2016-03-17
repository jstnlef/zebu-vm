use ast::ir::*;
use vm::context::VMContext;

use compiler::CompilerPass;

pub struct DefUsePass {
    name: &'static str,
}

impl DefUsePass {
    pub fn new(name: &'static str) -> DefUsePass {
        DefUsePass{name: name}
    }
}

impl CompilerPass for DefUsePass {
    fn name(&self) -> &'static str {
        self.name
    }
    
    fn visit_inst(&mut self, vm_context: &VMContext, node: &mut TreeNode) {
        match node.v {
            TreeNode_::Instruction(ref inst) => {
                for p_node in inst.list_operands() {
                    p_node.use_count.set(p_node.use_count.get() + 1)
                }
            },
            TreeNode_::Value(_) => panic!("expected instruction node")
        }
    }
}