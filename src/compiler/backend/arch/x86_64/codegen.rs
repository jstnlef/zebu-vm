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
    fn start_exception_block(&mut self, block_name: MuName) -> ValueLocation;
    fn set_block_livein(&mut self, block_name: MuName, live_in: &Vec<P<Value>>);
    fn set_block_liveout(&mut self, block_name: MuName, live_out: &Vec<P<Value>>);
    fn end_block(&mut self, block_name: MuName);

    fn emit_frame_grow(&mut self);
    fn emit_frame_shrink(&mut self);
    
    fn emit_nop(&mut self, bytes: usize);

    // comparison
    fn emit_cmp_r_r  (&mut self, op1: Reg, op2: Reg);
    fn emit_cmp_imm_r(&mut self, op1: i32, op2: Reg);
    fn emit_cmp_mem_r(&mut self, op1: Reg, op2: Reg);

    // gpr move

    // mov imm64 to r64
    fn emit_mov_r64_imm64  (&mut self, dest: Reg, src: i64);
    // mov r64 to fpr
    fn emit_mov_fpr_r64 (&mut self, dest: Reg, src: Reg);

    fn emit_mov_r_imm  (&mut self, dest: Reg, src: i32);
    fn emit_mov_r_mem  (&mut self, dest: Reg, src: Mem); // load
    fn emit_mov_r_r    (&mut self, dest: Reg, src: Reg);
    fn emit_mov_mem_r  (&mut self, dest: Mem, src: Reg); // store
    fn emit_mov_mem_imm(&mut self, dest: Mem, src: i32); // store

    // zero/sign extend mov
    fn emit_movs_r_r   (&mut self, dest: Reg, src: Reg);
    fn emit_movz_r_r   (&mut self, dest: Reg, src: Reg);

    // set byte
    fn emit_sets_r8    (&mut self, dest: Reg);
    fn emit_setz_r8    (&mut self, dest: Reg);
    fn emit_seto_r8    (&mut self, dest: Reg);
    fn emit_setb_r8    (&mut self, dest: Reg);

    fn emit_seta_r  (&mut self, dest: Reg);
    fn emit_setae_r  (&mut self, dest: Reg);
    fn emit_setb_r  (&mut self, dest: Reg);
    fn emit_setbe_r  (&mut self, dest: Reg);
    fn emit_sete_r  (&mut self, dest: Reg);
    fn emit_setg_r  (&mut self, dest: Reg);
    fn emit_setge_r  (&mut self, dest: Reg);
    fn emit_setl_r  (&mut self, dest: Reg);
    fn emit_setle_r  (&mut self, dest: Reg);
    fn emit_setne_r  (&mut self, dest: Reg);

    // gpr conditional move

    fn emit_cmova_r_r  (&mut self, dest: Reg, src: Reg);
    fn emit_cmova_r_mem(&mut self, dest: Reg, src: Mem); // load

    fn emit_cmovae_r_r  (&mut self, dest: Reg, src: Reg);
    fn emit_cmovae_r_mem(&mut self, dest: Reg, src: Mem); // load

    fn emit_cmovb_r_r  (&mut self, dest: Reg, src: Reg);
    fn emit_cmovb_r_mem(&mut self, dest: Reg, src: Mem); // load

    fn emit_cmovbe_r_r  (&mut self, dest: Reg, src: Reg);
    fn emit_cmovbe_r_mem(&mut self, dest: Reg, src: Mem); // load

    fn emit_cmove_r_r  (&mut self, dest: Reg, src: Reg);
    fn emit_cmove_r_mem(&mut self, dest: Reg, src: Mem); // load

    fn emit_cmovg_r_r  (&mut self, dest: Reg, src: Reg);
    fn emit_cmovg_r_mem(&mut self, dest: Reg, src: Mem); // load

    fn emit_cmovge_r_r  (&mut self, dest: Reg, src: Reg);
    fn emit_cmovge_r_mem(&mut self, dest: Reg, src: Mem); // load

    fn emit_cmovl_r_r  (&mut self, dest: Reg, src: Reg);
    fn emit_cmovl_r_mem(&mut self, dest: Reg, src: Mem); // load

    fn emit_cmovle_r_r  (&mut self, dest: Reg, src: Reg);
    fn emit_cmovle_r_mem(&mut self, dest: Reg, src: Mem); // load

    fn emit_cmovne_r_r  (&mut self, dest: Reg, src: Reg);
    fn emit_cmovne_r_mem(&mut self, dest: Reg, src: Mem); // load

    // lea
    fn emit_lea_r64(&mut self, dest: Reg, src: Mem);

    // and
    fn emit_and_r_imm(&mut self, dest: Reg, src: i32);
    fn emit_and_r_r  (&mut self, dest: Reg, src: Reg);
    fn emit_and_r_mem(&mut self, dest: Reg, src: Mem);

    // or
    fn emit_or_r_r  (&mut self, dest: Reg, src: Reg);
    fn emit_or_r_imm(&mut self, dest: Reg, src: i32);
    fn emit_or_r_mem(&mut self, dest: Reg, src: Mem);

    // xor
    fn emit_xor_r_r  (&mut self, dest: Reg, src: Reg);
    fn emit_xor_r_mem(&mut self, dest: Reg, src: Mem);
    fn emit_xor_r_imm(&mut self, dest: Reg, src: i32);

    // add
    fn emit_add_r_r  (&mut self, dest: Reg, src: Reg);
    fn emit_add_r_mem(&mut self, dest: Reg, src: Mem);
    fn emit_add_r_imm(&mut self, dest: Reg, src: i32);
    
    // sub
    fn emit_sub_r_r  (&mut self, dest: Reg, src: Reg);
    fn emit_sub_r_mem(&mut self, dest: Reg, src: Mem);
    fn emit_sub_r_imm(&mut self, dest: Reg, src: i32);

    // multiply
    fn emit_mul_r  (&mut self, src: Reg);
    fn emit_mul_mem(&mut self, src: Mem);

    // div
    fn emit_div_r   (&mut self, src: Reg);
    fn emit_div_mem (&mut self, src: Mem);

    // idiv
    fn emit_idiv_r  (&mut self, src: Reg);
    fn emit_idiv_mem(&mut self, src: Mem);

    // shl
    fn emit_shl_r_cl    (&mut self, dest: Reg);
    fn emit_shl_r_imm8  (&mut self, dest: Reg, src: i8);

    fn emit_shr_r_cl    (&mut self, dest: &P<Value>);
    fn emit_shr_r_imm8  (&mut self, dest: &P<Value>, src: i8);

    fn emit_sar_r_cl    (&mut self, dest: &P<Value>);
    fn emit_sar_r_imm8  (&mut self, dest: &P<Value>, src: i8);

    fn emit_cqo(&mut self); // sign extend rax to rdx:rax
    fn emit_cdq(&mut self); // sign extend eax to edx:eax
    fn emit_cwd(&mut self); // sign extend ax  to dx:ax
    
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
    
    fn emit_call_near_rel32(&mut self, callsite: String, func: MuName) -> ValueLocation;
    fn emit_call_near_r64(&mut self, callsite: String, func: &P<Value>) -> ValueLocation;
    fn emit_call_near_mem64(&mut self, callsite: String, func: &P<Value>) -> ValueLocation;
    
    fn emit_ret(&mut self);

    fn emit_push_r64(&mut self, src: &P<Value>);
    fn emit_push_imm32(&mut self, src: i32);
    fn emit_pop_r64(&mut self, dest: &P<Value>);

    // fpr move
    fn emit_movsd_f64_f64  (&mut self, dest: &P<Value>, src: &P<Value>);
    fn emit_movsd_f64_mem64(&mut self, dest: &P<Value>, src: &P<Value>); // load
    fn emit_movsd_mem64_f64(&mut self, dest: &P<Value>, src: &P<Value>); // store

    // fp add
    fn emit_addsd_f64_f64  (&mut self, dest: Reg, src: Reg);
    fn emit_addsd_f64_mem64(&mut self, dest: Reg, src: Mem);

    // fp sub
    fn emit_subsd_f64_f64  (&mut self, dest: Reg, src: Reg);
    fn emit_subsd_f64_mem64(&mut self, dest: Reg, src: Mem);

    // fp div
    fn emit_divsd_f64_f64  (&mut self, dest: Reg, src: Reg);
    fn emit_divsd_f64_mem64(&mut self, dest: Reg, src: Mem);

    // fp mul
    fn emit_mulsd_f64_f64  (&mut self, dest: Reg, src: Reg);
    fn emit_mulsd_f64_mem64(&mut self, dest: Reg, src: Mem);

    // fp comparison
    fn emit_comisd_f64_f64  (&mut self, op1: Reg, op2: Reg);
    fn emit_ucomisd_f64_f64 (&mut self, op1: Reg, op2: Reg);

    // fp conversion
    fn emit_cvtsi2sd_f64_r  (&mut self, dest: Reg, src: Reg);
    fn emit_cvtsd2si_r_f64  (&mut self, dest: Reg, src: Reg);
    fn emit_cvttsd2si_r_f64 (&mut self, dest: Reg, src: Reg);

    // used for unsigned int to fp conversion

    // unpack low data - interleave low byte
    fn emit_punpckldq_f64_mem128(&mut self, dest: Reg, src: Mem);
    // substract packed double-fp
    fn emit_subpd_f64_mem128   (&mut self, dest: Reg, src: Mem);
    // packed double-fp horizontal add
    fn emit_haddpd_f64_f64     (&mut self, dest: Reg, src: Reg);

    // move aligned packed double-precision fp values
    fn emit_movapd_f64_mem128(&mut self, dest: Reg, src: Mem);
    fn emit_movapd_f64_f64   (&mut self, dest: Reg, src: Mem);
}
