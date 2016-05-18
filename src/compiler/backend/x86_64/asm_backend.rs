#![allow(unused_variables)]

use compiler::backend::x86_64::CodeGenerator;

use ast::ptr::P;
use ast::ir::*;
use ast::types::*;
use ast::inst::*;

use std::collections::HashMap;
use std::fmt;
use std::str;

struct ASMCode {
    name: MuTag, 
    code: Vec<String>,
    regs: HashMap<MuID, RegUses>,
}

pub struct ASMCodeGen {
    cur: Option<ASMCode>,
    
    all_code: HashMap<MuTag, ASMCode>
}

struct RegUses {
    locs: Vec<ASMLocation>
}

struct ASMLocation {
    line: usize,
    index: usize,
    len: usize
}

impl ASMLocation {
    fn new(line: usize, index: usize, len: usize) -> ASMLocation {
        ASMLocation{
            line: line,
            index: index,
            len: len
        }
    }
}

const REG_PLACEHOLDER_LEN : usize = 3;
lazy_static! {
    pub static ref REG_PLACEHOLDER : String = {
        let blank_spaces = [' ' as u8; REG_PLACEHOLDER_LEN];
        
        format!("%{}", str::from_utf8(&blank_spaces).unwrap())
    };
}

impl ASMCodeGen {
    pub fn new() -> ASMCodeGen {
        ASMCodeGen {
            cur: None,
            all_code: HashMap::new()
        }
    }
    
    fn cur(&self) -> &ASMCode {
        self.cur.as_ref().unwrap()
    }
    
    fn cur_mut(&mut self) -> &mut ASMCode {
        self.cur.as_mut().unwrap()
    }
    
    fn replace(s: &mut String, index: usize, replace: &str, replace_len: usize) {
        let vec = unsafe {s.as_mut_vec()};
        
        for i in 0..replace_len {
            if i < replace.len() {
                vec[index + i] = replace.as_bytes()[i] as u8;
            } else {
                vec[index + i] = ' ' as u8;
            }
        }
    }
    
    /// return line number for this code
    fn add_assembly(&mut self, code: String) -> usize {
        let len = self.cur_mut().code.len();
        self.cur_mut().code.push(code);
        
        len
    }
    
    fn use_reg(&mut self, reg: &P<Value>, loc: ASMLocation) {
        let id = reg.extract_ssa_id().unwrap();
        
        let code = self.cur_mut();
        if code.regs.contains_key(&id) {
            let reg_uses = code.regs.get_mut(&id).unwrap();
            reg_uses.locs.push(loc);
        } else {
            code.regs.insert(id, RegUses {
                    locs: vec![loc]
                });
        } 
    }
    
    fn asm_reg_op(&self, op: &P<Value>) -> String {
        let id = op.extract_ssa_id().unwrap();
        if id < RESERVED_NODE_IDS_FOR_MACHINE {
            // machine reg
            format!("%{}", op.tag)
        } else {
            // virtual register, use place holder
            REG_PLACEHOLDER.clone()
        }
    }
    
    fn asm_block_label(&self, label: MuTag) -> String {
        format!("{}_{}", self.cur().name, label)
    }
}

impl CodeGenerator for ASMCodeGen {
    fn start_code(&mut self, func_name: MuTag) {
        self.cur = Some(ASMCode {
                name: func_name,
                code: vec![],
                regs: HashMap::new()
            });
        
        self.add_assembly(format!(".globl {}", func_name));
    }
    
    fn finish_code(&mut self) {
        let finish = self.cur.take().unwrap();
        
        self.all_code.insert(finish.name, finish);
    }
    
    fn print_cur_code(&self) {
        println!("");
        
        if self.cur.is_some() {
            let code = self.cur.as_ref().unwrap();
            
            println!("code for {}: ", code.name);
            for line in code.code.iter() {
                println!("{}", line);
            }
        } else {
            println!("no current code");
        }
        
        println!("");
    }
    
    fn start_block(&mut self, block_name: MuTag) {
        let label = format!("{}:", self.asm_block_label(block_name));
        self.add_assembly(label);
    }
    
    fn emit_cmp_r64_r64(&mut self, op1: &P<Value>, op2: &P<Value>) {
        trace!("emit: cmp {} {}", op1, op2);
        
        let reg1 = self.asm_reg_op(op1);
        let reg2 = self.asm_reg_op(op2);
        
        let asm = format!("cmpq {} {}", reg1, reg2);
        let line = self.add_assembly(asm);
        
        let loc_reg1 = ASMLocation::new(line, 4 + 1, reg1.len());
        self.use_reg(op1, loc_reg1);
        
        let loc_reg2 = ASMLocation::new(line, 4 + 1 + reg1.len() + 1, reg2.len());
        self.use_reg(op2, loc_reg2);
    }
    
