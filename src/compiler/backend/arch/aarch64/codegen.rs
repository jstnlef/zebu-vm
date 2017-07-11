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

use ast::ptr::P;
use ast::ir::*;
use runtime::ValueLocation;

use compiler::machine_code::MachineCode;
use compiler::backend::{Reg, Mem};

pub trait CodeGenerator {
    fn start_code(&mut self, func_name: MuName, entry: MuName) -> ValueLocation;
    fn finish_code(&mut self, func_name: MuName) -> (Box<MachineCode + Sync + Send>, ValueLocation);

    // generate unnamed sequence of linear code (no branch)
    fn start_code_sequence(&mut self);
    fn finish_code_sequence(&mut self) -> Box<MachineCode + Sync + Send>;

    fn print_cur_code(&self);

    fn start_block(&mut self, block_name: MuName);
    fn block_exists(&self, block_name: MuName) -> bool;
    fn start_exception_block(&mut self, block_name: MuName) -> ValueLocation;
    fn end_block(&mut self, block_name: MuName);

    // add CFI info
    fn add_cfi_sections(&mut self, arg: &str);
    fn add_cfi_startproc(&mut self);
    fn add_cfi_endproc(&mut self);
    fn add_cfi_def_cfa(&mut self, reg: Reg, offset: i32);
    fn add_cfi_def_cfa_register(&mut self, reg: Reg);
    fn add_cfi_def_cfa_offset(&mut self, offset: i32);
    fn add_cfi_offset(&mut self, reg: Reg, offset: i32);

    //==================================================================================================

    // emit code to adjust frame
    fn emit_frame_grow(&mut self); // Emits a SUB

    // Used to pass a string that the assembler will interpret as an immediate argument
    // (This is neccesary to support the use of ELF relocations like ':tprel_hi12:foo')
    fn emit_add_str(&mut self, dest: Reg, src1: Reg, src2: &str);

    // stack minimpulation
    fn emit_push_pair(&mut self, src1: Reg, src2: Reg, stack: Reg); // Emits a STP
    fn emit_pop_pair(&mut self, dest1: Reg, dest2: Reg, stack: Reg); // Emits a LDP

    // For callee saved loads and stores (flags them so that only they are removed)
    fn emit_ldr_callee_saved(&mut self, dest: Reg, src: Mem);
    fn emit_str_callee_saved(&mut self, dest: Mem, src: Reg);

    /* Bellow ar all ARMv8-A Aarch64 instruction menmonics (with all operand modes) except:
        PRFM, PRFUM, CRC32*
        All advanced SIMD instructions (except MOVI)

    NOTE:
        with loads and stores the menmonic indicated may be given a suffix indicating the size and signenedness of the access
        also b_cond's menmononic is 'B.cond' (where cond is the value of the 'cond' parameter)
        all other instructions have the menmonic being the first word of the function name after emit_
            (subsequent words are used to disambiguate different overloads)
    NOTE unless otherwise indicated:
        An instruction that dosn't start with an F operates on GPRS, those that start with an F operate on FPRs.
        All instructions operate on 32-bit and 64-bit registers (but all register arguments must be the same size)
        Also all arguments that may take the SP can't take the ZR (and vice versa)
    */


    // loads
    fn emit_ldr(&mut self, dest: Reg/*GPR or FPR*/, src: Mem, signed: bool); // supports the full full range of addressing modes
    fn emit_ldtr(&mut self, dest: Reg, src: Mem, signed: bool); // [base, #simm9]
    fn emit_ldur(&mut self, dest: Reg/*GPR or FPR*/, src: Mem, signed: bool); // [base, #simm9]
    fn emit_ldxr(&mut self, dest: Reg, src: Mem);// [base]
    fn emit_ldaxr(&mut self, dest: Reg, src: Mem);// [base]
    fn emit_ldar(&mut self, dest: Reg, src: Mem);// [base]

