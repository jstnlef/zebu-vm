use ast::ir::*;
use ast::ptr::*;
use ast::inst::Instruction_::*;
use vm::context::VMContext;

use compiler::CompilerPass;

pub struct InstructionSelection {
    name: &'static str
}

impl InstructionSelection {
    pub fn new() -> InstructionSelection {
        InstructionSelection{name: "Instruction Selection (x64)"}
    }
}

impl CompilerPass for InstructionSelection {
    fn name(&self) -> &'static str {
        self.name
    }
    
    #[allow(unused_variables)]
    fn start_function(&mut self, vm_context: &VMContext, func: &mut MuFunction) {
        debug!("{}", self.name());
    }
    
    fn visit_function(&mut self, vm_context: &VMContext, func: &mut MuFunction) {
        for block_label in func.block_trace.as_ref().unwrap() {
            let block = func.content.as_mut().unwrap().get_block_mut(block_label);
            
            let block_content = block.content.as_mut().unwrap();
            
            for inst in block_content.body.iter_mut() {
                instruction_select(inst);
            }
        }
    }
}

fn instruction_select(inst: &mut P<TreeNode>) {
    match inst.v {
        TreeNode_::Instruction(ref inst) => {
            match inst.v {
                Branch2{cond, ref true_dest, ref false_dest, true_prob} => {
                    let ref cond = inst.ops.borrow()[cond];
                    
                    match cond.v {
                        TreeNode_::Instruction(ref inst) => {
                            match inst.v {
                                CmpOp(op, op1, op2) => {
                                    
                                }, 
                                
                                _ => panic!("unexpected inst as child node of branch2: {}", cond)
                            }
                        },
                        
                        TreeNode_::Value(ref pv) => {
                            
                        }
                    }
                },
                
                _ => unimplemented!()
            }
        },
        _ => panic!("expected instruction")
    }
} 