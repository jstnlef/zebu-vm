#![allow(unused_variables)]

use compiler::backend::x86_64::CodeGenerator;

use ast::ptr::P;
use ast::ir::*;
use ast::types::*;

pub struct ASMCodeGen {
    foo: usize
}

impl ASMCodeGen {
    pub fn new() -> ASMCodeGen {
        ASMCodeGen {foo: 0}
    }
}

impl CodeGenerator for ASMCodeGen {
    fn emit_cmp_r64_r64(&mut self, op1: &P<Value>, op2: &P<Value>) {
        
    }
    
    fn emit_cmp_r64_imm32(&mut self, op1: &P<Value>, op2: &P<Value>) {
        
    }
    
    fn emit_cmp_r64_mem64(&mut self, op1: &P<Value>, op2: &P<Value>) {
        
    }
    
    fn emit_mov_r64_imm32(&mut self, dest: &P<Value>, src: &P<Value>) {
        
    }
    
    fn emit_mov_r64_mem64(&mut self, dest: &P<Value>, src: &P<Value>) {
        
    }
}