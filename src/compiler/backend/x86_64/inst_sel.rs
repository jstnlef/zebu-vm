use ast::ir::*;
use ast::ptr::*;
use ast::inst::Instruction;
use ast::inst::Destination;
use ast::inst::DestArg;
use ast::inst::Instruction_;
use ast::op;
use ast::op::OpCode;
use ast::types;
use vm::context::VMContext;

use compiler::CompilerPass;
use compiler::backend::x86_64;
use compiler::backend::x86_64::CodeGenerator;
use compiler::backend::x86_64::ASMCodeGen;

pub struct InstructionSelection {
    name: &'static str,
    
    backend: Box<CodeGenerator>
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
    fn instruction_select(&mut self, node: &'a P<TreeNode>) {
        trace!("instsel on node {}", node);
        
//        let mut state = inst.state.borrow_mut();
//        *state = Some(BURSState::new(MATCH_RES_LEN));
        
        match node.v {
            TreeNode_::Instruction(ref inst) => {
                match inst.v {
                    Instruction_::Branch2{cond, ref true_dest, ref false_dest, true_prob} => {
                        // move this to trace generation
                        // assert here
                        let (fallthrough_dest, branch_dest, branch_if_true) = {
                            if true_prob > 0.5f32 {
                                (true_dest, false_dest, false)
                            } else {
                                (false_dest, true_dest, true)
                            }
                        };
                        
                        let ops = inst.ops.borrow();
                        
                        self.process_dest(&ops, fallthrough_dest);
                        self.process_dest(&ops, branch_dest);
    
                        let ref cond = ops[cond];
                        
                        if self.match_cmp_res(cond) {
                            trace!("emit cmp_eq-branch2");
                            match self.emit_cmp_res(cond) {
                                op::CmpOp::EQ => self.backend.emit_je(branch_dest),
                                op::CmpOp::NE => self.backend.emit_jne(branch_dest),
                                op::CmpOp::SGE => self.backend.emit_jae(branch_dest),
                                op::CmpOp::SGT => self.backend.emit_ja(branch_dest),
                                op::CmpOp::SLE => self.backend.emit_jbe(branch_dest),
                                op::CmpOp::SLT => self.backend.emit_jb(branch_dest),
                                _ => unimplemented!()
                            }
                        } else if self.match_ireg(cond) {
                            trace!("emit ireg-branch2");
                            
                            let cond_reg = self.emit_ireg(cond);
                            
                            // emit: cmp cond_reg 1
                            self.backend.emit_cmp_r64_imm32(&cond_reg, 1);
                            // emit: je #branch_dest
                            self.backend.emit_je(branch_dest);                            
                        } else {
                            unimplemented!();
                        }
                    },
                    
                    Instruction_::Branch1(ref dest) => {
                        let ops = inst.ops.borrow();
                                            
                        self.process_dest(&ops, dest);
                        
                        trace!("emit branch1");
                        // jmp
                        self.backend.emit_jmp(dest);
                    },
                    
                    Instruction_::ExprCall{ref data, is_abort} => {
                        trace!("deal with pre-call convention");
                        
                        let ops = inst.ops.borrow();
                        for arg_index in data.args.iter() {
                            let ref arg = ops[*arg_index];
                            trace!("arg {}", arg);
                            match arg.op {
                                OpCode::RegI64 | OpCode::IntImmI64 => {
                                    trace!("emit move-gpr-arg");
                                    // move to register
                                },
                                OpCode::RegFP | OpCode::FPImm => {
                                    trace!("emit move-fpr-arg");
                                    // move to fp register
                                },
                                _ => {
                                    trace!("nested: compute arg");
                                    // instself for arg first
                                    self.instruction_select(arg);
                                    
                                    // mov based on type
                                    trace!("emit move-arg after computing arg");
                                }
                            }
                        }
                        
                        let ref func = ops[data.func];
                        // check direct call or indirect                        
                        
                        // deal with ret vals
                        unimplemented!()
                    },
                    
                    Instruction_::Return(ref vals) => {
                        let ops = inst.ops.borrow();                    
                        for val_index in vals.iter() {
                            let ref val = ops[*val_index];
                            trace!("return val: {}", val);
                            
                            match val.op {
                                OpCode::RegI64 | OpCode::IntImmI64 => {
                                    trace!("emit move-gpr-ret");
                                    // move to return register
                                }
                                OpCode::RegFP | OpCode::FPImm => {
                                    trace!("emit move-fpr-ret");
                                    // move to return fp register
                                }
                                _ => {
                                    trace!("nested: compute return val");
                                    // instsel for return val first
                                    self.instruction_select(val);
                                    
                                    // move based on type
                                    trace!("emit move-ret-val after computing arg");
                                }
                            }
                        }
                        
                        self.backend.emit_ret();
                    },
                    
                    Instruction_::BinOp(op, op1, op2) => {
                        let ops = inst.ops.borrow();
                        
                        match op {
                            op::BinOp::Add => {
                                if self.match_ireg(&ops[op1]) && self.match_ireg(&ops[op2]) {
                                    trace!("emit add-ireg-ireg");
                                    
                                    let reg_op1 = self.emit_ireg(&ops[op1]);
                                    let reg_op2 = self.emit_ireg(&ops[op2]);
                                    let res_tmp = self.emit_get_result(node);
                                    
                                    // mov op1, res
                                    self.backend.emit_mov_r64_r64(&res_tmp, &reg_op1);
                                    // add op2 res
                                    self.backend.emit_add_r64_r64(&res_tmp, &reg_op2);
                                } else if self.match_ireg(&ops[op1]) && self.match_iimm(&ops[op2]) {
                                    trace!("emit add-ireg-imm");
                                    
                                    let reg_op1 = self.emit_ireg(&ops[op1]);
                                    let reg_op2 = self.emit_get_iimm(&ops[op2]);
                                    let res_tmp = self.emit_get_result(node);
                                    
                                    // mov op1, res
                                    self.backend.emit_mov_r64_r64(&res_tmp, &reg_op1);
                                    // add op2, res
                                    self.backend.emit_add_r64_imm32(&res_tmp, reg_op2);
                                } else if self.match_iimm(&ops[op1]) && self.match_ireg(&ops[op2]) {
                                    trace!("emit add-imm-ireg");
                                    unimplemented!();
                                } else if self.match_ireg(&ops[op1]) && self.match_mem(&ops[op2]) {
                                    trace!("emit add-ireg-mem");
                                    
                                    let reg_op1 = self.emit_ireg(&ops[op1]);
                                    let reg_op2 = self.emit_mem(&ops[op2]);
                                    let res_tmp = self.emit_get_result(node);
                                    
                                    // mov op1, res
                                    self.backend.emit_mov_r64_r64(&res_tmp, &reg_op1);
                                    // add op2 res
                                    self.backend.emit_add_r64_mem64(&res_tmp, &reg_op2);
                                } else if self.match_mem(&ops[op1]) && self.match_ireg(&ops[op2]) {
                                    trace!("emit add-mem-ireg");
                                    unimplemented!();
                                } else {
                                    unimplemented!()
                                }
                            },
                            op::BinOp::Sub => {
                                if self.match_ireg(&ops[op1]) && self.match_ireg(&ops[op2]) {
                                    trace!("emit sub-ireg-ireg");
                                    
                                    let reg_op1 = self.emit_ireg(&ops[op1]);
                                    let reg_op2 = self.emit_ireg(&ops[op2]);
                                    let res_tmp = self.emit_get_result(node);
                                    
                                    // mov op1, res
                                    self.backend.emit_mov_r64_r64(&res_tmp, &reg_op1);
                                    // add op2 res
                                    self.backend.emit_sub_r64_r64(&res_tmp, &reg_op2);
                                } else if self.match_ireg(&ops[op1]) && self.match_iimm(&ops[op2]) {
                                    trace!("emit sub-ireg-imm");

                                    let reg_op1 = self.emit_ireg(&ops[op1]);
                                    let reg_op2 = self.emit_get_iimm(&ops[op2]);
                                    let res_tmp = self.emit_get_result(node);
                                    
                                    // mov op1, res
                                    self.backend.emit_mov_r64_r64(&res_tmp, &reg_op1);
                                    // add op2, res
                                    self.backend.emit_sub_r64_imm32(&res_tmp, reg_op2);
                                } else if self.match_iimm(&ops[op1]) && self.match_ireg(&ops[op2]) {
                                    trace!("emit sub-imm-ireg");
                                    unimplemented!();
                                } else if self.match_ireg(&ops[op1]) && self.match_mem(&ops[op2]) {
                                    trace!("emit sub-ireg-mem");
                                    
                                    let reg_op1 = self.emit_ireg(&ops[op1]);
                                    let reg_op2 = self.emit_mem(&ops[op2]);
                                    let res_tmp = self.emit_get_result(node);
                                    
                                    // mov op1, res
                                    self.backend.emit_mov_r64_r64(&res_tmp, &reg_op1);
                                    // sub op2 res
                                    self.backend.emit_sub_r64_mem64(&res_tmp, &reg_op2);
                                } else if self.match_mem(&ops[op1]) && self.match_ireg(&ops[op2]) {
                                    trace!("emit add-mem-ireg");
                                    unimplemented!();
                                } else {
                                    unimplemented!()
                                }
                            },
                            op::BinOp::Mul => {
                                unimplemented!()
                            },
                            
                            _ => unimplemented!()
                        }
                    }
    
                    _ => unimplemented!()
                } // main switch
            },
            
            TreeNode_::Value(ref p) => {

            }
        }
    }
    