    fn emit_ldp(&mut self, dest1: Reg, dest2: Reg/*GPR or FPR*/, src: Mem); // [base, #simm7], [base], #simm7, [base, #simm7]!
    fn emit_ldxp(&mut self, dest1: Reg, dest2: Reg, src: Mem); // [base]
    fn emit_ldaxp(&mut self, dest1: Reg, dest2: Reg, src: Mem); // [base]
    fn emit_ldnp(&mut self, dest1: Reg/*GPR or FPR*/, dest2: Reg/*GPR or FPR*/, src: Mem); // [base, #simm7]


    // Stores
    fn emit_str(&mut self, dest: Mem, src: Reg/*GPR or FPR*/); // supports the full full range of addressing modes
    fn emit_sttr(&mut self, dest: Mem, src: Reg); // [base, #simm9]
    fn emit_stur(&mut self, dest: Mem, src: Reg/*GPR or FPR*/); // [base, #simm9]
    fn emit_stlr(&mut self, dest: Mem, src: Reg); // [base]
    fn emit_stxr(&mut self, dest: Mem, status: Reg, src: Reg); // [base]
    fn emit_stlxr(&mut self, dest: Mem, status: Reg, src: Reg); // [base]

    fn emit_stp(&mut self, dest: Mem, src1: Reg, src2: Reg);  // [base, #simm7], [base], #simm7, [base, #simm7]!
    fn emit_stxp(&mut self, dest: Mem, status: Reg, src1: Reg, src2: Reg); // [base]
    fn emit_stlxp(&mut self, dest: Mem, status: Reg, src1: Reg, src2: Reg); // [base]
    fn emit_stnp(&mut self, dest: Mem, src1: Reg/*GPR or FPR*/, src2: Reg/*GPR or FPR*/); // [base, #simm7]

    // branching

    // calls
    fn emit_bl(&mut self, callsite: String, func: MuName, pe: Option<MuName>, is_native: bool) -> ValueLocation;
    fn emit_blr(&mut self, callsite: String, func: Reg, pe: Option<MuName>) -> ValueLocation;

    // Branches
    fn emit_b(&mut self, dest_name: MuName);
    fn emit_b_cond(&mut self, cond: &str, dest_name: MuName);
    fn emit_br(&mut self, dest_address: Reg);
    fn emit_ret(&mut self, src: Reg);
    fn emit_cbnz(&mut self, src: Reg, dest_name: MuName);
    fn emit_cbz(&mut self, src: Reg, dest_name: MuName);
    fn emit_tbnz(&mut self, src1: Reg, src2: u8, dest_name: MuName);
    fn emit_tbz(&mut self, src1: Reg, src2: u8, dest_name: MuName);

    // Read and write flags
    fn emit_msr(&mut self, dest: &str, src: Reg);
    fn emit_mrs(&mut self, dest: Reg, src: &str);

    // Address calculation
    fn emit_adr(&mut self, dest: Reg, src: Reg);
    fn emit_adrp(&mut self, dest: Reg, src: Reg);

