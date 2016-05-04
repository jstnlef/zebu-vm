use ast::ir::*;
use ast::ptr::*;
use ast::inst::Destination;
use ast::inst::DestArg;
use ast::inst::Instruction_::*;
use ast::op;
use ast::op::OpCode;
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

    #[allow(unused_variables)]
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

#[allow(unused_variables)]
fn instruction_select(inst: &mut P<TreeNode>) {
    trace!("instsel on node {}", inst);
    match inst.v {
        TreeNode_::Instruction(ref inst) => {
            match inst.v {
                Branch2{cond, ref true_dest, ref false_dest, true_prob} => {
                    let (fallthrough_dest, branch_dest) = {
                        if true_prob > 0.5f32 {
                            (true_dest, false_dest)
                        } else {
                            (false_dest, true_dest)
                        }
                    };
                    
                    let mut ops = inst.ops.borrow_mut();
                    
                    process_dest(&mut ops, fallthrough_dest);
                    process_dest(&mut ops, branch_dest);

                    let ref mut cond = ops[cond];
                    
                    match cond.op {
                        OpCode::Comparison(op) => {
                            trace!("Tile comp-branch2");                            
                            match cond.v {
                                TreeNode_::Instruction(ref inst) => {
                                    match inst.v {
                                        CmpOp(op, op1, op2) => {
                                            // cmp op1 op2
                                            // jcc branch_dest
                                            // #fallthrough_dest:
                                            // ..
                                        },
        
                                        _ => panic!("expected a comparison op")
                                    }
                                },
                                
                                _ => panic!("expected a comparison inst")
                            }
                        },
                        
                        OpCode::RegI64 | OpCode::IntImmI64 => {
                            trace!("Tile value-branch2");
                            // test/cmp pv 0
                            // jcc branch_dest
                            // #fallthrough_dest:
                            // ...                            
                        },
                        
                        _ => {
                            trace!("nested: compute cond");
                            // instsel for cond first
                            instruction_select(cond);
                            
                            // test/cmp res 0
                            // jcc branch_dest
                            // #fallthrough_dest:
                            // ...
                            trace!("Tile value-branch2 after computing cond")
                        }
                    }
                },
                
                Branch1(ref dest) => {
                    let mut ops = inst.ops.borrow_mut();
                                        
                    process_dest(&mut ops, dest);
                    
                    trace!("Tile branch1");
                    // jmp
                },
                
                ExprCall{ref data, is_abort} => {
                    trace!("Tile exprcall");
                    
                    let mut ops = inst.ops.borrow_mut();
                    for arg_index in data.args.iter() {
                        let ref mut arg = ops[*arg_index];
                        trace!("arg {}", arg);
                        match arg.op {
                            OpCode::RegI64 | OpCode::IntImmI64 => {
                                trace!("Tile move-gpr-arg");
                                // move to register
                            },
                            OpCode::RegFP | OpCode::FPImm => {
                                trace!("Tile move-fpr-arg");
                                // move to fp register
                            },
                            _ => {
                                trace!("nested: compute arg");
                                // instself for arg first
                                instruction_select(arg);
                                
                                // mov based on type
                                trace!("Tile move-arg after computing arg");
                            }
                        }
                    }
                },
                
                Return(ref vals) => {
                    let mut ops = inst.ops.borrow_mut();                    
                    for val_index in vals.iter() {
                        let ref mut val = ops[*val_index];
                        trace!("return val: {}", val);
                        
                        match val.op {
                            OpCode::RegI64 | OpCode::IntImmI64 => {
                                trace!("Tile move-gpr-ret");
                                // move to return register
                            }
                            OpCode::RegFP | OpCode::FPImm => {
                                trace!("Tile move-fpr-ret");
                                // move to return fp register
                            }
                            _ => {
                                trace!("nested: compute return val");
                                // instsel for return val first
                                instruction_select(val);
                                
                                // move based on type
                                trace!("Tile move-ret-val after computing arg");
                            }
                        }
                    }
                },
                
                BinOp(op, op1, op2) => {
                    match op {
                        op::BinOp::Add => {
                            trace!("Tile add");
                            // mov op1, res
                            // add op2 res
                        },
                        op::BinOp::Sub => {
                            trace!("Tile sub")
                            // mov op1, res
                            // sub op1, res
                        },
                        op::BinOp::Mul => {
                            trace!("Tile mul")
                            // mov op1 rax
                            // mul op2 rax
                            // mov rax res
                        },
                        
                        _ => unimplemented!()
                    }
                }

                _ => unimplemented!()
            } // main switch
        },
        _ => panic!("expected instruction")
    }
}

#[allow(unused_variables)]
fn process_dest(ops: &mut Vec<P<TreeNode>>, dest: &Destination) {
    for dest_arg in dest.args.iter() {
        match dest_arg {
            &DestArg::Normal(op_index) => {
                let ref mut arg = ops[op_index];
                match arg.op {
                    OpCode::RegI64 
                    | OpCode::RegFP
                    | OpCode::IntImmI64
                    | OpCode::FPImm => {
                        // do nothing
                    },
                    _ => {
                        trace!("nested: compute arg for branch");
                        // nested: compute arg
                        instruction_select(arg);
                    }
                }
            },
            &DestArg::Freshbound(_) => unimplemented!()
        }
    }
}