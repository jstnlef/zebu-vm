use ast::ptr::P;
use ast::ir::*;
use ast::inst::*;

pub trait CodeGenerator {
    fn emit_cmp_r64_r64(&mut self, op1: &P<Value>, op2: &P<Value>);
    fn emit_cmp_r64_imm32(&mut self, op1: &P<Value>, op2: u32);
    fn emit_cmp_r64_mem64(&mut self, op1: &P<Value>, op2: &P<Value>);
    
    fn emit_mov_r64_imm32(&mut self, dest: &P<Value>, src: u32);
    fn emit_mov_r64_mem64(&mut self, dest: &P<Value>, src: &P<Value>);
    fn emit_mov_r64_r64(&mut self, dest: &P<Value>, src: &P<Value>);
    
    fn emit_add_r64_r64(&mut self, dest: &P<Value>, src: &P<Value>);
    fn emit_add_r64_mem64(&mut self, dest: &P<Value>, src: &P<Value>);
    fn emit_add_r64_imm32(&mut self, dest: &P<Value>, src: u32);
    
    fn emit_sub_r64_r64(&mut self, dest: &P<Value>, src: &P<Value>);
    fn emit_sub_r64_mem64(&mut self, dest: &P<Value>, src: &P<Value>);
    fn emit_sub_r64_imm32(&mut self, dest: &P<Value>, src: u32);
    
    fn emit_mul_r64(&mut self, src: &P<Value>);
    fn emit_mul_mem64(&mut self, src: &P<Value>);
    
    fn emit_jmp(&mut self, dest: &Destination);
    fn emit_je(&mut self, dest: &Destination);
    fn emit_jne(&mut self, dest: &Destination);
    fn emit_ja(&mut self, dest: &Destination);
    fn emit_jae(&mut self, dest: &Destination);
    fn emit_jb(&mut self, dest: &Destination);
    fn emit_jbe(&mut self, dest: &Destination);
    fn emit_jg(&mut self, dest: &Destination);
    fn emit_jge(&mut self, dest: &Destination);
    fn emit_jl(&mut self, dest: &Destination);
    fn emit_jle(&mut self, dest: &Destination);
    
    fn emit_call_near_rel32(&mut self, func: MuTag);
    fn emit_call_near_r64(&mut self, func: &P<Value>);
    fn emit_call_near_mem64(&mut self, func: &P<Value>);
    
    fn emit_ret(&mut self);
    
    fn emit_push(&mut self, src: &P<Value>);
    fn emit_pop(&mut self, dest: &P<Value>);
}