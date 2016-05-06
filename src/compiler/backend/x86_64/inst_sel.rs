use ast::ir::*;
use ast::ptr::*;
use ast::inst::Destination;
use ast::inst::DestArg;
use ast::inst::Instruction_::*;
use ast::op;
use ast::op::OpCode;
use ast::types::*;
use vm::context::VMContext;

use compiler::CompilerPass;
use compiler::backend::x86_64::*;

pub struct InstructionSelection {
    name: &'static str,
    backend: Box<CodeGenerator>
}

#[derive(Clone)]
pub enum MatchResult {
    REG(P<Value>),
    MEM{base: P<Value>, index: P<Value>, scale: P<Value>, disp: P<Value>},
    IMM(P<Value>),
    FP_REG(P<Value>),
    FP_IMM(P<Value>),
}

macro_rules! results_as {
    ($results: expr, $expect: pat) => {
        {
            let find_pattern = |x: Vec<MatchResult>| {
                for i in x.iter() {
                    match i {
                        &$expect => return Some(i.clone()),
                        _ => continue
                    }
                }
            
                None
            };
            
            find_pattern($results)
        };
    }
}

macro_rules! match_result {
    ($result1: expr, $expect1: pat, $result2: expr, $expect2: pat, $block: block) => {
        {
            let r1 = results_as!($result1, $expect1);
            let r2 = results_as!($result2, $expect2);
            if r1.is_some() && r2.is_some() $block
        }
    };
    
    ($result1: expr, $expect1: pat, $block) => {
        {
            let r1 = results_as!($result1, $expect1);
            if r1.is_some() $block
        }
    };
}

impl <'a> InstructionSelection {
    pub fn new() -> InstructionSelection {
        InstructionSelection{
            name: "Instruction Selection (x64)",
            backend: Box::new(ASMCodeGen::new())
        }
    }
    
    // in this pass, we assume that
    // 1. all temporaries will use 64bit registers
    // 2. we do not need to backup/restore caller-saved registers
    // 3. we need to backup/restore all the callee-saved registers
    // if any of these assumption breaks, we will need to re-emit the code
    #[allow(unused_variables)]
    fn instruction_select(&mut self, inst: &'a P<TreeNode>) -> Option<Vec<MatchResult>> {
        trace!("instsel on node {}", inst);
        match inst.v {
            TreeNode_::Instruction(ref inst) => {
                match inst.v {
                    Branch2{cond, ref true_dest, ref false_dest, true_prob} => {
                        let (fallthrough_dest, branch_dest, branch_if_true) = {
                            if true_prob > 0.5f32 {
                                (true_dest, false_dest, false)
                            } else {
                                (false_dest, true_dest, true)
                            }
                        };
                        
                        let mut ops = inst.ops.borrow_mut();
                        
                        self.process_dest(&mut ops, fallthrough_dest);
                        self.process_dest(&mut ops, branch_dest);
    
                        let ref cond = ops[cond];
                        
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
                                                let op1 = self.instruction_select(&ops[op1]).unwrap();
                                                let op2 = self.instruction_select(&ops[op2]).unwrap();
                                                
                                                match_result!(op1, MatchResult::REG(_), op2, MatchResult::REG(_), {
                                                    
                                                });
                                                
//                                                // x86 cmp only allows second op as immediate
//                                                let (op1, op2, branch_if_true) = {
//                                                    if op1.is_int_const() && op2.is_int_reg() {
//                                                        (op2, op1, !branch_if_true)
//                                                    } else {
//                                                        (op1, op2, branch_if_true)
//                                                    }
//                                                };
//                                                
//                                                if op1.is_int_reg() && op2.is_int_reg() {
//                                                    self.backend.emit_cmp_r64_r64(op1, op2);
//                                                } else if op1.is_int_reg() && op2.is_int_const() {
//                                                    // x86 only supports immediates smaller than 32bits
//                                                    let ty : &MuType_ = &op2.ty;
//                                                    match ty {
//                                                        &MuType_::Int(len) if len <= 32 => {
//                                                            self.backend.emit_cmp_r64_imm32(op1, op2);
//                                                        },
//                                                        &MuType_::Int(len) if len > 32  => {
//                                                            self.backend.emit_cmp_r64_mem64(op1, op2);
//                                                        },
//                                                        _ => panic!("{} is supposed to be int type", ty)
//                                                    }
//                                                } else if op1.is_int_const() && op2.is_int_reg() {
//                                                    panic!("expected op2 as imm and op1 as reg found op1: {:?}, op2: {:?}", op1, op2);
//                                                } else if op1.is_int_const() && op2.is_int_const() {
//                                                    
//                                                }
                                                
                                                None
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
                                
                                None
                            },
                            
                            _ => {
                                trace!("nested: compute cond");
                                // instsel for cond first
                                self.instruction_select(cond);
                                
                                // test/cmp res 0
                                // jcc branch_dest
                                // #fallthrough_dest:
                                // ...
                                trace!("Tile value-branch2 after computing cond");
                                
                                None
                            }
                        }
                    },
                    
                    Branch1(ref dest) => {
                        let mut ops = inst.ops.borrow_mut();
                                            
                        self.process_dest(&mut ops, dest);
                        
                        trace!("Tile branch1");
                        // jmp
                        
                        None
                    },
                    
                    ExprCall{ref data, is_abort} => {
                        trace!("Tile exprcall");
                        
                        let ops = inst.ops.borrow_mut();
                        for arg_index in data.args.iter() {
                            let ref arg = ops[*arg_index];
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
                                    self.instruction_select(arg);
                                    
                                    // mov based on type
                                    trace!("Tile move-arg after computing arg");
                                }
                            }
                        }
                        
                        // emit call
                        
                        // return ret vals
                        None
                    },
                    
                    Return(ref vals) => {
                        let ops = inst.ops.borrow_mut();                    
                        for val_index in vals.iter() {
                            let ref val = ops[*val_index];
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
                                    self.instruction_select(val);
                                    
                                    // move based on type
                                    trace!("Tile move-ret-val after computing arg");
                                }
                            }
                        }
                        
                        None
                    },
                    
                    BinOp(op, op1, op2) => {
                        match op {
                            op::BinOp::Add => {
                                trace!("Tile add");
                                // mov op1, res
                                // add op2 res
                                
                                None
                            },
                            op::BinOp::Sub => {
                                trace!("Tile sub");
                                // mov op1, res
                                // sub op1, res
                                
                                None
                            },
                            op::BinOp::Mul => {
                                trace!("Tile mul");
                                // mov op1 rax
                                // mul op2 rax
                                // mov rax res
                                
                                None
                            },
                            
                            _ => unimplemented!()
                        }
                    }
    
                    _ => unimplemented!()
                } // main switch
            },
            
            TreeNode_::Value(ref p) => {
                None
            }
        }
    }
    
    #[allow(unused_variables)]
    fn process_dest(&mut self, ops: &mut Vec<P<TreeNode>>, dest: &Destination) {
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
                            self.instruction_select(arg);
                        }
                    }
                },
                &DestArg::Freshbound(_) => unimplemented!()
            }
        }
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
                self.instruction_select(inst);
            }
        }
    }
}