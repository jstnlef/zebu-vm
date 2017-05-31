use ast::ir::*;
use ast::ptr::*;
use vm::VM;

use compiler::CompilerPass;
use std::any::Any;

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
            use_value(val, func_context);
        },
        _ => {} // dont worry about instruction
    }
}

fn use_value(val: &P<Value>, func_context: &mut FunctionContext) {
        match val.v {
            Value_::SSAVar(ref id) => {
                let entry = func_context.values.get_mut(id).unwrap();
                entry.increase_use_count();
            },
            _ => {} // dont worry about constants
        }    
}

impl CompilerPass for DefUse {
    fn name(&self) -> &'static str {
        self.name
    }

    fn as_any(&self) -> &Any {
        self
    }
    
    #[allow(unused_variables)]
    fn start_block(&mut self, vm: &VM, func_context: &mut FunctionContext, block: &mut Block) {
        // if an SSA appears in keepalives, its use count increases
        let ref mut keepalives = block.content.as_mut().unwrap().keepalives;
        if keepalives.is_some() {
            for op in keepalives.as_mut().unwrap().iter_mut() {
                use_value(op, func_context);
            }
        }
    }
    
    #[allow(unused_variables)]
    fn visit_inst(&mut self, vm: &VM, func_context: &mut FunctionContext, node: &TreeNode) {
        // if an SSA appears in operands of instrs, its use count increases
        match node.v {
            TreeNode_::Instruction(ref inst) => {
                for op in inst.ops.iter() {
                    use_op(op, func_context);
                }
            },
            _ => panic!("expected instruction node in visit_inst()")
        }
    }

    #[allow(unused_variables)]
    fn start_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        for entry in func.context.values.values() {
            entry.reset_use_count();
        }
    }
    
    #[allow(unused_variables)]
    fn finish_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        debug!("check use count for variables");
        
        for entry in func.context.values.values() {
            debug!("{}: {}", entry, entry.use_count())
        }
    }
}
