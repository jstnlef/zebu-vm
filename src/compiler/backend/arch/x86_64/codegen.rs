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
use ast::ptr::P;
use runtime::ValueLocation;

use compiler::backend::{Mem, Reg};
use compiler::machine_code::MachineCode;

/// CodeGenerator provides an interface to emit x86_64 code for instruction selection.
/// This allows us to implement the other parts of the compiler (mostly instruction selection)
/// without assuming code generator. Currently there is only an assembly backend
/// that implements this interface for ahead-of-time compilation. We plan to add
/// a binary backend for just-in-time compilation.
pub trait CodeGenerator {
    /// starts code for a function
    fn start_code(&mut self, func_name: MuName, entry: MuName) -> ValueLocation;
    /// finishes code for a function
    fn finish_code(&mut self, func_name: MuName) -> (Box<MachineCode + Sync + Send>, ValueLocation);

    /// starts a sequence of linear code (no branch)
    fn start_code_sequence(&mut self);
    /// finishes code for a sequence
    fn finish_code_sequence(&mut self) -> Box<MachineCode + Sync + Send>;

    /// outputs current code (via debug! log)
    fn print_cur_code(&self);

    /// starts a block
    fn start_block(&mut self, block_name: MuName);
    /// starts an exceptional block, and returns its code address
    fn start_exception_block(&mut self, block_name: MuName) -> ValueLocation;
    /// finishes a block (must have called start_block() or start_excpetion_block() first)
    fn end_block(&mut self, block_name: MuName);

    // adds CFI info
    fn add_cfi_startproc(&mut self);
    fn add_cfi_endproc(&mut self);
    fn add_cfi_def_cfa_register(&mut self, reg: Reg);
    fn add_cfi_def_cfa_offset(&mut self, offset: i32);
    fn add_cfi_offset(&mut self, reg: Reg, offset: i32);

    // emit code to adjust frame size
    fn emit_frame_grow(&mut self);

    fn emit_nop(&mut self, bytes: usize);

    // comparison
    fn emit_cmp_r_r(&mut self, op1: Reg, op2: Reg);
    fn emit_cmp_imm_r(&mut self, op1: i32, op2: Reg);
    fn emit_cmp_mem_r(&mut self, op1: Mem, op2: Reg);
    fn emit_cmp_r_mem(&mut self, op1: Reg, op2: Mem);

    fn emit_test_r_r(&mut self, op1: Reg, op2: Reg);
    fn emit_test_imm_r(&mut self, op1: i32, op2: Reg);

    // gpr move

    // mov imm64 to r64
    fn emit_mov_r64_imm64(&mut self, dest: Reg, src: i64);
    // bitcast between int and floatpoint of same length
    fn emit_mov_fpr_r64(&mut self, dest: Reg, src: Reg);
    fn emit_mov_fpr_r32(&mut self, dest: Reg, src: Reg);
    fn emit_mov_r64_fpr(&mut self, dest: Reg, src: Reg);
    fn emit_mov_r32_fpr(&mut self, dest: Reg, src: Reg);

    fn emit_mov_r_imm(&mut self, dest: Reg, src: i32);
    fn emit_mov_r_mem(&mut self, dest: Reg, src: Mem); // load
    fn emit_mov_r_r(&mut self, dest: Reg, src: Reg);
    fn emit_mov_mem_r(&mut self, dest: Mem, src: Reg); // store
                                                       // we can infer imm length from Reg, but cannot from Mem
                                                       // because mem may only have type as ADDRESS_TYPE
    fn emit_mov_mem_imm(&mut self, dest: Mem, src: i32, oplen: usize); // store

    fn emit_mov_mem_r_callee_saved(&mut self, dest: Mem, src: Reg); // store callee saved register
    fn emit_mov_r_mem_callee_saved(&mut self, dest: Reg, src: Mem); // load callee saved register

    // zero/sign extend mov
    fn emit_movs_r_r(&mut self, dest: Reg, src: Reg);
    fn emit_movz_r_r(&mut self, dest: Reg, src: Reg);

    // set byte
    fn emit_sets_r8(&mut self, dest: Reg);
    fn emit_setz_r8(&mut self, dest: Reg);
    fn emit_seto_r8(&mut self, dest: Reg);
    fn emit_setb_r8(&mut self, dest: Reg);

