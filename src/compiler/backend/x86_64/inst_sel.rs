use ast::ir::*;
use ast::ptr::*;
use ast::inst::Instruction;
use ast::inst::Destination;
use ast::inst::DestArg;
use ast::inst::Instruction_;
use ast::op;
use ast::op::OpCode;
use ast::types;
use ast::types::MuType_;
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
                                op::CmpOp::UGE => self.backend.emit_jae(branch_dest),
                                op::CmpOp::UGT => self.backend.emit_ja(branch_dest),
                                op::CmpOp::ULE => self.backend.emit_jbe(branch_dest),
                                op::CmpOp::ULT => self.backend.emit_jb(branch_dest),
                                op::CmpOp::SGE => self.backend.emit_jge(branch_dest),
                                op::CmpOp::SGT => self.backend.emit_jg(branch_dest),
                                op::CmpOp::SLE => self.backend.emit_jle(branch_dest),
                                op::CmpOp::SLT => self.backend.emit_jl(branch_dest),
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
                        let rets = inst.value.as_ref().unwrap();
                        let ref func = ops[data.func];
                        let ref func_sig = match func.v {
                            TreeNode_::Value(ref pv) => {
                                let ty : &MuType_ = &pv.ty;
                                match ty {
                                    &MuType_::FuncRef(ref sig)
                                    | &MuType_::UFuncPtr(ref sig) => sig,
                                    _ => panic!("expected funcref/ptr type")
                                }
                            },
                            _ => panic!("expected funcref/ptr type")
                        };
                        
                        debug_assert!(func_sig.ret_tys.len() == data.args.len());
                        debug_assert!(func_sig.arg_tys.len() == rets.len());
                                                
                        let mut gpr_arg_count = 0;
                        let mut fpr_arg_count = 0;
                        for arg_index in data.args.iter() {
                            let ref arg = ops[*arg_index];
                            trace!("arg {}", arg);
                            
                            if self.match_ireg(arg) {
                                let arg = self.emit_ireg(arg);
                                
                                if gpr_arg_count < x86_64::ARGUMENT_GPRs.len() {
                                    self.backend.emit_mov_r64_r64(&x86_64::ARGUMENT_GPRs[gpr_arg_count], &arg);
                                    gpr_arg_count += 1;
                                } else {
                                    // use stack to pass argument
                                    unimplemented!();
                                }
                            } else if self.match_iimm(arg) {
                                let arg = self.emit_get_iimm(arg);
                                
                                if gpr_arg_count < x86_64::ARGUMENT_GPRs.len() {
                                    self.backend.emit_mov_r64_imm32(&x86_64::ARGUMENT_GPRs[gpr_arg_count], arg);
                                    gpr_arg_count += 1;
                                } else {
                                    // use stack to pass argument
                                    unimplemented!();
                                }
                            } else {
                                unimplemented!();
                            }
                        }
                        
                        // check direct call or indirect
                        if self.match_funcref_const(func) {
                            let target = self.emit_get_funcref_const(func);
                            
                            self.backend.emit_call_near_rel32(target);
                        } else if self.match_ireg(func) {
                            let target = self.emit_ireg(func);
                            
                            self.backend.emit_call_near_r64(&target);
                        } else if self.match_mem(func) {
                            let target = self.emit_mem(func);
                            
                            self.backend.emit_call_near_mem64(&target);
                        } else {
                            unimplemented!();
                        }
                        
                        // deal with ret vals
                        let mut gpr_ret_count = 0;
                        let mut fpr_ret_count = 0;
                        for val in rets {
                            if val.is_int_reg() {
                                if gpr_ret_count < x86_64::RETURN_GPRs.len() {
                                    self.backend.emit_mov_r64_r64(&val, &x86_64::RETURN_GPRs[gpr_ret_count]);
                                    gpr_ret_count += 1;
                                } else {
                                    // get return value by stack
                                    unimplemented!();
                                }
                            } else {
                                // floating point register
                                unimplemented!();
                            }
                        }
                    },
                    
                    Instruction_::Return(_) => {
                        self.emit_common_epilogue(inst);
                        
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
                                    let imm_op2 = self.emit_get_iimm(&ops[op2]);
                                    let res_tmp = self.emit_get_result(node);
                                    
                                    // mov op1, res
                                    self.backend.emit_mov_r64_r64(&res_tmp, &reg_op1);
                                    // add op2, res
                                    self.backend.emit_sub_r64_imm32(&res_tmp, imm_op2);
                                } else if self.match_iimm(&ops[op1]) && self.match_ireg(&ops[op2]) {
                                    trace!("emit sub-imm-ireg");
                                    unimplemented!();
                                } else if self.match_ireg(&ops[op1]) && self.match_mem(&ops[op2]) {
                                    trace!("emit sub-ireg-mem");
                                    
                                    let reg_op1 = self.emit_ireg(&ops[op1]);
                                    let mem_op2 = self.emit_mem(&ops[op2]);
                                    let res_tmp = self.emit_get_result(node);
                                    
                                    // mov op1, res
                                    self.backend.emit_mov_r64_r64(&res_tmp, &reg_op1);
                                    // sub op2 res
                                    self.backend.emit_sub_r64_mem64(&res_tmp, &mem_op2);
                                } else if self.match_mem(&ops[op1]) && self.match_ireg(&ops[op2]) {
                                    trace!("emit add-mem-ireg");
                                    unimplemented!();
                                } else {
                                    unimplemented!()
                                }
                            },
                            op::BinOp::Mul => {
                                // mov op1 -> rax
                                let rax = x86_64::RAX.clone();
                                let op1 = &ops[op1];
                                if self.match_ireg(op1) {
                                    let reg_op1 = self.emit_ireg(op1);
                                    
                                    self.backend.emit_mov_r64_r64(&rax, &reg_op1);
                                } else if self.match_iimm(op1) {
                                    let imm_op1 = self.emit_get_iimm(op1);
                                    
                                    self.backend.emit_mov_r64_imm32(&rax, imm_op1);
                                } else if self.match_mem(op1) {
                                    let mem_op1 = self.emit_mem(op1);
                                    
                                    self.backend.emit_mov_r64_mem64(&rax, &mem_op1);
                                } else {
                                    unimplemented!();
                                }
                                
                                // mul op2 -> rax
                                let op2 = &ops[op2];
                                if self.match_ireg(op2) {
                                    let reg_op2 = self.emit_ireg(op2);
                                    
                                    self.backend.emit_mul_r64(&reg_op2);
                                } else if self.match_iimm(op2) {
                                    let imm_op2 = self.emit_get_iimm(op2);
                                    
                                    // put imm in a temporary
                                    // here we use result reg as temporary
                                    let res_tmp = self.emit_get_result(node);
                                    self.backend.emit_mov_r64_imm32(&res_tmp, imm_op2);
                                    
                                    self.backend.emit_mul_r64(&res_tmp);
                                } else if self.match_mem(op2) {
                                    let mem_op2 = self.emit_mem(op2);
                                    
                                    self.backend.emit_mul_mem64(&mem_op2);
                                } else {
                                    unimplemented!();
                                }
                                
                                // mov rax -> result
                                let res_tmp = self.emit_get_result(node);
                                self.backend.emit_mov_r64_r64(&res_tmp, &rax);
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
    
    fn emit_common_prologue(&mut self, args: &Vec<P<Value>>) {
        self.backend.start_block("prologue");
        
        // push all callee-saved registers
        for reg in x86_64::CALLEE_SAVED_GPRs.iter() {
            self.backend.emit_push_r64(&reg);
        }
        
        // unload arguments
        let mut gpr_arg_count = 0;
        let mut fpr_arg_count = 0;
        for arg in args {
            if arg.is_int_reg() {
                if gpr_arg_count < x86_64::ARGUMENT_GPRs.len() {
                    self.backend.emit_mov_r64_r64(&arg, &x86_64::ARGUMENT_GPRs[gpr_arg_count]);
                    gpr_arg_count += 1;
                } else {
                    // unload from stack
                    unimplemented!();
                }
            } else if arg.is_fp_reg() {
                unimplemented!();
            } else {
                panic!("expect an arg value to be either int reg or fp reg");
            }
        }
    }
    
    fn emit_common_epilogue(&mut self, ret_inst: &Instruction) {
        self.backend.start_block("epilogue");        
        
        // pop all callee-saved registers
        for reg in x86_64::CALLEE_SAVED_GPRs.iter() {
            self.backend.emit_pop_r64(&reg);
        }
        
        let ref ops = ret_inst.ops.borrow();
        let ret_val_indices = match ret_inst.v {
            Instruction_::Return(ref vals) => vals,
            _ => panic!("expected ret inst")
        };
        
        let mut gpr_ret_count = 0;
        let mut fpr_ret_count = 0;
        for i in ret_val_indices {
            let ref ret_val = ops[*i];
            if self.match_ireg(ret_val) {
                let reg_ret_val = self.emit_ireg(ret_val);
                
                self.backend.emit_mov_r64_r64(&x86_64::RETURN_GPRs[gpr_ret_count], &reg_ret_val);
                gpr_ret_count += 1;
            } else if self.match_iimm(ret_val) {
                let imm_ret_val = self.emit_get_iimm(ret_val);
                
                self.backend.emit_mov_r64_imm32(&x86_64::RETURN_GPRs[gpr_ret_count], imm_ret_val);
                gpr_ret_count += 1;
            } else {
                unimplemented!();
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
    
    fn match_fpreg(&mut self, op: &P<TreeNode>) -> bool {
        unimplemented!()
    }
    
    fn match_iimm(&mut self, op: &P<TreeNode>) -> bool {
        match op.v {
            TreeNode_::Value(ref pv) if x86_64::is_valid_x86_imm(pv) => true,
            _ => false
        }
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
    
    fn match_funcref_const(&mut self, op: &P<TreeNode>) -> bool {
        match op.v {
            TreeNode_::Value(ref pv) => {
                match pv.v {
                    Value_::Constant(Constant::FuncRef(_)) => true,
                    Value_::Constant(Constant::UFuncRef(_)) => true,
                    _ => false
                }
            },
            _ => false 
        }
    }
    
    fn emit_get_funcref_const(&mut self, op: &P<TreeNode>) -> MuTag {
        match op.v {
            TreeNode_::Value(ref pv) => {
                match pv.v {
                    Value_::Constant(Constant::FuncRef(tag))
                    | Value_::Constant(Constant::UFuncRef(tag)) => tag,
                    _ => panic!("expected a (u)funcref const")
                }
            },
            _ => panic!("expected a (u)funcref const")
        }
    }
    
    fn match_mem(&mut self, op: &P<TreeNode>) -> bool {
        unimplemented!()
    }
    
    fn emit_mem(&mut self, op: &P<TreeNode>) -> P<Value> {
        unimplemented!()
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
        
        self.backend.start_code(func.fn_name);
        
        // prologue (get arguments from entry block first)        
        let entry_block = func.content.as_ref().unwrap().get_entry_block();
        let ref args = entry_block.content.as_ref().unwrap().args;
        self.emit_common_prologue(args);
    }

    #[allow(unused_variables)]
    fn visit_function(&mut self, vm_context: &VMContext, func: &mut MuFunction) {
        for block_label in func.block_trace.as_ref().unwrap() {
            let block = func.content.as_mut().unwrap().get_block_mut(block_label);
            
            self.backend.start_block(block.label);

            let block_content = block.content.as_mut().unwrap();

            for inst in block_content.body.iter_mut() {
                self.instruction_select(inst);
            }
        }
    }
    
    #[allow(unused_variables)]
    fn finish_function(&mut self, vm_context: &VMContext, func: &mut MuFunction) {
        self.backend.print_cur_code();
        
        self.backend.finish_code();
    }
}