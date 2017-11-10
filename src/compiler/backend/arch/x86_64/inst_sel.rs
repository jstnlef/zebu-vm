// Copyright 2017 The Australian National University
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use ast::ir::*;
use ast::ptr::*;
use ast::inst::*;
use ast::op;
use ast::op::*;
use ast::types::*;
use vm::VM;
use runtime::mm;
use runtime::mm::OBJECT_HEADER_SIZE;
use runtime::ValueLocation;
use runtime::thread;
use runtime::entrypoints;
use runtime::entrypoints::RuntimeEntrypoint;

use compiler::PROLOGUE_BLOCK_NAME;
use compiler::CompilerPass;
use compiler::backend::*;
use compiler::backend::x86_64;
use compiler::backend::x86_64::*;
use compiler::backend::x86_64::callconv;
use compiler::backend::x86_64::callconv::CallConvResult;
use compiler::machine_code::CompiledFunction;
use compiler::frame::Frame;

use utils::math;
use utils::{POINTER_SIZE, WORD_SIZE};
use utils::{BitSize, ByteSize};

use std::collections::HashMap;
use std::collections::LinkedList;
use std::sync::Arc;
use std::any::Any;

lazy_static! {
    /// struct<int32, int32, int32, int32>
    static ref LONG_4_TYPE : P<MuType> = P(
        MuType::new(new_internal_id(), MuType_::mustruct(Mu("long_4"),
        vec![UINT32_TYPE.clone(); 4]))
    );

    /// constant for converting unsigned integer to floating point
    static ref UITOFP_C0 : P<Value> = P(Value{
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

    /// struct<int64, int64>
    static ref QUAD_2_TYPE : P<MuType> = P(
        MuType::new(new_internal_id(), MuType_::mustruct(Mu("quad_2"),
        vec![UINT64_TYPE.clone(); 2]))
    );

    /// constant for converting unsigned integer to floating point
    static ref UITOFP_C1 : P<Value> = P(Value{
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

    /// constant for converting double to unsigned integer
    static ref FPTOUI_C_DOUBLE : P<Value> = P(Value{
        hdr: MuEntityHeader::named(new_internal_id(), Mu("FPTOUI_C_DOUBLE")),
        ty : UINT64_TYPE.clone(),
        v  : Value_::Constant(Constant::Int(4890909195324358656u64))
    });

    /// constant for converting float to unsigned integer
    pub static ref FPTOUI_C_FLOAT : P<Value> = P(Value{
        hdr: MuEntityHeader::named(new_internal_id(), Mu("FPTOUI_C_FLOAT")),
        ty : UINT32_TYPE.clone(),
        v  : Value_::Constant(Constant::Int(1593835520u64))
    });
}

/// for some IR instructions, we need a call into runtime
/// for efficiency, we may emit runtime fastpath directly in assembly
//  FIXME: we should have a separate pass to rewrite the instruction into a fastpath (in IR),
//         and a call to slowpath, then instruction selection (see Issue#6)
const INLINE_FASTPATH: bool = false;

pub struct InstructionSelection {
    name: &'static str,
    /// backend code generator
    backend: Box<CodeGenerator>,

    // information about the function being compiled
    /// ID of current function version being compiled
    current_fv_id: MuID,
    /// name of current function version being compiled
    current_fv_name: MuName,
    /// signature of current function being compiled
    current_sig: Option<P<MuFuncSig>>,
    /// used to create a unique callsite ID for current function
    current_callsite_id: usize,
    /// frame for current function
    current_frame: Option<Frame>,
    /// block that is currently being compiled
    /// the block may be created during instruction selection
    current_block: Option<MuName>,
    /// IR block that is currently being compiled
    /// use this block name to trace some information at IR level
    current_block_in_ir: Option<MuName>,
    /// start location of current function
    current_func_start: Option<ValueLocation>,
    /// technically this is a map in that each Key is unique, but we will never try and
    /// add duplicate keys, or look things up, so a list of tuples is faster than a Map.
    /// A list of tuples, the first is the name of a callsite, the next is the callsite destination,
    /// the last is the size of arguments pushed on the stack
    current_callsites: LinkedList<(MuName, MuID, usize)>,
    // key: block id, val: block location
    current_exn_blocks: HashMap<MuID, MuName>,
    /// constants used in this function that are put to memory
    /// key: value id, val: constant value
    current_constants: HashMap<MuID, P<Value>>,
    /// constants used in this function that are put to memory
    /// key: value id, val: memory location
    current_constants_locs: HashMap<MuID, P<Value>>
}

impl<'a> InstructionSelection {
    #[cfg(feature = "aot")]
    pub fn new() -> InstructionSelection {
        InstructionSelection {
            name: "Instruction Selection (x64)",
            backend: Box::new(ASMCodeGen::new()),

            current_fv_id: 0,
            current_fv_name: Arc::new(String::new()),
            current_sig: None,
            current_callsite_id: 0,
            current_frame: None,
            // which block we are generating code for
            current_block: None,
            // it is possible the block is newly created in instruction
            // selection
            // but sometimes we want to know its control flow
            // so we need to track what block it is from the IR

            // FIXME: ideally we should not create new blocks in instruction selection
            // see Issue #6
            current_block_in_ir: None,
            current_func_start: None,
            current_callsites: LinkedList::new(),
            current_exn_blocks: HashMap::new(),

            current_constants: HashMap::new(),
            current_constants_locs: HashMap::new()
        }
    }

    #[cfg(feature = "jit")]
    pub fn new() -> InstructionSelection {
        unimplemented!()
    }

    /// we use hand-written pattern matching rules for instruction selection
    /// for a pattern that can match several rules, the first rule met will be executed, and chosen.
    fn instruction_select(
        &mut self,
        node: &'a TreeNode,
        f_content: &FunctionContent,
        f_context: &mut FunctionContext,
        vm: &VM
    ) {
        trace!("instsel on node#{} {}", node.id(), node);

        match node.v {
            TreeNode_::Instruction(ref inst) => {
                match inst.v {
                    Instruction_::Branch2 {
                        cond,
                        ref true_dest,
                        ref false_dest,
                        ..
                    } => {
                        trace!("instsel on BRANCH2");
                        let (fallthrough_dest, branch_dest) = (false_dest, true_dest);

                        let ref ops = inst.ops;
                        self.process_dest(&ops, fallthrough_dest, f_content, f_context, vm);
                        self.process_dest(&ops, branch_dest, f_content, f_context, vm);

                        let branch_target = f_content.get_block(branch_dest.target.id()).name();

                        let ref cond = ops[cond];
                        if self.match_cmp_res(cond) {
                            // this branch2's cond is from a comparison result
                            trace!("emit cmp_res-branch2");
                            match self.emit_cmp_res(cond, f_content, f_context, vm) {
                                op::CmpOp::EQ => self.backend.emit_je(branch_target),
                                op::CmpOp::NE => self.backend.emit_jne(branch_target),
                                op::CmpOp::UGE => self.backend.emit_jae(branch_target),
                                op::CmpOp::UGT => self.backend.emit_ja(branch_target),
                                op::CmpOp::ULE => self.backend.emit_jbe(branch_target),
                                op::CmpOp::ULT => self.backend.emit_jb(branch_target),
                                op::CmpOp::SGE => self.backend.emit_jge(branch_target),
                                op::CmpOp::SGT => self.backend.emit_jg(branch_target),
                                op::CmpOp::SLE => self.backend.emit_jle(branch_target),
                                op::CmpOp::SLT => self.backend.emit_jl(branch_target),

                                // floating point
                                op::CmpOp::FOEQ | op::CmpOp::FUEQ => {
                                    self.backend.emit_je(branch_target)
                                }
                                op::CmpOp::FONE | op::CmpOp::FUNE => {
                                    self.backend.emit_jne(branch_target)
                                }
                                op::CmpOp::FOGT | op::CmpOp::FUGT => {
                                    self.backend.emit_ja(branch_target)
                                }
                                op::CmpOp::FOGE | op::CmpOp::FUGE => {
                                    self.backend.emit_jae(branch_target)
                                }
                                op::CmpOp::FOLT | op::CmpOp::FULT => {
                                    self.backend.emit_jb(branch_target)
                                }
                                op::CmpOp::FOLE | op::CmpOp::FULE => {
                                    self.backend.emit_jbe(branch_target)
                                }

                                _ => unimplemented!()
                            }
                        } else if self.match_ireg(cond) {
                            // this branch2 cond is a temporary with value, or an instruction that
                            // emits a temporary
                            trace!("emit ireg-branch2");

                            let cond_reg = self.emit_ireg(cond, f_content, f_context, vm);

                            // emit: cmp cond_reg 1
                            self.backend.emit_cmp_imm_r(1, &cond_reg);
                            // emit: je #branch_dest
                            self.backend.emit_je(branch_target);
                        } else {
                            panic!("unexpected cond in BRANCH2: {}", cond)
                        }
                    }

                    Instruction_::Select {
                        cond,
                        true_val,
                        false_val
                    } => {
                        use ast::op::CmpOp::*;

                        trace!("instsel on SELECT");
                        let ref ops = inst.ops;

                        let ref cond = ops[cond];
                        let ref true_val = ops[true_val];
                        let ref false_val = ops[false_val];

                        // generate comparison
                        let cmpop = if self.match_cmp_res(cond) {
                            self.emit_cmp_res(cond, f_content, f_context, vm)
                        } else if self.match_ireg(cond) {
                            let tmp_cond = self.emit_ireg(cond, f_content, f_context, vm);
                            // emit: cmp cond_reg 1
                            self.backend.emit_cmp_imm_r(1, &tmp_cond);

                            EQ
                        } else {
                            panic!("expected cond to be ireg, found {}", cond)
                        };

                        // emit code to move values
                        if self.match_ireg(true_val) {
                            // moving integers/pointers
                            let tmp_res = self.get_result_value(node);

                            // use cmov for 16/32/64bit integer
                            // use jcc  for 8 bit
                            // FIXME: could use 32bit register to implement 8bit select
                            match tmp_res.ty.get_int_length() {
                                // cmov
                                Some(len) if len > 8 => {
                                    let tmp_true =
                                        self.emit_ireg(true_val, f_content, f_context, vm);
                                    let tmp_false =
                                        self.emit_ireg(false_val, f_content, f_context, vm);

                                    // mov tmp_false -> tmp_res
                                    self.backend.emit_mov_r_r(&tmp_res, &tmp_false);

                                    match cmpop {
                                        EQ => self.backend.emit_cmove_r_r(&tmp_res, &tmp_true),
                                        NE => self.backend.emit_cmovne_r_r(&tmp_res, &tmp_true),
                                        SGE => self.backend.emit_cmovge_r_r(&tmp_res, &tmp_true),
                                        SGT => self.backend.emit_cmovg_r_r(&tmp_res, &tmp_true),
                                        SLE => self.backend.emit_cmovle_r_r(&tmp_res, &tmp_true),
                                        SLT => self.backend.emit_cmovl_r_r(&tmp_res, &tmp_true),
                                        UGE => self.backend.emit_cmovae_r_r(&tmp_res, &tmp_true),
                                        UGT => self.backend.emit_cmova_r_r(&tmp_res, &tmp_true),
                                        ULE => self.backend.emit_cmovbe_r_r(&tmp_res, &tmp_true),
                                        ULT => self.backend.emit_cmovb_r_r(&tmp_res, &tmp_true),

                                        FOEQ | FUEQ => {
                                            self.backend.emit_cmove_r_r(&tmp_res, &tmp_true)
                                        }
                                        FONE | FUNE => {
                                            self.backend.emit_cmovne_r_r(&tmp_res, &tmp_true)
                                        }
                                        FOGT | FUGT => {
                                            self.backend.emit_cmova_r_r(&tmp_res, &tmp_true)
                                        }
                                        FOGE | FUGE => {
                                            self.backend.emit_cmovae_r_r(&tmp_res, &tmp_true)
                                        }
                                        FOLT | FULT => {
                                            self.backend.emit_cmovb_r_r(&tmp_res, &tmp_true)
                                        }
                                        FOLE | FULE => {
                                            self.backend.emit_cmovbe_r_r(&tmp_res, &tmp_true)
                                        }

                                        // FFALSE/FTRUE unimplemented
                                        _ => unimplemented!()
                                    }
                                }
                                // jcc - for 8-bits integer
                                _ => {
                                    let blk_true = make_block_name(&node.name(), "select_true");
                                    let blk_false = make_block_name(&node.name(), "select_false");
                                    let blk_end = make_block_name(&node.name(), "select_end");

                                    // jump to blk_true if true
                                    match cmpop {
                                        EQ => self.backend.emit_je(blk_true.clone()),
                                        NE => self.backend.emit_jne(blk_true.clone()),
                                        SGE => self.backend.emit_jge(blk_true.clone()),
                                        SGT => self.backend.emit_jg(blk_true.clone()),
                                        SLE => self.backend.emit_jle(blk_true.clone()),
                                        SLT => self.backend.emit_jl(blk_true.clone()),
                                        UGE => self.backend.emit_jae(blk_true.clone()),
                                        UGT => self.backend.emit_ja(blk_true.clone()),
                                        ULE => self.backend.emit_jbe(blk_true.clone()),
                                        ULT => self.backend.emit_jb(blk_true.clone()),

                                        FOEQ | FUEQ => self.backend.emit_je(blk_true.clone()),
                                        FONE | FUNE => self.backend.emit_jne(blk_true.clone()),
                                        FOGT | FUGT => self.backend.emit_ja(blk_true.clone()),
                                        FOGE | FUGE => self.backend.emit_jae(blk_true.clone()),
                                        FOLT | FULT => self.backend.emit_jb(blk_true.clone()),
                                        FOLE | FULE => self.backend.emit_jbe(blk_true.clone()),

                                        // FFALSE/FTRUE unimplemented
                                        _ => unimplemented!()
                                    }

                                    // finishing current block
                                    self.finish_block();

                                    // blk_false:
                                    self.start_block(blk_false.clone());
                                    // mov false result here
                                    self.emit_move_node_to_value(
                                        &tmp_res,
                                        &false_val,
                                        f_content,
                                        f_context,
                                        vm
                                    );
                                    // jmp to end
                                    self.backend.emit_jmp(blk_end.clone());
                                    // finishing current block
                                    self.finish_block();

                                    // blk_true:
                                    self.start_block(blk_true.clone());
                                    // mov true value -> result
                                    self.emit_move_node_to_value(
                                        &tmp_res,
                                        &true_val,
                                        f_content,
                                        f_context,
                                        vm
                                    );
                                    self.finish_block();

                                    // blk_end:
                                    self.start_block(blk_end.clone());
                                }
                            }
                        } else if self.match_fpreg(true_val) {
                            let tmp_res = self.get_result_value(node);

                            let blk_true = make_block_name(&node.name(), "select_true");
                            let blk_false = make_block_name(&node.name(), "select_false");
                            let blk_end = make_block_name(&node.name(), "select_end");

                            // jump to blk_true if true
                            match cmpop {
                                EQ => self.backend.emit_je(blk_true.clone()),
                                NE => self.backend.emit_jne(blk_true.clone()),
                                SGE => self.backend.emit_jge(blk_true.clone()),
                                SGT => self.backend.emit_jg(blk_true.clone()),
                                SLE => self.backend.emit_jle(blk_true.clone()),
                                SLT => self.backend.emit_jl(blk_true.clone()),
                                UGE => self.backend.emit_jae(blk_true.clone()),
                                UGT => self.backend.emit_ja(blk_true.clone()),
                                ULE => self.backend.emit_jbe(blk_true.clone()),
                                ULT => self.backend.emit_jb(blk_true.clone()),

                                FOEQ | FUEQ => self.backend.emit_je(blk_true.clone()),
                                FONE | FUNE => self.backend.emit_jne(blk_true.clone()),
                                FOGT | FUGT => self.backend.emit_ja(blk_true.clone()),
                                FOGE | FUGE => self.backend.emit_jae(blk_true.clone()),
                                FOLT | FULT => self.backend.emit_jb(blk_true.clone()),
                                FOLE | FULE => self.backend.emit_jbe(blk_true.clone()),

                                _ => unimplemented!()
                            }

                            // finishing current block
                            self.finish_block();

                            // blk_false:
                            self.start_block(blk_false.clone());
                            // mov false result here
                            self.emit_move_node_to_value(
                                &tmp_res,
                                &false_val,
                                f_content,
                                f_context,
                                vm
                            );
                            // jmp to end
                            self.backend.emit_jmp(blk_end.clone());

                            // finishing current block
                            self.finish_block();

                            // blk_true:
                            self.start_block(blk_true.clone());
                            // mov true value -> result
                            self.emit_move_node_to_value(
                                &tmp_res,
                                &true_val,
                                f_content,
                                f_context,
                                vm
                            );
                            self.finish_block();

                            // blk_end:
                            self.start_block(blk_end.clone());
                        } else {
                            unimplemented!()
                        }
                    }

                    Instruction_::CmpOp(_, _, _) => {
                        use ast::op::CmpOp::*;
                        trace!("instsel on CMPOP");

                        let tmp_res = self.get_result_value(node);
                        assert!(tmp_res.ty.get_int_length().is_some());
                        assert!(tmp_res.ty.get_int_length().unwrap() == 1);

                        // set byte to result
                        match self.emit_cmp_res(node, f_content, f_context, vm) {
                            EQ => self.backend.emit_sete_r(&tmp_res),
                            NE => self.backend.emit_setne_r(&tmp_res),
                            SGE => self.backend.emit_setge_r(&tmp_res),
                            SGT => self.backend.emit_setg_r(&tmp_res),
                            SLE => self.backend.emit_setle_r(&tmp_res),
                            SLT => self.backend.emit_setl_r(&tmp_res),
                            UGE => self.backend.emit_setae_r(&tmp_res),
                            UGT => self.backend.emit_seta_r(&tmp_res),
                            ULE => self.backend.emit_setbe_r(&tmp_res),
                            ULT => self.backend.emit_setb_r(&tmp_res),

                            FOEQ | FUEQ => self.backend.emit_sete_r(&tmp_res),
                            FONE | FUNE => self.backend.emit_setne_r(&tmp_res),
                            FOGT | FUGT => self.backend.emit_seta_r(&tmp_res),
                            FOGE | FUGE => self.backend.emit_setae_r(&tmp_res),
                            FOLT | FULT => self.backend.emit_setb_r(&tmp_res),
                            FOLE | FULE => self.backend.emit_setbe_r(&tmp_res),

                            // FFALSE/FTRUE
                            _ => unimplemented!()
                        }
                    }

                    Instruction_::Branch1(ref dest) => {
                        trace!("instsel on BRANCH1");
                        let ref ops = inst.ops;

                        self.process_dest(&ops, dest, f_content, f_context, vm);

                        let target = f_content.get_block(dest.target.id()).name();
                        // jmp
                        self.backend.emit_jmp(target);
                    }

                    Instruction_::Switch {
                        cond,
                        ref default,
                        ref branches
                    } => {
                        trace!("instsel on SWITCH");
                        let ref ops = inst.ops;
                        let ref cond = ops[cond];

                        if self.match_ireg(cond) {
                            let tmp_cond = self.emit_ireg(cond, f_content, f_context, vm);

                            // currently implementing switch as cascading conditional branch
                            // This is slow if there are many 'case' arms. We should consider
                            // using a switch table

                            // emit each branch
                            for &(case_op_index, ref case_dest) in branches {
                                let ref case_op = ops[case_op_index];

                                // process dest
                                self.process_dest(&ops, case_dest, f_content, f_context, vm);

                                let target = f_content.get_block(case_dest.target.id()).name();

                                if self.match_iimm(case_op) {
                                    let imm = self.node_iimm_to_i32(case_op);

                                    // cmp case cond
                                    self.backend.emit_cmp_imm_r(imm, &tmp_cond);
                                    // je dest
                                    self.backend.emit_je(target);
                                } else if self.match_ireg(case_op) {
                                    let tmp_case_op =
                                        self.emit_ireg(case_op, f_content, f_context, vm);

                                    // cmp case cond
                                    self.backend.emit_cmp_r_r(&tmp_case_op, &tmp_cond);
                                    // je dest
                                    self.backend.emit_je(target);
                                } else {
                                    panic!(
                                        "expecting ireg cond to be either iimm or ireg: {}",
                                        cond
                                    );
                                }

                                self.finish_block();
                                let block_name = make_block_name(
                                    &node.name(),
                                    format!("switch_not_met_case_{}", case_op_index).as_str()
                                );
                                self.start_block(block_name);
                            }

                            // emit default
                            self.process_dest(&ops, default, f_content, f_context, vm);

                            let default_target = f_content.get_block(default.target.id()).name();
                            self.backend.emit_jmp(default_target);
                        } else {
                            // other EQ-comparable types, e.g. floating point
                            unimplemented!()
                        }
                    }

                    Instruction_::ExprCall { ref data, is_abort } => {
                        trace!("instsel on EXPRCALL");

                        if is_abort {
                            // if any exception throws from the callee,
                            // we should abort execution, otherwise rethrow the exception
                            // FIXME: implement is_abort
                            unimplemented!()
                        }

                        self.emit_mu_call(
                            inst, // inst: &Instruction,
                            data, // calldata: &CallData,
                            None, // resumption: Option<&ResumptionData>,
                            node, // cur_node: &TreeNode,
                            f_content,
                            f_context,
                            vm
                        );
                    }

                    Instruction_::Call {
                        ref data,
                        ref resume
                    } => {
                        trace!("instsel on CALL");

                        self.emit_mu_call(inst, data, Some(resume), node, f_content, f_context, vm);
                    }

                    Instruction_::ExprCCall { ref data, is_abort } => {
                        trace!("instsel on EXPRCCALL");

                        if is_abort {
                            // if any exception throws from the callee,
                            // we should abort execution, otherwise rethrow the exception
                            // FIXME: implement is_abort
                            unimplemented!()
                        }

                        self.emit_c_call_ir(inst, data, None, node, f_content, f_context, vm);
                    }

                    Instruction_::CCall {
                        ref data,
                        ref resume
                    } => {
                        trace!("instsel on CCALL");

                        self.emit_c_call_ir(
                            inst,
                            data,
                            Some(resume),
                            node,
                            f_content,
                            f_context,
                            vm
                        );
                    }

                    Instruction_::Return(_) => {
                        trace!("instsel on RETURN");

                        self.emit_common_epilogue(inst, f_content, f_context, vm);

                        self.backend.emit_ret();
                    }

                    Instruction_::BinOp(op, op1, op2) => {
                        trace!("instsel on BINOP");

                        self.emit_binop(node, inst, op, op1, op2, f_content, f_context, vm);
                    }

                    Instruction_::BinOpWithStatus(op, status, op1, op2) => {
                        trace!("instsel on BINOP_STATUS");

                        self.emit_binop(node, inst, op, op1, op2, f_content, f_context, vm);

                        let values = inst.value.as_ref().unwrap();
                        let mut status_value_index = 1;

                        // status flags only works with int operations
                        if RegGroup::get_from_value(&values[0]) == RegGroup::GPR {
                            // for mul, div, idiv, some of the flags may not generated
                            // from the computation, and we may need extra code
                            // to get the flags
                            // FIXME: See Issue#22

                            // negative flag
                            if status.flag_n {
                                let tmp_status = values[status_value_index].clone();
                                status_value_index += 1;

                                self.backend.emit_sets_r8(&tmp_status);
                            }

                            // zero flag
                            if status.flag_z {
                                let tmp_status = values[status_value_index].clone();
                                status_value_index += 1;

                                self.backend.emit_setz_r8(&tmp_status);
                            }

                            // unsigned overflow
                            if status.flag_c {
                                let tmp_status = values[status_value_index].clone();
                                status_value_index += 1;

                                match op {
                                    BinOp::Add | BinOp::Sub | BinOp::Mul => {
                                        self.backend.emit_setb_r8(&tmp_status);
                                    }
                                    _ => panic!("Only Add/Sub/Mul has #C flag")
                                }
                            }

                            // signed overflow
                            if status.flag_v {
                                let tmp_status = values[status_value_index].clone();

                                match op {
                                    BinOp::Add | BinOp::Sub | BinOp::Mul => {
                                        self.backend.emit_seto_r8(&tmp_status);
                                    }
                                    _ => panic!("Only Add/Sub/Mul has #V flag")
                                }
                            }
                        } else if RegGroup::get_from_value(&values[0]) == RegGroup::GPREX {
                            unimplemented!()
                        } else {
                            panic!("only int operations allow binop status flags")
                        }
                    }

                    Instruction_::ConvOp {
                        operation,
                        ref from_ty,
                        ref to_ty,
                        operand
                    } => {
                        trace!("instsel on CONVOP");

                        let ref ops = inst.ops;
                        let ref op = ops[operand];

                        match operation {
                            // Truncate (from int to int)
                            op::ConvOp::TRUNC => {
                                let tmp_res = self.get_result_value(node);
                                let to_ty_size = vm.get_backend_type_size(tmp_res.ty.id());

                                if self.match_ireg(op) {
                                    let tmp_op = self.emit_ireg(op, f_content, f_context, vm);

                                    // mov op -> result
                                    match to_ty_size {
                                        1 => {
                                            self.backend.emit_mov_r_r(&tmp_res, unsafe {
                                                &tmp_op.as_type(UINT8_TYPE.clone())
                                            })
                                        }
                                        2 => {
                                            self.backend.emit_mov_r_r(&tmp_res, unsafe {
                                                &tmp_op.as_type(UINT16_TYPE.clone())
                                            })
                                        }
                                        4 => {
                                            self.backend.emit_mov_r_r(&tmp_res, unsafe {
                                                &tmp_op.as_type(UINT32_TYPE.clone())
                                            })
                                        }
                                        8 => {
                                            self.backend.emit_mov_r_r(&tmp_res, unsafe {
                                                &tmp_op.as_type(UINT64_TYPE.clone())
                                            })
                                        }
                                        _ => panic!("unsupported int size: {}", to_ty_size)
                                    }
                                } else if self.match_ireg_ex(op) {
                                    let (op_l, _) = self.emit_ireg_ex(op, f_content, f_context, vm);

                                    match to_ty_size {
                                        1 | 2 => {
                                            self.backend.emit_movz_r_r(
                                                unsafe { &tmp_res.as_type(UINT32_TYPE.clone()) },
                                                &op_l
                                            )
                                        }
                                        4 | 8 => self.backend.emit_mov_r_r(&tmp_res, &op_l),
                                        _ => panic!("unsupported int size: {}", to_ty_size)
                                    }
                                } else {
                                    panic!("unexpected op (expect ireg): {}", op);
                                }
                            }
                            // Zero extend (from int to int)
                            op::ConvOp::ZEXT => {
                                if self.match_ireg(op) {
                                    let tmp_op = self.emit_ireg(op, f_content, f_context, vm);
                                    let tmp_res = self.get_result_value(node);

                                    // movz op -> result
                                    let from_ty_size = vm.get_backend_type_size(from_ty.id());
                                    let to_ty_size = vm.get_backend_type_size(to_ty.id());

                                    // we treat int1 as int8, so it is possible
                                    // from_ty_size == to_ty_size == 1 byte
                                    assert!(from_ty_size <= to_ty_size);

                                    if from_ty_size != to_ty_size {
                                        match (from_ty_size, to_ty_size) {
                                            // int32 to int64
                                            (4, 8) => {
                                                // zero extend from 32 bits to 64 bits is
                                                // a mov instruction
                                                // x86 does not have movzlq (32 to 64)

                                                // tmp_op is int32, but tmp_res is int64
                                                // we want to force a 32-to-32 mov, so high bits
                                                // of the destination will be zeroed
                                                let tmp_res32 =
                                                    unsafe { tmp_res.as_type(UINT32_TYPE.clone()) };

                                                self.backend.emit_mov_r_r(&tmp_res32, &tmp_op);
                                            }
                                            // any int to int128
                                            (_, 16) => {
                                                let (res_l, res_h) =
                                                    self.split_int128(&tmp_res, f_context, vm);

                                                // use the temp as 64bit temp, mask it
                                                let tmp_op64 =
                                                    unsafe { &tmp_op.as_type(UINT64_TYPE.clone()) };
                                                self.emit_apply_mask(
                                                    &tmp_op64,
                                                    from_ty_size * 8,
                                                    f_context,
                                                    vm
                                                );

                                                // use temp as lower bits
                                                // clear higher bits
                                                self.backend.emit_mov_r_r(&res_l, &tmp_op64);
                                                self.backend.emit_mov_r_imm(&res_h, 0);
                                            }
                                            // other cases
                                            _ => {
                                                self.backend.emit_movz_r_r(&tmp_res, &tmp_op);
                                            }
                                        }
                                    } else {
                                        self.backend.emit_mov_r_r(&tmp_res, &tmp_op);
                                    }
                                } else {
                                    panic!("unexpected op (expect ireg): {}", op);
                                }
                            }
                            // Sign extend (from int to int)
                            op::ConvOp::SEXT => {
                                if self.match_ireg(op) {
                                    let tmp_op = self.emit_ireg(op, f_content, f_context, vm);
                                    let tmp_res = self.get_result_value(node);

                                    // movs op -> result
                                    let from_ty_size = vm.get_backend_type_size(from_ty.id());
                                    let to_ty_size = vm.get_backend_type_size(to_ty.id());

                                    // we treat int1 as int8, so it is possible
                                    // from_ty_size == to_ty_size == 1 byte
                                    assert!(from_ty_size <= to_ty_size);

                                    if from_ty_size != to_ty_size {
                                        match (from_ty_size, to_ty_size) {
                                            // int64 to int128
                                            (8, 16) => {
                                                let (res_l, res_h) =
                                                    self.split_int128(&tmp_res, f_context, vm);

                                                // mov tmp_op -> res_h
                                                // sar res_h 63
                                                self.backend.emit_mov_r_r(&res_h, &tmp_op);
                                                self.backend.emit_sar_r_imm8(&res_h, 63i8);

                                                // mov tmp_op -> res_l
                                                self.backend.emit_mov_r_r(&res_l, &tmp_op);
                                            }
                                            // int32 to int128
                                            (_, 16) => {
                                                let (res_l, res_h) =
                                                    self.split_int128(&tmp_res, f_context, vm);

                                                // movs tmp_op -> res_l
                                                self.backend.emit_movs_r_r(&res_l, &tmp_op);

                                                // mov res_l -> res_h
                                                // sar res_h 63
                                                self.backend.emit_mov_r_r(&res_h, &res_l);
                                                self.backend.emit_sar_r_imm8(&res_h, 63i8);
                                            }
                                            _ => self.backend.emit_movs_r_r(&tmp_res, &tmp_op)
                                        }
                                    } else {
                                        // FIXME: sign extending <int1> 1 to <int8>
                                        // is not a plain move
                                        self.backend.emit_mov_r_r(&tmp_res, &tmp_op);
                                    }
                                } else {
                                    panic!("unexpected op (expect ireg): {}", op)
                                }
                            }
                            // pointer/ref cast
                            op::ConvOp::REFCAST | op::ConvOp::PTRCAST => {
                                // just a mov (and hopefully reg alloc will coalesce it)
                                let tmp_res = self.get_result_value(node);

                                if self.match_ireg(op) {
                                    let tmp_op = self.emit_ireg(op, f_content, f_context, vm);
                                    self.backend.emit_mov_r_r(&tmp_res, &tmp_op);
                                } else if self.match_mem(op) {
                                    let mem_op = self.emit_mem(op, f_content, f_context, vm);
                                    self.backend.emit_lea_r64(&tmp_res, &mem_op);
                                } else {
                                    panic!("unexpected op (expect ireg): {}", op)
                                }
                            }
                            // signed integer to floating point
                            op::ConvOp::SITOFP => {
                                let tmp_res = self.get_result_value(node);

                                if self.match_ireg(op) {
                                    let tmp_op = self.emit_ireg(op, f_content, f_context, vm);
                                    match to_ty.v {
                                        MuType_::Double => {
                                            self.backend.emit_cvtsi2sd_f64_r(&tmp_res, &tmp_op)
                                        }
                                        MuType_::Float => {
                                            self.backend.emit_cvtsi2ss_f32_r(&tmp_res, &tmp_op)
                                        }
                                        _ => {
                                            panic!(
                                                "expecting fp type as to type in SITOFP, found {}",
                                                to_ty
                                            )
                                        }
                                    }
                                } else if self.match_ireg_ex(op) {
                                    unimplemented!()
                                } else {
                                    panic!("unexpected op (expect ireg/ireg_ex): {}", op)
                                }
                            }
                            // floating point to signed integer
                            op::ConvOp::FPTOSI => {
                                let tmp_res = self.get_result_value(node);

                                assert!(
                                    self.match_fpreg(op),
                                    "unexpected op (expected fpreg): {}",
                                    op
                                );
                                let tmp_op = self.emit_fpreg(op, f_content, f_context, vm);

                                let to_ty_size = vm.get_backend_type_size(to_ty.id());
                                match to_ty_size {
                                    1 | 2 | 4 | 8 => {
                                        match from_ty.v {
                                            MuType_::Double => {
                                                self.backend.emit_cvtsd2si_r_f64(&tmp_res, &tmp_op)
                                            }
                                            MuType_::Float => {
                                                self.backend.emit_cvtss2si_r_f32(&tmp_res, &tmp_op)
                                            }
                                            _ => {
                                                panic!(
                                                    "expected fp type as from type in FPTOSI, \
                                                     found {}",
                                                    from_ty
                                                )
                                            }
                                        }
                                    }
                                    16 => unimplemented!(),
                                    _ => {
                                        panic!(
                                            "unexpected support integer type as to_type: {}",
                                            to_ty
                                        )
                                    }
                                }

                            }
                            // unsigned integer to floating point
                            op::ConvOp::UITOFP => {
                                let tmp_res = self.get_result_value(node);

                                if self.match_ireg(op) {
                                    let tmp_op = self.emit_ireg(op, f_content, f_context, vm);

                                    let op_ty_size = vm.get_backend_type_size(tmp_op.ty.id());

                                    if to_ty.is_double() {
                                        match op_ty_size {
                                            8 => {
                                                // movd/movq op -> res
                                                self.backend.emit_mov_fpr_r64(&tmp_res, &tmp_op);

                                                // punpckldq UITOFP_C0, tmp_res -> tmp_res
                                                // (interleaving low bytes:
                                                // xmm = xmm[0] mem[0] xmm[1] mem[1])
                                                let mem_c0 = self.get_mem_for_const(&UITOFP_C0, vm);
                                                self.backend
                                                    .emit_punpckldq_f64_mem128(&tmp_res, &mem_c0);

                                                // subpd UITOFP_C1, tmp_res -> tmp_res
                                                let mem_c1 = self.get_mem_for_const(&UITOFP_C1, vm);
                                                self.backend
                                                    .emit_subpd_f64_mem128(&tmp_res, &mem_c1);

                                                // haddpd tmp_res, tmp_res -> tmp_res
                                                self.backend
                                                    .emit_haddpd_f64_f64(&tmp_res, &tmp_res);
                                            }
                                            4 => {
                                                let tmp = self.make_temporary(
                                                    f_context,
                                                    UINT64_TYPE.clone(),
                                                    vm
                                                );

                                                // movl op -> tmp(32)
                                                let tmp32 =
                                                    unsafe { tmp.as_type(UINT32_TYPE.clone()) };
                                                self.backend.emit_mov_r_r(&tmp32, &tmp_op);

                                                // cvtsi2sd %tmp(64) -> %tmp_res
                                                self.backend.emit_cvtsi2sd_f64_r(&tmp_res, &tmp);
                                            }
                                            2 | 1 => {
                                                let tmp_op32 =
                                                    unsafe { tmp_op.as_type(UINT32_TYPE.clone()) };

                                                // apply mask, otherwise higher bits are arbitrary
                                                self.emit_apply_mask(
                                                    &tmp_op32,
                                                    op_ty_size * 8,
                                                    f_context,
                                                    vm
                                                );

                                                self.backend
                                                    .emit_cvtsi2sd_f64_r(&tmp_res, &tmp_op32);
                                            }
                                            _ => {
                                                panic!("not implemented int length {}", op_ty_size)
                                            }
                                        }
                                    } else if to_ty.is_float() {
                                        match op_ty_size {
                                            8 => {
                                                // movl %tmp_op -> %tmp1
                                                let tmp1 = self.make_temporary(
                                                    f_context,
                                                    UINT32_TYPE.clone(),
                                                    vm
                                                );
                                                self.backend.emit_mov_r_r(&tmp1, unsafe {
                                                    &tmp_op.as_type(UINT32_TYPE.clone())
                                                });

                                                // andl %tmp1 $1 -> %tmp1
                                                self.backend.emit_and_r_imm(&tmp1, 1);

                                                // testq %tmp_op %tmp_op
                                                self.backend.emit_test_r_r(&tmp_op, &tmp_op);

                                                let blk_if_signed = make_block_name(
                                                    &node.name(),
                                                    "uitofp_float_if_signed"
                                                );
                                                let blk_if_not_signed = make_block_name(
                                                    &node.name(),
                                                    "uitofp_float_if_not_signed"
                                                );
                                                let blk_done = make_block_name(
                                                    &node.name(),
                                                    "uitofp_float_done"
                                                );

                                                // js %if_signed
                                                self.backend.emit_js(blk_if_signed.clone());
                                                self.finish_block();

                                                // blk_if_not_signed:
                                                self.start_block(blk_if_not_signed);

                                                // cvtsi2ss %tmp_op -> %tmp_res
                                                self.backend.emit_cvtsi2ss_f32_r(&tmp_res, &tmp_op);

                                                // jmp blk_done
                                                self.backend.emit_jmp(blk_done.clone());
                                                self.finish_block();

                                                // blk_if_signed:
                                                self.start_block(blk_if_signed);

                                                // shr %tmp_op $1 -> %tmp_op
                                                self.backend.emit_shr_r_imm8(&tmp_op, 1);

                                                // or %tmp_op %tmp1 -> %tmp1
                                                self.backend.emit_or_r_r(
                                                    unsafe { &tmp1.as_type(UINT64_TYPE.clone()) },
                                                    &tmp_op
                                                );

                                                // cvtsi2ss %tmp1 -> %tmp_res
                                                self.backend.emit_cvtsi2ss_f32_r(&tmp_res, &tmp1);

                                                // addss %tmp_res %tmp_res -> %tmp_res
                                                self.backend.emit_addss_f32_f32(&tmp_res, &tmp_res);
                                                self.finish_block();

                                                self.start_block(blk_done);
                                            }
                                            4 => {
                                                // movl %tmp_op -> %tmp1
                                                let tmp1 = self.make_temporary(
                                                    f_context,
                                                    UINT32_TYPE.clone(),
                                                    vm
                                                );
                                                self.backend.emit_mov_r_r(&tmp1, &tmp_op);

                                                // cvtsi2ssq %tmp1(64) -> %tmp_res
                                                self.backend.emit_cvtsi2ss_f32_r(
                                                    &tmp_res,
                                                    unsafe { &tmp1.as_type(UINT64_TYPE.clone()) }
                                                );
                                            }
                                            2 | 1 => {
                                                let tmp_op32 =
                                                    unsafe { tmp_op.as_type(UINT32_TYPE.clone()) };

                                                // apply mask, otherwise higher bits are arbitrary
                                                self.emit_apply_mask(
                                                    &tmp_op32,
                                                    op_ty_size * 8,
                                                    f_context,
                                                    vm
                                                );

                                                // cvtsi2ss %tmp_op32 -> %tmp_res
                                                self.backend
                                                    .emit_cvtsi2ss_f32_r(&tmp_res, &tmp_op32);
                                            }
                                            _ => {
                                                panic!("not implemented int length {}", op_ty_size)
                                            }
                                        }
                                    } else {
                                        panic!("expect double or float")
                                    }
                                } else if self.match_ireg_ex(op) {
                                    unimplemented!()
                                } else {
                                    panic!("expect op to be ireg/ireg_ex, found {}", op)
                                }
                            }
                            op::ConvOp::FPTOUI => {
                                let tmp_res = self.get_result_value(node);

                                assert!(
                                    self.match_fpreg(op),
                                    "unexpected op (expected fpreg): {}",
                                    op
                                );
                                let tmp_op = self.emit_fpreg(op, f_content, f_context, vm);
                                let res_ty_size = vm.get_backend_type_size(tmp_res.ty.id());

                                if from_ty.is_double() {
                                    match res_ty_size {
                                        16 => unimplemented!(),
                                        8 => {
                                            let tmp1 = self.make_temporary(
                                                f_context,
                                                DOUBLE_TYPE.clone(),
                                                vm
                                            );
                                            let tmp2 = self.make_temporary(
                                                f_context,
                                                DOUBLE_TYPE.clone(),
                                                vm
                                            );

                                            // movsd FPTOUI_C_DOUBLE -> %tmp1
                                            let mem_c =
                                                self.get_mem_for_const(&FPTOUI_C_DOUBLE, vm);
                                            self.backend.emit_movsd_f64_mem64(&tmp1, &mem_c);

                                            // movapd %tmp_op -> %tmp2
                                            self.backend.emit_movapd_f64_f64(&tmp2, &tmp_op);

                                            // subsd %tmp1, %tmp2 -> %tmp2
                                            self.backend.emit_subsd_f64_f64(&tmp2, &tmp1);

                                            // cvttsd2si %tmp2 -> %tmp_res
                                            self.backend.emit_cvttsd2si_r_f64(&tmp_res, &tmp2);

                                            let tmp_const = self.make_temporary(
                                                f_context,
                                                UINT64_TYPE.clone(),
                                                vm
                                            );
                                            // mov 0x8000000000000000 -> %tmp_const
                                            self.backend.emit_mov_r64_imm64(
                                                &tmp_const,
                                                -9223372036854775808i64
                                            );

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
                                            let tmp_res64 =
                                                unsafe { tmp_res.as_type(UINT64_TYPE.clone()) };

                                            // cvttsd2si %tmp_op -> %tmp_res(64)
                                            self.backend.emit_cvttsd2si_r_f64(&tmp_res64, &tmp_op);
                                        }
                                        2 | 1 => {
                                            let tmp_res32 =
                                                unsafe { tmp_res.as_type(UINT32_TYPE.clone()) };

                                            // cvttsd2si %tmp_op -> %tmp_res(32)
                                            self.backend.emit_cvttsd2si_r_f64(&tmp_res32, &tmp_op);

                                            // movz %tmp_res -> %tmp_res(32)
                                            self.backend.emit_movz_r_r(&tmp_res32, &tmp_res);
                                        }
                                        _ => panic!("not implemented int length {}", res_ty_size)
                                    }
                                } else if from_ty.is_float() {
                                    match res_ty_size {
                                        16 => unimplemented!(),
                                        8 => {
                                            let tmp1 = self.make_temporary(
                                                f_context,
                                                FLOAT_TYPE.clone(),
                                                vm
                                            );
                                            let tmp2 = self.make_temporary(
                                                f_context,
                                                FLOAT_TYPE.clone(),
                                                vm
                                            );

                                            // movss FPTOUI_C_FLOAT -> %tmp1
                                            let mem_c = self.get_mem_for_const(&FPTOUI_C_FLOAT, vm);
                                            self.backend.emit_movss_f32_mem32(&tmp1, &mem_c);

                                            // movaps %tmp_op -> %tmp2
                                            self.backend.emit_movaps_f32_f32(&tmp2, &tmp_op);

                                            // subss %tmp1, %tmp2 -> %tmp2
                                            self.backend.emit_subss_f32_f32(&tmp2, &tmp1);

                                            // cvttss2si %tmp2 -> %tmp_res
                                            self.backend.emit_cvttss2si_r_f32(&tmp_res, &tmp2);

                                            let tmp_const = self.make_temporary(
                                                f_context,
                                                UINT64_TYPE.clone(),
                                                vm
                                            );
                                            // mov 0x8000000000000000 -> %tmp_const
                                            self.backend.emit_mov_r64_imm64(
                                                &tmp_const,
                                                -9223372036854775808i64
                                            );

                                            // xor %tmp_res, %tmp_const -> %tmp_const
                                            self.backend.emit_xor_r_r(&tmp_const, &tmp_res);

                                            // cvttss2si %tmp_op -> %tmp_res
                                            self.backend.emit_cvttss2si_r_f32(&tmp_res, &tmp_op);

                                            // ucomiss %tmp_op %tmp1
                                            self.backend.emit_ucomiss_f32_f32(&tmp1, &tmp_op);

                                            // cmovaeq %tmp_const -> %tmp_res
                                            self.backend.emit_cmovae_r_r(&tmp_res, &tmp_const);
                                        }
                                        4 => {
                                            let tmp_res64 =
                                                unsafe { tmp_res.as_type(UINT64_TYPE.clone()) };

                                            // cvttss2si %tmp_op -> %tmp_res(64)
                                            self.backend.emit_cvttss2si_r_f32(&tmp_res64, &tmp_op);
                                        }
                                        2 | 1 => {
                                            let tmp_res32 =
                                                unsafe { tmp_res.as_type(UINT32_TYPE.clone()) };

                                            // cvttss2si %tmp_op -> %tmp_res(32)
                                            self.backend.emit_cvttss2si_r_f32(&tmp_res32, &tmp_op);

                                            // movz %tmp_res(32) -> %tmp_res
                                            self.backend.emit_movz_r_r(&tmp_res32, &tmp_res);
                                        }
                                        _ => panic!("not implemented int length {}", res_ty_size)
                                    }
                                } else {
                                    panic!("expect double or float")
                                }
                            }
                            op::ConvOp::FPTRUNC => {
                                let tmp_res = self.get_result_value(node);

                                assert!(
                                    self.match_fpreg(op),
                                    "unexpected op (expected fpreg): {}",
                                    op
                                );
                                let tmp_op = self.emit_fpreg(op, f_content, f_context, vm);
                                if from_ty.is_double() && to_ty.is_float() {
                                    self.backend.emit_cvtsd2ss_f32_f64(&tmp_res, &tmp_op);
                                } else {
                                    panic!(
                                        "FPTRUNC from {} to {} is not supported \
                                         (only support FPTRUNC from double to float)",
                                        from_ty,
                                        to_ty
                                    );
                                }
                            }
                            op::ConvOp::FPEXT => {
                                let tmp_res = self.get_result_value(node);
                                assert!(
                                    self.match_fpreg(op),
                                    "unexpected op (expected fpreg): {}",
                                    op
                                );
                                let tmp_op = self.emit_fpreg(op, f_content, f_context, vm);
                                if from_ty.is_float() && to_ty.is_double() {
                                    self.backend.emit_cvtss2sd_f64_f32(&tmp_res, &tmp_op);
                                } else {
                                    panic!(
                                        "FPEXT from {} to {} is not supported\
                                         (only support FPEXT from float to double)",
                                        from_ty,
                                        to_ty
                                    );
                                }
                            }
                            op::ConvOp::BITCAST => {
                                let tmp_res = self.get_result_value(node);
                                let tmp_op = if self.match_fpreg(op) {
                                    self.emit_fpreg(op, f_content, f_context, vm)
                                } else if self.match_ireg(op) {
                                    self.emit_ireg(op, f_content, f_context, vm)
                                } else {
                                    panic!("expected op for BITCAST (expected ireg/fpreg): {}", op)
                                };

                                let ref from_ty = tmp_op.ty;
                                let ref to_ty = tmp_res.ty;

                                let from_ty_size = vm.get_backend_type_size(from_ty.id());
                                let to_ty_size = vm.get_backend_type_size(to_ty.id());
                                assert!(
                                    from_ty_size == to_ty_size,
                                    "BITCAST only works between int/fp of same length"
                                );
                                assert!(
                                    from_ty_size == 8 || from_ty_size == 4,
                                    "BITCAST only works for int32/float or int64/double"
                                );

                                if from_ty.is_fp() && to_ty.is_int() {
                                    if from_ty_size == 8 {
                                        self.backend.emit_mov_r64_fpr(&tmp_res, &tmp_op);
                                    } else if from_ty_size == 4 {
                                        self.backend.emit_mov_r32_fpr(&tmp_res, &tmp_op);
                                    } else {
                                        unreachable!()
                                    }
                                } else if from_ty.is_int() && to_ty.is_fp() {
                                    if from_ty_size == 8 {
                                        self.backend.emit_mov_fpr_r64(&tmp_res, &tmp_op);
                                    } else if from_ty_size == 4 {
                                        self.backend.emit_mov_fpr_r32(&tmp_res, &tmp_op);
                                    } else {
                                        unreachable!()
                                    }
                                } else {
                                    panic!(
                                        "expected BITCAST between int and fp,\
                                         found {} and {}",
                                        from_ty,
                                        to_ty
                                    )
                                }
                            }
                        }
                    }

                    // load on x64 generates mov inst (no matter what order is specified)
                    // https://www.cl.cam.ac.uk/~pes20/cpp/cpp0xmappings.html
                    Instruction_::Load { order, mem_loc, .. } => {
                        trace!("instsel on LOAD");

                        let ref ops = inst.ops;
                        let ref loc_op = ops[mem_loc];

                        // check allowed order for LOAD
                        match order {
                            MemoryOrder::Relaxed |
                            MemoryOrder::Consume |
                            MemoryOrder::Acquire |
                            MemoryOrder::SeqCst |
                            MemoryOrder::NotAtomic => {}
                            _ => panic!("unsupported order {:?} for LOAD", order)
                        }

                        let resolved_loc =
                            self.emit_node_addr_to_value(loc_op, f_content, f_context, vm);
                        let res_temp = self.get_result_value(node);

                        if self.match_ireg(node) {
                            self.backend.emit_mov_r_mem(&res_temp, &resolved_loc);
                        } else if self.match_ireg_ex(node) {
                            // FIXME: this load is not atomic, check memory order
                            let (res_l, res_h) = self.split_int128(&res_temp, f_context, vm);

                            // load lower half
                            self.backend.emit_mov_r_mem(&res_l, &resolved_loc);

                            // shift ptr, and load higher half
                            let loc = self.addr_const_offset_adjust(
                                resolved_loc.extract_memory_location().unwrap(),
                                POINTER_SIZE as u64,
                                vm
                            );
                            let val_loc = self.make_memory_from_location(loc, vm);
                            self.backend.emit_mov_r_mem(&res_h, &val_loc);
                        } else if self.match_fpreg(node) {
                            match res_temp.ty.v {
                                MuType_::Double => {
                                    self.backend.emit_movsd_f64_mem64(&res_temp, &resolved_loc)
                                }
                                MuType_::Float => {
                                    self.backend.emit_movss_f32_mem32(&res_temp, &resolved_loc)
                                }
                                _ => panic!("expect double or float")
                            }
                        } else {
                            // load other types
                            unimplemented!()
                        }
                    }

                    Instruction_::Store {
                        order,
                        mem_loc,
                        value,
                        ..
                    } => {
                        trace!("instsel on STORE");

                        let ref ops = inst.ops;
                        let ref loc_op = ops[mem_loc];
                        let ref val_op = ops[value];

                        // need mfence after the store? we need mfence for SeqCst store
                        let need_fence: bool = {
                            match order {
                                MemoryOrder::Relaxed |
                                MemoryOrder::Release |
                                MemoryOrder::NotAtomic => false,
                                MemoryOrder::SeqCst => true,
                                _ => panic!("unsupported order {:?} for STORE", order)
                            }
                        };

                        let resolved_loc =
                            self.emit_node_addr_to_value(loc_op, f_content, f_context, vm);

                        // emit store
                        if self.match_iimm(val_op) {
                            let (val, len) = self.node_iimm_to_i32_with_len(val_op);
                            self.backend.emit_mov_mem_imm(&resolved_loc, val, len);
                        } else if self.match_ireg(val_op) {
                            let val = self.emit_ireg(val_op, f_content, f_context, vm);
                            self.backend.emit_mov_mem_r(&resolved_loc, &val);
                        } else if self.match_ireg_ex(val_op) {
                            let (val_l, val_h) =
                                self.emit_ireg_ex(val_op, f_content, f_context, vm);
                            // store lower half
                            self.backend.emit_mov_mem_r(&resolved_loc, &val_l);

                            // shift pointer, and store higher hal
                            let loc = self.addr_const_offset_adjust(
                                resolved_loc.extract_memory_location().unwrap(),
                                POINTER_SIZE as u64,
                                vm
                            );
                            let loc_val = self.make_memory_from_location(loc, vm);

                            self.backend.emit_mov_mem_r(&loc_val, &val_h);
                        } else if self.match_fpreg(val_op) {
                            let val = self.emit_fpreg(val_op, f_content, f_context, vm);

                            match val.ty.v {
                                MuType_::Double => {
                                    self.backend.emit_movsd_mem64_f64(&resolved_loc, &val)
                                }
                                MuType_::Float => {
                                    self.backend.emit_movss_mem32_f32(&resolved_loc, &val)
                                }
                                _ => panic!("unexpected fp type: {}", val.ty)
                            }
                        } else {
                            // store other types
                            unimplemented!()
                        }

                        if need_fence {
                            self.backend.emit_mfence();
                        }
                    }

                    Instruction_::Fence(order) => {
                        trace!("instsel on FENCE");

                        // check order
                        match order {
                            MemoryOrder::Consume |
                            MemoryOrder::Acquire |
                            MemoryOrder::Release |
                            MemoryOrder::AcqRel => {
                                // ignore
                            }
                            MemoryOrder::SeqCst => {
                                self.backend.emit_mfence();
                            }
                            _ => panic!("unsupported order {:?} for FENCE")
                        }
                    }

                    // memory insts: calculate the address, then lea
                    Instruction_::GetIRef(_) |
                    Instruction_::GetFieldIRef { .. } |
                    Instruction_::GetVarPartIRef { .. } |
                    Instruction_::ShiftIRef { .. } |
                    Instruction_::GetElementIRef { .. } => {
                        // if we reach here, it means we want to store the address in a variable
                        trace!("instsel on GET FIELD/VARPART/ELEM IREF, SHIFTIREF");

                        let mem_addr = self.emit_inst_addr_to_value(node, f_content, f_context, vm);
                        let tmp_res = self.get_result_value(node);

                        self.backend.emit_lea_r64(&tmp_res, &mem_addr);
                    }

                    Instruction_::ThreadExit => {
                        trace!("instsel on THREADEXIT");

                        // get thread local and add offset to get sp_loc
                        let tl = self.emit_get_threadlocal(Some(node), f_content, f_context, vm);
                        self.emit_load_base_offset(
                            &tl,
                            &tl,
                            *thread::NATIVE_SP_LOC_OFFSET as i32,
                            vm
                        );

                        // emit a call to swap_back_to_native_stack(sp_loc: Address)
                        self.emit_runtime_entry(
                            &entrypoints::THREAD_EXIT,
                            vec![tl.clone()],
                            None,
                            Some(node),
                            f_content,
                            f_context,
                            vm
                        );
                    }

                    Instruction_::CommonInst_GetThreadLocal => {
                        trace!("instsel on GETTHREADLOCAL");
                        // get thread local
                        let tl = self.emit_get_threadlocal(Some(node), f_content, f_context, vm);

                        // load [tl + USER_TLS_OFFSET] -> tmp_res
                        let tmp_res = self.get_result_value(node);
                        self.emit_load_base_offset(
                            &tmp_res,
                            &tl,
                            *thread::USER_TLS_OFFSET as i32,
                            vm
                        );
                    }
                    Instruction_::CommonInst_SetThreadLocal(op) => {
                        trace!("instsel on SETTHREADLOCAL");

                        let ref ops = inst.ops;
                        let ref op = ops[op];

                        assert!(self.match_ireg(op));
                        let tmp_op = self.emit_ireg(op, f_content, f_context, vm);

                        // get thread local
                        let tl = self.emit_get_threadlocal(Some(node), f_content, f_context, vm);

                        // store tmp_op -> [tl + USER_TLS_OFFSET]
                        self.emit_store_base_offset(
                            &tl,
                            *thread::USER_TLS_OFFSET as i32,
                            &tmp_op,
                            vm
                        );
                    }

                    // FIXME: the semantic of Pin/Unpin is different from spec
                    // See Issue #33
                    Instruction_::CommonInst_Pin(op) => {
                        use runtime::mm::GC_MOVES_OBJECT;
                        trace!("instsel on PIN");

                        // call pin() in GC
                        let ref ops = inst.ops;
                        let ref op = ops[op];

                        assert!(self.match_ireg(op));
                        let tmp_op = self.emit_ireg(op, f_content, f_context, vm);
                        let tmp_res = self.get_result_value(node);

                        if GC_MOVES_OBJECT {
                            self.emit_runtime_entry(
                                &entrypoints::PIN_OBJECT,
                                vec![tmp_op.clone()],
                                Some(vec![tmp_res]),
                                Some(node),
                                f_content,
                                f_context,
                                vm
                            );
                        } else {
                            // FIXME: this is problematic, as we are not keeping the object alive
                            self.backend.emit_mov_r_r(&tmp_res, &tmp_op);
                        }
                    }
                    Instruction_::CommonInst_Unpin(op) => {
                        use runtime::mm::GC_MOVES_OBJECT;
                        trace!("instsel on UNPIN");

                        // call unpin() in GC
                        let ref ops = inst.ops;
                        let ref op = ops[op];

                        assert!(self.match_ireg(op));
                        let tmp_op = self.emit_ireg(op, f_content, f_context, vm);

                        if GC_MOVES_OBJECT {
                            self.emit_runtime_entry(
                                &entrypoints::UNPIN_OBJECT,
                                vec![tmp_op.clone()],
                                None,
                                Some(node),
                                f_content,
                                f_context,
                                vm
                            );
                        }
                    }
                    Instruction_::CommonInst_GetAddr(op) => {
                        trace!("instsel on GETADDR");

                        // assume it is pinned
                        let ref op = inst.ops[op];
                        assert!(self.match_ireg(op));
                        let tmp_op = self.emit_ireg(op, f_content, f_context, vm);
                        let tmp_res = self.get_result_value(node);

                        self.emit_move_value_to_value(&tmp_res, &tmp_op);
                    }

                    Instruction_::Move(op) => {
                        trace!("instsel on MOVE (internal IR)");

                        let ref ops = inst.ops;
                        let ref op = ops[op];
                        let tmp_res = self.get_result_value(node);

                        self.emit_move_node_to_value(&tmp_res, op, f_content, f_context, vm);
                    }

                    Instruction_::New(ref ty) => {
                        trace!("instsel on NEW");
                        assert!(!ty.is_hybrid());

                        let ty_info = vm.get_backend_type_info(ty.id());
                        let size = ty_info.size;
                        let ty_align = ty_info.alignment;

                        // get allocator
                        let tmp_allocator = self.emit_get_allocator(node, f_content, f_context, vm);
                        // allocate and init
                        self.emit_alloc_const_size(
                            &tmp_allocator,
                            size,
                            ty_align,
                            node,
                            &ty_info,
                            f_content,
                            f_context,
                            vm
                        );
                    }

                    Instruction_::NewHybrid(ref ty, var_len) => {
                        trace!("instsel on NEWHYBRID");
                        assert!(ty.is_hybrid());

                        let ty_info = vm.get_backend_type_info(ty.id());
                        let ty_align = ty_info.alignment;
                        let fix_part_size = ty_info.size;
                        let var_ty_size = match ty_info.elem_size {
                            Some(sz) => sz,
                            None => {
                                panic!("expect HYBRID type here with elem_size, found {}", ty_info)
                            }
                        };

                        let ref ops = inst.ops;
                        let ref op_var_len = ops[var_len];
                        let tmp_allocator = self.emit_get_allocator(node, f_content, f_context, vm);
                        // size is known at compile time
                        if self.match_iconst_any(op_var_len) {
                            let const_var_len = op_var_len.as_value().extract_int_const().unwrap();
                            let const_size = mm::check_hybrid_size(
                                fix_part_size + var_ty_size * (const_var_len as usize)
                            );
                            self.emit_alloc_const_size(
                                &tmp_allocator,
                                const_size,
                                ty_align,
                                node,
                                &ty_info,
                                f_content,
                                f_context,
                                vm
                            );
                        } else {
                            debug_assert!(self.match_ireg(op_var_len));
                            let tmp_var_len = op_var_len.as_value().clone();

                            let tmp_fix_size = self.make_int64_const(fix_part_size as u64, vm);
                            let tmp_var_size = self.make_int64_const(var_ty_size as u64, vm);
                            let tmp_align = self.make_int64_const(ty_align as u64, vm);
                            let tmp_tyid = {
                                let enc = ty_info.gc_type.as_ref().unwrap();
                                let id = vm.get_gc_type_id(enc);
                                self.make_int64_const(id as u64, vm)
                            };
                            let tmp_full_tyid = {
                                let enc = ty_info.gc_type_hybrid_full.as_ref().unwrap();
                                let id = vm.get_gc_type_id(enc);
                                self.make_int64_const(id as u64, vm)
                            };
                            let tmp_res = self.get_result_value(node);
                            self.emit_runtime_entry(
                                &entrypoints::ALLOC_VAR_SIZE,
                                vec![
                                    tmp_fix_size,
                                    tmp_var_size,
                                    tmp_var_len,
                                    tmp_align,
                                    tmp_tyid,
                                    tmp_full_tyid,
                                ],
                                Some(vec![tmp_res].clone()),
                                Some(node),
                                f_content,
                                f_context,
                                vm
                            );
                        }
                    }

                    /*Instruction_::AllocA(ref ty) => {
                        trace!("instsel on AllocA");

                        if cfg!(debug_assertions) {
                            match ty.v {
                                MuType_::Hybrid(_) => panic!("cannot use ALLOCA for hybrid,\
                                                             use ALLOCAHYBRID instead"),
                                _ => {}
                            }
                        }

                        let ty_info = vm.get_backend_type_info(ty.id());
                        let size = ty_info.size;
                        let ty_align= ty_info.alignment;
                        if 16 % ty_align != 0 {
                            // It's not trivial to allign this type...
                            unimplemented!()
                        }
                        // Round size up to the nearest multiple of 16
                        let size = ((size + 16 - 1)/16)*16;
                    }*/
                    Instruction_::Throw(op_index) => {
                        trace!("instsel on THROW");

                        let ref ops = inst.ops;
                        let ref exception_obj = ops[op_index];

                        self.emit_runtime_entry(
                            &entrypoints::THROW_EXCEPTION,
                            vec![exception_obj.clone_value()],
                            None,
                            Some(node),
                            f_content,
                            f_context,
                            vm
                        );
                    }

                    Instruction_::NewStack(func) => {
                        trace!("instsel on NEWSTACK");

                        let ref ops = inst.ops;
                        let ref func = ops[func];

                        let tmp_res = self.get_result_value(node);
                        let tmp_func = self.emit_ireg(func, f_content, f_context, vm);

                        let sig = match tmp_func.ty.v {
                            MuType_::FuncRef(ref sig) => sig.clone(),
                            _ => panic!("expected funcref")
                        };

                        let tmp_stack_arg_size = {
                            use compiler::backend::x86_64::callconv::swapstack;
                            let (size, _) = swapstack::compute_stack_args(&sig.arg_tys, vm);
                            self.make_int64_const(size as u64, vm)
                        };

                        self.emit_runtime_entry(
                            &entrypoints::NEW_STACK,
                            vec![tmp_func, tmp_stack_arg_size],
                            Some(vec![tmp_res]),
                            Some(node),
                            f_content,
                            f_context,
                            vm
                        );
                    }

                    Instruction_::NewThread {
                        stack,
                        thread_local,
                        is_exception,
                        ref args
                    } => {
                        trace!("instsel on NEWTHREAD");

                        let ref ops = inst.ops;
                        let res = self.get_result_value(node);
                        let stack = self.emit_ireg(&ops[stack], f_content, f_context, vm);
                        let tl = match thread_local {
                            Some(tl) => self.emit_ireg(&ops[tl], f_content, f_context, vm),
                            None => self.make_nullref(vm)
                        };

                        if is_exception {
                            let exc = self.emit_ireg(&ops[args[0]], f_content, f_context, vm);
                            self.emit_runtime_entry(
                                &entrypoints::NEW_THREAD_EXCEPTIONAL,
                                vec![stack, tl, exc],
                                Some(vec![res]),
                                Some(node),
                                f_content,
                                f_context,
                                vm
                            );
                        } else {
                            let new_sp = self.make_temporary(f_context, ADDRESS_TYPE.clone(), vm);
                            self.emit_load_base_offset(
                                &new_sp,
                                &stack,
                                *thread::MUSTACK_SP_OFFSET as i32,
                                vm
                            );

                            // prepare arguments on the new stack in the generated code
                            // check thread::MuStack::setup_args() for how we do it in the runtime
                            //
                            // 1. the stack arguments will be put to a reserved location during
                            //    MuStack::new(), it is from (new_sp - 2*POINTER_SIZE) to
                            //    (new_sp - 2*POINTER_SIZE - stack_arg_size)
                            // 2. the register arguments will be pushed to current SP, the start
                            //    function will consume them.
                            {
                                use compiler::backend::x86_64::callconv::swapstack;
                                use compiler::backend::x86_64::callconv::CallConvResult;
                                use compiler::backend::x86_64::{ARGUMENT_GPRS, ARGUMENT_FPRS};

                                let arg_values =
                                    self.process_arguments(&args, ops, f_content, f_context, vm);

                                // compute call convention
                                let arg_tys = arg_values.iter().map(|x| x.ty.clone()).collect();
                                let callconv = swapstack::compute_arguments(&arg_tys);

                                let mut gpr_args = vec![];
                                let mut fpr_args = vec![];
                                let mut stack_args = vec![];

                                for i in 0..callconv.len() {
                                    let ref arg = arg_values[i];
                                    let ref cc = callconv[i];

                                    match cc {
                                        &CallConvResult::GPR(_) => gpr_args.push(arg.clone()),
                                        &CallConvResult::GPREX(_, _) => {
                                            let (arg_l, arg_h) =
                                                self.split_int128(arg, f_context, vm);
                                            gpr_args.push(arg_l);
                                            gpr_args.push(arg_h);
                                        }
                                        &CallConvResult::FPR(_) => fpr_args.push(arg.clone()),
                                        &CallConvResult::STACK => stack_args.push(arg.clone())
                                    }
                                }

                                // for arguments that are not used, we push a 0
                                let zero = self.make_int64_const(0, vm);
                                let mut word_pushed = 0;
                                for i in 0..ARGUMENT_FPRS.len() {
                                    let val = {
                                        if i < fpr_args.len() {
                                            &fpr_args[i]
                                        } else {
                                            &zero
                                        }
                                    };
                                    word_pushed += 1;
                                    self.emit_store_base_offset(
                                        &new_sp,
                                        -(word_pushed * WORD_SIZE as i32),
                                        val,
                                        vm
                                    );
                                }
                                for i in 0..ARGUMENT_GPRS.len() {
                                    let val = {
                                        if i < gpr_args.len() {
                                            &gpr_args[i]
                                        } else {
                                            &zero
                                        }
                                    };
                                    word_pushed += 1;
                                    self.emit_store_base_offset(
                                        &new_sp,
                                        -(word_pushed * WORD_SIZE as i32),
                                        val,
                                        vm
                                    );
                                }

                                if !stack_args.is_empty() {
                                    // need to put stack arguments to the preserved space
                                    self.emit_store_stack_values(
                                        &stack_args,
                                        Some((&new_sp, 2 * WORD_SIZE as i32)),
                                        MU_CALL_CONVENTION,
                                        vm
                                    );
                                }

                                // adjust sp - we have pushed all argument registers
                                // (some could be 0 though)
                                self.backend
                                    .emit_sub_r_imm(&new_sp, word_pushed * WORD_SIZE as i32);
                                // store the sp back to MuStack
                                self.emit_store_base_offset(
                                    &stack,
                                    *thread::MUSTACK_SP_OFFSET as i32,
                                    &new_sp,
                                    vm
                                );

                                // call runtime entry
                                self.emit_runtime_entry(
                                    &entrypoints::NEW_THREAD_NORMAL,
                                    vec![stack, tl],
                                    Some(vec![res.clone()]),
                                    Some(node),
                                    f_content,
                                    f_context,
                                    vm
                                );
                            }
                        }
                    }

                    Instruction_::CurrentStack => {
                        trace!("instsel on CURRENT_STACK");

                        // get thread local
                        let tl = self.emit_get_threadlocal(Some(node), f_content, f_context, vm);
                        let tmp_res = self.get_result_value(node);

                        self.emit_load_base_offset(&tmp_res, &tl, *thread::STACK_OFFSET as i32, vm);
                    }

                    Instruction_::KillStack(op) => {
                        trace!("instsel on KILL_STACK");

                        let op = self.emit_ireg(&inst.ops[op], f_content, f_context, vm);
                        self.emit_runtime_entry(
                            &entrypoints::KILL_STACK,
                            vec![op],
                            None,
                            Some(node),
                            f_content,
                            f_context,
                            vm
                        );
                    }

                    Instruction_::SwapStackExpr {
                        stack,
                        is_exception,
                        ref args
                    } => {
                        trace!("instsel on SWAPSTACK_EXPR");
                        self.emit_swapstack(
                            is_exception,
                            false,
                            &node,
                            &inst,
                            stack,
                            args,
                            None,
                            f_content,
                            f_context,
                            vm
                        );
                    }
                    Instruction_::SwapStackExc {
                        stack,
                        is_exception,
                        ref args,
                        ref resume
                    } => {
                        trace!("instsel on SWAPSTACK_EXC");
                        self.emit_swapstack(
                            is_exception,
                            false,
                            &node,
                            &inst,
                            stack,
                            args,
                            Some(resume),
                            f_content,
                            f_context,
                            vm
                        );
                    }
                    Instruction_::SwapStackKill {
                        stack,
                        is_exception,
                        ref args
                    } => {
                        trace!("instsel on SWAPSTACK_KILL");
                        self.emit_swapstack(
                            is_exception,
                            true,
                            &node,
                            &inst,
                            stack,
                            args,
                            None,
                            f_content,
                            f_context,
                            vm
                        );
                    }

                    Instruction_::PrintHex(index) => {
                        trace!("instsel on PRINTHEX");

                        let ref ops = inst.ops;
                        let ref op = ops[index];

                        self.emit_runtime_entry(
                            &entrypoints::PRINT_HEX,
                            vec![op.clone_value()],
                            None,
                            Some(node),
                            f_content,
                            f_context,
                            vm
                        );
                    }

                    Instruction_::SetRetval(index) => {
                        trace!("instsel on SETRETVAL");

                        let ref ops = inst.ops;
                        let ref op = ops[index];

                        assert!(self.match_ireg(op));
                        let retval = self.emit_ireg(op, f_content, f_context, vm);

                        self.emit_runtime_entry(
                            &entrypoints::SET_RETVAL,
                            vec![retval],
                            None,
                            Some(node),
                            f_content,
                            f_context,
                            vm
                        );
                    }

                    _ => unimplemented!()
                } // main switch
            }

            TreeNode_::Value(_) => {
                // we recursively call instruction_select for all the nodes (and their children)
                // so this pattern will be reached
                // however we do not need to do anything for a Value node
            }
        }
    }

    /// makes a temporary P<Value> of given type
    fn make_temporary(
        &mut self,
        f_context: &mut FunctionContext,
        ty: P<MuType>,
        vm: &VM
    ) -> P<Value> {
        f_context.make_temporary(vm.next_id(), ty).clone_value()
    }

    /// makes a memory operand P<Value> for [base + offset]
    fn make_memory_op_base_offset(
        &mut self,
        base: &P<Value>,
        offset: i32,
        ty: P<MuType>,
        vm: &VM
    ) -> P<Value> {
        P(Value {
            hdr: MuEntityHeader::unnamed(vm.next_id()),
            ty: ty.clone(),
            v: Value_::Memory(MemoryLocation::Address {
                base: base.clone(),
                offset: Some(self.make_int64_const(offset as u64, vm)),
                index: None,
                scale: None
            })
        })
    }

    /// makes a memory operand P<Value> for [base + index * scale]
    fn make_memory_op_base_index(
        &mut self,
        base: &P<Value>,
        index: &P<Value>,
        scale: u8,
        ty: P<MuType>,
        vm: &VM
    ) -> P<Value> {
        P(Value {
            hdr: MuEntityHeader::unnamed(vm.next_id()),
            ty: ty.clone(),
            v: Value_::Memory(MemoryLocation::Address {
                base: base.clone(),
                offset: None,
                index: Some(index.clone()),
                scale: Some(scale)
            })
        })
    }

    /// makes a symbolic memory operand for global values
    fn make_memory_symbolic_global(
        &mut self,
        name: MuName,
        ty: P<MuType>,
        f_context: &mut FunctionContext,
        vm: &VM
    ) -> P<Value> {
        self.make_memory_symbolic(name, ty, true, false, f_context, vm)
    }

    /// makes a symbolic memory operand for native values
    fn make_memory_symbolic_native(
        &mut self,
        name: MuName,
        ty: P<MuType>,
        f_context: &mut FunctionContext,
        vm: &VM
    ) -> P<Value> {
        self.make_memory_symbolic(name, ty, false, true, f_context, vm)
    }

    /// makes a symbolic memory operand for a normal value (not global, not native)
    fn make_memory_symbolic_normal(
        &mut self,
        name: MuName,
        ty: P<MuType>,
        f_context: &mut FunctionContext,
        vm: &VM
    ) -> P<Value> {
        self.make_memory_symbolic(name, ty, false, false, f_context, vm)
    }

    /// makes a symbolic memory operand
    fn make_memory_symbolic(
        &mut self,
        name: MuName,
        ty: P<MuType>,
        is_global: bool,
        is_native: bool,
        f_context: &mut FunctionContext,
        vm: &VM
    ) -> P<Value> {
        if cfg!(feature = "sel4-rumprun") {
            // Same as Linux:
            // for a(%RIP), we need to load its address from a@GOTPCREL(%RIP)
            // then load from the address.
            // asm_backend will emit a@GOTPCREL(%RIP) for a(%RIP)
            let got_loc = P(Value {
                hdr: MuEntityHeader::unnamed(vm.next_id()),
                ty: ADDRESS_TYPE.clone(),
                v: Value_::Memory(MemoryLocation::Symbolic {
                    base: Some(x86_64::RIP.clone()),
                    label: name,
                    is_global: is_global,
                    is_native: is_native
                })
            });

            // mov (got_loc) -> actual_loc
            let actual_loc = self.make_temporary(f_context, ADDRESS_TYPE.clone(), vm);
            self.emit_move_value_to_value(&actual_loc, &got_loc);

            self.make_memory_op_base_offset(&actual_loc, 0, ty, vm)
        } else if cfg!(target_os = "macos") {
            P(Value {
                hdr: MuEntityHeader::unnamed(vm.next_id()),
                ty: ty,
                v: Value_::Memory(MemoryLocation::Symbolic {
                    base: Some(x86_64::RIP.clone()),
                    label: name,
                    is_global: is_global,
                    is_native: is_native
                })
            })
        } else if cfg!(target_os = "linux") {
            // for a global: a(%RIP), we need to load its address from a@GOTPCREL(%RIP)
            // then load from the address.
            // asm_backend will emit a@GOTPCREL(%RIP) for a(%RIP)
            let symbol_loc = P(Value {
                hdr: MuEntityHeader::unnamed(vm.next_id()),
                ty: ADDRESS_TYPE.clone(),
                v: Value_::Memory(MemoryLocation::Symbolic {
                    base: Some(x86_64::RIP.clone()),
                    label: name,
                    is_global: is_global,
                    is_native: is_native
                })
            });

            if is_global {
                // mov (got_loc) -> actual_loc
                let actual_loc = self.make_temporary(f_context, ADDRESS_TYPE.clone(), vm);
                self.emit_move_value_to_value(&actual_loc, &symbol_loc);

                self.make_memory_op_base_offset(&actual_loc, 0, ty, vm)
            } else {
                symbol_loc
            }
        } else {
            panic!("unsupported OS")
        }
    }

    /// makes a memory operand P<Value> from MemoryLocation
    fn make_memory_from_location(&mut self, loc: MemoryLocation, vm: &VM) -> P<Value> {
        P(Value {
            hdr: MuEntityHeader::unnamed(vm.next_id()),
            ty: ADDRESS_TYPE.clone(),
            v: Value_::Memory(loc)
        })
    }

    /// makes an integer constant P<Value>
    fn make_int64_const(&mut self, val: u64, vm: &VM) -> P<Value> {
        P(Value {
            hdr: MuEntityHeader::unnamed(vm.next_id()),
            ty: UINT64_TYPE.clone(),
            v: Value_::Constant(Constant::Int(val))
        })
    }

    /// makes an integer constant P<Value>
    fn make_int_const(&mut self, val: u64, ty: P<MuType>, vm: &VM) -> P<Value> {
        P(Value {
            hdr: MuEntityHeader::unnamed(vm.next_id()),
            ty,
            v: Value_::Constant(Constant::Int(val))
        })
    }

    /// makes a refvoid-typed null ref
    fn make_nullref(&mut self, vm: &VM) -> P<Value> {
        P(Value {
            hdr: MuEntityHeader::unnamed(vm.next_id()),
            ty: REF_VOID_TYPE.clone(),
            v: Value_::Constant(Constant::NullRef)
        })
    }

    /// emits code for binary operations
    fn emit_binop(
        &mut self,
        node: &TreeNode,
        inst: &Instruction,
        op: BinOp,
        mut op1: OpIndex,
        mut op2: OpIndex,
        f_content: &FunctionContent,
        f_context: &mut FunctionContext,
        vm: &VM
    ) {
        let ref ops = inst.ops;

        {
            // symmetric operators, we want to make sure that if any of the operands
            // will be treated specially, it is going to be op2.
            // so we check op1, if it is special, we swap them

            let ref node_op1 = ops[op1];

            let mut swap_operands = || {
                let t = op1;
                op1 = op2;
                op2 = t;
            };

            match op {
                op::BinOp::Add | op::BinOp::And | op::BinOp::Or | op::BinOp::Xor |
                op::BinOp::Mul => {
                    if self.match_iconst_zero(node_op1) || self.match_iconst_one(node_op1) ||
                        self.match_iimm(node_op1) ||
                        self.match_mem(node_op1)
                    {
                        swap_operands();
                    }
                }
                op::BinOp::FAdd | op::BinOp::FMul => {
                    if self.match_fconst_zero(node_op1) || self.match_mem(node_op1) {
                        swap_operands();
                    }
                }
                _ => {}
            }
        }

        self.emit_binop_internal(node, inst, op, op1, op2, f_content, f_context, vm)
    }

    /// emits code for binary operations with the assumption that op2 may be special
    fn emit_binop_internal(
        &mut self,
        node: &TreeNode,
        inst: &Instruction,
        op: BinOp,
        op1: OpIndex,
        op2: OpIndex,
        f_content: &FunctionContent,
        f_context: &mut FunctionContext,
        vm: &VM
    ) {
        let ref ops = inst.ops;

        let res_tmp = self.get_result_value(node);
        let ref op1 = ops[op1];
        let ref op2 = ops[op2];

        match op {
            op::BinOp::Add => {
                if self.match_ireg(op1) && self.match_iconst_zero(op2) {
                    // add zero is nop
                    trace!("emit add-ireg-0");

                    self.emit_move_node_to_value(&res_tmp, op1, f_content, f_context, vm);
                } else if self.match_ireg(op1) && self.match_iimm(op2) {
                    trace!("emit add-ireg-imm");

                    let reg_op1 = self.emit_ireg(op1, f_content, f_context, vm);
                    let reg_op2 = self.node_iimm_to_i32(op2);

                    // mov op1, res
                    self.backend.emit_mov_r_r(&res_tmp, &reg_op1);
                    // add op2, res
                    self.backend.emit_add_r_imm(&res_tmp, reg_op2);
                } else if self.match_ireg(op1) && self.match_mem(op2) {
                    trace!("emit add-ireg-mem");

                    let reg_op1 = self.emit_ireg(op1, f_content, f_context, vm);
                    let reg_op2 = self.emit_mem(op2, f_content, f_context, vm);

                    // mov op1, res
                    self.backend.emit_mov_r_r(&res_tmp, &reg_op1);
                    // add op2 res
                    self.backend.emit_add_r_mem(&res_tmp, &reg_op2);
                } else if self.match_ireg(op1) && self.match_ireg(op2) {
                    trace!("emit add-ireg-ireg");

                    let reg_op1 = self.emit_ireg(op1, f_content, f_context, vm);
                    let reg_op2 = self.emit_ireg(op2, f_content, f_context, vm);

                    // mov op1, res
                    self.backend.emit_mov_r_r(&res_tmp, &reg_op1);
                    // add op2 res
                    self.backend.emit_add_r_r(&res_tmp, &reg_op2);
                } else if self.match_ireg_ex(op1) && self.match_iconst_zero(op2) {
                    // add one is nop
                    trace!("emit add-iregex-0");

                    self.emit_move_node_to_value(&res_tmp, op1, f_content, f_context, vm);
                } else if self.match_ireg_ex(op1) && self.match_ireg_ex(op2) {
                    trace!("emit add-iregex-iregex");

                    let (op1_l, op1_h) = self.emit_ireg_ex(op1, f_content, f_context, vm);
                    let (op2_l, op2_h) = self.emit_ireg_ex(op2, f_content, f_context, vm);

                    // make result split
                    // mov op1 to res
                    let (res_l, res_h) = self.split_int128(&res_tmp, f_context, vm);
                    self.backend.emit_mov_r_r(&res_l, &op1_l);
                    self.backend.emit_mov_r_r(&res_h, &op1_h);

                    // add res_l op2_l -> res_l
                    self.backend.emit_add_r_r(&res_l, &op2_l);

                    // adc res_h op2_h -> res_h
                    self.backend.emit_adc_r_r(&res_h, &op2_h);
                } else {
                    panic!("unexpected op for node {:?}", node)
                }
            }
            op::BinOp::Sub => {
                if self.match_ireg(op1) && self.match_iconst_zero(op2) {
                    // sub zero is nop
                    trace!("emit sub-ireg-0");

                    self.emit_move_node_to_value(&res_tmp, op1, f_content, f_context, vm);
                } else if self.match_ireg(op1) && self.match_iimm(op2) {
                    trace!("emit sub-ireg-imm");

                    let reg_op1 = self.emit_ireg(op1, f_content, f_context, vm);
                    let imm_op2 = self.node_iimm_to_i32(op2);

                    // mov op1, res
                    self.backend.emit_mov_r_r(&res_tmp, &reg_op1);
                    // sub op2, res
                    self.backend.emit_sub_r_imm(&res_tmp, imm_op2);
                } else if self.match_ireg(op1) && self.match_mem(op2) {
                    trace!("emit sub-ireg-mem");

                    let reg_op1 = self.emit_ireg(op1, f_content, f_context, vm);
                    let mem_op2 = self.emit_mem(op2, f_content, f_context, vm);

                    // mov op1, res
                    self.backend.emit_mov_r_r(&res_tmp, &reg_op1);
                    // sub op2 res
                    self.backend.emit_sub_r_mem(&res_tmp, &mem_op2);
                } else if self.match_ireg(op1) && self.match_ireg(op2) {
                    trace!("emit sub-ireg-ireg");

                    let reg_op1 = self.emit_ireg(op1, f_content, f_context, vm);
                    let reg_op2 = self.emit_ireg(op2, f_content, f_context, vm);

                    // mov op1, res
                    self.backend.emit_mov_r_r(&res_tmp, &reg_op1);
                    // add op2 res
                    self.backend.emit_sub_r_r(&res_tmp, &reg_op2);
                } else if self.match_ireg_ex(op1) && self.match_iconst_zero(op2) {
                    // sub zero is nop
                    trace!("emit sub-iregex-0");

                    self.emit_move_node_to_value(&res_tmp, op1, f_content, f_context, vm);
                } else if self.match_ireg_ex(op1) && self.match_ireg_ex(op2) {
                    trace!("emit sub-iregex-iregex");

                    let (op1_l, op1_h) = self.emit_ireg_ex(op1, f_content, f_context, vm);
                    let (op2_l, op2_h) = self.emit_ireg_ex(op2, f_content, f_context, vm);

                    // make result split
                    // mov op1 to res
                    let (res_l, res_h) = self.split_int128(&res_tmp, f_context, vm);
                    self.backend.emit_mov_r_r(&res_l, &op1_l);
                    self.backend.emit_mov_r_r(&res_h, &op1_h);

                    // sub res_l op2_l -> res_l
                    self.backend.emit_sub_r_r(&res_l, &op2_l);

                    // sbb res_h op2_h -> res_h
                    self.backend.emit_sbb_r_r(&res_h, &op2_h);
                } else {
                    panic!("unexpected op for node {:?}", node)
                }
            }
            op::BinOp::And => {
                if self.match_ireg(op1) && self.match_iconst_zero(op2) {
                    // and with zero is setting result as zero
                    trace!("emit and-ireg-0");

                    self.emit_clear_value(&res_tmp, f_context, vm);
                } else if self.match_ireg(op1) && self.match_iimm(op2) {
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
                    let mem_op2 = self.emit_mem(op2, f_content, f_context, vm);

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
                } else if self.match_ireg_ex(op1) && self.match_iconst_zero(op2) {
                    // and with zero is setting result as zero
                    trace!("emit and-iregex-0");

                    self.emit_clear_value(&res_tmp, f_context, vm);
                } else if self.match_ireg_ex(op1) && self.match_ireg_ex(op2) {
                    trace!("emit and-iregex-iregex");

                    let (op1_l, op1_h) = self.emit_ireg_ex(op1, f_content, f_context, vm);
                    let (op2_l, op2_h) = self.emit_ireg_ex(op2, f_content, f_context, vm);

                    // make result split
                    // mov op1 to res
                    let (res_l, res_h) = self.split_int128(&res_tmp, f_context, vm);
                    self.backend.emit_mov_r_r(&res_l, &op1_l);
                    self.backend.emit_mov_r_r(&res_h, &op1_h);

                    // and res_l op2_l -> res_l
                    self.backend.emit_and_r_r(&res_l, &op2_l);

                    // and res_h op2_h -> res_h
                    self.backend.emit_and_r_r(&res_h, &op2_h);
                } else {
                    panic!("unexpected op for node {:?}", node)
                }
            }
            op::BinOp::Or => {
                if self.match_ireg(op1) && self.match_iconst_zero(op2) {
                    // or zero is nop
                    trace!("emit or-ireg-0");

                    self.emit_move_node_to_value(&res_tmp, op1, f_content, f_context, vm);
                }
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
                    let mem_op2 = self.emit_mem(op2, f_content, f_context, vm);

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
                } else if self.match_ireg(op1) && self.match_iconst_zero(op2) {
                    trace!("emit or-iregex-zero");

                    self.emit_move_node_to_value(&res_tmp, op1, f_content, f_context, vm);
                } else if self.match_ireg_ex(op1) && self.match_ireg_ex(op2) {
                    trace!("emit or-iregex-iregex");

                    let (op1_l, op1_h) = self.emit_ireg_ex(op1, f_content, f_context, vm);
                    let (op2_l, op2_h) = self.emit_ireg_ex(op2, f_content, f_context, vm);

                    // make result split
                    // mov op1 to res
                    let (res_l, res_h) = self.split_int128(&res_tmp, f_context, vm);
                    self.backend.emit_mov_r_r(&res_l, &op1_l);
                    self.backend.emit_mov_r_r(&res_h, &op1_h);

                    // or res_l op2_l -> res_l
                    self.backend.emit_or_r_r(&res_l, &op2_l);

                    // or res_h op2_h -> res_h
                    self.backend.emit_or_r_r(&res_h, &op2_h);
                } else {
                    panic!("unexpected op for node {:?}", node)
                }
            }
            op::BinOp::Xor => {
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
                    let mem_op2 = self.emit_mem(op2, f_content, f_context, vm);

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
                } else if self.match_ireg_ex(op1) && self.match_ireg_ex(op2) {
                    trace!("emit xor-iregex-iregex");

                    let (op1_l, op1_h) = self.emit_ireg_ex(op1, f_content, f_context, vm);
                    let (op2_l, op2_h) = self.emit_ireg_ex(op2, f_content, f_context, vm);

                    // make result split
                    // mov op1 to res
                    let (res_l, res_h) = self.split_int128(&res_tmp, f_context, vm);
                    self.backend.emit_mov_r_r(&res_l, &op1_l);
                    self.backend.emit_mov_r_r(&res_h, &op1_h);

                    // xor res_l op2_l -> res_l
                    self.backend.emit_xor_r_r(&res_l, &op2_l);

                    // xor res_h op2_h -> res_h
                    self.backend.emit_xor_r_r(&res_h, &op2_h);
                } else {
                    panic!("unexpected op for node {:?}", node)
                }
            }
            op::BinOp::Mul => {
                // special cases
                if self.match_ireg(op1) && self.match_iconst_zero(op2) {
                    // MUL with zero is zero
                    trace!("emit mul-ireg-0");
                    self.emit_clear_value(&res_tmp, f_context, vm);
                } else if self.match_ireg(op1) && self.match_iconst_one(op2) {
                    // MUL with one is the original value
                    trace!("emit mul-ireg-1");
                    self.emit_move_node_to_value(&res_tmp, op1, f_content, f_context, vm);
                } else if self.match_ireg(op1) && self.match_iconst_p2(op2) {
                    // MUL with a constant that is a power of 2 can be done with shl
                    trace!("emit mul-ireg-p2");
                    let tmp_op1 = self.emit_ireg(op1, f_content, f_context, vm);
                    let shift = self.node_iconst_to_p2(op2);
                    self.backend.emit_mov_r_r(&res_tmp, &tmp_op1);
                    self.backend.emit_shl_r_imm8(&res_tmp, shift as i8);
                } else if self.match_ireg_ex(op1) && self.match_iconst_zero(op2) {
                    // MUL with zero is zero
                    trace!("emit mul-iregex-0");
                    self.emit_clear_value(&res_tmp, f_context, vm);
                } else if self.match_ireg_ex(op1) && self.match_iconst_one(op2) {
                    // MUL with one is the original value
                    trace!("emit mul-iregex-1");
                    self.emit_move_node_to_value(&res_tmp, op1, f_content, f_context, vm);
                } else {
                    // mov op1 -> rax
                    let op_size = vm.get_backend_type_size(op1.as_value().ty.id());

                    match op_size {
                        1 | 2 | 4 | 8 => {
                            trace!("emit mul");

                            // we need to emit both operands first, then move one into RAX
                            let tmp_op1 = self.emit_ireg(op1, f_content, f_context, vm);
                            let tmp_op2 = self.emit_ireg(op2, f_content, f_context, vm);

                            // move op1 -> RAX
                            let mreg_op1 = match op_size {
                                8 => x86_64::RAX.clone(),
                                4 => x86_64::EAX.clone(),
                                2 => x86_64::AX.clone(),
                                1 => x86_64::AL.clone(),
                                _ => unimplemented!()
                            };
                            self.backend.emit_mov_r_r(&mreg_op1, &tmp_op1);

                            // mul op2
                            self.backend.emit_mul_r(&tmp_op2);

                            // mov rax -> result
                            let res_size = vm.get_backend_type_size(res_tmp.ty.id());
                            assert!(
                                res_size == op_size,
                                "op and res do not have matching type: {}",
                                node
                            );

                            match res_size {
                                8 => self.backend.emit_mov_r_r(&res_tmp, &x86_64::RAX),
                                4 => self.backend.emit_mov_r_r(&res_tmp, &x86_64::EAX),
                                2 => self.backend.emit_mov_r_r(&res_tmp, &x86_64::AX),
                                1 => self.backend.emit_mov_r_r(&res_tmp, &x86_64::AL),
                                _ => unimplemented!()
                            }
                        }
                        16 => {
                            if self.match_ireg_ex(op1) && self.match_ireg_ex(op2) {
                                trace!("emit mul128");

                                //     (hi, lo)
                                //      a   b
                                // x    c   d
                                // ------------
                                //      ad  bd
                                //  ad  bc
                                // ------------
                                //      t1  t2
                                //     (hi, lo)

                                let (b, a) = self.emit_ireg_ex(op1, f_content, f_context, vm);
                                let (d, c) = self.emit_ireg_ex(op2, f_content, f_context, vm);

                                // mov a -> t1
                                let t1 = self.make_temporary(f_context, UINT64_TYPE.clone(), vm);
                                self.backend.emit_mov_r_r(&t1, &a);

                                // imul d, t1 -> t1
                                self.backend.emit_imul_r_r(&t1, &d);

                                // mul d, b -> (RDX:RAX) as (carry:t2)
                                self.backend.emit_mov_r_r(&x86_64::RAX, &d);
                                self.backend.emit_mul_r(&b);

                                let t2 = self.make_temporary(f_context, UINT64_TYPE.clone(), vm);
                                self.backend.emit_mov_r_r(&t2, &x86_64::RAX);

                                // add t1, carry -> t1
                                self.backend.emit_add_r_r(&t1, &x86_64::RDX);

                                // mov c -> tt
                                let tt = self.make_temporary(f_context, UINT64_TYPE.clone(), vm);
                                self.backend.emit_mov_r_r(&tt, &c);

                                // imul b, tt -> tt
                                self.backend.emit_imul_r_r(&tt, &b);

                                // add t1, tt -> t1
                                self.backend.emit_add_r_r(&t1, &tt);

                                // result: t1(higher), t2(lower)
                                let (res_l, res_h) = self.split_int128(&res_tmp, f_context, vm);
                                self.backend.emit_mov_r_r(&res_l, &t2);
                                self.backend.emit_mov_r_r(&res_h, &t1);
                            } else {
                                panic!("unexpected op for node {:?}, expect int128 MUL", node)
                            }
                        }
                        _ => panic!("unsupported int size: {}", op_size)
                    }
                }
            }
            op::BinOp::Udiv => {
                let op_size = vm.get_backend_type_size(op1.as_value().ty.id());

                match op_size {
                    1 | 2 | 4 | 8 => {
                        if self.match_iconst_p2(op2) {
                            // we can simply logic shift right
                            let shift = self.node_iconst_to_p2(op2);

                            let tmp_op1 = self.emit_ireg(op1, f_content, f_context, vm);
                            self.backend.emit_mov_r_r(&res_tmp, &tmp_op1);
                            self.backend.emit_shr_r_imm8(&res_tmp, shift as i8);
                        } else {
                            self.emit_udiv(op1, op2, f_content, f_context, vm);

                            // mov rax -> result
                            let res_size = vm.get_backend_type_size(res_tmp.ty.id());
                            match res_size {
                                8 => self.backend.emit_mov_r_r(&res_tmp, &x86_64::RAX),
                                4 => self.backend.emit_mov_r_r(&res_tmp, &x86_64::EAX),
                                2 => self.backend.emit_mov_r_r(&res_tmp, &x86_64::AX),
                                1 => self.backend.emit_mov_r_r(&res_tmp, &x86_64::AL),
                                _ => panic!("unexpected res for node {:?}", node)
                            }
                        }
                    }
                    16 => {
                        // emit_ireg_ex will split 128 bits register to two 64-bits temporaries
                        // but here we want to pass 128-bit registers as argument, and let
                        // calling convention deal with splitting.
                        // FIXME: emit_ireg() may not be proper here (though it might work)
                        let reg_op1 = self.emit_ireg(&op1, f_content, f_context, vm);
                        let reg_op2 = self.emit_ireg(&op2, f_content, f_context, vm);

                        self.emit_runtime_entry(
                            &entrypoints::UDIV_U128,
                            vec![reg_op1, reg_op2],
                            Some(vec![res_tmp.clone()]),
                            Some(node),
                            f_content,
                            f_context,
                            vm
                        );
                    }
                    _ => panic!("unsupported int size: {}", op_size)
                }
            }
            op::BinOp::Sdiv => {
                let op_size = vm.get_backend_type_size(op1.as_value().ty.id());

                match op_size {
                    1 | 2 | 4 | 8 => {
                        if self.match_iconst_p2(op2) {
                            // we can simply arithmetic shift right
                            let shift = self.node_iconst_to_p2(op2);

                            let tmp_op1 = self.emit_ireg(op1, f_content, f_context, vm);
                            self.backend.emit_mov_r_r(&res_tmp, &tmp_op1);
                            self.backend.emit_sar_r_imm8(&res_tmp, shift as i8);
                        } else {
                            self.emit_idiv(op1, op2, f_content, f_context, vm);

                            // mov rax -> result
                            let res_size = vm.get_backend_type_size(res_tmp.ty.id());
                            match res_size {
                                8 => self.backend.emit_mov_r_r(&res_tmp, &x86_64::RAX),
                                4 => self.backend.emit_mov_r_r(&res_tmp, &x86_64::EAX),
                                2 => self.backend.emit_mov_r_r(&res_tmp, &x86_64::AX),
                                1 => self.backend.emit_mov_r_r(&res_tmp, &x86_64::AL),
                                _ => panic!("unexpected res for node {:?}", node)
                            }
                        }
                    }
                    16 => {
                        let reg_op1 = self.emit_ireg(&op1, f_content, f_context, vm);
                        let reg_op2 = self.emit_ireg(&op2, f_content, f_context, vm);

                        self.emit_runtime_entry(
                            &entrypoints::SDIV_I128,
                            vec![reg_op1, reg_op2],
                            Some(vec![res_tmp.clone()]),
                            Some(node),
                            f_content,
                            f_context,
                            vm
                        );
                    }
                    _ => panic!("unsupported int size: {}", op_size)
                }
            }
            op::BinOp::Urem => {
                let op_size = vm.get_backend_type_size(op1.as_value().ty.id());

                match op_size {
                    1 | 2 | 4 | 8 => {
                        self.emit_udiv(op1, op2, f_content, f_context, vm);

                        // mov rdx -> result
                        let res_size = vm.get_backend_type_size(res_tmp.ty.id());
                        match res_size {
                            8 => self.backend.emit_mov_r_r(&res_tmp, &x86_64::RDX),
                            4 => self.backend.emit_mov_r_r(&res_tmp, &x86_64::EDX),
                            2 => self.backend.emit_mov_r_r(&res_tmp, &x86_64::DX),
                            1 => self.backend.emit_mov_r_r(&res_tmp, &x86_64::AH),
                            _ => panic!("unexpected res for node {:?}", node)
                        }
                    }
                    16 => {
                        let reg_op1 = self.emit_ireg(&op1, f_content, f_context, vm);
                        let reg_op2 = self.emit_ireg(&op2, f_content, f_context, vm);

                        self.emit_runtime_entry(
                            &entrypoints::UREM_U128,
                            vec![reg_op1, reg_op2],
                            Some(vec![res_tmp.clone()]),
                            Some(node),
                            f_content,
                            f_context,
                            vm
                        );
                    }
                    _ => panic!("unsupported int size: {}", op_size)
                }
            }
            op::BinOp::Srem => {
                let op_size = vm.get_backend_type_size(op1.as_value().ty.id());

                match op_size {
                    1 | 2 | 4 | 8 => {
                        self.emit_idiv(op1, op2, f_content, f_context, vm);

                        // mov rdx -> result
                        let res_size = vm.get_backend_type_size(res_tmp.ty.id());
                        match res_size {
                            8 => self.backend.emit_mov_r_r(&res_tmp, &x86_64::RDX),
                            4 => self.backend.emit_mov_r_r(&res_tmp, &x86_64::EDX),
                            2 => self.backend.emit_mov_r_r(&res_tmp, &x86_64::DX),
                            1 => self.backend.emit_mov_r_r(&res_tmp, &x86_64::AH),
                            _ => panic!("unexpected res for node {:?}", node)
                        }
                    }
                    16 => {
                        let reg_op1 = self.emit_ireg(&op1, f_content, f_context, vm);
                        let reg_op2 = self.emit_ireg(&op2, f_content, f_context, vm);

                        self.emit_runtime_entry(
                            &entrypoints::SREM_I128,
                            vec![reg_op1, reg_op2],
                            Some(vec![res_tmp.clone()]),
                            Some(node),
                            f_content,
                            f_context,
                            vm
                        );
                    }
                    _ => panic!("unsupported int size: {}")
                }
            }

            op::BinOp::Shl => {
                if self.match_ireg(op1) && self.match_iconst_zero(op2) {
                    trace!("emit shl-ireg-0");

                    self.emit_move_node_to_value(&res_tmp, op1, f_content, f_context, vm);
                } else if self.match_ireg(op1) && self.match_iimm(op2) {
                    trace!("emit shl-ireg-iimm");

                    let tmp_op1 = self.emit_ireg(op1, f_content, f_context, vm);
                    let imm_op2 = self.node_iimm_to_i32(op2);

                    // mov op1 -> res
                    self.backend.emit_mov_r_r(&res_tmp, &tmp_op1);

                    // shl result, op2 -> result
                    self.backend.emit_shl_r_imm8(&res_tmp, imm_op2 as i8);
                } else if self.match_ireg(op1) && self.match_ireg(op2) {
                    trace!("emit shl-ireg-ireg");

                    let tmp_op1 = self.emit_ireg(op1, f_content, f_context, vm);
                    let tmp_op2 = self.emit_ireg(op2, f_content, f_context, vm);

                    // mov op2 -> cl
                    self.backend
                        .emit_mov_r_r(&x86_64::CL, unsafe { &tmp_op2.as_type(UINT8_TYPE.clone()) });

                    // mov op1 -> result
                    self.backend.emit_mov_r_r(&res_tmp, &tmp_op1);

                    // shl result, cl -> result
                    self.backend.emit_shl_r_cl(&res_tmp);
                } else if self.match_ireg_ex(op1) && self.match_iconst_zero(op2) {
                    trace!("emit shl-iregex-0");

                    self.emit_move_node_to_value(&res_tmp, op1, f_content, f_context, vm);
                } else if self.match_ireg_ex(op1) && self.match_ireg_ex(op2) {
                    trace!("emit shl-iregex-iregex");

                    let (op1_l, op1_h) = self.emit_ireg_ex(op1, f_content, f_context, vm);
                    let (op2_l, _) = self.emit_ireg_ex(op2, f_content, f_context, vm);
                    let (res_l, res_h) = self.split_int128(&res_tmp, f_context, vm);

                    // mov op2_l -> ecx (we do not care higher bits)
                    self.backend
                        .emit_mov_r_r(&x86_64::ECX, unsafe { &op2_l.as_type(UINT32_TYPE.clone()) });

                    // mov op1_h -> t1
                    let t1 = self.make_temporary(f_context, UINT64_TYPE.clone(), vm);
                    self.backend.emit_mov_r_r(&t1, &op1_h);

                    // shld op1_l, t1, cl -> t1
                    self.backend.emit_shld_r_r_cl(&t1, &op1_l);

                    // mov op1_l -> t2
                    let t2 = self.make_temporary(f_context, UINT64_TYPE.clone(), vm);
                    self.backend.emit_mov_r_r(&t2, &op1_l);

                    // shl t2, cl -> t2
                    self.backend.emit_shl_r_cl(&t2);

                    // clear res_l
                    self.backend.emit_mov_r_imm(&res_l, 0);

                    // test 64, cl
                    self.backend.emit_test_imm_r(64i32, &x86_64::CL);

                    // cmovne t2 -> t1
                    self.backend.emit_cmovne_r_r(&t1, &t2);

                    // cmove t2 -> res_l
                    self.backend.emit_cmove_r_r(&res_l, &t2);

                    // mov t1 -> res_h
                    self.backend.emit_mov_r_r(&res_h, &t1);
                } else {
                    panic!("unexpected op for node {:?}", node)
                }
            }
            op::BinOp::Lshr => {
                if self.match_ireg(op1) && self.match_iconst_zero(op2) {
                    trace!("emit lshr-ireg-0");
                    self.emit_move_node_to_value(&res_tmp, op1, f_content, f_context, vm);
                } else if self.match_ireg(op1) && self.match_iimm(op2) {
                    trace!("emit lshr-ireg-iimm");

                    let tmp_op1 = self.emit_ireg(op1, f_content, f_context, vm);
                    let imm_op2 = self.node_iimm_to_i32(op2);

                    // mov op1 -> res
                    self.backend.emit_mov_r_r(&res_tmp, &tmp_op1);

                    // lshr result, op2 -> result
                    self.backend.emit_shr_r_imm8(&res_tmp, imm_op2 as i8);
                } else if self.match_ireg(op1) && self.match_ireg(op2) {
                    trace!("emit lshr-ireg-ireg");

                    let tmp_op1 = self.emit_ireg(op1, f_content, f_context, vm);
                    let tmp_op2 = self.emit_ireg(op2, f_content, f_context, vm);

                    // mov op2 -> cl
                    self.backend
                        .emit_mov_r_r(&x86_64::CL, unsafe { &tmp_op2.as_type(UINT8_TYPE.clone()) });

                    // mov op1 -> result
                    self.backend.emit_mov_r_r(&res_tmp, &tmp_op1);

                    // lshr result, cl -> result
                    self.backend.emit_shr_r_cl(&res_tmp);
                } else if self.match_ireg(op1) && self.match_iconst_zero(op2) {
                    trace!("emit lshr-iregex-0");
                    self.emit_move_node_to_value(&res_tmp, op1, f_content, f_context, vm);
                } else if self.match_ireg_ex(op1) && self.match_ireg_ex(op2) {
                    trace!("emit lshr-iregex-iregex");

                    let (op1_l, op1_h) = self.emit_ireg_ex(op1, f_content, f_context, vm);
                    let (op2_l, _) = self.emit_ireg_ex(op2, f_content, f_context, vm);
                    let (res_l, res_h) = self.split_int128(&res_tmp, f_context, vm);

                    // mov op2_l -> ecx (we do not care higher bits)
                    self.backend
                        .emit_mov_r_r(&x86_64::ECX, unsafe { &op2_l.as_type(UINT32_TYPE.clone()) });

                    // mov op1_l -> t1
                    let t1 = self.make_temporary(f_context, UINT64_TYPE.clone(), vm);
                    self.backend.emit_mov_r_r(&t1, &op1_l);

                    // shrd op1_h, t1, cl -> t1
                    self.backend.emit_shrd_r_r_cl(&t1, &op1_h);

                    // mov op1_h -> t2
                    let t2 = self.make_temporary(f_context, UINT64_TYPE.clone(), vm);
                    self.backend.emit_mov_r_r(&t2, &op1_h);

                    // shr t2, cl -> t2
                    self.backend.emit_shr_r_cl(&t2);

                    // clear res_h
                    self.backend.emit_mov_r_imm(&res_h, 0);

                    // test 64, cl
                    self.backend.emit_test_imm_r(64i32, &x86_64::CL);

                    // cmovne t2 -> t1
                    self.backend.emit_cmovne_r_r(&t1, &t2);

                    // cmove t2 -> res_h
                    self.backend.emit_cmove_r_r(&res_h, &t2);

                    // mov t1 -> res_l
                    self.backend.emit_mov_r_r(&res_l, &t1);
                } else {
                    panic!("unexpected op for node {:?}", node)
                }
            }
            op::BinOp::Ashr => {
                if self.match_ireg(op1) && self.match_iconst_zero(op2) {
                    trace!("emit ashr-ireg-0");
                    self.emit_move_node_to_value(&res_tmp, op1, f_content, f_context, vm);
                } else if self.match_ireg(op1) && self.match_iimm(op2) {
                    trace!("emit ashr-ireg-iimm");

                    let tmp_op1 = self.emit_ireg(op1, f_content, f_context, vm);
                    let imm_op2 = self.node_iimm_to_i32(op2);

                    // mov op1 -> res
                    self.backend.emit_mov_r_r(&res_tmp, &tmp_op1);

                    // sar result, op2 -> result
                    self.backend.emit_sar_r_imm8(&res_tmp, imm_op2 as i8);
                } else if self.match_ireg(op1) && self.match_ireg(op2) {
                    trace!("emit ashr-ireg-ireg");

                    let tmp_op1 = self.emit_ireg(op1, f_content, f_context, vm);
                    let tmp_op2 = self.emit_ireg(op2, f_content, f_context, vm);

                    // mov op2 -> cl
                    self.backend
                        .emit_mov_r_r(&x86_64::CL, unsafe { &tmp_op2.as_type(UINT8_TYPE.clone()) });

                    // mov op1 -> result
                    self.backend.emit_mov_r_r(&res_tmp, &tmp_op1);

                    // sar result, cl -> result
                    self.backend.emit_sar_r_cl(&res_tmp);
                } else if self.match_ireg_ex(op1) && self.match_iconst_zero(op2) {
                    trace!("emit ashr-iregex-0");
                    self.emit_move_node_to_value(&res_tmp, op1, f_content, f_context, vm);
                } else if self.match_ireg_ex(op1) && self.match_ireg_ex(op2) {
                    trace!("emit ashr-iregex-iregex");

                    let (op1_l, op1_h) = self.emit_ireg_ex(op1, f_content, f_context, vm);
                    let (op2_l, _) = self.emit_ireg_ex(op2, f_content, f_context, vm);
                    let (res_l, res_h) = self.split_int128(&res_tmp, f_context, vm);

                    // mov op2_l -> ecx
                    self.backend
                        .emit_mov_r_r(&x86_64::ECX, unsafe { &op2_l.as_type(UINT32_TYPE.clone()) });

                    // mov op1_l -> t1
                    let t1 = self.make_temporary(f_context, UINT64_TYPE.clone(), vm);
                    self.backend.emit_mov_r_r(&t1, &op1_l);

                    // shrd op1_h, t1, cl -> t1
                    self.backend.emit_shrd_r_r_cl(&t1, &op1_h);

                    // mov op1_h -> t2
                    let t2 = self.make_temporary(f_context, UINT64_TYPE.clone(), vm);
                    self.backend.emit_mov_r_r(&t2, &op1_h);

                    // sar t2, cl -> t2
                    self.backend.emit_sar_r_cl(&t2);

                    // mov op1_h -> t3
                    let t3 = self.make_temporary(f_context, UINT64_TYPE.clone(), vm);
                    self.backend.emit_mov_r_r(&t3, &op1_h);

                    // sar t3, 63 -> t3
                    self.backend.emit_sar_r_imm8(&t3, 63i8);

                    // test 64 cl
                    self.backend.emit_test_imm_r(64i32, &x86_64::CL);

                    // cmovne t2 -> t1
                    self.backend.emit_cmovne_r_r(&t1, &t2);

                    // cmove t2 -> t3
                    self.backend.emit_cmove_r_r(&t3, &t2);

                    // t1 as lower, t3 as higher
                    self.backend.emit_mov_r_r(&res_l, &t1);
                    self.backend.emit_mov_r_r(&res_h, &t3);
                } else {
                    panic!("unexpected op for node {:?}", node)
                }
            }

            // floating point
            op::BinOp::FAdd => {
                if self.match_fpreg(op1) && self.match_fconst_zero(op2) {
                    trace!("emit add-fpreg-0");
                    self.emit_move_node_to_value(&res_tmp, op1, f_content, f_context, vm);
                } else if self.match_fpreg(op1) && self.match_mem(op2) {
                    trace!("emit add-fpreg-mem");

                    let reg_op1 = self.emit_fpreg(op1, f_content, f_context, vm);
                    let mem_op2 = self.emit_mem(op2, f_content, f_context, vm);

                    match reg_op1.ty.v {
                        MuType_::Double => {
                            // mov op1, res
                            self.backend.emit_movsd_f64_f64(&res_tmp, &reg_op1);
                            // add op2 res
                            self.backend.emit_addsd_f64_mem64(&res_tmp, &mem_op2);
                        }
                        MuType_::Float => {
                            // mov op1, res
                            self.backend.emit_movss_f32_f32(&res_tmp, &reg_op1);
                            // add op2 res
                            self.backend.emit_addss_f32_mem32(&res_tmp, &mem_op2);
                        }
                        _ => panic!("expect double or float")
                    }

                } else if self.match_fpreg(op1) && self.match_fpreg(op2) {
                    trace!("emit add-fpreg-fpreg");

                    let reg_op1 = self.emit_fpreg(op1, f_content, f_context, vm);
                    let reg_op2 = self.emit_fpreg(op2, f_content, f_context, vm);

                    match reg_op1.ty.v {
                        MuType_::Double => {
                            // movsd op1, res
                            self.backend.emit_movsd_f64_f64(&res_tmp, &reg_op1);
                            // add op2 res
                            self.backend.emit_addsd_f64_f64(&res_tmp, &reg_op2);
                        }
                        MuType_::Float => {
                            // movsd op1, res
                            self.backend.emit_movss_f32_f32(&res_tmp, &reg_op1);
                            // add op2 res
                            self.backend.emit_addss_f32_f32(&res_tmp, &reg_op2);
                        }
                        _ => panic!("expect double or float")
                    }
                } else {
                    panic!("unexpected op for node {:?}", node)
                }
            }

            op::BinOp::FSub => {
                if self.match_fpreg(op1) && self.match_fconst_zero(op2) {
                    trace!("emit sub-fpreg-0");
                    self.emit_move_node_to_value(&res_tmp, op1, f_content, f_context, vm);
                } else if self.match_fpreg(op1) && self.match_mem(op2) {
                    trace!("emit sub-fpreg-mem");

                    let reg_op1 = self.emit_fpreg(op1, f_content, f_context, vm);
                    let mem_op2 = self.emit_mem(op2, f_content, f_context, vm);

                    match reg_op1.ty.v {
                        MuType_::Double => {
                            // mov op1, res
                            self.backend.emit_movsd_f64_f64(&res_tmp, &reg_op1);
                            // sub op2 res
                            self.backend.emit_subsd_f64_mem64(&res_tmp, &mem_op2);
                        }
                        MuType_::Float => {
                            // mov op1, res
                            self.backend.emit_movss_f32_f32(&res_tmp, &reg_op1);
                            // sub op2 res
                            self.backend.emit_subss_f32_mem32(&res_tmp, &mem_op2);
                        }
                        _ => panic!("expect double or float")
                    }
                } else if self.match_fpreg(op1) && self.match_fpreg(op2) {
                    trace!("emit sub-fpreg-fpreg");

                    let reg_op1 = self.emit_fpreg(op1, f_content, f_context, vm);
                    let reg_op2 = self.emit_fpreg(op2, f_content, f_context, vm);

                    match reg_op1.ty.v {
                        MuType_::Double => {
                            // movsd op1, res
                            self.backend.emit_movsd_f64_f64(&res_tmp, &reg_op1);
                            // sub op2 res
                            self.backend.emit_subsd_f64_f64(&res_tmp, &reg_op2);
                        }
                        MuType_::Float => {
                            // movss op1, res
                            self.backend.emit_movss_f32_f32(&res_tmp, &reg_op1);
                            // sub op2 res
                            self.backend.emit_subss_f32_f32(&res_tmp, &reg_op2);
                        }
                        _ => panic!("expect double or float")
                    }
                } else {
                    panic!("unexpected op for node {:?}", node)
                }
            }

            op::BinOp::FMul => {
                if self.match_fpreg(op1) && self.match_fconst_zero(op2) {
                    trace!("emit mul-fpreg-0");
                    self.emit_clear_value(&res_tmp, f_context, vm);
                } else if self.match_fpreg(op1) && self.match_mem(op2) {
                    trace!("emit mul-fpreg-mem");

                    let reg_op1 = self.emit_fpreg(op1, f_content, f_context, vm);
                    let mem_op2 = self.emit_mem(op2, f_content, f_context, vm);

                    match reg_op1.ty.v {
                        MuType_::Double => {
                            // mov op1, res
                            self.backend.emit_movsd_f64_f64(&res_tmp, &reg_op1);
                            // mul op2 res
                            self.backend.emit_mulsd_f64_mem64(&res_tmp, &mem_op2);
                        }
                        MuType_::Float => {
                            // mov op1, res
                            self.backend.emit_movss_f32_f32(&res_tmp, &reg_op1);
                            // mul op2 res
                            self.backend.emit_mulss_f32_mem32(&res_tmp, &mem_op2);
                        }
                        _ => panic!("expect double or float")
                    }
                } else if self.match_fpreg(op1) && self.match_fpreg(op2) {
                    trace!("emit mul-fpreg-fpreg");

                    let reg_op1 = self.emit_fpreg(op1, f_content, f_context, vm);
                    let reg_op2 = self.emit_fpreg(op2, f_content, f_context, vm);

                    match reg_op1.ty.v {
                        MuType_::Double => {
                            // movsd op1, res
                            self.backend.emit_movsd_f64_f64(&res_tmp, &reg_op1);
                            // mul op2 res
                            self.backend.emit_mulsd_f64_f64(&res_tmp, &reg_op2);
                        }
                        MuType_::Float => {
                            // movss op1, res
                            self.backend.emit_movss_f32_f32(&res_tmp, &reg_op1);
                            // mul op2 res
                            self.backend.emit_mulss_f32_f32(&res_tmp, &reg_op2);
                        }
                        _ => panic!("expect double or float")
                    }
                } else {
                    panic!("unexpected op for node {:?}", node)
                }
            }

            op::BinOp::FDiv => {
                if self.match_fpreg(op1) && self.match_mem(op2) {
                    trace!("emit div-fpreg-mem");

                    let reg_op1 = self.emit_fpreg(op1, f_content, f_context, vm);
                    let mem_op2 = self.emit_mem(op2, f_content, f_context, vm);

                    match reg_op1.ty.v {
                        MuType_::Double => {
                            // mov op1, res
                            self.backend.emit_movsd_f64_f64(&res_tmp, &reg_op1);
                            // div op2 res
                            self.backend.emit_divsd_f64_mem64(&res_tmp, &mem_op2);
                        }
                        MuType_::Float => {
                            // mov op1, res
                            self.backend.emit_movss_f32_f32(&res_tmp, &reg_op1);
                            // div op2 res
                            self.backend.emit_divss_f32_mem32(&res_tmp, &mem_op2);
                        }
                        _ => panic!("expect double or float")
                    }
                } else if self.match_fpreg(op1) && self.match_fpreg(op2) {
                    trace!("emit div-fpreg-fpreg");

                    let reg_op1 = self.emit_fpreg(op1, f_content, f_context, vm);
                    let reg_op2 = self.emit_fpreg(op2, f_content, f_context, vm);

                    match reg_op1.ty.v {
                        MuType_::Double => {
                            // movsd op1, res
                            self.backend.emit_movsd_f64_f64(&res_tmp, &reg_op1);
                            // div op2 res
                            self.backend.emit_divsd_f64_f64(&res_tmp, &reg_op2);
                        }
                        MuType_::Float => {
                            // movss op1, res
                            self.backend.emit_movss_f32_f32(&res_tmp, &reg_op1);
                            // div op2 res
                            self.backend.emit_divss_f32_f32(&res_tmp, &reg_op2);
                        }
                        _ => panic!("expect double or float")
                    }
                } else {
                    panic!("unexpected op for node {:?}", node)
                }
            }

            op::BinOp::FRem => {
                if self.match_fpreg(op1) && self.match_fpreg(op2) {
                    trace!("emit frem-fpreg-fpreg");

                    let reg_op1 = self.emit_fpreg(op1, f_content, f_context, vm);
                    let reg_op2 = self.emit_fpreg(op2, f_content, f_context, vm);

                    let reg_tmp = self.get_result_value(node);

                    match reg_op1.ty.v {
                        MuType_::Double => {
                            self.emit_runtime_entry(
                                &entrypoints::FREM64,
                                vec![reg_op1.clone(), reg_op2.clone()],
                                Some(vec![reg_tmp.clone()]),
                                Some(node),
                                f_content,
                                f_context,
                                vm
                            );
                        }
                        MuType_::Float => {
                            self.emit_runtime_entry(
                                &entrypoints::FREM32,
                                vec![reg_op1.clone(), reg_op2.clone()],
                                Some(vec![reg_tmp.clone()]),
                                Some(node),
                                f_content,
                                f_context,
                                vm
                            );
                        }
                        _ => panic!("expect double or float")
                    }
                } else {
                    panic!("unexpected op for node {:?}", node)
                }
            }
        }
    }

    /// emits the allocation sequence
    /// * if the size is known at compile time, either emits allocation for small objects or
    ///   large objects
    /// * if the size is not known at compile time, emits a branch to check the size at runtime,
    ///   and call corresponding allocation
    fn emit_alloc_const_size(
        &mut self,
        tmp_allocator: &P<Value>,
        size: ByteSize,
        align: ByteSize,
        node: &TreeNode,
        backend_ty: &BackendType,
        f_content: &FunctionContent,
        f_context: &mut FunctionContext,
        vm: &VM
    ) -> P<Value> {
        let size = math::align_up(size + OBJECT_HEADER_SIZE, POINTER_SIZE);
        let encode = mm::gen_object_encode(backend_ty, size, vm);

        let tmp_res = self.get_result_value(node);
        let tmp_align = self.make_int64_const(align as u64, vm);
        let tmp_size = self.make_int64_const(size as u64, vm);

        if size <= mm::MAX_TINY_OBJECT {
            // alloc tiny
            self.emit_runtime_entry(
                &entrypoints::ALLOC_TINY,
                vec![tmp_allocator.clone(), tmp_size.clone(), tmp_align],
                Some(vec![tmp_res.clone()]),
                Some(node),
                f_content,
                f_context,
                vm
            );
            // init object
            let tmp_encode = self.make_int_const(encode.tiny().as_u64(), UINT8_TYPE.clone(), vm);
            self.emit_runtime_entry(
                &entrypoints::INIT_TINY,
                vec![tmp_allocator.clone(), tmp_res.clone(), tmp_encode],
                None,
                Some(node),
                f_content,
                f_context,
                vm
            );
        } else if size <= mm::MAX_MEDIUM_OBJECT {
            // this could be either a small object or a medium object

            // alloc normal
            self.emit_runtime_entry(
                &entrypoints::ALLOC_NORMAL,
                vec![tmp_allocator.clone(), tmp_size.clone(), tmp_align],
                Some(vec![tmp_res.clone()]),
                Some(node),
                f_content,
                f_context,
                vm
            );
            // init object
            if size < mm::MAX_SMALL_OBJECT {
                let tmp_encode =
                    self.make_int_const(encode.small().as_u64(), UINT16_TYPE.clone(), vm);
                self.emit_runtime_entry(
                    &entrypoints::INIT_SMALL,
                    vec![tmp_allocator.clone(), tmp_res.clone(), tmp_encode],
                    None,
                    Some(node),
                    f_content,
                    f_context,
                    vm
                );
            } else {
                let tmp_encode =
                    self.make_int_const(encode.medium().as_u64(), UINT32_TYPE.clone(), vm);
                self.emit_runtime_entry(
                    &entrypoints::INIT_MEDIUM,
                    vec![tmp_allocator.clone(), tmp_res.clone(), tmp_encode],
                    None,
                    Some(node),
                    f_content,
                    f_context,
                    vm
                );
            };
        } else {
            // large allocation
            self.emit_runtime_entry(
                &entrypoints::ALLOC_LARGE,
                vec![tmp_allocator.clone(), tmp_size.clone(), tmp_align],
                Some(vec![tmp_res.clone()]),
                Some(node),
                f_content,
                f_context,
                vm
            );
            // init object
            let encode = encode.large();
            let tmp_encode1 = self.make_int64_const(encode.size() as u64, vm);
            let tmp_encode2 = self.make_int64_const(encode.type_id() as u64, vm);
            self.emit_runtime_entry(
                &entrypoints::INIT_LARGE,
                vec![
                    tmp_allocator.clone(),
                    tmp_res.clone(),
                    tmp_encode1,
                    tmp_encode2,
                ],
                None,
                Some(node),
                f_content,
                f_context,
                vm
            );
        }

        tmp_res
    }

    /// emits code to get allocator for current thread
    fn emit_get_allocator(
        &mut self,
        node: &TreeNode,
        f_content: &FunctionContent,
        f_context: &mut FunctionContext,
        vm: &VM
    ) -> P<Value> {
        // ASM: %tl = get_thread_local()
        let tmp_tl = self.emit_get_threadlocal(Some(node), f_content, f_context, vm);

        // ASM: lea [%tl + allocator_offset] -> %tmp_allocator
        let allocator_offset = *thread::ALLOCATOR_OFFSET;
        let tmp_allocator = self.make_temporary(f_context, ADDRESS_TYPE.clone(), vm);
        self.emit_lea_base_offset(&tmp_allocator, &tmp_tl, allocator_offset as i32, vm);

        tmp_allocator
    }

    /// emits code for large object allocation
    fn emit_alloc_sequence_large(
        &mut self,
        tmp_allocator: P<Value>,
        size: P<Value>,
        align: usize,
        node: &TreeNode,
        f_content: &FunctionContent,
        f_context: &mut FunctionContext,
        vm: &VM
    ) -> P<Value> {
        let tmp_res = self.get_result_value(node);

        // ASM: %tmp_res = call muentry_alloc_large(%allocator, size, align)
        let const_align = self.make_int64_const(align as u64, vm);

        self.emit_runtime_entry(
            &entrypoints::ALLOC_LARGE,
            vec![tmp_allocator.clone(), size.clone(), const_align],
            Some(vec![tmp_res.clone()]),
            Some(node),
            f_content,
            f_context,
            vm
        );

        tmp_res
    }

    /// emits a load instruction of address [base + offset]
    fn emit_load_base_offset(
        &mut self,
        dest: &P<Value>,
        base: &P<Value>,
        offset: i32,
        vm: &VM
    ) -> P<Value> {
        let mem = self.make_memory_op_base_offset(base, offset, dest.ty.clone(), vm);

        self.emit_move_value_to_value(dest, &mem);

        mem
    }

    /// emits a store instruction of address [base + offset]
    fn emit_store_base_offset(&mut self, base: &P<Value>, offset: i32, src: &P<Value>, vm: &VM) {
        let mem = self.make_memory_op_base_offset(base, offset, src.ty.clone(), vm);
        self.emit_move_value_to_value(&mem, src);
    }

    /// emits a lea (load effective address) instruction of address [base + offset]
    fn emit_lea_base_offset(&mut self, dest: &P<Value>, base: &P<Value>, offset: i32, vm: &VM) {
        let mem = self.make_memory_op_base_offset(base, offset, ADDRESS_TYPE.clone(), vm);

        self.backend.emit_lea_r64(dest, &mem);
    }

    /// emits a push instruction
    fn emit_push(&mut self, op: &P<Value>) {
        if op.is_int_const() {
            if x86_64::is_valid_x86_imm(op) {
                let int = op.extract_int_const().unwrap();
                self.backend.emit_push_imm32(int as i32);
            } else {
                unimplemented!();
            }
        } else {
            self.backend.emit_push_r64(op);
        }
    }

    /// emits a udiv instruction
    fn emit_udiv(
        &mut self,
        op1: &TreeNode,
        op2: &TreeNode,
        f_content: &FunctionContent,
        f_context: &mut FunctionContext,
        vm: &VM
    ) {
        assert!(self.match_ireg(op1));
        let reg_op1 = self.emit_ireg(op1, f_content, f_context, vm);

        let op1_size = vm.get_backend_type_size(reg_op1.ty.id());
        match op1_size {
            8 => {
                // div uses RDX and RAX
                self.backend.emit_mov_r_r(&x86_64::RAX, &reg_op1);

                // xorq rdx, rdx -> rdx
                self.backend.emit_xor_r_r(&x86_64::RDX, &x86_64::RDX);
            }
            4 => {
                // div uses edx, eax
                self.backend.emit_mov_r_r(&x86_64::EAX, &reg_op1);

                // xor edx edx
                self.backend.emit_xor_r_r(&x86_64::EDX, &x86_64::EDX);
            }
            2 => {
                // div uses dx, ax
                self.backend.emit_mov_r_r(&x86_64::AX, &reg_op1);

                // xor dx, dx
                self.backend.emit_xor_r_r(&x86_64::DX, &x86_64::DX);
            }
            1 => {
                // div uses AX
                self.backend.emit_mov_r_r(&x86_64::AL, &reg_op1);
            }
            _ => panic!("unsupported int size for udiv: {}", op1_size)
        }

        // div op2
        if self.match_mem(op2) {
            let mem_op2 = self.emit_mem(op2, f_content, f_context, vm);
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
            panic!("unexpected op2 for udiv: {}", op2);
        }
    }

    /// emits an idiv instruction
    fn emit_idiv(
        &mut self,
        op1: &TreeNode,
        op2: &TreeNode,
        f_content: &FunctionContent,
        f_context: &mut FunctionContext,
        vm: &VM
    ) {
        assert!(self.match_ireg(op1));
        let reg_op1 = self.emit_ireg(op1, f_content, f_context, vm);

        let op1_size = vm.get_backend_type_size(reg_op1.ty.id());
        match op1_size {
            8 => {
                // idiv uses RDX and RAX
                self.backend.emit_mov_r_r(&x86_64::RAX, &reg_op1);

                // cqo: sign extend rax to rdx:rax
                self.backend.emit_cqo();
            }
            4 => {
                // idiv uses edx, eax
                self.backend.emit_mov_r_r(&x86_64::EAX, &reg_op1);

                // cdq: sign extend eax to edx:eax
                self.backend.emit_cdq();
            }
            2 => {
                // idiv uses dx, ax
                self.backend.emit_mov_r_r(&x86_64::AX, &reg_op1);

                // cwd: sign extend ax to dx:ax
                self.backend.emit_cwd();
            }
            1 => {
                // idiv uses AL
                self.backend.emit_mov_r_r(&x86_64::AL, &reg_op1);

                // sign extend al to ax
                self.backend.emit_movs_r_r(&x86_64::AX, &x86_64::AL);
            }
            _ => panic!("unsupported int size for idiv: {}", op1_size)
        }

        // idiv op2
        if self.match_mem(op2) {
            let mem_op2 = self.emit_mem(op2, f_content, f_context, vm);
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
            panic!("unexpected op2 for idiv: {}", op2);
        }
    }

    /// emits code to get the thread local variable (not the client thread local)
    fn emit_get_threadlocal(
        &mut self,
        cur_node: Option<&TreeNode>,
        f_content: &FunctionContent,
        f_context: &mut FunctionContext,
        vm: &VM
    ) -> P<Value> {
        let mut rets = self.emit_runtime_entry(
            &entrypoints::GET_THREAD_LOCAL,
            vec![],
            None,
            cur_node,
            f_content,
            f_context,
            vm
        );

        rets.pop().unwrap()
    }

    /// emits code to call a runtime entry function, always returns result temporaries
    /// (given or created)
    /// Note that rets is Option<Vec<P<Value>>. If rets is Some, return values will be put
    /// in the given temporaries. Otherwise create temporaries for return results
    fn emit_runtime_entry(
        &mut self,
        entry: &RuntimeEntrypoint,
        args: Vec<P<Value>>,
        rets: Option<Vec<P<Value>>>,
        cur_node: Option<&TreeNode>,
        f_content: &FunctionContent,
        f_context: &mut FunctionContext,
        vm: &VM
    ) -> Vec<P<Value>> {
        let sig = entry.sig.clone();

        let entry_name = {
            if vm.is_doing_jit() {
                unimplemented!()
            } else {
                let ref entry_loc = entry.aot;

                match entry_loc {
                    &ValueLocation::Relocatable(_, ref name) => name.clone(),
                    _ => panic!("expecting a relocatable value")
                }
            }
        };

        self.emit_c_call_internal(
            entry_name,
            sig,
            args,
            rets,
            cur_node,
            f_content,
            f_context,
            vm
        )
    }

    /// emits calling convention before a call instruction
    /// returns the stack arg offset - we will need this to collapse stack after the call
    fn emit_precall_convention(
        &mut self,
        sig: &MuFuncSig,
        args: &Vec<P<Value>>,
        conv: CallConvention,
        f_context: &mut FunctionContext,
        vm: &VM
    ) -> (usize, Vec<P<Value>>) {
        let callconv = {
            match conv {
                CallConvention::Mu => callconv::mu::compute_arguments(&sig.arg_tys),
                CallConvention::Foreign(ForeignFFI::C) => {
                    callconv::c::compute_arguments(&sig.arg_tys)
                }
            }
        };
        assert!(callconv.len() == args.len());

        let (reg_args, stack_args) =
            self.emit_precall_convention_regs_only(args, &callconv, f_context, vm);

        if !stack_args.is_empty() {
            // store stack arguments
            let size = self.emit_store_stack_values(&stack_args, None, conv, vm);
            // offset RSP
            self.backend.emit_sub_r_imm(&x86_64::RSP, size as i32);

            (size, reg_args)
        } else {
            (0, reg_args)
        }
    }

    /// emits calling convention to pass argument registers before a call instruction
    /// returns a tuple of (machine registers used, pass-by-stack arguments)
    fn emit_precall_convention_regs_only(
        &mut self,
        args: &Vec<P<Value>>,
        callconv: &Vec<CallConvResult>,
        f_context: &mut FunctionContext,
        vm: &VM
    ) -> (Vec<P<Value>>, Vec<P<Value>>) {
        let mut stack_args = vec![];
        let mut reg_args = vec![];

        for i in 0..callconv.len() {
            let ref arg = args[i];
            let ref cc = callconv[i];

            match cc {
                &CallConvResult::GPR(ref reg) => {
                    reg_args.push(reg.clone());
                    if arg.is_reg() {
                        self.backend.emit_mov_r_r(reg, arg);
                    } else if arg.is_const() {
                        let int_const = arg.extract_int_const().unwrap();
                        if x86_64::is_valid_x86_imm(arg) {
                            self.backend.emit_mov_r_imm(reg, int_const as i32);
                        } else {
                            assert!(reg.ty.get_int_length().unwrap() == 64);
                            self.backend.emit_mov_r64_imm64(reg, int_const as i64);
                        }
                    } else {
                        panic!("arg {} is put to GPR, but it is neither reg or const");
                    }
                }
                &CallConvResult::GPREX(ref reg_l, ref reg_h) => {
                    reg_args.push(reg_l.clone());
                    reg_args.push(reg_h.clone());
                    if arg.is_reg() {
                        let (arg_l, arg_h) = self.split_int128(arg, f_context, vm);
                        self.backend.emit_mov_r_r(reg_l, &arg_l);
                        self.backend.emit_mov_r_r(reg_h, &arg_h);
                    } else if arg.is_const() {
                        let const_vals = arg.extract_int_ex_const();

                        assert!(const_vals.len() == 2);
                        self.backend.emit_mov_r64_imm64(reg_l, const_vals[0] as i64);
                        self.backend.emit_mov_r64_imm64(reg_h, const_vals[1] as i64);
                    } else {
                        panic!("arg {} is put to GPREX, but it is neither reg or const");
                    }
                }
                &CallConvResult::FPR(ref reg) => {
                    reg_args.push(reg.clone());
                    if arg.is_reg() {
                        self.emit_move_value_to_value(reg, arg);
                    } else if arg.is_const() {
                        unimplemented!();
                    } else {
                        panic!("arg {} is put to FPR, but it is neither reg or const");
                    }
                }
                &CallConvResult::STACK => {
                    stack_args.push(arg.clone());
                }
            }
        }

        (reg_args, stack_args)
    }

    /// emits code that store values to the stack, returns the space required on the stack
    /// * if base is None, save values to RSP (starting from RSP-stack_val_size),
    ///   growing upwards (to higher stack address)
    /// * if base is Some, save values to base, growing upwards
    fn emit_store_stack_values(
        &mut self,
        stack_vals: &Vec<P<Value>>,
        base: Option<(&P<Value>, i32)>,
        conv: CallConvention,
        vm: &VM
    ) -> ByteSize {
        use compiler::backend::x86_64::callconv;

        let stack_arg_tys = stack_vals.iter().map(|x| x.ty.clone()).collect();
        let (stack_arg_size_with_padding, stack_arg_offsets) = match conv {
            CallConvention::Mu => callconv::mu::compute_stack_locations(&stack_arg_tys, vm),
            CallConvention::Foreign(ForeignFFI::C) => {
                callconv::c::compute_stack_locations(&stack_arg_tys, vm)
            }
        };

        // now, we just put all the args on the stack
        {
            if stack_arg_size_with_padding != 0 {
                let mut index = 0;
                let rsp_offset_before_call = -(stack_arg_size_with_padding as i32);

                for arg in stack_vals {
                    if let Some((base, offset)) = base {
                        self.emit_store_base_offset(
                            base,
                            offset + (stack_arg_offsets[index]) as i32,
                            &arg,
                            vm
                        );
                    } else {
                        self.emit_store_base_offset(
                            &x86_64::RSP,
                            rsp_offset_before_call + (stack_arg_offsets[index] as i32),
                            &arg,
                            vm
                        );
                    }
                    index += 1;
                }
            }
        }

        stack_arg_size_with_padding
    }

    /// emits calling convention after a call instruction
    /// If rets is Some, return values will be put in the given temporaries.
    /// Otherwise create temporaries for return results.
    fn emit_postcall_convention(
        &mut self,
        sig: &P<MuFuncSig>,
        rets: &Option<Vec<P<Value>>>,
        precall_stack_arg_size: usize,
        conv: CallConvention,
        f_context: &mut FunctionContext,
        vm: &VM
    ) -> Vec<P<Value>> {
        let callconv = {
            match conv {
                CallConvention::Mu => callconv::mu::compute_return_values(&sig.ret_tys),
                CallConvention::Foreign(ForeignFFI::C) => {
                    callconv::c::compute_return_values(&sig.ret_tys)
                }
            }
        };

        let return_vals: Vec<P<Value>> = match rets {
            &Some(ref rets) => rets.clone(),
            &None => {
                sig.ret_tys
                    .iter()
                    .map(|ty| self.make_temporary(f_context, ty.clone(), vm))
                    .collect()
            }
        };

        let (_, stack_locs) = {
            if precall_stack_arg_size != 0 {
                match conv {
                    CallConvention::Mu => callconv::mu::compute_stack_retvals(&sig.ret_tys, vm),
                    CallConvention::Foreign(ForeignFFI::C) => {
                        callconv::c::compute_stack_retvals(&sig.ret_tys, vm)
                    }
                }
            } else {
                (0, vec![])
            }
        };
        self.emit_unload_values(
            &return_vals,
            &callconv,
            &stack_locs,
            None,
            false,
            f_context,
            vm
        );

        // collapse space for stack_args
        if precall_stack_arg_size != 0 {
            self.backend
                .emit_add_r_imm(&x86_64::RSP, precall_stack_arg_size as i32);
        }

        return_vals
    }

    /// emits code to unload values
    /// * unloads values that are passed by register from callconv
    /// * unloads values that are passed by stack from stack_arg_offsets
    ///   if stack_pointer is None, unload stack arguments from current RSP
    ///   otherwise, unload stack arguments from the base and offset in stack_pointer
    fn emit_unload_values(
        &mut self,
        rets: &Vec<P<Value>>,
        callconv: &Vec<CallConvResult>,
        stack_arg_offsets: &Vec<ByteSize>,
        stack_pointer: Option<(&P<Value>, i32)>,
        is_unloading_args: bool,
        f_context: &mut FunctionContext,
        vm: &VM
    ) {
        assert!(callconv.len() == rets.len());

        let mut stack_args = vec![];

        for i in 0..callconv.len() {
            let ref cc = callconv[i];
            let ref val = rets[i];
            assert!(val.is_reg());

            match cc {
                &CallConvResult::GPR(ref reg) => {
                    self.backend.emit_mov_r_r(val, reg);
                    if is_unloading_args {
                        self.current_frame
                            .as_mut()
                            .unwrap()
                            .add_argument_by_reg(val.id(), reg.clone());
                    }
                }
                &CallConvResult::GPREX(ref reg_l, ref reg_h) => {
                    let (val_l, val_h) = self.split_int128(val, f_context, vm);
                    self.backend.emit_mov_r_r(&val_l, reg_l);
                    self.backend.emit_mov_r_r(&val_h, reg_h);
                    if is_unloading_args {
                        self.current_frame
                            .as_mut()
                            .unwrap()
                            .add_argument_by_reg(val_l.id(), reg_l.clone());
                        self.current_frame
                            .as_mut()
                            .unwrap()
                            .add_argument_by_reg(val_h.id(), reg_h.clone());
                    }
                }
                &CallConvResult::FPR(ref reg) => {
                    if val.ty.is_double() {
                        self.backend.emit_movsd_f64_f64(val, reg);
                    } else if val.ty.is_float() {
                        self.backend.emit_movss_f32_f32(val, reg);
                    } else {
                        panic!("expected double or float");
                    }

                    if is_unloading_args {
                        self.current_frame
                            .as_mut()
                            .unwrap()
                            .add_argument_by_reg(val.id(), reg.clone());
                    }
                }
                &CallConvResult::STACK => stack_args.push(val.clone())
            }
        }

        assert!(stack_args.len() == stack_arg_offsets.len());
        if !stack_args.is_empty() {
            for i in 0..stack_args.len() {
                let ref arg = stack_args[i];
                let offset = stack_arg_offsets[i] as i32;

                let stack_slot = if let Some((base, base_offset)) = stack_pointer {
                    self.emit_load_base_offset(arg, base, base_offset + offset, vm)
                } else {
                    self.emit_load_base_offset(arg, &x86_64::RSP, offset, vm)
                };

                if is_unloading_args {
                    self.current_frame
                        .as_mut()
                        .unwrap()
                        .add_argument_by_stack(arg.id(), stack_slot);
                }
            }
        }
    }

    /// emits a native call
    /// Note that rets is Option<Vec<P<Value>>. If rets is Some, return values will be put
    /// in the given temporaries. Otherwise create temporaries for return results
    #[allow(unused_variables)] // f_content is not used, but we keep it in case
    fn emit_c_call_internal(
        &mut self,
        func_name: CName,
        sig: P<CFuncSig>,
        args: Vec<P<Value>>,
        rets: Option<Vec<P<Value>>>,
        cur_node: Option<&TreeNode>,
        f_content: &FunctionContent,
        f_context: &mut FunctionContext,
        vm: &VM
    ) -> Vec<P<Value>> {
        let (stack_arg_size, args) =
            self.emit_precall_convention(&sig, &args, C_CALL_CONVENTION, f_context, vm);

        // make call
        if vm.is_doing_jit() {
            unimplemented!()
        } else {
            let callsite = self.new_callsite_label(cur_node);
            // assume ccall wont throw exception
            self.backend.emit_call_near_rel32(
                callsite.clone(),
                func_name,
                None,
                args,
                x86_64::ALL_CALLER_SAVED_REGS.to_vec(),
                true
            );

            // TODO: What if theres an exception block?
            self.current_callsites
                .push_back((callsite, 0, stack_arg_size));

            // record exception block (CCall may have an exception block)
            // FIXME: unimplemented for now (see Issue #42)
            if cur_node.is_some() {
                let cur_node = cur_node.unwrap();
                match cur_node.v {
                    TreeNode_::Instruction(Instruction {
                        v: Instruction_::CCall { .. },
                        ..
                    }) => unimplemented!(),
                    _ => {
                        // wont have an exception branch, ignore
                    }
                }
            }
        }

        self.emit_postcall_convention(
            &sig,
            &rets,
            stack_arg_size,
            C_CALL_CONVENTION,
            f_context,
            vm
        )
    }

    /// emits a CCALL
    /// currently only support calling a C function by name (Constant::ExnSymbol)
    #[allow(unused_variables)] // resumption is not used (CCALL exception is not implemented)
    fn emit_c_call_ir(
        &mut self,
        inst: &Instruction,
        calldata: &CallData,
        resumption: Option<&ResumptionData>,
        cur_node: &TreeNode,
        f_content: &FunctionContent,
        f_context: &mut FunctionContext,
        vm: &VM
    ) {
        let ref ops = inst.ops;

        // prepare args (they could be instructions, we need to emit inst and get value)
        let args = self.process_call_arguments(calldata, ops, f_content, f_context, vm);

        trace!("generating ccall");
        let ref func = ops[calldata.func];

        // type of the callee is defined as below in the Mu spec:
        // "The callee must have type T. The allowed type of T, and the number of return values,
        // are implementation-dependent and calling convention-dependent.
        // NOTE: T is usually ufuncptr<sig>, but the design also allows T to be the system call
        // number (integer) or anything meaningful for a particular implementation."

        // we only allow ufuncptr constant at the moment
        if self.match_func_const(func) {
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
                                sig,               // sig: P<CFuncSig>,
                                args,              // args: Vec<P<Value>>,
                                rets,              // Option<Vec<P<Value>>>,
                                Some(cur_node),    // Option<&TreeNode>,
                                f_content,         // &FunctionContent,
                                f_context,         // &mut FunctionContext,
                                vm
                            );
                        }
                        _ => {
                            panic!(
                                "expect a ufuncptr to be either address constant, \
                                 or symbol constant, we got {}",
                                pv
                            )
                        }
                    }
                }
                _ => {
                    // emit func pointer from an instruction
                    unimplemented!()
                }
            }
        } else {
            panic!("unsupported callee type for CCALL: {}", func)
        }
    }

    /// emits a CALL
    fn emit_mu_call(
        &mut self,
        inst: &Instruction,
        calldata: &CallData,
        resumption: Option<&ResumptionData>,
        node: &TreeNode,
        f_content: &FunctionContent,
        f_context: &mut FunctionContext,
        vm: &VM
    ) {
        trace!("deal with pre-call convention");

        let ref ops = inst.ops;
        let ref func = ops[calldata.func];
        let ref func_sig = match func.v {
            TreeNode_::Value(ref pv) => pv.ty.get_func_sig().unwrap(),
            TreeNode_::Instruction(ref inst) => {
                let ref funcref_val = inst.value.as_ref().unwrap()[0];
                funcref_val.ty.get_func_sig().unwrap()
            }
        };

        // arguments should match the signature
        assert!(func_sig.arg_tys.len() == calldata.args.len());
        // return values should match the signature
        if inst.value.is_some() {
            assert!(func_sig.ret_tys.len() == inst.value.as_ref().unwrap().len());
        } else {
            assert!(
                func_sig.ret_tys.len() == 0,
                "expect call inst's value doesnt match reg args. value: {:?}, ret args: {:?}",
                inst.value,
                func_sig.ret_tys
            );
        }

        // prepare args (they could be instructions, we need to emit inst and get value)
        let arg_values = self.process_call_arguments(calldata, ops, f_content, f_context, vm);
        let (stack_arg_size, arg_regs) =
            self.emit_precall_convention(func_sig, &arg_values, calldata.convention, f_context, vm);

        // check if this call has exception clause - need to tell backend about this
        let potentially_excepting = {
            if resumption.is_some() {
                let target_id = resumption.unwrap().exn_dest.target.id();
                Some(f_content.get_block(target_id).name())
            } else {
                None
            }
        };

        trace!("generating call inst");
        // check direct call or indirect
        let callsite = {
            if self.match_func_const(func) {
                let target_id = self.node_funcref_const_to_id(func);
                let funcs = vm.funcs().read().unwrap();
                let target = funcs.get(&target_id).unwrap().read().unwrap();

                if vm.is_doing_jit() {
                    unimplemented!()
                } else {
                    let callsite = self.new_callsite_label(Some(node));
                    self.backend.emit_call_near_rel32(
                        callsite,
                        target.name(),
                        potentially_excepting,
                        arg_regs,
                        x86_64::ALL_CALLER_SAVED_REGS.to_vec(),
                        false
                    )
                }
            } else if self.match_ireg(func) {
                let target = self.emit_ireg(func, f_content, f_context, vm);

                let callsite = self.new_callsite_label(Some(node));
                self.backend.emit_call_near_r64(
                    callsite,
                    &target,
                    potentially_excepting,
                    arg_regs,
                    x86_64::ALL_CALLER_SAVED_REGS.to_vec()
                )
            } else if self.match_mem(func) {
                let target = self.emit_mem(func, f_content, f_context, vm);

                let callsite = self.new_callsite_label(Some(node));
                self.backend.emit_call_near_mem64(
                    callsite,
                    &target,
                    potentially_excepting,
                    arg_regs,
                    x86_64::ALL_CALLER_SAVED_REGS.to_vec()
                )
            } else {
                panic!("unsupported callee type for CALL: {}", func);
            }
        };

        if resumption.is_some() {
            // record exception branch
            let ref exn_dest = resumption.as_ref().unwrap().exn_dest;
            let target_block_id = exn_dest.target.id();

            self.current_callsites
                .push_back((callsite.to_relocatable(), target_block_id, stack_arg_size));

            // insert an intermediate block to branch to normal
            // the branch is inserted later (because we need to deal with postcall convention)
            self.finish_block();
            let block_name = make_block_name(&node.name(), "normal_cont_for_call");
            self.start_block(block_name);
        } else {
            self.current_callsites
                .push_back((callsite.to_relocatable(), 0, stack_arg_size));
        }

        // deal with ret vals, collapse stack etc.
        self.emit_postcall_convention(
            &func_sig,
            &inst.value,
            stack_arg_size,
            calldata.convention,
            f_context,
            vm
        );

        // jump to target block
        if resumption.is_some() {
            self.backend
                .emit_jmp(resumption.as_ref().unwrap().normal_dest.target.name());
        }
    }

    /// emits code for swapstacks (all variants)
    fn emit_swapstack(
        &mut self,
        is_exception: bool, // whether we are throwing an exception to the new stack
        is_kill: bool,      // whether we are killing the old stack
        node: &TreeNode,
        inst: &Instruction,
        swappee: OpIndex,
        args: &Vec<OpIndex>,
        resumption: Option<&ResumptionData>,
        f_content: &FunctionContent,
        f_context: &mut FunctionContext,
        vm: &VM
    ) {
        use compiler::backend::x86_64::callconv::swapstack;

        let ref ops = inst.ops;

        // callsite label that will be used to mark the resumption point when
        // the current stack is swapped back
        let callsite_label = self.new_callsite_label(Some(node));

        // emit for all the arguments
        let mut arg_values = self.process_arguments(args, ops, f_content, f_context, vm);

        // load current stack ref
        let tl = self.emit_get_threadlocal(Some(node), f_content, f_context, vm);
        let cur_stackref = self.make_temporary(f_context, STACKREF_TYPE.clone(), vm);
        self.emit_load_base_offset(&cur_stackref, &tl, *thread::STACK_OFFSET as i32, vm);

        // store the new stack ref back to current MuThread
        let swappee = self.emit_ireg(&ops[swappee], f_content, f_context, vm);
        self.emit_store_base_offset(&tl, *thread::STACK_OFFSET as i32, &swappee, vm);

        // compute the locations of return values,
        // and how much space needs to be reserved on the stack
        let res_vals = match inst.value {
            Some(ref values) => values.to_vec(),
            None => vec![]
        };
        let res_tys = res_vals.iter().map(|x| x.ty.clone()).collect::<Vec<_>>();
        let (res_stack_size, res_locs) = swapstack::compute_stack_retvals(&res_tys, vm);

        if !is_kill {
            // if we are going to return to this stack, we need to push ret address, and RBP
            // otherwise, there is no need to push those (no one would access them)
            if vm.is_doing_jit() {
                unimplemented!()
            } else {
                if res_stack_size != 0 {
                    // reserve space on the stack for the return values of swapstack
                    self.backend
                        .emit_sub_r_imm(&x86_64::RSP, res_stack_size as i32);
                }

                // get return address (the instruction after the call
                let tmp_callsite_addr_loc = self.make_memory_symbolic_normal(
                    callsite_label.clone(),
                    ADDRESS_TYPE.clone(),
                    f_context,
                    vm
                );
                let tmp_callsite = self.make_temporary(f_context, ADDRESS_TYPE.clone(), vm);
                self.backend
                    .emit_lea_r64(&tmp_callsite, &tmp_callsite_addr_loc);

                // push return address
                self.backend.emit_push_r64(&tmp_callsite);
                // push base pointer
                self.backend.emit_push_r64(&x86_64::RBP);

                // save current SP
                self.emit_store_base_offset(
                    &cur_stackref,
                    *thread::MUSTACK_SP_OFFSET as i32,
                    &x86_64::RSP,
                    vm
                );
            }
        }

        // load the new sp from the swappee
        let new_sp = self.make_temporary(f_context, ADDRESS_TYPE.clone(), vm);
        self.emit_load_base_offset(&new_sp, &swappee, *thread::MUSTACK_SP_OFFSET as i32, vm);
        // swap to new stack
        self.backend.emit_mov_r_r(&x86_64::RSP, &new_sp);

        // now we are on the new stack
        // prepare arguments for continuation

        if is_exception {
            // we only have one argument (the exception object)
            debug_assert!(arg_values.len() == 1);
            // push RSP as the second argument (as we will call throw_exception_internal(),
            // which takes two arguments: exception object, and frame cursor)
            let tmp_framecursor = self.make_temporary(f_context, ADDRESS_TYPE.clone(), vm);
            self.backend.emit_mov_r_r(&tmp_framecursor, &x86_64::RSP);
            arg_values.push(tmp_framecursor);
        }

        // compute call convention
        let arg_tys = arg_values.iter().map(|x| x.ty.clone()).collect();
        let callconv = swapstack::compute_arguments(&arg_tys);

        // pass stack arguments
        let mut stack_args = vec![];
        for i in 0..callconv.len() {
            let ref cc = callconv[i];
            let ref arg = arg_values[i];

            match cc {
                &CallConvResult::STACK => stack_args.push(arg.clone()),
                _ => {}
            }
        }
        self.emit_store_stack_values(
            &stack_args,
            Some((&new_sp, 2 * WORD_SIZE as i32)),
            MU_CALL_CONVENTION,
            vm
        );

        // move arguments that can be passed by registers
        let (arg_regs, _) =
            self.emit_precall_convention_regs_only(&arg_values, &callconv, f_context, vm);

        if is_kill {
            // save first GPR argument
            self.backend.emit_push_r64(&x86_64::RDI);

            // kill the old stack
            self.emit_runtime_entry(
                &entrypoints::SAFECALL_KILL_STACK,
                vec![cur_stackref],
                None,
                Some(node),
                f_content,
                f_context,
                vm
            );

            // restore first GPR
            self.backend.emit_pop_r64(&x86_64::RDI);
        }

        // arguments are ready, we are starting continuation
        let potential_exception_dest = match resumption {
            Some(ref resumption) => {
                let target_id = resumption.exn_dest.target.id();
                Some(f_content.get_block(target_id).name())
            }
            None => None
        };

        if is_exception {
            // we will call into throw_exception_internal

            // reserve space on the new stack for exception handling routine to store
            // callee saved registers (as in muentry_throw_exception)
            self.backend
                .emit_sub_r_imm(&x86_64::RSP, (WORD_SIZE * CALLEE_SAVED_COUNT) as i32);

            // throws an exception
            // we are calling the internal ones as return address and base pointer are already
            // on the stack. and also we are saving all usable registers
            self.backend.emit_call_jmp(
                callsite_label.clone(),
                entrypoints::THROW_EXCEPTION_INTERNAL.aot.to_relocatable(),
                potential_exception_dest,
                arg_regs,
                x86_64::ALL_USABLE_MACHINE_REGS.to_vec(),
                true
            );
        } else {
            // pop RBP
            self.backend.emit_pop_r64(&x86_64::RBP);

            // pop resumption address into rax
            self.backend.emit_pop_r64(&x86_64::RAX);

            // push 0 - a fake return address
            // so that SP+8 is 16 bytes aligned (the same requirement as entring a function)
            self.backend.emit_push_imm32(0i32);

            // jmp to the resumption
            self.backend.emit_call_jmp_indirect(
                callsite_label.clone(),
                &x86_64::RAX,
                potential_exception_dest,
                arg_regs,
                x86_64::ALL_USABLE_MACHINE_REGS.to_vec()
            );
        }

        // the resumption starts here
        if !is_kill {
            // record this callsite
            let target_block_id = match resumption {
                Some(resumption) => resumption.exn_dest.target.id(),
                None => 0
            };
            self.current_callsites
                .push_back((callsite_label, target_block_id, res_stack_size));

            if resumption.is_some() {
                // the call instruction ends the block
                self.finish_block();

                let block = make_block_name(&node.name(), "stack_resumption");
                self.start_block(block);
            }

            // pop the fake return address
            self.backend.emit_add_r_imm(&x86_64::RSP, 8);

            // unload return values (arguments)
            let return_values = res_vals;
            let return_tys = return_values.iter().map(|x| x.ty.clone()).collect();
            let callconv = callconv::swapstack::compute_return_values(&return_tys);

            // values by registers
            self.emit_unload_values(
                &return_values,
                &callconv,
                &res_locs,
                None,
                false,
                f_context,
                vm
            );

            // collapse return value on stack
            if res_stack_size != 0 {
                self.backend
                    .emit_add_r_imm(&x86_64::RSP, res_stack_size as i32);
            }
        }
    }

    /// processes call arguments - gets P<Value> from P<TreeNode>, emits code if necessary
    fn process_call_arguments(
        &mut self,
        calldata: &CallData,
        ops: &Vec<P<TreeNode>>,
        f_content: &FunctionContent,
        f_context: &mut FunctionContext,
        vm: &VM
    ) -> Vec<P<Value>> {
        self.process_arguments(&calldata.args, ops, f_content, f_context, vm)
    }

    /// process arguments - gets P<Value> from P<TreeNode>, emits code if necessary
    fn process_arguments(
        &mut self,
        args: &Vec<OpIndex>,
        ops: &Vec<P<TreeNode>>,
        f_content: &FunctionContent,
        f_context: &mut FunctionContext,
        vm: &VM
    ) -> Vec<P<Value>> {
        let mut ret = vec![];

        for arg_index in args {
            let ref arg = ops[*arg_index];

            if self.match_iimm(arg) {
                let arg = self.node_iimm_to_value(arg);
                ret.push(arg);
            } else if self.match_ireg(arg) {
                let arg = self.emit_ireg(arg, f_content, f_context, vm);
                ret.push(arg);
            } else if self.match_ireg_ex(arg) {
                let arg = self.emit_ireg_ex_as_one(arg, f_content, f_context, vm);
                ret.push(arg);
            } else if self.match_fpreg(arg) {
                let arg = self.emit_fpreg(arg, f_content, f_context, vm);
                ret.push(arg);
            } else {
                unimplemented!();
            }
        }

        ret
    }

    /// processes a Destination clause, emits move to pass arguments to the destination
    /// It is problematic if we call process_dest() for multiway branches, but we have
    /// a remove_phi_node pass to insert intermediate blocks to move arguments so that
    /// the destination clause should have no arguments
    fn process_dest(
        &mut self,
        ops: &Vec<P<TreeNode>>,
        dest: &Destination,
        f_content: &FunctionContent,
        f_context: &mut FunctionContext,
        vm: &VM
    ) {
        for i in 0..dest.args.len() {
            let ref dest_arg = dest.args[i];
            match dest_arg {
                &DestArg::Normal(op_index) => {
                    let ref arg = ops[op_index];
                    let ref target_args = f_content
                        .get_block(dest.target.id())
                        .content
                        .as_ref()
                        .unwrap()
                        .args;
                    let ref target_arg = target_args[i];

                    self.emit_move_node_to_value(target_arg, &arg, f_content, f_context, vm);
                }
                &DestArg::Freshbound(_) => unimplemented!()
            }
        }
    }

    /// emits a prologue for a Mu function:
    /// 1. builds linkage with last frame (push rbp, move rsp->rbp)
    /// 2. reserves spaces for current frame (frame size unknown yet)
    /// 3. pushes callee saved registers (this may get eliminated if we do not use
    ///    callee saved for this function)
    /// 4. marshalls arguments (from argument register/stack to temporaries)
    fn emit_common_prologue(
        &mut self,
        sig: &MuFuncSig,
        args: &Vec<P<Value>>,
        f_context: &mut FunctionContext,
        vm: &VM
    ) {
        let block_name = Arc::new(format!("{}:{}", self.current_fv_name, PROLOGUE_BLOCK_NAME));
        self.backend.start_block(block_name.clone());

        // push rbp
        self.backend.emit_push_r64(&x86_64::RBP);
        if vm.vm_options.flag_emit_debug_info {
            self.backend.add_cfi_def_cfa_offset(16i32);
            self.backend.add_cfi_offset(&x86_64::RBP, -16i32);
        }

        // mov rsp -> rbp
        self.backend.emit_mov_r_r(&x86_64::RBP, &x86_64::RSP);
        if vm.vm_options.flag_emit_debug_info {
            self.backend.add_cfi_def_cfa_register(&x86_64::RBP);
        }

        // reserve spaces for current frame
        // add x, rbp -> rbp (x is negative, however we do not know x now)
        self.backend.emit_frame_grow();

        // push all callee-saved registers
        {
            let frame = self.current_frame.as_mut().unwrap();

            let rbp = x86_64::RBP.extract_ssa_id().unwrap();
            for i in 0..x86_64::CALLEE_SAVED_GPRS.len() {
                let ref reg = x86_64::CALLEE_SAVED_GPRS[i];
                // not pushing rbp (as we have done that)
                if reg.extract_ssa_id().unwrap() != rbp {
                    trace!("allocate frame slot for reg {}", reg);

                    let loc = frame.alloc_slot_for_callee_saved_reg(reg.clone(), vm);
                    self.backend.emit_mov_mem_r_callee_saved(&loc, &reg);
                }
            }
        }

        // unload arguments by registers
        {
            use compiler::backend::x86_64::callconv::mu;

            let callconv = mu::compute_arguments(&sig.arg_tys);
            let (_, stack_arg_offsets) = mu::compute_stack_args(&sig.arg_tys, vm);
            debug!("sig = {}", sig);
            debug!("args = {:?}", args);
            debug!("callconv = {:?}", args);

            // deal with arguments passed by stack
            // initial stack arg is at RBP+16
            //   arg           <- RBP + 16
            //   return addr
            //   old RBP       <- RBP
            self.emit_unload_values(
                args,
                &callconv,
                &stack_arg_offsets,
                Some((&x86_64::RBP, 16)),
                true,
                f_context,
                vm
            );
        }

        self.backend.end_block(block_name);
    }

    /// emits an epilogue for a Mu function:
    /// 1. marshalls return values (from temporaries to return registers/stack)
    /// 2. restores callee saved registers
    /// 3. collapses frame
    /// 4. restore rbp
    ///
    /// Note that we do not have a single exit block so the epilogue is
    /// not a block but a sequence of instructions inserted before return (see Issue #30)
    fn emit_common_epilogue(
        &mut self,
        ret_inst: &Instruction,
        f_content: &FunctionContent,
        f_context: &mut FunctionContext,
        vm: &VM
    ) {
        use compiler::backend::x86_64::callconv::mu;

        // prepare return regs
        let ref ops = ret_inst.ops;
        let ret_val_indices = match ret_inst.v {
            Instruction_::Return(ref vals) => vals,
            _ => panic!("expected ret inst")
        };

        let callconv = mu::compute_return_values(&self.current_sig.as_ref().unwrap().ret_tys);
        debug_assert!(callconv.len() == ret_val_indices.len());

        for i in 0..callconv.len() {
            let ref cc = callconv[i];
            let ref ret_val = ops[ret_val_indices[i]];

            match cc {
                &CallConvResult::GPR(ref reg) => {
                    if self.match_iimm(ret_val) {
                        let imm_ret_val = self.node_iimm_to_i32(ret_val);
                        self.backend.emit_mov_r_imm(reg, imm_ret_val);
                    } else if self.match_ireg(ret_val) {
                        let reg_ret_val = self.emit_ireg(ret_val, f_content, f_context, vm);
                        self.backend.emit_mov_r_r(reg, &reg_ret_val);
                    } else {
                        unreachable!()
                    }
                }
                &CallConvResult::GPREX(ref reg_l, ref reg_h) => {
                    if self.match_ireg_ex(ret_val) {
                        let (ret_val_l, ret_val_h) =
                            self.emit_ireg_ex(ret_val, f_content, f_context, vm);

                        self.backend.emit_mov_r_r(reg_l, &ret_val_l);
                        self.backend.emit_mov_r_r(reg_h, &ret_val_h);
                    } else {
                        unreachable!()
                    }
                }
                &CallConvResult::FPR(ref reg) => {
                    let reg_ret_val = self.emit_fpreg(ret_val, f_content, f_context, vm);
                    if reg_ret_val.ty.is_double() {
                        self.backend.emit_movsd_f64_f64(reg, &reg_ret_val);
                    } else if reg_ret_val.ty.is_float() {
                        self.backend.emit_movss_f32_f32(reg, &reg_ret_val);
                    } else {
                        unreachable!()
                    }
                }
                &CallConvResult::STACK => unimplemented!()
            }
        }

        // pop all callee-saved registers - reverse order
        {
            let frame = self.current_frame.as_mut().unwrap();
            for i in (0..x86_64::CALLEE_SAVED_GPRS.len()).rev() {
                let ref reg = x86_64::CALLEE_SAVED_GPRS[i];
                let reg_id = reg.extract_ssa_id().unwrap();
                if reg_id != x86_64::RBP.extract_ssa_id().unwrap() {
                    let loc = frame
                        .allocated
                        .get(&reg_id)
                        .unwrap()
                        .make_memory_op(reg.ty.clone(), vm);
                    self.backend.emit_mov_r_mem_callee_saved(&reg, &loc);
                }
            }
        }

        // frame shrink
        // RBP -> RSP
        self.backend.emit_mov_r_r(&x86_64::RSP, &x86_64::RBP);

        // pop rbp
        self.backend.emit_pop_r64(&x86_64::RBP);
    }

    /// matches a comparison result pattern
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

    /// emits code for a comparison result pattern
    fn emit_cmp_res(
        &mut self,
        cond: &TreeNode,
        f_content: &FunctionContent,
        f_context: &mut FunctionContext,
        vm: &VM
    ) -> op::CmpOp {
        match cond.v {
            TreeNode_::Instruction(ref inst) => {
                let ref ops = inst.ops;

                match inst.v {
                    Instruction_::CmpOp(op, op1, op2) => {
                        let op1 = &ops[op1];
                        let op2 = &ops[op2];

                        if op.is_int_cmp() {
                            if self.match_iimm(op1) && self.match_iimm(op2) {
                                // comparing two immediate numbers
                                let ty = op1.as_value().ty.clone();

                                let tmp_op1 = self.make_temporary(f_context, ty, vm);
                                let iimm_op1 = self.node_iimm_to_i32(op1);
                                self.backend.emit_mov_r_imm(&tmp_op1, iimm_op1);

                                let iimm_op2 = self.node_iimm_to_i32(op2);
                                self.backend.emit_cmp_imm_r(iimm_op2, &tmp_op1);

                                return op;
                            } else if self.match_ireg(op1) && self.match_iimm(op2) {
                                // comparing ireg and an immediate number (order matters)
                                let reg_op1 = self.emit_ireg(op1, f_content, f_context, vm);
                                let iimm_op2 = self.node_iimm_to_i32(op2);

                                self.backend.emit_cmp_imm_r(iimm_op2, &reg_op1);

                                return op;
                            } else if self.match_ireg(op1) && self.match_mem(op2) {
                                let reg_op1 = self.emit_ireg(op1, f_content, f_context, vm);
                                let mem_op2 = self.emit_mem(op2, f_content, f_context, vm);

                                self.backend.emit_cmp_mem_r(&mem_op2, &reg_op1);

                                return op;
                            } else if self.match_mem(op1) && self.match_ireg(op2) {
                                let mem_op1 = self.emit_mem(op1, f_content, f_context, vm);
                                let reg_op2 = self.emit_ireg(op2, f_content, f_context, vm);

                                self.backend.emit_cmp_r_mem(&reg_op2, &mem_op1);

                                return op;
                            } else if self.match_ireg(op1) && self.match_ireg(op2) {
                                // comparing two iregs (general case)
                                let reg_op1 = self.emit_ireg(op1, f_content, f_context, vm);
                                let reg_op2 = self.emit_ireg(op2, f_content, f_context, vm);

                                self.backend.emit_cmp_r_r(&reg_op2, &reg_op1);

                                return op;
                            } else if self.match_ireg_ex(op1) && self.match_ireg_ex(op2) {
                                // comparing two int128 integers
                                let (op1_l, op1_h) =
                                    self.emit_ireg_ex(op1, f_content, f_context, vm);
                                let (op2_l, op2_h) =
                                    self.emit_ireg_ex(op2, f_content, f_context, vm);

                                match op {
                                    CmpOp::EQ | CmpOp::NE => {
                                        // mov op1_h -> h
                                        let h =
                                            self.make_temporary(f_context, UINT64_TYPE.clone(), vm);
                                        self.backend.emit_mov_r_r(&h, &op1_h);

                                        // xor op2_h, h -> h
                                        self.backend.emit_xor_r_r(&h, &op2_h);

                                        // mov op1_l -> l
                                        let l =
                                            self.make_temporary(f_context, UINT64_TYPE.clone(), vm);
                                        self.backend.emit_mov_r_r(&l, &op1_l);

                                        // xor op2_l, l -> l
                                        self.backend.emit_xor_r_r(&l, &op2_l);

                                        // or h, l -> l
                                        self.backend.emit_or_r_r(&l, &h);

                                        return op;
                                    }
                                    CmpOp::UGT | CmpOp::SGT => {
                                        // cmp op1_l, op2_l
                                        self.backend.emit_cmp_r_r(&op1_l, &op2_l);

                                        // mov op2_h -> t
                                        // sbb t, op1_h -> t
                                        let t =
                                            self.make_temporary(f_context, UINT64_TYPE.clone(), vm);
                                        self.backend.emit_mov_r_r(&t, &op2_h);
                                        self.backend.emit_sbb_r_r(&t, &op1_h);

                                        match op {
                                            CmpOp::UGT => CmpOp::ULT,
                                            CmpOp::SGT => CmpOp::SLT,
                                            _ => unreachable!()
                                        }
                                    }
                                    CmpOp::UGE | CmpOp::SGE => {
                                        // cmp op2_l, op1_l
                                        self.backend.emit_cmp_r_r(&op2_l, &op1_l);

                                        // mov op1_h -> t
                                        // sbb t, op2_h -> t
                                        let t =
                                            self.make_temporary(f_context, UINT64_TYPE.clone(), vm);
                                        self.backend.emit_mov_r_r(&t, &op1_h);
                                        self.backend.emit_sbb_r_r(&t, &op2_h);

                                        op
                                    }
                                    CmpOp::ULT | CmpOp::SLT => {
                                        // cmp op2_l, op1_l
                                        self.backend.emit_cmp_r_r(&op2_l, &op1_l);

                                        // mov op1_h -> t
                                        // sbb t, op2_h -> t
                                        let t =
                                            self.make_temporary(f_context, UINT64_TYPE.clone(), vm);
                                        self.backend.emit_mov_r_r(&t, &op1_h);
                                        self.backend.emit_sbb_r_r(&t, &op2_h);

                                        op
                                    }
                                    CmpOp::ULE | CmpOp::SLE => {
                                        // cmp op2_l, op1_l
                                        self.backend.emit_cmp_r_r(&op2_l, &op1_l);

                                        // mov op1_h -> t
                                        // sbb t, op2_h -> t
                                        let t =
                                            self.make_temporary(f_context, UINT64_TYPE.clone(), vm);
                                        self.backend.emit_mov_r_r(&t, &op1_h);
                                        self.backend.emit_sbb_r_r(&t, &op2_h);

                                        match op {
                                            CmpOp::ULE => CmpOp::UGE,
                                            CmpOp::SLE => CmpOp::SGE,
                                            _ => unreachable!()
                                        }
                                    }

                                    _ => {
                                        panic!("expected CmpOp for int128 integers, found {}", cond)
                                    }
                                }
                            } else {
                                panic!("expect ireg/ireg_ex for integer comparison, found {}", cond)
                            }
                        } else {
                            // floating point comparison
                            let reg_op1 = self.emit_fpreg(op1, f_content, f_context, vm);
                            let reg_op2 = self.emit_fpreg(op2, f_content, f_context, vm);

                            match op {
                                op::CmpOp::FOEQ |
                                op::CmpOp::FOGT |
                                op::CmpOp::FOGE |
                                op::CmpOp::FOLT |
                                op::CmpOp::FOLE |
                                op::CmpOp::FONE => {
                                    match reg_op1.ty.v {
                                        MuType_::Double => {
                                            self.backend.emit_comisd_f64_f64(&reg_op2, &reg_op1)
                                        }
                                        MuType_::Float => {
                                            self.backend.emit_comiss_f32_f32(&reg_op2, &reg_op1)
                                        }
                                        _ => panic!("expect double or float")
                                    }

                                    op
                                }
                                op::CmpOp::FUEQ |
                                op::CmpOp::FUGT |
                                op::CmpOp::FUGE |
                                op::CmpOp::FULT |
                                op::CmpOp::FULE |
                                op::CmpOp::FUNE => {
                                    match reg_op1.ty.v {
                                        MuType_::Double => {
                                            self.backend.emit_ucomisd_f64_f64(&reg_op2, &reg_op1)
                                        }
                                        MuType_::Float => {
                                            self.backend.emit_ucomiss_f32_f32(&reg_op2, &reg_op1)
                                        }
                                        _ => panic!("expect double or float")
                                    }

                                    op
                                }
                                _ => {
                                    // FFALSE/FTRUE unimplemented
                                    unimplemented!()
                                }
                            }
                        }
                    }

                    _ => panic!("expect cmp res to emit")
                }
            }
            _ => panic!("expect cmp res to emit")
        }
    }

    /// matches an integer register pattern
    /// * temporaries that can be held in general purpose registers
    /// * instructions that generates exactly one result value that matches above
    fn match_ireg(&mut self, op: &TreeNode) -> bool {
        match op.v {
            TreeNode_::Instruction(ref inst) => {
                if inst.value.is_some() {
                    if inst.value.as_ref().unwrap().len() > 1 {
                        return false;
                    }

                    let ref value = inst.value.as_ref().unwrap()[0];

                    if RegGroup::get_from_value(&value) == RegGroup::GPR && value.is_reg() {
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }

            TreeNode_::Value(ref pv) => {
                match pv.v {
                    Value_::SSAVar(_) | Value_::Constant(_) => {
                        RegGroup::get_from_value(&pv) == RegGroup::GPR
                    }
                    Value_::Global(_) => {
                        // global is always a ireg (it is iref<T>)
                        true
                    }
                    _ => false
                }
            }
        }
    }

    /// matches an extended integer register (128 bits regsiter) pattern
    /// * temporaries that can be held in two general purpose registers
    /// * instructions that generates exactly one result value that matches above
    fn match_ireg_ex(&mut self, op: &TreeNode) -> bool {
        match op.v {
            TreeNode_::Instruction(ref inst) => {
                if inst.value.is_some() {
                    if inst.value.as_ref().unwrap().len() > 1 {
                        return false;
                    }

                    let ref value = inst.value.as_ref().unwrap()[0];

                    if RegGroup::get_from_value(&value) == RegGroup::GPREX && value.is_reg() {
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }

            TreeNode_::Value(ref pv) => RegGroup::get_from_value(&pv) == RegGroup::GPREX
        }
    }

    /// matches a floating point register pattern
    /// * temporaries that can be held in floating point registers
    /// * instructions that generates exactly one result value that matches above
    fn match_fpreg(&mut self, op: &TreeNode) -> bool {
        match op.v {
            TreeNode_::Instruction(ref inst) => {
                if inst.value.is_some() {
                    if inst.value.as_ref().unwrap().len() > 1 {
                        return false;
                    }

                    let ref value = inst.value.as_ref().unwrap()[0];

                    if RegGroup::get_from_value(&value) == RegGroup::FPR {
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }

            TreeNode_::Value(ref pv) => RegGroup::get_from_value(pv) == RegGroup::FPR
        }
    }

    /// emits code for an integer register pattern
    fn emit_ireg(
        &mut self,
        op: &TreeNode,
        f_content: &FunctionContent,
        f_context: &mut FunctionContext,
        vm: &VM
    ) -> P<Value> {
        match op.v {
            TreeNode_::Instruction(_) => {
                // recursively call instruction_select() on the node
                self.instruction_select(op, f_content, f_context, vm);

                // get first result as P<Value>
                self.get_result_value(op)
            }
            TreeNode_::Value(ref pv) => {
                match pv.v {
                    Value_::SSAVar(_) => {
                        // a temporary, simply returns it
                        pv.clone()
                    }
                    Value_::Constant(ref c) => {
                        let tmp = self.make_temporary(f_context, pv.ty.clone(), vm);
                        match c {
                            // an integer constant, puts it to a temporary
                            &Constant::Int(val) => {
                                if x86_64::is_valid_x86_imm(pv) {
                                    let val = self.value_iimm_to_i32(&pv);
                                    self.backend.emit_mov_r_imm(&tmp, val);
                                } else {
                                    assert!(tmp.ty.get_int_length().is_some());
                                    assert!(tmp.ty.get_int_length().unwrap() == 64);
                                    self.backend.emit_mov_r64_imm64(&tmp, val as i64);
                                }
                            }
                            &Constant::IntEx(ref vals) => {
                                let (tmp_l, tmp_h) = self.split_int128(&tmp, f_context, vm);

                                self.backend.emit_mov_r64_imm64(&tmp_l, vals[0] as i64);
                                self.backend.emit_mov_r64_imm64(&tmp_h, vals[1] as i64);
                            }
                            // a function reference, loads the funcref to a temporary
                            &Constant::FuncRef(ref func) => {
                                // our code for linux has one more level of indirection
                                // so we need a load for linux
                                // sel4-rumprun is the same as Linux here
                                if cfg!(feature = "sel4-rumprun") {
                                    let mem = self.get_mem_for_funcref(func.id(), vm);
                                    self.backend.emit_mov_r_mem(&tmp, &mem);
                                } else if cfg!(target_os = "macos") {
                                    let mem = self.get_mem_for_funcref(func.id(), vm);
                                    self.backend.emit_lea_r64(&tmp, &mem);
                                } else if cfg!(target_os = "linux") {
                                    let mem = self.get_mem_for_funcref(func.id(), vm);
                                    self.backend.emit_mov_r_mem(&tmp, &mem);
                                } else {
                                    unimplemented!()
                                }
                            }
                            // a null ref, puts 0 to a temporary
                            &Constant::NullRef => {
                                // xor a, a -> a will mess up register allocation validation
                                // since it uses a register with arbitrary value
                                // self.backend.emit_xor_r_r(&tmp, &tmp);

                                // for now, use mov -> a
                                self.backend.emit_mov_r_imm(&tmp, 0);
                            }
                            _ => panic!("expected a constant that fits ireg, found {}", c)
                        }

                        tmp
                    }
                    _ => panic!("value doesnt match with ireg: {}", pv)
                }
            }
        }
    }

    /// emits code for an extended integer register pattern
    fn emit_ireg_ex(
        &mut self,
        op: &TreeNode,
        f_content: &FunctionContent,
        f_context: &mut FunctionContext,
        vm: &VM
    ) -> (P<Value>, P<Value>) {
        match op.v {
            TreeNode_::Instruction(_) => {
                // recursively call instruction_select() on the node
                self.instruction_select(op, f_content, f_context, vm);

                // get first result as P<Value>
                let res = self.get_result_value(op);

                // split the value
                self.split_int128(&res, f_context, vm)
            }
            TreeNode_::Value(ref pv) => {
                match pv.v {
                    Value_::SSAVar(_) => self.split_int128(pv, f_context, vm),
                    Value_::Constant(Constant::IntEx(ref val)) => {
                        assert!(val.len() == 2);

                        let tmp_l = self.make_temporary(f_context, UINT64_TYPE.clone(), vm);
                        let tmp_h = self.make_temporary(f_context, UINT64_TYPE.clone(), vm);

                        self.backend.emit_mov_r64_imm64(&tmp_l, val[0] as i64);
                        self.backend.emit_mov_r64_imm64(&tmp_h, val[1] as i64);

                        (tmp_l, tmp_h)
                    }
                    _ => panic!("value doesnt match with ireg-ex: {}", pv)
                }
            }
        }
    }

    /// emits a 128-bit integer register as one temporary
    fn emit_ireg_ex_as_one(
        &mut self,
        op: &TreeNode,
        f_content: &FunctionContent,
        f_context: &mut FunctionContext,
        vm: &VM
    ) -> P<Value> {
        self.emit_ireg(op, f_content, f_context, vm)
    }

    /// emits code for a floating point register pattern
    fn emit_fpreg(
        &mut self,
        op: &TreeNode,
        f_content: &FunctionContent,
        f_context: &mut FunctionContext,
        vm: &VM
    ) -> P<Value> {
        match op.v {
            TreeNode_::Instruction(_) => {
                // recursively call instruction_select() on the node
                self.instruction_select(op, f_content, f_context, vm);

                // get the first result as P<Value>
                self.get_result_value(op)
            }
            TreeNode_::Value(ref pv) => {
                match pv.v {
                    Value_::SSAVar(_) => pv.clone(),
                    Value_::Constant(Constant::Double(_)) => {
                        let mem = self.get_mem_for_const(pv, vm);
                        let tmp_fp = self.make_temporary(f_context, DOUBLE_TYPE.clone(), vm);
                        self.backend.emit_movsd_f64_mem64(&tmp_fp, &mem);
                        tmp_fp
                    }
                    Value_::Constant(Constant::Float(_)) => {
                        let mem = self.get_mem_for_const(pv, vm);
                        let tmp_fp = self.make_temporary(f_context, FLOAT_TYPE.clone(), vm);
                        self.backend.emit_movss_f32_mem32(&tmp_fp, &mem);
                        tmp_fp
                    }
                    _ => panic!("expected fpreg")
                }
            }
        }
    }

    /// matches an integer const value
    fn match_iconst_any(&self, op: &TreeNode) -> bool {
        match op.v {
            TreeNode_::Value(ref pv) if pv.is_int_const() => true,
            _ => false
        }
    }

    /// matches an integer const zero
    fn match_iconst_zero(&mut self, op: &TreeNode) -> bool {
        match op.v {
            TreeNode_::Value(ref pv) if pv.is_const() => {
                if pv.is_int_const() {
                    pv.extract_int_const().unwrap() == 0
                } else if pv.is_int_ex_const() {
                    pv.extract_int_ex_const().iter().all(|x| *x == 0)
                } else {
                    false
                }
            }
            _ => false
        }
    }

    /// matches an integer const one
    fn match_iconst_one(&mut self, op: &TreeNode) -> bool {
        match op.v {
            TreeNode_::Value(ref pv) if pv.is_const() => {
                if pv.is_int_const() {
                    pv.extract_int_const().unwrap() == 1
                } else if pv.is_int_ex_const() {
                    let vals = pv.extract_int_ex_const();
                    vals[0] == 1 && vals[1..].iter().all(|x| *x == 0)
                } else {
                    false
                }
            }
            _ => false
        }
    }

    /// matches an integer that is power of 2
    fn match_iconst_p2(&mut self, op: &TreeNode) -> bool {
        match op.v {
            TreeNode_::Value(ref pv) if pv.is_const() => {
                if pv.is_int_const() {
                    math::is_power_of_two(pv.extract_int_const().unwrap() as usize).is_some()
                } else {
                    false
                }
            }
            _ => false
        }
    }

    fn node_iconst_to_p2(&mut self, op: &TreeNode) -> u8 {
        match op.v {
            TreeNode_::Value(ref pv) if pv.is_const() => {
                if pv.is_int_const() {
                    math::is_power_of_two(pv.extract_int_const().unwrap() as usize).unwrap()
                } else {
                    unreachable!()
                }
            }
            _ => unreachable!()
        }
    }

    /// matches a floatingpoint zero
    fn match_fconst_zero(&mut self, op: &TreeNode) -> bool {
        match op.v {
            TreeNode_::Value(ref pv) if pv.is_fp_const() => {
                match pv.v {
                    Value_::Constant(Constant::Float(val)) => val == 0f32,
                    Value_::Constant(Constant::Double(val)) => val == 0f64,
                    _ => false
                }
            }
            _ => false
        }
    }

    /// matches an integer immediate number pattern
    fn match_iimm(&mut self, op: &TreeNode) -> bool {
        match op.v {
            TreeNode_::Value(ref pv) if x86_64::is_valid_x86_imm(pv) => true,
            _ => false
        }
    }

    /// converts an integer immediate TreeNode to i32 constant
    fn node_iimm_to_i32(&mut self, op: &TreeNode) -> i32 {
        match op.v {
            TreeNode_::Value(ref pv) => self.value_iimm_to_i32(pv),
            _ => panic!("expected iimm")
        }
    }

    /// converts an integer immediate TreeNode to i32 constant
    /// and also returns its length (in bits)
    fn node_iimm_to_i32_with_len(&mut self, op: &TreeNode) -> (i32, usize) {
        match op.v {
            TreeNode_::Value(ref pv) => self.value_iimm_to_i32_with_len(pv),
            _ => panic!("expected iimm")
        }
    }

    /// converts an integer immediate TreeNode to P<Value>
    fn node_iimm_to_value(&mut self, op: &TreeNode) -> P<Value> {
        match op.v {
            TreeNode_::Value(ref pv) => pv.clone(),
            _ => panic!("expected iimm")
        }
    }

    /// converts an integer immediate P<Value> to i32 constant
    fn value_iimm_to_i32(&mut self, op: &P<Value>) -> i32 {
        self.value_iimm_to_i32_with_len(op).0
    }

    /// converts an integer immediate P<Value> to i32 constant
    /// and also returns the length of the int
    fn value_iimm_to_i32_with_len(&mut self, op: &P<Value>) -> (i32, usize) {
        let op_length = match op.ty.get_int_length() {
            Some(l) => l,
            None => panic!("expected an int")
        };

        let val = match op.v {
            Value_::Constant(Constant::Int(val)) => {
                debug_assert!(x86_64::is_valid_x86_imm(op));

                match op_length {
                    128 => panic!("cannot emit int128 as immediate"),
                    64 => val as i64 as i32,
                    32 => val as i32,
                    16 => val as i16 as i32,
                    1 | 8 => val as i8 as i32,
                    _ => panic!("unsupported int types")
                }
            }
            _ => panic!("expect iimm")
        };

        (val, op_length)
    }

    /// emits a TreeNode as address
    /// * for a temporary t, emits address [t]
    /// * for a global, emits code to fetch its address
    /// * for a memory operand, returns itself
    /// * for an instruction, emits a single address (if possible) using x86 addressing mode
    fn emit_node_addr_to_value(
        &mut self,
        op: &TreeNode,
        f_content: &FunctionContent,
        f_context: &mut FunctionContext,
        vm: &VM
    ) -> P<Value> {
        match op.v {
            TreeNode_::Value(ref pv) => {
                match pv.v {
                    Value_::SSAVar(_) => P(Value {
                        // use the variables as base address
                        hdr: MuEntityHeader::unnamed(vm.next_id()),
                        ty: pv.ty.get_referent_ty().unwrap(),
                        v: Value_::Memory(MemoryLocation::Address {
                            base: pv.clone(),
                            offset: None,
                            index: None,
                            scale: None
                        })
                    }),
                    Value_::Global(_) => {
                        if vm.is_doing_jit() {
                            // get address from vm
                            unimplemented!()
                        } else {
                            self.make_memory_symbolic_global(
                                pv.name(),
                                pv.ty.get_referent_ty().unwrap(),
                                f_context,
                                vm
                            )
                        }
                    }
                    Value_::Memory(_) => pv.clone(),
                    Value_::Constant(_) => unimplemented!()
                }
            }
            TreeNode_::Instruction(_) => self.emit_inst_addr_to_value(op, f_content, f_context, vm)
        }
    }

    /// emits the memory address P<Value> from an instruction
    fn emit_inst_addr_to_value(
        &mut self,
        op: &TreeNode,
        f_content: &FunctionContent,
        f_context: &mut FunctionContext,
        vm: &VM
    ) -> P<Value> {
        let mem = self.emit_inst_addr_to_value_inner(op, f_content, f_context, vm);

        P(Value {
            hdr: MuEntityHeader::unnamed(vm.next_id()),
            ty: ADDRESS_TYPE.clone(),
            v: Value_::Memory(mem)
        })
    }

    /// emits the memory address P<Value> from an instruction
    /// (this function will be called recursively)
    /// This function recognizes patterns from GetIRef, GetFieldIRef, GetVarPartIRef,
    /// ShiftIRef and GetElemIRef, and yields a single memory operand using addressing mode.
    fn emit_inst_addr_to_value_inner(
        &mut self,
        op: &TreeNode,
        f_content: &FunctionContent,
        f_context: &mut FunctionContext,
        vm: &VM
    ) -> MemoryLocation {
        match op.v {
            TreeNode_::Instruction(ref inst) => {
                let ref ops = inst.ops;

                match inst.v {
                    // GETIREF: returns operand
                    Instruction_::GetIRef(op_index) => {
                        trace!("MEM from GETIREF: {}", op);
                        let ref ref_op = ops[op_index];
                        self.emit_inst_addr_to_value_inner(ref_op, f_content, f_context, vm)
                    }
                    // GETFIELDIREF: adjust offset
                    Instruction_::GetFieldIRef { base, index, .. } => {
                        trace!("MEM from GETFIELDIREF: {}", op);
                        let ref base = ops[base];
                        let struct_ty = {
                            let ref iref_or_uptr_ty = base.as_value().ty;
                            match iref_or_uptr_ty.v {
                                MuType_::IRef(ref ty) | MuType_::UPtr(ref ty) => ty.clone(),
                                _ => {
                                    panic!(
                                        "expected the base for GetFieldIRef has a type of iref \
                                         or uptr, found type: {}",
                                        iref_or_uptr_ty
                                    )
                                }
                            }
                        };
                        let field_offset: i32 = self.get_field_offset(&struct_ty, index, vm);

                        let m = self.emit_inst_addr_to_value_inner(base, f_content, f_context, vm);
                        self.addr_const_offset_adjust(m, field_offset as u64, vm)
                    }
                    // GETVARPARTIREF: adjust offset
                    Instruction_::GetVarPartIRef { base, .. } => {
                        trace!("MEM from GETVARPARTIREF: {}", op);
                        let ref base = ops[base];
                        let struct_ty = match base.as_value().ty.get_referent_ty() {
                            Some(ty) => ty,
                            None => panic!("expecting an iref or uptr in GetVarPartIRef")
                        };
                        let fix_part_size = vm.get_backend_type_size(struct_ty.id());

                        let m = self.emit_inst_addr_to_value_inner(base, f_content, f_context, vm);
                        self.addr_const_offset_adjust(m, fix_part_size as u64, vm)
                    }
                    // SHIFTIREF:
                    // either adjust offset (if the index is constant), or use index/scale
                    Instruction_::ShiftIRef {
                        base,
                        offset: index,
                        ..
                    } |
                    Instruction_::GetElementIRef { base, index, .. } => {
                        let ref base = ops[base];
                        let ref index = ops[index];

                        let ref iref_ty = base.as_value().ty;
                        let base_ty = match iref_ty.get_referent_ty() {
                            Some(ty) => ty,
                            None => {
                                panic!(
                                    "expected op in ShiftIRef of type IRef, found type: {}",
                                    iref_ty
                                )
                            }
                        };
                        let ele_ty_size = {
                            let backend_ty = vm.get_backend_type_info(base_ty.id());
                            match inst.v {
                                Instruction_::ShiftIRef { .. } => {
                                    math::align_up(backend_ty.size, backend_ty.alignment)
                                }
                                Instruction_::GetElementIRef { .. } => {
                                    backend_ty.elem_size.unwrap()
                                }
                                _ => unreachable!()
                            }
                        };

                        if self.match_iimm(index) {
                            trace!("MEM from SHIFT/GETELEM IREF(constant): {}", op);
                            // byte offset is known at compile time,
                            // we can calculate it as a constant offset
                            let index = self.node_iimm_to_i32(index);
                            let shift_size = ele_ty_size as i32 * index;

                            let m =
                                self.emit_inst_addr_to_value_inner(base, f_content, f_context, vm);
                            self.addr_const_offset_adjust(m, shift_size as u64, vm)
                        } else {
                            trace!("MEM from SHIFT/GETELEM IREF(non constant): {}", op);

                            // we need to use index and scale

                            // index:
                            let tmp_index = self.emit_ireg(index, f_content, f_context, vm);
                            // make a copy, because we may need to alter index, and we dont want
                            // to change the original value
                            let tmp_index_copy =
                                self.make_temporary(f_context, tmp_index.ty.clone(), vm);
                            self.emit_move_value_to_value(&tmp_index_copy, &tmp_index);

                            // scale:
                            let scale: u8 = match ele_ty_size {
                                // if we can use x86 scale
                                8 | 4 | 2 | 1 => ele_ty_size as u8,
                                // if we can get byte offset by shifting
                                16 | 32 | 64 => {
                                    let shift = math::is_power_of_two(ele_ty_size).unwrap();
                                    // tmp_index_copy = tmp_index_copy << index
                                    self.backend.emit_shl_r_imm8(&tmp_index_copy, shift as i8);

                                    1
                                }
                                // otherwise we have to do multiplication
                                _ => {
                                    // mov ele_ty_size -> rax
                                    self.backend
                                        .emit_mov_r_imm(&x86_64::RAX, ele_ty_size as i32);
                                    // mul tmp_index_copy rax -> rdx:rax
                                    self.backend.emit_mul_r(&tmp_index_copy);
                                    // mov rax -> tmp_index_copy
                                    self.backend.emit_mov_r_r(&tmp_index_copy, &x86_64::RAX);

                                    1
                                }
                            };

                            let m =
                                self.emit_inst_addr_to_value_inner(base, f_content, f_context, vm);
                            self.addr_append_index_scale(m, tmp_index_copy, scale)
                        }
                    }
                    Instruction_::ConvOp {
                        operation: ConvOp::REFCAST,
                        operand,
                        ..
                    } |
                    Instruction_::ConvOp {
                        operation: ConvOp::PTRCAST,
                        operand,
                        ..
                    } |
                    Instruction_::Move(operand) => {
                        trace!("MEM from REF/PTRCAST/MOVE: {}", op);
                        let ref mem_op = inst.ops[operand];
                        self.emit_inst_addr_to_value_inner(mem_op, f_content, f_context, vm)
                    }
                    _ => {
                        warn!("MEM from general ireg inst: {}", op);
                        warn!("cannot fold into proper address mode");
                        let tmp = self.emit_ireg(op, f_content, f_context, vm);
                        MemoryLocation::Address {
                            base: tmp,
                            offset: None,
                            index: None,
                            scale: None
                        }
                    }
                }
            }
            TreeNode_::Value(ref pv) => {
                match pv.v {
                    Value_::SSAVar(_) => {
                        trace!("MEM from value/ssa: {}", op);
                        MemoryLocation::Address {
                            base: pv.clone(),
                            offset: None,
                            index: None,
                            scale: None
                        }
                    }
                    Value_::Constant(_) => {
                        error!("MEM from value/constant: {}", op);
                        panic!("trying to get memory from constant");
                    }
                    Value_::Global(_) => {
                        trace!("MEM from value/global: {}", op);
                        self.make_memory_symbolic_global(op.name(), pv.ty.clone(), f_context, vm)
                            .extract_memory_location()
                            .unwrap()
                    }
                    Value_::Memory(ref mem) => {
                        trace!("MEM from value/memory: {}", op);
                        mem.clone()
                    }
                }
            }
        }
    }

    /// adjusts the constant offset of a MemoryLocation
    fn addr_const_offset_adjust(
        &mut self,
        mem: MemoryLocation,
        more_offset: u64,
        vm: &VM
    ) -> MemoryLocation {
        match mem {
            MemoryLocation::Address {
                base,
                offset,
                index,
                scale
            } => {
                let new_offset = match offset {
                    Some(pv) => {
                        let old_offset = pv.extract_int_const().unwrap();
                        old_offset + more_offset
                    }
                    None => more_offset
                };

                MemoryLocation::Address {
                    base: base,
                    offset: Some(self.make_int64_const(new_offset, vm)),
                    index: index,
                    scale: scale
                }
            }
            _ => panic!("expected an address memory location")
        }
    }

    /// sets index and scale for a MemoryLocation,
    /// panics if this function tries to overwrite index and scale
    fn addr_append_index_scale(
        &mut self,
        mem: MemoryLocation,
        index_: P<Value>,
        scale_: u8
    ) -> MemoryLocation {
        match mem {
            MemoryLocation::Address {
                base,
                offset,
                index,
                scale
            } => {
                assert!(index.is_none());
                assert!(scale.is_none());

                MemoryLocation::Address {
                    base: base,
                    offset: offset,
                    index: Some(index_),
                    scale: Some(scale_)
                }
            }
            _ => panic!("expected an address memory location")
        }
    }

    /// matches a function reference/pointer pattern
    fn match_func_const(&mut self, op: &TreeNode) -> bool {
        match op.v {
            TreeNode_::Value(ref pv) => {
                let is_const = pv.is_const();
                let is_func = match &pv.ty.v {
                    &MuType_::FuncRef(_) | &MuType_::UFuncPtr(_) => true,
                    _ => false
                };

                is_const && is_func
            }
            _ => false
        }
    }

    /// converts a constant function reference to its function ID
    fn node_funcref_const_to_id(&mut self, op: &TreeNode) -> MuID {
        match op.v {
            TreeNode_::Value(ref pv) => {
                match &pv.v {
                    &Value_::Constant(Constant::FuncRef(ref hdr)) => hdr.id(),
                    _ => panic!("expected a funcref const")
                }
            }
            _ => panic!("expected a funcref const")
        }
    }

    /// matches a memory location pattern
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
                    Instruction_::Load { .. } => true,
                    _ => false
                }
            }
        }
    }

    /// emits code for a memory location pattern
    #[allow(unused_variables)]
    fn emit_mem(
        &mut self,
        op: &TreeNode,
        f_content: &FunctionContent,
        f_context: &mut FunctionContext,
        vm: &VM
    ) -> P<Value> {
        match op.v {
            TreeNode_::Value(ref pv) => {
                match pv.v {
                    Value_::Memory(_) => pv.clone(),
                    Value_::Global(ref ty) => {
                        self.make_memory_symbolic_global(op.name(), ty.clone(), f_context, vm)
                    }
                    _ => unimplemented!()
                }
            }
            TreeNode_::Instruction(ref inst) => {
                match inst.v {
                    Instruction_::Load { mem_loc, .. } => {
                        self.emit_node_addr_to_value(&inst.ops[mem_loc], f_content, f_context, vm)
                    }
                    _ => panic!("only Load instruction can match memory operand")
                }
            }
        }
    }

    /// returns the result P<Value> of a node
    /// * if the node is an instruction, returns its first result value
    /// * if the node is a value, returns the value
    fn get_result_value(&mut self, node: &TreeNode) -> P<Value> {
        match node.v {
            TreeNode_::Instruction(ref inst) => {
                if inst.value.is_some() {
                    let ref value = inst.value.as_ref().unwrap()[0];

                    if inst.value.as_ref().unwrap().len() > 1 {
                        warn!(
                            "retrieving value from a node with more than one value: {}, \
                             use the first value: {}",
                            node,
                            value
                        );
                    }

                    value.clone()
                } else {
                    panic!("expected result from the node {}", node);
                }
            }

            TreeNode_::Value(ref pv) => pv.clone()
        }
    }

    /// emits a move instruction from a TreeNode to a P<Value>
    /// This function matches source (TreeNode) pattern, emits its code, and then
    /// recognizes different source operands and destination operands, and emits
    /// corresponding move instruction
    fn emit_move_node_to_value(
        &mut self,
        dest: &P<Value>,
        src: &TreeNode,
        f_content: &FunctionContent,
        f_context: &mut FunctionContext,
        vm: &VM
    ) {
        // because src is a TreeNode, we may need to emit code to turn the node
        // into a value before calling emit_move_value_to_value()

        let ref dst_ty = dest.ty;
        if RegGroup::get_from_ty(&dst_ty) == RegGroup::GPR {
            if self.match_iimm(src) {
                // source is an immediate number
                let (src_imm, src_len) = self.node_iimm_to_i32_with_len(src);
                if RegGroup::get_from_value(&dest) == RegGroup::GPR && dest.is_reg() {
                    // move from immediate to register
                    self.backend.emit_mov_r_imm(dest, src_imm);
                } else if dest.is_mem() {
                    // move from immediate to memory
                    self.backend.emit_mov_mem_imm(dest, src_imm, src_len);
                } else {
                    panic!("unexpected dest: {}", dest);
                }
            } else if self.match_ireg(src) {
                // source is a register
                let src_reg = self.emit_ireg(src, f_content, f_context, vm);
                self.emit_move_value_to_value(dest, &src_reg);
            } else {
                panic!("expected src: {}", src);
            }
        } else if RegGroup::get_from_ty(&dst_ty) == RegGroup::GPREX {
            if self.match_ireg_ex(src) {
                let (op_l, op_h) = self.emit_ireg_ex(src, f_content, f_context, vm);
                let (res_l, res_h) = self.split_int128(dest, f_context, vm);

                self.backend.emit_mov_r_r(&res_l, &op_l);
                self.backend.emit_mov_r_r(&res_h, &op_h);
            } else {
                panic!("expected src as ireg_ex: {}", src);
            }
        } else if RegGroup::get_from_ty(&dst_ty) == RegGroup::FPR {
            if self.match_fpreg(src) {
                let src_reg = self.emit_fpreg(src, f_content, f_context, vm);
                self.emit_move_value_to_value(dest, &src_reg)
            } else {
                panic!("unexpected fp src: {}", src);
            }
        } else {
            warn!("move node {} to value {} unimplemented", src, dest);
            unimplemented!()
        }
    }

    /// emits a move instruction from between two P<Value>
    fn emit_move_value_to_value(&mut self, dest: &P<Value>, src: &P<Value>) {
        let ref src_ty = src.ty;

        if RegGroup::get_from_ty(&src_ty) == RegGroup::GPR {
            // gpr mov
            if dest.is_reg() && src.is_reg() {
                // reg -> reg
                self.backend.emit_mov_r_r(dest, src);
            } else if dest.is_reg() && src.is_mem() {
                // mem -> reg
                self.backend.emit_mov_r_mem(dest, src);
            } else if dest.is_reg() && src.is_const() {
                // imm -> reg
                let imm = self.value_iimm_to_i32(src);
                self.backend.emit_mov_r_imm(dest, imm);
            } else if dest.is_mem() && src.is_reg() {
                // reg -> mem
                self.backend.emit_mov_mem_r(dest, src);
            } else if dest.is_mem() && src.is_const() {
                // imm -> mem
                if x86_64::is_valid_x86_imm(src) {
                    let (imm, len) = self.value_iimm_to_i32_with_len(src);
                    self.backend.emit_mov_mem_imm(dest, imm, len);
                } else {
                    if src.is_const_zero() {
                        self.backend.emit_mov_mem_imm(dest, 0, WORD_SIZE * 8);
                    } else {
                        unimplemented!()
                        // TODO: we need f_context to create temporaries
                        // let imm64 = src.extract_int_const().unwrap();
                        // let tmp = self.make_temporary(f_context, UINT64_TYPE.clone(), vm);
                        // self.backend.emit_mov_r64_imm64(&tmp, imm64);
                        // self.backend.emit_mov_mem_r(&dest, &tmp);
                    }
                }
            } else if dest.is_mem() && src.is_mem() {
                // mem -> mem (need a temporary)
                unimplemented!();
            } else {
                panic!("unexpected gpr mov between {} -> {}", src, dest);
            }
        } else if RegGroup::get_from_ty(&src_ty) == RegGroup::GPREX {
            unimplemented!()
        } else if RegGroup::get_from_ty(&src_ty) == RegGroup::FPR {
            // fpr mov
            match src_ty.v {
                MuType_::Double => {
                    if dest.is_reg() && src.is_reg() {
                        // reg -> reg
                        self.backend.emit_movsd_f64_f64(dest, src);
                    } else if dest.is_reg() && src.is_mem() {
                        // mem -> reg
                        self.backend.emit_movsd_f64_mem64(dest, src);
                    } else if dest.is_reg() && src.is_const() {
                        // const -> reg
                        unimplemented!()
                    } else if dest.is_mem() && src.is_reg() {
                        // reg -> mem
                        self.backend.emit_movsd_mem64_f64(dest, src);
                    } else if dest.is_mem() && src.is_const() {
                        // const -> mem
                        unimplemented!()
                    } else if dest.is_mem() && src.is_mem() {
                        // mem -> mem
                        unimplemented!()
                    } else {
                        panic!("unexpected fpr mov between {} -> {}", src, dest);
                    }
                }
                MuType_::Float => {
                    if dest.is_reg() && src.is_reg() {
                        // reg -> reg
                        self.backend.emit_movss_f32_f32(dest, src);
                    } else if dest.is_reg() && src.is_mem() {
                        // mem -> reg
                        self.backend.emit_movss_f32_mem32(dest, src);
                    } else if dest.is_reg() && src.is_const() {
                        // const -> reg
                        unimplemented!()
                    } else if dest.is_mem() && src.is_reg() {
                        // reg -> mem
                        self.backend.emit_movss_mem32_f32(dest, src);
                    } else if dest.is_mem() && src.is_const() {
                        // const -> mem
                        unimplemented!()
                    } else if dest.is_mem() && src.is_mem() {
                        // mem -> mem
                        unimplemented!()
                    } else {
                        panic!("unexpected fpr mov between {} -> {}", src, dest);
                    }
                }
                _ => panic!("expect double or float")
            }
        } else {
            warn!("mov of type {} unimplemented", src_ty);
            unimplemented!()
        }
    }

    fn emit_clear_value(&mut self, val: &P<Value>, f_context: &mut FunctionContext, vm: &VM) {
        let ref val_ty = val.ty;

        if RegGroup::get_from_ty(val_ty) == RegGroup::GPR {
            self.backend.emit_xor_r_r(val, val);
        } else if RegGroup::get_from_ty(val_ty) == RegGroup::GPREX {
            let (val_l, val_h) = self.split_int128(val, f_context, vm);

            self.backend.emit_xor_r_r(&val_l, &val_l);
            self.backend.emit_xor_r_r(&val_h, &val_h);
        } else if RegGroup::get_from_ty(val_ty) == RegGroup::FPR {
            if val_ty.is_float() {
                self.backend.emit_xorps_f32_f32(val, val);
            } else if val_ty.is_double() {
                self.backend.emit_xorpd_f64_f64(val, val);
            } else {
                panic!("expect double or float")
            }
        } else {
            unimplemented!()
        }
    }

    /// emits code to get exception object from thread local storage
    fn emit_landingpad(
        &mut self,
        exception_arg: &P<Value>,
        f_content: &FunctionContent,
        f_context: &mut FunctionContext,
        vm: &VM
    ) {
        // get thread local and add offset to get exception_obj
        let tl = self.emit_get_threadlocal(None, f_content, f_context, vm);
        self.emit_load_base_offset(exception_arg, &tl, *thread::EXCEPTION_OBJ_OFFSET as i32, vm);
    }

    /// gets field offset of a struct type
    fn get_field_offset(&mut self, ty: &P<MuType>, index: usize, vm: &VM) -> i32 {
        let ty_info = vm.get_backend_type_info(ty.id());
        let layout = match ty_info.struct_layout.as_ref() {
            Some(layout) => layout,
            None => panic!("a struct type does not have a layout yet: {:?}", ty_info)
        };

        assert!(layout.len() > index);
        layout[index] as i32
    }

    /// creates a callsite label that is globally unique
    fn new_callsite_label(&mut self, cur_node: Option<&TreeNode>) -> MuName {
        let ret = {
            if cur_node.is_some() {
                make_block_name(
                    &cur_node.unwrap().name(),
                    format!("callsite_{}", self.current_callsite_id).as_str()
                )
            } else {
                Arc::new(format!(
                    "{}:callsite_{}",
                    self.current_fv_name,
                    self.current_callsite_id
                ))
            }
        };
        self.current_callsite_id += 1;
        ret
    }

    /// puts a constant in memory, and returns its memory location P<Value>
    fn get_mem_for_const(&mut self, val: &P<Value>, vm: &VM) -> P<Value> {
        let id = val.id();

        if self.current_constants_locs.contains_key(&id) {
            self.current_constants_locs.get(&id).unwrap().clone()
        } else {
            let const_value_loc = vm.allocate_const(val);
            let const_mem_val = match const_value_loc {
                ValueLocation::Relocatable(_, ref name) => {
                    P(Value {
                        hdr: MuEntityHeader::unnamed(vm.next_id()),
                        ty: ADDRESS_TYPE.clone(),
                        v: Value_::Memory(MemoryLocation::Symbolic {
                            base: Some(x86_64::RIP.clone()),
                            label: name.clone(),
                            is_global: false,
                            is_native: false
                        })
                    })
                }
                _ => panic!("expecting relocatable location, found {}", const_value_loc)
            };

            self.current_constants.insert(id, val.clone());
            self.current_constants_locs
                .insert(id, const_mem_val.clone());

            const_mem_val
        }
    }

    /// returns a memory location P<Value> for a function reference
    #[cfg(feature = "aot")]
    fn get_mem_for_funcref(&mut self, func_id: MuID, vm: &VM) -> P<Value> {
        let func_name = vm.get_name_for_func(func_id);

        P(Value {
            hdr: MuEntityHeader::unnamed(vm.next_id()),
            ty: ADDRESS_TYPE.clone(),
            v: Value_::Memory(MemoryLocation::Symbolic {
                base: Some(x86_64::RIP.clone()),
                label: func_name,
                is_global: true,
                is_native: false
            })
        })
    }

    /// splits a 128-bits integer into two 64-bits integers
    /// This function remembers mapping between 128-bits int and 64-bits ints, and always
    /// returns the same 64-bits split result for a 128-bit integer
    fn split_int128(
        &mut self,
        int128: &P<Value>,
        f_context: &mut FunctionContext,
        vm: &VM
    ) -> (P<Value>, P<Value>) {
        if f_context.get_value(int128.id()).unwrap().has_split() {
            let vec = f_context
                .get_value(int128.id())
                .unwrap()
                .get_split()
                .as_ref()
                .unwrap();
            (vec[0].clone(), vec[1].clone())
        } else {
            let arg_l = self.make_temporary(f_context, UINT64_TYPE.clone(), vm);
            let arg_h = self.make_temporary(f_context, UINT64_TYPE.clone(), vm);
            f_context
                .get_value_mut(int128.id())
                .unwrap()
                .set_split(vec![arg_l.clone(), arg_h.clone()]);

            (arg_l, arg_h)
        }
    }

    /// apply mask on an integer register
    fn emit_apply_mask(
        &mut self,
        reg: &P<Value>,
        length: BitSize,
        f_context: &mut FunctionContext,
        vm: &VM
    ) {
        if length <= 32 {
            let mask = if length == 32 {
                use std::u32;
                u32::MAX as i32
            } else {
                ((1u32 << length) - 1) as i32
            };
            self.backend.emit_and_r_imm(reg, mask);
        } else if length <= 64 {
            // the mask cannot be an immediate, we need to put it to a temp
            let mask = if length == 64 {
                use std::u64;
                u64::MAX as i64
            } else {
                ((1u64 << length) - 1) as i64
            };
            let tmp_mask = self.make_temporary(f_context, UINT64_TYPE.clone(), vm);
            self.backend.emit_mov_r64_imm64(&tmp_mask, mask);

            // apply mask
            self.backend.emit_and_r_r(reg, &tmp_mask);
        } else {
            panic!(
                "expect masking an integer register with length <= 64, found {}",
                length
            );
        }
    }

    /// finishes current block
    fn finish_block(&mut self) {
        let cur_block = self.current_block.as_ref().unwrap().clone();
        self.backend.end_block(cur_block.clone());

        self.current_block = None;
    }

    /// starts a new block
    fn start_block(&mut self, block: MuName) {
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

        // set up some context
        self.current_fv_id = func_ver.id();
        self.current_fv_name = func_ver.name();
        self.current_sig = Some(func_ver.sig.clone());
        self.current_frame = Some(Frame::new(func_ver.id()));
        self.current_func_start = Some({
            let funcs = vm.funcs().read().unwrap();
            let func = funcs.get(&func_ver.func_id).unwrap().read().unwrap();
            let start_loc = self.backend.start_code(func.name(), entry_block.name());
            if vm.vm_options.flag_emit_debug_info {
                self.backend.add_cfi_startproc();
            }

            start_loc
        });
        self.current_callsite_id = 0;
        self.current_callsites.clear();
        self.current_exn_blocks.clear();
        self.current_constants.clear();
        self.current_constants_locs.clear();

        // prologue (get arguments from entry block first)
        let ref args = entry_block.content.as_ref().unwrap().args;
        self.emit_common_prologue(&func_ver.sig, args, &mut func_ver.context, vm);
    }

    fn visit_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        let f_content = func.content.as_ref().unwrap();

        // compile block by block
        for block_id in func.block_trace.as_ref().unwrap() {
            // is this block an exception block?
            let is_exception_block = f_content.exception_blocks.contains(&block_id);

            let block = f_content.get_block(*block_id);
            let block_label = block.name();
            self.current_block = Some(block_label.clone());
            self.current_block_in_ir = Some(block_label.clone());
            let block_content = block.content.as_ref().unwrap();

            // we need to be aware of exception blocks
            // so that we can emit information to catch exceptions
            if is_exception_block {
                // exception block
                let loc = self.backend.start_exception_block(block_label.clone());
                self.current_exn_blocks
                    .insert(block.id(), loc.to_relocatable());
            } else {
                // normal block
                self.backend.start_block(block_label.clone());
            }

            if block.is_receiving_exception_arg() {
                // this block uses exception arguments
                // we need to emit landingpad for it
                let exception_arg = block_content.exn_arg.as_ref().unwrap();

                // need to insert a landing pad
                self.emit_landingpad(&exception_arg, f_content, &mut func.context, vm);
            }

            // doing the actual instruction selection
            for inst in block_content.body.iter() {
                self.instruction_select(&inst, f_content, &mut func.context, vm);
            }

            // end block
            // we may start block a, and end with block b (instruction selection may create blocks)
            {
                let current_block = self.current_block.as_ref().unwrap();
                self.backend.end_block(current_block.clone());
            }
            self.current_block = None;
            self.current_block_in_ir = None;
        }
    }

    fn finish_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        self.backend.print_cur_code();

        let func_name = {
            let funcs = vm.funcs().read().unwrap();
            let func = funcs.get(&func.func_id).unwrap().read().unwrap();
            func.name()
        };

        // have to do this before 'finish_code()'
        if vm.vm_options.flag_emit_debug_info {
            self.backend.add_cfi_endproc();
        }
        let (mc, func_end) = self.backend.finish_code(func_name.clone());

        // insert exception branch info
        let frame = match self.current_frame.take() {
            Some(frame) => frame,
            None => {
                panic!(
                    "no current_frame for function {} that is being compiled",
                    func_name
                )
            }
        };
        for &(ref callsite, block_id, stack_arg_size) in self.current_callsites.iter() {
            let block_loc = if block_id == 0 {
                None
            } else {
                Some(self.current_exn_blocks.get(&block_id).unwrap().clone())
            };

            vm.add_exception_callsite(
                Callsite::new(callsite.clone(), block_loc, stack_arg_size),
                self.current_fv_id
            );
        }

        let compiled_func = CompiledFunction::new(
            func.func_id,
            func.id(),
            mc,
            self.current_constants.clone(),
            self.current_constants_locs.clone(),
            frame,
            self.current_func_start.take().unwrap(),
            func_end
        );

        vm.add_compiled_func(compiled_func);
    }
}