    // Unary ops
    fn emit_mov(&mut self, dest: Reg, src: Reg);
    fn emit_mvn(&mut self, dest: Reg, src: Reg);
    fn emit_neg(&mut self, dest: Reg, src: Reg);
    fn emit_negs(&mut self, dest: Reg, src: Reg);
    fn emit_ngc(&mut self, dest: Reg, src: Reg);
    fn emit_ngcs(&mut self, dest: Reg, src: Reg);
    fn emit_sxtb(&mut self, dest: Reg/*32*/, src: Reg/*32*/);
    fn emit_sxth(&mut self, dest: Reg/*32*/, src: Reg/*32*/);
    fn emit_sxtw(&mut self, dest: Reg/*64*/, src: Reg/*32*/);
    fn emit_uxtb(&mut self, dest: Reg/*32*/, src: Reg/*32*/);
    fn emit_uxth(&mut self, dest: Reg/*32*/, src: Reg/*32*/);
    fn emit_cls(&mut self, dest: Reg, src: Reg);
    fn emit_clz(&mut self, dest: Reg, src: Reg);
    fn emit_rbit(&mut self, dest: Reg, src: Reg);
    fn emit_rev(&mut self, dest: Reg, src: Reg);
    fn emit_rev16(&mut self, dest: Reg, src: Reg);
    fn emit_rev32(&mut self, dest: Reg/*64*/, src: Reg);
    fn emit_rev64(&mut self, dest: Reg/*64*/, src: Reg); // alias of REV
    fn emit_fabs(&mut self, dest: Reg, src: Reg/*Must have different size*/);
    fn emit_fcvt(&mut self, dest: Reg, src: Reg/*Must have different size*/);
    fn emit_fcvtas(&mut self, dest: Reg/*GPR, may have different size*/, src: Reg);
    fn emit_fcvtau(&mut self, dest: Reg/*GPR, may have different size*/, src: Reg);
    fn emit_fcvtms(&mut self, dest: Reg/*GPR, may have different size*/, src: Reg);
    fn emit_fcvtmu(&mut self, dest: Reg/*GPR, may have different size*/, src: Reg);
    fn emit_fcvtns(&mut self, dest: Reg/*GPR, may have different size*/, src: Reg);
    fn emit_fcvtnu(&mut self, dest: Reg/*GPR, may have different size*/, src: Reg);
    fn emit_fcvtps(&mut self, dest: Reg/*GPR, may have different size*/, src: Reg);
    fn emit_fcvtpu(&mut self, dest: Reg/*GPR, may have different size*/, src: Reg);
    fn emit_fcvtzs(&mut self, dest: Reg/*GPR, may have different size*/, src: Reg);
    fn emit_fcvtzu(&mut self, dest: Reg/*GPR, may have different size*/, src: Reg);
    fn emit_fmov(&mut self, dest: Reg, src: Reg); // One register must be an FPR, the other may be a GPR or an FPR
    fn emit_fneg(&mut self, dest: Reg, src: Reg);
    fn emit_frinta(&mut self, dest: Reg, src: Reg);
    fn emit_frinti(&mut self, dest: Reg, src: Reg);
    fn emit_frintm(&mut self, dest: Reg, src: Reg);
    fn emit_frintn(&mut self, dest: Reg, src: Reg);
    fn emit_frintp(&mut self, dest: Reg, src: Reg);
    fn emit_frintx(&mut self, dest: Reg, src: Reg);
    fn emit_frintz(&mut self, dest: Reg, src: Reg);
    fn emit_fsqrt(&mut self, dest: Reg, src: Reg);
    fn emit_scvtf(&mut self, dest: Reg/*FPR, may have different size*/, src: Reg);
    fn emit_ucvtf(&mut self, dest: Reg/*FPR, may have different size*/, src: Reg);

    // Unary operations with shift
    fn emit_mov_shift(&mut self, dest: Reg, src: Reg, shift: &str, ammount: u8);
    fn emit_mvn_shift(&mut self, dest: Reg, src: Reg, shift: &str, ammount: u8);
    fn emit_neg_shift(&mut self, dest: Reg, src: Reg, shift: &str, ammount: u8);
    fn emit_negs_shift(&mut self, dest: Reg, src: Reg, shift: &str, ammount: u8);

    // Unary operations with immediates
    fn emit_mov_imm(&mut self, dest: Reg, src: u64);
    fn emit_movz(&mut self, dest: Reg, src: u16, shift: u8);
    fn emit_movk(&mut self, dest: Reg, src: u16, shift: u8);
    fn emit_movn(&mut self, dest: Reg, src: u16, shift: u8);
    fn emit_movi(&mut self, dest: Reg /*FPR*/, src: u64);
    fn emit_fmov_imm(&mut self, dest: Reg, src: f32);

