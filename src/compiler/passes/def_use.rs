use ast::ir::*;
use vm::context::VMContext;

use compiler::CompilerPass;

pub struct DefUse {
    name: &'static str,
}

impl DefUse {
    pub fn new() -> DefUse {
        DefUse{name: "Def-Use Pass"}
    }
}

impl CompilerPass for DefUse {
    fn name(&self) -> &'static str {
        self.name
    }
    
    fn visit_inst(&mut self, vm_context: &VMContext, func_context: &mut FunctionContext, node: &mut TreeNode) {
        match node.v {
            TreeNode_::Instruction(ref inst) => {
                for op in inst.list_operands() {
                    match op.v {
                        TreeNode_::Value(ref val) => {
                            match val.v {
                                Value_::SSAVar(ref id) => {
                                    let mut entry = func_context.values.get_mut(id).unwrap();
                                    entry.use_count.set(entry.use_count.get() + 1);
                                },
                                _ => {} // dont worry about constants
                            }
                        },
                        _ => {} // dont worry about instruction
                    }
                }
            },
            _ => panic!("expected instruction node in visit_inst()")
        }
    }
    
    fn finish_function(&mut self, vm_context: &VMContext, func: &mut MuFunction) {
        debug!("check use count for variables");
        
        for entry in func.context.values.values() {
            debug!("{}({}): {}", entry.tag, entry.id, entry.use_count.get())
        }
    }
}