    #[allow(unused_variables)]
    fn process_dest(&mut self, ops: &Vec<P<TreeNode>>, dest: &Destination) {
        for dest_arg in dest.args.iter() {
            match dest_arg {
                &DestArg::Normal(op_index) => {
                    let ref arg = ops[op_index];
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
    
    fn match_cmp_res(&mut self, op: &P<TreeNode>) -> bool {
        match op.v {
            TreeNode_::Instruction(ref inst) => {
                match inst.v {
                    Instruction_::CmpOp(_, _, _) => true,
                    _ => false
                }
            }
            TreeNode_::Value(_) => false
        }
    }
    
    fn emit_cmp_res(&mut self, cond: &P<TreeNode>) -> op::CmpOp {
        match cond.v {
            TreeNode_::Instruction(ref inst) => {
                let ops = inst.ops.borrow();                
                
                match inst.v {
                    Instruction_::CmpOp(op, op1, op2) => {
                        let op1 = &ops[op1];
                        let op2 = &ops[op2];
                        
                        if op::is_int_cmp(op) {                        
                            if self.match_ireg(op1) && self.match_ireg(op2) {
                                let reg_op1 = self.emit_ireg(op1);
                                let reg_op2 = self.emit_ireg(op2);
                                
                                self.backend.emit_cmp_r64_r64(&reg_op1, &reg_op2);
                            } else if self.match_ireg(op1) && self.match_iimm(op2) {
                                let reg_op1 = self.emit_ireg(op1);
                                let iimm_op2 = self.emit_get_iimm(op2);
                                
                                self.backend.emit_cmp_r64_imm32(&reg_op1, iimm_op2);
                            } else {
                                unimplemented!()
                            }
                        } else {
                            unimplemented!()
                        }
                        
                        op
                    }
                    
                    _ => panic!("expect cmp res to emit")
                }
            }
            _ => panic!("expect cmp res to emit")
        }
    }    
    
    fn match_ireg(&mut self, op: &P<TreeNode>) -> bool {
        match op.v {
            TreeNode_::Instruction(ref inst) => {
                if inst.value.is_some() {
                    if inst.value.as_ref().unwrap().len() > 1 {
                        return false;
                    }
                    
                    let ref value = inst.value.as_ref().unwrap()[0];
                    
                    if types::is_scalar(&value.ty) {
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            
            TreeNode_::Value(ref pv) => {
                pv.is_int_reg()
            }
        }
    }
    
    fn emit_ireg(&mut self, op: &P<TreeNode>) -> P<Value> {
        match op.v {
            TreeNode_::Instruction(_) => {
                self.instruction_select(op);
                
                self.emit_get_result(op)
            },
            TreeNode_::Value(ref pv) => {
                match pv.v {
                    Value_::Constant(_) => panic!("expected ireg"),
                    Value_::SSAVar(_) => {
                        pv.clone()
                    }
                }
            }
        }
    }
    
    fn match_iimm(&mut self, op: &P<TreeNode>) -> bool {
        match op.v {
            TreeNode_::Value(ref pv) if x86_64::is_valid_x86_imm(pv) => true,
            _ => false
        }
    }
    
    fn match_mem(&mut self, op: &P<TreeNode>) -> bool {
        unimplemented!()
    }
    
    fn emit_mem(&mut self, op: &P<TreeNode>) -> P<Value> {
        unimplemented!()
    }
    
    fn emit_get_iimm(&mut self, op: &P<TreeNode>) -> u32 {
        match op.v {
            TreeNode_::Value(ref pv) => {
                match pv.v {
                    Value_::Constant(Constant::Int(val)) => {
                        val as u32
                    },
                    _ => panic!("expected iimm")
                }
            },
            _ => panic!("expected iimm")
        }
    }
    
    fn emit_get_result(&mut self, node: &P<TreeNode>) -> P<Value> {
        match node.v {
            TreeNode_::Instruction(ref inst) => {
                if inst.value.is_some() {
                    if inst.value.as_ref().unwrap().len() > 1 {
                        panic!("expected ONE result from the node {}", node);
                    }
                    
                    let ref value = inst.value.as_ref().unwrap()[0];
                    
                    value.clone()
                } else {
                    panic!("expected result from the node {}", node);
                }
            }
            
            TreeNode_::Value(ref pv) => {
                pv.clone()
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