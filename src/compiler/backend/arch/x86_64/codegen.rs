use ast::ptr::P;
use ast::ir::*;
use runtime::ValueLocation;

use compiler::machine_code::MachineCode;

pub type Reg<'a> = &'a P<Value>;
pub type Mem<'a> = &'a P<Value>;

pub trait CodeGenerator {
    fn start_code(&mut self, func_name: MuName) -> ValueLocation;
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
    
    fn emit_nop(&mut self, bytes: usize);

    // comparison
    fn emit_cmp_r64_r64  (&mut self, op1: Reg, op2: Reg);
    fn emit_cmp_r64_imm32(&mut self, op1: Reg, op2: i32);
    fn emit_cmp_r64_mem64(&mut self, op1: Reg, op2: Mem);

    fn emit_cmp_r32_r32  (&mut self, op1: Reg, op2: Reg);
    fn emit_cmp_r32_imm32(&mut self, op1: Reg, op2: i32);
    fn emit_cmp_r32_mem32(&mut self, op1: Reg, op2: Mem);

    fn emit_cmp_r16_r16  (&mut self, op1: Reg, op2: Reg);
    fn emit_cmp_r16_imm16(&mut self, op1: Reg, op2: i16);
    fn emit_cmp_r16_mem16(&mut self, op1: Reg, op2: Mem);

    fn emit_cmp_r8_r8    (&mut self, op1: Reg, op2: Reg);
    fn emit_cmp_r8_imm8  (&mut self, op1: Reg, op2: i8);
    fn emit_cmp_r8_mem8  (&mut self, op1: Reg, op2: Mem);

    // gpr move
    
    fn emit_mov_r64_imm32  (&mut self, dest: Reg, src: i32);
    fn emit_mov_r64_mem64  (&mut self, dest: Reg, src: Mem); // load
    fn emit_mov_r64_r64    (&mut self, dest: Reg, src: Reg);
    fn emit_mov_mem64_r64  (&mut self, dest: Mem, src: Reg); // store
    fn emit_mov_mem64_imm32(&mut self, dest: Mem, src: i32);

    fn emit_mov_r32_imm32  (&mut self, dest: Reg, src: i32);
    fn emit_mov_r32_mem32  (&mut self, dest: Reg, src: Mem); // load
    fn emit_mov_r32_r32    (&mut self, dest: Reg, src: Reg);
    fn emit_mov_mem32_r32  (&mut self, dest: Mem, src: Reg); // store
    fn emit_mov_mem32_imm32(&mut self, dest: Mem, src: i32);
    
    fn emit_mov_r16_imm16  (&mut self, dest: Reg, src: i16);
    fn emit_mov_r16_mem16  (&mut self, dest: Reg, src: Mem); // load
    fn emit_mov_r16_r16    (&mut self, dest: Reg, src: Reg);
    fn emit_mov_mem16_r16  (&mut self, dest: Mem, src: Reg); // store
    fn emit_mov_mem16_imm16(&mut self, dest: Mem, src: i16);

    fn emit_mov_r8_imm8    (&mut self, dest: Reg, src: i8);
    fn emit_mov_r8_mem8    (&mut self, dest: Reg, src: Mem); // load
    fn emit_mov_r8_r8      (&mut self, dest: Reg, src: Mem);
    fn emit_mov_mem8_r8    (&mut self, dest: Mem, src: Reg); // store
    fn emit_mov_mem8_imm8  (&mut self, dest: Mem, src: i8);

    // lea
    fn emit_lea_r64(&mut self, dest: Reg, src: Reg);

    // and
    fn emit_and_r64_imm32(&mut self, dest: Reg, src: i32);
    fn emit_and_r64_r64  (&mut self, dest: Reg, src: Reg);
    fn emit_and_r64_mem64(&mut self, dest: Reg, src: Mem);

    fn emit_and_r32_imm32(&mut self, dest: Reg, src: i32);
    fn emit_and_r32_r32  (&mut self, dest: Reg, src: Reg);
    fn emit_and_r32_mem32(&mut self, dest: Reg, src: Mem);

    fn emit_and_r16_imm16(&mut self, dest: Reg, src: i16);
    fn emit_and_r16_r16  (&mut self, dest: Reg, src: Reg);
    fn emit_and_r16_mem16(&mut self, dest: Reg, src: Mem);

