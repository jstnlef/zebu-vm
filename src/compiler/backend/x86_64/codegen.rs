use ast::ptr::P;
use ast::ir::*;

pub trait CodeGenerator {
    fn emit_cmp_r64_r64(&mut self, op1: &P<Value>, op2: &P<Value>);
    fn emit_cmp_r64_imm32(&mut self, op1: &P<Value>, op2: &P<Value>);
    fn emit_cmp_r64_mem64(&mut self, op1: &P<Value>, op2: &P<Value>);
    
    fn emit_mov_r64_imm32(&mut self, dest: &P<Value>, src: &P<Value>);
    fn emit_mov_r64_mem64(&mut self, dest: &P<Value>, src: &P<Value>);
}