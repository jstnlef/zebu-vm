use ast::ir::*;
use ast::ptr::*;
use ast::inst::*;
use ast::op;
use ast::op::OpCode;
use ast::types;
use ast::types::*;
use vm::VM;
use runtime::mm;
use runtime::ValueLocation;
use runtime::thread;
use runtime::entrypoints;
use runtime::entrypoints::RuntimeEntrypoint;

use compiler::CompilerPass;
use compiler::backend;
use compiler::backend::PROLOGUE_BLOCK_NAME;
use compiler::backend::x86_64;
use compiler::backend::x86_64::CodeGenerator;
use compiler::backend::x86_64::ASMCodeGen;
use compiler::machine_code::CompiledFunction;
use compiler::frame::Frame;

use std::collections::HashMap;
use std::any::Any;

lazy_static! {
    pub static ref LONG_4_TYPE : P<MuType> = P(
        MuType::new(new_internal_id(), MuType_::mustruct(Mu("long_4"), vec![UINT32_TYPE.clone(); 4]))
    );

    pub static ref UITOFP_C0 : P<Value> = P(Value{
        hdr: MuEntityHeader::named(new_internal_id(), Mu("UITOFP_C0")),
        ty : LONG_4_TYPE.clone(),
        v  : Value_::Constant(Constant::List(vec![
            P(Value{
                hdr: MuEntityHeader::unnamed(new_internal_id()),
                ty: UINT32_TYPE.clone(),
                v : Value_::Constant(Constant::Int(1127219200u64))
            }),
            P(Value{
                hdr: MuEntityHeader::unnamed(new_internal_id()),
                ty: UINT32_TYPE.clone(),
                v : Value_::Constant(Constant::Int(1160773632u64))
            }),
            P(Value{
                hdr: MuEntityHeader::unnamed(new_internal_id()),
                ty: UINT32_TYPE.clone(),
                v : Value_::Constant(Constant::Int(0u64))
            }),
            P(Value{
                hdr: MuEntityHeader::unnamed(new_internal_id()),
                ty: UINT32_TYPE.clone(),
                v : Value_::Constant(Constant::Int(0u64))
            })
        ]))
    });

    pub static ref QUAD_2_TYPE : P<MuType> = P(
        MuType::new(new_internal_id(), MuType_::mustruct(Mu("quad_2"), vec![UINT64_TYPE.clone(); 2]))
    );

    pub static ref UITOFP_C1 : P<Value> = P(Value{
        hdr: MuEntityHeader::named(new_internal_id(), Mu("UITOFP_C1")),
        ty : QUAD_2_TYPE.clone(),
        v  : Value_::Constant(Constant::List(vec![
            P(Value{
                hdr: MuEntityHeader::unnamed(new_internal_id()),
                ty: UINT64_TYPE.clone(),
                v : Value_::Constant(Constant::Int(4841369599423283200u64))
            }),
            P(Value{
                hdr: MuEntityHeader::unnamed(new_internal_id()),
                ty: UINT64_TYPE.clone(),
                v : Value_::Constant(Constant::Int(4985484787499139072u64))
            })
        ]))
    });

    pub static ref FPTOUI_C : P<Value> = P(Value{
        hdr: MuEntityHeader::named(new_internal_id(), Mu("FPTOUI_C")),
        ty : UINT64_TYPE.clone(),
        v  : Value_::Constant(Constant::Int(4890909195324358656u64))
    });
}

pub struct InstructionSelection {
    name: &'static str,
    backend: Box<CodeGenerator>,

    current_fv_id: MuID,
    current_callsite_id: usize,
    current_frame: Option<Frame>,
    current_block: Option<MuName>,
    current_func_start: Option<ValueLocation>,
    // key: block id, val: callsite that names the block as exception block
    current_exn_callsites: HashMap<MuID, Vec<ValueLocation>>,
    // key: block id, val: block location
    current_exn_blocks: HashMap<MuID, ValueLocation>,

    current_constants: HashMap<MuID, P<Value>>,
    current_constants_locs: HashMap<MuID, P<Value>>
}

