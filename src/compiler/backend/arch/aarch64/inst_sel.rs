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
use compiler::backend::EPILOGUE_BLOCK_NAME;

use compiler::backend::aarch64::*;
use compiler::machine_code::CompiledFunction;
use compiler::frame::Frame;

use std::collections::HashMap;
use std::any::Any;

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

// TODO: Move all functions that are in here that don't need access to 'self' (or only call functions that don't need access to self (even if called on self)) to Mod.rs
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
    fn instruction_select(&mut self, node: &'a TreeNode, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) {
        trace!("instsel on node#{} {}", node.id(), node);

        match node.v {
            TreeNode_::Instruction(ref inst) => {
                match inst.v {
                    // TODO: Optimise if cond is a flag from a binary operation?
                    Instruction_::Branch2 { cond, ref true_dest, ref false_dest, true_prob } => {
                        trace!("instsel on BRANCH2");
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
                            let mut cmpop = self.emit_cmp_res(cond, f_content, f_context, vm);
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
                        } else {
                            let cond_reg = self.emit_ireg(cond, f_content, f_context, vm);

                            if branch_if_true {
                                self.backend.emit_tbnz(&cond_reg, 0, branch_target.clone());
                            } else {
                                self.backend.emit_tbz(&cond_reg, 0, branch_target.clone());
                            }
                        };
                    },

                    Instruction_::Select { cond, true_val, false_val } => {
                        use ast::op::CmpOp::*;

                        trace!("instsel on SELECT");
                        let ops = inst.ops.read().unwrap();

                        let ref cond = ops[cond];
                        let ref true_val = ops[true_val];
                        let ref false_val = ops[false_val];

                        let tmp_res = self.get_result_value(node, 0);

                        // moving integers/pointers
                        // generate compare
                        let cmpop = if self.match_cmp_res(cond) {
                            self.emit_cmp_res(cond, f_content, f_context, vm)
                        } else if self.match_ireg(cond) {
                            let tmp_cond = self.emit_ireg(cond, f_content, f_context, vm);
                            self.backend.emit_cmp_imm(&tmp_cond, 0, false);
                            NE
                        } else {
                            panic!("expected ireg, found {}", cond)
                        };

                        let tmp_true = self.emit_reg(true_val, f_content, f_context, vm);
                        let tmp_false = self.emit_reg(false_val, f_content, f_context, vm);

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

                    Instruction_::CmpOp(op, op1, op2) => {
                        use ast::op::CmpOp::*;

                        trace!("instsel on CMPOP");
                        let ops = inst.ops.read().unwrap();
                        let ref op1 = ops[op1];
                        let ref op2 = ops[op2];

                        let tmp_res = self.get_result_value(node, 0);

                        debug_assert!(tmp_res.ty.get_int_length().is_some());
                        debug_assert!(tmp_res.ty.get_int_length().unwrap() == 1);

                        let cmpop = self.emit_cmp_res_op(op, &op1, &op2, f_content, f_context, vm);
                        let cond = get_condition_codes(cmpop);

                        if cmpop == FFALSE {
                            emit_mov_u64(self.backend.as_mut(), &tmp_res, 0);
                        } else if cmpop == FTRUE {
                            emit_mov_u64(self.backend.as_mut(), &tmp_res, 1);
                        } else {
                            self.backend.emit_cset(&tmp_res, cond[0]);

                            // Note: some compariosns can't be computed based on a single aarch64 flag
                            // insted they are computed as a condition OR NOT another condition.
                            if cond.len() == 2 {
                                self.backend.emit_csinc(&tmp_res, &tmp_res, &WZR, invert_condition_code(cond[1]));
                            }
                        }
                    }

                    Instruction_::Branch1(ref dest) => {
                        trace!("instsel on BRANCH1");
                        let ops = inst.ops.read().unwrap();

                        self.process_dest(&ops, dest, f_content, f_context, vm);

                        let target = f_content.get_block(dest.target).name().unwrap();

                        trace!("emit branch1");
                        // jmp
                        self.backend.emit_b(target);
                    },

                    Instruction_::Switch { cond, ref default, ref branches } => {
                        trace!("instsel on SWITCH");
                        let ops = inst.ops.read().unwrap();

                        let ref cond = ops[cond];

                        if self.match_ireg(cond) {
                            let tmp_cond = self.emit_ireg(cond, f_content, f_context, vm);
                            self.emit_zext(&tmp_cond);

                            // emit each branch
                            for &(case_op_index, ref case_dest) in branches {
                                let ref case_op = ops[case_op_index];

                                // process dest
                                self.process_dest(&ops, case_dest, f_content, f_context, vm);

                                let target = f_content.get_block(case_dest.target).name().unwrap();

                                let mut imm_val = 0 as u64;
                                // Is one of the arguments a valid immediate?
                                let emit_imm = if match_node_int_imm(&case_op) {
                                    imm_val = node_imm_to_u64(&case_op);
                                    is_valid_arithmetic_imm(imm_val)
                                } else {
                                    false
                                };

                                if emit_imm {
                                    let imm_shift = imm_val > 4096;
                                    let imm_op2 = if imm_shift { imm_val >> 12 } else { imm_val };
                                    self.backend.emit_cmp_imm(&tmp_cond, imm_op2 as u16, imm_shift);
                                } else {
                                    let tmp_case_op = self.emit_ireg(case_op, f_content, f_context, vm);
                                    self.emit_zext(&tmp_case_op);
                                    self.backend.emit_cmp(&tmp_cond, &tmp_case_op);
                                }

                                self.backend.emit_b_cond("EQ", target);

                                self.finish_block(&vec![]);
                                self.start_block(format!("{}_switch_not_met_case_{}", node.id(), case_op_index), &vec![]);
                            }

                            // emit default
                            self.process_dest(&ops, default, f_content, f_context, vm);

                            let default_target = f_content.get_block(default.target).name().unwrap();
                            self.backend.emit_b(default_target);
                        } else {
                            panic!("expecting cond in switch to be ireg: {}", cond);
                        }
                    }

                    Instruction_::ExprCall { ref data, is_abort } => {
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

                    Instruction_::Call { ref data, ref resume } => {
                        trace!("instsel on CALL");

                        self.emit_mu_call(
                            inst,
                            data,
                            Some(resume),
                            node,
                            f_content, f_context, vm);
                    },

                    Instruction_::ExprCCall { ref data, is_abort } => {
                        trace!("instsel on EXPRCCALL");

                        if is_abort {
                            unimplemented!()
                        }

                        self.emit_c_call_ir(inst, data, None, node, f_content, f_context, vm);
                    }

                    Instruction_::CCall { ref data, ref resume } => {
                        trace!("instsel on CCALL");

                        self.emit_c_call_ir(inst, data, Some(resume), node, f_content, f_context, vm);
                    }

                    Instruction_::Return(ref vals) => {
                        trace!("instsel on RETURN");

                        // prepare return regs
                        let ref ops = inst.ops.read().unwrap();
                        // TODO: Are vals in the same order as the return types in the functions signature?

                        let ret_tys = vals.iter().map(|i| node_type(&ops[*i])).collect();
                        let ret_type = self.combine_return_types(&ret_tys);

                        // Note: this shouldn't cause any overhead in the generated code if the register is never used
                        let temp_xr = make_temporary(f_context, ADDRESS_TYPE.clone(), vm);

                        if self.compute_return_allocation(&ret_type, &vm) > 0 {
                            // Load the saved value of XR into temp_xr
                            self.emit_load_base_offset(&temp_xr, &FP, -8, f_context, vm);
                        }

                        let n = ret_tys.len(); // number of return values
                        if n == 0 {
                            // Do nothing
                        } else if n == 1{
                            let ret_loc = self.compute_return_locations(&ret_type, &temp_xr, &vm);
                            self.emit_move_node_to_value(&ret_loc, &ops[vals[0]], f_content, f_context, vm);
                        } else {
                            let ret_loc = self.compute_return_locations(&ret_type, &temp_xr, &vm);

                            let mut i = 0;
                            for ret_index in vals {
                                let ret_val = self.emit_node_value(&ops[*ret_index], f_content, f_context, vm);
                                let ref ty = ret_val.ty;
                                let offset = self.get_field_offset(&ret_type, i, &vm);

                                match ty.v {
                                    MuType_::Vector(_, _) | MuType_::Tagref64 => unimplemented!(),
                                    MuType_::Void => panic!("Unexpected void"),
                                    MuType_::Struct(_) | MuType_::Array(_, _) => unimplemented!(),
                                    MuType_::Hybrid(_) => panic!("Can't return a hybrid"),
                                    // Integral, pointer or floating point type
                                    _ => self.insert_bytes(&ret_loc, &ret_val, offset as i64, f_context, vm),
                                }

                                i += 1;
                            }
                        }

                        self.backend.emit_b(EPILOGUE_BLOCK_NAME.to_string());
                    },

                    Instruction_::BinOp(op, op1, op2) => {
                        trace!("instsel on BINOP");
                        self.emit_binop(node, inst, op, BinOpStatus { flag_n: false, flag_z: false, flag_c: false, flag_v: false }, op1, op2, f_content, f_context, vm);
                    },

                    Instruction_::BinOpWithStatus(op, status, op1, op2) => {
                        trace!("instsel on BINOP_STATUS");
                        self.emit_binop(node, inst, op, status, op1, op2, f_content, f_context, vm);
                    }

                    Instruction_::ConvOp { operation, ref from_ty, ref to_ty, operand } => {
                        trace!("instsel on CONVOP");

                        let ops = inst.ops.read().unwrap();

                        let ref op = ops[operand];

                        let tmp_res = self.get_result_value(node, 0);
                        let tmp_op = self.emit_reg(op, f_content, f_context, vm);

                        let from_ty_size = get_bit_size(&from_ty, vm);
                        let to_ty_size = get_bit_size(&to_ty, vm);

                        match operation {
                            op::ConvOp::TRUNC => {
                                self.backend.emit_mov(&tmp_res, &cast_value(&tmp_op, &to_ty));
                            },
                            op::ConvOp::ZEXT => {
                                if from_ty_size != to_ty_size {
                                    self.backend.emit_ubfx(&tmp_res, &cast_value(&tmp_op, &to_ty), 0, from_ty_size as u8);
                                } else {
                                    self.backend.emit_mov(&tmp_res, &tmp_op);
                                }
                            },
                            op::ConvOp::SEXT => {
                                if from_ty_size != to_ty_size {
                                    self.backend.emit_sbfx(&tmp_res, &cast_value(&tmp_op, &to_ty), 0, from_ty_size as u8);
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

                        let resolved_loc = self.emit_node_addr_to_value(loc_op, f_content, f_context, vm);
                        let res = self.get_result_value(node, 0);

                        if use_acquire {
                            // Can only have a base for a LDAR
                            let temp_loc = self.emit_mem_base(&resolved_loc, f_context, vm);
                            match res.ty.v {
                                // Have to load a temporary GPR first
                                MuType_::Float => {
                                    let temp = make_temporary(f_context, UINT32_TYPE.clone(), vm);
                                    self.backend.emit_ldar(&temp, &temp_loc);
                                    self.backend.emit_fmov(&res, &temp);
                                }
                                MuType_::Double => {
                                    let temp = make_temporary(f_context, UINT64_TYPE.clone(), vm);
                                    self.backend.emit_ldar(&temp, &temp_loc);
                                    self.backend.emit_fmov(&res, &temp);
                                }
                                // Can load the register directly
                                _ =>  self.backend.emit_ldar(&res, &temp_loc)
                            };
                        } else {
                            let temp_loc = emit_mem(self.backend.as_mut(), &resolved_loc, f_context, vm);
                            self.backend.emit_ldr(&res, &temp_loc, false);
                        }
                    }

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

                        let resolved_loc = self.emit_node_addr_to_value(loc_op, f_content, f_context, vm);
                        let val = self.emit_reg(val_op, f_content, f_context, vm);

                        if use_release {
                            // Can only have a base for a STLR
                            let temp_loc = self.emit_mem_base(&resolved_loc, f_context, vm);

                            match val.ty.v {
                                // Have to store a temporary GPR
                                MuType_::Float => {
                                    let temp = make_temporary(f_context, UINT32_TYPE.clone(), vm);
                                    self.backend.emit_fmov(&temp, &val);
                                    self.backend.emit_stlr(&temp_loc, &temp);
                                }
                                MuType_::Double => {
                                    let temp = make_temporary(f_context, UINT64_TYPE.clone(), vm);
                                    self.backend.emit_fmov(&temp, &val);
                                    self.backend.emit_stlr(&temp_loc, &temp);
                                }
                                // Can load the register directly
                                _ => self.backend.emit_stlr(&temp_loc, &val)
                            };
                        } else {
                            let temp_loc = emit_mem(self.backend.as_mut(), &resolved_loc, f_context, vm);
                            self.backend.emit_str(&temp_loc, &val);
                        }
                    }

                    Instruction_::CmpXchg{is_ptr, is_weak, success_order, fail_order, mem_loc, expected_value, desired_value} => {
                        // Note: this uses the same operations as GCC (for the C++ atomic cmpxchg)
                        // Clang is slightly different and ignores the 'fail_order'
                        let use_acquire = match fail_order {
                            MemoryOrder::Acquire | MemoryOrder::SeqCst => true,
                            MemoryOrder::Relaxed => match success_order {
                                MemoryOrder::Acquire | MemoryOrder::AcqRel | MemoryOrder::SeqCst => true,
                                MemoryOrder::Relaxed | MemoryOrder::Release => false,
                                _ => panic!("didnt expect success order {:?} for cmpxchg", success_order)
                            },
                            _ => panic!("didnt expect fail order {:?} for cmpxchg", fail_order)
                        };
                        let use_release = match fail_order {
                            MemoryOrder::Acquire => match success_order {
                                MemoryOrder::Relaxed | MemoryOrder::Release | MemoryOrder::AcqRel | MemoryOrder::SeqCst => true,
                                MemoryOrder::Acquire => false,
                                _ => panic!("didnt expect success order {:?} for cmpxchg", success_order)
                            },
                            MemoryOrder::SeqCst => true,
                            MemoryOrder::Relaxed => match success_order {
                                MemoryOrder::Release | MemoryOrder::AcqRel | MemoryOrder::SeqCst => true,
                                MemoryOrder::Relaxed | MemoryOrder::Acquire => false,
                                _ => panic!("didnt expect success order {:?} for cmpxchg", success_order)
                            },
                            _ => panic!("didnt expect fail order {:?} for cmpxchg", fail_order)
                        };


                        let ops = inst.ops.read().unwrap();
                        let loc = self.emit_node_addr_to_value(&ops[mem_loc], f_content, f_context, vm);
                        let expected = self.emit_reg(&ops[expected_value], f_content, f_context, vm);
                        let desired = self.emit_reg(&ops[desired_value], f_content, f_context, vm);

                        let res_value = self.get_result_value(node, 0);
                        let res_success = self.get_result_value(node, 1);


                        let blk_cmpxchg_start = format!("{}_cmpxchg_start", node.id());
                        let blk_cmpxchg_failed = format!("{}_cmpxchg_failed", node.id());
                        let blk_cmpxchg_succeded = format!("{}_cmpxchg_succeded", node.id());

                        self.finish_block(&vec![loc.clone(),expected.clone(), desired.clone()]);

                        // cmpxchg_start:
                        self.start_block(blk_cmpxchg_start.clone(), &vec![loc.clone(),expected.clone(), desired.clone()]);

                        if use_acquire {
                            match res_value.ty.v {
                                // Have to load a temporary GPR first
                                MuType_::Float => {
                                    let temp = make_temporary(f_context, UINT32_TYPE.clone(), vm);
                                    self.backend.emit_ldaxr(&temp, &loc);
                                    self.backend.emit_fmov(&res_value, &temp);
                                }
                                MuType_::Double => {
                                    let temp = make_temporary(f_context, UINT64_TYPE.clone(), vm);
                                    self.backend.emit_ldaxr(&temp, &loc);
                                    self.backend.emit_fmov(&res_value, &temp);
                                }
                                // Can load the register directly
                                _ => self.backend.emit_ldaxr(&res_value, &loc)
                            };
                        } else {
                            match res_value.ty.v {
                                // Have to load a temporary GPR first
                                MuType_::Float => {
                                    let temp = make_temporary(f_context, UINT32_TYPE.clone(), vm);
                                    self.backend.emit_ldxr(&temp, &loc);
                                    self.backend.emit_fmov(&res_value, &temp);
                                }
                                MuType_::Double => {
                                    let temp = make_temporary(f_context, UINT64_TYPE.clone(), vm);
                                    self.backend.emit_ldxr(&temp, &loc);
                                    self.backend.emit_fmov(&res_value, &temp);
                                }
                                // Can load the register directly
                                _ => self.backend.emit_ldxr(&res_value, &loc)
                            };
                        }

                        if expected.is_int_reg() {
                            self.backend.emit_cmp(&res_value, &expected);
                        } else {
                            self.backend.emit_fcmp(&res_value, &expected);
                        }
                        self.backend.emit_b_cond("NE", blk_cmpxchg_failed.clone());

                        if use_release {
                            match desired.ty.v {
                                // Have to store a temporary GPR
                                MuType_::Float => {
                                    let temp = make_temporary(f_context, UINT32_TYPE.clone(), vm);
                                    self.backend.emit_fmov(&temp, &desired);
                                    self.backend.emit_stlxr(&loc, &res_success, &temp);
                                }
                                MuType_::Double => {
                                    let temp = make_temporary(f_context, UINT64_TYPE.clone(), vm);
                                    self.backend.emit_fmov(&temp, &desired);
                                    self.backend.emit_stlxr(&loc, &res_success, &temp);
                                }
                                // Can load the register directly
                                _ => self.backend.emit_stlxr(&loc, &res_success, &desired)
                            };
                        } else {
                            match desired.ty.v {
                                // Have to store a temporary GPR
                                MuType_::Float => {
                                    let temp = make_temporary(f_context, UINT32_TYPE.clone(), vm);
                                    self.backend.emit_fmov(&temp, &desired);
                                    self.backend.emit_stxr(&loc, &res_success, &temp);
                                }
                                MuType_::Double => {
                                    let temp = make_temporary(f_context, UINT64_TYPE.clone(), vm);
                                    self.backend.emit_fmov(&temp, &desired);
                                    self.backend.emit_stxr(&loc, &res_success, &temp);
                                }
                                // Can load the register directly
                                _ => self.backend.emit_stxr(&loc, &res_success, &desired)
                            };
                        }

                        if !is_weak {
                            // Store failed, try again
                            self.backend.emit_cbnz(&res_success, blk_cmpxchg_start.clone());
                        }

                        self.backend.emit_b(blk_cmpxchg_succeded.clone());

                        self.finish_block(&vec![res_success.clone(), res_value.clone()]);

                        // cmpxchg_failed:
                        self.start_block(blk_cmpxchg_failed.clone(), &vec![res_success.clone(), res_value.clone()]);

                        self.backend.emit_clrex();
                        // Set res_success to 1 (the same value STXR/STLXR uses to indicate failure)
                        self.backend.emit_mov_imm(&res_success, 1);

                        self.finish_block(&vec![res_success.clone(), res_value.clone()]);

                        // cmpxchg_succeded:
                        self.start_block(blk_cmpxchg_succeded.clone(), &vec![res_success.clone(), res_value.clone()]);
                        // this NOT is needed as STXR/STLXR returns sucess as '0', wheras the Mu spec says it should be 1
                        self.backend.emit_eor_imm(&res_success, &res_success, 1);
                    }
                    Instruction_::GetIRef(_)
                    | Instruction_::GetFieldIRef { .. }
                    | Instruction_::GetElementIRef{..}
                    | Instruction_::GetVarPartIRef { .. }
                    | Instruction_::ShiftIRef { .. } => {
                        trace!("instsel on GET/FIELD/VARPARTIREF, SHIFTIREF");
                        let mem_addr = self.emit_get_mem_from_inst(node, f_content, f_context, vm);
                        let tmp_res = self.get_result_value(node, 0);
                        self.emit_calculate_address(&tmp_res, &mem_addr, f_context, vm);
                    }

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

                    // TODO: Implement this similar to a return (where theres a common exit block)
                    // and change SWAP_BACK_TO_NATIV_STACK and swap_to_mu_stack so they don't handle the callee saved registers
                    // (this instruction should then guarentee that they are restored (in the same way a Return does)
                    Instruction_::ThreadExit => {
                        trace!("instsel on THREADEXIT");
                        // emit a call to swap_back_to_native_stack(sp_loc: Address)

                        // get thread local and add offset to get sp_loc
                        let tl = self.emit_get_threadlocal(Some(node), f_content, f_context, vm);
                        self.backend.emit_add_imm(&tl, &tl, *thread::NATIVE_SP_LOC_OFFSET as u16, false);

                        self.emit_runtime_entry(&entrypoints::SWAP_BACK_TO_NATIVE_STACK, vec![tl.clone()], None, Some(node), f_content, f_context, vm);
                    }


                    Instruction_::CommonInst_GetThreadLocal => {
                        trace!("instsel on GETTHREADLOCAL");
                        // get thread local
                        let tl = self.emit_get_threadlocal(Some(node), f_content, f_context, vm);

                        let tmp_res = self.get_result_value(node, 0);

                        // load [tl + USER_TLS_OFFSET] -> tmp_res
                        self.emit_load_base_offset(&tmp_res, &tl, *thread::USER_TLS_OFFSET as i64, f_context, vm);
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
                        self.emit_store_base_offset(&tl, *thread::USER_TLS_OFFSET as i64, &tmp_op, f_context, vm);
                    }

                    Instruction_::CommonInst_Pin(op) => {
                        trace!("instsel on PIN");
                        if !mm::GC_MOVES_OBJECT {
                            // non-moving GC: pin is a nop (move from op to result)
                            let ops = inst.ops.read().unwrap();
                            let ref op = ops[op];

                            let tmp_res = self.get_result_value(node, 0);

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

                        let tmp_res = self.get_result_value(node, 0);

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
                        let ty_align = ty_info.alignment;

                        let const_size = make_value_int_const(size as u64, vm);

                        let tmp_allocator = self.emit_get_allocator(node, f_content, f_context, vm);
                        let tmp_res = self.emit_alloc_sequence(tmp_allocator.clone(), const_size, ty_align, node, f_content, f_context, vm);

                        // ASM: call muentry_init_object(%allocator, %tmp_res, %encode)
                        let encode = make_value_int_const(mm::get_gc_type_encode(ty_info.gc_type.id), vm);
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

                            if match_node_int_imm(var_len) {
                                let var_len = node_imm_to_u64(var_len);
                                let actual_size = fix_part_size + var_ty_size * (var_len as usize);
                                (
                                    make_value_int_const(actual_size as u64, vm),
                                    make_value_int_const(var_len as u64, vm)
                                )
                            } else {
                                let tmp_actual_size = make_temporary(f_context, UINT64_TYPE.clone(), vm);
                                let tmp_var_len = self.emit_ireg(var_len, f_content, f_context, vm);

                                // tmp_actual_size = tmp_var_len*var_ty_size
                                emit_mul_u64(self.backend.as_mut(), &tmp_actual_size, &tmp_var_len, f_context, vm, var_ty_size as u64);
                                // tmp_actual_size = tmp_var_len*var_ty_size + fix_part_size
                                self.emit_add_u64(&tmp_actual_size, &tmp_actual_size, f_context, vm, fix_part_size as u64);
                                (tmp_actual_size, tmp_var_len)
                            }
                        };

                        let tmp_allocator = self.emit_get_allocator(node, f_content, f_context, vm);
                        let tmp_res = self.emit_alloc_sequence(tmp_allocator.clone(), actual_size, ty_align, node, f_content, f_context, vm);

                        // ASM: call muentry_init_object(%allocator, %tmp_res, %encode)
                        let encode = make_value_int_const(mm::get_gc_type_encode(ty_info.gc_type.id), vm);
                        self.emit_runtime_entry(
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

                        self.emit_runtime_entry(
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

                        self.emit_runtime_entry(
                            &entrypoints::PRINT_HEX,
                            vec![op.clone_value()],
                            None,
                            Some(node), f_content, f_context, vm
                        );
                    }

                    // Runtime Entry
                    Instruction_::SetRetval(index) => {
                        trace!("instsel on SETRETVAL");

                        let ref ops = inst.ops.read().unwrap();
                        let ref op  = ops[index];

                        self.emit_runtime_entry(
                            &entrypoints::SET_RETVAL,
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

    fn make_value_base_offset(&mut self, base: &P<Value>, offset: i64, ty: &P<MuType>, vm: &VM) -> P<Value> {
        let mem = self.make_memory_location_base_offset(base, offset, vm);
        self.make_value_from_memory(mem, ty, vm)
    }

    fn make_value_from_memory(&mut self, mem: MemoryLocation, ty: &P<MuType>, vm: &VM) -> P<Value> {
        P(Value {
            hdr: MuEntityHeader::unnamed(vm.next_id()),
            ty: ty.clone(),
            v: Value_::Memory(mem)
        })
    }

    fn make_memory_location_base_offset(&mut self, base: &P<Value>, offset: i64, vm: &VM) -> MemoryLocation {
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
                offset: Some(make_value_int_const(offset as u64, vm)),
                scale: 1,
                signed: true,
            }
        }
    }

    // Same as emit_mem except returns a memory location with only a base
    // NOTE: This code duplicates allot of code in emit_mem and emit_calculate_address
    fn emit_mem_base(&mut self, pv: &P<Value>, f_context: &mut FunctionContext, vm: &VM) -> P<Value> {
        match pv.v {
            Value_::Memory(ref mem) => {
                let base = match mem {
                    &MemoryLocation::VirtualAddress{ref base, ref offset, scale, signed} => {
                        if offset.is_some() {
                            let offset = offset.as_ref().unwrap();
                            if match_value_int_imm(offset) {
                                let offset_val = value_imm_to_i64(offset);
                                if offset_val == 0 {
                                    base.clone() // trivial
                                } else {
                                    let temp = make_temporary(f_context, pv.ty.clone(), vm);
                                    self.emit_add_u64(&temp, &base, f_context, vm, (offset_val * scale as i64) as u64);
                                    temp
                                }
                            } else {
                                let offset = emit_ireg_value(self.backend.as_mut(), offset, f_context, vm);

                                // TODO: If scale == r*m (for some 0 <= m <= 4), multiply offset by r
                                // then use and add_ext(,...,m)
                                if scale.is_power_of_two() && is_valid_immediate_extension(log2(scale)) {
                                    let temp = make_temporary(f_context, pv.ty.clone(), vm);
                                    // temp = base + offset << log2(scale)
                                    self.backend.emit_add_ext(&temp, &base, &offset, signed, log2(scale) as u8);
                                    temp
                                } else {
                                    let temp_offset = make_temporary(f_context, offset.ty.clone(), vm);

                                    // temp_offset = offset * scale
                                    emit_mul_u64(self.backend.as_mut(), &temp_offset, &offset, f_context, vm, scale);

                                    // Don't need to create a new register, just overwrite temp_offset
                                    let temp = cast_value(&temp_offset, &pv.ty);
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

                            if match_value_int_imm(&offset) {
                                let offset = value_imm_to_u64(&offset);
                                if offset == 0 {
                                    // Offset is 0, it can be ignored
                                    base.clone()
                                } else {
                                    let temp = make_temporary(f_context, pv.ty.clone(), vm);
                                    self.emit_add_u64(&temp, &base, f_context, vm, offset as u64);
                                    temp
                                }
                            } else if offset.is_int_reg() {
                                let temp = make_temporary(f_context, pv.ty.clone(), vm);
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
                    &MemoryLocation::Symbolic{ref label, is_global} => {
                        let temp = make_temporary(f_context, pv.ty.clone(), vm);
                        emit_addr_sym(self.backend.as_mut(), &temp, &pv, vm);
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

    fn make_memory_location_base_offset_scale(&mut self, base: &P<Value>, offset: &P<Value>, scale: u64, signed: bool) -> MemoryLocation {
        MemoryLocation::VirtualAddress{
            base: base.clone(),
            offset: Some(offset.clone()),
            scale: scale,
            signed: signed
        }
    }

    // Returns a memory location that points to 'Base + offset*scale + more_offset'
    fn memory_location_shift(&mut self, mem: MemoryLocation, more_offset: i64, f_context: &mut FunctionContext, vm: &VM) -> MemoryLocation {
        if more_offset == 0 {
            return mem; // No need to do anything
        }
        match mem {
            MemoryLocation::VirtualAddress { ref base, ref offset, scale, signed } => {
                let mut new_scale = 1;
                let new_offset =
                    if offset.is_some() {
                        let offset = offset.as_ref().unwrap();
                        if match_value_int_imm(&offset) {
                            let offset = offset.extract_int_const()*scale + (more_offset as u64);
                            make_value_int_const(offset as u64, vm)
                        } else {
                            let offset = emit_ireg_value(self.backend.as_mut(), &offset, f_context, vm);
                            let temp = make_temporary(f_context, offset.ty.clone(), vm);

                            if more_offset % (scale as i64) == 0 {
                                // temp = offset + more_offset/scale
                                self.emit_add_u64(&temp, &offset, f_context, vm, (more_offset/(scale as i64)) as u64);
                                new_scale = scale;
                            } else {
                                // temp = offset*scale + more_offset
                                emit_mul_u64(self.backend.as_mut(), &temp, &offset, f_context, vm, scale);
                                self.emit_add_u64(&temp, &temp, f_context, vm, more_offset as u64);
                            }

                            temp
                        }
                    }
                        else {
                            make_value_int_const(more_offset as u64, vm)
                        };

                // if offset was an immediate or more_offset % scale != 0:
                //      new_offset = offset*scale+more_offset
                //      new_scale = 1
                // otherwise:
                //      new_offset = offset + more_offset/scale
                //      new_scale = scale
                // Either way: (new_offset*new_scale) = offset*scale+more_offset
                MemoryLocation::VirtualAddress {
                    base: base.clone(),
                    offset: Some(new_offset),
                    scale: new_scale,
                    signed: signed,
                }
            },
            _ => panic!("expected a VirtualAddress memory location")
        }
    }

    // Returns a memory location that points to 'Base + offset*scale + more_offset*new_scale'
    fn memory_location_shift_scale(&mut self, mem: MemoryLocation, more_offset:  &P<Value>, new_scale: u64, f_context: &mut FunctionContext, vm: &VM) -> MemoryLocation {
        if match_value_int_imm(&more_offset) {
            let more_offset = value_imm_to_i64(&more_offset);
            self.memory_location_shift(mem, more_offset * (new_scale as i64), f_context, vm)
        } else {
            let mut new_scale = new_scale;
            match mem {
                MemoryLocation::VirtualAddress { ref base, ref offset, scale, signed } => {
                    let offset =
                        if offset.is_some() {
                            let offset = offset.as_ref().unwrap();
                            if match_value_int_imm(&offset) {
                                let temp = make_temporary(f_context, offset.ty.clone(), vm);
                                let offset_scaled = (offset.extract_int_const() as i64)*(scale as i64);
                                if offset_scaled % (new_scale as i64) == 0 {
                                    self.emit_add_u64(&temp, &more_offset, f_context, vm, (offset_scaled / (new_scale as i64)) as u64);
                                    // new_scale*temp = (more_offset + (offset*scale)/new_scale)
                                    //                = more_offset*new_scale + offset*scale
                                } else {
                                    // temp = more_offset*new_scale + offset*scale
                                    emit_mul_u64(self.backend.as_mut(), &temp, &more_offset, f_context, vm, new_scale);
                                    self.emit_add_u64(&temp, &temp, f_context, vm, offset_scaled as u64);
                                    new_scale = 1;
                                }
                                temp
                            } else {
                                let offset = emit_ireg_value(self.backend.as_mut(), &offset, f_context, vm);
                                let temp = make_temporary(f_context, offset.ty.clone(), vm);

                                if new_scale == scale {
                                    // just add the offsets
                                    self.backend.emit_add_ext(&temp, &more_offset, &temp, signed, 0);
                                }  else {
                                    // temp = offset * scale
                                    emit_mul_u64(self.backend.as_mut(), &temp, &offset, f_context, vm, scale);

                                    if new_scale.is_power_of_two() && is_valid_immediate_extension(log2(new_scale)) {
                                        // temp = (offset * scale) + more_offset << log2(new_scale)
                                        self.backend.emit_add_ext(&temp, &temp, &more_offset, signed, log2(new_scale) as u8);
                                    } else {
                                        // temp_more = more_offset * new_scale
                                        let temp_more = make_temporary(f_context, offset.ty.clone(), vm);
                                        emit_mul_u64(self.backend.as_mut(), &temp_more, &more_offset, f_context, vm, new_scale);

                                        // temp = (offset * scale) + (more_offset * new_scale);
                                        self.backend.emit_add_ext(&temp, &temp_more, &temp, signed, 0);
                                    }

                                    new_scale = 1;
                                }
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

    // Returns the size of the operation
    // TODO: If the RHS of an ADD is negative change it to a SUB (and vice versa)
    // TODO: Treat XOR 1....1, arg and XOR arg, 1....1 specially (1....1 is an invalid logical immediate, but the operation is non trivial so it should be optimised to res = MVN arg)
    // Note: Assume that trivial operations are to be optimised by the Mu IR compiler (but this function still needs to work correctly if they aren't optimsed away)
    // TODO: Use a shift when dividing or multiplying by a power of two
    fn emit_binop(&mut self, node: &TreeNode, inst: &Instruction, op: BinOp, status: BinOpStatus, op1: OpIndex, op2: OpIndex, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) {
        use std;
        let mut op1 = op1;
        let mut op2 = op2;
        let ops = inst.ops.read().unwrap();
        let res = self.get_result_value(node, 0);

        // Get the size (in bits) of the type the operation is on
        let n = get_bit_size(&res.ty, vm);
        let output_status = status.flag_n || status.flag_z || status.flag_c || status.flag_v;
        let mut status_value_index = 0;
        // NOTE: XZR is just a dummy value here (it will not be used)
        let tmp_status_n = if status.flag_n {
            status_value_index += 1;
            self.get_result_value(node, status_value_index)
        } else { XZR.clone() };
        let tmp_status_z = if status.flag_z {
            status_value_index += 1;
            self.get_result_value(node, status_value_index)
        } else { XZR.clone() };
        let tmp_status_c = if status.flag_c {
            status_value_index += 1;
            self.get_result_value(node, status_value_index)
        } else { XZR.clone() };
        let tmp_status_v = if status.flag_v {
            status_value_index += 1;
            self.get_result_value(node, status_value_index)
        } else { XZR.clone() };

        // TODO: Division by zero exception (note: must explicitly check for this, arm dosn't do it)
        match op {
            // The lower n bits of the result will be correct, and will not depend
            // on the > n bits of op1 or op2
            op::BinOp::Add => {
                let mut imm_val = 0 as u64;
                // Is one of the arguments a valid immediate?
                let emit_imm = if match_node_int_imm(&ops[op2]) {
                    imm_val = node_imm_to_u64(&ops[op2]);
                    is_valid_arithmetic_imm(imm_val)
                } else if match_node_int_imm(&ops[op1]) {
                    imm_val = node_imm_to_u64(&ops[op1]);
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

                    let reg_op1 = self.emit_ireg(&ops[op1], f_content, f_context, vm);
                    let imm_shift = imm_val > 4096;
                    let imm_op2 = if imm_shift { imm_val >> 12 } else { imm_val };

                    if output_status {
                        self.emit_zext(&reg_op1);
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

                    let reg_op1 = self.emit_ireg(&ops[op1], f_content, f_context, vm);
                    let reg_op2 = self.emit_ireg(&ops[op2], f_content, f_context, vm);

                    if output_status {
                        self.emit_zext(&reg_op1);
                        if n == 1 {
                            // adds_ext dosn't support extending 1 bit numbers
                            self.emit_zext(&reg_op2);
                            self.backend.emit_adds(&res, &reg_op1, &reg_op2);
                        } else {
                            // Emit an adds that zero extends op2
                            self.backend.emit_adds_ext(&res, &reg_op1, &reg_op2, false, 0);
                        }

                        if status.flag_v {
                            if n < 32 {
                                let tmp = make_temporary(f_context, UINT32_TYPE.clone(), vm);

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
                if match_node_int_imm(&ops[op2]) &&
                    is_valid_arithmetic_imm(node_imm_to_u64(&ops[op2])) &&

                    // If this was true, then the immediate would need to be 1 extended,
                    // which would result in an immediate with too many bits
                    !(status.flag_c && n < 32) {
                    // Can't compute the carry but using a subs_imm instruction
                    trace!("emit sub-ireg-imm");

                    let reg_op1 = self.emit_ireg(&ops[op1], f_content, f_context, vm);
                    let imm_val = node_imm_to_u64(&ops[op2]);
                    let imm_shift = imm_val > 4096;
                    let imm_op2 = if imm_shift { imm_val >> 12 } else { imm_val };

                    if output_status {
                        self.emit_zext(&reg_op1);
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

                    let reg_op1 = self.emit_ireg(&ops[op1], f_content, f_context, vm);
                    let reg_op2 = self.emit_ireg(&ops[op2], f_content, f_context, vm);

                    if output_status {
                        self.emit_zext(&reg_op1);

                        if status.flag_c {
                            // Note: reg_op2 is 'one'-extended so that SUB res, zext(reg_op1), oext(reg_op2)
                            // Is equivelent to: ADD res, zext(reg_op1), zext(~reg_op2), +1
                            // (this allows the carry flag to be computed as the 'n'th bit of res

                            self.emit_oext(&reg_op2);
                            self.backend.emit_subs(&res, &reg_op1, &reg_op2);
                        } else if n == 1 {
                            // if the carry flag isn't been computed, just zero extend op2
                            self.emit_zext(&reg_op2);
                            self.backend.emit_subs(&res, &reg_op1, &reg_op2);
                        } else {
                            // Emit an subs that zero extends op2
                            self.backend.emit_subs_ext(&res, &reg_op1, &reg_op2, false, 0);
                        }


                        if status.flag_v {
                            if n < 32 {
                                let tmp = make_temporary(f_context, UINT32_TYPE.clone(), vm);

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
                let emit_imm = if match_node_int_imm(&ops[op2]) {
                    imm_val = node_imm_to_u64(&ops[op2]);
                    is_valid_logical_imm(imm_val, n)
                } else if match_node_int_imm(&ops[op1]) {
                    imm_val = node_imm_to_u64(&ops[op1]);
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

                    let reg_op1 = self.emit_ireg(&ops[op1], f_content, f_context, vm);

                    if output_status {
                        self.backend.emit_ands_imm(&res, &reg_op1, replicate_logical_imm(imm_val, n));
                    } else {
                        self.backend.emit_and_imm(&res, &reg_op1, replicate_logical_imm(imm_val, n));
                    }
                } else {
                    trace!("emit and-ireg-ireg");

                    let reg_op1 = self.emit_ireg(&ops[op1], f_content, f_context, vm);
                    let reg_op2 = self.emit_ireg(&ops[op2], f_content, f_context, vm);

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
                let emit_imm = if match_node_int_imm(&ops[op2]) {
                    imm_val = node_imm_to_u64(&ops[op2]);
                    is_valid_logical_imm(imm_val, n)
                } else if match_node_int_imm(&ops[op1]) {
                    imm_val = node_imm_to_u64(&ops[op1]);
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

                    let reg_op1 = self.emit_ireg(&ops[op1], f_content, f_context, vm);

                    self.backend.emit_orr_imm(&res, &reg_op1, replicate_logical_imm(imm_val, n));
                } else {
                    trace!("emit or-ireg-ireg");

                    let reg_op1 = self.emit_ireg(&ops[op1], f_content, f_context, vm);
                    let reg_op2 = self.emit_ireg(&ops[op2], f_content, f_context, vm);

                    self.backend.emit_orr(&res, &reg_op1, &reg_op2);
                }
            },
            op::BinOp::Xor => {
                let mut imm_val = 0 as u64;
                // Is one of the arguments a valid immediate?
                let emit_imm = if match_node_int_imm(&ops[op2]) {
                    imm_val = node_imm_to_u64(&ops[op2]);
                    is_valid_logical_imm(imm_val, n)
                } else if match_node_int_imm(&ops[op1]) {
                    imm_val = node_imm_to_u64(&ops[op1]);
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

                    let reg_op1 = self.emit_ireg(&ops[op1], f_content, f_context, vm);

                    self.backend.emit_eor_imm(&res, &reg_op1, replicate_logical_imm(imm_val, n));
                } else {
                    trace!("emit xor-ireg-ireg");

                    let reg_op1 = self.emit_ireg(&ops[op1], f_content, f_context, vm);
                    let reg_op2 = self.emit_ireg(&ops[op2], f_content, f_context, vm);

                    self.backend.emit_eor(&res, &reg_op1, &reg_op2);
                }
            },

            op::BinOp::Mul => {
                trace!("emit mul-ireg-ireg");

                let reg_op1 = self.emit_ireg(&ops[op1], f_content, f_context, vm);
                let reg_op2 = self.emit_ireg(&ops[op2], f_content, f_context, vm);

                self.emit_zext(&reg_op1);
                self.emit_zext(&reg_op2);

                if status.flag_c || status.flag_v {
                    if n < 32 {
                        // A normal multiply will give the correct upper 'n' bits
                        self.backend.emit_mul(&res, &reg_op1, &reg_op2);
                        // Test the upper 'n' bits of the result
                        self.backend.emit_tst_imm(&res, (bits_ones(n) << n));
                    } else if n == 32 {
                        // the 64-bit register version of res
                        let res_64 = cast_value(&res, &UINT64_TYPE);
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
                    self.emit_sext(&res);
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

                let reg_op1 = self.emit_ireg(&ops[op1], f_content, f_context, vm);
                let reg_op2 = self.emit_ireg(&ops[op2], f_content, f_context, vm);

                // zero extend both arguments (in case they are less than 32 bits)
                self.emit_zext(&reg_op1);
                self.emit_zext(&reg_op2);
                self.backend.emit_udiv(&res, &reg_op1, &reg_op2);
            },
            op::BinOp::Sdiv => {
                trace!("emit mul-ireg-ireg");

                let reg_op1 = self.emit_ireg(&ops[op1], f_content, f_context, vm);
                let reg_op2 = self.emit_ireg(&ops[op2], f_content, f_context, vm);

                // sign extend both arguments (in case they are less than 32 bits)
                self.emit_sext(&reg_op1);
                self.emit_sext(&reg_op2);
                self.backend.emit_sdiv(&res, &reg_op1, &reg_op2);
            },
            op::BinOp::Urem => {
                trace!("emit mul-ireg-ireg");

                let reg_op1 = self.emit_ireg(&ops[op1], f_content, f_context, vm);
                let reg_op2 = self.emit_ireg(&ops[op2], f_content, f_context, vm);

                // zero extend both arguments (in case they are less than 32 bits)
                self.emit_zext(&reg_op1);
                self.emit_zext(&reg_op2);

                self.backend.emit_udiv(&res, &reg_op1, &reg_op2);
                // calculate the remained from the division
                self.backend.emit_msub(&res, &res, &reg_op2, &reg_op1);
            },
            op::BinOp::Srem => {
                trace!("emit mul-ireg-ireg");

                let reg_op1 = self.emit_ireg(&ops[op1], f_content, f_context, vm);
                let reg_op2 = self.emit_ireg(&ops[op2], f_content, f_context, vm);

                // sign extend both arguments (in case they are less than 32 bits)
                self.emit_sext(&reg_op1);
                self.emit_sext(&reg_op2);
                self.backend.emit_sdiv(&res, &reg_op1, &reg_op2);
                self.backend.emit_msub(&res, &res, &reg_op2, &reg_op1);
            },

            op::BinOp::Shl => {
                if match_node_int_imm(&ops[op2]) {
                    trace!("emit shl-ireg-imm");

                    let reg_op1 = self.emit_ireg(&ops[op1], f_content, f_context, vm);
                    let imm_op2 = node_imm_to_u64(&ops[op2]) %
                        (res.ty.get_int_length().unwrap() as u64);

                    self.backend.emit_lsl_imm(&res, &reg_op1, imm_op2 as u8);
                } else {
                    trace!("emit shl-ireg-ireg");

                    let reg_op1 = self.emit_ireg(&ops[op1], f_content, f_context, vm);
                    let reg_op2 = self.emit_ireg(&ops[op2], f_content, f_context, vm);

                    // Will be reg_op1, or reg_op2
                    let reg_op2_use = self.emit_shift_mask(&reg_op1, &reg_op2);
                    self.backend.emit_lsl(&res, &reg_op1, &reg_op2_use);
                }
            },
            op::BinOp::Lshr => {
                if match_node_int_imm(&ops[op2]) {
                    trace!("emit lshr-ireg-imm");

                    let reg_op1 = self.emit_ireg(&ops[op1], f_content, f_context, vm);
                    let imm_op2 = node_imm_to_u64(&ops[op2]) %
                        (res.ty.get_int_length().unwrap() as u64);

                    self.backend.emit_lsr_imm(&res, &reg_op1, imm_op2 as u8);
                } else {
                    trace!("emit lshr-ireg-ireg");

                    let reg_op1 = self.emit_ireg(&ops[op1], f_content, f_context, vm);
                    let reg_op2 = self.emit_ireg(&ops[op2], f_content, f_context, vm);

                    // Will be reg_op1, or reg_op2
                    let reg_op2_use = self.emit_shift_mask(&reg_op1, &reg_op2);
                    self.backend.emit_lsr(&res, &reg_op1, &reg_op2_use);
                }
            },
            op::BinOp::Ashr => {
                if match_node_int_imm(&ops[op2]) {
                    trace!("emit ashr-ireg-imm");

                    let reg_op1 = self.emit_ireg(&ops[op1], f_content, f_context, vm);
                    let imm_op2 = node_imm_to_u64(&ops[op2]) %
                        (res.ty.get_int_length().unwrap() as u64);

                    self.backend.emit_asr_imm(&res, &reg_op1, imm_op2 as u8);
                } else {
                    trace!("emit ashr-ireg-ireg");

                    let reg_op1 = self.emit_ireg(&ops[op1], f_content, f_context, vm);
                    let reg_op2 = self.emit_ireg(&ops[op2], f_content, f_context, vm);

                    // Will be reg_op1, or reg_op2
                    let reg_op2_use = self.emit_shift_mask(&reg_op1, &reg_op2);
                    self.backend.emit_asr(&res, &reg_op1, &reg_op2_use);
                }
            },

            // floating point
            op::BinOp::FAdd => {
                trace!("emit add-fpreg-fpreg");

                let reg_op1 = self.emit_fpreg(&ops[op1], f_content, f_context, vm);
                let reg_op2 = self.emit_fpreg(&ops[op2], f_content, f_context, vm);

                self.backend.emit_fadd(&res, &reg_op1, &reg_op2);
            }
            op::BinOp::FSub => {
                trace!("emit sub-fpreg-fpreg");

                let reg_op1 = self.emit_fpreg(&ops[op1], f_content, f_context, vm);
                let reg_op2 = self.emit_fpreg(&ops[op2], f_content, f_context, vm);

                self.backend.emit_fsub(&res, &reg_op1, &reg_op2);
            }
            op::BinOp::FMul => {
                trace!("emit mul-fpreg-fpreg");

                let reg_op1 = self.emit_fpreg(&ops[op1], f_content, f_context, vm);
                let reg_op2 = self.emit_fpreg(&ops[op2], f_content, f_context, vm);

                self.backend.emit_fmul(&res, &reg_op1, &reg_op2);
            }
            op::BinOp::FDiv => {
                trace!("emit div-fpreg-fpreg");

                let reg_op1 = self.emit_fpreg(&ops[op1], f_content, f_context, vm);
                let reg_op2 = self.emit_fpreg(&ops[op2], f_content, f_context, vm);

                self.backend.emit_fdiv(&res, &reg_op1, &reg_op2);
            }
            op::BinOp::FRem => {
                trace!("emit rem-fpreg-fpreg");

                let reg_op1 = self.emit_fpreg(&ops[op1], f_content, f_context, vm);
                let reg_op2 = self.emit_fpreg(&ops[op2], f_content, f_context, vm);

                // TODO: Directly call the c functions fmodf and fmod (repsectivlly)
                if n == 32 {
                    self.emit_runtime_entry(&entrypoints::FREM32, vec![reg_op1.clone(), reg_op2.clone()], Some(vec![res.clone()]), Some(node), f_content, f_context, vm);
                } else {
                    self.emit_runtime_entry(&entrypoints::FREM64, vec![reg_op1.clone(), reg_op2.clone()], Some(vec![res.clone()]), Some(node), f_content, f_context, vm);
                }
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
                    self.emit_sext(&res);
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
                    self.emit_sext(&res);
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

    fn emit_alloc_sequence(&mut self, tmp_allocator: P<Value>, size: P<Value>, align: usize, node: &TreeNode, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) -> P<Value> {
        if size.is_int_const() {
            // size known at compile time, we can choose to emit alloc_small or large now
            let size_i = size.extract_int_const();

            if size_i + OBJECT_HEADER_SIZE as u64 > mm::LARGE_OBJECT_THRESHOLD as u64 {
                self.emit_alloc_sequence_large(tmp_allocator, size, align, node, f_content, f_context, vm)
            } else {
                self.emit_alloc_sequence_small(tmp_allocator, size, align, node, f_content, f_context, vm)
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
                let size_with_hdr = make_temporary(f_context, UINT64_TYPE.clone(), vm);
                self.emit_add_u64(&size_with_hdr, &size, f_context, vm, OBJECT_HEADER_SIZE as u64);
                self.emit_cmp_u64(&size_with_hdr, f_context, vm, mm::LARGE_OBJECT_THRESHOLD as u64);
            } else {
                self.emit_cmp_u64(&size, f_context, vm, mm::LARGE_OBJECT_THRESHOLD as u64);
            }
            self.backend.emit_b_cond("GT", blk_alloc_large.clone());
            self.finish_block(&vec![]);

            self.start_block(format!("{}_allocsmall", node.id()), &vec![]);
            let tmp_res = self.emit_alloc_sequence_small(tmp_allocator.clone(), size.clone(), align, node, f_content, f_context, vm);
            self.backend.emit_b(blk_alloc_large_end.clone());

            self.finish_block(&vec![tmp_res.clone()]);

            // alloc_large:
            self.start_block(blk_alloc_large.clone(), &vec![size.clone()]);
            let tmp_res = self.emit_alloc_sequence_large(tmp_allocator.clone(), size, align, node, f_content, f_context, vm);
            self.finish_block(&vec![tmp_res.clone()]);

            // alloc_large_end:
            self.start_block(blk_alloc_large_end.clone(), &vec![]);

            tmp_res
        }
    }

    fn emit_get_allocator(&mut self, node: &TreeNode, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) -> P<Value> {
        // ASM: %tl = get_thread_local()
        let tmp_tl = self.emit_get_threadlocal(Some(node), f_content, f_context, vm);

        // ASM: lea [%tl + allocator_offset] -> %tmp_allocator
        let allocator_offset = *thread::ALLOCATOR_OFFSET;
        let tmp_allocator = make_temporary(f_context, ADDRESS_TYPE.clone(), vm);
        self.emit_add_u64(&tmp_allocator, &tmp_tl, f_context, vm, allocator_offset as u64);
        tmp_allocator
    }

    fn emit_alloc_sequence_large(&mut self, tmp_allocator: P<Value>, size: P<Value>, align: usize, node: &TreeNode, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) -> P<Value> {
        let tmp_res = self.get_result_value(node, 0);

        // ASM: %tmp_res = call muentry_alloc_large(%allocator, size, align)
        let const_align = make_value_int_const(align as u64, vm);

        self.emit_runtime_entry(
            &entrypoints::ALLOC_LARGE,
            vec![tmp_allocator.clone(), size.clone(), const_align],
            Some(vec![tmp_res.clone()]),
            Some(node), f_content, f_context, vm
        );

        tmp_res
    }

    fn emit_alloc_sequence_small(&mut self, tmp_allocator: P<Value>, size: P<Value>, align: usize, node: &TreeNode, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) -> P<Value> {
        if INLINE_FASTPATH {
            unimplemented!(); // (inline the generated code in alloc() in immix_mutator.rs??)
        } else {
            // directly call 'alloc'
            let tmp_res = self.get_result_value(node, 0);

            let const_align = make_value_int_const(align as u64, vm);

            self.emit_runtime_entry(
                &entrypoints::ALLOC_FAST,
                vec![tmp_allocator.clone(), size.clone(), const_align],
                Some(vec![tmp_res.clone()]),
                Some(node), f_content, f_context, vm
            );

            tmp_res
        }
    }

    fn emit_load_base_offset(&mut self, dest: &P<Value>, base: &P<Value>, offset: i64, f_context: &mut FunctionContext, vm: &VM) -> P<Value> {
        let mem = self.make_value_base_offset(base, offset, &dest.ty, vm);
        let mem = emit_mem(self.backend.as_mut(), &mem, f_context, vm);
        self.backend.emit_ldr(dest, &mem, false);
        mem
    }

    fn emit_store_base_offset(&mut self, base: &P<Value>, offset: i64, src: &P<Value>, f_context: &mut FunctionContext, vm: &VM) {
        let mem = self.make_value_base_offset(base, offset, &src.ty, vm);
        let mem = emit_mem(self.backend.as_mut(), &mem, f_context, vm);
        self.backend.emit_str(&mem, src);
    }

    // TODO: Inline this function call (it's like 4 lines of assembly...)
    fn emit_get_threadlocal(
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
    fn emit_runtime_entry(
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


    // Note: if tys has more than 1 element, then this will return a new struct type
    // , but each call will generate a different name for this struct type (but the layout will be identical)
    fn combine_return_types(&self, tys: &Vec<P<MuType>>) -> P<MuType>{
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
    fn compute_return_allocation(&self, t: &P<MuType>, vm: &VM) -> usize
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

    // Returns a list of registers used for return values (used to set the 'livein' for the epilogue blokc)
    fn compute_return_registers(&mut self, t: &P<MuType>, vm: &VM) -> Vec<P<Value>>
    {
        use ast::types::MuType_::*;
        let size = round_up(vm.get_type_size(t.id()), 8);
        match t.v {
            Vector(_, _) | Tagref64 => unimplemented!(),
            Float | Double =>
                vec![get_alias_for_length(RETURN_FPRs[0].id(), get_bit_size(&t, vm))],

            Hybrid(_) => panic!("cant return a hybrid"),
            Struct(_) | Array(_, _) => {
                let hfa_n = hfa_length(t.clone());
                if hfa_n > 0 {
                    let mut res = vec![get_alias_for_length(RETURN_FPRs[0].id(), get_bit_size(&t, vm)/hfa_n)];
                    for i in 1..hfa_n {
                        res.push(get_alias_for_length(RETURN_FPRs[0].id(), get_bit_size(&t, vm)/hfa_n));
                    }
                    res
                } else if size <= 8 {
                    // Return in a single GRP
                    vec![get_alias_for_length(RETURN_GPRs[0].id(), get_bit_size(&t, vm))]
                } else if size <= 16 {
                    // Return in 2 GPRs
                    vec![RETURN_GPRs[0].clone(), RETURN_GPRs[0].clone()]

                } else {
                    // Returned on the stack
                    vec![]
                }
            }

            Void => vec![], // Nothing to return
            // Integral or pointer type
            Int(_) | Ref(_) | IRef(_) | WeakRef(_) | UPtr(_) | ThreadRef | StackRef | FuncRef(_) | UFuncPtr(_) =>
            // can return in GPR
                vec![get_alias_for_length(RETURN_GPRs[0].id(), get_bit_size(&t, vm))]
        }
    }

    fn compute_return_locations(&mut self, t: &P<MuType>, loc: &P<Value>, vm: &VM) -> P<Value>
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
                } else if size <= 8 {
                    // Return in a singe GRPs
                    get_alias_for_length(RETURN_GPRs[0].id(), get_bit_size(t, vm))
                } else if size <= 16 {
                    // Return in 2 GPRS
                    RETURN_GPRs[0].clone()
                } else {
                    // Return at the location pointed to by loc
                    self.make_value_base_offset(&loc, 0, &t, vm)
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
    fn compute_argument_locations(&mut self, arg_types: &Vec<P<MuType>>, stack: &P<Value>, offset: i64, vm: &VM) -> (Vec<bool>, Vec<P<Value>>, usize) {
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
                Hybrid(_) => panic!("hybrid argument not supported"),

                Vector(_, _) | Tagref64 => unimplemented!(),
                Float | Double => {
                    if nsrn < 8 {
                        locations.push(get_alias_for_length(ARGUMENT_FPRs[nsrn].id(), get_bit_size(&t, vm)));
                        nsrn += 1;
                    } else {
                        nsrn = 8;
                        locations.push(self.make_value_base_offset(&stack, offset + (nsaa as i64), &t, vm));
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
                            locations.push(self.make_value_base_offset(&stack, offset + (nsaa as i64), &t, vm));
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
                            locations.push(self.make_value_base_offset(&stack, offset + (nsaa as i64) as i64, &t, vm));
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
                            locations.push(self.make_value_base_offset(&stack, offset + (nsaa as i64) as i64, &t, vm));
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


    // returns the stack arg offset - we will need this to collapse stack after the call
    fn emit_precall_convention(&mut self, args: &Vec<P<Value>>, arg_tys: &Vec<P<MuType>>, return_size: usize, f_context: &mut FunctionContext, vm: &VM) -> usize
    {
        //sig.ret_tys
        let (is_iref, locations, stack_size) = self.compute_argument_locations(&arg_tys, &SP, 0, &vm);

        if return_size > 0 {
            // Reserve space on the stack for the return value
            self.emit_sub_u64(&SP, &SP, f_context, &vm, return_size as u64);

            // XR needs to point to where the callee should return arguments
            self.backend.emit_mov(&XR, &SP);
        }
        // Reserve space on the stack for all stack arguments
        self.emit_sub_u64(&SP, &SP, f_context, &vm, stack_size as u64);

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
                _ => self.emit_move_value_to_value(&arg_loc, &arg_val, f_context, vm)
            }
        }

        stack_size
    }

    fn emit_postcall_convention(&mut self, ret_tys: &Vec<P<MuType>>, rets: &Option<Vec<P<Value>>>, ret_type: &P<MuType>, arg_size: usize, ret_size: usize, f_context: &mut FunctionContext, vm: &VM) -> Vec<P<Value>> {
        // deal with ret vals
        let mut return_vals = vec![];

        self.emit_add_u64(&SP, &SP, f_context, &vm, arg_size as u64);

        let n = ret_tys.len(); // number of return values
        if n == 0 {
            // Do nothing
        } else if n == 1{
            let ret_loc = self.compute_return_locations(&ret_type, &SP, &vm);

            let ref ty = ret_tys[0];
            let ret_val = match rets {
                &Some(ref rets) => rets[0].clone(),
                &None => {
                    let tmp_node = f_context.make_temporary(vm.next_id(), ty.clone());
                    tmp_node.clone_value()
                }
            };

            self.emit_move_value_to_value(&ret_val, &ret_loc, f_context, vm);
            return_vals.push(ret_val);
        } else {
            let ret_loc = self.compute_return_locations(&ret_type, &SP, &vm);

            for ret_index in 0..ret_tys.len() {
                let ref ty = ret_tys[ret_index];
                let offset = self.get_field_offset(ret_type, ret_index, &vm);
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
                    _ => self.extract_bytes(&ret_val, &ret_loc, offset as i64, f_context, vm),
                }
                return_vals.push(ret_val);
            }
        }

        // We have now read the return values, and can free space from the stack
        self.emit_add_u64(&SP, &SP, f_context, &vm, ret_size as u64);

        return_vals
    }

    // Copies src to dest+off, dest can be a memory location or a machine register
    // (in the case of a machine register, sucessivie registers of the same size are considered
    // part of dest).
    // WARNING: It is assumed that dest and src do not overlap
    fn insert_bytes(&mut self, dest: &P<Value>, src: &P<Value>, offset: i64, f_context: &mut FunctionContext, vm: &VM)
    {
        if dest.is_mem() {
            let dest_loc = match dest.v {
                Value_::Memory(ref mem) => {
                    let mem = self.memory_location_shift(mem.clone(), offset, f_context, vm);
                    self.make_value_from_memory(mem, &dest.ty, vm)
                },
                _ => panic!("Wrong kind of memory value"),
            };
            self.emit_move_value_to_value(&dest_loc, &src, f_context, vm);
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
                    self.emit_move_value_to_value(&dest_reg, &src, f_context, vm);
                } else {
                    let tmp_src = if src.is_int_reg() { src.clone() } else { make_temporary(f_context, src.ty.clone(), vm) };

                    if !src.is_int_reg() {
                        // A temporary is being used, move src to it
                        self.emit_move_value_to_value(&tmp_src, &src, f_context, vm);
                    }

                    if dest_reg.is_int_reg() {
                        // Copy to dest_reg, 'src_size' bits starting at 'reg_offset' in src
                        // (leaving other bits unchanged)
                        self.backend.emit_bfi(&dest_reg, &tmp_src, reg_offset as u8, src_size as u8);
                    } else {
                        // floating point register, need to move dest to an int register first
                        let tmp_dest = make_temporary(f_context, tmp_src.ty.clone(), vm);
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
    fn extract_bytes(&mut self, dest: &P<Value>, src: &P<Value>, offset: i64, f_context: &mut FunctionContext, vm: &VM)
    {
        if src.is_mem() {
            let src_loc = match src.v {

                Value_::Memory(ref mem) => {
                    let mem = self.memory_location_shift(mem.clone(), offset, f_context, vm);
                    self.make_value_from_memory(mem, &src.ty, vm)
                },
                _ => panic!("Wrong kind of memory value"),
            };
            // TODO: what if 'dest is in more than 1 register
            self.emit_move_value_to_value(&dest, &src_loc, f_context, vm);
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
                    self.emit_move_value_to_value(&dest, &src_reg, f_context, vm);
                } else {
                    let tmp_dest = if dest.is_int_reg() { dest.clone() } else { make_temporary(f_context, dest.ty.clone(), vm) };

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
                        self.emit_move_value_to_value(&dest, &tmp_dest, f_context, vm);
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
    fn  emit_c_call_internal(
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
        let return_type = self.combine_return_types(&sig.ret_tys);
        let return_size = self.compute_return_allocation(&return_type, &vm);
        let stack_arg_size = self.emit_precall_convention(&args, &sig.arg_tys, return_size, f_context, vm);

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

        self.emit_postcall_convention(&sig.ret_tys, &rets, &return_type, stack_arg_size, return_size, f_context, vm)
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

            if match_node_imm(arg) {
                let arg = node_imm_to_value(arg);
                arg_values.push(arg);
            } else if self.match_reg(arg) {
                let arg = self.emit_reg(arg, f_content, f_context, vm);
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
                            self.emit_c_call_internal(
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

            if match_node_imm(arg) {
                let arg = node_imm_to_value(arg);
                arg_values.push(arg);
            } else if self.match_reg(arg) {
                let arg = self.emit_reg(arg, f_content, f_context, vm);
                arg_values.push(arg);
            } else {
                unimplemented!();
            }
        }
        let return_type = self.combine_return_types(&func_sig.ret_tys);
        let return_size = self.compute_return_allocation(&return_type, &vm);
        let stack_arg_size = self.emit_precall_convention(&arg_values, &func_sig.arg_tys, return_size, f_context, vm);

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
                let target = self.emit_ireg(func, f_content, f_context, vm);

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
        self.emit_postcall_convention(&func_sig.ret_tys, &inst.value, &return_type, stack_arg_size, return_size, f_context, vm);
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

    fn emit_common_prologue(&mut self, args: &Vec<P<Value>>, sig: &P<CFuncSig>, f_context: &mut FunctionContext, vm: &VM) {
        // no livein
        self.start_block(PROLOGUE_BLOCK_NAME.to_string(), &vec![]);

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

        // reserve spaces for current frame
        self.backend.emit_frame_grow(); // will include space for callee saved registers

        // We need to return arguments in the memory area pointed to by XR, so we need to save it
        let ret_ty = self.combine_return_types(&sig.ret_tys);
        if self.compute_return_allocation(&ret_ty, &vm) > 0 {
            trace!("allocate frame slot for {}", XR.clone());
            let loc = self.current_frame.as_mut().unwrap().alloc_slot_for_callee_saved_reg(XR.clone(), vm);
            let loc = emit_mem(self.backend.as_mut(), &loc, f_context, vm);
            self.backend.emit_str(&loc, &XR);
        }

        // push all callee-saved registers
        for i in 0..CALLEE_SAVED_FPRs.len() {
            let ref reg = CALLEE_SAVED_FPRs[i];

            trace!("allocate frame slot for reg {}", reg);
            let loc = self.current_frame.as_mut().unwrap().alloc_slot_for_callee_saved_reg(reg.clone(), vm);
            let loc = emit_mem(self.backend.as_mut(), &loc, f_context, vm);
            self.backend.emit_str_callee_saved(&loc, &reg);
        }
        for i in 0..CALLEE_SAVED_GPRs.len() {
            let ref reg = CALLEE_SAVED_GPRs[i];
            trace!("allocate frame slot for regs {}", reg);

            let loc = self.current_frame.as_mut().unwrap().alloc_slot_for_callee_saved_reg(reg.clone(), vm);
            let loc = emit_mem(self.backend.as_mut(), &loc, f_context, vm);
            self.backend.emit_str_callee_saved(&loc, &reg);
        }

        // unload arguments
        // Read arguments starting from FP+16 (FP points to the frame record (the previouse FP and LR)
        let (is_iref, locations, stack_size) = self.compute_argument_locations(&sig.arg_tys, &FP, 16, &vm);

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
                        self.emit_load(&arg_val, &arg_loc, f_context, vm);
                        self.current_frame.as_mut().unwrap().add_argument_by_stack(arg_val.id(), arg_loc.clone());
                    }
                }
            }
        }

        // liveout = entry block's args
        self.finish_block(args);
    }

    // Todo: Don't emit this if the function never returns
    fn emit_common_epilogue(&mut self, ret_tys: &Vec<P<MuType>>, f_context: &mut FunctionContext, vm: &VM) {
        let ret_type = self.combine_return_types(&ret_tys);

        // Live in are the registers that hold the return values
        // (if the value is returned through 'XR' than the caller is responsible for managing lifetime)
        let livein = self.compute_return_registers(&ret_type, vm);
        self.start_block(EPILOGUE_BLOCK_NAME.to_string(), &livein);

        // pop all callee-saved registers
        for i in (0..CALLEE_SAVED_GPRs.len()).rev() {
            let ref reg = CALLEE_SAVED_GPRs[i];
            let reg_id = reg.extract_ssa_id().unwrap();
            let loc = self.current_frame.as_mut().unwrap().allocated.get(&reg_id).unwrap().make_memory_op(reg.ty.clone(), vm);
            let loc = emit_mem(self.backend.as_mut(), &loc, f_context, vm);
            self.backend.emit_ldr_callee_saved(reg, &loc);
        }
        for i in (0..CALLEE_SAVED_FPRs.len()).rev() {
            let ref reg = CALLEE_SAVED_FPRs[i];

            let reg_id = reg.extract_ssa_id().unwrap();
            let loc = self.current_frame.as_mut().unwrap().allocated.get(&reg_id).unwrap().make_memory_op(reg.ty.clone(), vm);
            let loc = emit_mem(self.backend.as_mut(), &loc, f_context, vm);
            self.backend.emit_ldr_callee_saved(reg, &loc);
        }

        // frame shrink
        self.backend.emit_frame_shrink();

        // Pop the link register and frame pointers
        self.backend.emit_pop_pair(&FP, &LR, &SP);

        // Note: the stack pointer should now be what it was when the function was called
        self.backend.emit_ret(&LR); // return to the Link Register

        self.finish_block(&vec![]); // No live out
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
                        self.emit_cmp_res_op(op, op1, op2, f_content, f_context, vm)
                    }
                    _ => panic!("expect cmp res to emit")
                }
            }
            _ => panic!("expect cmp res to emit")
        }
    }

    fn emit_calculate_address(&mut self, dest: &P<Value>, src: &P<Value>, f_context: &mut FunctionContext, vm: &VM) {
        let src = emit_mem(self.backend.as_mut(), &src, f_context, vm);
        match src.v {
            // offset(base,index,scale)
            Value_::Memory(MemoryLocation::Address{ref base, ref offset, shift, signed}) => {
                if offset.is_some() {
                    let ref offset = offset.as_ref().unwrap();

                    if match_value_int_imm(&offset) {
                        let offset = value_imm_to_u64(&offset);
                        if offset == 0 {
                            // Offset is 0, address calculation is trivial
                            self.backend.emit_mov(&dest, &base);
                        } else {
                            self.emit_add_u64(&dest, &base, f_context, vm, offset as u64);
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

            Value_::Memory(MemoryLocation::Symbolic{ref label, is_global}) => {
                emit_addr_sym(self.backend.as_mut(), &dest, &src, vm);
            }
            _ => panic!("expect mem location as value")
        }
    }
    // TODO: Check ZEXT and SEXT are happening when they should
    fn emit_cmp_res_op(&mut self, op: CmpOp, op1: &P<TreeNode>, op2: &P<TreeNode>, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) -> op::CmpOp {
        let mut op1 = op1;
        let mut op2 = op2;
        let mut op = op;
        if op == CmpOp::FFALSE || op == CmpOp::FTRUE {
            return op; // No comparison needed
        }
        use std;
        let mut swap = false; // Whether op1 and op2 have been swapped
        if op::is_int_cmp(op) {
            let n = node_type(op1).get_int_length().unwrap();

            let mut imm_val = 0 as u64;
            // Is one of the arguments a valid immediate?
            let emit_imm = if match_node_int_imm(&op2) {
                imm_val = node_imm_to_u64(&op2);
                if op.is_signed() {
                    imm_val = get_signed_value(imm_val, n) as u64;
                }
                is_valid_arithmetic_imm(imm_val)
            } else if match_node_int_imm(&op1) {
                imm_val = node_imm_to_u64(&op1);
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
                let reg_op1 = self.emit_ireg(op1, f_content, f_context, vm);
                let imm_shift = imm_val > 4096;
                let imm_op2 = if imm_shift { imm_val >> 12 } else { imm_val };

                if op.is_signed() {
                    self.emit_sext(&reg_op1);
                } else {
                    self.emit_zext(&reg_op1);
                }

                self.backend.emit_cmp_imm(&reg_op1, imm_op2 as u16, imm_shift);
            } else {
                let reg_op1 = self.emit_ireg(op1, f_content, f_context, vm);
                let reg_op2 = self.emit_ireg(op2, f_content, f_context, vm);

                if op.is_signed() {
                    self.emit_sext(&reg_op1);
                    self.emit_sext(&reg_op2);
                } else {
                    self.emit_zext(&reg_op1);
                    self.emit_zext(&reg_op2);
                }
                self.backend.emit_cmp(&reg_op1, &reg_op2);
            }

            return op;
        } else {
            // Is one of the arguments 0
            let emit_imm = if match_f32imm(&op2) {
                node_imm_to_f32(&op2) == 0.0
            } else if match_f32imm(&op1) {
                if node_imm_to_f32(&op1) == 0.0 {
                    std::mem::swap(&mut op1, &mut op2);
                    swap = true;
                    true
                } else {
                    false
                }
            } else if match_f64imm(&op2) {
                node_imm_to_f64(&op2) == 0.0
            } else if match_f64imm(&op1) {
                if node_imm_to_f64(&op1) == 0.0 {
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
                let reg_op1 = self.emit_fpreg(op1, f_content, f_context, vm);
                self.backend.emit_fcmp_0(&reg_op1);
            } else {
                let reg_op1 = self.emit_fpreg(op1, f_content, f_context, vm);
                let reg_op2 = self.emit_fpreg(op2, f_content, f_context, vm);

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

    fn match_reg(&mut self, op: &TreeNode) -> bool {
        match op.v {
            TreeNode_::Instruction(ref inst) => {
                if inst.value.is_some() {
                    if inst.value.as_ref().unwrap().len() > 1 {
                        return false;
                    }

                    let ref value = inst.value.as_ref().unwrap()[0];

                    if value.is_int_reg() || value.is_fp_reg() {
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }

            TreeNode_::Value(ref pv) => {
                pv.is_int_reg() || pv.is_int_const() || pv.is_fp_reg()
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
    fn emit_sub_u64(&mut self, dest: &P<Value>, src: &P<Value>, f_context: &mut FunctionContext, vm: &VM, val: u64)
    {
        if (val as i64) < 0 {
            self.emit_add_u64(&dest, &src, f_context, vm, (-(val as i64) as u64));
        } else if val == 0 {
            if dest.id() != src.id() {
                self.backend.emit_mov(&dest, &src);
            }
        } else if is_valid_arithmetic_imm(val) {
            let imm_shift = val > 4096;
            let imm_val = if imm_shift { val >> 12 } else { val };
            self.backend.emit_sub_imm(&dest, &src, imm_val as u16, imm_shift);
        } else {
            let tmp = make_temporary(f_context, UINT64_TYPE.clone(), vm);
            emit_mov_u64(self.backend.as_mut(), &tmp, val);
            self.backend.emit_sub(&dest, &src, &tmp);
        }
    }

    // Increment the register by an immediate value
    fn emit_add_u64(&mut self, dest: &P<Value>, src: &P<Value>, f_context: &mut FunctionContext, vm: &VM, val: u64)
    {
        if (val as i64) < 0 {
            self.emit_sub_u64(&dest, &src, f_context, vm, (-(val as i64) as u64));
        } else if val == 0 {
            if dest.id() != src.id() {
                self.backend.emit_mov(&dest, &src);
            }
        } else if is_valid_arithmetic_imm(val) {
            let imm_shift = val > 4096;
            let imm_val = if imm_shift { val >> 12 } else { val };
            self.backend.emit_add_imm(&dest, &src, imm_val as u16, imm_shift);
        } else {
            let tmp = make_temporary(f_context, UINT64_TYPE.clone(), vm);
            emit_mov_u64(self.backend.as_mut(), &tmp, val);
            self.backend.emit_add(&dest, &src, &tmp);
        }
    }

    // Compare register with value
    fn emit_cmp_u64(&mut self, src1: &P<Value>, f_context: &mut FunctionContext, vm: &VM, val: u64)
    {
        if (val as i64) < 0 {
            self.emit_cmn_u64(&src1, f_context, vm, (-(val as i64) as u64));
        } else if val == 0 {
            // Operation has no effect
        } else if is_valid_arithmetic_imm(val) {
            let imm_shift = val > 4096;
            let imm_val = if imm_shift { val >> 12 } else { val };
            self.backend.emit_cmp_imm(&src1, imm_val as u16, imm_shift);
        } else {
            let tmp = make_temporary(f_context, UINT64_TYPE.clone(), vm);
            emit_mov_u64(self.backend.as_mut(), &tmp, val);
            self.backend.emit_cmp(&src1, &tmp);
        }
    }

    // Compare register with value
    fn emit_cmn_u64(&mut self, src1: &P<Value>, f_context: &mut FunctionContext, vm: &VM, val: u64)
    {
        if (val as i64) < 0 {
            self.emit_cmp_u64(&src1, f_context, vm, (-(val as i64) as u64));
        } else if val == 0 {
            // Operation has no effect
        } else if is_valid_arithmetic_imm(val) {
            let imm_shift = val > 4096;
            let imm_val = if imm_shift { val >> 12 } else { val };
            self.backend.emit_cmn_imm(&src1, imm_val as u16, imm_shift);
        } else {
            let tmp = make_temporary(f_context, UINT64_TYPE.clone(), vm);
            emit_mov_u64(self.backend.as_mut(), &tmp, val);
            self.backend.emit_cmn(&src1, &tmp);
        }
    }

    // sign extends reg, to fit in a 32/64 bit register
    fn emit_sext(&mut self, reg: &P<Value>)
    {
        let nreg = check_op_len(&reg.ty); // The size of the aarch64 register
        let nmu = reg.ty.get_int_length().unwrap(); // the size of the Mu type

        // No need to sign extend the zero register
        if nreg > nmu && !is_zero_register(&reg) {
            self.backend.emit_sbfx(&reg, &reg, 0, nmu as u8);
        }
    }

    // zero extends reg, to fit in a 32/64 bit register
    fn emit_zext(&mut self, reg: &P<Value>)
    {
        let nreg = check_op_len(&reg.ty); // The size of the aarch64 register
        let nmu = reg.ty.get_int_length().unwrap(); // the size of the Mu type

        // No need to zero extend the zero register
        if nreg > nmu && !is_zero_register(&reg) {
            self.backend.emit_ubfx(&reg, &reg, 0, nmu as u8);
        }
    }

    // one extends reg, to fit in a 32/64 bit register
    fn emit_oext(&mut self, reg: &P<Value>)
    {
        let nreg = check_op_len(&reg.ty); // The size of the aarch64 register
        let nmu = reg.ty.get_int_length().unwrap(); // the size of the Mu type

        if nreg > nmu {
            if is_zero_register(&reg) {
                unimplemented!(); // Can't one extend the zero register
            }
            self.backend.emit_orr_imm(&reg, &reg, bits_ones(nreg - nmu) << nmu)
        }
    }

    // Masks 'src' so that it can be used to shift 'dest'
    // Returns a register that should be used for the shift operand (may be dest or src)
    fn emit_shift_mask<'b>(&mut self, dest: &'b P<Value>, src: &'b P<Value>) -> &'b P<Value>
    {
        let ndest = dest.ty.get_int_length().unwrap() as u64;

        if ndest < 32 { // 16 or 8 bits (need to mask it)
            self.backend.emit_and_imm(&dest, &src, ndest - 1);
            &dest
        } else {
            &src
        }
    }

    fn emit_mov_f64(&mut self, dest: &P<Value>, f_context: &mut FunctionContext, vm: &VM, val: f64)
    {
        use std::mem;
        if val == 0.0 {
            self.backend.emit_fmov(&dest, &XZR);
        } else if is_valid_f64_imm(val) {
            self.backend.emit_fmov_imm(&dest, val as f32);
        } else {
            match f64_to_aarch64_u64(val) {
                Some(v) => {
                    // Can use a MOVI to load the immediate
                    self.backend.emit_movi(&dest, v);
                }
                None => {
                    // Have to load a temporary GPR with the value first
                    let tmp_int = make_temporary(f_context, UINT64_TYPE.clone(), vm);
                    emit_mov_u64(self.backend.as_mut(), &tmp_int, unsafe { mem::transmute::<f64, u64>(val) });

                    // then move it to an FPR
                    self.backend.emit_fmov(&dest, &tmp_int);
                }
            }
        }
    }

    fn emit_mov_f32(&mut self, dest: &P<Value>, f_context: &mut FunctionContext, vm: &VM, val: f32)
    {
        use std::mem;
        if val == 0.0 {
            self.backend.emit_fmov(&dest, &WZR);
        } else if is_valid_f32_imm(val) {
            self.backend.emit_fmov_imm(&dest, val);
        } else {
            // Have to load a temporary GPR with the value first
            let tmp_int = make_temporary(f_context, UINT32_TYPE.clone(), vm);

            emit_mov_u64(self.backend.as_mut(), &tmp_int, unsafe { mem::transmute::<f32, u32>(val) } as u64);
            // then move it to an FPR
            self.backend.emit_fmov(&dest, &tmp_int);
        }
    }

    // Emits a reg (either an ireg or freg)
    fn emit_reg(&mut self, op: &TreeNode, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) -> P<Value> {
        match op.v {
            TreeNode_::Instruction(_) => {
                self.instruction_select(op, f_content, f_context, vm);

                self.get_result_value(op, 0)
            },
            TreeNode_::Value(ref pv) => self.emit_reg_value(pv, f_context, vm)
        }
    }

    // TODO: Deal with memory case
    fn emit_reg_value(&mut self, pv: &P<Value>, f_context: &mut FunctionContext, vm: &VM) -> P<Value> {
        match pv.v {
            Value_::SSAVar(_) => pv.clone(),
            Value_::Constant(ref c) => {
                match c {
                    &Constant::Int(val) => {
                        /*if val == 0 {
                            // TODO emit the zero register (NOTE: it can't be used by all instructions)
                            // Use the zero register (saves having to use a temporary)
                            get_alias_for_length(XZR.id(), get_bit_size(&pv.ty, vm))
                        } else {*/
                        let tmp = make_temporary(f_context, pv.ty.clone(), vm);
                        debug!("tmp's ty: {}", tmp.ty);
                        emit_mov_u64(self.backend.as_mut(), &tmp, val);
                        tmp
                        //}
                    },
                    &Constant::FuncRef(_) => {
                        unimplemented!()
                    },
                    &Constant::NullRef => {
                        let tmp = make_temporary(f_context, pv.ty.clone(), vm);
                        self.backend.emit_mov_imm(&tmp, 0);
                        tmp
                        //get_alias_for_length(XZR.id(), get_bit_size(&pv.ty, vm))
                    },
                    &Constant::Double(val) => {
                        let tmp = make_temporary(f_context, pv.ty.clone(), vm);
                        self.emit_mov_f64(&tmp, f_context, vm, val);
                        tmp
                    }
                    &Constant::Float(val) => {
                        let tmp = make_temporary(f_context, pv.ty.clone(), vm);
                        self.emit_mov_f32(&tmp, f_context, vm, val);
                        tmp
                    },
                    _ => panic!("expected fpreg or ireg")
                }
            },
            _ => panic!("expected fpreg or ireg")
        }
    }

    fn emit_ireg(&mut self, op: &TreeNode, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) -> P<Value> {
        match op.v {
            TreeNode_::Instruction(_) => {
                self.instruction_select(op, f_content, f_context, vm);

                self.get_result_value(op, 0)
            },
            TreeNode_::Value(ref pv) => emit_ireg_value(self.backend.as_mut(), pv, f_context, vm)
        }
    }

    fn emit_fpreg(&mut self, op: &TreeNode, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) -> P<Value> {
        match op.v {
            TreeNode_::Instruction(_) => {
                self.instruction_select(op, f_content, f_context, vm);
                self.get_result_value(op, 0)
            },
            TreeNode_::Value(ref pv) => self.emit_fpreg_value(pv, f_context, vm)
        }
    }

    // TODO: Deal with memory case
    fn emit_fpreg_value(&mut self, pv: &P<Value>, f_context: &mut FunctionContext, vm: &VM) -> P<Value> {
        match pv.v {
            Value_::SSAVar(_) => pv.clone(),
            Value_::Constant(Constant::Double(val)) => {
                let tmp = make_temporary(f_context, DOUBLE_TYPE.clone(), vm);
                self.emit_mov_f64(&tmp, f_context, vm, val);
                tmp
            }
            Value_::Constant(Constant::Float(val)) => {
                let tmp = make_temporary(f_context, FLOAT_TYPE.clone(), vm);
                self.emit_mov_f32(&tmp, f_context, vm, val);
                tmp
            },
            _ => panic!("expected fpreg")
        }
    }

    // TODO: what exactly is this doing??
    fn emit_node_addr_to_value(&mut self, op: &TreeNode, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) -> P<Value> {
        match op.v {
            TreeNode_::Value(ref pv) => {
                match pv.v {
                    Value_::SSAVar(_) => P(Value{
                        hdr: MuEntityHeader::unnamed(vm.next_id()),
                        ty: pv.ty.clone(),
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
                                P(Value {
                                    hdr: MuEntityHeader::unnamed(vm.next_id()),
                                    ty: pv.ty.clone(),
                                    v: Value_::Memory(MemoryLocation::Symbolic {
                                        label: pv.name().unwrap(),
                                        is_global: true
                                    })
                                })
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
            ty: node_type(&op).clone(),
            v: Value_::Memory(mem)
        })
    }

    fn emit_get_mem_from_inst_inner(&mut self, op: &TreeNode, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) -> MemoryLocation {
        match op.v {
            TreeNode_::Instruction(ref inst) => {
                let ref ops = inst.ops.read().unwrap();

                match inst.v {
                    // GETIREF <T> opnd = opnd
                    Instruction_::GetIRef(op_index) => {
                        let ref ref_op = ops[op_index];
                        let temp = self.emit_ireg(ref_op, f_content, f_context, vm);

                        self.make_memory_location_base_offset(&temp, 0, vm)
                    }

                    // GETFIELDIREF < T1 index > opnd = opnd + offset_of(T1.index)
                    Instruction_::GetFieldIRef{base, index, ..} => {
                        let struct_ty = {
                            let ref iref_or_uptr_ty = ops[base].clone_value().ty;
                            match iref_or_uptr_ty.v {
                                MuType_::IRef(ref ty)
                                | MuType_::UPtr(ref ty) => ty.clone(),
                                _ => panic!("expected the base for GetFieldIRef has a type of iref or uptr, found type: {}", iref_or_uptr_ty)
                            }
                        };
                        let field_offset = self.get_field_offset(&struct_ty, index, vm);
                        self.emit_offset_ref(&ops[base], field_offset, f_content, f_context, vm)
                    }

                    // GETVARPARTIREF < T1 > opnd = opnd + offset_of(T1.var_part)
                    Instruction_::GetVarPartIRef{base, ..} => {
                        let struct_ty = match ops[base].clone_value().ty.get_referenced_ty() {
                            Some(ty) => ty,
                            None => panic!("expecting an iref or uptr in GetVarPartIRef")
                        };
                        let fix_part_size = vm.get_backend_type_info(struct_ty.id()).size;
                        self.emit_offset_ref(&ops[base], fix_part_size as i64, f_content, f_context, vm)
                    }

                    // SHIFTIREF < T1 T2 > opnd offset = opnd + offset*size_of(T1)
                    Instruction_::ShiftIRef{base, offset, ..} => {
                        let element_type = ops[base].clone_value().ty.get_referenced_ty().unwrap();
                        let element_size = vm.get_backend_type_info(element_type.id()).size;
                        self.emit_shift_ref(&ops[base], &ops[offset], element_size, f_content, f_context, vm)
                    }
                    // GETELEMIREF <T1 T2> opnd index = opnd + index*element_size(T1)
                    Instruction_::GetElementIRef{base, index, ..} => {
                        let element_type = ops[base].clone_value().ty.get_referenced_ty().unwrap().get_elem_ty().unwrap();
                        let element_size = vm.get_backend_type_info(element_type.id()).size;

                        self.emit_shift_ref(&ops[base], &ops[index], element_size, f_content, f_context, vm)
                    }
                    _ => panic!("Not a memory reference instruction")
                }
            },
            _ => panic!("expecting a instruction that yields a memory address")
        }
    }

    // Implementes GETVARPARTIREF and GETFIELDIREF
    fn emit_offset_ref(&mut self, base: &TreeNode, offset: i64, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) -> MemoryLocation {
        match base.v {
            TreeNode_::Instruction(Instruction{v: Instruction_::GetIRef{..}, ..}) |
            TreeNode_::Instruction(Instruction{v: Instruction_::GetFieldIRef{..}, ..}) |
            TreeNode_::Instruction(Instruction{v: Instruction_::GetElementIRef{..}, ..}) |
            TreeNode_::Instruction(Instruction{v: Instruction_::GetVarPartIRef{..}, ..}) |
            TreeNode_::Instruction(Instruction{v: Instruction_::ShiftIRef{..}, ..}) => {
                let mem = self.emit_get_mem_from_inst_inner(base, f_content, f_context, vm);
                self.memory_location_shift(mem, offset, f_context, vm)
            },
            _ => {
                let tmp = self.emit_ireg(base, f_content, f_context, vm);
                self.make_memory_location_base_offset(&tmp, offset, vm)
            }
        }
    }

    // Implementes SHIFTIREF and GETELEMENTIREF
    fn emit_shift_ref(&mut self, base: &TreeNode, offset: &TreeNode, element_size: usize, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) -> MemoryLocation {
        if match_node_int_imm(offset) {
            let offset = node_imm_to_u64(offset);
            let shift_size = (element_size as i64) * (offset as i64);

            match base.v {
                // SHIFTIREF(GETVARPARTIREF(_), imm) -> add shift_size to old offset
                TreeNode_::Instruction(Instruction { v: Instruction_::GetIRef { .. }, .. }) |
                TreeNode_::Instruction(Instruction { v: Instruction_::GetFieldIRef { .. }, .. }) |
                TreeNode_::Instruction(Instruction { v: Instruction_::GetElementIRef { .. }, .. }) |
                TreeNode_::Instruction(Instruction { v: Instruction_::GetVarPartIRef { .. }, .. }) |
                TreeNode_::Instruction(Instruction { v: Instruction_::ShiftIRef { .. }, .. }) => {
                    let mem = self.emit_get_mem_from_inst_inner(base, f_content, f_context, vm);
                    self.memory_location_shift(mem, shift_size, f_context, vm)
                },
                // SHIFTIREF(ireg, imm) -> [base + SHIFT_SIZE]
                _ => {
                    let tmp = self.emit_ireg(base, f_content, f_context, vm);
                    self.make_memory_location_base_offset(&tmp, shift_size, vm)
                }
            }
        } else {
            let tmp_offset = self.emit_ireg(offset, f_content, f_context, vm);

            match base.v {
                TreeNode_::Instruction(Instruction { v: Instruction_::GetIRef { .. }, .. }) |
                TreeNode_::Instruction(Instruction { v: Instruction_::GetFieldIRef { .. }, .. }) |
                TreeNode_::Instruction(Instruction { v: Instruction_::GetElementIRef { .. }, .. }) |
                TreeNode_::Instruction(Instruction { v: Instruction_::GetVarPartIRef { .. }, .. }) |
                TreeNode_::Instruction(Instruction { v: Instruction_::ShiftIRef { .. }, .. }) => {
                    let mem = self.emit_get_mem_from_inst_inner(base, f_content, f_context, vm);
                    self.memory_location_shift_scale(mem, &tmp_offset, element_size as u64, f_context, vm)
                },
                _ => {
                    let tmp = self.emit_ireg(base, f_content, f_context, vm);
                    self.make_memory_location_base_offset_scale(&tmp, &tmp_offset, element_size as u64, true)
                }
            }
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

    fn get_result_value(&mut self, node: &TreeNode, index: usize) -> P<Value> {
        match node.v {
            TreeNode_::Instruction(ref inst) => {
                if inst.value.is_some() {
                    let ref value = inst.value.as_ref().unwrap()[index];

                    value.clone()
                } else {
                    panic!("expected result from the node {}", node);
                }
            }

            TreeNode_::Value(ref pv) => {
                if index > 0 {
                    panic!("Didn't expect a value when there is more than one result");
                }
                pv.clone()
            }
        }
    }

    // TODO: This has been modified to simply use iregs and fpregs (NEED TO FIX THIS??)
    fn emit_node_value(&mut self, op: &TreeNode, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) -> P<Value> {
        match op.v {
            TreeNode_::Instruction(_) => {
                self.instruction_select(op, f_content, f_context, vm);

                self.get_result_value(op, 0)
            },
            TreeNode_::Value(ref pv) => pv.clone()
        }
    }

    // TODO: This has been modified to simply use iregs and fpregs (NEED TO FIX THIS??)
    fn emit_move_node_to_value(&mut self, dest: &P<Value>, src: &TreeNode, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) {
        let ref dst_ty = dest.ty;

        if !types::is_fp(dst_ty) && types::is_scalar(dst_ty) {
            if match_node_int_imm(src) {
                let src_imm = node_imm_to_u64(src);
                if dest.is_int_reg() {
                    emit_mov_u64(self.backend.as_mut(), dest, src_imm);
                } else if dest.is_mem() {
                    let tmp = make_temporary(f_context, dest.ty.clone(), vm);
                    emit_mov_u64(self.backend.as_mut(), &tmp, src_imm);
                    self.emit_store(dest, &tmp, f_context, vm);
                } else {
                    panic!("unexpected dest: {}", dest);
                }
            } else if self.match_ireg(src) {
                let src_reg = self.emit_ireg(src, f_content, f_context, vm);
                self.emit_move_value_to_value(dest, &src_reg, f_context, vm);
            } else {
                panic!("expected src: {}", src);
            }
        } else if types::is_fp(dst_ty) && types::is_scalar(dst_ty) {
            if match_node_int_imm(src) {
                if dst_ty.v == MuType_::Double {
                    let src_imm = node_imm_to_f64(src);
                    if dest.is_fp_reg() {
                        self.emit_mov_f64(dest, f_context, vm, src_imm);
                    } else if dest.is_mem() {
                        let tmp = make_temporary(f_context, dest.ty.clone(), vm);
                        self.emit_mov_f64(&tmp, f_context, vm,  src_imm);
                        self.emit_store(dest, &tmp, f_context, vm);
                    } else {
                        panic!("unexpected dest: {}", dest);
                    }
                } else  { // dst_ty.v == MuType_::Float
                    let src_imm = node_imm_to_f32(src);
                    if dest.is_fp_reg() {
                        self.emit_mov_f32(dest, f_context, vm,  src_imm);
                    } else if dest.is_mem() {
                        let tmp = make_temporary(f_context, dest.ty.clone(), vm);
                        self.emit_mov_f32(&tmp, f_context, vm,  src_imm);
                        self.emit_store(dest, &tmp, f_context, vm);
                    } else {
                        panic!("unexpected dest: {}", dest);
                    }
                }
            }
            if self.match_fpreg(src) {
                let src_reg = self.emit_fpreg(src, f_content, f_context, vm);
                self.emit_move_value_to_value(dest, &src_reg, f_context, vm)
            } else {
                panic!("unexpected fp src: {}", src);
            }
        } else {
            unimplemented!()
        }
    }

    fn emit_move_value_to_value(&mut self, dest: &P<Value>, src: &P<Value>, f_context: &mut FunctionContext, vm: &VM) {
        let ref src_ty = src.ty;
        if types::is_scalar(src_ty) && !types::is_fp(src_ty) {
            // gpr mov
            if dest.is_int_reg() && src.is_int_const() {
                let imm = value_imm_to_u64(src);
                emit_mov_u64(self.backend.as_mut(), dest, imm);
            } else if dest.is_int_reg() && src.is_int_reg() {
                self.backend.emit_mov(dest, src);
            } else if dest.is_int_reg() && src.is_mem() {
                self.emit_load(&dest, &src, f_context, vm);
            } else if dest.is_mem() {
                let temp = emit_ireg_value(self.backend.as_mut(), src, f_context, vm);
                self.emit_store(dest, &temp, f_context, vm);
            } else {
                panic!("unexpected gpr mov between {} -> {}", src, dest);
            }
        } else if types::is_scalar(src_ty) && types::is_fp(src_ty) {
            // fpr mov
            if dest.is_fp_reg() && match_value_f32imm(src) {
                let src = value_imm_to_f32(src);
                self.emit_mov_f32(dest, f_context, vm, src);
            } else if dest.is_fp_reg() && match_value_f64imm(src) {
                let src = value_imm_to_f64(src);
                self.emit_mov_f64(dest, f_context, vm, src);
            } else if dest.is_fp_reg() && src.is_fp_reg() {
                self.backend.emit_fmov(dest, src);
            } else if dest.is_fp_reg() && src.is_mem() {
                self.emit_load(&dest, &src, f_context, vm);
            } else if dest.is_mem() {
                let temp = self.emit_fpreg_value(src, f_context, vm);
                self.emit_store(dest, &temp, f_context, vm);
            } else {
                panic!("unexpected fpr mov between {} -> {}", src, dest);
            }
        } else {
            panic!("unexpected mov of type {}", src_ty)
        }
    }

    fn emit_load(&mut self, dest: &P<Value>, src: &P<Value>, f_context: &mut FunctionContext, vm: &VM) {
        let src = emit_mem(self.backend.as_mut(), &src, f_context, vm);
        self.backend.emit_ldr(&dest, &src, false);
    }

    fn emit_store(&mut self, dest: &P<Value>, src: &P<Value>, f_context: &mut FunctionContext, vm: &VM) {
        let dest = emit_mem(self.backend.as_mut(), &dest, f_context, vm);
        self.backend.emit_str(&dest, &src);
    }

    fn emit_landingpad(&mut self, exception_arg: &P<Value>, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) {
        // get thread local and add offset to get exception_obj
        let tl = self.emit_get_threadlocal(None, f_content, f_context, vm);
        self.emit_load_base_offset(exception_arg, &tl, *thread::EXCEPTION_OBJ_OFFSET as i64, f_context, vm);
    }

    fn get_field_offset(&mut self, ty: &P<MuType>, index: usize, vm: &VM) -> i64 {
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
                        ty : ADDRESS_TYPE.clone(), // TODO: What is the type of the constant
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

    fn finish_block(&mut self, live_out: &Vec<P<Value>>) {
        let cur_block = self.current_block.as_ref().unwrap().clone();
        self.backend.end_block(cur_block.clone());
        self.backend.set_block_liveout(cur_block, &live_out);
    }

    fn start_block(&mut self, block: String, live_in: &Vec<P<Value>>) {
        self.current_block = Some(block.clone());
        self.backend.start_block(block.clone());
        self.backend.set_block_livein(block, &live_in);
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
        self.emit_common_prologue(args, &func_ver.sig, &mut func_ver.context, vm);
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
                self.emit_landingpad(&exception_arg, f_content, &mut func.context, vm);
            } else {
                // live in is args of the block
                self.backend.set_block_livein(block_label.clone(), &block_content.args);
            }

            // live out is the union of all branch args of this block
            let live_out = block_content.get_out_arguments();

            // doing the actual instruction selection
            for inst in block_content.body.iter() {
                self.instruction_select(&inst, f_content, &mut func.context, vm);
            }

            // we may start block a, and end with block b (instruction selection may create blocks)
            // we set liveout to current block
            self.finish_block(&live_out);
            self.current_block = None;
        }
    }

    fn finish_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        self.emit_common_epilogue(&func.sig.ret_tys, &mut func.context, vm);

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
