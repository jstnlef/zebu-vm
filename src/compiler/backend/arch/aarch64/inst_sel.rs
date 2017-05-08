#![allow(unused_variables)]
#![warn(unused_imports)]
#![warn(unreachable_code)]
#![warn(dead_code)]
use ast::ir::*;
use ast::ptr::*;
use ast::inst::*;
use ast::op;
use ast::op::*;
use ast::types;
use ast::types::*;
use vm::VM;
use runtime::mm;
use runtime::mm::objectmodel::OBJECT_HEADER_SIZE;

use runtime::ValueLocation;
use runtime::thread;
use runtime::entrypoints;
use runtime::entrypoints::RuntimeEntrypoint;

use compiler::CompilerPass;

use compiler::backend::PROLOGUE_BLOCK_NAME;

use compiler::backend::aarch64::*;
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

const INLINE_FASTPATH : bool = false;

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
        InstructionSelection {
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
    // * we do not need to backup/restore caller-saved registers
    // if any of these assumption breaks, we will need to re-emit the code
    fn aarch64_instruction_select(&mut self, node: &'a TreeNode, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) {
        trace!("instsel on node#{} {}", node.id(), node);

        match node.v {
            TreeNode_::Instruction(ref inst) => {
                match inst.v {
                    // aarch64
                    Instruction_::Branch2 { cond, ref true_dest, ref false_dest, true_prob } => {
                        // TODO: How does control flow pass to fallthrough_dest ???
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

                        let mut cmpop = if self.match_cmp_res(cond) {
                            trace!("emit cmp_res-branch2");
                            self.aarch64_emit_cmp_res(cond, f_content, f_context, vm)
                        } else {
                            let cond_reg = self.aarch64_emit_ireg(cond, f_content, f_context, vm);
                            self.backend.emit_cmp_imm(&cond_reg, 0, false);
                            op::CmpOp::NE
                        };

                        if !branch_if_true {
                            cmpop = cmpop.invert();
                        }

                        let cond = get_condition_codes(cmpop);

                        if cmpop == op::CmpOp::FFALSE {
                            ; // Do nothing
                        } else if cmpop == op::CmpOp::FTRUE {
                            self.backend.emit_b(branch_target);
                        } else {
                            self.backend.emit_b_cond(cond[0], branch_target.clone());

                            if cond.len() == 2 {
                                self.backend.emit_b_cond(cond[1], branch_target);
                            }
                        }
                    },

                    // aarch64
                    Instruction_::Select { cond, true_val, false_val } => {
                        use ast::op::CmpOp::*;

                        trace!("instsel on SELECT");
                        let ops = inst.ops.read().unwrap();

                        let ref cond = ops[cond];
                        let ref true_val = ops[true_val];
                        let ref false_val = ops[false_val];

                        let tmp_res = self.get_result_value(node);

                        // moving integers/pointers
                        // generate compare
                        let cmpop = if self.match_cmp_res(cond) {
                            self.aarch64_emit_cmp_res(cond, f_content, f_context, vm)
                        } else if self.match_ireg(cond) {
                            let tmp_cond = self.aarch64_emit_ireg(cond, f_content, f_context, vm);
                            self.backend.emit_cmp_imm(&tmp_cond, 0, false);
                            NE
                        } else {
                            panic!("expected ireg, found {}", cond)
                        };

                        let tmp_true = self.aarch64_emit_reg(true_val, f_content, f_context, vm);
                        let tmp_false = self.aarch64_emit_reg(false_val, f_content, f_context, vm);

                        let cond = get_condition_codes(cmpop);

                        if self.match_ireg(true_val) {
                            if cmpop == FFALSE {
                                self.backend.emit_mov(&tmp_res, &tmp_false);
                            } else if cmpop == FTRUE {
                                self.backend.emit_mov(&tmp_res, &tmp_true);
                            } else {
                                self.backend.emit_csel(&tmp_res, &tmp_true, &tmp_false, cond[0]);

                                if cond.len() == 2 {
                                    self.backend.emit_csel(&tmp_res, &tmp_true, &tmp_res, cond[1]);
                                }
                            }
                        } else if self.match_fpreg(true_val) {
                            if cmpop == FFALSE {
                                self.backend.emit_fmov(&tmp_res, &tmp_false);
                            } else if cmpop == FTRUE {
                                self.backend.emit_fmov(&tmp_res, &tmp_true);
                            } else {
                                self.backend.emit_fcsel(&tmp_res, &tmp_true, &tmp_false, cond[0]);

                                if cond.len() == 2 {
                                    self.backend.emit_fcsel(&tmp_res, &tmp_true, &tmp_res, cond[1]);
                                }
                            }
                        } else {
                            // moving vectors, floatingpoints
                            unimplemented!()
                        }
                    },

                    // aarch64
                    Instruction_::CmpOp(op, op1, op2) => {
                        use ast::op::CmpOp::*;

                        trace!("instsel on CMPOP");
                        let ops = inst.ops.read().unwrap();
                        let ref op1 = ops[op1];
                        let ref op2 = ops[op2];

                        let tmp_res = self.get_result_value(node);

                        debug_assert!(tmp_res.ty.get_int_length().is_some());
                        debug_assert!(tmp_res.ty.get_int_length().unwrap() == 1);

                        let cmpop = self.aarch64_emit_cmp_res_op(op, &op1, &op2, f_content, f_context, vm);
                        let cond = get_condition_codes(cmpop);

                        if cmpop == FFALSE {
                            self.aarch64_emit_mov_u64(&tmp_res, 0);
                        } else if cmpop == FTRUE {
                            self.aarch64_emit_mov_u64(&tmp_res, 1);
                        } else {
                            self.backend.emit_cset(&tmp_res, cond[0]);

                            // Note: some compariosns can't be computed based on a single aarch64 flag
                            // insted they are computed as a condition OR NOT another condition.
                            if cond.len() == 2 {
                                self.backend.emit_csinc(&tmp_res, &tmp_res, &WZR, invert_condition_code(cond[1]));
                            }
                        }
                    }

                    // aarch64
                    Instruction_::Branch1(ref dest) => {
                        trace!("instsel on BRANCH1");
                        let ops = inst.ops.read().unwrap();

                        self.process_dest(&ops, dest, f_content, f_context, vm);

                        let target = f_content.get_block(dest.target).name().unwrap();

                        trace!("emit branch1");
                        // jmp
                        self.backend.emit_b(target);
                    },

                    // aarch64
                    Instruction_::Switch { cond, ref default, ref branches } => {
                        trace!("instsel on SWITCH");
                        let ops = inst.ops.read().unwrap();

                        let ref cond = ops[cond];

                        if self.match_ireg(cond) {
                            let tmp_cond = self.aarch64_emit_ireg(cond, f_content, f_context, vm);
                            self.aarch64_emit_zext(&tmp_cond);

                            // emit each branch
                            for &(case_op_index, ref case_dest) in branches {
                                let ref case_op = ops[case_op_index];

                                // process dest
                                self.process_dest(&ops, case_dest, f_content, f_context, vm);

                                let target = f_content.get_block(case_dest.target).name().unwrap();

                                let mut imm_val = 0 as u64;
                                // Is one of the arguments a valid immediate?
                                let emit_imm = if self.aarch64_match_node_iimm(&case_op) {
                                    imm_val = self.aarch64_node_iimm_to_u64(&case_op);
                                    is_valid_arithmetic_imm(imm_val)
                                } else {
                                    false
                                };

                                if emit_imm {
                                    let imm_shift = imm_val > 4096;
                                    let imm_op2 = if imm_shift { imm_val >> 12 } else { imm_val };

                                    self.backend.emit_cmp_imm(&tmp_cond, imm_op2 as u16, imm_shift);
                                } else {
                                    let tmp_case_op = self.aarch64_emit_ireg(case_op, f_content, f_context, vm);
                                    self.aarch64_emit_zext(&tmp_case_op);
                                    self.backend.emit_cmp(&tmp_cond, &tmp_case_op);
                                }

                                self.backend.emit_b_cond("EQ", target);

                                self.finish_block();
                                self.start_block(format!("{}_switch_not_met_case_{}", node.id(), case_op_index));
                            }

                            // emit default
                            self.process_dest(&ops, default, f_content, f_context, vm);

                            let default_target = f_content.get_block(default.target).name().unwrap();
                            self.backend.emit_b(default_target);
                        } else {
                            panic!("expecting cond in switch to be ireg: {}", cond);
                        }
                    }

                    // aarch64
                    Instruction_::ExprCall { ref data, is_abort } => {
                        trace!("instsel on EXPRCALL");

                        if is_abort {
                            unimplemented!()
                        }

                        self.aarch64_emit_mu_call(
                            inst, // inst: &Instruction,
                            data, // calldata: &CallData,
                            None, // resumption: Option<&ResumptionData>,
                            node, // cur_node: &TreeNode, 
                            f_content, f_context, vm);
                    },

                    // aarch64
                    Instruction_::Call { ref data, ref resume } => {
                        trace!("instsel on CALL");

                        self.aarch64_emit_mu_call(
                            inst,
                            data,
                            Some(resume),
                            node,
                            f_content, f_context, vm);
                    },

                    // aarch64
                    Instruction_::ExprCCall { ref data, is_abort } => {
                        trace!("instsel on EXPRCCALL");

                        if is_abort {
                            unimplemented!()
                        }

                        self.aarch64_emit_c_call_ir(inst, data, None, node, f_content, f_context, vm);
                    }

                    // aarch64
                    Instruction_::CCall { ref data, ref resume } => {
                        trace!("instsel on CCALL");

                        self.aarch64_emit_c_call_ir(inst, data, Some(resume), node, f_content, f_context, vm);
                    }

                    // aarch64
                    Instruction_::Return(_) => {
                        trace!("instsel on RETURN");

                        self.aarch64_emit_common_epilogue(inst, f_content, f_context, vm);

                        self.backend.emit_ret(&LR); // return to the Link Register
                    },

                    // aarch64
                    Instruction_::BinOp(op, op1, op2) => {
                        trace!("instsel on BINOP");
                        self.aarch64_emit_binop(node, inst, op, BinOpStatus { flag_n: false, flag_z: false, flag_c: false, flag_v: false }, op1, op2, f_content, f_context, vm);
                    },

                    // aarch64
                    Instruction_::BinOpWithStatus(op, status, op1, op2) => {
                        trace!("instsel on BINOP_STATUS");
                        self.aarch64_emit_binop(node, inst, op, status, op1, op2, f_content, f_context, vm);
                    }

                    // aarch64
                    Instruction_::ConvOp { operation, ref from_ty, ref to_ty, operand } => {
                        trace!("instsel on CONVOP");

                        let ops = inst.ops.read().unwrap();

                        let ref op = ops[operand];

                        let tmp_res = self.get_result_value(node);
                        let tmp_op = self.aarch64_emit_reg(op, f_content, f_context, vm);

                        let from_ty_size = get_bit_size(&from_ty, vm);
                        let to_ty_size = get_bit_size(&to_ty, vm);

                        match operation {
                            op::ConvOp::TRUNC => {
                                self.backend.emit_mov(&tmp_res, unsafe { &tmp_op.as_type(tmp_res.ty.clone()) });
                            },
                            op::ConvOp::ZEXT => {
                                if from_ty_size != to_ty_size {
                                    self.backend.emit_ubfx(&tmp_res, unsafe { &tmp_op.as_type(tmp_res.ty.clone()) }, 0, from_ty_size as u8);
                                } else {
                                    self.backend.emit_mov(&tmp_res, &tmp_op);
                                }
                            },
                            op::ConvOp::SEXT => {
                                if from_ty_size != to_ty_size {
                                    self.backend.emit_sbfx(&tmp_res, unsafe { &tmp_op.as_type(tmp_res.ty.clone()) }, 0, from_ty_size as u8);
                                } else {
                                    self.backend.emit_mov(&tmp_res, &tmp_op);
                                }
                            },
                            op::ConvOp::REFCAST | op::ConvOp::PTRCAST => {
                                // just a mov (and hopefully reg alloc will coalesce it)
                                self.backend.emit_mov(&tmp_res, &tmp_op);
                            },

                            op::ConvOp::UITOFP => {
                                self.backend.emit_ucvtf(&tmp_res, &tmp_op);
                            },

                            op::ConvOp::SITOFP => {
                                self.backend.emit_scvtf(&tmp_res, &tmp_op);
                            },

                            op::ConvOp::FPTOUI => {
                                self.backend.emit_fcvtzu(&tmp_res, &tmp_op);
                            },

                            op::ConvOp::FPTOSI => {
                                self.backend.emit_fcvtzs(&tmp_res, &tmp_op);
                            },

                            op::ConvOp::BITCAST => {
                                self.backend.emit_fmov(&tmp_res, &tmp_op);
                            },
                            op::ConvOp::FPTRUNC | op::ConvOp::FPEXT => {
                                self.backend.emit_fcvt(&tmp_res, &tmp_op);
                            },
                        }
                    }

                    // aarch64
                    Instruction_::Load { is_ptr, order, mem_loc } => {
                        trace!("instsel on LOAD");
                        let ops = inst.ops.read().unwrap();
                        let ref loc_op = ops[mem_loc];

                        // Whether to use a load acquire
                        let use_acquire = match order {
                            MemoryOrder::Relaxed | MemoryOrder::NotAtomic => false,
                            MemoryOrder::Consume | MemoryOrder::Acquire | MemoryOrder::SeqCst => true,
                            _ => panic!("didnt expect order {:?} with load inst", order)
                        };

                        let resolved_loc = self.aarch64_emit_node_addr_to_value(loc_op, f_content, f_context, vm);
                        let res_temp = self.get_result_value(node);

                        if use_acquire {
                            // Can only have a base for a LDAR
                            let temp_loc = self.aarch64_emit_mem_base(&resolved_loc, f_context, vm);
                            self.backend.emit_ldar(&res_temp, &temp_loc);
                        } else {
                            let temp_loc = self.aarch64_emit_mem(&resolved_loc, f_context, vm);
                            self.backend.emit_ldr(&res_temp, &temp_loc, false);
                        }
                    }

                    // aarch64
                    Instruction_::Store { is_ptr, order, mem_loc, value } => {
                        trace!("instsel on STORE");
                        let ops = inst.ops.read().unwrap();
                        let ref loc_op = ops[mem_loc];
                        let ref val_op = ops[value];

                        // Whether to use a store release or not
                        let use_release = match order {
                            MemoryOrder::Relaxed | MemoryOrder::NotAtomic => false,
                            MemoryOrder::Release | MemoryOrder::SeqCst => true,
                            _ => panic!("didnt expect order {:?} with load inst", order)
                        };

                        let resolved_loc = self.aarch64_emit_node_addr_to_value(loc_op, f_content, f_context, vm);
                        let val = self.aarch64_emit_reg(val_op, f_content, f_context, vm);

                        if use_release {
                            // Can only have a base for a STLR
                            let temp_loc = self.aarch64_emit_mem_base(&resolved_loc, f_context, vm);
                            self.backend.emit_stlr(&temp_loc, &val);
                        } else {
                            let temp_loc = self.aarch64_emit_mem(&resolved_loc, f_context, vm);
                            self.backend.emit_str(&temp_loc, &val);
                        }
                    }

                    // aarch64
                    Instruction_::GetIRef(_)
                    | Instruction_::GetFieldIRef { .. }
                    | Instruction_::GetElementIRef{..}
                    | Instruction_::GetVarPartIRef { .. }
                    | Instruction_::ShiftIRef { .. } => {
                        trace!("instsel on GET/FIELD/VARPARTIREF, SHIFTIREF");
                        let mem_addr = self.aarch64_emit_get_mem_from_inst(node, f_content, f_context, vm);
                        let tmp_res = self.get_result_value(node);
                        self.aarch64_emit_calculate_address(&tmp_res, &mem_addr, f_context, vm);
                    }

                    // aarch64
                    Instruction_::Fence(order) => {
                        trace!("instsel on FENCE");

                        // Whether to emit a load fence or a normal one
                        let use_load = match order {
                            MemoryOrder::Release | MemoryOrder::SeqCst | MemoryOrder::AcqRel => false,
                            MemoryOrder::Acquire => true,
                            _ => panic!("didnt expect order {:?} with load inst", order)
                        };

                        if use_load {
                            // Data Memory Barrirer for Inner Shariable Domain (for Load accesses only)
                            self.backend.emit_dmb("ISHLD");
                        } else {
                            // Data Memory Barrirer for Inner Shariable Domain
                            self.backend.emit_dmb("ISH");
                        }
                    }
                    // aarch64
                    Instruction_::ThreadExit => {
                        trace!("instsel on THREADEXIT");
                        // emit a call to swap_back_to_native_stack(sp_loc: Address)

                        // get thread local and add offset to get sp_loc
                        let tl = self.aarch64_emit_get_threadlocal(Some(node), f_content, f_context, vm);
                        self.backend.emit_add_imm(&tl, &tl, *thread::NATIVE_SP_LOC_OFFSET as u16, false);

                        self.aarch64_emit_runtime_entry(&entrypoints::SWAP_BACK_TO_NATIVE_STACK, vec![tl.clone()], None, Some(node), f_content, f_context, vm);
                    }

                    // aarch64
                    Instruction_::CommonInst_GetThreadLocal => {
                        trace!("instsel on GETTHREADLOCAL");
                        // get thread local
                        let tl = self.aarch64_emit_get_threadlocal(Some(node), f_content, f_context, vm);

                        let tmp_res = self.get_result_value(node);

                        // load [tl + USER_TLS_OFFSET] -> tmp_res
                        self.aarch64_emit_load_base_offset(&tmp_res, &tl, *thread::USER_TLS_OFFSET as i64, f_context, vm);
                    }

                    // aarch64
                    Instruction_::CommonInst_SetThreadLocal(op) => {
                        trace!("instsel on SETTHREADLOCAL");
                        let ops = inst.ops.read().unwrap();
                        let ref op = ops[op];

                        debug_assert!(self.match_ireg(op));

                        let tmp_op = self.aarch64_emit_ireg(op, f_content, f_context, vm);

                        // get thread local
                        let tl = self.aarch64_emit_get_threadlocal(Some(node), f_content, f_context, vm);

                        // store tmp_op -> [tl + USER_TLS_OFFSTE]
                        self.aarch64_emit_store_base_offset(&tl, *thread::USER_TLS_OFFSET as i64, &tmp_op, f_context, vm);
                    }

                    // aarch64
                    Instruction_::CommonInst_Pin(op) => {
                        trace!("instsel on PIN");
                        if !mm::GC_MOVES_OBJECT {
                            // non-moving GC: pin is a nop (move from op to result)
                            let ops = inst.ops.read().unwrap();
                            let ref op = ops[op];

                            let tmp_res = self.get_result_value(node);

                            self.aarch64_emit_move_node_to_value(&tmp_res, op, f_content, f_context, vm);
                        } else {
                            unimplemented!()
                        }
                    }

                    // aarch64
                    Instruction_::CommonInst_Unpin(_) => {
                        trace!("instsel on UNPIN");
                        if !mm::GC_MOVES_OBJECT {
                            // do nothing
                        } else {
                            unimplemented!()
                        }
                    }

                    // aarch64
                    Instruction_::Move(op) => {
                        trace!("instsel on MOVE (internal IR)");
                        let ops = inst.ops.read().unwrap();
                        let ref op = ops[op];

                        let tmp_res = self.get_result_value(node);

                        self.aarch64_emit_move_node_to_value(&tmp_res, op, f_content, f_context, vm);
                    }

                    // AARCH64
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
                        let ty_align = ty_info.alignment;

                        let const_size = self.aarch64_make_value_int_const(size as u64, vm);

                        let tmp_allocator = self.aarch64_emit_get_allocator(node, f_content, f_context, vm);
                        let tmp_res = self.aarch64_emit_alloc_sequence(tmp_allocator.clone(), const_size, ty_align, node, f_content, f_context, vm);

                        // ASM: call muentry_init_object(%allocator, %tmp_res, %encode)
                        let encode = self.aarch64_make_value_int_const(mm::get_gc_type_encode(ty_info.gc_type.id), vm);
                        self.aarch64_emit_runtime_entry(
                            &entrypoints::INIT_OBJ,
                            vec![tmp_allocator.clone(), tmp_res.clone(), encode],
                            None,
                            Some(node), f_content, f_context, vm
                        );
                    }

                    // aarch64
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

                            if self.aarch64_match_node_iimm(var_len) {
                                let var_len = self.aarch64_node_iimm_to_u64(var_len);
                                let actual_size = fix_part_size + var_ty_size * (var_len as usize);
                                (
                                    self.aarch64_make_value_int_const(actual_size as u64, vm),
                                    self.aarch64_make_value_int_const(var_len as u64, vm)
                                )
                            } else {
                                let tmp_actual_size = self.make_temporary(f_context, UINT64_TYPE.clone(), vm);
                                let tmp_var_len = self.aarch64_emit_ireg(var_len, f_content, f_context, vm);

                                // tmp_actual_size = tmp_var_len*var_ty_size
                                if var_ty_size.is_power_of_two() {
                                    self.backend.emit_lsl_imm(&tmp_actual_size, &tmp_var_len, log2(var_ty_size as u64) as u8);
                                } else {
                                    let temp_mul = self.make_temporary(f_context, UINT64_TYPE.clone(), vm);
                                    self.aarch64_emit_mov_u64(&tmp_actual_size, var_ty_size as u64);
                                    self.backend.emit_mul(&tmp_actual_size, &tmp_var_len, &temp_mul);
                                }
                                // tmp_actual_size = tmp_var_len*var_ty_size + fix_part_size
                                self.aarch64_emit_add_u64(&tmp_actual_size, &tmp_actual_size, f_context, vm, fix_part_size as u64);
                                (tmp_actual_size, tmp_var_len)
                            }
                        };

                        let tmp_allocator = self.aarch64_emit_get_allocator(node, f_content, f_context, vm);
                        let tmp_res = self.aarch64_emit_alloc_sequence(tmp_allocator.clone(), actual_size, ty_align, node, f_content, f_context, vm);

                        // ASM: call muentry_init_object(%allocator, %tmp_res, %encode)
                        let encode = self.aarch64_make_value_int_const(mm::get_gc_type_encode(ty_info.gc_type.id), vm);
                        self.aarch64_emit_runtime_entry(
                            &entrypoints::INIT_HYBRID,
                            vec![tmp_allocator.clone(), tmp_res.clone(), encode, length],
                            None,
                            Some(node), f_content, f_context, vm
                        );
                    }

                    // Runtime Entry
                    Instruction_::Throw(op_index) => {
                        trace!("instsel on THROW");
                        let ops = inst.ops.read().unwrap();
                        let ref exception_obj = ops[op_index];

                        self.aarch64_emit_runtime_entry(
                            &entrypoints::THROW_EXCEPTION,
                            vec![exception_obj.clone_value()],
                            None,
                            Some(node), f_content, f_context, vm);
                    }

                    // Runtime Entry
                    Instruction_::PrintHex(index) => {
                        trace!("instsel on PRINTHEX");
                        let ops = inst.ops.read().unwrap();
                        let ref op = ops[index];

                        self.aarch64_emit_runtime_entry(
                            &entrypoints::PRINT_HEX,
                            vec![op.clone_value()],
                            None,
                            Some(node), f_content, f_context, vm
                        );
                    }

                    _ => unimplemented!()
                } // main switch
            },

            TreeNode_::Value(_) => {}
        }
    }

    fn make_temporary(&mut self, f_context: &mut FunctionContext, ty: P<MuType>, vm: &VM) -> P<Value> {
        f_context.make_temporary(vm.next_id(), ty).clone_value()
    }


    fn aarch64_make_value_int_const(&mut self, val: u64, vm: &VM) -> P<Value> {
        P(Value {
            hdr: MuEntityHeader::unnamed(vm.next_id()),
            ty: UINT64_TYPE.clone(),
            v: Value_::Constant(Constant::Int(val))
        })
    }

    // TODO: Deleted f_context
    fn aarch64_make_value_base_offset(&mut self, base: &P<Value>, offset: i64, ty: &P<MuType>, vm: &VM) -> P<Value> {
        let mem = self.aarch64_make_memory_location_base_offset(base, offset, vm);
        self.aarch64_make_value_from_memory(mem, ty, vm)
    }

    fn aarch64_make_value_from_memory(&mut self, mem: MemoryLocation, ty: &P<MuType>, vm: &VM) -> P<Value> {
        P(Value {
            hdr: MuEntityHeader::unnamed(vm.next_id()),
            ty: ty.clone(),
            v: Value_::Memory(mem)
        })
    }

    fn aarch64_make_memory_location_base_offset(&mut self, base: &P<Value>, offset: i64, vm: &VM) -> MemoryLocation {
        if offset == 0 {
            MemoryLocation::VirtualAddress{
                base: base.clone(),
                offset: None,
                scale: 1,
                signed: true,
            }
        } else {
            MemoryLocation::VirtualAddress{
                base: base.clone(),
                offset: Some(self.aarch64_make_value_int_const(offset as u64, vm)),
                scale: 1,
                signed: true,
            }
        }
    }

    fn aarch64_emit_mem(&mut self, pv: &P<Value>, f_context: &mut FunctionContext, vm: &VM) -> P<Value> {
        let n = vm.get_backend_type_info(pv.ty.id()).alignment;
        match pv.v {
            Value_::Memory(ref mem) => {
                match mem {
                    &MemoryLocation::VirtualAddress{ref base, ref offset, scale, signed} => {
                        let mut shift = 0 as u8;
                        let offset =
                            if offset.is_some() {
                                let offset = offset.as_ref().unwrap();
                                if self.aarch64_match_value_iimm(offset) {
                                    let mut offset_val = self.aarch64_value_iimm_to_i64(offset);
                                    offset_val *= scale as i64;
                                    if is_valid_immediate_offset(offset_val, n) {
                                        Some(self.aarch64_make_value_int_const(offset_val as u64, vm))
                                    } else {
                                        let offset = self.make_temporary(f_context, UINT64_TYPE.clone(), vm);
                                        self.aarch64_emit_mov_u64(&offset, offset_val as u64);
                                        Some(offset)
                                    }
                                } else {
                                    let offset = self.aarch64_emit_ireg_value(offset, f_context, vm);

                                    // TODO: If scale == n*m (for some m), set shift = n, and multiply index by m
                                    if !is_valid_immediate_scale(scale, n) {
                                        let temp = self.make_temporary(f_context, offset.ty.clone(), vm);

                                        // TODO: Will this be correct if offset is treated as signed (i think so...)
                                        if scale.is_power_of_two() {
                                            self.backend.emit_lsl_imm(&temp, &offset, log2(scale as u64) as u8);
                                        } else {
                                            let temp_mul = self.make_temporary(f_context, offset.ty.clone(), vm);
                                            self.aarch64_emit_mov_u64(&temp_mul, scale as u64);
                                            self.backend.emit_mul(&temp, &offset, &temp_mul);
                                        }

                                        Some(temp)
                                    } else {
                                        shift = log2(scale) as u8;
                                        Some(offset)
                                    }
                                }
                            }
                            else {
                                None
                            };

                        P(Value {
                            hdr: MuEntityHeader::unnamed(vm.next_id()),
                            ty: pv.ty.clone(),
                            v: Value_::Memory(MemoryLocation::Address {
                                base: base.clone(),
                                offset: offset,
                                shift: shift,
                                signed: signed
                            })
                        })
                    }
                    _ => pv.clone()
                }
            }
            _ => panic!("expected memory")
        }
    }

    #[warn(unused_variables)] // Same as emit_mem except returns a memory location with only a base
    // NOTE: This code duplicates allot of code in aarch64_emit_mem and aarch64_emit_calculate_address
    fn aarch64_emit_mem_base(&mut self, pv: &P<Value>, f_context: &mut FunctionContext, vm: &VM) -> P<Value> {
        match pv.v {
            Value_::Memory(ref mem) => {
                let base = match mem {
                    &MemoryLocation::VirtualAddress{ref base, ref offset, scale, signed} => {
                        if offset.is_some() {
                            let offset = offset.as_ref().unwrap();
                            if self.aarch64_match_value_iimm(offset) {
                                let offset_val = self.aarch64_value_iimm_to_i64(offset);
                                if offset_val == 0 {
                                    base.clone() // trivial
                                } else {
                                    let temp = self.make_temporary(f_context, ADDRESS_TYPE.clone(), vm);
                                    self.aarch64_emit_add_u64(&temp, &base, f_context, vm, (offset_val * scale as i64) as u64);
                                    temp
                                }
                            } else {
                                let offset = self.aarch64_emit_ireg_value(offset, f_context, vm);

                                // TODO: If scale == r*m (for some 0 <= m <= 4), multiply offset by r
                                // then use and add_ext(,...,m)
                                if scale.is_power_of_two() && is_valid_immediate_extension(log2(scale)) {
                                    let temp = self.make_temporary(f_context, ADDRESS_TYPE.clone(), vm);
                                    self.backend.emit_add_ext(&temp, &base, &offset, signed, log2(scale) as u8);
                                    temp
                                } else {
                                    let temp_offset = self.make_temporary(f_context, offset.ty.clone(), vm);

                                    // TODO: Will this be correct if offset is treated as signed (i think so...)
                                    if scale.is_power_of_two() {
                                        self.backend.emit_lsl_imm(&temp_offset, &offset, log2(scale as u64) as u8);
                                    } else {
                                        let temp_mul = self.make_temporary(f_context, offset.ty.clone(), vm);
                                        self.aarch64_emit_mov_u64(&temp_mul, scale as u64);
                                        self.backend.emit_mul(&temp_offset, &offset, &temp_mul);
                                    }

                                    // Don't need to create a new register, just overwrite temp_offset
                                    let temp = unsafe { temp_offset.as_type(ADDRESS_TYPE.clone()) };
                                    // Need to use add_ext, in case offset is 32-bits
                                    self.backend.emit_add_ext(&temp, &base, &temp_offset, signed, 0);
                                    temp
                                }
                            }
                        }
                        else {
                            base.clone() // trivial
                        }
                    }
                    &MemoryLocation::Address{ref base, ref offset, shift, signed} => {
                        if offset.is_some() {
                            let ref offset = offset.as_ref().unwrap();

                            if self.aarch64_match_value_iimm(&offset) {
                                let offset = self.aarch64_value_iimm_to_u64(&offset);
                                if offset == 0 {
                                    // Offset is 0, it can be ignored
                                    base.clone()
                                } else {
                                    let temp = self.make_temporary(f_context, ADDRESS_TYPE.clone(), vm);
                                    self.aarch64_emit_add_u64(&temp, &base, f_context, vm, offset as u64);
                                    temp
                                }
                            } else if offset.is_int_reg() {
                                let temp = self.make_temporary(f_context, ADDRESS_TYPE.clone(), vm);
                                self.backend.emit_add_ext(&temp, &base, &offset, signed, shift);
                                temp
                            } else {
                                panic!("Offset should be an integer register or a constant")
                            }
                        } else {
                            // Simple base address
                            base.clone()
                        }
                    }
                    &MemoryLocation::Symbolic{..} => {
                        let temp = self.make_temporary(f_context, ADDRESS_TYPE.clone(), vm);
                        self.backend.emit_adr(&temp, &pv);
                        temp
                    },
                };

                P(Value {
                    hdr: MuEntityHeader::unnamed(vm.next_id()),
                    ty: pv.ty.clone(),
                    v: Value_::Memory(MemoryLocation::Address {
                        base: base.clone(),
                        offset: None,
                        shift: 0,
                        signed: false
                    })
                })
            }
            _ => panic!("expected memory")
        }
    }

    fn aarch64_make_memory_location_base_offset_scale(&mut self, base: &P<Value>, offset: &P<Value>, scale: u64, signed: bool) -> MemoryLocation {
        MemoryLocation::VirtualAddress{
            base: base.clone(),
            offset: Some(offset.clone()),
            scale: scale,
            signed: signed
        }
    }

    // Returns a memory location pointing to 'base + (offset+more_offset)*scale'
    /*fn aarch64_memory_location_adjust_offset(&mut self, mem: MemoryLocation, more_offset: i64, f_context: &mut FunctionContext, vm: &VM) -> MemoryLocation {
        match mem {
            MemoryLocation::VirtualAddress { base, offset, scale, signed } => {
                let offset =
                    if offset.is_some() {
                        let offset = offset.unwrap();
                        if self.aarch64_match_value_iimm(&offset) {
                            let offset = offset.extract_int_const() + (more_offset as u64);
                            self.aarch64_make_value_int_const(offset as u64, vm)
                        } else {
                            let temp = self.make_temporary(f_context, offset.ty.clone(), vm);
                            let offset = self.aarch64_emit_ireg_value(&offset, f_context, vm);
                            self.aarch64_emit_add_u64(&temp, &offset, f_context, vm, more_offset as u64);
                            temp
                        }
                    }
                    else {
                        self.aarch64_make_value_int_const(more_offset as u64, vm)
                    };
                MemoryLocation::VirtualAddress {
                    base: base.clone(),
                    offset: Some(offset),
                    scale: scale,
                    signed: signed,
                }

            },
            _ => panic!("expected a VirtualAddress memory location")
        }
    }*/
    // Returns a memory location that points to 'Base + offset*scale + more_offset'
    fn aarch64_memory_location_shift(&mut self, mem: MemoryLocation, more_offset: i64, f_context: &mut FunctionContext, vm: &VM) -> MemoryLocation {
        match mem {
            MemoryLocation::VirtualAddress { base, offset, scale, signed } => {
                let offset =
                    if offset.is_some() {
                        let offset = offset.unwrap();
                        if self.aarch64_match_value_iimm(&offset) {
                            let offset = offset.extract_int_const()*scale + (more_offset as u64);
                            self.aarch64_make_value_int_const(offset as u64, vm)
                        } else {
                            let offset = self.aarch64_emit_ireg_value(&offset, f_context, vm);
                            let temp = self.make_temporary(f_context, offset.ty.clone(), vm);

                            if scale == 1 {
                                // do nothing, temp = offset*scale
                                self.backend.emit_mov(&temp, &offset);
                            } else if scale.is_power_of_two() {
                                // temp = offset << log2(scale)
                                self.backend.emit_lsl_imm(&temp, &offset, log2(scale as u64) as u8);
                            } else {
                                // temp = offset * scale
                                let temp_mul = self.make_temporary(f_context, offset.ty.clone(), vm);
                                self.aarch64_emit_mov_u64(&temp_mul, scale as u64);
                                self.backend.emit_mul(&temp, &offset, &temp_mul);
                            }

                            self.aarch64_emit_add_u64(&temp, &temp, f_context, vm, more_offset as u64);
                            temp
                        }
                    }
                    else {
                        self.aarch64_make_value_int_const(more_offset as u64, vm)
                    };
                MemoryLocation::VirtualAddress {
                    base: base.clone(),
                    offset: Some(offset),
                    scale: 1,
                    signed: signed,
                }

            },
            _ => panic!("expected a VirtualAddress memory location")
        }
    }

    // Returns a memory location that points to 'Base + offset*scale + more_offset*new_scale'
    fn aarch64_memory_location_shift_scale(&mut self, mem: MemoryLocation, more_offset:  &P<Value>, new_scale: u64, f_context: &mut FunctionContext, vm: &VM) -> MemoryLocation {
        if self.aarch64_match_value_iimm(&more_offset) {
            let more_offset = self.aarch64_value_iimm_to_i64(&more_offset);
            let mem = self.aarch64_memory_location_shift(mem, more_offset, f_context, vm);
            self.aarch64_memory_location_append_scale(mem, new_scale)
        } else {
            match mem {
                MemoryLocation::VirtualAddress { base, offset, scale, signed } => {
                    let offset =
                        if offset.is_some() {
                            let offset = offset.unwrap();
                            if self.aarch64_match_value_iimm(&offset) {
                                let temp = self.make_temporary(f_context, offset.ty.clone(), vm);
                                self.aarch64_emit_add_u64(&temp, &more_offset, f_context, vm, offset.extract_int_const() * scale);
                                temp
                            } else {
                                let offset = self.aarch64_emit_ireg_value(&offset, f_context, vm);
                                let temp = self.make_temporary(f_context, offset.ty.clone(), vm);

                                if scale == 1 {
                                    // do nothing, temp = offset*scale
                                    self.backend.emit_mov(&temp, &offset);
                                } else if scale.is_power_of_two() {
                                    // temp = offset << log2(scale)
                                    self.backend.emit_lsl_imm(&temp, &offset, log2(scale as u64) as u8);
                                } else {
                                    // temp = offset * scale
                                    let temp_mul = self.make_temporary(f_context, offset.ty.clone(), vm);
                                    self.aarch64_emit_mov_u64(&temp_mul, scale as u64);
                                    self.backend.emit_mul(&temp, &offset, &temp_mul);
                                }

                                let temp_more = self.make_temporary(f_context, offset.ty.clone(), vm);

                                if new_scale.is_power_of_two() {
                                    // temp_more = more_offset << log2(new_scale)
                                    self.backend.emit_lsl_imm(&temp_more, &more_offset, log2(scale as u64) as u8);
                                } else {
                                    // temp_more = more_offset * new_scale
                                    let temp_mul = self.make_temporary(f_context, more_offset.ty.clone(), vm);
                                    self.aarch64_emit_mov_u64(&temp_mul, new_scale as u64);
                                    self.backend.emit_mul(&temp_more, &more_offset, &temp_mul);
                                }

                                // TODO: If scale is a valid ext_shift then use it here (and don't pre multiple offset)
                                self.backend.emit_add_ext(&temp, &temp_more, &temp, signed, 0);
                                temp
                            }
                        } else {
                            more_offset.clone()
                        };
                    MemoryLocation::VirtualAddress {
                        base: base.clone(),
                        offset: Some(offset),
                        scale: new_scale,
                        signed: signed,
                    }
                },
                _ => panic!("expected a VirtualAddress memory location")
            }
        }
    }


    // UNUSED
    fn aarch64_memory_location_append_offset(&mut self, mem: MemoryLocation, new_offset: &P<Value>, new_signed: bool) -> MemoryLocation {
        match mem {
            MemoryLocation::VirtualAddress { base, offset, scale, signed } => {
                self.aarch64_make_memory_location_base_offset_scale(&base, &new_offset, scale, new_signed)
            },
            _ => panic!("expected an address memory location")
        }
    }

    // UNUSED
    fn aarch64_memory_location_append_offset_scale(&mut self, mem: MemoryLocation, new_offset: &P<Value>, new_scale: u64, new_signed: bool) -> MemoryLocation {
        match mem {
            MemoryLocation::VirtualAddress { ref base, ref offset, scale, signed } => {
                self.aarch64_make_memory_location_base_offset_scale(&base, &new_offset, new_scale, new_signed)
            },
            _ => panic!("expected an address memory location")
        }
    }

    // UNUSED
    fn aarch64_memory_location_append_scale(&mut self, mem: MemoryLocation, new_scale: u64) -> MemoryLocation {
        match mem {
            MemoryLocation::VirtualAddress { ref base, ref offset, scale, signed } => {
                match offset.as_ref() {
                    Some(ref offset) => self.aarch64_make_memory_location_base_offset_scale(&base, &offset, new_scale, signed),
                    _ => panic!("A scale requires an offset")
                }

            },
            _ => panic!("expected an address memory location")
        }
    }

    // Returns the size of the operation
    // TODO: If the RHS of an ADD is negative change it to a SUB (and vice versa)
    // TODO: Treat SUB 0, Op2  and EOR 0, Op2 specially
    // Note: Assume that trivial operations will be optimised away by the Mu IR compiler
    // TODO: Use a shift when dividing or multiplying by a power of two
    fn aarch64_emit_binop(&mut self, node: &TreeNode, inst: &Instruction, op: BinOp, status: BinOpStatus, op1: OpIndex, op2: OpIndex, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) {
        use std;
        let mut op1 = op1;
        let mut op2 = op2;
        let ops = inst.ops.read().unwrap();
        let res = self.get_result_value(node);

        // Get the size (in bits) of the type the operation is on
        let n = get_bit_size(&res.ty, vm);
        let output_status = status.flag_n || status.flag_z || status.flag_c || status.flag_v;
        let values = inst.value.as_ref().unwrap();
        let mut status_value_index = 0;
        // NOTE: XZR is just a dummy value here (it will not be used)
        let tmp_status_n = if status.flag_n {
            status_value_index += 1;
            values[status_value_index].clone()
        } else { XZR.clone() };
        let tmp_status_z = if status.flag_z {
            status_value_index += 1;
            values[status_value_index].clone()
        } else { XZR.clone() };
        let tmp_status_c = if status.flag_c {
            status_value_index += 1;
            values[status_value_index].clone()
        } else { XZR.clone() };
        let tmp_status_v = if status.flag_v {
            status_value_index += 1;
            values[status_value_index].clone()
        } else { XZR.clone() };

        // TODO: Division by zero exception (note: must explicitly check for this, arm dosn't do it)
        // TODO: (Unneccesary??) Check that flags aren't output for instructions that don't support them
        match op {
            // The lower n bits of the result will be correct, and will not depend
            // on the > n bits of op1 or op2
            op::BinOp::Add => {
                let mut imm_val = 0 as u64;
                // Is one of the arguments a valid immediate?
                let emit_imm = if self.aarch64_match_node_iimm(&ops[op2]) {
                    imm_val = self.aarch64_node_iimm_to_u64(&ops[op2]);
                    is_valid_arithmetic_imm(imm_val)
                } else if self.aarch64_match_node_iimm(&ops[op1]) {
                    imm_val = self.aarch64_node_iimm_to_u64(&ops[op1]);
                    // if op1 is a valid immediate, swap it with op2
                    if is_valid_arithmetic_imm(imm_val) {
                        std::mem::swap(&mut op1, &mut op2);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };

                if emit_imm {
                    trace!("emit add-ireg-imm");

                    let reg_op1 = self.aarch64_emit_ireg(&ops[op1], f_content, f_context, vm);
                    let imm_shift = imm_val > 4096;
                    let imm_op2 = if imm_shift { imm_val >> 12 } else { imm_val };

                    if output_status {
                        self.aarch64_emit_zext(&reg_op1);
                        self.backend.emit_adds_imm(&res, &reg_op1, imm_op2 as u16, imm_shift);

                        if status.flag_v {
                            if n < 32 {
                                // tmp_status[n-1] = 1 iff res and op1 have different signs
                                self.backend.emit_eor(&tmp_status_v, &res, &reg_op1);
                                // tmp[n-1] = 1 iff op1 and op2 have different signs

                                // Sign bit of op2 is 0
                                if !get_bit(imm_val, n - 1) {
                                    // tmp_status[n-1] = 1 iff res and op1 have different signs
                                    //      and op1 has the same sign as op2 (which is 0)
                                    self.backend.emit_bic(&tmp_status_v, &tmp_status_v, &reg_op1);
                                } else {
                                    // tmp_status[n-1] = 1 iff res and op1 have different signs
                                    //      and op1 has the same sign as op2 (which is 1)
                                    self.backend.emit_and(&tmp_status_v, &tmp_status_v, &reg_op1);
                                }

                                // Check the sign bit of tmp_status (i.e. tmp_status[n-1])
                                self.backend.emit_tst_imm(&tmp_status_v, 1 << (n - 1));
                                self.backend.emit_cset(&tmp_status_v, "NE");
                            } else {
                                self.backend.emit_cset(&tmp_status_v, "VS");
                            }
                        }
                        if status.flag_c {
                            if n < 32 {
                                // Test the carry bit of res
                                self.backend.emit_tst_imm(&res, 1 << n);
                                self.backend.emit_cset(&tmp_status_c, "NE");
                            } else {
                                self.backend.emit_cset(&tmp_status_c, "CS");
                            }
                        }
                    } else {
                        self.backend.emit_add_imm(&res, &reg_op1, imm_op2 as u16, imm_shift);
                    }
                } else {
                    trace!("emit add-ireg-ireg");

                    let reg_op1 = self.aarch64_emit_ireg(&ops[op1], f_content, f_context, vm);
                    let reg_op2 = self.aarch64_emit_ireg(&ops[op2], f_content, f_context, vm);

                    if output_status {
                        self.aarch64_emit_zext(&reg_op1);
                        if n == 1 {
                            // adds_ext dosn't support extending 1 bit numbers
                            self.aarch64_emit_zext(&reg_op2);
                            self.backend.emit_adds(&res, &reg_op1, &reg_op2);
                        } else {
                            // Emit an adds that zero extends op2
                            self.backend.emit_adds_ext(&res, &reg_op1, &reg_op2, false, 0);
                        }

                        if status.flag_v {
                            if n < 32 {
                                let tmp = self.make_temporary(f_context, UINT32_TYPE.clone(), vm);

                                // tmp_status[n-1] = 1 iff res and op1 have different signs
                                self.backend.emit_eor(&tmp_status_v, &res, &reg_op1);
                                // tmp[n-1] = 1 iff op1 and op2 have different signs
                                self.backend.emit_eor(&tmp, &reg_op1, &reg_op2);

                                // tmp_status[n-1] = 1 iff res and op1 have different signs
                                //      and op1 and op2 have the same sign
                                self.backend.emit_bic(&tmp_status_v, &tmp_status_v, &tmp);

                                // Check tmp_status[n-1]
                                self.backend.emit_tst_imm(&tmp_status_v, 1 << (n - 1));
                                self.backend.emit_cset(&tmp_status_v, "NE");
                            } else {
                                self.backend.emit_cset(&tmp_status_v, "VS");
                            }
                        }

                        if status.flag_c {
                            if n < 32 {
                                // Test the carry bit of res
                                self.backend.emit_tst_imm(&res, 1 << n);
                                self.backend.emit_cset(&tmp_status_c, "NE");
                            } else {
                                self.backend.emit_cset(&tmp_status_c, "CS");
                            }
                        }
                    } else {
                        self.backend.emit_add(&res, &reg_op1, &reg_op2);
                    }
                }
            },
            op::BinOp::Sub => {
                // TODO: Case when the immediate needs to be 1' or sign-extended...
                if self.aarch64_match_node_iimm(&ops[op2]) &&
                    is_valid_arithmetic_imm(self.aarch64_node_iimm_to_u64(&ops[op2])) &&
                    !(status.flag_c && n < 32) {
                    // Can't compute the carry but using a subs_imm instruction
                    trace!("emit sub-ireg-imm");

                    let reg_op1 = self.aarch64_emit_ireg(&ops[op1], f_content, f_context, vm);
                    let imm_val = self.aarch64_node_iimm_to_u64(&ops[op2]);
                    let imm_shift = imm_val > 4096;
                    let imm_op2 = if imm_shift { imm_val >> 12 } else { imm_val };

                    if output_status {
                        self.aarch64_emit_zext(&reg_op1);
                        self.backend.emit_subs_imm(&res, &reg_op1, imm_op2 as u16, imm_shift);

                        if status.flag_v {
                            if n < 32 {
                                // tmp_status[n-1] = 1 iff res and op1 have different signs
                                self.backend.emit_eor(&tmp_status_v, &res, &reg_op1);
                                // tmp[n-1] = 1 iff op1 and op2 have different signs

                                // Sign bit of op2 is 0
                                if imm_val & (1 << (n - 1)) == 0 {
                                    // tmp_status[n-1] = 1 iff res and op1 have different signs
                                    //      and op1 has the same sign as -op2 (which is 1)
                                    self.backend.emit_and(&tmp_status_v, &tmp_status_v, &reg_op1);
                                } else {
                                    // tmp_status[n-1] = 1 iff res and op1 have different signs
                                    //      and op1 has the same sign as op2 (which is 0)
                                    self.backend.emit_bic(&tmp_status_v, &tmp_status_v, &reg_op1);
                                }

                                // Check the sign bit of tmp_status (i.e. tmp_status[n-1])
                                self.backend.emit_tst_imm(&tmp_status_v, 1 << (n - 1));
                                self.backend.emit_cset(&tmp_status_v, "NE");
                            } else {
                                self.backend.emit_cset(&tmp_status_v, "VS");
                            }
                        }

                        if status.flag_c {
                            if n < 32 {
                                // Test the carry bit of res
                                self.backend.emit_tst_imm(&res, 1 << n);
                                self.backend.emit_cset(&tmp_status_c, "NE");
                            } else {
                                self.backend.emit_cset(&tmp_status_c, "CS");
                            }
                        }
                    } else {
                        self.backend.emit_sub_imm(&res, &reg_op1, imm_op2 as u16, imm_shift);
                    }
                } else {
                    trace!("emit sub-ireg-ireg");

                    let reg_op1 = self.aarch64_emit_ireg(&ops[op1], f_content, f_context, vm);
                    let reg_op2 = self.aarch64_emit_ireg(&ops[op2], f_content, f_context, vm);

                    if output_status {
                        self.aarch64_emit_zext(&reg_op1);

                        if status.flag_c {
                            // Note: reg_op2 is 'one'-extended so that SUB res, zext(reg_op1), oext(reg_op2)
                            // Is equivelent to: ADD res, zext(reg_op1), zext(~reg_op2), +1
                            // (this allows the carry flag to be computed as the 'n'th bit of res

                            self.aarch64_emit_oext(&reg_op2);
                            self.backend.emit_subs(&res, &reg_op1, &reg_op2);
                        } else if n == 1 {
                            // if the carry flag isn't been computed, just zero extend op2
                            self.aarch64_emit_zext(&reg_op2);
                            self.backend.emit_subs(&res, &reg_op1, &reg_op2);
                        } else {
                            // Emit an subs that zero extends op2
                            self.backend.emit_subs_ext(&res, &reg_op1, &reg_op2, false, 0);
                        }


                        if status.flag_v {
                            if n < 32 {
                                let tmp = self.make_temporary(f_context, UINT32_TYPE.clone(), vm);

                                // tmp_status[n-1] = 1 iff res and op1 have different signs
                                self.backend.emit_eor(&tmp_status_v, &res, &reg_op1);
                                // tmp[n-1] = 1 iff op1 and -op2 have different signs
                                self.backend.emit_eon(&tmp, &reg_op1, &reg_op2);

                                // tmp_status[n-1] = 1 iff res and op1 have different signs
                                //      and op1 and op2 have the same sign
                                self.backend.emit_bic(&tmp_status_v, &tmp_status_v, &tmp);

                                // Check tmp_status[n-1]
                                self.backend.emit_tst_imm(&tmp_status_v, 1 << (n - 1));
                                self.backend.emit_cset(&tmp_status_v, "NE");
                            } else {
                                self.backend.emit_cset(&tmp_status_v, "VS");
                            }
                        }

                        if status.flag_c {
                            if n < 32 {
                                // Test the carry bit of res
                                self.backend.emit_tst_imm(&res, 1 << n);
                                self.backend.emit_cset(&tmp_status_c, "NE");
                            } else {
                                self.backend.emit_cset(&tmp_status_c, "CS");
                            }
                        }
                    } else {
                        self.backend.emit_sub(&res, &reg_op1, &reg_op2);
                    }
                }
            },

            op::BinOp::And => {
                let mut imm_val = 0 as u64;
                // Is one of the arguments a valid immediate?
                let emit_imm = if self.aarch64_match_node_iimm(&ops[op2]) {
                    imm_val = self.aarch64_node_iimm_to_u64(&ops[op2]);
                    is_valid_logical_imm(imm_val, n)
                } else if self.aarch64_match_node_iimm(&ops[op1]) {
                    imm_val = self.aarch64_node_iimm_to_u64(&ops[op1]);
                    // if op1 is a valid immediate, swap it with op2
                    if is_valid_logical_imm(imm_val, n) {
                        std::mem::swap(&mut op1, &mut op2);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };

                if emit_imm {
                    trace!("emit and-ireg-imm");

                    let reg_op1 = self.aarch64_emit_ireg(&ops[op1], f_content, f_context, vm);

                    if output_status {
                        self.backend.emit_ands_imm(&res, &reg_op1, replicate_logical_imm(imm_val, n));
                    } else {
                        self.backend.emit_and_imm(&res, &reg_op1, replicate_logical_imm(imm_val, n));
                    }
                } else {
                    trace!("emit and-ireg-ireg");

                    let reg_op1 = self.aarch64_emit_ireg(&ops[op1], f_content, f_context, vm);
                    let reg_op2 = self.aarch64_emit_ireg(&ops[op2], f_content, f_context, vm);

                    if output_status {
                        self.backend.emit_ands(&res, &reg_op1, &reg_op2);
                    } else {
                        self.backend.emit_and(&res, &reg_op1, &reg_op2);
                    }
                }
            },
            op::BinOp::Or => {
                let mut imm_val = 0 as u64;
                // Is one of the arguments a valid immediate?
                let emit_imm = if self.aarch64_match_node_iimm(&ops[op2]) {
                    imm_val = self.aarch64_node_iimm_to_u64(&ops[op2]);
                    is_valid_logical_imm(imm_val, n)
                } else if self.aarch64_match_node_iimm(&ops[op1]) {
                    imm_val = self.aarch64_node_iimm_to_u64(&ops[op1]);
                    // if op1 is a valid immediate, swap it with op2
                    if is_valid_logical_imm(imm_val, n) {
                        std::mem::swap(&mut op1, &mut op2);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };

                if emit_imm {
                    trace!("emit or-ireg-imm");

                    let reg_op1 = self.aarch64_emit_ireg(&ops[op1], f_content, f_context, vm);

                    self.backend.emit_orr_imm(&res, &reg_op1, replicate_logical_imm(imm_val, n));
                } else {
                    trace!("emit or-ireg-ireg");

                    let reg_op1 = self.aarch64_emit_ireg(&ops[op1], f_content, f_context, vm);
                    let reg_op2 = self.aarch64_emit_ireg(&ops[op2], f_content, f_context, vm);

                    self.backend.emit_orr(&res, &reg_op1, &reg_op2);
                }
            },
            op::BinOp::Xor => {
                let mut imm_val = 0 as u64;
                // Is one of the arguments a valid immediate?
                let emit_imm = if self.aarch64_match_node_iimm(&ops[op2]) {
                    imm_val = self.aarch64_node_iimm_to_u64(&ops[op2]);
                    is_valid_logical_imm(imm_val, n)
                } else if self.aarch64_match_node_iimm(&ops[op1]) {
                    imm_val = self.aarch64_node_iimm_to_u64(&ops[op1]);
                    // if op1 is a valid immediate, swap it with op2
                    if is_valid_logical_imm(imm_val, n) {
                        std::mem::swap(&mut op1, &mut op2);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };

                if emit_imm {
                    trace!("emit xor-ireg-imm");

                    let reg_op1 = self.aarch64_emit_ireg(&ops[op1], f_content, f_context, vm);

                    self.backend.emit_eor_imm(&res, &reg_op1, replicate_logical_imm(imm_val, n));
                } else {
                    trace!("emit xor-ireg-ireg");

                    let reg_op1 = self.aarch64_emit_ireg(&ops[op1], f_content, f_context, vm);
                    let reg_op2 = self.aarch64_emit_ireg(&ops[op2], f_content, f_context, vm);

                    self.backend.emit_eor(&res, &reg_op1, &reg_op2);
                }
            },

            op::BinOp::Mul => {
                trace!("emit mul-ireg-ireg");

                let reg_op1 = self.aarch64_emit_ireg(&ops[op1], f_content, f_context, vm);
                let reg_op2 = self.aarch64_emit_ireg(&ops[op2], f_content, f_context, vm);

                self.aarch64_emit_zext(&reg_op1);
                self.aarch64_emit_zext(&reg_op2);

                if status.flag_c || status.flag_v {
                    if n < 32 {
                        // A normal multiply will give the correct upper 'n' bits
                        self.backend.emit_mul(&res, &reg_op1, &reg_op2);
                        // Test the upper 'n' bits of the result
                        self.backend.emit_tst_imm(&res, (bits_ones(n) << n));
                    } else if n == 32 {
                        // the 64-bit register version of res
                        let res_64 = unsafe { &res.as_type(UINT64_TYPE.clone()) };
                        // Compute the full 64-bit product of reg_op1 and reg_op2
                        self.backend.emit_umull(&res_64, &reg_op1, &reg_op2);
                        // Test the upper n bits of the result
                        self.backend.emit_tst_imm(&res, 0xFFFFFFFF00000000);
                    } else if n == 64 {
                        // Compute the upper 64-bits of the true product
                        self.backend.emit_umulh(&res, &reg_op1, &reg_op2);
                        // Test the 64-bits of res
                        self.backend.emit_tst_imm(&res, 0xFFFFFFFFFFFFFFFF);
                        // Get the lower 64-bits of the true product
                        self.backend.emit_mul(&res, &reg_op1, &reg_op2);
                    } else {
                        panic!("Unexpeceded integer length {}", n);
                    }

                    // Flags C and V are the same
                    if status.flag_c {
                        self.backend.emit_cset(&tmp_status_c, "NE");
                    }

                    if status.flag_v {
                        self.backend.emit_cset(&tmp_status_v, "NE");
                    }
                } else {
                    // Just do a normal multiply
                    self.backend.emit_mul(&res, &reg_op1, &reg_op2);
                }

                if status.flag_n || status.flag_z {
                    self.aarch64_emit_sext(&res);
                    self.backend.emit_cmp_imm(&res, 0, false);

                    if status.flag_n {
                        self.backend.emit_cset(&tmp_status_n, "MI");
                    }

                    if status.flag_z {
                        self.backend.emit_cset(&tmp_status_z, "EQ");
                    }
                }
            },
            op::BinOp::Udiv => {
                trace!("emit mul-ireg-ireg");

                let reg_op1 = self.aarch64_emit_ireg(&ops[op1], f_content, f_context, vm);
                let reg_op2 = self.aarch64_emit_ireg(&ops[op2], f_content, f_context, vm);

                // zero extend both arguments (in case they are less than 32 bits)
                self.aarch64_emit_zext(&reg_op1);
                self.aarch64_emit_zext(&reg_op2);
                self.backend.emit_udiv(&res, &reg_op1, &reg_op2);
            },
            op::BinOp::Sdiv => {
                trace!("emit mul-ireg-ireg");

                let reg_op1 = self.aarch64_emit_ireg(&ops[op1], f_content, f_context, vm);
                let reg_op2 = self.aarch64_emit_ireg(&ops[op2], f_content, f_context, vm);

                // sign extend both arguments (in case they are less than 32 bits)
                self.aarch64_emit_sext(&reg_op1);
                self.aarch64_emit_sext(&reg_op2);
                self.backend.emit_sdiv(&res, &reg_op1, &reg_op2);
            },
            op::BinOp::Urem => {
                trace!("emit mul-ireg-ireg");

                let reg_op1 = self.aarch64_emit_ireg(&ops[op1], f_content, f_context, vm);
                let reg_op2 = self.aarch64_emit_ireg(&ops[op2], f_content, f_context, vm);

                // zero extend both arguments (in case they are less than 32 bits)
                self.aarch64_emit_zext(&reg_op1);
                self.aarch64_emit_zext(&reg_op2);

                self.backend.emit_udiv(&res, &reg_op1, &reg_op2);
                // calculate the remained from the division
                self.backend.emit_msub(&res, &res, &reg_op2, &reg_op1);
            },
            op::BinOp::Srem => {
                trace!("emit mul-ireg-ireg");

                let reg_op1 = self.aarch64_emit_ireg(&ops[op1], f_content, f_context, vm);
                let reg_op2 = self.aarch64_emit_ireg(&ops[op2], f_content, f_context, vm);

                // sign extend both arguments (in case they are less than 32 bits)
                self.aarch64_emit_sext(&reg_op1);
                self.aarch64_emit_sext(&reg_op2);
                self.backend.emit_sdiv(&res, &reg_op1, &reg_op2);
                self.backend.emit_msub(&res, &res, &reg_op2, &reg_op1);
            },

            op::BinOp::Shl => {
                if self.aarch64_match_node_iimm(&ops[op2]) {
                    trace!("emit shl-ireg-imm");

                    let reg_op1 = self.aarch64_emit_ireg(&ops[op1], f_content, f_context, vm);
                    let imm_op2 = self.aarch64_node_iimm_to_u64(&ops[op2]) %
                        (res.ty.get_int_length().unwrap() as u64);

                    self.backend.emit_lsl_imm(&res, &reg_op1, imm_op2 as u8);
                } else {
                    trace!("emit shl-ireg-ireg");

                    let reg_op1 = self.aarch64_emit_ireg(&ops[op1], f_content, f_context, vm);
                    let reg_op2 = self.aarch64_emit_ireg(&ops[op2], f_content, f_context, vm);

                    // Will be reg_op1, or reg_op2
                    let reg_op2_use = self.aarch64_emit_shift_mask(&reg_op1, &reg_op2);
                    self.backend.emit_lsl(&res, &reg_op1, &reg_op2_use);
                }
            },
            op::BinOp::Lshr => {
                if self.aarch64_match_node_iimm(&ops[op2]) {
                    trace!("emit lshr-ireg-imm");

                    let reg_op1 = self.aarch64_emit_ireg(&ops[op1], f_content, f_context, vm);
                    let imm_op2 = self.aarch64_node_iimm_to_u64(&ops[op2]) %
                        (res.ty.get_int_length().unwrap() as u64);

                    self.backend.emit_lsr_imm(&res, &reg_op1, imm_op2 as u8);
                } else {
                    trace!("emit lshr-ireg-ireg");

                    let reg_op1 = self.aarch64_emit_ireg(&ops[op1], f_content, f_context, vm);
                    let reg_op2 = self.aarch64_emit_ireg(&ops[op2], f_content, f_context, vm);

                    // Will be reg_op1, or reg_op2
                    let reg_op2_use = self.aarch64_emit_shift_mask(&reg_op1, &reg_op2);
                    self.backend.emit_lsr(&res, &reg_op1, &reg_op2_use);
                }
            },
            op::BinOp::Ashr => {
                if self.aarch64_match_node_iimm(&ops[op2]) {
                    trace!("emit ashr-ireg-imm");

                    let reg_op1 = self.aarch64_emit_ireg(&ops[op1], f_content, f_context, vm);
                    let imm_op2 = self.aarch64_node_iimm_to_u64(&ops[op2]) %
                        (res.ty.get_int_length().unwrap() as u64);

                    self.backend.emit_asr_imm(&res, &reg_op1, imm_op2 as u8);
                } else {
                    trace!("emit ashr-ireg-ireg");

                    let reg_op1 = self.aarch64_emit_ireg(&ops[op1], f_content, f_context, vm);
                    let reg_op2 = self.aarch64_emit_ireg(&ops[op2], f_content, f_context, vm);

                    // Will be reg_op1, or reg_op2
                    let reg_op2_use = self.aarch64_emit_shift_mask(&reg_op1, &reg_op2);
                    self.backend.emit_asr(&res, &reg_op1, &reg_op2_use);
                }
            },

            // floating point
            op::BinOp::FAdd => {
                trace!("emit add-fpreg-fpreg");

                let reg_op1 = self.aarch64_emit_fpreg(&ops[op1], f_content, f_context, vm);
                let reg_op2 = self.aarch64_emit_fpreg(&ops[op2], f_content, f_context, vm);

                self.backend.emit_fadd(&res, &reg_op1, &reg_op2);
            }
            op::BinOp::FSub => {
                trace!("emit sub-fpreg-fpreg");

                let reg_op1 = self.aarch64_emit_fpreg(&ops[op1], f_content, f_context, vm);
                let reg_op2 = self.aarch64_emit_fpreg(&ops[op2], f_content, f_context, vm);

                self.backend.emit_fsub(&res, &reg_op1, &reg_op2);
            }
            op::BinOp::FMul => {
                trace!("emit mul-fpreg-fpreg");

                let reg_op1 = self.aarch64_emit_fpreg(&ops[op1], f_content, f_context, vm);
                let reg_op2 = self.aarch64_emit_fpreg(&ops[op2], f_content, f_context, vm);

                self.backend.emit_fmul(&res, &reg_op1, &reg_op2);
            }
            op::BinOp::FDiv => {
                trace!("emit div-fpreg-fpreg");

                let reg_op1 = self.aarch64_emit_fpreg(&ops[op1], f_content, f_context, vm);
                let reg_op2 = self.aarch64_emit_fpreg(&ops[op2], f_content, f_context, vm);

                self.backend.emit_fdiv(&res, &reg_op1, &reg_op2);
            }
            op::BinOp::FRem => {
                trace!("emit rem-fpreg-fpreg");

                let reg_op1 = self.aarch64_emit_fpreg(&ops[op1], f_content, f_context, vm);
                let reg_op2 = self.aarch64_emit_fpreg(&ops[op2], f_content, f_context, vm);

                // TODO: What about 32-bit FREMS??
                self.aarch64_emit_runtime_entry(&entrypoints::FREM, vec![reg_op1.clone(), reg_op2.clone()], Some(vec![res.clone()]), Some(node), f_content, f_context, vm);
            }
        }

        if output_status {
            match op {
                op::BinOp::Add | op::BinOp::Sub => {
                    if status.flag_n {
                        if n < 32 {
                            // Test the sign bit of res
                            self.backend.emit_tst_imm(&res, (1 << (n - 1)));
                            self.backend.emit_cset(&tmp_status_n, "NE");
                        } else {
                            self.backend.emit_cset(&tmp_status_n, "MI");
                        }
                    }

                    if status.flag_z {
                        // Need to calculate the sign bit through masking
                        if n < 32 {
                            // Test the lower 'n' bits of res
                            self.backend.emit_tst_imm(&res, bits_ones(n));
                            self.backend.emit_cset(&tmp_status_z, "EQ");
                        } else {
                            self.backend.emit_cset(&tmp_status_z, "EQ");
                        }
                    }
                    // the C and V flags are computed above
                }
                // And has flags
                op::BinOp::And => {
                    // sign extend result (so that we can compare it properly)
                    self.aarch64_emit_sext(&res);
                    if n < 32 {
                        // compare with zero to compute the status flags
                        self.backend.emit_cmp_imm(&res, 0, false)
                    }

                    if status.flag_n {
                        self.backend.emit_cset(&tmp_status_n, "MI");
                    }

                    if status.flag_z {
                        self.backend.emit_cset(&tmp_status_z, "EQ");
                    }
                }

                // The result of these instructions will have the correct sign bit (even for n < 32)
                // And since overflow is not possible res will be 0 iff the lower 'n' bits of res is 0
                // (thus a comparison to 0 will produce the correct N and Z flags without needing to sign extend the result)
                op::BinOp::Sdiv | op::BinOp::Srem => {
                    self.backend.emit_cmp_imm(&res, 0, false);

                    if status.flag_n {
                        self.backend.emit_cset(&tmp_status_n, "MI");
                    }

                    if status.flag_z {
                        self.backend.emit_cset(&tmp_status_z, "EQ");
                    }
                }

                // All other operations that have flags just have the N and Z flags, but there are no instructions that set them automatically
                _ => {
                    self.aarch64_emit_sext(&res);
                    self.backend.emit_cmp_imm(&res, 0, false);

                    if status.flag_n {
                        self.backend.emit_cset(&tmp_status_n, "MI");
                    }

                    if status.flag_z {
                        self.backend.emit_cset(&tmp_status_z, "EQ");
                    }
                }
            }
        }
    }

    fn aarch64_emit_alloc_sequence(&mut self, tmp_allocator: P<Value>, size: P<Value>, align: usize, node: &TreeNode, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) -> P<Value> {
        if size.is_int_const() {
            // size known at compile time, we can choose to emit alloc_small or large now
            let size_i = size.extract_int_const();

            if size_i + OBJECT_HEADER_SIZE as u64 > mm::LARGE_OBJECT_THRESHOLD as u64 {
                self.aarch64_emit_alloc_sequence_large(tmp_allocator, size, align, node, f_content, f_context, vm)
            } else {
                self.aarch64_emit_alloc_sequence_small(tmp_allocator, size, align, node, f_content, f_context, vm)
            }
        } else {
            // size is unknown at compile time
            // we need to emit both alloc small and alloc large,
            // and it is decided at runtime

            // emit: CMP size, THRESHOLD
            // emit: B.GT ALLOC_LARGE
            // emit: >> small object alloc
            // emit: B ALLOC_LARGE_END
            // emit: ALLOC_LARGE:
            // emit: >> large object alloc
            // emit: ALLOC_LARGE_END:
            let blk_alloc_large = format!("{}_alloc_large", node.id());
            let blk_alloc_large_end = format!("{}_alloc_large_end", node.id());

            if OBJECT_HEADER_SIZE != 0 {
                let size_with_hdr = self.make_temporary(f_context, UINT64_TYPE.clone(), vm);
                self.aarch64_emit_add_u64(&size_with_hdr, &size, f_context, vm, OBJECT_HEADER_SIZE as u64);
                self.aarch64_emit_cmp_u64(&size_with_hdr, f_context, vm, mm::LARGE_OBJECT_THRESHOLD as u64);
            } else {
                self.aarch64_emit_cmp_u64(&size, f_context, vm, mm::LARGE_OBJECT_THRESHOLD as u64);
            }
            self.backend.emit_b_cond("GT", blk_alloc_large.clone());

            self.finish_block();
            self.start_block(format!("{}_allocsmall", node.id()));

            // alloc small here
            let tmp_res = self.aarch64_emit_alloc_sequence_small(tmp_allocator.clone(), size.clone(), align, node, f_content, f_context, vm);

            self.backend.emit_b(blk_alloc_large_end.clone());

            // finishing current block
            let cur_block = self.current_block.as_ref().unwrap().clone();
            self.backend.end_block(cur_block.clone());
            self.backend.set_block_liveout(cur_block.clone(), &vec![tmp_res.clone()]);

            // alloc_large:
            self.current_block = Some(blk_alloc_large.clone());
            self.backend.start_block(blk_alloc_large.clone());
            self.backend.set_block_livein(blk_alloc_large.clone(), &vec![size.clone()]);

            let tmp_res = self.aarch64_emit_alloc_sequence_large(tmp_allocator.clone(), size, align, node, f_content, f_context, vm);

            self.backend.end_block(blk_alloc_large.clone());
            self.backend.set_block_liveout(blk_alloc_large.clone(), &vec![tmp_res.clone()]);

            // alloc_large_end:
            self.backend.start_block(blk_alloc_large_end.clone());
            self.current_block = Some(blk_alloc_large_end.clone());

            tmp_res
        }
    }

    fn aarch64_emit_get_allocator(&mut self, node: &TreeNode, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) -> P<Value> {
        // ASM: %tl = get_thread_local()
        let tmp_tl = self.aarch64_emit_get_threadlocal(Some(node), f_content, f_context, vm);

        // ASM: lea [%tl + allocator_offset] -> %tmp_allocator
        let allocator_offset = *thread::ALLOCATOR_OFFSET;
        let tmp_allocator = self.make_temporary(f_context, ADDRESS_TYPE.clone(), vm);
        self.aarch64_emit_add_u64(&tmp_allocator, &tmp_tl, f_context, vm, allocator_offset as u64);
        tmp_allocator
    }

    fn aarch64_emit_alloc_sequence_large(&mut self, tmp_allocator: P<Value>, size: P<Value>, align: usize, node: &TreeNode, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) -> P<Value> {
        let tmp_res = self.get_result_value(node);

        // ASM: %tmp_res = call muentry_alloc_large(%allocator, size, align)
        let const_align = self.aarch64_make_value_int_const(align as u64, vm);

        self.aarch64_emit_runtime_entry(
            &entrypoints::ALLOC_LARGE,
            vec![tmp_allocator.clone(), size.clone(), const_align],
            Some(vec![tmp_res.clone()]),
            Some(node), f_content, f_context, vm
        );

        tmp_res
    }

    fn aarch64_emit_alloc_sequence_small(&mut self, tmp_allocator: P<Value>, size: P<Value>, align: usize, node: &TreeNode, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) -> P<Value> {
        if INLINE_FASTPATH {
            unimplemented!(); // (inline the generated code in alloc() in immix_mutator.rs??)
        } else {
            // directly call 'alloc'
            let tmp_res = self.get_result_value(node);

            let const_align = self.aarch64_make_value_int_const(align as u64, vm);

            self.aarch64_emit_runtime_entry(
                &entrypoints::ALLOC_FAST,
                vec![tmp_allocator.clone(), size.clone(), const_align],
                Some(vec![tmp_res.clone()]),
                Some(node), f_content, f_context, vm
            );

            tmp_res
        }
    }

    fn aarch64_emit_load_base_offset(&mut self, dest: &P<Value>, base: &P<Value>, offset: i64, f_context: &mut FunctionContext, vm: &VM) -> P<Value> {
        let mem = self.aarch64_make_value_base_offset(base, offset, &dest.ty, vm);
        let mem = self.aarch64_emit_mem(&mem, f_context, vm);
        self.backend.emit_ldr(dest, &mem, false);
        mem
    }

    fn aarch64_emit_store_base_offset(&mut self, base: &P<Value>, offset: i64, src: &P<Value>, f_context: &mut FunctionContext, vm: &VM) {
        let mem = self.aarch64_make_value_base_offset(base, offset, &src.ty, vm);
        let mem = self.aarch64_emit_mem(&mem, f_context, vm);
        self.backend.emit_str(&mem, src);
    }

    fn aarch64_emit_get_threadlocal(
        &mut self,
        cur_node: Option<&TreeNode>,
        f_content: &FunctionContent,
        f_context: &mut FunctionContext,
        vm: &VM) -> P<Value> {
        let mut rets = self.aarch64_emit_runtime_entry(&entrypoints::GET_THREAD_LOCAL, vec![], None, cur_node, f_content, f_context, vm);

        rets.pop().unwrap()
    }

    // ret: Option<Vec<P<Value>>
    // if ret is Some, return values will put stored in given temporaries
    // otherwise create temporaries
    // always returns result temporaries (given or created)
    fn aarch64_emit_runtime_entry(
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

        self.aarch64_emit_c_call_internal(entry_name, sig, args, rets, cur_node, f_content, f_context, vm)
    }


    // Note: if tys has more than 1 element, then this will return a new struct type
    // , but each call will generate a different name for this struct type (but the layout will be identical)
    fn aarch64_combine_return_types(&self, tys: &Vec<P<MuType>>) -> P<MuType>{
        let n = tys.len();
        if n == 0 {
            VOID_TYPE.clone()
        } else if n == 1 {
            tys[0].clone()
        } else {
            P(MuType::new(new_internal_id(), MuType_::mustruct(format!("return_type#{}", new_internal_id()), tys.to_vec())))
        }
    }

    // How much space needs to be allocated on the stack to hold the return value (returns 0 if no space needs to be allocated)
    fn aarch64_compute_return_allocation(&self, t: &P<MuType>, vm: &VM) -> usize
    {
        use ast::types::MuType_::*;
        let size = round_up(vm.get_type_size(t.id()), 8);
        match t.v {
            Vector(_, _) | Tagref64 => unimplemented!(),
            Float | Double => 0, // Can return in FPR
            Hybrid(_) => panic!("cant return a hybrid"), // don't know how much space to reserve for it
            Struct(_) | Array(_, _) => {
                if hfa_length(t.clone()) > 0 || size <= 16 {
                    0 // Can return in register (or multiple registers)
                } else {
                    round_up(size, 16)
                }
            }

            Void => 0, // Don't need to return anything
            // Integral or pointer type
            Int(_) | Ref(_) | IRef(_) | WeakRef(_) | UPtr(_) |
            ThreadRef | StackRef | FuncRef(_) | UFuncPtr(_) => 0, // can return in GPR
        }
    }
    // Returns a value for each return value
    fn aarch64_compute_return_locations(&mut self, t: &P<MuType>, loc: &P<Value>, vm: &VM) -> P<Value>
    {
        use ast::types::MuType_::*;
        let size = round_up(vm.get_type_size(t.id()), 8);
        match t.v {
            Vector(_, _) | Tagref64 => unimplemented!(),
            Float | Double => get_alias_for_length(RETURN_FPRs[0].id(), get_bit_size(t, vm)),
            Hybrid(_) => panic!("cant return a hybrid"),
            Struct(_) | Array(_, _) => {
                let hfa_n = hfa_length(t.clone());
                if hfa_n > 0 {
                    // Return in a sequence of FPRs
                    get_alias_for_length(RETURN_FPRs[0].id(), get_bit_size(t, vm)/hfa_n)
                }
                else if size <= 16 {
                    // Return in a sequence of GRPs
                    RETURN_GPRs[0].clone()
                } else {
                    // Return at the location pointed to by XR
                    self.aarch64_make_value_base_offset(&loc, 0, &t, vm)
                }
            }

            Void => panic!("Nothing to return"),
            // Integral or pointer type
            Int(_) | Ref(_) | IRef(_) | WeakRef(_) | UPtr(_) | ThreadRef | StackRef | FuncRef(_) | UFuncPtr(_) =>
                // can return in GPR
                get_alias_for_length(RETURN_GPRs[0].id(), get_bit_size(t, vm))
        }
    }
    // TODO: Thoroughly test this (compare with code generated by GCC with variouse different types???)
    // The algorithm presented here is derived from the ARM AAPCS64 reference
    // Returns a vector indicating whether each should be passed as an IRef (and not directly),
    // a vector referencing to the location of each argument (in memory or a register) and the amount of stack space used
    // NOTE: It currently does not support vectors/SIMD types (or aggregates of such types)
    fn aarch64_compute_argument_locations(&mut self, arg_types: &Vec<P<MuType>>, stack: &P<Value>, offset: i64, vm: &VM) -> (Vec<bool>, Vec<P<Value>>, usize) {
        if arg_types.len() == 0 {
            // nothing to do
            return (vec![], vec![], 0);
        }

        let mut ngrn = 0 as usize; // The Next General-purpose Register Number
        let mut nsrn = 0 as usize; // The Next SIMD and Floating-point Register Number
        let mut nsaa = 0 as usize; // The next stacked argument address (an offset from the SP)
        use ast::types::MuType_::*;

        // reference[i] = true indicates the argument is passed an IRef to a location on the stack
        let mut reference : Vec<bool> = vec![];
        for t in arg_types {
            reference.push(
                hfa_length(t.clone()) == 0 && // HFA's aren't converted to IRef's
                match t.v {
                    Hybrid(_) => panic!("Hybrid argument not supported"), // size can't be statically determined
                    Struct(_) | Array(_, _) if vm.get_type_size(t.id()) > 16 => true, //  type is too large
                    Vector(_, _) | Tagref64 => unimplemented!(),
                    _ => false
                }
            );
        }
        // TODO: How does passing arguments by reference effect the stack size??
        let mut locations : Vec<P<Value>> = vec![];
        for i in 0..arg_types.len() {
            let i = i as usize;
            let t = if reference[i] { P(MuType::new(new_internal_id(), MuType_::IRef(arg_types[i].clone()))) } else { arg_types[i].clone() };
            let size = round_up(vm.get_type_size(t.id()), 8);
            let align = vm.get_backend_type_info(t.id()).alignment;
            match t.v {
                Hybrid(_) =>  panic!("hybrid argument not supported"),

                Vector(_, _) | Tagref64 => unimplemented!(),
                Float | Double => {
                    if nsrn < 8 {
                        locations.push(get_alias_for_length(ARGUMENT_FPRs[nsrn].id(), get_bit_size(&t, vm)));
                        nsrn += 1;
                    } else {
                        nsrn = 8;
                        locations.push(self.aarch64_make_value_base_offset(&stack, offset + (nsaa as i64), &t, vm));
                        nsaa += size;
                    }
                }
                Struct(_) | Array(_, _) => {
                    let hfa_n = hfa_length(t.clone());
                    if hfa_n > 0 {
                        if nsrn + hfa_n <= 8 {
                            // Note: the argument will occupy succesiv registers (one for each element)
                            locations.push(get_alias_for_length(ARGUMENT_FPRs[nsrn].id(), get_bit_size(&t, vm)/hfa_n));
                            nsrn += hfa_n;
                        } else {
                            nsrn = 8;
                            locations.push(self.aarch64_make_value_base_offset(&stack, offset + (nsaa as i64), &t, vm));
                            nsaa += size;
                        }
                    } else {
                        if align == 16 && ngrn % 2 != 0 {
                            ngrn += 1; // align ngrn to an even number
                        }

                        if size  <= 8*(8 - ngrn) {
                            // The struct should be packed, starting here
                            // (note: this may result in multiple struct fields in the same regsiter
                            // or even floating points in a GPR)
                            locations.push(ARGUMENT_GPRs[ngrn].clone());
                            // How many GPRS are taken up by t
                            ngrn += if size % 8 != 0 { size/8 + 1 } else { size/8 };
                        } else {
                            ngrn = 8;
                            nsaa = round_up(nsaa, round_up(align, 8));
                            locations.push(self.aarch64_make_value_base_offset(&stack, offset + (nsaa as i64) as i64, &t, vm));
                            nsaa += size;
                        }
                    }
                }

                Void =>  panic!("void argument not supported"),

                // Integral or pointer type
                Int(_) | Ref(_) | IRef(_) | WeakRef(_) |  UPtr(_) |
                ThreadRef | StackRef | FuncRef(_) | UFuncPtr(_) => {
                    if size <= 8 {
                        if ngrn < 8 {
                            locations.push(get_alias_for_length(ARGUMENT_GPRs[ngrn].id(), get_bit_size(&t, vm)));
                            ngrn += 1;
                        } else {
                            nsaa = round_up(nsaa, round_up(align, 8));
                            locations.push(self.aarch64_make_value_base_offset(&stack, offset + (nsaa as i64) as i64, &t, vm));
                            nsaa += size;
                        }

                    } else {
                        unimplemented!(); // Integer type is too large
                    }
                }
            }
        }

        (reference, locations, round_up(nsaa, 16) as usize)
    }


    //#[warn(unused_variables)]
    // returns the stack arg offset - we will need this to collapse stack after the call
    fn aarch64_emit_precall_convention(&mut self, args: &Vec<P<Value>>, arg_tys: &Vec<P<MuType>>, return_size: usize, f_context: &mut FunctionContext, vm: &VM) -> usize
    {
        //sig.ret_tys
        let (is_iref, locations, stack_size) = self.aarch64_compute_argument_locations(&arg_tys, &SP, 0, &vm);

        // Reserve space on the stack for the return value
        self.aarch64_emit_sub_u64(&SP, &SP, f_context, &vm, return_size as u64);

        if return_size > 0 {
            // XR needs to point to where the callee should return arguments
            self.backend.emit_mov(&XR, &SP);
        }
        // Reserve space on the stack for all stack arguments
        self.aarch64_emit_sub_u64(&SP, &SP, f_context, &vm, stack_size as u64);

        for i in 0..args.len() {
            let i = i as usize;
            let ref arg_val = args[i];
            let ref arg_loc = locations[i];
            match arg_val.ty.v {
                MuType_::Hybrid(_) =>  panic!("hybrid argument not supported"),

                MuType_::Vector(_, _) | MuType_::Tagref64 => unimplemented!(),

                MuType_::Struct(_) | MuType_::Array(_, _) => {
                    unimplemented!(); // Todo (note: these may be passed as IRef's)
                }

                MuType_::Void => panic!("void argument not supported"),

                // Everything else is simple
                _ => self.aarch64_emit_move_value_to_value(&arg_loc, &arg_val, f_context, vm)
            }
        }

        stack_size
    }

    fn aarch64_emit_postcall_convention(&mut self, ret_tys: &Vec<P<MuType>>, rets: &Option<Vec<P<Value>>>, ret_type: &P<MuType>, arg_size: usize, ret_size: usize, f_context: &mut FunctionContext, vm: &VM) -> Vec<P<Value>> {
        // deal with ret vals
        let mut return_vals = vec![];

        self.aarch64_emit_add_u64(&SP, &SP, f_context, &vm, arg_size as u64);

        let n = ret_tys.len(); // number of return values
        if n == 0 {
            // Do nothing
        } else if n == 1{
            let ret_loc = self.aarch64_compute_return_locations(&ret_type, &SP, &vm);

            let ref ty = ret_tys[0];
            let ret_val = match rets {
                &Some(ref rets) => rets[0].clone(),
                &None => {
                    let tmp_node = f_context.make_temporary(vm.next_id(), ty.clone());
                    tmp_node.clone_value()
                }
            };

            self.aarch64_emit_move_value_to_value(&ret_val, &ret_loc, f_context, vm);
            return_vals.push(ret_val);
        } else {
            let ret_loc = self.aarch64_compute_return_locations(&ret_type, &SP, &vm);

            for ret_index in 0..ret_tys.len() {
                let ref ty = ret_tys[ret_index];
                let offset = self.aarch64_get_field_offset(ret_type, ret_index, &vm);
                let ret_val = match rets {
                    &Some(ref rets) => rets[ret_index].clone(),
                    &None => {
                        let tmp_node = f_context.make_temporary(vm.next_id(), ty.clone());
                        tmp_node.clone_value()
                    }
                };

                match ty.v {
                    MuType_::Vector(_, _) | MuType_::Tagref64 => unimplemented!(),
                    MuType_::Void => panic!("Unexpected void"),
                    MuType_::Struct(_) | MuType_::Array(_, _) => unimplemented!(),

                    // Integral, pointer of floating point type
                    _ => self.aarch64_extract_bytes(&ret_val, &ret_loc, offset as i64, f_context, vm),
                }
                return_vals.push(ret_val);
            }
        }

        // We have now read the return values, and can free space from the stack
        self.aarch64_emit_add_u64(&SP, &SP, f_context, &vm, ret_size as u64);

        return_vals
    }

    // Copies src to dest+off, dest can be a memory location or a machine register
    // (in the case of a machine register, sucessivie registers of the same size are considered
    // part of dest).
    // WARNING: It is assumed that dest and src do not overlap
    fn aarch64_insert_bytes(&mut self, dest: &P<Value>, src: &P<Value>, offset: i64, f_context: &mut FunctionContext, vm: &VM)
    {
        if dest.is_mem() {
            let dest_loc = match dest.v {
                Value_::Memory(ref mem) => {
                    let mem = self.aarch64_memory_location_shift(mem.clone(), offset, f_context, vm);
                    self.aarch64_make_value_from_memory(mem, &dest.ty, vm)
                },
                _ => panic!("Wrong kind of memory value"),
            };
            // TODO: what if 'src is in more than 1 register
            self.aarch64_emit_move_value_to_value(&dest_loc, &src, f_context, vm);
        } else if is_machine_reg(dest) {
            // Repeat this for each 8'bytes of src

            // Size of each dest unit
            let dest_size = get_bit_size(&dest.ty, vm) as i64;
            let src_size = get_bit_size(&src.ty, vm) as i64;

            // How many registers past the first 1 do we need to copy to
            let reg_distance = offset*8 / dest_size;
            let reg_offset = offset*8 % dest_size;

            // We need to copy to multiple registers
                if src_size > dest_size + reg_offset {
                unimplemented!();
            } else {
                let dest_reg = get_register_from_id(dest.id() + 2*(reg_distance as usize));

                if reg_offset == 0 && dest_size == src_size {
                    // nothing special needs to be done
                    self.aarch64_emit_move_value_to_value(&dest_reg, &src, f_context, vm);
                } else {
                    let tmp_src = if src.is_int_reg() { src.clone() } else { self.make_temporary(f_context, src.ty.clone(), vm) };

                    if !src.is_int_reg() {
                        // A temporary is being used, move src to it
                        self.aarch64_emit_move_value_to_value(&tmp_src, &src, f_context, vm);
                    }

                    if dest_reg.is_int_reg() {
                        // Copy to dest_reg, 'src_size' bits starting at 'reg_offset' in src
                        // (leaving other bits unchanged)
                        self.backend.emit_bfi(&dest_reg, &tmp_src, reg_offset as u8, src_size as u8);
                    } else {
                        // floating point register, need to move dest to an int register first
                        let tmp_dest = self.make_temporary(f_context, tmp_src.ty.clone(), vm);
                        self.backend.emit_fmov(&tmp_dest, &dest_reg);
                        self.backend.emit_bfi(&tmp_dest, &tmp_src, reg_offset as u8, src_size as u8);

                        // Now move it back to the FPR
                        self.backend.emit_fmov(&dest_reg, &tmp_dest);
                    }
                }
            }
        } else {
            panic!("This function should only be used to move from a machine register and Memory");
        }
    }

    // Copies src+off, to dest src can be a memory location or a machine register
    // (in the case of a machine register, sucessivie registers of the same size are considered
    // part of src).
    // WARNING: It is assumed that dest and src do not overlap
    fn aarch64_extract_bytes(&mut self, dest: &P<Value>, src: &P<Value>, offset: i64, f_context: &mut FunctionContext, vm: &VM)
    {
        if src.is_mem() {
            let src_loc = match src.v {

                Value_::Memory(ref mem) => {
                    let mem = self.aarch64_memory_location_shift(mem.clone(), offset, f_context, vm);
                    self.aarch64_make_value_from_memory(mem, &src.ty, vm)
                },
                _ => panic!("Wrong kind of memory value"),
            };
            // TODO: what if 'dest is in more than 1 register
            self.aarch64_emit_move_value_to_value(&dest, &src_loc, f_context, vm);
        } else if is_machine_reg(src) {
            // Repeat this for each 8'bytes of dest

            // Size of each src unit
            let src_size = get_bit_size(&src.ty, vm) as i64;
            let dest_size = get_bit_size(&dest.ty, vm) as i64;

            // How many registers past the first 1 do we need to copy from
            let reg_distance = offset*8 / src_size;
            let reg_offset = offset*8 % src_size;

            // We need to copy from multiple registers
            if dest_size + reg_offset > src_size {
                unimplemented!();
            } else {
                let src_reg = get_register_from_id(src.id() + 2*(reg_distance as usize));

                if reg_offset == 0 {
                    // nothing special needs to be done
                    self.aarch64_emit_move_value_to_value(&dest, &src_reg, f_context, vm);
                } else {
                    let tmp_dest = if dest.is_int_reg() { dest.clone() } else { self.make_temporary(f_context, dest.ty.clone(), vm) };

                    if src_reg.is_int_reg() {
                        // Copy from src_reg, 'dest_size' bits starting at 'reg_offset' and store
                        // in dest (leaving other bits unchanged    )
                        self.backend.emit_bfxil(&tmp_dest, &src_reg, reg_offset as u8, dest_size as u8);
                    } else {
                        // floating point register, need to copy to an int register first
                        self.backend.emit_fmov(&tmp_dest, &src_reg);
                        self.backend.emit_bfxil(&tmp_dest, &tmp_dest, reg_offset as u8, dest_size as u8);
                    }

                    if !dest.is_int_reg() {
                        // A temporary was used, move the value to dest
                        self.aarch64_emit_move_value_to_value(&dest, &tmp_dest, f_context, vm);
                    }
                }
            }
        } else {
            panic!("This function should only be used to move from a machine register and Memory");
        }
    }
    // ret: Option<Vec<P<Value>>
    // if ret is Some, return values will put stored in given temporaries
    // otherwise create temporaries
    // always returns result temporaries (given or created)
    fn  aarch64_emit_c_call_internal(
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
        let return_type = self.aarch64_combine_return_types(&sig.ret_tys);
        let return_size = self.aarch64_compute_return_allocation(&return_type, &vm);
        let stack_arg_size = self.aarch64_emit_precall_convention(&args, &sig.arg_tys, return_size, f_context, vm);

        // make call
        if vm.is_running() {
            unimplemented!()
        } else {
            let callsite = self.new_callsite_label(cur_node);
            self.backend.emit_bl(callsite, func_name, None); // assume ccall wont throw exception

            // record exception block (CCall may have an exception block)
            if cur_node.is_some() {
                let cur_node = cur_node.unwrap();
                if cur_node.op == OpCode::CCall {
                    unimplemented!()
                }
            }
        }

        self.aarch64_emit_postcall_convention(&sig.ret_tys, &rets, &return_type, stack_arg_size, return_size, f_context, vm)
    }

    #[allow(unused_variables)] // resumption not implemented
    fn aarch64_emit_c_call_ir(
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
                let arg = self.aarch64_emit_ireg(arg, f_content, f_context, vm);
                arg_values.push(arg);
            } else if self.aarch64_match_node_iimm(arg) {
                let arg = self.node_iimm_to_value(arg);
                arg_values.push(arg);
            } else {
                unimplemented!();
            }
        }
        let arg_values = arg_values;

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
                            self.aarch64_emit_c_call_internal(
                                func_name.clone(), //func_name: CName,
                                sig, // sig: P<CFuncSig>,
                                arg_values, // args: Vec<P<Value>>,
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

    fn aarch64_emit_mu_call(
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
                let ty: &MuType = &pv.ty;
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
                let arg = self.aarch64_emit_ireg(arg, f_content, f_context, vm);
                arg_values.push(arg);
            } else if self.aarch64_match_node_iimm(arg) {
                let arg = self.node_iimm_to_value(arg);
                arg_values.push(arg);
            } else {
                unimplemented!();
            }
        }
        let return_type = self.aarch64_combine_return_types(&func_sig.ret_tys);
        let return_size = self.aarch64_compute_return_allocation(&return_type, &vm);
        let stack_arg_size = self.aarch64_emit_precall_convention(&arg_values, &func_sig.arg_tys, return_size, f_context, vm);

        // check if this call has exception clause - need to tell backend about this
        let potentially_excepting = {
            if resumption.is_some() {
                let target_id = resumption.unwrap().exn_dest.target;
                Some(f_content.get_block(target_id).name().unwrap())
            } else {
                None
            }
        };

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
                    self.backend.emit_bl(callsite, target.name().unwrap(), potentially_excepting)
                }
            } else {
                let target = self.aarch64_emit_ireg(func, f_content, f_context, vm);

                let callsite = self.new_callsite_label(Some(cur_node));
                self.backend.emit_blr(callsite, &target, potentially_excepting)
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
        self.aarch64_emit_postcall_convention(&func_sig.ret_tys, &inst.value, &return_type, stack_arg_size, return_size, f_context, vm);
    }

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
                    //                            self.aarch64_instruction_select(arg, cur_func);
                    //
                    //                            self.emit_get_result(arg);
                    //                        }
                    //                    }
                    //
                    let ref target_args = f_content.get_block(dest.target).content.as_ref().unwrap().args;
                    let ref target_arg = target_args[i];

                    self.aarch64_emit_move_node_to_value(target_arg, &arg, f_content, f_context, vm);
                },
                &DestArg::Freshbound(_) => unimplemented!()
            }
        }
    }

    fn aarch64_emit_common_prologue(&mut self, args: &Vec<P<Value>>, sig: &P<CFuncSig>, f_context: &mut FunctionContext, vm: &VM) {
        let block_name = PROLOGUE_BLOCK_NAME.to_string();
        self.backend.start_block(block_name.clone());

        // no livein
        // liveout = entry block's args
        self.backend.set_block_livein(block_name.clone(), &vec![]);
        self.backend.set_block_liveout(block_name.clone(), args);

        // Push the frame pointer and link register onto the stack
        self.backend.emit_push_pair(&LR, &FP, &SP);
        if vm.vm_options.flag_emit_debug_info {
            self.backend.add_cfi_def_cfa_offset(16i32);
            self.backend.add_cfi_offset(&FP, -16i32);
        }

        // Set the frame pointer to be the stack pointer
        self.backend.emit_mov(&FP, &SP);
        if vm.vm_options.flag_emit_debug_info {
            self.backend.add_cfi_def_cfa_register(&FP);
        }

        // Note: this needs to be as a seperate block so the self.current_frame can be borrowed mutably multiple times
        {
            // push all callee-saved registers
            let frame = self.current_frame.as_mut().unwrap();

            // For every pair (2*i, 2*i+1) of callee saved FPRs
            for i in 0..CALLEE_SAVED_FPRs.len() / 2 {
                let ref reg1 = CALLEE_SAVED_FPRs[2 * i];
                let ref reg2 = CALLEE_SAVED_FPRs[2 * i + 1];

                trace!("allocate frame slot for regs {}, {}", reg1, reg2);
                self.backend.emit_push_pair(&reg1, &reg2, &SP);
                frame.alloc_slot_for_callee_saved_reg(reg1.clone(), vm);
                frame.alloc_slot_for_callee_saved_reg(reg2.clone(), vm);
            }

            // For every pair (2*i, 2*i+1) of callee saved GPRs
            for i in 0..(CALLEE_SAVED_GPRs.len()) / 2 {
                let ref reg1 = CALLEE_SAVED_GPRs[2 * i];
                let ref reg2 = CALLEE_SAVED_GPRs[2 * i + 1];

                trace!("allocate frame slot for regs {}, {}", reg1, reg2);
                self.backend.emit_push_pair(&reg1, &reg2, &SP);
                frame.alloc_slot_for_callee_saved_reg(reg1.clone(), vm);
                frame.alloc_slot_for_callee_saved_reg(reg2.clone(), vm);
            }
        }

        // reserve spaces for current frame
        self.backend.emit_frame_grow();

        // We need to return arguments in the memory area pointed to by XR, so we need to save it
        let ret_ty = self.aarch64_combine_return_types(&sig.ret_tys);
        if self.aarch64_compute_return_allocation(&ret_ty, &vm) > 0 {
            self.backend.emit_push_pair(&XR, &XZR, &SP);
        }

        // unload arguments
        // Read arguments starting from FP+16 (FP points to the frame record (the previouse FP and LR)
        let (is_iref, locations, stack_size) = self.aarch64_compute_argument_locations(&sig.arg_tys, &FP, 16, &vm);

        for i in 0..args.len() {
            let i = i as usize;
            let ref arg_val = args[i];
            let ref arg_loc = locations[i];
            match arg_val.ty.v {
                MuType_::Hybrid(_) =>  panic!("hybrid argument not supported"),

                MuType_::Vector(_, _) | MuType_::Tagref64 => unimplemented!(),
                MuType_::Float | MuType_::Double => {
                    if arg_loc.is_fp_reg() {
                        // Argument is passed in a floating point register
                        self.backend.emit_fmov(&arg_val, &arg_loc);
                        self.current_frame.as_mut().unwrap().add_argument_by_reg(arg_val.id(), arg_loc.clone());
                    } else {
                        debug_assert!(arg_loc.is_mem());
                        // Argument is on the stack
                        self.current_frame.as_mut().unwrap().add_argument_by_stack(arg_val.id(), arg_loc.clone());
                    }
                }
                MuType_::Struct(_) | MuType_::Array(_, _) => {
                    unimplemented!(); // Todo (note: these may be passed as IRef's)
                }

                MuType_::Void => panic!("void argument not supported"),

                // Integral or pointer type
               _  => {
                    if arg_loc.is_int_reg() {
                        // Argument is passed in an integer point register
                        self.backend.emit_mov(&arg_val, &arg_loc);
                        self.current_frame.as_mut().unwrap().add_argument_by_reg(arg_val.id(), arg_loc.clone());
                    } else {
                        debug_assert!(arg_loc.is_mem());
                        // Argument is on the stack
                        self.aarch64_emit_load(&arg_val, &arg_loc, f_context, vm);
                        self.current_frame.as_mut().unwrap().add_argument_by_stack(arg_val.id(), arg_loc.clone());
                    }
                }
            }
        }

        self.backend.end_block(block_name);
    }

    fn aarch64_emit_common_epilogue(&mut self, ret_inst: &Instruction, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) {
        // epilogue is not a block (its a few instruction inserted before return)
        // FIXME: this may change in the future

        // prepare return regs
        let ref ops = ret_inst.ops.read().unwrap();
        // TODO: Are ret_val_indices in the same order as the return types in the functions signature?
        let ret_val_indices = match ret_inst.v {
            Instruction_::Return(ref vals) => vals,
            _ => panic!("expected ret inst")
        };

        let ret_tys = ret_val_indices.iter().map(|i| self.node_type(&ops[*i])).collect();
        let ret_type = self.aarch64_combine_return_types(&ret_tys);
        // Note: this shouldn't cause any overhead in the generated code if the register is never used
        let temp_xr = self.make_temporary(f_context, ADDRESS_TYPE.clone(), vm);

        if self.aarch64_compute_return_allocation(&ret_type, &vm) > 0 {
            // Load the saved value of XR into temp_xr
            self.backend.emit_pop_pair(&XZR, &temp_xr, &SP);
        }

        let n = ret_tys.len(); // number of return values
        if n == 0 {
            // Do nothing
        } else if n == 1{
            let ret_loc = self.aarch64_compute_return_locations(&ret_type, &temp_xr, &vm);
            self.aarch64_emit_move_node_to_value(&ret_loc, &ops[ret_val_indices[0]], f_content, f_context, vm);
        } else {
            let ret_loc = self.aarch64_compute_return_locations(&ret_type, &temp_xr, &vm);

            let mut i = 0;
            for ret_index in ret_val_indices {
                let ret_val = self.aarch64_emit_node_value(&ops[*ret_index], f_content, f_context, vm);
                let ref ty = ret_val.ty;
                let offset = self.aarch64_get_field_offset(&ret_type, i, &vm);

                match ty.v {
                    MuType_::Vector(_, _) | MuType_::Tagref64 => unimplemented!(),
                    MuType_::Void => panic!("Unexpected void"),
                    MuType_::Struct(_) | MuType_::Array(_, _) => unimplemented!(),

                    // Integral, pointer of floating point type
                    _ => self.aarch64_insert_bytes(&ret_loc, &ret_val, offset as i64, f_context, vm),
                }

                i += 1;
            }
        }

        // frame shrink
        self.backend.emit_frame_shrink();

        // pop all callee-saved registers
        // For every pair (2*i, 2*i+1) of callee saved GPRs
        for i in (0.. (CALLEE_SAVED_GPRs.len()) / 2).rev() {
            let ref reg1 = CALLEE_SAVED_GPRs[2 * i + 1];
            let ref reg2 = CALLEE_SAVED_GPRs[2 * i];

            self.backend.emit_pop_pair(&reg1, &reg2, &SP);
        }
        // For every pair (2*i, 2*i+1) of callee saved FPRs
        for i in (0 .. CALLEE_SAVED_FPRs.len() / 2).rev() {
            let ref reg1 = CALLEE_SAVED_FPRs[2 * i + 1];
            let ref reg2 = CALLEE_SAVED_FPRs[2 * i];

            self.backend.emit_pop_pair(&reg1, &reg2, &SP);
        }


        // Pop the link register and frame pointers
        self.backend.emit_pop_pair(&FP, &LR, &SP);

        // Note: the stack pointer should now be what it was when the function was called
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

    fn aarch64_emit_cmp_res(&mut self, cond: &TreeNode, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) -> op::CmpOp {
        match cond.v {
            TreeNode_::Instruction(ref inst) => {
                let ops = inst.ops.read().unwrap();

                match inst.v {
                    Instruction_::CmpOp(op, op1, op2) => {
                        let op1 = &ops[op1];
                        let op2 = &ops[op2];
                        self.aarch64_emit_cmp_res_op(op, op1, op2, f_content, f_context, vm)
                    }
                    _ => panic!("expect cmp res to emit")
                }
            }
            _ => panic!("expect cmp res to emit")
        }
    }

    fn aarch64_emit_calculate_address(&mut self, dest: &P<Value>, src: &P<Value>, f_context: &mut FunctionContext, vm: &VM) {
        let src = self.aarch64_emit_mem(&src, f_context, vm);
        match src.v {
            // offset(base,index,scale)
            Value_::Memory(MemoryLocation::Address{ref base, ref offset, shift, signed}) => {
                if offset.is_some() {
                    let ref offset = offset.as_ref().unwrap();

                    if self.aarch64_match_value_iimm(&offset) {
                        let offset = self.aarch64_value_iimm_to_u64(&offset);
                        if offset == 0 {
                            // Offset is 0, address calculation is trivial
                            self.backend.emit_mov(&dest, &base);
                        } else {
                            self.aarch64_emit_add_u64(&dest, &base, f_context, vm, offset as u64);
                        }
                    } else if offset.is_int_reg() {
                        self.backend.emit_add_ext(&dest, &base, &offset, signed, shift);
                    } else {
                        panic!("Offset should be an integer register or a constant")
                    }
                } else {
                    // Simple base address
                    self.backend.emit_mov(&dest, &base);
                }
            },

            Value_::Memory(MemoryLocation::Symbolic{..}) => {
                self.backend.emit_adr(&dest, &src);
            },
            _ => panic!("expect mem location as value")
        }
    }
    // TODO: Check ZEXT and SEXT are happening when they should
    fn aarch64_emit_cmp_res_op(&mut self, op: CmpOp, op1: &P<TreeNode>, op2: &P<TreeNode>, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) -> op::CmpOp {
        let mut op1 = op1;
        let mut op2 = op2;
        let mut op = op;
        if op == CmpOp::FFALSE || op == CmpOp::FTRUE {
            return op; // No comparison needed
        }
        use std;
        let mut swap = false; // Whether op1 and op2 have been swapped
        if op::is_int_cmp(op) {
            let n = self.aarch64_node_type(op1).get_int_length().unwrap();

            if self.aarch64_match_node_iimm(op1) && self.aarch64_match_node_iimm(op2) {
                let val1 = self.aarch64_node_iimm_to_u64(op1);
                let val2 = self.aarch64_node_iimm_to_u64(op2);

                let result = if match op {
                    op::CmpOp::SGE => get_signed_value(val1, n) >= get_signed_value(val2, n),
                    op::CmpOp::SLE => get_signed_value(val1, n) <= get_signed_value(val2, n),

                    op::CmpOp::SGT => get_signed_value(val1, n) > get_signed_value(val2, n),
                    op::CmpOp::SLT => get_signed_value(val1, n) < get_signed_value(val2, n),

                    op::CmpOp::UGE => val1 >= val2,
                    op::CmpOp::ULE => val1 <= val2,

                    op::CmpOp::UGT => val1 > val2,
                    op::CmpOp::ULT => val1 < val2,
                    op::CmpOp::EQ => val1 == val2,
                    op::CmpOp::NE => val1 != val2,
                    _ => panic!("Unknown integer comparison op")
                } { 0b0100 << 27 } else { 0 };
                let tmp_nzcv = self.make_temporary(f_context, UINT32_TYPE.clone(), vm);
                self.aarch64_emit_mov_u64(&tmp_nzcv, result);
                self.backend.emit_msr("NZCV", &tmp_nzcv);
                return op::CmpOp::EQ;
            }

            let mut imm_val = 0 as u64;
            // Is one of the arguments a valid immediate?
            let emit_imm = if self.aarch64_match_node_iimm(&op2) {
                imm_val = self.aarch64_node_iimm_to_u64(&op2);
                if op.is_signed() {
                    imm_val = get_signed_value(imm_val, n) as u64;
                }
                is_valid_arithmetic_imm(imm_val)
            } else if self.aarch64_match_node_iimm(&op1) {
                imm_val = self.aarch64_node_iimm_to_u64(&op1);
                if op.is_signed() {
                    imm_val = get_signed_value(imm_val, n) as u64;
                }
                // if op1 is a valid immediate, swap it with op2
                if is_valid_arithmetic_imm(imm_val) {
                    std::mem::swap(&mut op1, &mut op2);
                    swap = true;
                    true
                } else {
                    false
                }
            } else {
                false
            };

            if swap {
                op = op.swap_operands()
            }

            if emit_imm {

                // TODO: Sign extend the immediate?
                let reg_op1 = self.aarch64_emit_ireg(op1, f_content, f_context, vm);
                let imm_shift = imm_val > 4096;
                let imm_op2 = if imm_shift { imm_val >> 12 } else { imm_val };

                if op.is_signed() {
                    self.aarch64_emit_sext(&reg_op1);
                } else {
                    self.aarch64_emit_zext(&reg_op1);
                }

                self.backend.emit_cmp_imm(&reg_op1, imm_op2 as u16, imm_shift);
            } else {
                let reg_op1 = self.aarch64_emit_ireg(op1, f_content, f_context, vm);
                let reg_op2 = self.aarch64_emit_ireg(op2, f_content, f_context, vm);

                if op.is_signed() {
                    self.aarch64_emit_sext(&reg_op1);
                    self.aarch64_emit_sext(&reg_op2);
                } else {
                    self.aarch64_emit_zext(&reg_op1);
                    self.aarch64_emit_zext(&reg_op2);
                }
                self.backend.emit_cmp(&reg_op1, &reg_op2);
            }

            return op;
        } else {
            // Can do the comparison now
            if self.aarch64_match_f32imm(op1) && self.aarch64_match_f32imm(op2) {
                let val1 = self.aarch64_node_iimm_to_f32(op1);
                let val2 = self.aarch64_node_iimm_to_f32(op2);

                let result = if match op {
                    // Note: rusts comparison operations are all 'ordered', except != (since it is the inverse of ==)
                    op::CmpOp::FUNO => val1.is_nan() || val2.is_nan(),
                    op::CmpOp::FUEQ => val1.is_nan() || val2.is_nan() || val1 == val2,
                    op::CmpOp::FUNE => val1 != val2,
                    op::CmpOp::FUGE => val1.is_nan() || val2.is_nan() || val1 >= val2,
                    op::CmpOp::FULE => val1.is_nan() || val2.is_nan() || val1 <= val2,
                    op::CmpOp::FUGT => val1.is_nan() || val2.is_nan() || val1 > val2,
                    op::CmpOp::FULT => val1.is_nan() || val2.is_nan() || val1 < val2,

                    op::CmpOp::FORD => !val1.is_nan() && !val2.is_nan(),
                    op::CmpOp::FOEQ => val1 == val2,
                    op::CmpOp::FONE => !val1.is_nan() && !val2.is_nan() && val1 != val2,
                    op::CmpOp::FOGE => val1 >= val2,
                    op::CmpOp::FOLE => val1 <= val2,
                    op::CmpOp::FOGT => val1 > val2,
                    op::CmpOp::FOLT => val1 < val2,
                    _ => panic!("Unknown floating point comparison op")
                } { 0b0100 << 27 } else { 0 };
                let tmp_nzcv = self.make_temporary(f_context, UINT32_TYPE.clone(), vm);
                self.aarch64_emit_mov_u64(&tmp_nzcv, result);
                self.backend.emit_msr("NZCV", &tmp_nzcv);
                return op::CmpOp::FOEQ;
            } else if self.aarch64_match_f64imm(op1) && self.aarch64_match_f64imm(op2) {
                // Same as above, but for f64
                let val1 = self.aarch64_node_iimm_to_f64(op1);
                let val2 = self.aarch64_node_iimm_to_f64(op2);

                let result = if match op {
                    op::CmpOp::FUNO => val1.is_nan() || val2.is_nan(),
                    op::CmpOp::FUEQ => val1.is_nan() || val2.is_nan() || val1 == val2,
                    op::CmpOp::FUNE => val1 != val2,
                    op::CmpOp::FUGE => val1.is_nan() || val2.is_nan() || val1 >= val2,
                    op::CmpOp::FULE => val1.is_nan() || val2.is_nan() || val1 <= val2,
                    op::CmpOp::FUGT => val1.is_nan() || val2.is_nan() || val1 > val2,
                    op::CmpOp::FULT => val1.is_nan() || val2.is_nan() || val1 < val2,

                    op::CmpOp::FORD => !val1.is_nan() && !val2.is_nan(),
                    op::CmpOp::FOEQ => val1 == val2,
                    op::CmpOp::FONE => !val1.is_nan() && !val2.is_nan() && val1 != val2,
                    op::CmpOp::FOGE => val1 >= val2,
                    op::CmpOp::FOLE => val1 <= val2,
                    op::CmpOp::FOGT => val1 > val2,
                    op::CmpOp::FOLT => val1 < val2,
                    _ => panic!("Unknown floating point comparison op")
                } { 0b0100 << 27 } else { 0 };
                let tmp_nzcv = self.make_temporary(f_context, UINT32_TYPE.clone(), vm);
                self.aarch64_emit_mov_u64(&tmp_nzcv, result);
                self.backend.emit_msr("NZCV", &tmp_nzcv);
                return op::CmpOp::FOEQ;
            }

            // Is one of the arguments 0
            let emit_imm = if self.aarch64_match_f32imm(&op2) {
                self.aarch64_node_iimm_to_f32(&op2) == 0.0
            } else if self.aarch64_match_f32imm(&op1) {
                if self.aarch64_node_iimm_to_f32(&op1) == 0.0 {
                    std::mem::swap(&mut op1, &mut op2);
                    swap = true;
                    true
                } else {
                    false
                }
            } else if self.aarch64_match_f64imm(&op2) {
                self.aarch64_node_iimm_to_f64(&op2) == 0.0
            } else if self.aarch64_match_f64imm(&op1) {
                if self.aarch64_node_iimm_to_f64(&op1) == 0.0 {
                    std::mem::swap(&mut op1, &mut op2);
                    swap = true;
                    true
                } else {
                    false
                }
            } else {
                false
            };

            if emit_imm {
                let reg_op1 = self.aarch64_emit_fpreg(op1, f_content, f_context, vm);
                self.backend.emit_fcmp_0(&reg_op1);
            } else {
                let reg_op1 = self.aarch64_emit_fpreg(op1, f_content, f_context, vm);
                let reg_op2 = self.aarch64_emit_fpreg(op2, f_content, f_context, vm);

                self.backend.emit_fcmp(&reg_op1, &reg_op2);
            }

            if swap {
                // Swap the comparison operation
                match op {
                    op::CmpOp::FUGE => CmpOp::FULE,
                    op::CmpOp::FULE => CmpOp::FUGE,

                    op::CmpOp::FUGT => CmpOp::FULT,
                    op::CmpOp::FULT => CmpOp::FUGT,

                    op::CmpOp::FOGE => CmpOp::FOLE,
                    op::CmpOp::FOLE => CmpOp::FOGE,

                    op::CmpOp::FOGT => CmpOp::FOLT,
                    op::CmpOp::FOLT => CmpOp::FOGT,
                    _ => op // all Other floating point comparisons are reflexive
                }
            } else {
                op
            }
        }
    }

    fn node_type(&mut self, op: &TreeNode) -> P<MuType> {
        match op.v {
            TreeNode_::Instruction(ref inst) => {
                if inst.value.is_some() {
                    if inst.value.as_ref().unwrap().len() > 1 {
                        panic!("Too many results from instruction");
                    }

                    let ref value = inst.value.as_ref().unwrap()[0];

                    value.ty.clone()
                } else {
                    panic!("Instruction has no values");
                }
            }

            TreeNode_::Value(ref pv) => {
                pv.ty.clone()
            }
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

    // Decrement the register by an immediate value
    fn aarch64_emit_sub_u64(&mut self, dest: &P<Value>, src: &P<Value>, f_context: &mut FunctionContext, vm: &VM, val: u64)
    {
        if (val as i64) < 0 {
            self.aarch64_emit_add_u64(&dest, &src, f_context, vm, (-(val as i64) as u64));
        } else if val == 0 {
            // Operation has no effect
        } else if is_valid_arithmetic_imm(val) {
            let imm_shift = val > 4096;
            let imm_val = if imm_shift { val >> 12 } else { val };
            self.backend.emit_sub_imm(&dest, &src, imm_val as u16, imm_shift);
        } else {
            let tmp = self.make_temporary(f_context, UINT64_TYPE.clone(), vm);
            self.aarch64_emit_mov_u64(&tmp, val);
            self.backend.emit_sub(&dest, &src, &tmp);
        }
    }

    // Increment the register by an immediate value
    fn aarch64_emit_add_u64(&mut self, dest: &P<Value>, src: &P<Value>, f_context: &mut FunctionContext, vm: &VM, val: u64)
    {
        if (val as i64) < 0 {
            self.aarch64_emit_sub_u64(&dest, &src, f_context, vm, (-(val as i64) as u64));
        } else if val == 0 {
            // Operation has no effect
        } else if is_valid_arithmetic_imm(val) {
            let imm_shift = val > 4096;
            let imm_val = if imm_shift { val >> 12 } else { val };
            self.backend.emit_add_imm(&dest, &src, imm_val as u16, imm_shift);
        } else {
            let tmp = self.make_temporary(f_context, UINT64_TYPE.clone(), vm);
            self.aarch64_emit_mov_u64(&tmp, val);
            self.backend.emit_add(&dest, &src, &tmp);
        }
    }

    // Compare register with value
    fn aarch64_emit_cmp_u64(&mut self, src1: &P<Value>, f_context: &mut FunctionContext, vm: &VM, val: u64)
    {
        if (val as i64) < 0 {
            self.aarch64_emit_cmn_u64(&src1, f_context, vm, (-(val as i64) as u64));
        } else if val == 0 {
            // Operation has no effect
        } else if is_valid_arithmetic_imm(val) {
            let imm_shift = val > 4096;
            let imm_val = if imm_shift { val >> 12 } else { val };
            self.backend.emit_cmp_imm(&src1, imm_val as u16, imm_shift);
        } else {
            let tmp = self.make_temporary(f_context, UINT64_TYPE.clone(), vm);
            self.aarch64_emit_mov_u64(&tmp, val);
            self.backend.emit_cmp(&src1, &tmp);
        }
    }

    // Compare register with value
    fn aarch64_emit_cmn_u64(&mut self, src1: &P<Value>, f_context: &mut FunctionContext, vm: &VM, val: u64)
    {
        if (val as i64) < 0 {
            self.aarch64_emit_cmp_u64(&src1, f_context, vm, (-(val as i64) as u64));
        } else if val == 0 {
            // Operation has no effect
        } else if is_valid_arithmetic_imm(val) {
            let imm_shift = val > 4096;
            let imm_val = if imm_shift { val >> 12 } else { val };
            self.backend.emit_cmn_imm(&src1, imm_val as u16, imm_shift);
        } else {
            let tmp = self.make_temporary(f_context, UINT64_TYPE.clone(), vm);
            self.aarch64_emit_mov_u64(&tmp, val);
            self.backend.emit_cmn(&src1, &tmp);
        }
    }
    
    // sign extends reg, to fit in a 32/64 bit register
    fn aarch64_emit_sext(&mut self, reg: &P<Value>)
    {
        let nreg = check_op_len(&reg.ty); // The size of the aarch64 register
        let nmu = reg.ty.get_int_length().unwrap(); // the size of the Mu type

        if nreg > nmu {
            self.backend.emit_sbfx(&reg, &reg, 0, nmu as u8);
        }
    }

    // zero extends reg, to fit in a 32/64 bit register
    fn aarch64_emit_zext(&mut self, reg: &P<Value>)
    {
        let nreg = check_op_len(&reg.ty); // The size of the aarch64 register
        let nmu = reg.ty.get_int_length().unwrap(); // the size of the Mu type

        if nreg > nmu {
            self.backend.emit_ubfx(&reg, &reg, 0, nmu as u8);
        }
    }

    // one extends reg, to fit in a 32/64 bit register
    fn aarch64_emit_oext(&mut self, reg: &P<Value>)
    {
        let nreg = check_op_len(&reg.ty); // The size of the aarch64 register
        let nmu = reg.ty.get_int_length().unwrap(); // the size of the Mu type

        if nreg > nmu {
            self.backend.emit_orr_imm(&reg, &reg, bits_ones(nreg - nmu) << nmu)
        }
    }

    // Masks 'src' so that it can be used to shift 'dest'
    // Returns a register that should be used for the shift operand (may be dest or src)
    fn aarch64_emit_shift_mask<'b>(&mut self, dest: &'b P<Value>, src: &'b P<Value>) -> &'b P<Value>
    {
        let ndest = dest.ty.get_int_length().unwrap() as u64;

        if ndest < 32 { // 16 or 8 bits (need to mask it)
            self.backend.emit_and_imm(&dest, &src, ndest - 1);
            &dest
        } else {
            &src
        }
    }

    fn aarch64_emit_mov_u64(&mut self, dest: &P<Value>, val: u64)
    {
        let n = dest.ty.get_int_length().unwrap();
        // Can use one instruction
        if n <= 16 {
            self.backend.emit_movz(&dest, val as u16, 0);
        } else if val == 0 {
            self.backend.emit_movz(&dest, 0, 0);
        } else if val == (-1i64) as u64 {
            self.backend.emit_movn(&dest, 0, 0);
        } else if val > 0xFF && is_valid_logical_imm(val, n) {
            // Value is more than 16 bits
            self.backend.emit_mov_imm(&dest, replicate_logical_imm(val, n));

        // Have to use more than one instruciton
        } else {
            // Note n > 16, so there are at least two halfwords in n

            // How many halfowrds are zero or one
            let mut n_zeros = ((val & 0xFF == 0x00) as u64) + ((val & 0xFF00 == 0x0000) as u64);
            let mut n_ones = ((val & 0xFF == 0xFF) as u64) + ((val & 0xFF00 == 0xFF00) as u64);
            if n >= 32 {
                n_zeros += (val & 0xFF0000 == 0xFF0000) as u64;
                n_ones += (val & 0xFF0000 == 0xFF0000) as u64;
                if n >= 48 {
                    n_zeros += (val & 0xFF000000 == 0xFF000000) as u64;
                    n_ones += (val & 0xFF000000 == 0xFF000000) as u64;
                }
            }

            let (pv0, pv1, pv2, pv3) = split_aarch64_iimm(val);
            let mut movzn = false; // whether a movz/movn has been emmited yet

            if n_ones > n_zeros {
                // It will take less instructions to use MOVN
                // MOVN(dest, v, n) will set dest = !(v << n)

                if pv0 != 0xFF {
                    self.backend.emit_movn(&dest, !pv0, 0);
                    movzn = true;
                }
                if pv1 != 0xFF {
                    if !movzn {
                        self.backend.emit_movn(&dest, !pv1, 16);
                        movzn = true;
                    } else {
                        self.backend.emit_movk(&dest, pv1, 16);
                    }
                }
                if n >= 32 && pv2 != 0xFF {
                    if !movzn {
                        self.backend.emit_movn(&dest, !pv2, 32);
                        movzn = true;
                    } else {
                        self.backend.emit_movk(&dest, pv2, 32);
                    }
                }
                if n >= 48 && pv3 != 0xFF {
                    if !movzn {
                        self.backend.emit_movn(&dest, pv3, 48);
                    } else {
                        self.backend.emit_movk(&dest, pv3, 48);
                    }
                }
            } else {
                // It will take less instructions to use MOVZ
                // MOVZ(dest, v, n) will set dest = (v << n)
                // MOVK(dest, v, n) will set dest = dest[64-0]:[n];
                if pv0 != 0 {
                    self.backend.emit_movz(&dest, pv0, 0);
                    movzn = true;
                }
                if pv1 != 0 {
                    if !movzn {
                        self.backend.emit_movz(&dest, pv1, 16);
                        movzn = true;
                    } else {
                        self.backend.emit_movk(&dest, pv1, 16);
                    }
                }
                if n >= 32 && pv2 != 0 {
                    if !movzn {
                        self.backend.emit_movz(&dest, pv2, 32);
                        movzn = true;
                    } else {
                        self.backend.emit_movk(&dest, pv2, 32);
                    }
                }
                if n >= 48 && pv3 != 0 {
                    if !movzn {
                        self.backend.emit_movz(&dest, pv3, 48);
                    } else {
                        self.backend.emit_movk(&dest, pv3, 48);
                    }
                }
            }
        }
    }

    fn aarch64_emit_mov_f64(&mut self, dest: &P<Value>, f_context: &mut FunctionContext, vm: &VM, val: f64)
    {
        use std::mem;
        if is_valid_f64_imm(val) {
            self.backend.emit_fmov_imm(&dest, val as f32);
        } else {
            match f64_to_aarch64_u64(val) {
                Some(v) => {
                    // Can use a MOVI to load the immediate
                    self.backend.emit_movi(&dest, v);
                }
                None => {
                    // Have to load a temporary GPR with the value first
                    let tmp_int = self.make_temporary(f_context, UINT64_TYPE.clone(), vm);
                    self.aarch64_emit_mov_u64(&tmp_int, unsafe { mem::transmute::<f64, u64>(val) });

                    // then move it to an FPR
                    self.backend.emit_fmov(&dest, &tmp_int);
                }
            }
        }
    }

    fn aarch64_emit_mov_f32(&mut self, dest: &P<Value>, f_context: &mut FunctionContext, vm: &VM, val: f32)
    {
        use std::mem;
        if is_valid_f32_imm(val) {
            self.backend.emit_fmov_imm(&dest, val);
        } else {
            // Have to load a temporary GPR with the value first
            let tmp_int = self.make_temporary(f_context, UINT32_TYPE.clone(), vm);

            self.aarch64_emit_mov_u64(&tmp_int, unsafe { mem::transmute::<f32, u32>(val) } as u64);
            // then move it to an FPR
            self.backend.emit_fmov(&dest, &tmp_int);
        }
    }

    // Emits a reg (either an ireg or freg)
    fn aarch64_emit_reg(&mut self, op: &TreeNode, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) -> P<Value> {
        match op.v {
            TreeNode_::Instruction(_) => {
                self.aarch64_instruction_select(op, f_content, f_context, vm);

                self.get_result_value(op)
            },
            TreeNode_::Value(ref pv) => self.aarch64_emit_reg_value(pv, f_context, vm)
        }
    }

    // TODO: Deal with memory case
    fn aarch64_emit_reg_value(&mut self, pv: &P<Value>, f_context: &mut FunctionContext, vm: &VM) -> P<Value> {
        match pv.v {
            Value_::SSAVar(_) => pv.clone(),
            Value_::Constant(ref c) => {
                let tmp = self.make_temporary(f_context, pv.ty.clone(), vm);
                match c {
                    &Constant::Int(val) => {
                        debug!("tmp's ty: {}", tmp.ty);
                        self.aarch64_emit_mov_u64(&tmp, val);
                    },
                    &Constant::FuncRef(_) => {
                        unimplemented!()
                    },
                    &Constant::NullRef => {
                        self.backend.emit_movz(&tmp, 0, 0);
                    },
                    &Constant::Double(val) => {
                        self.aarch64_emit_mov_f64(&tmp, f_context, vm, val);
                    }
                    &Constant::Float(val) => {
                        self.aarch64_emit_mov_f32(&tmp, f_context, vm, val);
                    },
                    _ => panic!("expected fpreg or ireg")
                }

                tmp
            },
            _ => panic!("expected fpreg or ireg")
        }
    }

    fn aarch64_emit_ireg(&mut self, op: &TreeNode, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) -> P<Value> {
        match op.v {
            TreeNode_::Instruction(_) => {
                self.aarch64_instruction_select(op, f_content, f_context, vm);

                self.get_result_value(op)
            },
            TreeNode_::Value(ref pv) => self.aarch64_emit_ireg_value(pv, f_context, vm)
        }
    }

    // TODO: Deal with memory case
    fn aarch64_emit_ireg_value(&mut self, pv: &P<Value>, f_context: &mut FunctionContext, vm: &VM) -> P<Value> {
        match pv.v {
            Value_::SSAVar(_) => pv.clone(),
            Value_::Constant(ref c) => {
                let tmp = self.make_temporary(f_context, pv.ty.clone(), vm);
                match c {
                    &Constant::Int(val) => {
                        debug!("tmp's ty: {}", tmp.ty);
                        self.aarch64_emit_mov_u64(&tmp, val);
                    },
                    &Constant::FuncRef(_) => {
                        unimplemented!()
                    },
                    &Constant::NullRef => {
                        self.backend.emit_movz(&tmp, 0, 0);
                    },
                    _ => panic!("expected ireg")
                }

                tmp
            },
            _ => panic!("expected ireg")
        }
    }

    fn aarch64_emit_fpreg(&mut self, op: &TreeNode, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) -> P<Value> {
        match op.v {
            TreeNode_::Instruction(_) => {
                self.aarch64_instruction_select(op, f_content, f_context, vm);
                self.get_result_value(op)
            },
            TreeNode_::Value(ref pv) => self.aarch64_emit_fpreg_value(pv, f_context, vm)
        }
    }

    // TODO: Deal with memory case
    fn aarch64_emit_fpreg_value(&mut self, pv: &P<Value>, f_context: &mut FunctionContext, vm: &VM) -> P<Value> {
        match pv.v {
            Value_::SSAVar(_) => pv.clone(),
            Value_::Constant(Constant::Double(val)) => {
                let tmp = self.make_temporary(f_context, DOUBLE_TYPE.clone(), vm);
                self.aarch64_emit_mov_f64(&tmp, f_context, vm, val);
                tmp
            }
            Value_::Constant(Constant::Float(val)) => {
                let tmp = self.make_temporary(f_context, FLOAT_TYPE.clone(), vm);
                self.aarch64_emit_mov_f32(&tmp, f_context, vm, val);
                tmp
            },
            _ => panic!("expected fpreg")
        }
    }

    fn aarch64_match_f32imm(&mut self, op: &TreeNode) -> bool {
        match op.v {
            TreeNode_::Value(ref pv) => match pv.v {
                Value_::Constant(Constant::Float(_)) => true,
                _ => false
            },
            _ => false
        }
    }

    fn aarch64_match_f64imm(&mut self, op: &TreeNode) -> bool {
        match op.v {
            TreeNode_::Value(ref pv) => match pv.v {
                Value_::Constant(Constant::Double(_)) => true,
                _ => false
            },
            _ => false
        }
    }

    fn aarch64_match_value_f64imm(&mut self, op: &P<Value>) -> bool {
        match op.v {
            Value_::Constant(Constant::Double(_)) => true,
            _ => false
        }
    }

    fn aarch64_match_value_f32imm(&mut self, op: &P<Value>) -> bool {
        match op.v {
            Value_::Constant(Constant::Float(_)) => true,
            _ => false
        }
    }

    #[warn(unused_variables)] // The type of the node (for a value node)
    fn aarch64_node_type(&mut self, op: &TreeNode) -> P<MuType> {
        match op.v {
            TreeNode_::Value(ref pv) => pv.ty.clone(),
            _ => panic!("expected node value")
        }
    }

    fn aarch64_match_value_iimm(&mut self, op: &P<Value>) -> bool {
        match op.v {
            Value_::Constant(Constant::Int(_)) => true,
            _ => false
        }
    }

    fn aarch64_match_node_iimm(&mut self, op: &TreeNode) -> bool {
        match op.v {
            TreeNode_::Value(ref pv) => self.aarch64_match_value_iimm(pv),
            _ => false
        }
    }

    fn aarch64_node_iimm_to_u64(&mut self, op: &TreeNode) -> u64 {
        match op.v {
            TreeNode_::Value(ref pv) => self.aarch64_value_iimm_to_u64(pv),
            _ => panic!("expected iimm")
        }
    }

    fn aarch64_node_iimm_to_f64(&mut self, op: &TreeNode) -> f64 {
        match op.v {
            TreeNode_::Value(ref pv) => self.aarch64_value_iimm_to_f64(pv),
            _ => panic!("expected iimm")
        }
    }

    fn aarch64_node_iimm_to_f32(&mut self, op: &TreeNode) -> f32 {
        match op.v {
            TreeNode_::Value(ref pv) => self.aarch64_value_iimm_to_f32(pv),
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

    fn aarch64_value_iimm_to_f32(&mut self, op: &P<Value>) -> f32 {
        match op.v {
            Value_::Constant(Constant::Float(val)) => {
                val as f32
            },
            _ => panic!("expected iimm float")
        }
    }

    fn aarch64_value_iimm_to_f64(&mut self, op: &P<Value>) -> f64 {
        match op.v {
            Value_::Constant(Constant::Double(val)) => {
                val as f64
            },
            _ => panic!("expected iimm double")
        }
    }

    fn aarch64_value_iimm_to_u64(&self, op: &P<Value>) -> u64 {
        match op.v {
            Value_::Constant(Constant::Int(val)) =>
                get_unsigned_value(val as u64, op.ty.get_int_length().unwrap()),
            _ => panic!("expected iimm int")
        }
    }

    fn aarch64_value_iimm_to_i64(&self, op: &P<Value>) -> i64 {
        match op.v {
            Value_::Constant(Constant::Int(val)) =>
                get_signed_value(val as u64, op.ty.get_int_length().unwrap()),
            _ => panic!("expected iimm int")
        }
    }


    // TODO: what exactly is this doing??
    fn aarch64_emit_node_addr_to_value(&mut self, op: &TreeNode, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) -> P<Value> {
        match op.v {
            TreeNode_::Value(ref pv) => {
                match pv.v {
                    Value_::SSAVar(_) => P(Value{
                        hdr: MuEntityHeader::unnamed(vm.next_id()),
                        ty: types::get_referent_ty(& pv.ty).unwrap(),
                        v: Value_::Memory(MemoryLocation::Address{
                            base: pv.clone(),
                            offset: None,
                            shift: 0,
                            signed: false
                        })
                    }),
                    Value_::Global(_) => {
                        if vm.is_running() {
                            // get address from vm
                            unimplemented!()
                        } else {
                            // symbolic
                            if cfg!(target_os = "linux") {
                                let got_loc = P(Value {
                                    hdr: MuEntityHeader::unnamed(vm.next_id()),
                                    ty: pv.ty.clone(),
                                    v: Value_::Memory(MemoryLocation::Symbolic {
                                        label: pv.name().unwrap(),
                                        is_global: true
                                    })
                                });

                                // mov (got_loc) -> actual_loc
                                let actual_loc = self.make_temporary(f_context, pv.ty.clone(), vm);
                                self.aarch64_emit_move_value_to_value(&actual_loc, &got_loc, f_context, vm);
                                self.aarch64_make_value_base_offset(&actual_loc, 0, &types::get_referent_ty(&pv.ty).unwrap(), vm)
                            } else {
                                unimplemented!()
                            }
                        }
                    },
                    Value_::Memory(_) => pv.clone(),
                    Value_::Constant(_) => unimplemented!()
                }
            }
            TreeNode_::Instruction(_) => self.aarch64_emit_get_mem_from_inst(op, f_content, f_context, vm)
        }
    }

    fn aarch64_emit_get_mem_from_inst(&mut self, op: &TreeNode, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) -> P<Value> {
        let mem = self.aarch64_emit_get_mem_from_inst_inner(op, f_content, f_context, vm);

        P(Value{
            hdr: MuEntityHeader::unnamed(vm.next_id()),
            ty: ADDRESS_TYPE.clone(),
            v: Value_::Memory(mem)
        })
    }

    fn aarch64_emit_get_mem_from_inst_inner(&mut self, op: &TreeNode, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) -> MemoryLocation {
        match op.v {
            TreeNode_::Instruction(ref inst) => {
                let ref ops = inst.ops.read().unwrap();

                match inst.v {
                    // GETIREF <T> opnd = &opnd
                    Instruction_::GetIRef(op_index) => {
                        let ref ref_op = ops[op_index];
                        let temp = self.aarch64_emit_ireg(ref_op, f_content, f_context, vm);

                        let ret = self.aarch64_make_memory_location_base_offset(&temp, 0, vm);
                        trace!("MEM from GETIREF: {}", ret);
                        ret
                    }

                    // GETFIELDIREF <T1 index> opnd = &opnd + T1.field_offset[i]
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

                        let field_offset = self.aarch64_get_field_offset(&struct_ty, index, vm);

                        match base.v {
                            TreeNode_::Instruction(Instruction{v: Instruction_::GetIRef{..}, ..}) |
                            TreeNode_::Instruction(Instruction{v: Instruction_::GetFieldIRef{..}, ..}) |
                            TreeNode_::Instruction(Instruction{v: Instruction_::GetElementIRef{..}, ..}) |
                            TreeNode_::Instruction(Instruction{v: Instruction_::GetVarPartIRef{..}, ..}) |
                            TreeNode_::Instruction(Instruction{v: Instruction_::ShiftIRef{..}, ..}) => {
                                let mem = self.aarch64_emit_get_mem_from_inst_inner(base, f_content, f_context, vm);
                                let ret = self.aarch64_memory_location_shift(mem, field_offset, f_context, vm);

                                trace!("MEM from GETFIELDIREF(inst): {}", ret);
                                ret
                            },

                            _ => {
                                let tmp = self.aarch64_emit_ireg(base, f_content, f_context, vm);
                                let ret = self.aarch64_make_memory_location_base_offset(&tmp, field_offset, vm);

                                trace!("MEM from GETFIELDIREF(ireg): {}", ret);
                                ret
                            }
                        }
                    }

                    // GETFIELDIREF <T1> opnd = &opnd + T1.var_offset[i]
                    Instruction_::GetVarPartIRef{base, ..} => {
                        let ref base = ops[base];

                        let struct_ty = match base.clone_value().ty.get_referenced_ty() {
                            Some(ty) => ty,
                            None => panic!("expecting an iref or uptr in GetVarPartIRef")
                        };

                        let fix_part_size = vm.get_backend_type_info(struct_ty.id()).size;

                        match base.v {
                            TreeNode_::Instruction(Instruction{v: Instruction_::GetIRef{..}, ..}) |
                            TreeNode_::Instruction(Instruction{v: Instruction_::GetFieldIRef{..}, ..}) |
                            TreeNode_::Instruction(Instruction{v: Instruction_::GetElementIRef{..}, ..}) |
                            TreeNode_::Instruction(Instruction{v: Instruction_::GetVarPartIRef{..}, ..}) |
                            TreeNode_::Instruction(Instruction{v: Instruction_::ShiftIRef{..}, ..}) => {
                                let mem = self.aarch64_emit_get_mem_from_inst_inner(base, f_content, f_context, vm);

                                let ret = self.aarch64_memory_location_shift(mem, fix_part_size as i64, f_context, vm);

                                trace!("MEM from GETIVARPARTIREF(inst): {}", ret);
                                ret
                            },
                            _ => {
                                let tmp = self.aarch64_emit_ireg(base, f_content, f_context, vm);
                                let ret = self.aarch64_make_memory_location_base_offset(&tmp, fix_part_size as i64, vm);

                                trace!("MEM from GETVARPARTIREF(ireg): {}", ret);
                                ret
                            }
                        }
                    }

                    // GETFIELDIREF <T1 T2> opnd offset = &opnd + offset*T1.size
                    Instruction_::ShiftIRef{base, offset, ..} => {
                        let element_type = ops[base].clone_value().ty.get_referenced_ty().unwrap();
                        let element_size = vm.get_backend_type_info(element_type.id()).size;
                        self.aarch64_emit_offset_ref(&ops[base], &ops[offset], element_size, f_content, f_context, vm)
                    }
                    // GETELEMIREF <T1 T2> opnd index = &opnd + index*T1.element_size
                    Instruction_::GetElementIRef{base, index, ..} => {
                        let element_type = ops[base].clone_value().ty.get_referenced_ty().unwrap().get_elem_ty().unwrap();
                        let element_size = vm.get_backend_type_info(element_type.id()).size;

                        self.aarch64_emit_offset_ref(&ops[base], &ops[index], element_size, f_content, f_context, vm)
                    }
                    _ => panic!("Not a memory reference instruction")
                }
            },
            _ => panic!("expecting a instruction that yields a memory address")
        }
    }

    // Implementes SHIFTIREF and GETELEMENTIREF
    fn aarch64_emit_offset_ref(&mut self, base: &TreeNode, offset: &TreeNode, element_size: usize, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) -> MemoryLocation {
        if self.aarch64_match_node_iimm(offset) {
            let offset = self.aarch64_node_iimm_to_u64(offset);
            let shift_size = (element_size as i64) * (offset as i64);

            let mem = match base.v {
                // SHIFTIREF(GETVARPARTIREF(_), imm) -> add shift_size to old offset
                TreeNode_::Instruction(Instruction { v: Instruction_::GetIRef { .. }, .. }) |
                TreeNode_::Instruction(Instruction { v: Instruction_::GetFieldIRef { .. }, .. }) |
                TreeNode_::Instruction(Instruction { v: Instruction_::GetElementIRef { .. }, .. }) |
                TreeNode_::Instruction(Instruction { v: Instruction_::GetVarPartIRef { .. }, .. }) |
                TreeNode_::Instruction(Instruction { v: Instruction_::ShiftIRef { .. }, .. }) => {
                    let mem = self.aarch64_emit_get_mem_from_inst_inner(base, f_content, f_context, vm);

                    let ret = self.aarch64_memory_location_shift(mem, shift_size, f_context, vm);

                    trace!("MEM from SHIFTIREF(inst, imm): {}", ret);
                    ret
                },
                // SHIFTIREF(ireg, imm) -> [base + SHIFT_SIZE]
                _ => {
                    let tmp = self.aarch64_emit_ireg(base, f_content, f_context, vm);

                    let ret = self.aarch64_make_memory_location_base_offset(&tmp, shift_size, vm);

                    trace!("MEM from SHIFTIREF(ireg, imm): {}", ret);
                    ret
                }
            };

            mem
        } else {
            let tmp_offset = self.aarch64_emit_ireg(offset, f_content, f_context, vm);

            let mem = match base.v {
                TreeNode_::Instruction(Instruction { v: Instruction_::GetIRef { .. }, .. }) |
                TreeNode_::Instruction(Instruction { v: Instruction_::GetFieldIRef { .. }, .. }) |
                TreeNode_::Instruction(Instruction { v: Instruction_::GetElementIRef { .. }, .. }) |
                TreeNode_::Instruction(Instruction { v: Instruction_::GetVarPartIRef { .. }, .. }) |
                TreeNode_::Instruction(Instruction { v: Instruction_::ShiftIRef { .. }, .. }) => {
                    let mem = self.aarch64_emit_get_mem_from_inst_inner(base, f_content, f_context, vm);
                    let ret = self.aarch64_memory_location_shift_scale(mem, &tmp_offset, element_size as u64, f_context, vm);

                    trace!("MEM from SHIFTIREF(inst, ireg): {}", ret);
                    ret
                },
                _ => {
                    let tmp = self.aarch64_emit_ireg(base, f_content, f_context, vm);
                    let ret = self.aarch64_make_memory_location_base_offset_scale(&tmp, &tmp_offset, element_size as u64, true);

                    trace!("MEM from SHIFTIREF(ireg, ireg): {}", ret);
                    ret
                }
            };

            mem
        } // */
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

    fn get_result_value(&mut self, node: &TreeNode) -> P<Value> {
        match node.v {
            TreeNode_::Instruction(ref inst) => {
                if inst.value.is_some() {
                    let ref value = inst.value.as_ref().unwrap()[0];

                    if inst.value.as_ref().unwrap().len() > 1 {
                        warn!("retrieving value from a node with more than one value: {}, use the first value: {}", node, value);
                    }

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

    // TODO: This has been modified to simply use iregs and fpregs (NEED TO FIX THIS??)
    fn aarch64_emit_node_value(&mut self, op: &TreeNode, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) -> P<Value> {
        match op.v {
            TreeNode_::Instruction(_) => {
                self.aarch64_instruction_select(op, f_content, f_context, vm);

                self.get_result_value(op)
            },
            TreeNode_::Value(ref pv) => pv.clone()
        }
    }

    // TODO: This has been modified to simply use iregs and fpregs (NEED TO FIX THIS??)
    fn aarch64_emit_move_node_to_value(&mut self, dest: &P<Value>, src: &TreeNode, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) {
        let ref dst_ty = dest.ty;

        if !types::is_fp(dst_ty) && types::is_scalar(dst_ty) {
            if self.aarch64_match_node_iimm(src) {
                let src_imm = self.aarch64_node_iimm_to_u64(src);
                if dest.is_int_reg() {
                    self.aarch64_emit_mov_u64(dest, src_imm);
                } else if dest.is_mem() {
                    let tmp = self.make_temporary(f_context, dest.ty.clone(), vm);
                    self.aarch64_emit_mov_u64(&tmp, src_imm);
                    self.aarch64_emit_store(dest, &tmp, f_context, vm);
                } else {
                    panic!("unexpected dest: {}", dest);
                }
            } else if self.match_ireg(src) {
                let src_reg = self.aarch64_emit_ireg(src, f_content, f_context, vm);
                self.aarch64_emit_move_value_to_value(dest, &src_reg, f_context, vm);
            } else {
                panic!("expected src: {}", src);
            }
        } else if types::is_fp(dst_ty) && types::is_scalar(dst_ty) {
            if self.aarch64_match_node_iimm(src) {
                if dst_ty.v == MuType_::Double {
                    let src_imm = self.aarch64_node_iimm_to_f64(src);
                    if dest.is_fp_reg() {
                        self.aarch64_emit_mov_f64(dest, f_context, vm, src_imm);
                    } else if dest.is_mem() {
                        let tmp = self.make_temporary(f_context, dest.ty.clone(), vm);
                        self.aarch64_emit_mov_f64(&tmp, f_context, vm,  src_imm);
                        self.aarch64_emit_store(dest, &tmp, f_context, vm);
                    } else {
                        panic!("unexpected dest: {}", dest);
                    }
                } else  { // dst_ty.v == MuType_::Float
                    let src_imm = self.aarch64_node_iimm_to_f32(src);
                    if dest.is_fp_reg() {
                        self.aarch64_emit_mov_f32(dest, f_context, vm,  src_imm);
                    } else if dest.is_mem() {
                        let tmp = self.make_temporary(f_context, dest.ty.clone(), vm);
                        self.aarch64_emit_mov_f32(&tmp, f_context, vm,  src_imm);
                        self.aarch64_emit_store(dest, &tmp, f_context, vm);
                    } else {
                        panic!("unexpected dest: {}", dest);
                    }
                }
            }
            if self.match_fpreg(src) {
                let src_reg = self.aarch64_emit_fpreg(src, f_content, f_context, vm);
                self.aarch64_emit_move_value_to_value(dest, &src_reg, f_context, vm)
            } else {
                panic!("unexpected fp src: {}", src);
            }
        } else {
            unimplemented!()
        } 
    }

    fn aarch64_emit_move_value_to_value(&mut self, dest: &P<Value>, src: &P<Value>, f_context: &mut FunctionContext, vm: &VM) {
        let ref src_ty = src.ty;
        if types::is_scalar(src_ty) && !types::is_fp(src_ty) {
            // gpr mov
            if dest.is_int_reg() && src.is_int_const() {
                let imm = self.aarch64_value_iimm_to_u64(src);
                self.aarch64_emit_mov_u64(dest, imm);
            } else if dest.is_int_reg() && src.is_int_reg() {
                self.backend.emit_mov(dest, src);
            } else if dest.is_int_reg() && src.is_mem() {
                self.aarch64_emit_load(&dest, &src, f_context, vm);
            } else if dest.is_mem() {
                let temp = self.aarch64_emit_ireg_value(src, f_context, vm);
                self.aarch64_emit_store(dest, &temp, f_context, vm);
            } else {
                panic!("unexpected gpr mov between {} -> {}", src, dest);
            }
        } else if types::is_scalar(src_ty) && types::is_fp(src_ty) {
            // fpr mov
            if dest.is_fp_reg() && self.aarch64_match_value_f32imm(src) {
                let src = self.aarch64_value_iimm_to_f32(src);
                self.aarch64_emit_mov_f32(dest, f_context, vm, src);
            } else if dest.is_fp_reg() && self.aarch64_match_value_f64imm(src) {
                let src = self.aarch64_value_iimm_to_f64(src);
                self.aarch64_emit_mov_f64(dest, f_context, vm, src);
            } else if dest.is_fp_reg() && src.is_fp_reg() {
                self.backend.emit_fmov(dest, src);
            } else if dest.is_fp_reg() && src.is_mem() {
                self.aarch64_emit_load(&dest, &src, f_context, vm);
            } else if dest.is_mem() {
                let temp = self.aarch64_emit_fpreg_value(src, f_context, vm);
                self.aarch64_emit_store(dest, &temp, f_context, vm);
            } else {
                panic!("unexpected fpr mov between {} -> {}", src, dest);
            }
        } else {
            panic!("unexpected mov of type {}", src_ty)
        }
    }

    fn aarch64_emit_load(&mut self, dest: &P<Value>, src: &P<Value>, f_context: &mut FunctionContext, vm: &VM) {
        let src = self.aarch64_emit_mem(&src, f_context, vm);
        self.backend.emit_ldr(&dest, &src, false);
    }

    fn aarch64_emit_store(&mut self, dest: &P<Value>, src: &P<Value>, f_context: &mut FunctionContext, vm: &VM) {
        let dest = self.aarch64_emit_mem(&dest, f_context, vm);
        self.backend.emit_str(&dest, &src);
    }
    
    fn aarch64_emit_landingpad(&mut self, exception_arg: &P<Value>, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) {
        // get thread local and add offset to get exception_obj
        let tl = self.aarch64_emit_get_threadlocal(None, f_content, f_context, vm);
        self.aarch64_emit_load_base_offset(exception_arg, &tl, *thread::EXCEPTION_OBJ_OFFSET as i64, f_context, vm);
    }

    fn aarch64_get_field_offset(&mut self, ty: &P<MuType>, index: usize, vm: &VM) -> i64 {
        let ty_info = vm.get_backend_type_info(ty.id());
        let layout  = match ty_info.struct_layout.as_ref() {
            Some(layout) => layout,
            None => panic!("a struct type does not have a layout yet: {:?}", ty_info)
        };
        debug_assert!(layout.len() > index);

        layout[index] as i64
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

    fn aarch64_get_mem_for_const(&mut self, val: P<Value>, vm: &VM) -> P<Value> {
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

    fn start_function(&mut self, vm: &VM, func_ver: &mut MuFunctionVersion) {
        debug!("{}", self.name());

        let entry_block = func_ver.content.as_ref().unwrap().get_entry_block();

        self.current_fv_id = func_ver.id();
        self.current_frame = Some(Frame::new(func_ver.id()));
        self.current_func_start = Some({
            let funcs = vm.funcs().read().unwrap();
            let func = funcs.get(&func_ver.func_id).unwrap().read().unwrap();
            let start_loc = self.backend.start_code(func.name().unwrap(), entry_block.name().unwrap());
            if vm.vm_options.flag_emit_debug_info {
                self.backend.add_cfi_startproc();
            }

            start_loc
        });
        self.current_callsite_id = 0;
        self.current_exn_callsites.clear();
        self.current_exn_blocks.clear();

        self.current_constants.clear();
        self.current_constants_locs.clear();
        
        // prologue (get arguments from entry block first)
        let ref args = entry_block.content.as_ref().unwrap().args;
        //args: &Vec<P<Value>>, sig: &P<CFuncSig>, vm: &VM
        self.aarch64_emit_common_prologue(args, &func_ver.sig, &mut func_ver.context, vm);
    }

    fn visit_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        let f_content = func.content.as_ref().unwrap();
        
        for block_id in func.block_trace.as_ref().unwrap() {
            // is this block an exception block?
            let is_exception_block = f_content.exception_blocks.contains(&block_id);

            let block = f_content.get_block(*block_id);
            let block_label = block.name().unwrap();
            self.current_block = Some(block_label.clone());
            
            let block_content = block.content.as_ref().unwrap();

            if is_exception_block {
                // exception block
                // we need to be aware of exception blocks so that we can emit information to catch exceptions

                let loc = self.backend.start_exception_block(block_label.clone());
                self.current_exn_blocks.insert(block.id(), loc);
            } else {
                // normal block
                self.backend.start_block(block_label.clone());
            }

            if block.is_receiving_exception_arg() {
                // this block uses exception arguments
                // we need to add it to livein, and also emit landingpad for it

                let exception_arg = block_content.exn_arg.as_ref().unwrap();
                
                // live in is args of the block + exception arg
                let mut livein = block_content.args.to_vec();
                livein.push(exception_arg.clone());
                self.backend.set_block_livein(block_label.clone(), &livein);
                
                // need to insert a landing pad
                self.aarch64_emit_landingpad(&exception_arg, f_content, &mut func.context, vm);
            } else {
                // live in is args of the block
                self.backend.set_block_livein(block_label.clone(), &block_content.args);                    
            }
            
            // live out is the union of all branch args of this block
            let live_out = block_content.get_out_arguments();

            // doing the actual instruction selection
            for inst in block_content.body.iter() {
                self.aarch64_instruction_select(&inst, f_content, &mut func.context, vm);
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
    
    fn finish_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        self.backend.print_cur_code();
        
        let func_name = {
            let funcs = vm.funcs().read().unwrap();
            let func = funcs.get(&func.func_id).unwrap().read().unwrap();
            func.name().unwrap()
        };

        // have to do this before 'finish_code()'
        if vm.vm_options.flag_emit_debug_info {
            self.backend.add_cfi_endproc();
        }
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
