#![allow(unused_variables)]

use compiler::backend::x86_64::CodeGenerator;

use ast::ptr::P;
use ast::ir::*;
use ast::types::*;
use ast::inst::*;

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
        trace!("emit: cmp {} {}", op1, op2);
    }
    
    fn emit_cmp_r64_imm32(&mut self, op1: &P<Value>, op2: u32) {
        trace!("emit: cmp {} {}", op1, op2);
    }
    
    fn emit_cmp_r64_mem64(&mut self, op1: &P<Value>, op2: &P<Value>) {
        trace!("emit: cmp {} {}", op1, op2);
    }
    
    fn emit_mov_r64_imm32(&mut self, dest: &P<Value>, src: u32) {
        trace!("emit: mov {} -> {}", src, dest);
    }
    
    fn emit_mov_r64_mem64(&mut self, dest: &P<Value>, src: &P<Value>) {
        trace!("emit: mov {} -> {}", src, dest);
    }
    
    fn emit_mov_r64_r64(&mut self, dest: &P<Value>, src: &P<Value>) {
        trace!("emit: mov {} -> {}", src, dest);
    }
    
    fn emit_add_r64_r64(&mut self, dest: &P<Value>, src: &P<Value>) {
        trace!("emit: add {}, {} -> {}", dest, src, dest);
    }
    
    fn emit_add_r64_mem64(&mut self, dest: &P<Value>, src: &P<Value>) {
        trace!("emit: add {}, {} -> {}", dest, src, dest);
    }
    
    fn emit_add_r64_imm32(&mut self, dest: &P<Value>, src: u32) {
        trace!("emit: add {}, {} -> {}", dest, src, dest);
    }
    
    fn emit_sub_r64_r64(&mut self, dest: &P<Value>, src: &P<Value>) {
        trace!("emit: sub {}, {} -> {}", dest, src, dest);
    }
    
    fn emit_sub_r64_mem64(&mut self, dest: &P<Value>, src: &P<Value>) {
        trace!("emit: sub {}, {} -> {}", dest, src, dest);
    }
    
    fn emit_sub_r64_imm32(&mut self, dest: &P<Value>, src: u32) {
        trace!("emit: sub {}, {} -> {}", dest, src, dest);
    }
    
    fn emit_mul_r64(&mut self, src: &P<Value>) {
        trace!("emit: mul rax, {} -> rax", src);
    }
    
    fn emit_mul_mem64(&mut self, src: &P<Value>) {
        trace!("emit: mul rax, {} -> rax", src);
    }
    
    fn emit_jmp(&mut self, dest: &Destination) {
        trace!("emit: jmp {}", dest.target);
    }
    
    fn emit_je(&mut self, dest: &Destination) {
        trace!("emit: je {}", dest.target);
    }
    
    fn emit_jne(&mut self, dest: &Destination) {
        trace!("emit: jne {}", dest.target);
    }
    
    fn emit_ja(&mut self, dest: &Destination) {
        trace!("emit: ja {}", dest.target);
    }
    
    fn emit_jae(&mut self, dest: &Destination) {
        trace!("emit: jae {}", dest.target);
    }
    
    fn emit_jb(&mut self, dest: &Destination) {
        trace!("emit: jb {}", dest.target);
    }
    
    fn emit_jbe(&mut self, dest: &Destination) {
        trace!("emit: jbe {}", dest.target);
    }
    
    fn emit_jg(&mut self, dest: &Destination) {
        trace!("emit: jg {}", dest.target);
    }
    
    fn emit_jge(&mut self, dest: &Destination) {
        trace!("emit: jge {}", dest.target);
    }
    
    fn emit_jl(&mut self, dest: &Destination) {
        trace!("emit: jl {}", dest.target);
    }
    
    fn emit_jle(&mut self, dest: &Destination) {
        trace!("emit: jle {}", dest.target);
    }    
    
    fn emit_call_near_rel32(&mut self, func: MuTag) {
        trace!("emit: call {}", func);
    }
    
    fn emit_call_near_r64(&mut self, func: &P<Value>) {
        trace!("emit: call {}", func);
    }
    
    fn emit_call_near_mem64(&mut self, func: &P<Value>) {
        trace!("emit: call {}", func);
    }
    
    fn emit_ret(&mut self) {
        trace!("emit: ret");
    }
    
    fn emit_push(&mut self, src: &P<Value>) {
        trace!("emit: push {}", src);
    }
    
    fn emit_pop(&mut self, dest: &P<Value>) {
        trace!("emit: pop {}", dest);
    }    
}