    fn emit_seta_r(&mut self, dest: Reg);
    fn emit_setae_r(&mut self, dest: Reg);
    fn emit_setb_r(&mut self, dest: Reg);
    fn emit_setbe_r(&mut self, dest: Reg);
    fn emit_sete_r(&mut self, dest: Reg);
    fn emit_setg_r(&mut self, dest: Reg);
    fn emit_setge_r(&mut self, dest: Reg);
    fn emit_setl_r(&mut self, dest: Reg);
    fn emit_setle_r(&mut self, dest: Reg);
    fn emit_setne_r(&mut self, dest: Reg);

    // gpr conditional move

    fn emit_cmova_r_r(&mut self, dest: Reg, src: Reg);
    fn emit_cmova_r_mem(&mut self, dest: Reg, src: Mem); // load

    fn emit_cmovae_r_r(&mut self, dest: Reg, src: Reg);
    fn emit_cmovae_r_mem(&mut self, dest: Reg, src: Mem); // load

    fn emit_cmovb_r_r(&mut self, dest: Reg, src: Reg);
    fn emit_cmovb_r_mem(&mut self, dest: Reg, src: Mem); // load

    fn emit_cmovbe_r_r(&mut self, dest: Reg, src: Reg);
    fn emit_cmovbe_r_mem(&mut self, dest: Reg, src: Mem); // load

    fn emit_cmove_r_r(&mut self, dest: Reg, src: Reg);
    fn emit_cmove_r_mem(&mut self, dest: Reg, src: Mem); // load

    fn emit_cmovg_r_r(&mut self, dest: Reg, src: Reg);
    fn emit_cmovg_r_mem(&mut self, dest: Reg, src: Mem); // load

    fn emit_cmovge_r_r(&mut self, dest: Reg, src: Reg);
    fn emit_cmovge_r_mem(&mut self, dest: Reg, src: Mem); // load

    fn emit_cmovl_r_r(&mut self, dest: Reg, src: Reg);
    fn emit_cmovl_r_mem(&mut self, dest: Reg, src: Mem); // load

    fn emit_cmovle_r_r(&mut self, dest: Reg, src: Reg);
    fn emit_cmovle_r_mem(&mut self, dest: Reg, src: Mem); // load

    fn emit_cmovne_r_r(&mut self, dest: Reg, src: Reg);
    fn emit_cmovne_r_mem(&mut self, dest: Reg, src: Mem); // load

    // lea
    fn emit_lea_r64(&mut self, dest: Reg, src: Mem);

    // and
    fn emit_and_r_imm(&mut self, dest: Reg, src: i32);
    fn emit_and_r_r(&mut self, dest: Reg, src: Reg);
    fn emit_and_r_mem(&mut self, dest: Reg, src: Mem);

    // or
    fn emit_or_r_r(&mut self, dest: Reg, src: Reg);
    fn emit_or_r_imm(&mut self, dest: Reg, src: i32);
    fn emit_or_r_mem(&mut self, dest: Reg, src: Mem);

    // xor
    fn emit_xor_r_r(&mut self, dest: Reg, src: Reg);
    fn emit_xor_r_mem(&mut self, dest: Reg, src: Mem);
    fn emit_xor_r_imm(&mut self, dest: Reg, src: i32);

    // add
    fn emit_add_r_r(&mut self, dest: Reg, src: Reg);
    fn emit_add_r_mem(&mut self, dest: Reg, src: Mem);
    fn emit_add_r_imm(&mut self, dest: Reg, src: i32);

    // add with carry
    fn emit_adc_r_r(&mut self, dest: Reg, src: Reg);
    fn emit_adc_r_mem(&mut self, dest: Reg, src: Mem);
    fn emit_adc_r_imm(&mut self, dest: Reg, src: i32);

    // sub
    fn emit_sub_r_r(&mut self, dest: Reg, src: Reg);
    fn emit_sub_r_mem(&mut self, dest: Reg, src: Mem);
    fn emit_sub_r_imm(&mut self, dest: Reg, src: i32);