    fn emit_cmp_r64_imm32(&mut self, op1: &P<Value>, op2: u32) {
        trace!("emit: cmp {} {}", op1, op2);
        
        let reg1 = self.asm_reg_op(op1);
        
        let asm = format!("cmpq {} ${}", reg1, op2);
        let line = self.add_assembly(asm);
        
        let loc_reg1 = ASMLocation::new(line, 4 + 1, reg1.len());
        self.use_reg(op1, loc_reg1);
    }
    
    fn emit_cmp_r64_mem64(&mut self, op1: &P<Value>, op2: &P<Value>) {
        trace!("emit: cmp {} {}", op1, op2);
        unimplemented!()
    }
    
    fn emit_mov_r64_imm32(&mut self, dest: &P<Value>, src: u32) {
        trace!("emit: mov {} -> {}", src, dest);
        
        let reg1 = self.asm_reg_op(dest);
        
        let asm = format!("movq {} ${}", src, reg1);
        let line = self.add_assembly(asm);
        
        let loc_reg1 = ASMLocation::new(line, 4 + 1, reg1.len());
        self.use_reg(dest, loc_reg1);
    }
    
    fn emit_mov_r64_mem64(&mut self, dest: &P<Value>, src: &P<Value>) {
        trace!("emit: mov {} -> {}", src, dest);
        unimplemented!()
    }
    
    fn emit_mov_r64_r64(&mut self, dest: &P<Value>, src: &P<Value>) {
        trace!("emit: mov {} -> {}", src, dest);
        
        let reg1 = self.asm_reg_op(dest);
        let reg2 = self.asm_reg_op(src);
        
        let asm = format!("movq {} {}", reg2, reg1);
        let line = self.add_assembly(asm);
        
        let loc_reg1 = ASMLocation::new(line, 4 + 1, reg1.len());
        self.use_reg(dest, loc_reg1);
        let loc_reg2 = ASMLocation::new(line, 4 + 1 + reg1.len() + 1, reg2.len());
        self.use_reg(src, loc_reg2);
    }
    
    fn emit_add_r64_r64(&mut self, dest: &P<Value>, src: &P<Value>) {
        trace!("emit: add {}, {} -> {}", dest, src, dest);
        
        let reg1 = self.asm_reg_op(dest);
        let reg2 = self.asm_reg_op(src);
        
        let asm = format!("addq {} {}", reg2, reg1);
        let line = self.add_assembly(asm);
        
        let loc_reg1 = ASMLocation::new(line, 4 + 1, reg1.len());
        self.use_reg(dest, loc_reg1);
        let loc_reg2 = ASMLocation::new(line, 4 + 1 + reg1.len() + 1, reg2.len());
        self.use_reg(src, loc_reg2);
    }
    
    fn emit_add_r64_mem64(&mut self, dest: &P<Value>, src: &P<Value>) {
        trace!("emit: add {}, {} -> {}", dest, src, dest);
        unimplemented!()
    }
    
    fn emit_add_r64_imm32(&mut self, dest: &P<Value>, src: u32) {
        trace!("emit: add {}, {} -> {}", dest, src, dest);
        
        let reg1 = self.asm_reg_op(dest);
        
        let asm = format!("addq {} ${}", src, reg1);
        let line = self.add_assembly(asm);
        
        let loc_reg1 = ASMLocation::new(line, 4 + 1, reg1.len());
        self.use_reg(dest, loc_reg1);
    }
    
    fn emit_sub_r64_r64(&mut self, dest: &P<Value>, src: &P<Value>) {
        trace!("emit: sub {}, {} -> {}", dest, src, dest);
        
        let reg1 = self.asm_reg_op(dest);
        let reg2 = self.asm_reg_op(src);
        
        let asm = format!("subq {} {}", reg2, reg1);
        let line = self.add_assembly(asm);
        
        let loc_reg1 = ASMLocation::new(line, 4 + 1, reg1.len());
        self.use_reg(dest, loc_reg1);
        let loc_reg2 = ASMLocation::new(line, 4 + 1 + reg1.len() + 1, reg2.len());
        self.use_reg(src, loc_reg2);        
    }
    
    fn emit_sub_r64_mem64(&mut self, dest: &P<Value>, src: &P<Value>) {
        trace!("emit: sub {}, {} -> {}", dest, src, dest);
        unimplemented!()
    }
    
    fn emit_sub_r64_imm32(&mut self, dest: &P<Value>, src: u32) {
        trace!("emit: sub {}, {} -> {}", dest, src, dest);
        
        let reg1 = self.asm_reg_op(dest);
        
        let asm = format!("subq {} ${}", src, reg1);
        let line = self.add_assembly(asm);
        
        let loc_reg1 = ASMLocation::new(line, 4 + 1, reg1.len());
        self.use_reg(dest, loc_reg1);        
    }
    
