use ast::ir::*;
use ast::ptr::*;
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

fn use_op(op: &P<TreeNode>, func_context: &mut FunctionContext) {
    match op.v {
        TreeNode_::Value(ref val) => {
            match val.v {
                Value_::SSAVar(ref id) => {
                    let entry = func_context.values.get_mut(id).unwrap();
                    entry.use_count.set(entry.use_count.get() + 1);
                },
                _ => {} // dont worry about constants
            }
        },
        _ => {} // dont worry about instruction
    }
}

impl CompilerPass for DefUse {
    fn name(&self) -> &'static str {
        self.name
    }
    
    #[allow(unused_variables)]
    fn visit_block(&mut self, vm_context: &VMContext, func_context: &mut FunctionContext, block: &mut Block) {
        // if an SSA appears in keepalives, its use count increases
        let ref mut keepalives = block.content.as_mut().unwrap().keepalives;
        if keepalives.is_some() {
            for op in keepalives.as_mut().unwrap().iter_mut() {
                use_op(op, func_context);
            }
        }
    }
    
    #[allow(unused_variables)]
    fn visit_inst(&mut self, vm_context: &VMContext, func_context: &mut FunctionContext, node: &mut TreeNode) {
        // if an SSA appears in operands of instrs, its use count increases
        match node.v {
            TreeNode_::Instruction(ref inst) => {
                for op in inst.ops.borrow().iter() {
                    use_op(op, func_context);
                }
            },
            _ => panic!("expected instruction node in visit_inst()")
        }
    }
    
    #[allow(unused_variables)]
    fn finish_function(&mut self, vm_context: &VMContext, func: &mut MuFunction) {
        debug!("check use count for variables");
        
        for entry in func.context.values.values() {
            debug!("{}({}): {}", entry.tag, entry.id, entry.use_count.get())
        }
    }
}