impl <'a> InstructionSelection {
    #[cfg(feature = "aot")]
    pub fn new() -> InstructionSelection {
        InstructionSelection{
            name: "Instruction Selection (x64)",
            backend: Box::new(ASMCodeGen::new()),

            current_fv_id: 0,
            current_callsite_id: 0,
            current_frame: None,
            current_block: None,
            current_func_start: None,
            // key: block id, val: callsite that names the block as exception block
            current_exn_callsites: HashMap::new(), 
            current_exn_blocks: HashMap::new(),

            current_constants: HashMap::new(),
            current_constants_locs: HashMap::new()
        }
    }

    #[cfg(feature = "jit")]
    pub fn new() -> InstructionSelection {
        unimplemented!()
    }
    
    // in this pass, we assume that
    // 1. all temporaries will use 64bit registers
    // 2. we do not need to backup/restore caller-saved registers
    // 3. we need to backup/restore all the callee-saved registers
    // if any of these assumption breaks, we will need to re-emit the code
    #[allow(unused_variables)]
    fn instruction_select(&mut self, node: &'a TreeNode, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) {
        trace!("instsel on node#{} {}", node.id(), node);
        
        match node.v {
            TreeNode_::Instruction(ref inst) => {
                match inst.v {
                    Instruction_::Branch2{cond, ref true_dest, ref false_dest, true_prob} => {
                        trace!("instsel on BRANCH2");
                        // 'branch_if_true' == true, we emit cjmp the same as CmpOp  (je  for EQ, jne for NE)
                        // 'branch_if_true' == false, we emit opposite cjmp as CmpOp (jne for EQ, je  for NE)
                        let (fallthrough_dest, branch_dest, branch_if_true) = {
                            if true_prob > 0.5f32 {
                                (true_dest, false_dest, false)
                            } else {
                                (false_dest, true_dest, true)
                            }
                        };
                        
                        let ops = inst.ops.read().unwrap();
                        
                        self.process_dest(&ops, fallthrough_dest, f_content, f_context, vm);
                        self.process_dest(&ops, branch_dest, f_content, f_context, vm);
                        
                        let branch_target = f_content.get_block(branch_dest.target).name().unwrap();
    
                        let ref cond = ops[cond];
                        
                        if self.match_cmp_res(cond) {
                            trace!("emit cmp_res-branch2");
                            match self.emit_cmp_res(cond, f_content, f_context, vm) {
                                op::CmpOp::EQ => {
                                    if branch_if_true {
                                        self.backend.emit_je(branch_target);
                                    } else {
                                        self.backend.emit_jne(branch_target);
                                    }
                                },
                                op::CmpOp::NE => {
                                    if branch_if_true {
                                        self.backend.emit_jne(branch_target);
                                    } else {
                                        self.backend.emit_je(branch_target);
                                    }
                                },
                                op::CmpOp::UGE => {
                                    if branch_if_true {
                                        self.backend.emit_jae(branch_target);
                                    } else {
                                        self.backend.emit_jb(branch_target);
                                    }
                                },
                                op::CmpOp::UGT => {
                                    if branch_if_true {
                                        self.backend.emit_ja(branch_target);
                                    } else {
                                        self.backend.emit_jbe(branch_target);
                                    }
                                },
                                op::CmpOp::ULE => {
                                    if branch_if_true {
                                        self.backend.emit_jbe(branch_target);
                                    } else {
                                        self.backend.emit_ja(branch_target);
                                    }
                                },
                                op::CmpOp::ULT => {
                                    if branch_if_true {
                                        self.backend.emit_jb(branch_target);
                                    } else {
                                        self.backend.emit_jae(branch_target);
                                    }
                                },
                                op::CmpOp::SGE => {
                                    if branch_if_true {
                                        self.backend.emit_jge(branch_target);
                                    } else {
                                        self.backend.emit_jl(branch_target);
                                    }
                                },
                                op::CmpOp::SGT => {
                                    if branch_if_true {
                                        self.backend.emit_jg(branch_target);
                                    } else {
                                        self.backend.emit_jle(branch_target);
                                    }
                                },
                                op::CmpOp::SLE => {
                                    if branch_if_true {
                                        self.backend.emit_jle(branch_target);
                                    } else {
                                        self.backend.emit_jg(branch_target);
                                    }
                                },
                                op::CmpOp::SLT => {
                                    if branch_if_true {
                                        self.backend.emit_jl(branch_target);
                                    } else {
                                        self.backend.emit_jge(branch_target);
                                    }
                                },

                                // floating point

                                op::CmpOp::FOEQ | op::CmpOp::FUEQ => {
                                    if branch_if_true {
                                        self.backend.emit_je(branch_target);
                                    } else {
                                        self.backend.emit_jne(branch_target);
                                    }
                                },
                                op::CmpOp::FONE | op::CmpOp::FUNE => {
                                    if branch_if_true {
                                        self.backend.emit_jne(branch_target);
                                    } else {
                                        self.backend.emit_je(branch_target);
                                    }
                                },
                                op::CmpOp::FOGT | op::CmpOp::FUGT => {
                                    if branch_if_true {
                                        self.backend.emit_ja(branch_target);
                                    } else {
                                        self.backend.emit_jbe(branch_target);
                                    }
                                },
                                op::CmpOp::FOGE | op::CmpOp::FUGE => {
                                    if branch_if_true {
                                        self.backend.emit_jae(branch_target);
                                    } else {
                                        self.backend.emit_jb(branch_target);
                                    }
                                },
                                op::CmpOp::FOLT | op::CmpOp::FULT => {
                                    if branch_if_true {
                                        self.backend.emit_jb(branch_target);
                                    } else {
                                        self.backend.emit_jae(branch_target);
                                    }
                                },
                                op::CmpOp::FOLE | op::CmpOp::FULE => {
                                    if branch_if_true {
                                        self.backend.emit_jbe(branch_target);
                                    } else {
                                        self.backend.emit_ja(branch_target);
                                    }
                                },

                                _ => unimplemented!()
                            }
                        } else if self.match_ireg(cond) {
                            trace!("emit ireg-branch2");
                            
                            let cond_reg = self.emit_ireg(cond, f_content, f_context, vm);
                            
                            // emit: cmp cond_reg 1
                            self.backend.emit_cmp_imm_r(1, &cond_reg);
                            // emit: je #branch_dest
                            self.backend.emit_je(branch_target);
                        } else {
                            unimplemented!();
                        }
                    },

                    Instruction_::Select{cond, true_val, false_val} => {
                        use ast::op::CmpOp::*;

                        trace!("instsel on SELECT");
                        let ops = inst.ops.read().unwrap();

                        let ref cond = ops[cond];
                        let ref true_val = ops[true_val];
                        let ref false_val = ops[false_val];

                        if self.match_ireg(true_val) {
                            // moving integers/pointers
                            let tmp_res   = self.get_result_value(node);

                            // generate compare
                            let cmpop = if self.match_cmp_res(cond) {
                                self.emit_cmp_res(cond, f_content, f_context, vm)
                            } else if self.match_ireg(cond) {
                                let tmp_cond = self.emit_ireg(cond, f_content, f_context, vm);
                                // emit: cmp cond_reg 1
                                self.backend.emit_cmp_imm_r(1, &tmp_cond);

                                EQ
                            } else {
                                panic!("expected ireg, found {}", cond)
                            };

                            // use cmov for 16/32/64bit integeer
                            // use jcc  for 8 bit
                            match tmp_res.ty.get_int_length() {
                                // cmov
                                Some(len) if len > 8 => {
                                    let tmp_true  = self.emit_ireg(true_val, f_content, f_context, vm);
                                    let tmp_false = self.emit_ireg(false_val, f_content, f_context, vm);

                                    // mov tmp_false -> tmp_res
                                    self.backend.emit_mov_r_r(&tmp_res, &tmp_false);

                                    match cmpop {
                                        EQ  => self.backend.emit_cmove_r_r (&tmp_res, &tmp_true),
                                        NE  => self.backend.emit_cmovne_r_r(&tmp_res, &tmp_true),
                                        SGE => self.backend.emit_cmovge_r_r(&tmp_res, &tmp_true),
                                        SGT => self.backend.emit_cmovg_r_r (&tmp_res, &tmp_true),
                                        SLE => self.backend.emit_cmovle_r_r(&tmp_res, &tmp_true),
                                        SLT => self.backend.emit_cmovl_r_r (&tmp_res, &tmp_true),
                                        UGE => self.backend.emit_cmovae_r_r(&tmp_res, &tmp_true),
                                        UGT => self.backend.emit_cmova_r_r (&tmp_res, &tmp_true),
                                        ULE => self.backend.emit_cmovbe_r_r(&tmp_res, &tmp_true),
                                        ULT => self.backend.emit_cmovb_r_r (&tmp_res, &tmp_true),

                                        FOEQ | FUEQ => self.backend.emit_cmove_r_r (&tmp_res, &tmp_true),
                                        FONE | FUNE => self.backend.emit_cmovne_r_r(&tmp_res, &tmp_true),
                                        FOGT | FUGT => self.backend.emit_cmova_r_r (&tmp_res, &tmp_true),
                                        FOGE | FUGE => self.backend.emit_cmovae_r_r(&tmp_res, &tmp_true),
                                        FOLT | FULT => self.backend.emit_cmovb_r_r (&tmp_res, &tmp_true),
                                        FOLE | FULE => self.backend.emit_cmovbe_r_r(&tmp_res, &tmp_true),

                                        _ => unimplemented!()
                                    }
                                }
                                // jcc
                                _ => {
                                    let blk_true  = format!("{}_select_true", node.id());
                                    let blk_false = format!("{}_select_false", node.id());
                                    let blk_end   = format!("{}_select_end", node.id());

                                    // jump to blk_true if true
                                    match cmpop {
                                        EQ  => self.backend.emit_je (blk_true.clone()),
                                        NE  => self.backend.emit_jne(blk_true.clone()),
                                        SGE => self.backend.emit_jge(blk_true.clone()),
                                        SGT => self.backend.emit_jg (blk_true.clone()),
                                        SLE => self.backend.emit_jle(blk_true.clone()),
                                        SLT => self.backend.emit_jl (blk_true.clone()),
                                        UGE => self.backend.emit_jae(blk_true.clone()),
                                        UGT => self.backend.emit_ja (blk_true.clone()),
                                        ULE => self.backend.emit_jbe(blk_true.clone()),
                                        ULT => self.backend.emit_jb (blk_true.clone()),

                                        FOEQ | FUEQ => self.backend.emit_je (blk_true.clone()),
                                        FONE | FUNE => self.backend.emit_jne(blk_true.clone()),
                                        FOGT | FUGT => self.backend.emit_ja (blk_true.clone()),
                                        FOGE | FUGE => self.backend.emit_jae(blk_true.clone()),
                                        FOLT | FULT => self.backend.emit_jb (blk_true.clone()),
                                        FOLE | FULE => self.backend.emit_jbe(blk_true.clone()),

                                        _ => unimplemented!()
                                    }

                                    // finishing current block
                                    let cur_block = self.current_block.as_ref().unwrap().clone();
                                    self.backend.end_block(cur_block.clone());

                                    // blk_false:
                                    self.current_block = Some(blk_false.clone());
                                    self.backend.start_block(blk_false.clone());

                                    // mov false result here
                                    self.emit_move_node_to_value(&tmp_res, &false_val, f_content, f_context, vm);

                                    // jmp to end
                                    self.backend.emit_jmp(blk_end.clone());

                                    // finishing current block
                                    let cur_block = self.current_block.as_ref().unwrap().clone();
                                    self.backend.end_block(cur_block.clone());

                                    // blk_true:
                                    self.current_block = Some(blk_true.clone());
                                    self.backend.start_block(blk_true.clone());
                                    // mov true value -> result
                                    self.emit_move_node_to_value(&tmp_res, &true_val, f_content, f_context, vm);

                                    self.backend.end_block(blk_true.clone());

                                    // blk_end:
                                    self.backend.start_block(blk_end.clone());
                                    self.current_block = Some(blk_end.clone());
                                }
                            }
                        } else {
                            // moving vectors, floatingpoints
                            unimplemented!()
                        }
                    },

                    Instruction_::CmpOp(op, op1, op2) => {
                        use ast::op::CmpOp::*;

                        trace!("instsel on CMPOP");
                        let ops = inst.ops.read().unwrap();
                        let ref op1 = ops[op1];
                        let ref op2 = ops[op2];

                        let tmp_res = self.get_result_value(node);

                        // cmov only take (16/32/64bits registers)
                        // make res64, and set to zero
                        let tmp_res64 = self.make_temporary(f_context, UINT64_TYPE.clone(), vm);
                        self.backend.emit_xor_r_r(&tmp_res64, &tmp_res64);

                        // set tmp1 as 1 (cmov doesnt allow immediate or reg8 as operand)
                        let tmp_1 = self.make_temporary(f_context, UINT64_TYPE.clone(), vm);
                        self.backend.emit_mov_r_imm(&tmp_1, 1);

                        // cmov 1 to result
                        match self.emit_cmp_res(node, f_content, f_context, vm) {
                            EQ  => self.backend.emit_cmove_r_r (&tmp_res64, &tmp_1),
                            NE  => self.backend.emit_cmovne_r_r(&tmp_res64, &tmp_1),
                            SGE => self.backend.emit_cmovge_r_r(&tmp_res64, &tmp_1),
                            SGT => self.backend.emit_cmovg_r_r (&tmp_res64, &tmp_1),
                            SLE => self.backend.emit_cmovle_r_r(&tmp_res64, &tmp_1),
                            SLT => self.backend.emit_cmovl_r_r (&tmp_res64, &tmp_1),
                            UGE => self.backend.emit_cmovae_r_r(&tmp_res64, &tmp_1),
                            UGT => self.backend.emit_cmova_r_r (&tmp_res64, &tmp_1),
                            ULE => self.backend.emit_cmovbe_r_r(&tmp_res64, &tmp_1),
                            ULT => self.backend.emit_cmovb_r_r (&tmp_res64, &tmp_1),

                            FOEQ | FUEQ => self.backend.emit_cmove_r_r (&tmp_res64, &tmp_1),
                            FONE | FUNE => self.backend.emit_cmovne_r_r(&tmp_res64, &tmp_1),
                            FOGT | FUGT => self.backend.emit_cmova_r_r (&tmp_res64, &tmp_1),
                            FOGE | FUGE => self.backend.emit_cmovae_r_r(&tmp_res64, &tmp_1),
                            FOLT | FULT => self.backend.emit_cmovb_r_r (&tmp_res64, &tmp_1),
                            FOLE | FULE => self.backend.emit_cmovbe_r_r(&tmp_res64, &tmp_1),

                            _ => unimplemented!()
                        }

                        // truncate tmp_res64 to tmp_res (probably u8)
                        self.backend.emit_mov_r_r(&tmp_res, &tmp_res64);
                    }

                    Instruction_::Branch1(ref dest) => {
                        trace!("instsel on BRANCH1");
                        let ops = inst.ops.read().unwrap();
                                            
                        self.process_dest(&ops, dest, f_content, f_context, vm);
                        
                        let target = f_content.get_block(dest.target).name().unwrap();
                        
                        trace!("emit branch1");
                        // jmp
                        self.backend.emit_jmp(target);
                    },

                    Instruction_::Switch{cond, ref default, ref branches} => {
                        trace!("instsel on SWITCH");
                        let ops = inst.ops.read().unwrap();

                        let ref cond = ops[cond];

                        if self.match_ireg(cond) {
                            let tmp_cond = self.emit_ireg(cond, f_content, f_context, vm);

                            // emit each branch
                            for &(case_op_index, ref case_dest) in branches {
                                let ref case_op = ops[case_op_index];

                                // process dest
                                self.process_dest(&ops, case_dest, f_content, f_context, vm);

                                let target = f_content.get_block(case_dest.target).name().unwrap();

                                if self.match_iimm(case_op) {
                                    let imm = self.node_iimm_to_i32(case_op);

                                    // cmp case cond
                                    self.backend.emit_cmp_imm_r(imm, &tmp_cond);
                                    // je dest
                                    self.backend.emit_je(target);
                                } else if self.match_ireg(case_op) {
                                    let tmp_case_op = self.emit_ireg(case_op, f_content, f_context, vm);

                                    // cmp case cond
                                    self.backend.emit_cmp_r_r(&tmp_case_op, &tmp_cond);
                                    // je dest
                                    self.backend.emit_je(target);
                                } else {
                                    panic!("expecting ireg cond to be either iimm or ireg: {}", cond);
                                }

                                self.finish_block();
                                self.start_block(format!("{}_switch_not_met_case_{}", node.id(), case_op_index));
                            }

                            // emit default
                            self.process_dest(&ops, default, f_content, f_context, vm);
                            
                            let default_target = f_content.get_block(default.target).name().unwrap();
                            self.backend.emit_jmp(default_target);
                        } else {
                            panic!("expecting cond in switch to be ireg: {}", cond);
                        }
                    }
                    
                    Instruction_::ExprCall{ref data, is_abort} => {
                        trace!("instsel on EXPRCALL");

                        if is_abort {
                            unimplemented!()
                        }
                        
                        self.emit_mu_call(
                            inst, // inst: &Instruction,
                            data, // calldata: &CallData,
                            None, // resumption: Option<&ResumptionData>,
                            node, // cur_node: &TreeNode, 
                            f_content, f_context, vm);
                    },
                    
                    Instruction_::Call{ref data, ref resume} => {
                        trace!("instsel on CALL");

                        self.emit_mu_call(
                            inst, 
                            data, 
                            Some(resume), 
                            node, 
                            f_content, f_context, vm);
                    },

                    Instruction_::ExprCCall{ref data, is_abort} => {
                        trace!("instsel on EXPRCCALL");

                        if is_abort {
                            unimplemented!()
                        }

                        self.emit_c_call_ir(inst, data, None, node, f_content, f_context, vm);
                    }

                    Instruction_::CCall{ref data, ref resume} => {
                        trace!("instsel on CCALL");

                        self.emit_c_call_ir(inst, data, Some(resume), node, f_content, f_context, vm);
                    }
                    
                    Instruction_::Return(_) => {
                        trace!("instsel on RETURN");

                        self.emit_common_epilogue(inst, f_content, f_context, vm);
                        
                        self.backend.emit_ret();
                    },
                    
                    Instruction_::BinOp(op, op1, op2) => {
                        trace!("instsel on BINOP");

                        let ops = inst.ops.read().unwrap();

                        let res_tmp = self.get_result_value(node);
                        
                        match op {
                            op::BinOp::Add => {
                                if self.match_ireg(&ops[op1]) && self.match_iimm(&ops[op2]) {
                                    trace!("emit add-ireg-imm");
                                    
                                    let reg_op1 = self.emit_ireg(&ops[op1], f_content, f_context, vm);
                                    let reg_op2 = self.node_iimm_to_i32(&ops[op2]);
                                    
                                    // mov op1, res
                                    self.backend.emit_mov_r_r(&res_tmp, &reg_op1);
                                    // add op2, res
                                    self.backend.emit_add_r_imm(&res_tmp, reg_op2);
                                } else if self.match_ireg(&ops[op1]) && self.match_mem(&ops[op2]) {
                                    trace!("emit add-ireg-mem");
                                    
                                    let reg_op1 = self.emit_ireg(&ops[op1], f_content, f_context, vm);
                                    let reg_op2 = self.emit_mem(&ops[op2], vm);
                                    
                                    // mov op1, res
                                    self.backend.emit_mov_r_r(&res_tmp, &reg_op1);
                                    // add op2 res
                                    self.backend.emit_add_r_mem(&res_tmp, &reg_op2);
                                } else if self.match_ireg(&ops[op1]) && self.match_ireg(&ops[op2]) {
                                    trace!("emit add-ireg-ireg");

                                    let reg_op1 = self.emit_ireg(&ops[op1], f_content, f_context, vm);
                                    let reg_op2 = self.emit_ireg(&ops[op2], f_content, f_context, vm);

                                    // mov op1, res
                                    self.backend.emit_mov_r_r(&res_tmp, &reg_op1);
                                    // add op2 res
                                    self.backend.emit_add_r_r(&res_tmp, &reg_op2);
                                } else {
                                    unimplemented!()
                                }
                            },
                            op::BinOp::Sub => {
                                if self.match_ireg(&ops[op1]) && self.match_iimm(&ops[op2]) {
                                    trace!("emit sub-ireg-imm");

                                    let reg_op1 = self.emit_ireg(&ops[op1], f_content, f_context, vm);
                                    let imm_op2 = self.node_iimm_to_i32(&ops[op2]);
                                    
                                    // mov op1, res
                                    self.backend.emit_mov_r_r(&res_tmp, &reg_op1);
                                    // add op2, res
                                    self.backend.emit_sub_r_imm(&res_tmp, imm_op2);
                                } else if self.match_ireg(&ops[op1]) && self.match_mem(&ops[op2]) {
                                    trace!("emit sub-ireg-mem");
                                    
                                    let reg_op1 = self.emit_ireg(&ops[op1], f_content, f_context, vm);
                                    let mem_op2 = self.emit_mem(&ops[op2], vm);
                                    
                                    // mov op1, res
                                    self.backend.emit_mov_r_r(&res_tmp, &reg_op1);
                                    // sub op2 res
                                    self.backend.emit_sub_r_mem(&res_tmp, &mem_op2);
                                } else if self.match_ireg(&ops[op1]) && self.match_ireg(&ops[op2]) {
                                    trace!("emit sub-ireg-ireg");

                                    let reg_op1 = self.emit_ireg(&ops[op1], f_content, f_context, vm);
                                    let reg_op2 = self.emit_ireg(&ops[op2], f_content, f_context, vm);

                                    // mov op1, res
                                    self.backend.emit_mov_r_r(&res_tmp, &reg_op1);
                                    // add op2 res
                                    self.backend.emit_sub_r_r(&res_tmp, &reg_op2);
                                } else {
                                    unimplemented!()
                                }
                            },
                            op::BinOp::And => {
                                let op1 = &ops[op1];
                                let op2 = &ops[op2];

                                if self.match_ireg(op1) && self.match_iimm(op2) {
                                    trace!("emit and-ireg-iimm");

                                    let tmp_op1 = self.emit_ireg(op1, f_content, f_context, vm);
                                    let imm_op2 = self.node_iimm_to_i32(op2);

                                    // mov op1 -> res
                                    self.backend.emit_mov_r_r(&res_tmp, &tmp_op1);
                                    // and op2, res -> res
                                    self.backend.emit_and_r_imm(&res_tmp, imm_op2);
                                } else if self.match_ireg(op1) && self.match_mem(op2) {
                                    trace!("emit and-ireg-mem");

                                    let tmp_op1 = self.emit_ireg(op1, f_content, f_context, vm);
                                    let mem_op2 = self.emit_mem(op2, vm);

                                    // mov op1, res
                                    self.backend.emit_mov_r_r(&res_tmp, &tmp_op1);
                                    // and op2, res -> res
                                    self.backend.emit_and_r_mem(&res_tmp, &mem_op2);
                                } else if self.match_ireg(op1) && self.match_ireg(op2) {
                                    trace!("emit and-ireg-ireg");

                                    let tmp_op1 = self.emit_ireg(op1, f_content, f_context, vm);
                                    let tmp_op2 = self.emit_ireg(op2, f_content, f_context, vm);

                                    // mov op1, res
                                    self.backend.emit_mov_r_r(&res_tmp, &tmp_op1);
                                    // and op2, res -> res
                                    self.backend.emit_and_r_r(&res_tmp, &tmp_op2);
                                } else {
                                    unimplemented!()
                                }
                            },
                            op::BinOp::Or => {
                                let op1 = &ops[op1];
                                let op2 = &ops[op2];

                                if self.match_ireg(op1) && self.match_iimm(op2) {
                                    trace!("emit or-ireg-iimm");

                                    let tmp_op1 = self.emit_ireg(op1, f_content, f_context, vm);
                                    let imm_op2 = self.node_iimm_to_i32(op2);

                                    // mov op1 -> res
                                    self.backend.emit_mov_r_r(&res_tmp, &tmp_op1);
                                    // Or op2, res -> res
                                    self.backend.emit_or_r_imm(&res_tmp, imm_op2);
                                } else if self.match_ireg(op1) && self.match_mem(op2) {
                                    trace!("emit or-ireg-mem");

                                    let tmp_op1 = self.emit_ireg(op1, f_content, f_context, vm);
                                    let mem_op2 = self.emit_mem(op2, vm);

                                    // mov op1, res
                                    self.backend.emit_mov_r_r(&res_tmp, &tmp_op1);
                                    // Or op2, res -> res
                                    self.backend.emit_or_r_mem(&res_tmp, &mem_op2);
                                } else if self.match_ireg(op1) && self.match_ireg(op2) {
                                    trace!("emit or-ireg-ireg");

                                    let tmp_op1 = self.emit_ireg(op1, f_content, f_context, vm);
                                    let tmp_op2 = self.emit_ireg(op2, f_content, f_context, vm);

                                    // mov op1, res
                                    self.backend.emit_mov_r_r(&res_tmp, &tmp_op1);
                                    // Or op2, res -> res
                                    self.backend.emit_or_r_r(&res_tmp, &tmp_op2);
                                } else {
                                    unimplemented!()
                                }
                            },
                            op::BinOp::Xor => {
                                let op1 = &ops[op1];
                                let op2 = &ops[op2];

                                if self.match_ireg(op1) && self.match_iimm(op2) {
                                    trace!("emit xor-ireg-iimm");

                                    let tmp_op1 = self.emit_ireg(op1, f_content, f_context, vm);
                                    let imm_op2 = self.node_iimm_to_i32(op2);

                                    // mov op1 -> res
                                    self.backend.emit_mov_r_r(&res_tmp, &tmp_op1);
                                    // xor op2, res -> res
                                    self.backend.emit_xor_r_imm(&res_tmp, imm_op2);
                                } else if self.match_ireg(op1) && self.match_mem(op2) {
                                    trace!("emit xor-ireg-mem");

                                    let tmp_op1 = self.emit_ireg(op1, f_content, f_context, vm);
                                    let mem_op2 = self.emit_mem(op2, vm);

                                    // mov op1, res
                                    self.backend.emit_mov_r_r(&res_tmp, &tmp_op1);
                                    // xor op2, res -> res
                                    self.backend.emit_xor_r_mem(&res_tmp, &mem_op2);
                                } else if self.match_ireg(op1) && self.match_ireg(op2) {
                                    trace!("emit xor-ireg-ireg");

                                    let tmp_op1 = self.emit_ireg(op1, f_content, f_context, vm);
                                    let tmp_op2 = self.emit_ireg(op2, f_content, f_context, vm);

                                    // mov op1, res
                                    self.backend.emit_mov_r_r(&res_tmp, &tmp_op1);
                                    // xor op2, res -> res
                                    self.backend.emit_xor_r_r(&res_tmp, &tmp_op2);
                                } else {
                                    unimplemented!()
                                }
                            }
                            op::BinOp::Mul => {
                                // mov op1 -> rax
                                let op1 = &ops[op1];

                                let mreg_op1 = match op1.clone_value().ty.get_int_length() {
                                    Some(64) => x86_64::RAX.clone(),
                                    Some(32) => x86_64::EAX.clone(),
                                    Some(16) => x86_64::AX.clone(),
                                    Some(8)  => x86_64::AL.clone(),
                                    _ => unimplemented!()
                                };

                                if self.match_iimm(op1) {
                                    let imm_op1 = self.node_iimm_to_i32(op1);
                                    
                                    self.backend.emit_mov_r_imm(&mreg_op1, imm_op1);
                                } else if self.match_mem(op1) {
                                    let mem_op1 = self.emit_mem(op1, vm);
                                    
                                    self.backend.emit_mov_r_mem(&mreg_op1, &mem_op1);
                                } else if self.match_ireg(op1) {
                                    let reg_op1 = self.emit_ireg(op1, f_content, f_context, vm);

                                    self.backend.emit_mov_r_r(&mreg_op1, &reg_op1);
                                } else {
                                    unimplemented!();
                                }
                                
                                // mul op2
                                let op2 = &ops[op2];
                                if self.match_iimm(op2) {
                                    let imm_op2 = self.node_iimm_to_i32(op2);
                                    
                                    // put imm in a temporary
                                    // here we use result reg as temporary
                                    self.backend.emit_mov_r_imm(&res_tmp, imm_op2);
                                    
                                    self.backend.emit_mul_r(&res_tmp);
                                } else if self.match_mem(op2) {
                                    let mem_op2 = self.emit_mem(op2, vm);
                                    
                                    self.backend.emit_mul_mem(&mem_op2);
                                } else if self.match_ireg(op2) {
                                    let reg_op2 = self.emit_ireg(op2, f_content, f_context, vm);

                                    self.backend.emit_mul_r(&reg_op2);
                                } else {
                                    unimplemented!();
                                }
                                
                                // mov rax -> result
                                match res_tmp.ty.get_int_length() {
                                    Some(64) => self.backend.emit_mov_r_r(&res_tmp, &x86_64::RAX),
                                    Some(32) => self.backend.emit_mov_r_r(&res_tmp, &x86_64::EAX),
                                    Some(16) => self.backend.emit_mov_r_r(&res_tmp, &x86_64::AX),
                                    Some(8)  => self.backend.emit_mov_r_r(&res_tmp, &x86_64::AL),
                                    _ => unimplemented!()
                                }

                            },
                            op::BinOp::Udiv => {
                                let op1 = &ops[op1];
                                let op2 = &ops[op2];

                                self.emit_udiv(op1, op2, f_content, f_context, vm);

                                // mov rax -> result
                                match res_tmp.ty.get_int_length() {
                                    Some(64) => {
                                        self.backend.emit_mov_r_r(&res_tmp, &x86_64::RAX);
                                    }
                                    Some(32) => {
                                        self.backend.emit_mov_r_r(&res_tmp, &x86_64::EAX);
                                    }
                                    Some(16) => {
                                        self.backend.emit_mov_r_r(&res_tmp, &x86_64::AX);
                                    }
                                    Some(8)  => {
                                        self.backend.emit_mov_r_r(&res_tmp, &x86_64::AL);
                                    }
                                    _ => unimplemented!()
                                }
                            },
                            op::BinOp::Sdiv => {
                                let op1 = &ops[op1];
                                let op2 = &ops[op2];

                                self.emit_idiv(op1, op2, f_content, f_context, vm);

                                // mov rax -> result
                                match res_tmp.ty.get_int_length() {
                                    Some(64) => {
                                        self.backend.emit_mov_r_r(&res_tmp, &x86_64::RAX);
                                    }
                                    Some(32) => {
                                        self.backend.emit_mov_r_r(&res_tmp, &x86_64::EAX);
                                    }
                                    Some(16) => {
                                        self.backend.emit_mov_r_r(&res_tmp, &x86_64::AX);
                                    }
                                    Some(8)  => {
                                        self.backend.emit_mov_r_r(&res_tmp, &x86_64::AL);
                                    }
                                    _ => unimplemented!()
                                }
                            },
                            op::BinOp::Urem => {
                                let op1 = &ops[op1];
                                let op2 = &ops[op2];

                                self.emit_udiv(op1, op2, f_content, f_context, vm);

                                // mov rdx -> result
                                match res_tmp.ty.get_int_length() {
                                    Some(64) => {
                                        self.backend.emit_mov_r_r(&res_tmp, &x86_64::RDX);
                                    }
                                    Some(32) => {
                                        self.backend.emit_mov_r_r(&res_tmp, &x86_64::EDX);
                                    }
                                    Some(16) => {
                                        self.backend.emit_mov_r_r(&res_tmp, &x86_64::DX);
                                    }
                                    Some(8)  => {
                                        self.backend.emit_mov_r_r(&res_tmp, &x86_64::AH);
                                    }
                                    _ => unimplemented!()
                                }
                            },
                            op::BinOp::Srem => {
                                let op1 = &ops[op1];
                                let op2 = &ops[op2];

                                self.emit_idiv(op1, op2, f_content, f_context, vm);

                                // mov rdx -> result
                                match res_tmp.ty.get_int_length() {
                                    Some(64) => {
                                        self.backend.emit_mov_r_r(&res_tmp, &x86_64::RDX);
                                    }
                                    Some(32) => {
                                        self.backend.emit_mov_r_r(&res_tmp, &x86_64::EDX);
                                    }
                                    Some(16) => {
                                        self.backend.emit_mov_r_r(&res_tmp, &x86_64::DX);
                                    }
                                    Some(8)  => {
                                        self.backend.emit_mov_r_r(&res_tmp, &x86_64::AH);
                                    }
                                    _ => unimplemented!()
                                }
                            },

                            op::BinOp::Shl => {
                                let op1 = &ops[op1];
                                let op2 = &ops[op2];

                                if self.match_mem(op1) {
                                    unimplemented!()
                                } else if self.match_ireg(op1) {
                                    let tmp_op1 = self.emit_ireg(op1, f_content, f_context, vm);

                                    if self.match_iimm(op2) {
                                        let imm_op2 = self.node_iimm_to_i32(op2) as i8;

                                        // shl op1, op2 -> op1
                                        self.backend.emit_shl_r_imm8(&tmp_op1, imm_op2);

                                        // mov op1 -> result
                                        self.backend.emit_mov_r_r(&res_tmp, &tmp_op1);
                                    } else if self.match_ireg(op2) {
                                        let tmp_op2 = self.emit_ireg(op2, f_content, f_context, vm);

                                        // mov op2 -> cl
                                        self.backend.emit_mov_r_r(&x86_64::CL, &tmp_op2);

                                        // shl op1, cl -> op1
                                        self.backend.emit_shl_r_cl(&tmp_op1);

                                        // mov op1 -> result
                                        self.backend.emit_mov_r_r(&res_tmp, &tmp_op1);
                                    } else {
                                        panic!("unexpected op2 (not ireg not iimm): {}", op2);
                                    }
                                } else {
                                    panic!("unexpected op1 (not ireg not mem): {}", op1);
                                }
                            },
                            op::BinOp::Lshr => {
                                let op1 = &ops[op1];
                                let op2 = &ops[op2];

                                if self.match_mem(op1) {
                                    unimplemented!()
                                } else if self.match_ireg(op1) {
                                    let tmp_op1 = self.emit_ireg(op1, f_content, f_context, vm);

                                    if self.match_iimm(op2) {
                                        let imm_op2 = self.node_iimm_to_i32(op2) as i8;

                                        // shr op1, op2 -> op1
                                        self.backend.emit_shr_r_imm8(&tmp_op1, imm_op2);

                                        // mov op1 -> result
                                        self.backend.emit_mov_r_r(&res_tmp, &tmp_op1);
                                    } else if self.match_ireg(op2) {
                                        let tmp_op2 = self.emit_ireg(op2, f_content, f_context, vm);

                                        // mov op2 -> cl
                                        self.backend.emit_mov_r_r(&x86_64::CL, &tmp_op2);

                                        // shr op1, cl -> op1
                                        self.backend.emit_shr_r_cl(&tmp_op1);

                                        // mov op1 -> result
                                        self.backend.emit_mov_r_r(&res_tmp, &tmp_op1);
                                    } else {
                                        panic!("unexpected op2 (not ireg not iimm): {}", op2);
                                    }
                                } else {
                                    panic!("unexpected op1 (not ireg not mem): {}", op1);
                                }
                            },
                            op::BinOp::Ashr => {
                                let op1 = &ops[op1];
                                let op2 = &ops[op2];

                                if self.match_mem(op1) {
                                    unimplemented!()
                                } else if self.match_ireg(op1) {
                                    let tmp_op1 = self.emit_ireg(op1, f_content, f_context, vm);

                                    if self.match_iimm(op2) {
                                        let imm_op2 = self.node_iimm_to_i32(op2) as i8;

                                        // sar op1, op2 -> op1
                                        self.backend.emit_sar_r_imm8(&tmp_op1, imm_op2);

                                        // mov op1 -> result
                                        self.backend.emit_mov_r_r(&res_tmp, &tmp_op1);
                                    } else if self.match_ireg(op2) {
                                        let tmp_op2 = self.emit_ireg(op2, f_content, f_context, vm);

                                        // mov op2 -> cl
                                        self.backend.emit_mov_r_r(&x86_64::CL, &tmp_op2);

                                        // sar op1, cl -> op1
                                        self.backend.emit_sar_r_cl(&tmp_op1);

                                        // mov op1 -> result
                                        self.backend.emit_mov_r_r(&res_tmp, &tmp_op1);
                                    } else  {
                                        panic!("unexpected op2 (not ireg not iimm): {}", op2);
                                    }
                                } else {
                                    panic!("unexpected op1 (not ireg not mem): {}", op1);
                                }
                            },


                            // floating point
                            op::BinOp::FAdd => {
                                if self.match_fpreg(&ops[op1]) && self.match_mem(&ops[op2]) {
                                    trace!("emit add-fpreg-mem");

                                    let reg_op1 = self.emit_fpreg(&ops[op1], f_content, f_context, vm);
                                    let mem_op2 = self.emit_mem(&ops[op2], vm);

                                    // mov op1, res
                                    self.backend.emit_movsd_f64_f64(&res_tmp, &reg_op1);
                                    // sub op2 res
                                    self.backend.emit_addsd_f64_mem64(&res_tmp, &mem_op2);
                                } else if self.match_fpreg(&ops[op1]) && self.match_fpreg(&ops[op2]) {
                                    trace!("emit add-fpreg-fpreg");

                                    let reg_op1 = self.emit_fpreg(&ops[op1], f_content, f_context, vm);
                                    let reg_op2 = self.emit_fpreg(&ops[op2], f_content, f_context, vm);

                                    // movsd op1, res
                                    self.backend.emit_movsd_f64_f64(&res_tmp, &reg_op1);
                                    // add op2 res
                                    self.backend.emit_addsd_f64_f64(&res_tmp, &reg_op2);
                                } else {
                                    panic!("unexpected fadd: {}", node)
                                }
                            }

                            op::BinOp::FSub => {
                                if self.match_fpreg(&ops[op1]) && self.match_mem(&ops[op2]) {
                                    trace!("emit sub-fpreg-mem");

                                    let reg_op1 = self.emit_fpreg(&ops[op1], f_content, f_context, vm);
                                    let mem_op2 = self.emit_mem(&ops[op2], vm);

                                    // mov op1, res
                                    self.backend.emit_movsd_f64_f64(&res_tmp, &reg_op1);
                                    // sub op2 res
                                    self.backend.emit_subsd_f64_mem64(&res_tmp, &mem_op2);
                                } else if self.match_fpreg(&ops[op1]) && self.match_fpreg(&ops[op2]) {
                                    trace!("emit sub-fpreg-fpreg");

                                    let reg_op1 = self.emit_fpreg(&ops[op1], f_content, f_context, vm);
                                    let reg_op2 = self.emit_fpreg(&ops[op2], f_content, f_context, vm);

                                    // movsd op1, res
                                    self.backend.emit_movsd_f64_f64(&res_tmp, &reg_op1);
                                    // sub op2 res
                                    self.backend.emit_subsd_f64_f64(&res_tmp, &reg_op2);
                                } else {
                                    panic!("unexpected fsub: {}", node)
                                }
                            }

                            op::BinOp::FMul => {
                                if self.match_fpreg(&ops[op1]) && self.match_mem(&ops[op2]) {
                                    trace!("emit mul-fpreg-mem");

                                    let reg_op1 = self.emit_fpreg(&ops[op1], f_content, f_context, vm);
                                    let mem_op2 = self.emit_mem(&ops[op2], vm);

                                    // mov op1, res
                                    self.backend.emit_movsd_f64_f64(&res_tmp, &reg_op1);
                                    // mul op2 res
                                    self.backend.emit_mulsd_f64_mem64(&res_tmp, &mem_op2);
                                } else if self.match_fpreg(&ops[op1]) && self.match_fpreg(&ops[op2]) {
                                    trace!("emit mul-fpreg-fpreg");

                                    let reg_op1 = self.emit_fpreg(&ops[op1], f_content, f_context, vm);
                                    let reg_op2 = self.emit_fpreg(&ops[op2], f_content, f_context, vm);

                                    // movsd op1, res
                                    self.backend.emit_movsd_f64_f64(&res_tmp, &reg_op1);
                                    // mul op2 res
                                    self.backend.emit_mulsd_f64_f64(&res_tmp, &reg_op2);
                                } else {
                                    panic!("unexpected fmul: {}", node)
                                }
                            }

                            op::BinOp::FDiv => {
                                if self.match_fpreg(&ops[op1]) && self.match_mem(&ops[op2]) {
                                    trace!("emit div-fpreg-mem");

                                    let reg_op1 = self.emit_fpreg(&ops[op1], f_content, f_context, vm);
                                    let mem_op2 = self.emit_mem(&ops[op2], vm);

                                    // mov op1, res
                                    self.backend.emit_movsd_f64_f64(&res_tmp, &reg_op1);
                                    // div op2 res
                                    self.backend.emit_divsd_f64_mem64(&res_tmp, &mem_op2);
                                } else if self.match_fpreg(&ops[op1]) && self.match_fpreg(&ops[op2]) {
                                    trace!("emit div-fpreg-fpreg");

                                    let reg_op1 = self.emit_fpreg(&ops[op1], f_content, f_context, vm);
                                    let reg_op2 = self.emit_fpreg(&ops[op2], f_content, f_context, vm);

                                    // movsd op1, res
                                    self.backend.emit_movsd_f64_f64(&res_tmp, &reg_op1);
                                    // div op2 res
                                    self.backend.emit_divsd_f64_f64(&res_tmp, &reg_op2);
                                } else {
                                    panic!("unexpected fdiv: {}", node)
                                }
                            }

                            op::BinOp::FRem => {
                                if self.match_fpreg(&ops[op1]) && self.match_fpreg(&ops[op2]) {
                                    trace!("emit frem-fpreg-fpreg");

                                    let reg_op1 = self.emit_fpreg(&ops[op1], f_content, f_context, vm);
                                    let reg_op2 = self.emit_fpreg(&ops[op2], f_content, f_context, vm);

                                    let reg_tmp = self.get_result_value(node);

                                    self.emit_runtime_entry(&entrypoints::FREM, vec![reg_op1.clone(), reg_op2.clone()], Some(vec![reg_tmp.clone()]), Some(node), f_content, f_context, vm);
                                } else {
                                    panic!("unexpected fdiv: {}", node)
                                }
                            }
                        }
                    }

                    Instruction_::ConvOp{operation, ref from_ty, ref to_ty, operand} => {
                        trace!("instsel on CONVOP");

                        let ops = inst.ops.read().unwrap();

                        let ref op = ops[operand];

                        match operation {
                            op::ConvOp::TRUNC => {
                                if self.match_ireg(op) {
                                    let tmp_op = self.emit_ireg(op, f_content, f_context, vm);
                                    let tmp_res = self.get_result_value(node);

                                    // mov op -> result
                                    self.backend.emit_mov_r_r(&tmp_res, &tmp_op);
                                } else {
                                    panic!("unexpected op (expect ireg): {}", op);
                                }
                            }
                            op::ConvOp::ZEXT => {
                                if self.match_ireg(op) {
                                    let tmp_op = self.emit_ireg(op, f_content, f_context, vm);
                                    let tmp_res = self.get_result_value(node);

                                    // movz op -> result
                                    let from_ty_size = vm.get_backend_type_info(from_ty.id()).size;
                                    let to_ty_size   = vm.get_backend_type_info(to_ty.id()).size;

                                    if from_ty_size != to_ty_size {
                                        if from_ty_size == 4 && to_ty_size == 8 {
                                            // zero extend from 32 bits to 64 bits is a mov instruction
                                            // x86 does not have movzlq (32 to 64)

                                            // tmp_op is int32, but tmp_res is int64
                                            // we want to force a 32-to-32 mov, so high bits of the destination will be zeroed

                                            let tmp_res32 = unsafe {tmp_res.as_type(UINT32_TYPE.clone())};

                                            self.backend.emit_mov_r_r(&tmp_res32, &tmp_op);
                                        } else {
                                            self.backend.emit_movz_r_r(&tmp_res, &tmp_op);
                                        }
                                    } else {
                                        self.backend.emit_mov_r_r(&tmp_res, &tmp_op);
                                    }
                                } else {
                                    panic!("unexpected op (expect ireg): {}", op);
                                }
                            },
                            op::ConvOp::SEXT => {
                                if self.match_ireg(op) {
                                    let tmp_op = self.emit_ireg(op, f_content, f_context, vm);
                                    let tmp_res = self.get_result_value(node);

                                    // movs op -> result
                                    let from_ty_size = vm.get_backend_type_info(from_ty.id()).size;
                                    let to_ty_size   = vm.get_backend_type_info(to_ty.id()).size;

                                    if from_ty_size != to_ty_size {
                                        self.backend.emit_movs_r_r(&tmp_res, &tmp_op);
                                    } else {
                                        self.backend.emit_mov_r_r(&tmp_res, &tmp_op);
                                    }
                                } else {
                                    panic!("unexpected op (expect ireg): {}", op)
                                }
                            }
                            op::ConvOp::REFCAST | op::ConvOp::PTRCAST => {
                                // just a mov (and hopefully reg alloc will coalesce it)
                                let tmp_res = self.get_result_value(node);

                                if self.match_ireg(op) {
                                    let tmp_op = self.emit_ireg(op, f_content, f_context, vm);
                                    self.backend.emit_mov_r_r(&tmp_res, &tmp_op);
                                } else {
                                    panic!("unexpected op (expect ireg): {}", op)
                                }
                            }
                            op::ConvOp::SITOFP => {
                                let tmp_res = self.get_result_value(node);

                                if self.match_ireg(op) {
                                    let tmp_op = self.emit_ireg(op, f_content, f_context, vm);
                                    self.backend.emit_cvtsi2sd_f64_r(&tmp_res, &tmp_op);
                                } else {
                                    panic!("unexpected op (expected ireg): {}", op)
                                }
                            }
                            op::ConvOp::FPTOSI => {
                                let tmp_res = self.get_result_value(node);

                                if self.match_fpreg(op) {
                                    let tmp_op = self.emit_fpreg(op, f_content, f_context, vm);
                                    self.backend.emit_cvtsd2si_r_f64(&tmp_res, &tmp_op);
                                } else {
                                    panic!("unexpected op (expected fpreg): {}", op)
                                }
                            }
                            op::ConvOp::UITOFP => {
                                let tmp_res = self.get_result_value(node);

                                if self.match_ireg(op) {
                                    let tmp_op = self.emit_ireg(op, f_content, f_context, vm);

                                    let op_ty_size = vm.get_backend_type_info(tmp_op.ty.id()).size;

                                    match op_ty_size {
                                        8 => {
                                            // movd/movq op -> res
                                            self.backend.emit_mov_fpr_r64(&tmp_res, &tmp_op);

                                            // punpckldq UITOFP_C0, tmp_res -> tmp_res
                                            // (interleaving low bytes: xmm = xmm[0] mem[0] xmm[1] mem[1]
                                            let mem_c0 = self.get_mem_for_const(UITOFP_C0.clone(), vm);
                                            self.backend.emit_punpckldq_f64_mem128(&tmp_res, &mem_c0);

                                            // subpd UITOFP_C1, tmp_res -> tmp_res
                                            let mem_c1 = self.get_mem_for_const(UITOFP_C1.clone(), vm);
                                            self.backend.emit_subpd_f64_mem128(&tmp_res, &mem_c1);

                                            // haddpd tmp_res, tmp_res -> tmp_res
                                            self.backend.emit_haddpd_f64_f64(&tmp_res, &tmp_res);
                                        }
                                        4 => {
                                            let tmp = self.make_temporary(f_context, UINT64_TYPE.clone(), vm);

                                            // movl op -> tmp(32)
                                            let tmp32 = unsafe {tmp.as_type(UINT32_TYPE.clone())};
                                            self.backend.emit_mov_r_r(&tmp32, &tmp_op);

                                            // cvtsi2sd %tmp(64) -> %tmp_res
                                            self.backend.emit_cvtsi2sd_f64_r(&tmp_res, &tmp);
                                        }
                                        2 | 1 => {
                                            let tmp_op32 = unsafe {tmp_op.as_type(UINT32_TYPE.clone())};
                                            self.backend.emit_cvtsi2sd_f64_r(&tmp_res, &tmp_op32);
                                        }
                                        _ => panic!("not implemented int length {}", op_ty_size)
                                    }
                                } else {
                                    panic!("unexpected op (expected ireg): {}", op)
                                }
                            }
                            op::ConvOp::FPTOUI => {
                                let tmp_res = self.get_result_value(node);

                                let res_ty_size = vm.get_backend_type_info(tmp_res.ty.id()).size;

                                if self.match_fpreg(op) {
                                    let tmp_op = self.emit_fpreg(op, f_content, f_context, vm);

                                    match res_ty_size {
                                        8 => {
                                            let tmp1 = self.make_temporary(f_context, DOUBLE_TYPE.clone(), vm);
                                            let tmp2 = self.make_temporary(f_context, DOUBLE_TYPE.clone(), vm);

                                            // movsd FPTOUI_C -> %tmp1
                                            let mem_c = self.get_mem_for_const(FPTOUI_C.clone(), vm);
                                            self.backend.emit_movsd_f64_mem64(&tmp1, &mem_c);

                                            // movapd %tmp_op -> %tmp2
                                            self.backend.emit_movapd_f64_f64(&tmp2, &tmp_op);

                                            // subsd %tmp1, %tmp2 -> %tmp2
                                            self.backend.emit_subsd_f64_f64(&tmp2, &tmp1);

                                            // cvttsd2si %tmp2 -> %tmp_res
                                            self.backend.emit_cvttsd2si_r_f64(&tmp_res, &tmp2);

                                            let tmp_const = self.make_temporary(f_context, UINT64_TYPE.clone(), vm);
                                            // mov 0x8000000000000000 -> %tmp_const
                                            self.backend.emit_mov_r64_imm64(&tmp_const, -9223372036854775808i64);

                                            // xor %tmp_res, %tmp_const -> %tmp_const
                                            self.backend.emit_xor_r_r(&tmp_const, &tmp_res);

                                            // cvttsd2si %tmp_op -> %tmp_res
                                            self.backend.emit_cvttsd2si_r_f64(&tmp_res, &tmp_op);

                                            // ucomisd %tmp_op %tmp1
                                            self.backend.emit_ucomisd_f64_f64(&tmp1, &tmp_op);

                                            // cmovaeq %tmp_const -> %tmp_res
                                            self.backend.emit_cmovae_r_r(&tmp_res, &tmp_const);
                                        }
                                        4 => {
                                            let tmp_res64 = unsafe {tmp_res.as_type(UINT64_TYPE.clone())};

                                            // cvttsd2si %tmp_op -> %tmp_res(64)
                                            self.backend.emit_cvttsd2si_r_f64(&tmp_res64, &tmp_op);
                                        }
                                        2 | 1 => {
                                            let tmp_res32 = unsafe {tmp_res.as_type(UINT32_TYPE.clone())};

                                            // cvttsd2si %tmp_op -> %tmp_res(32)
                                            self.backend.emit_cvttsd2si_r_f64(&tmp_res32, &tmp_op);

                                            // movz %tmp_res -> %tmp_res(32)
                                            self.backend.emit_movz_r_r(&tmp_res32, &tmp_res);
                                        }
                                        _ => panic!("not implemented int length {}", res_ty_size)
                                    }
                                } else {
                                    panic!("unexpected op (expected ireg): {}", op)
                                }
                            }
                            _ => unimplemented!()
                        }
                    }
                    
                    // load on x64 generates mov inst (no matter what order is specified)
                    // https://www.cl.cam.ac.uk/~pes20/cpp/cpp0xmappings.html
                    Instruction_::Load{is_ptr, order, mem_loc} => {
                        trace!("instsel on LOAD");

                        let ops = inst.ops.read().unwrap();
                        let ref loc_op = ops[mem_loc];
                        
                        // check order
                        match order {
                            MemoryOrder::Relaxed 
                            | MemoryOrder::Consume 
                            | MemoryOrder::Acquire
                            | MemoryOrder::SeqCst
                            | MemoryOrder::NotAtomic => {},
                            _ => panic!("didnt expect order {:?} with store inst", order)
                        }                        

                        let resolved_loc = self.emit_node_addr_to_value(loc_op, f_content, f_context, vm);
                        let res_temp = self.get_result_value(node);
                        
                        if self.match_ireg(node) {
                            // emit mov(GPR)
                            self.backend.emit_mov_r_mem(&res_temp, &resolved_loc);
                        } else {
                            // emit mov(FPR)
                            self.backend.emit_movsd_f64_mem64(&res_temp, &resolved_loc);
                        }
                    }
                    
                    Instruction_::Store{is_ptr, order, mem_loc, value} => {
                        trace!("instsel on STORE");

                        let ops = inst.ops.read().unwrap();
                        let ref loc_op = ops[mem_loc];
                        let ref val_op = ops[value];
                        
                        let generate_plain_mov : bool = {
                            match order {
                                MemoryOrder::Relaxed
                                | MemoryOrder::Release
                                | MemoryOrder::NotAtomic => true,
                                MemoryOrder::SeqCst => false,
                                _ => panic!("didnt expect order {:?} with store inst", order)
                            }
                        };
                        
                        let resolved_loc = self.emit_node_addr_to_value(loc_op, f_content, f_context, vm);

                        if self.match_iimm(val_op) {
                            let val = self.node_iimm_to_i32(val_op);
                            if generate_plain_mov {
                                self.backend.emit_mov_mem_imm(&resolved_loc, val);
                            } else {
                                unimplemented!()
                            }
                        } else if self.match_ireg(val_op) {
                            let val = self.emit_ireg(val_op, f_content, f_context, vm);
                            if generate_plain_mov {
                                self.backend.emit_mov_mem_r(&resolved_loc, &val);
                            } else {
                                unimplemented!()
                            }
                        } else {
                            let val = self.emit_fpreg(val_op, f_content, f_context, vm);

                            // emit mov(FPR)
                            self.backend.emit_movsd_mem64_f64(&resolved_loc, &val);
                        }
                    }

                    // memory insts: calculate the address, then lea
                    Instruction_::GetIRef(_)
                    | Instruction_::GetFieldIRef{..}
                    | Instruction_::GetVarPartIRef{..}
                    | Instruction_::ShiftIRef{..} => {
                        trace!("instsel on GET/FIELD/VARPARTIREF, SHIFTIREF");

                        let mem_addr = self.emit_get_mem_from_inst(node, f_content, f_context, vm);
                        let tmp_res  = self.get_result_value(node);

                        self.backend.emit_lea_r64(&tmp_res, &mem_addr);
                    }
                    
                    Instruction_::ThreadExit => {
                        trace!("instsel on THREADEXIT");
                        // emit a call to swap_back_to_native_stack(sp_loc: Address)
                        
                        // get thread local and add offset to get sp_loc
                        let tl = self.emit_get_threadlocal(Some(node), f_content, f_context, vm);
                        self.backend.emit_add_r_imm(&tl, *thread::NATIVE_SP_LOC_OFFSET as i32);
                        
                        self.emit_runtime_entry(&entrypoints::SWAP_BACK_TO_NATIVE_STACK, vec![tl.clone()], None, Some(node), f_content, f_context, vm);
                    }

                    Instruction_::CommonInst_GetThreadLocal => {
                        trace!("instsel on GETTHREADLOCAL");
                        // get thread local
                        let tl = self.emit_get_threadlocal(Some(node), f_content, f_context, vm);

                        let tmp_res = self.get_result_value(node);

                        // load [tl + USER_TLS_OFFSET] -> tmp_res
                        self.emit_load_base_offset(&tmp_res, &tl, *thread::USER_TLS_OFFSET as i32, vm);
                    }
                    Instruction_::CommonInst_SetThreadLocal(op) => {
                        trace!("instsel on SETTHREADLOCAL");

                        let ops = inst.ops.read().unwrap();
                        let ref op = ops[op];

                        debug_assert!(self.match_ireg(op));

                        let tmp_op = self.emit_ireg(op, f_content, f_context, vm);

                        // get thread local
                        let tl = self.emit_get_threadlocal(Some(node), f_content, f_context, vm);

                        // store tmp_op -> [tl + USER_TLS_OFFSTE]
                        self.emit_store_base_offset(&tl, *thread::USER_TLS_OFFSET as i32, &tmp_op, vm);
                    }

                    Instruction_::CommonInst_Pin(op) => {
                        trace!("instsel on PIN");

                        if !mm::GC_MOVES_OBJECT {
                            // non-moving GC: pin is a nop (move from op to result)
                            let ops = inst.ops.read().unwrap();
                            let ref op = ops[op];

                            let tmp_res = self.get_result_value(node);

                            self.emit_move_node_to_value(&tmp_res, op, f_content, f_context, vm);
                        } else {
                            unimplemented!()
                        }
                    }
                    Instruction_::CommonInst_Unpin(_) => {
                        trace!("instsel on UNPIN");

                        if !mm::GC_MOVES_OBJECT {
                            // do nothing
                        } else {
                            unimplemented!()
                        }
                    }


                    Instruction_::Move(op) => {
                        trace!("instsel on MOVE (internal IR)");

                        let ops = inst.ops.read().unwrap();
                        let ref op = ops[op];

                        let tmp_res = self.get_result_value(node);

                        self.emit_move_node_to_value(&tmp_res, op, f_content, f_context, vm);
                    }
                    
                    Instruction_::New(ref ty) => {
                        trace!("instsel on NEW");

                        if cfg!(debug_assertions) {
                            match ty.v {
                                MuType_::Hybrid(_) => panic!("cannot use NEW for hybrid, use NEWHYBRID instead"),
                                _ => {}
                            }
                        }

                        let ty_info = vm.get_backend_type_info(ty.id());
                        let size = ty_info.size;
                        let ty_align= ty_info.alignment;

                        let const_size = self.make_value_int_const(size as u64, vm);

                        let tmp_allocator = self.emit_get_allocator(node, f_content, f_context, vm);
                        let tmp_res = self.emit_alloc_sequence(tmp_allocator.clone(), const_size, ty_align, node, f_content, f_context, vm);

                        // ASM: call muentry_init_object(%allocator, %tmp_res, %encode)
                        let encode = self.make_value_int_const(mm::get_gc_type_encode(ty_info.gc_type.id), vm);
                        self.emit_runtime_entry(
                            &entrypoints::INIT_OBJ,
                            vec![tmp_allocator.clone(), tmp_res.clone(), encode],
                            None,
                            Some(node), f_content, f_context, vm
                        );
                    }

                    Instruction_::NewHybrid(ref ty, var_len) => {
                        trace!("instsel on NEWHYBRID");

                        if cfg!(debug_assertions) {
                            match ty.v {
                                MuType_::Hybrid(_) => {},
                                _ => panic!("NEWHYBRID is only for allocating hybrid types, use NEW for others")
                            }
                        }

                        let ty_info = vm.get_backend_type_info(ty.id());
                        let ty_align = ty_info.alignment;
                        let fix_part_size = ty_info.size;
                        let var_ty_size = match ty.v {
                            MuType_::Hybrid(ref name) => {
                                let map_lock = HYBRID_TAG_MAP.read().unwrap();
                                let hybrid_ty_ = map_lock.get(name).unwrap();
                                let var_ty = hybrid_ty_.get_var_ty();

                                vm.get_backend_type_info(var_ty.id()).size
                            },
                            _ => panic!("only expect HYBRID type here")
                        };

                        // actual size = fix_part_size + var_ty_size * len
                        let (actual_size, length) = {
                            let ops = inst.ops.read().unwrap();
                            let ref var_len = ops[var_len];

                            if self.match_iimm(var_len) {
                                let var_len = self.node_iimm_to_i32(var_len);
                                let actual_size = fix_part_size + var_ty_size * (var_len as usize);

                                (
                                    self.make_value_int_const(actual_size as u64, vm),
                                    self.make_value_int_const(var_len as u64, vm)
                                )
                            } else {
                                let tmp_actual_size = self.make_temporary(f_context, UINT64_TYPE.clone(), vm);
                                let tmp_var_len = self.emit_ireg(var_len, f_content, f_context, vm);

                                let is_power_of_two = |x: usize| {
                                    use std::i8;

                                    let mut power_of_two = 1;
                                    let mut i: i8 = 0;
                                    while power_of_two < x && i < i8::MAX {
                                        power_of_two *= 2;
                                        i += 1;
                                    }

                                    if power_of_two == x {
                                        Some(i)
                                    } else {
                                        None
                                    }
                                };

                                match is_power_of_two(var_ty_size) {
                                    Some(shift) => {
                                        // a shift-left will get the total size of var part
                                        self.backend.emit_shl_r_imm8(&tmp_var_len, shift);

                                        // add with fix-part size
                                        self.backend.emit_add_r_imm(&tmp_var_len, fix_part_size as i32);

                                        // mov result to tmp_actual_size
                                        self.backend.emit_mov_r_r(&tmp_actual_size, &tmp_var_len);
                                    }
                                    None => {
                                        // we need to do a multiply

                                        // mov var_ty_size -> rax
                                        self.backend.emit_mov_r_imm(&x86_64::RAX, var_ty_size as i32);

                                        // mul tmp_var_len, rax -> rdx:rax
                                        self.backend.emit_mul_r(&tmp_var_len);

                                        // add with fix-part size
                                        self.backend.emit_add_r_imm(&x86_64::RAX, fix_part_size as i32);

                                        // mov result to tmp_actual_size
                                        self.backend.emit_mov_r_r(&tmp_actual_size, &x86_64::RAX);
                                    }
                                }

                                (tmp_actual_size, tmp_var_len)
                            }
                        };

                        let tmp_allocator = self.emit_get_allocator(node, f_content, f_context, vm);
                        let tmp_res = self.emit_alloc_sequence(tmp_allocator.clone(), actual_size, ty_align, node, f_content, f_context, vm);

                        // ASM: call muentry_init_object(%allocator, %tmp_res, %encode)
                        let encode = self.make_value_int_const(mm::get_gc_type_encode(ty_info.gc_type.id), vm);
                        self.emit_runtime_entry(
                            &entrypoints::INIT_HYBRID,
                            vec![tmp_allocator.clone(), tmp_res.clone(), encode, length],
                            None,
                            Some(node), f_content, f_context, vm
                        );
                    }
                    
                    Instruction_::Throw(op_index) => {
                        trace!("instsel on THROW");

                        let ops = inst.ops.read().unwrap();
                        let ref exception_obj = ops[op_index];
                        
                        self.emit_runtime_entry(
                            &entrypoints::THROW_EXCEPTION, 
                            vec![exception_obj.clone_value()], 
                            None,
                            Some(node), f_content, f_context, vm);
                    }
    
                    _ => unimplemented!()
                } // main switch
            },
            
            TreeNode_::Value(ref p) => {
        
            }
        }
    }
    
    fn make_temporary(&mut self, f_context: &mut FunctionContext, ty: P<MuType>, vm: &VM) -> P<Value> {
        f_context.make_temporary(vm.next_id(), ty).clone_value()
    }
    
    fn make_memory_op_base_offset (&mut self, base: &P<Value>, offset: i32, ty: P<MuType>, vm: &VM) -> P<Value> {
        P(Value{
            hdr: MuEntityHeader::unnamed(vm.next_id()),
            ty: ty.clone(),
            v: Value_::Memory(MemoryLocation::Address{
                base: base.clone(),
                offset: Some(self.make_value_int_const(offset as u64, vm)),
                index: None,
                scale: None
            })
        })
    }

    fn make_memory_op_base_index(&mut self, base: &P<Value>, index: &P<Value>, scale: u8, ty: P<MuType>, vm: &VM) -> P<Value> {
        P(Value{
            hdr: MuEntityHeader::unnamed(vm.next_id()),
            ty: ty.clone(),
            v: Value_::Memory(MemoryLocation::Address{
                base: base.clone(),
                offset: None,
                index: Some(index.clone()),
                scale: Some(scale)
            })
        })
    }
    
    fn make_value_int_const (&mut self, val: u64, vm: &VM) -> P<Value> {
        P(Value{
            hdr: MuEntityHeader::unnamed(vm.next_id()),
            ty: UINT64_TYPE.clone(),
            v: Value_::Constant(Constant::Int(val))
        })
    }

    fn emit_alloc_sequence (&mut self, tmp_allocator: P<Value>, size: P<Value>, align: usize, node: &TreeNode, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) -> P<Value> {
        if size.is_int_const() {
            // size known at compile time, we can choose to emit alloc_small or large now
            if size.extract_int_const() > mm::LARGE_OBJECT_THRESHOLD as u64 {
                self.emit_alloc_sequence_large(tmp_allocator, size, align, node, f_content, f_context, vm)
            } else {
                self.emit_alloc_sequence_small(tmp_allocator, size, align, node, f_content, f_context, vm)
            }
        } else {
            // size is unknown at compile time
            // we need to emit both alloc small and alloc large,
            // and it is decided at runtime

            // emit: cmp size, THRESHOLD
            // emit: jg ALLOC_LARGE
            // emit: >> small object alloc
            // emit: jmp ALLOC_LARGE_END
            // emit: ALLOC_LARGE:
            // emit: >> large object alloc
            // emit: ALLOC_LARGE_END:
            let blk_alloc_large = format!("{}_alloc_large", node.id());
            let blk_alloc_large_end = format!("{}_alloc_large_end", node.id());

            self.backend.emit_cmp_imm_r(mm::LARGE_OBJECT_THRESHOLD as i32, &size);
            self.backend.emit_jg(blk_alloc_large.clone());

            self.finish_block();
            self.start_block(format!("{}_allocsmall", node.id()));

            // alloc small here
            let tmp_res = self.emit_alloc_sequence_small(tmp_allocator.clone(), size.clone(), align, node, f_content, f_context, vm);

            self.backend.emit_jmp(blk_alloc_large_end.clone());

            // finishing current block
            let cur_block = self.current_block.as_ref().unwrap().clone();
            self.backend.end_block(cur_block.clone());
            self.backend.set_block_liveout(cur_block.clone(), &vec![tmp_res.clone()]);

            // alloc_large:
            self.current_block = Some(blk_alloc_large.clone());
            self.backend.start_block(blk_alloc_large.clone());
            self.backend.set_block_livein(blk_alloc_large.clone(), &vec![size.clone()]);

            let tmp_res = self.emit_alloc_sequence_large(tmp_allocator.clone(), size, align, node, f_content, f_context, vm);

            self.backend.end_block(blk_alloc_large.clone());
            self.backend.set_block_liveout(blk_alloc_large.clone(), &vec![tmp_res.clone()]);

            // alloc_large_end:
            self.backend.start_block(blk_alloc_large_end.clone());
            self.current_block = Some(blk_alloc_large_end.clone());

            tmp_res
        }
    }

    fn emit_get_allocator (&mut self, node: &TreeNode, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) -> P<Value> {
        // ASM: %tl = get_thread_local()
        let tmp_tl = self.emit_get_threadlocal(Some(node), f_content, f_context, vm);

        // ASM: lea [%tl + allocator_offset] -> %tmp_allocator
        let allocator_offset = *thread::ALLOCATOR_OFFSET;
        let tmp_allocator = self.make_temporary(f_context, ADDRESS_TYPE.clone(), vm);
        self.emit_lea_base_immoffset(&tmp_allocator, &tmp_tl, allocator_offset as i32, vm);

        tmp_allocator
    }

    fn emit_alloc_sequence_large (&mut self, tmp_allocator: P<Value>, size: P<Value>, align: usize, node: &TreeNode, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) -> P<Value> {
        let tmp_res = self.get_result_value(node);

        // ASM: %tmp_res = call muentry_alloc_large(%allocator, size, align)
        let const_align = self.make_value_int_const(align as u64, vm);

        self.emit_runtime_entry(
            &entrypoints::ALLOC_LARGE,
            vec![tmp_allocator.clone(), size.clone(), const_align],
            Some(vec![tmp_res.clone()]),
            Some(node), f_content, f_context, vm
        );

        tmp_res
    }

    fn emit_alloc_sequence_small (&mut self, tmp_allocator: P<Value>, size: P<Value>, align: usize, node: &TreeNode, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) -> P<Value> {
        // emit immix allocation fast path

        // ASM: %tl = get_thread_local()
        let tmp_tl = self.emit_get_threadlocal(Some(node), f_content, f_context, vm);

        // ASM: mov [%tl + allocator_offset + cursor_offset] -> %cursor
        let cursor_offset = *thread::ALLOCATOR_OFFSET + *mm::ALLOCATOR_CURSOR_OFFSET;
        let tmp_cursor = self.make_temporary(f_context, ADDRESS_TYPE.clone(), vm);
        self.emit_load_base_offset(&tmp_cursor, &tmp_tl, cursor_offset as i32, vm);

        // alignup cursor (cursor + align - 1 & !(align - 1))
        // ASM: lea align-1(%cursor) -> %start
        let align = align as i32;
        let tmp_start = self.make_temporary(f_context, ADDRESS_TYPE.clone(), vm);
        self.emit_lea_base_immoffset(&tmp_start, &tmp_cursor, align - 1, vm);
        // ASM: and %start, !(align-1) -> %start
        self.backend.emit_and_r_imm(&tmp_start, !(align - 1) as i32);

        // bump cursor
        // ASM: add %size, %start -> %end
        // or lea size(%start) -> %end
        let tmp_end = self.make_temporary(f_context, ADDRESS_TYPE.clone(), vm);
        if size.is_int_const() {
            let offset = size.extract_int_const() as i32;
            self.emit_lea_base_immoffset(&tmp_end, &tmp_start, offset, vm);
        } else {
            self.backend.emit_mov_r_r(&tmp_end, &tmp_start);
            self.backend.emit_add_r_r(&tmp_end, &size);
        }

        // check with limit
        // ASM: cmp %end, [%tl + allocator_offset + limit_offset]
        let limit_offset = *thread::ALLOCATOR_OFFSET + *mm::ALLOCATOR_LIMIT_OFFSET;
        let mem_limit = self.make_memory_op_base_offset(&tmp_tl, limit_offset as i32, ADDRESS_TYPE.clone(), vm);
        self.backend.emit_cmp_mem_r(&mem_limit, &tmp_end);

        // branch to slow path if end > limit (end - limit > 0)
        // ASM: jg alloc_slow
        let slowpath = format!("{}_allocslow", node.id());
        self.backend.emit_jg(slowpath.clone());

        // finish current block
        self.finish_block();
        self.start_block(format!("{}_updatecursor", node.id()));

        // update cursor
        // ASM: mov %end -> [%tl + allocator_offset + cursor_offset]
        self.emit_store_base_offset(&tmp_tl, cursor_offset as i32, &tmp_end, vm);

        // put start as result
        // ASM: mov %start -> %result
        let tmp_res = self.get_result_value(node);
        self.backend.emit_mov_r_r(&tmp_res, &tmp_start);

        // ASM jmp alloc_end
        let allocend = format!("{}_alloc_small_end", node.id());
        self.backend.emit_jmp(allocend.clone());

        // finishing current block
        let cur_block = self.current_block.as_ref().unwrap().clone();
        self.backend.end_block(cur_block.clone());
        self.backend.set_block_liveout(cur_block.clone(), &vec![tmp_res.clone()]);

        // alloc_slow:
        // call alloc_slow(size, align) -> %ret
        // new block (no livein)
        self.current_block = Some(slowpath.clone());
        self.backend.start_block(slowpath.clone());
        self.backend.set_block_livein(slowpath.clone(), &vec![size.clone()]);

        // arg1: allocator address
        // arg2: size
        // arg3: align
        let const_align= self.make_value_int_const(align as u64, vm);

        self.emit_runtime_entry(
            &entrypoints::ALLOC_SLOW,
            vec![tmp_allocator.clone(), size.clone(), const_align],
            Some(vec![
            tmp_res.clone()
            ]),
            Some(node), f_content, f_context, vm
        );

        // end block (no liveout other than result)
        self.backend.end_block(slowpath.clone());
        self.backend.set_block_liveout(slowpath.clone(), &vec![tmp_res.clone()]);

        // block: alloc_end
        self.backend.start_block(allocend.clone());
        self.current_block = Some(allocend.clone());

        tmp_res
    }

    fn emit_load_base_offset (&mut self, dest: &P<Value>, base: &P<Value>, offset: i32, vm: &VM) {
        let mem = self.make_memory_op_base_offset(base, offset, dest.ty.clone(), vm);

        if dest.is_int_reg() {
            self.backend.emit_mov_r_mem(dest, &mem);
        } else if dest.is_fp_reg() {
            self.backend.emit_movsd_f64_mem64(dest, &mem);
        } else {
            unimplemented!();
        }
    }
    
    fn emit_store_base_offset (&mut self, base: &P<Value>, offset: i32, src: &P<Value>, vm: &VM) {
        let mem = self.make_memory_op_base_offset(base, offset, src.ty.clone(), vm);
        
        self.backend.emit_mov_mem_r(&mem, src);
    }
    
    fn emit_lea_base_immoffset(&mut self, dest: &P<Value>, base: &P<Value>, offset: i32, vm: &VM) {
        let mem = self.make_memory_op_base_offset(base, offset, ADDRESS_TYPE.clone(), vm);
        
        self.backend.emit_lea_r64(dest, &mem);
    }

    fn emit_udiv (
        &mut self,
        op1: &TreeNode, op2: &TreeNode,
        f_content: &FunctionContent,
        f_context: &mut FunctionContext,
        vm: &VM)
    {
        debug_assert!(self.match_ireg(op1));
        let reg_op1 = self.emit_ireg(op1, f_content, f_context, vm);

        match reg_op1.ty.get_int_length() {
            Some(64) => {
                // div uses RDX and RAX
                self.backend.emit_mov_r_r(&x86_64::RAX, &reg_op1);

                // xorq rdx, rdx -> rdx
                self.backend.emit_xor_r_r(&x86_64::RDX, &x86_64::RDX);
            }
            Some(32) => {
                // div uses edx, eax
                self.backend.emit_mov_r_r(&x86_64::EAX, &reg_op1);

                // xor edx edx
                self.backend.emit_xor_r_r(&x86_64::EDX, &x86_64::EDX);
            }
            Some(16) => {
                // div uses dx, ax
                self.backend.emit_mov_r_r(&x86_64::AX, &reg_op1);

                // xor dx, dx
                self.backend.emit_xor_r_r(&x86_64::DX, &x86_64::DX);
            },
            Some(8) => {
                // div uses AX
                self.backend.emit_mov_r_r(&x86_64::AL, &reg_op1);
            }
            _ => unimplemented!()
        }

        // div op2
        if self.match_mem(op2) {
            let mem_op2 = self.emit_mem(op2, vm);

            self.backend.emit_div_mem(&mem_op2);
        } else if self.match_iimm(op2) {
            let imm = self.node_iimm_to_i32(op2);
            // moving to a temp
            let temp = self.make_temporary(f_context, reg_op1.ty.clone(), vm);
            self.backend.emit_mov_r_imm(&temp, imm);

            // div tmp
            self.backend.emit_div_r(&temp);
        } else if self.match_ireg(op2) {
            let reg_op2 = self.emit_ireg(op2, f_content, f_context, vm);

            self.backend.emit_div_r(&reg_op2);
        } else {
            unimplemented!();
        }
    }

    fn emit_idiv (
        &mut self,
        op1: &TreeNode, op2: &TreeNode,
        f_content: &FunctionContent,
        f_context: &mut FunctionContext,
        vm: &VM)
    {
        debug_assert!(self.match_ireg(op1));
        let reg_op1 = self.emit_ireg(op1, f_content, f_context, vm);

        match reg_op1.ty.get_int_length() {
            Some(64) => {
                // idiv uses RDX and RAX
                self.backend.emit_mov_r_r(&x86_64::RAX, &reg_op1);

                // cqo: sign extend rax to rdx:rax
                self.backend.emit_cqo();
            }
            Some(32) => {
                // idiv uses edx, eax
                self.backend.emit_mov_r_r(&x86_64::EAX, &reg_op1);

                // cdq: sign extend eax to edx:eax
                self.backend.emit_cdq();
            }
            Some(16) => {
                // idiv uses dx, ax
                self.backend.emit_mov_r_r(&x86_64::AX, &reg_op1);

                // cwd: sign extend ax to dx:ax
                self.backend.emit_cwd();
            },
            Some(8) => {
                // idiv uses AL
                self.backend.emit_mov_r_r(&x86_64::AL, &reg_op1);

                // sign extend al to ax
                self.backend.emit_movs_r_r(&x86_64::AX, &x86_64::AL);
            }
            _ => unimplemented!()
        }

        // idiv op2
        if self.match_mem(op2) {
            let mem_op2 = self.emit_mem(op2, vm);

            self.backend.emit_idiv_mem(&mem_op2);
        } else if self.match_iimm(op2) {
            let imm = self.node_iimm_to_i32(op2);
            // moving to a temp
            let temp = self.make_temporary(f_context, reg_op1.ty.clone(), vm);
            self.backend.emit_mov_r_imm(&temp, imm);

            // idiv temp
            self.backend.emit_idiv_r(&temp);
        } else if self.match_ireg(op2) {
            let reg_op2 = self.emit_ireg(op2, f_content, f_context, vm);

            self.backend.emit_idiv_r(&reg_op2);
        } else {
            unimplemented!();
        }
    }
    
    fn emit_get_threadlocal (
        &mut self, 
        cur_node: Option<&TreeNode>,
        f_content: &FunctionContent, 
        f_context: &mut FunctionContext, 
        vm: &VM) -> P<Value> {
        let mut rets = self.emit_runtime_entry(&entrypoints::GET_THREAD_LOCAL, vec![], None, cur_node, f_content, f_context, vm);
        
        rets.pop().unwrap()
    }
    
    // ret: Option<Vec<P<Value>>
    // if ret is Some, return values will put stored in given temporaries
    // otherwise create temporaries
    // always returns result temporaries (given or created)
    fn emit_runtime_entry (
        &mut self, 
        entry: &RuntimeEntrypoint, 
        args: Vec<P<Value>>, 
        rets: Option<Vec<P<Value>>>,
        cur_node: Option<&TreeNode>, 
        f_content: &FunctionContent, 
        f_context: &mut FunctionContext, 
        vm: &VM) -> Vec<P<Value>> {
        let sig = entry.sig.clone();
        
        let entry_name = {
            if vm.is_running() {
                unimplemented!()
            } else {
                let ref entry_loc = entry.aot;
                
                match entry_loc {
                    &ValueLocation::Relocatable(_, ref name) => name.clone(),
                    _ => panic!("expecting a relocatable value")
                }
            }
        };
        
        self.emit_c_call_internal(entry_name, sig, args, rets, cur_node, f_content, f_context, vm)
    }
    
    // returns the stack arg offset - we will need this to collapse stack after the call
    fn emit_precall_convention(
        &mut self,
        args: &Vec<P<Value>>, 
        vm: &VM) -> usize {
        // if we need to save caller saved regs
        // put it here (since this is fastpath compile, we wont have them)
        
        // put args into registers if we can
        // in the meantime record args that do not fit in registers
        let mut stack_args : Vec<P<Value>> = vec![];        
        let mut gpr_arg_count = 0;
        for arg in args.iter() {
            if arg.is_int_reg() {
                if gpr_arg_count < x86_64::ARGUMENT_GPRs.len() {
                    let arg_gpr = {
                        let ref reg64 = x86_64::ARGUMENT_GPRs[gpr_arg_count];
                        let expected_len = arg.ty.get_int_length().unwrap();
                        x86_64::get_alias_for_length(reg64.id(), expected_len)
                    };

                    self.backend.emit_mov_r_r(&arg_gpr, &arg);
                    gpr_arg_count += 1;
                } else {
                    // use stack to pass argument
                    stack_args.push(arg.clone());
                }
            } else if arg.is_int_const() {
                let arg_gpr = {
                    let ref reg64 = x86_64::ARGUMENT_GPRs[gpr_arg_count];
                    let expected_len = arg.ty.get_int_length().unwrap();
                    x86_64::get_alias_for_length(reg64.id(), expected_len)
                };

                if x86_64::is_valid_x86_imm(arg) {                
                    let int_const = arg.extract_int_const() as i32;
                    
                    if gpr_arg_count < x86_64::ARGUMENT_GPRs.len() {
                        self.backend.emit_mov_r_imm(&arg_gpr, int_const);
                        gpr_arg_count += 1;
                    } else {
                        // use stack to pass argument
                        stack_args.push(arg.clone());
                    }
                } else {
                    // FIXME: put the constant to memory
                    let int_const = arg.extract_int_const() as i64;
                    self.backend.emit_mov_r64_imm64(&arg_gpr, int_const);
                    gpr_arg_count += 1;
                }
            } else if arg.is_mem() {
                unimplemented!()
            } else {
                // floating point
                unimplemented!()
            }
        }

        if !stack_args.is_empty() {
            // deal with stack arg, put them on stack
            // in reverse order, i.e. push the rightmost arg first to stack
            stack_args.reverse();

            // "The end of the input argument area shall be aligned on a 16
            // (32, if __m256 is passed on stack) byte boundary." - x86 ABI
            // if we need to special align the args, we do it now
            // (then the args will be put to stack following their regular alignment)
            let stack_arg_tys = stack_args.iter().map(|x| x.ty.clone()).collect();
            let (stack_arg_size, _, stack_arg_offsets) = backend::sequetial_layout(&stack_arg_tys, vm);
            let mut stack_arg_size_with_padding = stack_arg_size;
            if stack_arg_size % 16 == 0 {
                // do not need to adjust rsp
            } else if stack_arg_size % 8 == 0 {
                // adjust rsp by -8 (push a random padding value)
                self.backend.emit_push_imm32(0x7777);
                stack_arg_size_with_padding += 8;
            } else {
                panic!("expecting stack arguments to be at least 8-byte aligned, but it has size of {}", stack_arg_size);
            }

            // now, we just put all the args on the stack
            {
                let mut index = 0;
                for arg in stack_args {
                    self.emit_store_base_offset(&x86_64::RSP, - (stack_arg_offsets[index] as i32), &arg, vm);
                    index += 1;
                }

                self.backend.emit_add_r_imm(&x86_64::RSP, (- (stack_arg_size as i32)) as i32);
            }

            stack_arg_size_with_padding
        } else {
            0
        }
    }

    fn emit_postcall_convention(
        &mut self,
        sig: &P<CFuncSig>,
        rets: &Option<Vec<P<Value>>>,
        precall_stack_arg_size: usize,
        f_context: &mut FunctionContext,
        vm: &VM
    ) -> Vec<P<Value>> {
        // deal with ret vals
        let mut return_vals = vec![];

        let mut gpr_ret_count = 0;
        let mut fpr_ret_count = 0;

        for ret_index in 0..sig.ret_tys.len() {
            let ref ty = sig.ret_tys[ret_index];

            let ret_val = match rets {
                &Some(ref rets) => rets[ret_index].clone(),
                &None => {
                    let tmp_node = f_context.make_temporary(vm.next_id(), ty.clone());
                    tmp_node.clone_value()
                }
            };

            if ret_val.is_int_reg() {
                if gpr_ret_count < x86_64::RETURN_GPRs.len() {
                    let ret_gpr = {
                        let ref reg64 = x86_64::RETURN_GPRs[gpr_ret_count];
                        let expected_len = ret_val.ty.get_int_length().unwrap();
                        x86_64::get_alias_for_length(reg64.id(), expected_len)
                    };

                    self.backend.emit_mov_r_r(&ret_val, &ret_gpr);
                    gpr_ret_count += 1;
                } else {
                    // get return value by stack
                    unimplemented!()
                }
            } else if ret_val.is_fp_reg() {
                // floating point register
                if fpr_ret_count < x86_64::RETURN_FPRs.len() {
                    let ref ret_fpr = x86_64::RETURN_FPRs[fpr_ret_count];

                    self.backend.emit_movsd_f64_f64(&ret_val, &ret_fpr);
                    fpr_ret_count += 1;
                } else {
                    // get return value by stack
                    unimplemented!()
                }
            } else {
                unimplemented!()
            }

            return_vals.push(ret_val);
        }

        // remove stack_args
        if precall_stack_arg_size != 0 {
            self.backend.emit_add_r_imm(&x86_64::RSP, precall_stack_arg_size as i32);
        }

        return_vals
    }
    
    #[allow(unused_variables)]
    // ret: Option<Vec<P<Value>>
    // if ret is Some, return values will put stored in given temporaries
    // otherwise create temporaries
    // always returns result temporaries (given or created)
    fn emit_c_call_internal(
        &mut self, 
        func_name: CName, 
        sig: P<CFuncSig>, 
        args: Vec<P<Value>>, 
        rets: Option<Vec<P<Value>>>,
        cur_node: Option<&TreeNode>,
        f_content: &FunctionContent, 
        f_context: &mut FunctionContext, 
        vm: &VM) -> Vec<P<Value>> 
    {
        let stack_arg_size = self.emit_precall_convention(&args, vm);
        
        // make call
        if vm.is_running() {
            unimplemented!()
        } else {
            let callsite = self.new_callsite_label(cur_node);
            self.backend.emit_call_near_rel32(callsite, func_name);
            
            // record exception block (CCall may have an exception block)
            if cur_node.is_some() {
                let cur_node = cur_node.unwrap(); 
                if cur_node.op == OpCode::CCall {
                    unimplemented!()
                }
            }
        }
        
        self.emit_postcall_convention(&sig, &rets, stack_arg_size, f_context, vm)
    }

    #[allow(unused_variables)] // resumption not implemented
    fn emit_c_call_ir(
        &mut self,
        inst: &Instruction,
        calldata: &CallData,
        resumption: Option<&ResumptionData>,
        cur_node: &TreeNode,
        f_content: &FunctionContent,
        f_context: &mut FunctionContext,
        vm: &VM)
    {
        let ops = inst.ops.read().unwrap();

        // prepare args (they could be instructions, we need to emit inst and get value)
        let mut arg_values = vec![];
        for arg_index in calldata.args.iter() {
            let ref arg = ops[*arg_index];

            if self.match_ireg(arg) {
                let arg = self.emit_ireg(arg, f_content, f_context, vm);
                arg_values.push(arg);
            } else if self.match_iimm(arg) {
                let arg = self.node_iimm_to_value(arg);
                arg_values.push(arg);
            } else {
                unimplemented!();
            }
        }
        let args = arg_values;

        trace!("generating ccall");
        let ref func = ops[calldata.func];

        if self.match_funcref_const(func) {
            match func.v {
                TreeNode_::Value(ref pv) => {
                    let sig = match pv.ty.v {
                        MuType_::UFuncPtr(ref sig) => sig.clone(),
                        _ => panic!("expected ufuncptr type with ccall, found {}", pv)
                    };

                    let rets = inst.value.clone();

                    match pv.v {
                        Value_::Constant(Constant::Int(_)) => unimplemented!(),
                        Value_::Constant(Constant::ExternSym(ref func_name)) => {
                            self.emit_c_call_internal(
                                func_name.clone(), //func_name: CName,
                                sig, // sig: P<CFuncSig>,
                                args, // args: Vec<P<Value>>,
                                rets, // Option<Vec<P<Value>>>,
                                Some(cur_node), // Option<&TreeNode>,
                                f_content, // &FunctionContent,
                                f_context, // &mut FunctionContext,
                                vm);
                        },
                        _ => panic!("expect a ufuncptr to be either address constant, or symbol constant, we have {}", pv)
                    }
                },
                _ => unimplemented!()
            }
        }
    }
    
    fn emit_mu_call(
        &mut self,
        inst: &Instruction,
        calldata: &CallData,
        resumption: Option<&ResumptionData>,
        cur_node: &TreeNode, 
        f_content: &FunctionContent, 
        f_context: &mut FunctionContext, 
        vm: &VM) {
        trace!("deal with pre-call convention");
        
        let ops = inst.ops.read().unwrap();
        let ref func = ops[calldata.func];
        let ref func_sig = match func.v {
            TreeNode_::Value(ref pv) => {
                let ty : &MuType = &pv.ty;
                match ty.v {
                    MuType_::FuncRef(ref sig)
                    | MuType_::UFuncPtr(ref sig) => sig,
                    _ => panic!("expected funcref/ptr type")
                }
            },
            _ => panic!("expected funcref/ptr type")
        };
        
        debug_assert!(func_sig.arg_tys.len() == calldata.args.len());
        if cfg!(debug_assertions) {
            if inst.value.is_some() {
                assert!(func_sig.ret_tys.len() == inst.value.as_ref().unwrap().len());
            } else {
                assert!(func_sig.ret_tys.len() == 0, "expect call inst's value doesnt match reg args. value: {:?}, ret args: {:?}", inst.value, func_sig.ret_tys);
            }
        }

        // prepare args (they could be instructions, we need to emit inst and get value)
        let mut arg_values = vec![];
        for arg_index in calldata.args.iter() {
            let ref arg = ops[*arg_index];

            if self.match_ireg(arg) {
                let arg = self.emit_ireg(arg, f_content, f_context, vm);
                arg_values.push(arg);
            } else if self.match_iimm(arg) {
                let arg = self.node_iimm_to_value(arg);
                arg_values.push(arg);
            } else {
                unimplemented!();
            }
        }
        let stack_arg_size = self.emit_precall_convention(&arg_values, vm);
        
        trace!("generating call inst");
        // check direct call or indirect
        let callsite = {
            if self.match_funcref_const(func) {
                let target_id = self.node_funcref_const_to_id(func);
                let funcs = vm.funcs().read().unwrap();
                let target = funcs.get(&target_id).unwrap().read().unwrap();
                                            
                if vm.is_running() {
                    unimplemented!()
                } else {
                    let callsite = self.new_callsite_label(Some(cur_node));
                    self.backend.emit_call_near_rel32(callsite, target.name().unwrap())
                }
            } else if self.match_ireg(func) {
                let target = self.emit_ireg(func, f_content, f_context, vm);
                
                let callsite = self.new_callsite_label(Some(cur_node));
                self.backend.emit_call_near_r64(callsite, &target)
            } else if self.match_mem(func) {
                let target = self.emit_mem(func, vm);
                
                let callsite = self.new_callsite_label(Some(cur_node));
                self.backend.emit_call_near_mem64(callsite, &target)
            } else {
                unimplemented!()
            }
        };
        
        // record exception branch
        if resumption.is_some() {
            let ref exn_dest = resumption.as_ref().unwrap().exn_dest;
            let target_block = exn_dest.target;
            
            if self.current_exn_callsites.contains_key(&target_block) {
                let callsites = self.current_exn_callsites.get_mut(&target_block).unwrap();
                callsites.push(callsite);
            } else {
                let mut callsites = vec![];
                callsites.push(callsite);
                self.current_exn_callsites.insert(target_block, callsites);
            } 
        }
        
        // deal with ret vals
        self.emit_postcall_convention(
            &func_sig, &inst.value,
            stack_arg_size, f_context, vm);
    }
    
    #[allow(unused_variables)]
    fn process_dest(&mut self, ops: &Vec<P<TreeNode>>, dest: &Destination, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) {
        for i in 0..dest.args.len() {
            let ref dest_arg = dest.args[i];
            match dest_arg {
                &DestArg::Normal(op_index) => {
                    let ref arg = ops[op_index];
//                    match arg.op {
//                        OpCode::RegI64 
//                        | OpCode::RegFP
//                        | OpCode::IntImmI64
//                        | OpCode::FPImm => {
//                            // do nothing
//                        },
//                        _ => {
//                            trace!("nested: compute arg for branch");
//                            // nested: compute arg
//                            self.instruction_select(arg, cur_func);
//                            
//                            self.emit_get_result(arg);
//                        }
//                    }
//                    
                    let ref target_args = f_content.get_block(dest.target).content.as_ref().unwrap().args;
                    let ref target_arg = target_args[i];
                    
                    self.emit_move_node_to_value(target_arg, &arg, f_content, f_context, vm);
                },
                &DestArg::Freshbound(_) => unimplemented!()
            }
        }
    }
    
    fn emit_common_prologue(&mut self, args: &Vec<P<Value>>, vm: &VM) {
        let block_name = PROLOGUE_BLOCK_NAME.to_string();
        self.backend.start_block(block_name.clone());
        
        // no livein
        // liveout = entry block's args
        self.backend.set_block_livein(block_name.clone(), &vec![]);
        self.backend.set_block_liveout(block_name.clone(), args);
        
        // push rbp
        self.backend.emit_push_r64(&x86_64::RBP);
        // mov rsp -> rbp
        self.backend.emit_mov_r_r(&x86_64::RBP, &x86_64::RSP);
        
        // push all callee-saved registers
        {
            let frame = self.current_frame.as_mut().unwrap();
            let rbp = x86_64::RBP.extract_ssa_id().unwrap();
            for i in 0..x86_64::CALLEE_SAVED_GPRs.len() {
                let ref reg = x86_64::CALLEE_SAVED_GPRs[i];
                // not pushing rbp (as we have done that)
                if reg.extract_ssa_id().unwrap() !=  rbp {
                    trace!("allocate frame slot for reg {}", reg);
                    self.backend.emit_push_r64(&reg);
                    frame.alloc_slot_for_callee_saved_reg(reg.clone(), vm);
                }
            }
        }

        // reserve spaces for current frame
        // add x, rbp -> rbp (x is negative, however we do not know x now)
        self.backend.emit_frame_grow();
        
        // unload arguments
        let mut gpr_arg_count = 0;
        let mut fpr_arg_count = 0;
        // initial stack arg is at RBP+16
        //   arg           <- RBP + 16
        //   return addr
        //   old RBP       <- RBP
        let mut stack_arg_offset : i32 = 16;
        for arg in args {
            if arg.is_int_reg() {
                if gpr_arg_count < x86_64::ARGUMENT_GPRs.len() {
                    let arg_gpr = {
                        let ref reg64 = x86_64::ARGUMENT_GPRs[gpr_arg_count];
                        let expected_len = arg.ty.get_int_length().unwrap();
                        x86_64::get_alias_for_length(reg64.id(), expected_len)
                    };

                    self.backend.emit_mov_r_r(&arg, &arg_gpr);

                    gpr_arg_count += 1;
                } else {
                    // unload from stack
                    self.emit_load_base_offset(&arg, &x86_64::RBP.clone(), stack_arg_offset, vm);
                    
                    // move stack_arg_offset by the size of 'arg'
                    let arg_size = vm.get_backend_type_info(arg.ty.id()).size;
                    stack_arg_offset += arg_size as i32;
                }
            } else if arg.is_fp_reg() {
                if fpr_arg_count < x86_64::ARGUMENT_FPRs.len() {
                    self.backend.emit_movsd_f64_f64(&arg, &x86_64::ARGUMENT_FPRs[fpr_arg_count]);
                    fpr_arg_count += 1;
                } else {
                    // unload from stack
                    self.emit_load_base_offset(&arg, &x86_64::RBP.clone(), stack_arg_offset, vm);

                    // move stack_arg_offset by the size of 'arg'
                    let arg_size = vm.get_backend_type_info(arg.ty.id()).size;
                    stack_arg_offset += arg_size as i32;
                }
            } else {
                // args that are not fp or int (possibly struct/array/etc)
                unimplemented!();
            }
        }
        
        self.backend.end_block(block_name);
    }
    
    fn emit_common_epilogue(&mut self, ret_inst: &Instruction, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) {
        // epilogue is not a block (its a few instruction inserted before return)
        // FIXME: this may change in the future

        // prepare return regs
        let ref ops = ret_inst.ops.read().unwrap();
        let ret_val_indices = match ret_inst.v {
            Instruction_::Return(ref vals) => vals,
            _ => panic!("expected ret inst")
        };
        
        let mut gpr_ret_count = 0;
        let mut fpr_ret_count = 0;
        for i in ret_val_indices {
            let ref ret_val = ops[*i];
            if self.match_ireg(ret_val) {
                let reg_ret_val = self.emit_ireg(ret_val, f_content, f_context, vm);

                let ret_gpr = {
                    let ref reg64 = x86_64::RETURN_GPRs[gpr_ret_count];
                    let expected_len = reg_ret_val.ty.get_int_length().unwrap();
                    x86_64::get_alias_for_length(reg64.id(), expected_len)
                };
                
                self.backend.emit_mov_r_r(&ret_gpr, &reg_ret_val);
                gpr_ret_count += 1;
            } else if self.match_iimm(ret_val) {
                let imm_ret_val = self.node_iimm_to_i32(ret_val);
                
                self.backend.emit_mov_r_imm(&x86_64::RETURN_GPRs[gpr_ret_count], imm_ret_val);
                gpr_ret_count += 1;
            } else if self.match_fpreg(ret_val) {
                let reg_ret_val = self.emit_fpreg(ret_val, f_content, f_context, vm);

                self.backend.emit_movsd_f64_f64(&x86_64::RETURN_FPRs[fpr_ret_count], &reg_ret_val);
                fpr_ret_count += 1;
            } else {
                unimplemented!();
            }
        }

        // frame shrink
        self.backend.emit_frame_shrink();
        
        // pop all callee-saved registers - reverse order
        for i in (0..x86_64::CALLEE_SAVED_GPRs.len()).rev() {
            let ref reg = x86_64::CALLEE_SAVED_GPRs[i];
            if reg.extract_ssa_id().unwrap() != x86_64::RBP.extract_ssa_id().unwrap() {
                self.backend.emit_pop_r64(&reg);
            }
        }
        
        // pop rbp
        self.backend.emit_pop_r64(&x86_64::RBP);
    }
    
    fn match_cmp_res(&mut self, op: &TreeNode) -> bool {
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
    
    fn emit_cmp_res(&mut self, cond: &TreeNode, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) -> op::CmpOp {
        match cond.v {
            TreeNode_::Instruction(ref inst) => {
                let ops = inst.ops.read().unwrap();                
                
                match inst.v {
                    Instruction_::CmpOp(op, op1, op2) => {
                        let op1 = &ops[op1];
                        let op2 = &ops[op2];
                        
                        if op::is_int_cmp(op) {
                            if self.match_iimm(op1) && self.match_iimm(op2) {
                                let ty : &P<MuType> = match op1.clone_value().ty.get_int_length() {
                                    Some(64) => &UINT64_TYPE,
                                    Some(32) => &UINT32_TYPE,
                                    Some(16) => &UINT16_TYPE,
                                    Some(8)  => &UINT8_TYPE,
                                    _ => unimplemented!()
                                };

                                let tmp_op1 = self.make_temporary(f_context, ty.clone(), vm);
                                let iimm_op1 = self.node_iimm_to_i32(op1);
                                self.backend.emit_mov_r_imm(&tmp_op1, iimm_op1);

                                let iimm_op2 = self.node_iimm_to_i32(op2);
                                self.backend.emit_cmp_imm_r(iimm_op2, &tmp_op1);

                                return op;
                            } else if self.match_ireg(op1) && self.match_iimm(op2) {
                                let reg_op1 = self.emit_ireg(op1, f_content, f_context, vm);
                                let iimm_op2 = self.node_iimm_to_i32(op2);

                                // we adopt at&t syntax
                                // so CMP op1 op2
                                // is actually CMP op2 op1 (in machine code)
                                self.backend.emit_cmp_imm_r(iimm_op2, &reg_op1);

                                return op;
                            } else if self.match_ireg(op1) && self.match_ireg(op2) {
                                let reg_op1 = self.emit_ireg(op1, f_content, f_context, vm);
                                let reg_op2 = self.emit_ireg(op2, f_content, f_context, vm);

                                self.backend.emit_cmp_r_r(&reg_op2, &reg_op1);

                                return op;
                            } else {
                                unimplemented!()
                            }
                        } else {
                            // floating point comparison
                            let reg_op1 = self.emit_fpreg(op1, f_content, f_context, vm);
                            let reg_op2 = self.emit_fpreg(op2, f_content, f_context, vm);

                            match op {
                                op::CmpOp::FOEQ
                                | op::CmpOp::FOGT
                                | op::CmpOp::FOGE
                                | op::CmpOp::FOLT
                                | op::CmpOp::FOLE
                                | op::CmpOp::FONE => {
                                    self.backend.emit_comisd_f64_f64(&reg_op2, &reg_op1);

                                    op
                                },
                                op::CmpOp::FUEQ
                                | op::CmpOp::FUGT
                                | op::CmpOp::FUGE
                                | op::CmpOp::FULT
                                | op::CmpOp::FULE
                                | op::CmpOp::FUNE => {
                                    self.backend.emit_ucomisd_f64_f64(&reg_op2, &reg_op1);

                                    op
                                },
                                _ => unimplemented!()
                            }
                        }
                    }
                    
                    _ => panic!("expect cmp res to emit")
                }
            }
            _ => panic!("expect cmp res to emit")
        }
    }    
    
    fn match_ireg(&mut self, op: &TreeNode) -> bool {
        match op.v {
            TreeNode_::Instruction(ref inst) => {
                if inst.value.is_some() {
                    if inst.value.as_ref().unwrap().len() > 1 {
                        return false;
                    }
                    
                    let ref value = inst.value.as_ref().unwrap()[0];
                    
                    if value.is_int_reg() {
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            
            TreeNode_::Value(ref pv) => {
                pv.is_int_reg() || pv.is_int_const()
            }
        }
    }

    fn match_fpreg(&mut self, op: &TreeNode) -> bool {
        match op.v {
            TreeNode_::Instruction(ref inst) => {
                if inst.value.is_some() {
                    if inst.value.as_ref().unwrap().len() > 1 {
                        return false;
                    }

                    let ref value = inst.value.as_ref().unwrap()[0];

                    if value.is_fp_reg() {
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }

            TreeNode_::Value(ref pv) => {
                pv.is_fp_reg()
            }
        }
    }
    
    fn emit_ireg(&mut self, op: &TreeNode, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) -> P<Value> {
        match op.v {
            TreeNode_::Instruction(_) => {
                self.instruction_select(op, f_content, f_context, vm);
                
                self.get_result_value(op)
            },
            TreeNode_::Value(ref pv) => {
                match pv.v {
                    Value_::SSAVar(_) => pv.clone(),
                    Value_::Constant(ref c) => {
                        let tmp = self.make_temporary(f_context, pv.ty.clone(), vm);
                        match c {
                            &Constant::Int(val) => {
                                if x86_64::is_valid_x86_imm(pv) {
                                    let val = self.value_iimm_to_i32(&pv);

                                    debug!("tmp's ty: {}", tmp.ty);
                                    self.backend.emit_mov_r_imm(&tmp, val)
                                } else {
                                    self.backend.emit_mov_r64_imm64(&tmp, val as i64);
                                }
                            },
                            &Constant::FuncRef(_) => {
                                unimplemented!()
                            },
                            &Constant::NullRef => {
                                self.backend.emit_xor_r_r(&tmp, &tmp);
                            },
                            _ => panic!("expected ireg")
                        }

                        tmp
                    },
                    _ => panic!("expected ireg")
                }
            }
        }
    }

    fn emit_fpreg(&mut self, op: &TreeNode, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) -> P<Value> {
        match op.v {
            TreeNode_::Instruction(_) => {
                self.instruction_select(op, f_content, f_context, vm);

                self.get_result_value(op)
            },
            TreeNode_::Value(ref pv) => {
                match pv.v {
                    Value_::SSAVar(_) => pv.clone(),
                    Value_::Constant(Constant::Double(val)) => {
                        use std::mem;

                        // val into u64
                        let val_u64 : u64 = unsafe {mem::transmute(val)};

                        // mov val_u64 -> tmp_int
                        let tmp_int = self.make_temporary(f_context, UINT64_TYPE.clone(), vm);
                        self.backend.emit_mov_r64_imm64(&tmp_int, val_u64 as i64);

                        // movq tmp_int -> tmp_fp
                        let tmp_fp = self.make_temporary(f_context, DOUBLE_TYPE.clone(), vm);
                        self.backend.emit_mov_fpr_r64(&tmp_fp, &tmp_int);

                        tmp_fp
                    }
                    Value_::Constant(Constant::Float(_)) => {
                        unimplemented!()
                    },
                    _ => panic!("expected fpreg")
                }
            }
        }
    }
    
    fn match_iimm(&mut self, op: &TreeNode) -> bool {
        match op.v {
            TreeNode_::Value(ref pv) if x86_64::is_valid_x86_imm(pv) => true,
            _ => false
        }
    }
    
    fn node_iimm_to_i32(&mut self, op: &TreeNode) -> i32 {
        match op.v {
            TreeNode_::Value(ref pv) => self.value_iimm_to_i32(pv),
            _ => panic!("expected iimm")
        }
    }

    fn node_iimm_to_value(&mut self, op: &TreeNode) -> P<Value> {
        match op.v {
            TreeNode_::Value(ref pv) => {
                pv.clone()
            }
            _ => panic!("expected iimm")
        }
    }

    fn value_iimm_to_i32(&mut self, op: &P<Value>) -> i32 {
        match op.v {
            Value_::Constant(Constant::Int(val)) => {
                debug_assert!(x86_64::is_valid_x86_imm(op));

                val as i32
            },
            _ => panic!("expected iimm")
        }
    }
    
    fn emit_node_addr_to_value(&mut self, op: &TreeNode, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) -> P<Value> {
        match op.v {
            TreeNode_::Value(ref pv) => {
                match pv.v {
                    Value_::SSAVar(_) => P(Value{
                        hdr: MuEntityHeader::unnamed(vm.next_id()),
                        ty: types::get_referent_ty(& pv.ty).unwrap(),
                        v: Value_::Memory(MemoryLocation::Address{
                            base: pv.clone(),
                            offset: None,
                            index: None,
                            scale: None
                        })
                    }),
                    Value_::Global(_) => {
                        if vm.is_running() {
                            // get address from vm
                            unimplemented!()
                        } else {
                            // symbolic
                            if cfg!(target_os = "macos") {
                                P(Value {
                                    hdr: MuEntityHeader::unnamed(vm.next_id()),
                                    ty: types::get_referent_ty(&pv.ty).unwrap(),
                                    v: Value_::Memory(MemoryLocation::Symbolic {
                                        base: Some(x86_64::RIP.clone()),
                                        label: pv.name().unwrap(),
                                        is_global: true,
                                    })
                                })
                            } else if cfg!(target_os = "linux") {
                                // for a(%RIP), we need to load its address from a@GOTPCREL(%RIP)
                                // then load from the address.
                                // asm_backend will emit a@GOTPCREL(%RIP) for a(%RIP)
                                let got_loc = P(Value {
                                    hdr: MuEntityHeader::unnamed(vm.next_id()),
                                    ty: pv.ty.clone(),
                                    v: Value_::Memory(MemoryLocation::Symbolic {
                                        base: Some(x86_64::RIP.clone()),
                                        label: pv.name().unwrap(),
                                        is_global: true
                                    })
                                });

                                // mov (got_loc) -> actual_loc
                                let actual_loc = self.make_temporary(f_context, pv.ty.clone(), vm);
                                self.emit_move_value_to_value(&actual_loc, &got_loc);

                                self.make_memory_op_base_offset(&actual_loc, 0, types::get_referent_ty(&pv.ty).unwrap(), vm)
                            } else {
                                unimplemented!()
                            }
                        }
                    },
                    Value_::Memory(_) => pv.clone(),
                    Value_::Constant(_) => unimplemented!()
                }
            }
            TreeNode_::Instruction(_) => self.emit_get_mem_from_inst(op, f_content, f_context, vm)
        }
    }

    fn emit_get_mem_from_inst(&mut self, op: &TreeNode, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) -> P<Value> {
        let mem = self.emit_get_mem_from_inst_inner(op, f_content, f_context, vm);

        P(Value{
            hdr: MuEntityHeader::unnamed(vm.next_id()),
            ty: ADDRESS_TYPE.clone(),
            v: Value_::Memory(mem)
        })
    }

    fn addr_const_offset_adjust(&mut self, mem: MemoryLocation, more_offset: u64, vm: &VM) -> MemoryLocation {
        match mem {
            MemoryLocation::Address { base, offset, index, scale } => {
                let new_offset = match offset {
                    Some(pv) => {
                        let old_offset = pv.extract_int_const();
                        old_offset + more_offset
                    },
                    None => more_offset
                };

                MemoryLocation::Address {
                    base: base,
                    offset: Some(self.make_value_int_const(new_offset, vm)),
                    index: index,
                    scale: scale
                }
            },
            _ => panic!("expected an address memory location")
        }
    }

    #[allow(unused_variables)]
    fn addr_append_index_scale(&mut self, mem: MemoryLocation, index: P<Value>, scale: u8, vm: &VM) -> MemoryLocation {
        match mem {
            MemoryLocation::Address {base, offset, ..} => {
                MemoryLocation::Address {
                    base: base,
                    offset: offset,
                    index: Some(index),
                    scale: Some(scale)
                }
            },
            _ => panic!("expected an address memory location")
        }
    }

    fn emit_get_mem_from_inst_inner(&mut self, op: &TreeNode, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) -> MemoryLocation {        match op.v {
            TreeNode_::Instruction(ref inst) => {
                let ref ops = inst.ops.read().unwrap();
                
                match inst.v {
                    // GETIREF -> [base]
                    Instruction_::GetIRef(op_index) => {
                        let ref ref_op = ops[op_index];
                        let tmp_op = self.emit_ireg(ref_op, f_content, f_context, vm);

                        let ret = MemoryLocation::Address {
                            base: tmp_op,
                            offset: None,
                            index: None,
                            scale: None
                        };

                        trace!("MEM from GETIREF: {}", ret);
                        ret
                    }
                    Instruction_::GetFieldIRef{base, index, ..} => {
                        let ref base = ops[base];

                        let struct_ty = {
                            let ref iref_or_uptr_ty = base.clone_value().ty;

                            match iref_or_uptr_ty.v {
                                MuType_::IRef(ref ty)
                                | MuType_::UPtr(ref ty) => ty.clone(),
                                _ => panic!("expected the base for GetFieldIRef has a type of iref or uptr, found type: {}", iref_or_uptr_ty)
                            }
                        };

                        let field_offset : i32 = self.get_field_offset(&struct_ty, index, vm);

                        match base.v {
                            // GETFIELDIREF(GETIREF) -> add FIELD_OFFSET to old offset
                            TreeNode_::Instruction(Instruction{v: Instruction_::GetIRef(_), ..}) => {
                                let mem = self.emit_get_mem_from_inst_inner(base, f_content, f_context, vm);
                                let ret = self.addr_const_offset_adjust(mem, field_offset as u64, vm);

                                trace!("MEM from GETFIELDIREF(GETIREF): {}", ret);
                                ret
                            },
                            // GETFIELDIREF(ireg) -> [base + FIELD_OFFSET]
                            _ => {
                                let tmp = self.emit_ireg(base, f_content, f_context, vm);

                                let ret = MemoryLocation::Address {
                                    base: tmp,
                                    offset: Some(self.make_value_int_const(field_offset as u64, vm)),
                                    index: None,
                                    scale: None
                                };

                                trace!("MEM from GETFIELDIREF(ireg): {}", ret);
                                ret
                            }
                        }
                    }
                    Instruction_::GetVarPartIRef{base, ..} => {
                        let ref base = ops[base];

                        let struct_ty = match base.clone_value().ty.get_referenced_ty() {
                            Some(ty) => ty,
                            None => panic!("expecting an iref or uptr in GetVarPartIRef")
                        };

                        let fix_part_size = vm.get_backend_type_info(struct_ty.id()).size;

                        match base.v {
                            // GETVARPARTIREF(GETIREF) -> add FIX_PART_SIZE to old offset
                            TreeNode_::Instruction(Instruction{v: Instruction_::GetIRef(_), ..}) => {
                                let mem = self.emit_get_mem_from_inst_inner(base, f_content, f_context, vm);

                                let ret = self.addr_const_offset_adjust(mem, fix_part_size as u64, vm);

                                trace!("MEM from GETIVARPARTIREF(GETIREF): {}", ret);
                                ret
                            },
                            // GETVARPARTIREF(ireg) -> [base + VAR_PART_SIZE]
                            _ => {
                                let tmp = self.emit_ireg(base, f_content, f_context, vm);

                                let ret = MemoryLocation::Address {
                                    base: tmp,
                                    offset: Some(self.make_value_int_const(fix_part_size as u64, vm)),
                                    index: None,
                                    scale: None
                                };

                                trace!("MEM from GETVARPARTIREF(ireg): {}", ret);
                                ret
                            }
                        }
                    }
                    Instruction_::ShiftIRef{base, offset, ..} => {
                        let ref base = ops[base];
                        let ref offset = ops[offset];

                        let ref base_ty = base.clone_value().ty;
                        let ele_ty = match base_ty.get_referenced_ty() {
                            Some(ty) => ty,
                            None => panic!("expected op in ShiftIRef of type IRef, found type: {}", base_ty)
                        };
                        let ele_ty_size = vm.get_backend_type_info(ele_ty.id()).size;

                        if self.match_iimm(offset) {
                            let index = self.node_iimm_to_i32(offset);
                            let shift_size = ele_ty_size as i32 * index;

                            let mem = match base.v {
                                // SHIFTIREF(GETVARPARTIREF(_), imm) -> add shift_size to old offset
                                TreeNode_::Instruction(Instruction{v: Instruction_::GetVarPartIRef{..}, ..}) => {
                                    let mem = self.emit_get_mem_from_inst_inner(base, f_content, f_context, vm);

                                    let ret = self.addr_const_offset_adjust(mem, shift_size as u64, vm);

                                    trace!("MEM from SHIFTIREF(GETVARPARTIREF(_), imm): {}", ret);
                                    ret
                                },
                                // SHIFTIREF(ireg, imm) -> [base + SHIFT_SIZE]
                                _ => {
                                    let tmp = self.emit_ireg(base, f_content, f_context, vm);

                                    let ret = MemoryLocation::Address {
                                        base: tmp,
                                        offset: Some(self.make_value_int_const(shift_size as u64, vm)),
                                        index: None,
                                        scale: None
                                    };

                                    trace!("MEM from SHIFTIREF(ireg, imm): {}", ret);
                                    ret
                                }
                            };

                            mem
                        } else {
                            let tmp_index = self.emit_ireg(offset, f_content, f_context, vm);

                            let scale : u8 = match ele_ty_size {
                                8 | 4 | 2 | 1 => ele_ty_size as u8,
                                _  => unimplemented!()
                            };

                            let mem = match base.v {
                                // SHIFTIREF(GETVARPARTIREF(_), ireg) -> add index and scale
                                TreeNode_::Instruction(Instruction{v: Instruction_::GetVarPartIRef{..}, ..}) => {
                                    let mem = self.emit_get_mem_from_inst_inner(base, f_content, f_context, vm);

                                    let ret = self.addr_append_index_scale(mem, tmp_index, scale, vm);

                                    trace!("MEM from SHIFTIREF(GETVARPARTIREF(_), ireg): {}", ret);
                                    ret
                                },
                                // SHIFTIREF(ireg, ireg) -> base + index * scale
                                _ => {
                                    let tmp = self.emit_ireg(base, f_content, f_context, vm);

                                    let ret = MemoryLocation::Address {
                                        base: tmp,
                                        offset: None,
                                        index: Some(tmp_index),
                                        scale: Some(scale)
                                    };

                                    trace!("MEM from SHIFTIREF(ireg, ireg): {}", ret);
                                    ret
                                }
                            };

                            mem
                        }
                    }
                    Instruction_::GetElementIRef {base, index, ..} => {
                        let ref base = ops[base];
                        let ref index = ops[index];

                        let ref iref_array_ty = base.clone_value().ty;
                        let array_ty = match iref_array_ty.get_referenced_ty() {
                            Some(ty) => ty,
                            None => panic!("expected base in GetElemIRef to be type IRef, found {}", iref_array_ty)
                        };
                        let ele_ty = match array_ty.get_elem_ty() {
                            Some(ty) => ty,
                            None => panic!("expected base in GetElemIRef to be type Array, found {}", array_ty)
                        };
                        let ele_ty_size = vm.get_backend_type_info(ele_ty.id()).size;

                        if self.match_iimm(index) {
                            let index = self.node_iimm_to_i32(index);
                            let offset = ele_ty_size as i32 * index;

                            match base.v {
                                // GETELEMIREF(GETIREF) -> add offset
                                TreeNode_::Instruction(Instruction{v: Instruction_::GetIRef(_), ..}) => {
                                    let mem = self.emit_get_mem_from_inst_inner(base, f_content, f_context, vm);
                                    let ret = self.addr_const_offset_adjust(mem, offset as u64, vm);

                                    trace!("MEM from GETELEMIREF(GETIREF, const): {}", ret);
                                    ret
                                },
                                // GETELEMIREF(GETFIELDIREF) -> add offset
                                TreeNode_::Instruction(Instruction{v: Instruction_::GetFieldIRef{..}, ..}) => {
                                    let mem = self.emit_get_mem_from_inst_inner(base, f_content, f_context, vm);
                                    let ret = self.addr_const_offset_adjust(mem, offset as u64, vm);

                                    trace!("MEM from GETELEMIREF(GETFIELDIREF, const): {}", ret);
                                    ret
                                },
                                // GETELEMIREF(ireg) => [base + offset]
                                _ => {
                                    let tmp = self.emit_ireg(base, f_content, f_context, vm);

                                    let ret = MemoryLocation::Address {
                                        base: tmp,
                                        offset: Some(self.make_value_int_const(offset as u64, vm)),
                                        index: None,
                                        scale: None
                                    };

                                    trace!("MEM from GETELEMIREF(ireg, const): {}", ret);
                                    ret
                                }
                            }
                        } else {
                            let tmp_index = self.emit_ireg(index, f_content, f_context, vm);
                            let scale : u8 = match ele_ty_size {
                                8 | 4 | 2 | 1 => ele_ty_size as u8,
                                _  => unimplemented!()
                            };

                            match base.v {
                                // GETELEMIREF(IREF, ireg) -> add index and scale
                                TreeNode_::Instruction(Instruction{v: Instruction_::GetIRef(_), ..}) => {
                                    let mem = self.emit_get_mem_from_inst_inner(base, f_content, f_context, vm);
                                    let ret = self.addr_append_index_scale(mem, tmp_index, scale, vm);

                                    trace!("MEM from GETELEMIREF(GETIREF, ireg): {}", ret);
                                    ret
                                }
                                // GETELEMIREF(GETFIELDIREF, ireg) -> add index and scale
                                TreeNode_::Instruction(Instruction{v: Instruction_::GetFieldIRef{..}, ..}) => {
                                    let mem = self.emit_get_mem_from_inst_inner(base, f_content, f_context, vm);
                                    let ret = self.addr_append_index_scale(mem, tmp_index, scale, vm);

                                    trace!("MEM from GETELEMIREF(GETFIELDIREF, ireg): {}", ret);
                                    ret
                                }
                                // GETELEMIREF(ireg, ireg)
                                _ => {
                                    let tmp = self.emit_ireg(base, f_content, f_context, vm);

                                    let ret = MemoryLocation::Address {
                                        base: tmp,
                                        offset: None,
                                        index: Some(tmp_index),
                                        scale: Some(scale)
                                    };

                                    trace!("MEM from GETELEMIREF(ireg, ireg): {}", ret);
                                    ret
                                }
                            }
                        }

                    }
                    _ => unimplemented!()
                }
            },
            _ => panic!("expecting a instruction that yields a memory address")
        }
    }
    
    fn match_funcref_const(&mut self, op: &TreeNode) -> bool {
        match op.v {
            TreeNode_::Value(ref pv) => {
                let is_const = match pv.v {
                    Value_::Constant(_) => true,
                    _ => false
                };

                let is_func = match &pv.ty.v {
                    &MuType_::FuncRef(_)
                    | &MuType_::UFuncPtr(_) => true,
                    _ => false
                };

                is_const && is_func
            },
            _ => false 
        }
    }
    
    fn node_funcref_const_to_id(&mut self, op: &TreeNode) -> MuID {
        match op.v {
            TreeNode_::Value(ref pv) => {
                match pv.v {
                    Value_::Constant(Constant::FuncRef(id)) => id,
                    _ => panic!("expected a funcref const")
                }
            },
            _ => panic!("expected a funcref const")
        }
    }
    
    #[allow(unused_variables)]
    fn match_mem(&mut self, op: &TreeNode) -> bool {
        match op.v {
            TreeNode_::Value(ref pv) => {
                match pv.v {
                    Value_::Memory(_) => true,
                    Value_::Global(_) => true,
                    _ => false
                }
            }
            TreeNode_::Instruction(ref inst) => {
                match inst.v {
                    Instruction_::Load{..} => true,
                    _ => false
                }
            }
        }
    }
    
    #[allow(unused_variables)]
    fn emit_mem(&mut self, op: &TreeNode, vm: &VM) -> P<Value> {
        unimplemented!()
    }
    
    fn get_result_value(&mut self, node: &TreeNode) -> P<Value> {
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
    
    fn emit_move_node_to_value(&mut self, dest: &P<Value>, src: &TreeNode, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) {
        let ref dst_ty = dest.ty;
        
        if !types::is_fp(dst_ty) && types::is_scalar(dst_ty) {
            if self.match_iimm(src) {
                let src_imm = self.node_iimm_to_i32(src);
                if dest.is_int_reg() {
                    self.backend.emit_mov_r_imm(dest, src_imm);
                } else if dest.is_mem() {
                    self.backend.emit_mov_mem_imm(dest, src_imm);
                } else {
                    panic!("unexpected dest: {}", dest);
                }
            } else if self.match_ireg(src) {
                let src_reg = self.emit_ireg(src, f_content, f_context, vm);
                self.emit_move_value_to_value(dest, &src_reg);
            } else {
                panic!("expected src: {}", src);
            }
        } else if types::is_fp(dst_ty) && types::is_scalar(dst_ty) {
            if self.match_fpreg(src) {
                let src_reg = self.emit_fpreg(src, f_content, f_context, vm);
                self.emit_move_value_to_value(dest, &src_reg)
            } else {
                panic!("unexpected fp src: {}", src);
            }
        } else {
            unimplemented!()
        } 
    }

    fn emit_move_value_to_value(&mut self, dest: &P<Value>, src: &P<Value>) {
        let ref src_ty = src.ty;

        if types::is_scalar(src_ty) && !types::is_fp(src_ty) {
            // gpr mov
            if dest.is_int_reg() && src.is_int_reg() {
                self.backend.emit_mov_r_r(dest, src);
            } else if dest.is_int_reg() && src.is_mem() {
                self.backend.emit_mov_r_mem(dest, src);
            } else if dest.is_int_reg() && src.is_int_const() {
                let imm = self.value_iimm_to_i32(src);
                self.backend.emit_mov_r_imm(dest, imm);
            } else if dest.is_mem() && src.is_int_reg() {
                self.backend.emit_mov_mem_r(dest, src);
            } else if dest.is_mem() && src.is_int_const() {
                let imm = self.value_iimm_to_i32(src);
                self.backend.emit_mov_mem_imm(dest, imm);
            } else {
                panic!("unexpected gpr mov between {} -> {}", src, dest);
            }
        } else if types::is_scalar(src_ty) && types::is_fp(src_ty) {
            // fpr mov
            if dest.is_fp_reg() && src.is_fp_reg() {
                self.backend.emit_movsd_f64_f64(dest, src);
            } else if dest.is_fp_reg() && src.is_mem() {
                self.backend.emit_movsd_f64_mem64(dest, src);
            } else if dest.is_mem() && src.is_fp_reg() {
                self.backend.emit_movsd_mem64_f64(dest, src);
            } else {
                panic!("unexpected fpr mov between {} -> {}", src, dest);
            }
        } else {
            panic!("unexpected mov of type {}", src_ty)
        }
    }
    
    fn emit_landingpad(&mut self, exception_arg: &P<Value>, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) {
        // get thread local and add offset to get exception_obj
        let tl = self.emit_get_threadlocal(None, f_content, f_context, vm);
        self.emit_load_base_offset(exception_arg, &tl, *thread::EXCEPTION_OBJ_OFFSET as i32, vm);
    }

    fn get_field_offset(&mut self, ty: &P<MuType>, index: usize, vm: &VM) -> i32 {
        let ty_info = vm.get_backend_type_info(ty.id());
        let layout  = match ty_info.struct_layout.as_ref() {
            Some(layout) => layout,
            None => panic!("a struct type does not have a layout yet: {:?}", ty_info)
        };
        debug_assert!(layout.len() > index);

        layout[index] as i32
    }
    
    fn new_callsite_label(&mut self, cur_node: Option<&TreeNode>) -> String {
        let ret = {
            if cur_node.is_some() {
                format!("callsite_{}_{}_{}", self.current_fv_id, cur_node.unwrap().id(), self.current_callsite_id)
            } else {
                format!("callsite_{}_anon_{}", self.current_fv_id, self.current_callsite_id)
            }
        };
        self.current_callsite_id += 1;
        ret
    }

    fn get_mem_for_const(&mut self, val: P<Value>, vm: &VM) -> P<Value> {
        let id = val.id();

        if self.current_constants.contains_key(&id) {
            self.current_constants.get(&id).unwrap().clone()
        } else {
            let const_value_loc = vm.allocate_const(val.clone());

            let const_mem_val = match const_value_loc {
                ValueLocation::Relocatable(_, ref name) => {
                    P(Value {
                        hdr: MuEntityHeader::unnamed(vm.next_id()),
                        ty : ADDRESS_TYPE.clone(),
                        v  : Value_::Memory(MemoryLocation::Symbolic {
                            base: Some(x86_64::RIP.clone()),
                            label: name.clone(),
                            is_global: false
                        })
                    })
                }
                _ => panic!("expecting relocatable location, found {}", const_value_loc)
            };

            self.current_constants.insert(id, val.clone());
            self.current_constants_locs.insert(id, const_mem_val.clone());

            const_mem_val
        }
    }

    fn finish_block(&mut self) {
        let cur_block = self.current_block.as_ref().unwrap().clone();
        self.backend.end_block(cur_block.clone());
    }

    fn start_block(&mut self, block: String) {
        self.current_block = Some(block.clone());
        self.backend.start_block(block);
    }
}

impl CompilerPass for InstructionSelection {
    fn name(&self) -> &'static str {
        self.name
    }

    fn as_any(&self) -> &Any {
        self
    }

    #[allow(unused_variables)]
    fn start_function(&mut self, vm: &VM, func_ver: &mut MuFunctionVersion) {
        debug!("{}", self.name());

        self.current_fv_id = func_ver.id();
        self.current_frame = Some(Frame::new(func_ver.id()));
        self.current_func_start = Some({
            let funcs = vm.funcs().read().unwrap();
            let func = funcs.get(&func_ver.func_id).unwrap().read().unwrap();
            self.backend.start_code(func.name().unwrap())        
        });
        self.current_callsite_id = 0;
        self.current_exn_callsites.clear();
        self.current_exn_blocks.clear();

        self.current_constants.clear();
        self.current_constants_locs.clear();
        
        // prologue (get arguments from entry block first)        
        let entry_block = func_ver.content.as_ref().unwrap().get_entry_block();
        let ref args = entry_block.content.as_ref().unwrap().args;
        self.emit_common_prologue(args, vm);
    }

    #[allow(unused_variables)]
    fn visit_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        let f_content = func.content.as_ref().unwrap();
        
        for block_id in func.block_trace.as_ref().unwrap() {
            let block = f_content.get_block(*block_id);
            let block_label = block.name().unwrap();
            self.current_block = Some(block_label.clone());            
            
            let block_content = block.content.as_ref().unwrap();
            
            if block.is_exception_block() {
                let loc = self.backend.start_exception_block(block_label.clone());
                self.current_exn_blocks.insert(block.id(), loc);
                
                let exception_arg = block_content.exn_arg.as_ref().unwrap();
                
                // live in is args of the block + exception arg
                let mut livein = block_content.args.to_vec();
                livein.push(exception_arg.clone());
                self.backend.set_block_livein(block_label.clone(), &livein);
                
                // need to insert a landing pad
                self.emit_landingpad(&exception_arg, f_content, &mut func.context, vm);
            } else {
                self.backend.start_block(block_label.clone());
                
                // live in is args of the block
                self.backend.set_block_livein(block_label.clone(), &block_content.args);                    
            }
            
            // live out is the union of all branch args of this block
            let live_out = block_content.get_out_arguments();

            for inst in block_content.body.iter() {
                self.instruction_select(&inst, f_content, &mut func.context, vm);
            }
            
            // we may start block a, and end with block b (instruction selection may create blocks)
            // we set liveout to current block 
            {
                let current_block = self.current_block.as_ref().unwrap();
                self.backend.set_block_liveout(current_block.clone(), &live_out);
                self.backend.end_block(current_block.clone());
            }            
            self.current_block = None;
        }
    }
    
    #[allow(unused_variables)]
    fn finish_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        self.backend.print_cur_code();
        
        let func_name = {
            let funcs = vm.funcs().read().unwrap();
            let func = funcs.get(&func.func_id).unwrap().read().unwrap();
            func.name().unwrap()
        };
        
        let (mc, func_end) = self.backend.finish_code(func_name.clone());
        
        // insert exception branch info
        let mut frame = match self.current_frame.take() {
            Some(frame) => frame,
            None => panic!("no current_frame for function {} that is being compiled", func_name)
        };
        for block_id in self.current_exn_blocks.keys() {
            let block_loc = match self.current_exn_blocks.get(&block_id) {
                Some(loc) => loc,
                None => panic!("failed to find exception block {}", block_id)
            };
            let callsites = match self.current_exn_callsites.get(&block_id) {
                Some(callsite) => callsite,
                None => panic!("failed to find callsite for block {}", block_id)
            };
            
            for callsite in callsites {
                frame.add_exception_callsite(callsite.clone(), block_loc.clone());
            }
        }
        
        let compiled_func = CompiledFunction::new(func.func_id, func.id(), mc,
                                                  self.current_constants.clone(), self.current_constants_locs.clone(),
                                                  frame, self.current_func_start.take().unwrap(), func_end);
        
        vm.add_compiled_func(compiled_func);
    }
}