    // Extended binary ops
    fn emit_add_ext(&mut self, dest: Reg/*GPR or SP*/, src1: Reg/*GPR or SP*/, src2: Reg, signed: bool, shift: u8);
    fn emit_adds_ext(&mut self, dest: Reg, src1: Reg/*GPR or SP*/, src2: Reg, signed: bool, shift: u8);
    fn emit_sub_ext(&mut self, dest: Reg/*GPR or SP*/, src1: Reg/*GPR or SP*/, src2: Reg, signed: bool, shift: u8);
    fn emit_subs_ext(&mut self, dest: Reg, src1: Reg/*GPR or SP*/, src2: Reg, signed: bool, shift: u8);

    // Multiplication
    fn emit_mul(&mut self, dest: Reg, src1: Reg, src2: Reg);
    fn emit_mneg(&mut self, dest: Reg, src1: Reg, src2: Reg);
    fn emit_smulh(&mut self, dest: Reg/*64*/, src1: Reg/*64*/, src2: Reg/*64*/);
    fn emit_umulh(&mut self, dest: Reg/*64*/, src1: Reg/*64*/, src2: Reg/*64*/);
    fn emit_smnegl(&mut self, dest: Reg/*64*/, src1: Reg/*32*/, src2: Reg/*32*/);
    fn emit_smull(&mut self, dest: Reg/*64*/, src1: Reg/*32*/, src2: Reg/*32*/);
    fn emit_umnegl(&mut self, dest: Reg/*64*/, src1: Reg/*32*/, src2: Reg/*32*/);
    fn emit_umull(&mut self, dest: Reg/*64*/, src1: Reg/*32*/, src2: Reg/*32*/);

    // Other binaries
    fn emit_adc(&mut self, dest: Reg, src1: Reg, src2: Reg);
    fn emit_adcs(&mut self, dest: Reg, src1: Reg, src2: Reg);
    fn emit_add(&mut self, dest: Reg, src1: Reg/*GPR or SP*/, src2: Reg);
    fn emit_adds(&mut self, dest: Reg, src1: Reg/*GPR or SP*/, src2: Reg);
    fn emit_sbc(&mut self, dest: Reg, src1: Reg, src2: Reg);
    fn emit_sbcs(&mut self, dest: Reg, src1: Reg, src2: Reg);
    fn emit_sub(&mut self, dest: Reg, src1: Reg, src2: Reg);
    fn emit_subs(&mut self, dest: Reg, src1: Reg, src2: Reg);
    fn emit_sdiv(&mut self, dest: Reg, src1: Reg, src2: Reg);
    fn emit_udiv(&mut self, dest: Reg, src1: Reg, src2: Reg);
    fn emit_asr(&mut self, dest: Reg, src1: Reg, src2: Reg);
    fn emit_asrv(&mut self, dest: Reg, src1: Reg, src2: Reg); // Alias of ASR
    fn emit_lsl(&mut self, dest: Reg, src1: Reg, src2: Reg);
    fn emit_lslv(&mut self, dest: Reg, src1: Reg, src2: Reg); // Alias of LSL
    fn emit_lsr(&mut self, dest: Reg, src1: Reg, src2: Reg);
    fn emit_lsrv(&mut self, dest: Reg, src1: Reg, src2: Reg); // Alias of LSR
    fn emit_ror(&mut self, dest: Reg, src1: Reg, src2: Reg);
    fn emit_bic(&mut self, dest: Reg, src1: Reg, src2: Reg);
    fn emit_bics(&mut self, dest: Reg, src1: Reg, src2: Reg);
    fn emit_and(&mut self, dest: Reg, src1: Reg, src2: Reg);
    fn emit_ands(&mut self, dest: Reg, src1: Reg, src2: Reg);
    fn emit_eon(&mut self, dest: Reg, src1: Reg, src2: Reg);
    fn emit_eor(&mut self, dest: Reg, src1: Reg, src2: Reg);
    fn emit_orn(&mut self, dest: Reg, src1: Reg, src2: Reg);
    fn emit_orr(&mut self, dest: Reg, src1: Reg, src2: Reg);
    fn emit_fadd(&mut self, dest: Reg, src1: Reg, src2: Reg);
    fn emit_fdiv(&mut self, dest: Reg, src1: Reg, src2: Reg);
    fn emit_fmax(&mut self, dest: Reg, src1: Reg, src2: Reg);
    fn emit_fmaxnm(&mut self, dest: Reg, src1: Reg, src2: Reg);
    fn emit_fmin(&mut self, dest: Reg, src1: Reg, src2: Reg);
    fn emit_fminm(&mut self, dest: Reg, src1: Reg, src2: Reg);
    fn emit_fmul(&mut self, dest: Reg, src1: Reg, src2: Reg);
    fn emit_fnmul(&mut self, dest: Reg, src1: Reg, src2: Reg);
    fn emit_fsub(&mut self, dest: Reg, src1: Reg, src2: Reg);