    // sub with borrow
    fn emit_sbb_r_r(&mut self, dest: Reg, src: Reg);
    fn emit_sbb_r_mem(&mut self, dest: Reg, src: Mem);
    fn emit_sbb_r_imm(&mut self, dest: Reg, src: i32);

    // inc and dec
    fn emit_inc_r(&mut self, dest: Reg);
    fn emit_inc_mem(&mut self, dest: Mem);
    fn emit_dec_r(&mut self, dest: Reg);
    fn emit_dec_mem(&mut self, dest: Mem);

    // multiply
    fn emit_mul_r(&mut self, src: Reg);
    fn emit_mul_mem(&mut self, src: Mem);

    // signed multiply
    fn emit_imul_r_r(&mut self, dest: Reg, src: Reg);

    // div
    fn emit_div_r(&mut self, src: Reg);
    fn emit_div_mem(&mut self, src: Mem);

    // idiv
    fn emit_idiv_r(&mut self, src: Reg);
    fn emit_idiv_mem(&mut self, src: Mem);

    // shl
    fn emit_shl_r_cl(&mut self, dest: Reg);
    fn emit_shl_r_imm8(&mut self, dest: Reg, src: i8);
    fn emit_shld_r_r_cl(&mut self, dest: Reg, src: Reg);

    fn emit_shr_r_cl(&mut self, dest: &P<Value>);
    fn emit_shr_r_imm8(&mut self, dest: &P<Value>, src: i8);
    fn emit_shrd_r_r_cl(&mut self, dest: Reg, src: Reg);

    fn emit_sar_r_cl(&mut self, dest: &P<Value>);
    fn emit_sar_r_imm8(&mut self, dest: &P<Value>, src: i8);

    fn emit_cqo(&mut self); // sign extend rax to rdx:rax
    fn emit_cdq(&mut self); // sign extend eax to edx:eax
    fn emit_cwd(&mut self); // sign extend ax  to dx:ax

    // jump, conditional jump
    fn emit_jmp(&mut self, dest: MuName);
    fn emit_je(&mut self, dest: MuName);
    fn emit_jne(&mut self, dest: MuName);
    fn emit_ja(&mut self, dest: MuName);
    fn emit_jae(&mut self, dest: MuName);
    fn emit_jb(&mut self, dest: MuName);
    fn emit_jbe(&mut self, dest: MuName);
    fn emit_jg(&mut self, dest: MuName);
    fn emit_jge(&mut self, dest: MuName);
    fn emit_jl(&mut self, dest: MuName);
    fn emit_jle(&mut self, dest: MuName);
    fn emit_js(&mut self, dest: MuName);

    // call
    fn emit_call_near_rel32(
        &mut self,
        callsite: MuName,
        func: MuName,
        pe: Option<MuName>,
        uses: Vec<P<Value>>,
        defs: Vec<P<Value>>,
        is_native: bool
    ) -> ValueLocation;
    fn emit_call_near_r64(
        &mut self,
        callsite: MuName,
        func: &P<Value>,
        pe: Option<MuName>,
        uses: Vec<P<Value>>,
        defs: Vec<P<Value>>
    ) -> ValueLocation;
    fn emit_call_near_mem64(
        &mut self,
        callsite: MuName,
        func: &P<Value>,
        pe: Option<MuName>,
        uses: Vec<P<Value>>,
        defs: Vec<P<Value>>
    ) -> ValueLocation;

    // sometimes we use jmp as a call (but without pushing return address)
    fn emit_call_jmp(
        &mut self,
        callsite: MuName,
        func: MuName,
        pe: Option<MuName>,
        uses: Vec<P<Value>>,
        defs: Vec<P<Value>>,
        is_native: bool
    ) -> ValueLocation;
    fn emit_call_jmp_indirect(
        &mut self,
        callsite: MuName,
        func: &P<Value>,
        pe: Option<MuName>,
        uses: Vec<P<Value>>,
        defs: Vec<P<Value>>
    ) -> ValueLocation;

    fn emit_ret(&mut self);

    // push/pop
    fn emit_push_r64(&mut self, src: &P<Value>);
    fn emit_push_imm32(&mut self, src: i32);
    fn emit_pop_r64(&mut self, dest: &P<Value>);

