use ast::ptr::P;
use ast::ir::*;
use runtime::ValueLocation;

use compiler::machine_code::MachineCode;

pub trait CodeGenerator {
    fn start_code(&mut self, func_name: MuName) -> ValueLocation;
    fn finish_code(&mut self, func_name: MuName) -> (Box<MachineCode + Sync + Send>, ValueLocation);
    
    fn print_cur_code(&self);
    
    fn start_block(&mut self, block_name: MuName);
    fn start_exception_block(&mut self, block_name: MuName) -> ValueLocation;
    fn set_block_livein(&mut self, block_name: MuName, live_in: &Vec<P<Value>>);
    fn set_block_liveout(&mut self, block_name: MuName, live_out: &Vec<P<Value>>);
    fn end_block(&mut self, block_name: MuName);
    
    fn emit_nop(&mut self, bytes: usize);
    
    fn emit_cmp_r64_r64(&mut self, op1: &P<Value>, op2: &P<Value>);
    fn emit_cmp_r64_imm32(&mut self, op1: &P<Value>, op2: i32);
    fn emit_cmp_r64_mem64(&mut self, op1: &P<Value>, op2: &P<Value>);
    
    fn emit_mov_r64_imm32(&mut self, dest: &P<Value>, src: i32);
    fn emit_mov_r64_mem64(&mut self, dest: &P<Value>, src: &P<Value>); // load
    fn emit_mov_r64_r64(&mut self, dest: &P<Value>, src: &P<Value>);
    fn emit_mov_mem64_r64(&mut self, dest: &P<Value>, src: &P<Value>); // store
    fn emit_mov_mem64_imm32(&mut self, dest: &P<Value>, src: i32);
    
    fn emit_lea_r64(&mut self, dest: &P<Value>, src: &P<Value>);
    
    fn emit_and_r64_imm32(&mut self, dest: &P<Value>, src: i32);
    fn emit_and_r64_r64(&mut self, dest: &P<Value>, src: &P<Value>);
    
    fn emit_add_r64_r64(&mut self, dest: &P<Value>, src: &P<Value>);
    fn emit_add_r64_mem64(&mut self, dest: &P<Value>, src: &P<Value>);
    fn emit_add_r64_imm32(&mut self, dest: &P<Value>, src: i32);
    
    fn emit_sub_r64_r64(&mut self, dest: &P<Value>, src: &P<Value>);
    fn emit_sub_r64_mem64(&mut self, dest: &P<Value>, src: &P<Value>);
    fn emit_sub_r64_imm32(&mut self, dest: &P<Value>, src: i32);
    
    fn emit_mul_r64(&mut self, src: &P<Value>);
    fn emit_mul_mem64(&mut self, src: &P<Value>);
    
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
    fn emit_pop_r64(&mut self, dest: &P<Value>);
}