    // Binary operations with shift
    fn emit_add_shift(&mut self, dest: Reg, src1: Reg, src2: Reg, shift: &str, amount: u8);
    fn emit_adds_shift(&mut self, dest: Reg, src1: Reg, src2: Reg, shift: &str, amount: u8);
    fn emit_sub_shift(&mut self, dest: Reg, src1: Reg, src2: Reg, shift: &str, amount: u8);
    fn emit_subs_shift(&mut self, dest: Reg, src1: Reg, src2: Reg, shift: &str, amount: u8);
    fn emit_bic_shift(&mut self, dest: Reg, src1: Reg, src2: Reg, shift: &str, amount: u8);
    fn emit_bics_shift(&mut self, dest: Reg, src1: Reg, src2: Reg, shift: &str, amount: u8);
    fn emit_and_shift(&mut self, dest: Reg, src1: Reg, src2: Reg, shift: &str, amount: u8);
    fn emit_ands_shift(&mut self, dest: Reg, src1: Reg, src2: Reg, shift: &str, amount: u8);
    fn emit_eon_shift(&mut self, dest: Reg, src1: Reg, src2: Reg, shift: &str, amount: u8);
    fn emit_eor_shift(&mut self, dest: Reg, src1: Reg, src2: Reg, shift: &str, amount: u8);
    fn emit_orn_shift(&mut self, dest: Reg, src1: Reg, src2: Reg, shift: &str, amount: u8);
    fn emit_orr_shift(&mut self, dest: Reg, src1: Reg, src2: Reg, shift: &str, amount: u8);

    // binary ops with immediates
    fn emit_add_imm(&mut self, dest: Reg/*GPR or SP*/, src1: Reg/*GPR or SP*/, src2: u16, shift: bool);
    fn emit_adds_imm(&mut self, dest: Reg, src1: Reg/*GPR or SP*/, src2: u16, shift: bool);
    fn emit_sub_imm(&mut self, dest: Reg/*GPR or SP*/, src1: Reg/*GPR or SP*/, src2: u16, shift: bool);
    fn emit_subs_imm(&mut self, dest: Reg, src1: Reg/*GPR or SP*/, src2: u16, shift: bool);

    fn emit_and_imm(&mut self, dest: Reg/*GPR or SP*/, src1: Reg, src2: u64);
    fn emit_ands_imm(&mut self, dest: Reg, src1: Reg, src2: u64);
    fn emit_eor_imm(&mut self, dest: Reg/*GPR or SP*/, src1: Reg, src2: u64);
    fn emit_orr_imm(&mut self, dest: Reg/*GPR or SP*/, src1: Reg, src2: u64);