    // fpr move
    fn emit_movsd_f64_f64(&mut self, dest: &P<Value>, src: &P<Value>);
    fn emit_movsd_f64_mem64(&mut self, dest: &P<Value>, src: &P<Value>); // load
    fn emit_movsd_mem64_f64(&mut self, dest: &P<Value>, src: &P<Value>); // store

    fn emit_movss_f32_f32(&mut self, dest: &P<Value>, src: &P<Value>);
    fn emit_movss_f32_mem32(&mut self, dest: &P<Value>, src: &P<Value>); // load
    fn emit_movss_mem32_f32(&mut self, dest: &P<Value>, src: &P<Value>); // store

    // fp add
    fn emit_addsd_f64_f64(&mut self, dest: Reg, src: Reg);
    fn emit_addsd_f64_mem64(&mut self, dest: Reg, src: Mem);

    fn emit_addss_f32_f32(&mut self, dest: Reg, src: Reg);
    fn emit_addss_f32_mem32(&mut self, dest: Reg, src: Mem);

    // fp sub
    fn emit_subsd_f64_f64(&mut self, dest: Reg, src: Reg);
    fn emit_subsd_f64_mem64(&mut self, dest: Reg, src: Mem);

    fn emit_subss_f32_f32(&mut self, dest: Reg, src: Reg);
    fn emit_subss_f32_mem32(&mut self, dest: Reg, src: Mem);

    // fp div
    fn emit_divsd_f64_f64(&mut self, dest: Reg, src: Reg);
    fn emit_divsd_f64_mem64(&mut self, dest: Reg, src: Mem);

    fn emit_divss_f32_f32(&mut self, dest: Reg, src: Reg);
    fn emit_divss_f32_mem32(&mut self, dest: Reg, src: Mem);

    // fp mul
    fn emit_mulsd_f64_f64(&mut self, dest: Reg, src: Reg);
    fn emit_mulsd_f64_mem64(&mut self, dest: Reg, src: Mem);

    fn emit_mulss_f32_f32(&mut self, dest: Reg, src: Reg);
    fn emit_mulss_f32_mem32(&mut self, dest: Reg, src: Mem);

    // fp comparison
    fn emit_comisd_f64_f64(&mut self, op1: Reg, op2: Reg);
    fn emit_ucomisd_f64_f64(&mut self, op1: Reg, op2: Reg);

    fn emit_comiss_f32_f32(&mut self, op1: Reg, op2: Reg);
    fn emit_ucomiss_f32_f32(&mut self, op1: Reg, op2: Reg);

    // fp bitwise
    fn emit_xorps_f32_f32(&mut self, dest: Reg, src: Reg);
    fn emit_xorpd_f64_f64(&mut self, dest: Reg, src: Reg);

    // fp conversion
    fn emit_cvtsi2sd_f64_r(&mut self, dest: Reg, src: Reg);
    fn emit_cvtsd2si_r_f64(&mut self, dest: Reg, src: Reg);

    fn emit_cvtsi2ss_f32_r(&mut self, dest: Reg, src: Reg);
    fn emit_cvtss2si_r_f32(&mut self, dest: Reg, src: Reg);

    // fp trunc
    fn emit_cvtsd2ss_f32_f64(&mut self, dest: Reg, src: Reg);
    fn emit_cvtss2sd_f64_f32(&mut self, dest: Reg, src: Reg);

    // used for unsigned int to fp conversion
    fn emit_cvttsd2si_r_f64(&mut self, dest: Reg, src: Reg);
    fn emit_cvttss2si_r_f32(&mut self, dest: Reg, src: Reg);

    // unpack low data - interleave low byte
    fn emit_punpckldq_f64_mem128(&mut self, dest: Reg, src: Mem);
    // substract packed double-fp
    fn emit_subpd_f64_mem128(&mut self, dest: Reg, src: Mem);
    // packed double-fp horizontal add
    fn emit_haddpd_f64_f64(&mut self, dest: Reg, src: Reg);

    // move aligned packed double-precision fp values
    fn emit_movapd_f64_mem128(&mut self, dest: Reg, src: Mem);
    fn emit_movapd_f64_f64(&mut self, dest: Reg, src: Mem);

    fn emit_movaps_f32_f32(&mut self, dest: Reg, src: Reg);

    // memory fence
    fn emit_mfence(&mut self);
}