    fn emit_and_r8_imm8  (&mut self, dest: Reg, src: i8);
    fn emit_and_r8_r8    (&mut self, dest: Reg, src: Reg);
    fn emit_and_r8_mem8  (&mut self, dest: Reg, src: Mem);

    // xor
    fn emit_xor_r64_r64  (&mut self, dest: Reg, src: Reg);
    fn emit_xor_r64_mem64(&mut self, dest: Reg, src: Mem);
    fn emit_xor_r64_imm32(&mut self, dest: Reg, src: i32);

    fn emit_xor_r32_r32  (&mut self, dest: Reg, src: Reg);
    fn emit_xor_r32_mem32(&mut self, dest: Reg, src: Mem);
    fn emit_xor_r32_imm32(&mut self, dest: Reg, src: i32);

    fn emit_xor_r16_r16  (&mut self, dest: Reg, src: Reg);
    fn emit_xor_r16_mem16(&mut self, dest: Reg, src: Reg);
    fn emit_xor_r16_imm16(&mut self, dest: Reg, src: i16);

    fn emit_xor_r8_r8    (&mut self, dest: Reg, src: Reg);
    fn emit_xor_r8_mem8  (&mut self, dest: Reg, src: Reg);
    fn emit_xor_r8_imm8  (&mut self, dest: Reg, src: i8);

    // and
    fn emit_add_r64_r64  (&mut self, dest: Reg, src: Reg);
    fn emit_add_r64_mem64(&mut self, dest: Reg, src: Mem);
    fn emit_add_r64_imm32(&mut self, dest: Reg, src: i32);
//
//    fn emit_add_r32_r32  (&mut self, dest: Reg, src: Reg);
//    fn emit_add_r32_mem32(&mut self, dest: Reg, src: Mem);
//    fn emit_add_r32_imm32(&mut self, dest: Reg, src: i32);
//
//    fn emit_add_r16_r16  (&mut self, dest: Reg, src: Reg);
//    fn emit_add_r16_mem16(&mut self, dest: Reg, src: Mem);
//    fn emit_add_r16_imm16(&mut self, dest: Reg, src: i16);
//
//    fn emit_add_r8_r8  (&mut self, dest: Reg, src: Reg);
//    fn emit_add_r8_mem8(&mut self, dest: Reg, src: Mem);
//    fn emit_add_r8_imm8(&mut self, dest: Reg, src: i8);

    fn emit_addsd_f64_f64  (&mut self, dest: &P<Value>, src: &P<Value>);
    fn emit_addsd_f64_mem64(&mut self, dest: &P<Value>, src: &P<Value>);
    
    fn emit_sub_r64_r64(&mut self, dest: &P<Value>, src: &P<Value>);
    fn emit_sub_r64_mem64(&mut self, dest: &P<Value>, src: &P<Value>);
    fn emit_sub_r64_imm32(&mut self, dest: &P<Value>, src: i32);
    
    fn emit_mul_r64  (&mut self, src: &P<Value>);
    fn emit_mul_mem64(&mut self, src: &P<Value>);

    fn emit_div_r64   (&mut self, src: &P<Value>);
    fn emit_div_mem64 (&mut self, src: &P<Value>);
    fn emit_idiv_r64  (&mut self, src: &P<Value>);
    fn emit_idiv_mem64(&mut self, src: &P<Value>);

    fn emit_shl_r64_cl    (&mut self, dest: &P<Value>);
    fn emit_shl_mem64_cl  (&mut self, dest: &P<Value>);
    fn emit_shl_r64_imm8  (&mut self, dest: &P<Value>, src: i8);
    fn emit_shl_mem64_imm8(&mut self, dest: &P<Value>, src: i8);

    fn emit_shr_r64_cl    (&mut self, dest: &P<Value>);
    fn emit_shr_mem64_cl  (&mut self, dest: &P<Value>);
    fn emit_shr_r64_imm8  (&mut self, dest: &P<Value>, src: i8);
    fn emit_shr_mem64_imm8(&mut self, dest: &P<Value>, src: i8);

    fn emit_sar_r64_cl    (&mut self, dest: &P<Value>);
    fn emit_sar_mem64_cl  (&mut self, dest: &P<Value>);
    fn emit_sar_r64_imm8  (&mut self, dest: &P<Value>, src: i8);
    fn emit_sar_mem64_imm8(&mut self, dest: &P<Value>, src: i8);

    fn emit_cqo(&mut self);
    
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
}