    fn emit_asr_imm(&mut self, dest: Reg, src1: Reg, src2: u8);
    fn emit_lsr_imm(&mut self, dest: Reg, src1: Reg, src2: u8);
    fn emit_lsl_imm(&mut self, dest: Reg, src1: Reg, src2: u8);
    fn emit_ror_imm(&mut self, dest: Reg, src1: Reg, src2: u8);

    // ternary ops

    fn emit_madd(&mut self, dest: Reg, src1: Reg, src2: Reg, src3: Reg);
    fn emit_msub(&mut self, dest: Reg, src1: Reg, src2: Reg, src3: Reg);
    fn emit_smaddl(&mut self, dest: Reg/*64*/, src1: Reg/*32*/, src2: Reg/*32*/, src3: Reg/*64*/);
    fn emit_smsubl(&mut self, dest: Reg/*64*/, src1: Reg/*32*/, src2: Reg/*32*/, src3: Reg/*64*/);
    fn emit_umaddl(&mut self, dest: Reg/*64*/, src1: Reg/*32*/, src2: Reg/*32*/, src3: Reg/*64*/);
    fn emit_umsubl(&mut self, dest: Reg/*64*/, src1: Reg/*32*/, src2: Reg/*32*/, src3: Reg/*64*/);
    fn emit_fmadd(&mut self, dest: Reg, src1: Reg, src2: Reg, src3: Reg);
    fn emit_fmsub(&mut self, dest: Reg, src1: Reg, src2: Reg, src3: Reg);
    fn emit_fnmadd(&mut self, dest: Reg, src1: Reg, src2: Reg, src3: Reg);
    fn emit_fnmsub(&mut self, dest: Reg, src1: Reg, src2: Reg, src3: Reg);

    // Ternary ops with immediates
    fn emit_bfm(&mut self, dest: Reg, src1: Reg, src2: u8, src3: u8);
    fn emit_bfi(&mut self, dest: Reg, src1: Reg, src2: u8, src3: u8);
    fn emit_bfxil(&mut self, dest: Reg, src1: Reg, src2: u8, src3: u8);
    fn emit_ubfm(&mut self, dest: Reg, src1: Reg, src2: u8, src3: u8);
    fn emit_ubfx(&mut self, dest: Reg, src1: Reg, src2: u8, src3: u8);
    fn emit_ubfiz(&mut self, dest: Reg, src1: Reg, src2: u8, src3: u8);
    fn emit_sbfm(&mut self, dest: Reg, src1: Reg, src2: u8, src3: u8);
    fn emit_sbfx(&mut self, dest: Reg, src1: Reg, src2: u8, src3: u8);
    fn emit_sbfiz(&mut self, dest: Reg, src1: Reg, src2: u8, src3: u8);

    // Comparison (dosn't store a result, only updates flags)
    fn emit_tst(&mut self, src1: Reg, src2: Reg);
    fn emit_cmn(&mut self, src1: Reg, src2: Reg);
    fn emit_cmp(&mut self, src1: Reg, src2: Reg);
    fn emit_fcmp(&mut self, src1: Reg, src2: Reg);
    fn emit_fcmpe(&mut self, src1: Reg, src2: Reg);

    // Comparisons with extension
    fn emit_cmn_ext(&mut self, src1: Reg/*GPR or SP*/, src2: Reg, signed: bool, shift: u8);
    fn emit_cmp_ext(&mut self, src1: Reg/*GPR or SP*/, src2: Reg, signed: bool, shift: u8);

    // Comparisons with shift
    fn emit_tst_shift(&mut self, src1: Reg, src2: Reg, shift: &str, ammount: u8);
    fn emit_cmn_shift(&mut self, src1: Reg, src2: Reg, shift: &str, ammount: u8);
    fn emit_cmp_shift(&mut self, src1: Reg, src2: Reg, shift: &str, ammount: u8);

    // Immediat Comparisons
    fn emit_tst_imm(&mut self, src1: Reg, src2: u64);
    fn emit_cmn_imm(&mut self, src1: Reg/*GPR or SP*/, src2: u16, shift : bool);
    fn emit_cmp_imm(&mut self, src1: Reg/*GPR or SP*/, src2: u16, shift : bool);