    fn emit_mul_r64(&mut self, src: &P<Value>) {
        trace!("emit: mul rax, {} -> rax", src);
        
        let reg = self.asm_reg_op(src);
        
        let asm = format!("mul {}", reg);
        let line = self.add_assembly(asm);
        
        let loc_reg = ASMLocation::new(line, 3 + 1, reg.len());
        self.use_reg(src, loc_reg);
    }
    
    fn emit_mul_mem64(&mut self, src: &P<Value>) {
        trace!("emit: mul rax, {} -> rax", src);
        unimplemented!()
    }
    
    fn emit_jmp(&mut self, dest: &Destination) {
        trace!("emit: jmp {}", dest.target);
        
        // symbolic label, we dont need to patch it
        let asm = format!("jmp {}", self.asm_block_label(dest.target));
        self.add_assembly(asm);
    }
    
    fn emit_je(&mut self, dest: &Destination) {
        trace!("emit: je {}", dest.target);
        
        let asm = format!("je {}", self.asm_block_label(dest.target));
        self.add_assembly(asm);        
    }
    
    fn emit_jne(&mut self, dest: &Destination) {
        trace!("emit: jne {}", dest.target);
        
        let asm = format!("jne {}", self.asm_block_label(dest.target));
        self.add_assembly(asm);        
    }
    
    fn emit_ja(&mut self, dest: &Destination) {
        trace!("emit: ja {}", dest.target);
        
        let asm = format!("ja {}", self.asm_block_label(dest.target));
        self.add_assembly(asm);        
    }
    
    fn emit_jae(&mut self, dest: &Destination) {
        trace!("emit: jae {}", dest.target);
        
        let asm = format!("jae {}", self.asm_block_label(dest.target));
        self.add_assembly(asm);        
    }
    
    fn emit_jb(&mut self, dest: &Destination) {
        trace!("emit: jb {}", dest.target);
        
        let asm = format!("jb {}", self.asm_block_label(dest.target));
        self.add_assembly(asm);        
    }
    
    fn emit_jbe(&mut self, dest: &Destination) {
        trace!("emit: jbe {}", dest.target);
        
        let asm = format!("jbe {}", self.asm_block_label(dest.target));
        self.add_assembly(asm);        
    }
    
    fn emit_jg(&mut self, dest: &Destination) {
        trace!("emit: jg {}", dest.target);
        
        let asm = format!("jg {}", self.asm_block_label(dest.target));
        self.add_assembly(asm);        
    }
    
    fn emit_jge(&mut self, dest: &Destination) {
        trace!("emit: jge {}", dest.target);
        
        let asm = format!("jge {}", self.asm_block_label(dest.target));
        self.add_assembly(asm);        
    }
    
    fn emit_jl(&mut self, dest: &Destination) {
        trace!("emit: jl {}", dest.target);
        
        let asm = format!("jl {}", self.asm_block_label(dest.target));
        self.add_assembly(asm);        
    }
    
    fn emit_jle(&mut self, dest: &Destination) {
        trace!("emit: jle {}", dest.target);
        
        let asm = format!("jle {}", self.asm_block_label(dest.target));
        self.add_assembly(asm);        
    }    
    
    fn emit_call_near_rel32(&mut self, func: MuTag) {
        trace!("emit: call {}", func);
        
        let asm = format!("call {}", func);
        self.add_assembly(asm);
    }
    
    fn emit_call_near_r64(&mut self, func: &P<Value>) {
        trace!("emit: call {}", func);
        unimplemented!()
    }
    
    fn emit_call_near_mem64(&mut self, func: &P<Value>) {
        trace!("emit: call {}", func);
        unimplemented!()
    }
    
    fn emit_ret(&mut self) {
        trace!("emit: ret");
        
        let asm = format!("ret");
        self.add_assembly(asm);
    }
    
    fn emit_push_r64(&mut self, src: &P<Value>) {
        trace!("emit: push {}", src);
        
        let reg = self.asm_reg_op(src);
        
        let asm = format!("pushq {}", reg);
        let line = self.add_assembly(asm);
        
        let loc_reg = ASMLocation::new(line, 5 + 1, reg.len());
        self.use_reg(src, loc_reg);
    }
    
    fn emit_pop_r64(&mut self, dest: &P<Value>) {
        trace!("emit: pop {}", dest);
        
        let reg = self.asm_reg_op(dest);
        
        let asm = format!("popq {}", reg);
        let line = self.add_assembly(asm);
        
        let loc_reg = ASMLocation::new(line, 4 + 1, reg.len());
        self.use_reg(dest, loc_reg);        
    }    
}