    // Comparison against 0
    fn emit_fcmp_0(&mut self, src: Reg);
    fn emit_fcmpe_0(&mut self, src: Reg);

    // Conditional ops
    fn emit_cset(&mut self, dest: Reg, cond: &str);
    fn emit_csetm(&mut self, dest: Reg, cond: &str);

    // Conditional unary ops
    fn emit_cinc(&mut self, dest: Reg, src: Reg, cond: &str);
    fn emit_cneg(&mut self, dest: Reg, src: Reg, cond: &str);
    fn emit_cinv(&mut self, dest: Reg, src: Reg, cond: &str);

    // Conditional binary ops
    fn emit_csel(&mut self, dest: Reg, src1: Reg, src2: Reg, cond: &str);
    fn emit_csinc(&mut self, dest: Reg, src1: Reg, src2: Reg, cond: &str);
    fn emit_csinv(&mut self, dest: Reg, src1: Reg, src2: Reg, cond: &str);
    fn emit_csneg(&mut self, dest: Reg, src1: Reg, src2: Reg, cond: &str);
    fn emit_fcsel(&mut self, dest: Reg, src1: Reg, src2: Reg, cond: &str);

    // Conditional comparisons
    fn emit_ccmn(&mut self, src1: Reg, src2: Reg, flags: u8, cond: &str);
    fn emit_ccmp(&mut self, src1: Reg, src2: Reg, flags: u8, cond: &str);
    fn emit_fccmp(&mut self, src1: Reg, src2: Reg, flags: u8, cond: &str);
    fn emit_fccmpe(&mut self, src1: Reg, src2: Reg, flags: u8, cond: &str);

    // Conditional comparisons (with immediate)
    fn emit_ccmn_imm(&mut self, src1: Reg, src2: u8, flags: u8, cond: &str);
    fn emit_ccmp_imm(&mut self, src1: Reg, src2: u8, flags: u8, cond: &str);

    fn emit_bfc(&mut self, dest: Reg, src1: u8, src2: u8);
    fn emit_extr(&mut self, dest: Reg, src1: Reg, src2: Reg, src3: u8);

    // Synchronisation
    fn emit_dsb(&mut self, option: &str);
    fn emit_dmb(&mut self, option: &str);
    fn emit_isb(&mut self, option: &str);
    fn emit_clrex(&mut self);

    // Hint instructions
    fn emit_sevl(&mut self);
    fn emit_sev(&mut self);
    fn emit_wfe(&mut self);
    fn emit_wfi(&mut self);
    fn emit_yield(&mut self);
    fn emit_nop(&mut self);
    fn emit_hint(&mut self, val: u8);

    // Debug instructions
    fn emit_drps(&mut self);
    fn emit_dcps1(&mut self, val: u16);
    fn emit_dcps2(&mut self, val: u16);
    fn emit_dcps3(&mut self, val: u16);

    // System instruction
    fn emit_dc(&mut self, option: &str, src: Reg);
    fn emit_at(&mut self, option: &str, src: Reg);
    fn emit_ic(&mut self, option: &str, src: Reg);
    fn emit_tlbi(&mut self, option: &str, src: Reg);

    fn emit_sys(&mut self, imm1: u8, cn: u8, cm: u8, imm2: u8, src: Reg);
    fn emit_sysl(&mut self, dest: Reg, imm1: u8, cn: u8, cm: u8, imm2: u8);

    // Exceptiuon instructions (NOTE: these will alter the PC)
    fn emit_brk(&mut self, val: u16);
    fn emit_hlt(&mut self, val: u16);
    fn emit_hvc(&mut self, val: u16);
    fn emit_smc(&mut self, val: u16);
    fn emit_svc(&mut self, val: u16);
    fn emit_eret(&mut self);
}
