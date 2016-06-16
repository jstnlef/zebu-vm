#![allow(unused_variables)]

use compiler::backend::x86_64;
use compiler::backend::x86_64::CodeGenerator;
use vm::machine_code::MachineCode;

use ast::ptr::P;
use ast::ir::*;
use ast::inst::*;

use std::collections::HashMap;
use std::str;
use std::usize;

struct ASMCode {
    name: MuTag, 
    code: Vec<ASM>,
    reg_defines: HashMap<MuID, Vec<ASMLocation>>,
    reg_uses: HashMap<MuID, Vec<ASMLocation>>,
    
    preds: Vec<Vec<usize>>,
    succs: Vec<Vec<usize>>,
    
    idx_to_blk: HashMap<usize, MuTag>,
    blk_to_idx: HashMap<MuTag, usize>,
    cond_branches: HashMap<usize, MuTag>,
    branches: HashMap<usize, MuTag>
}

impl MachineCode for ASMCode {
    fn number_of_insts(&self) -> usize {
        self.code.len()
    }
    
    fn is_move(&self, index: usize) -> bool {
        let inst = self.code.get(index);
        match inst {
            Some(inst) => inst.code.starts_with("mov"),
            None => false
        }
    }
    
    fn get_succs(&self, index: usize) -> &Vec<usize> {
        &self.succs[index]
    }
    
    fn get_preds(&self, index: usize) -> &Vec<usize> {
        &self.preds[index]
    }
    
    fn get_inst_reg_uses(&self, index: usize) -> &Vec<MuID> {
        &self.code[index].uses
    }
    
    fn get_inst_reg_defines(&self, index: usize) -> &Vec<MuID> {
        &self.code[index].defines
    }
    
    fn print(&self) {
        println!("");

        println!("code for {}: ", self.name);
        let n_insts = self.code.len();
        for i in 0..n_insts {
            let ref line = self.code[i];
            println!("#{}\t{}\t\tpred: {:?}, succ: {:?}", i, line.code, self.preds[i], self.succs[i]);
        }
        
        println!("");        
    }
}

struct ASM {
    code: String,
    defines: Vec<MuID>,
    uses: Vec<MuID>
}

impl ASM {
    fn symbolic(line: String) -> ASM {
        ASM {
            code: line,
            defines: vec![],
            uses: vec![]
        }
    }
    
    fn inst(inst: String, defines: Vec<MuID>, uses: Vec<MuID>) -> ASM {
        ASM {
            code: inst,
            defines: defines,
            uses: uses
        }
    }
    
    fn branch(line: String) -> ASM {
        ASM {
            code: line,
            defines: vec![],
            uses: vec![]
        }
    }
    
    fn call(line: String) -> ASM {
        ASM {
            code: line,
            defines: vec![],
            uses: vec![]
        }
    }
    
    fn ret(line: String) -> ASM {
        ASM {
            code: line,
            defines: vec![],
            uses: vec![]
        }
    }
}

#[derive(Clone, Debug)]
struct ASMLocation {
    line: usize,
    index: usize,
    len: usize
}

impl ASMLocation {
    /// the 'line' field will be updated later
    fn new(index: usize, len: usize) -> ASMLocation {
        ASMLocation{
            line: usize::MAX,
            index: index,
            len: len
        }
    }
}

pub struct ASMCodeGen {
    cur: Option<Box<ASMCode>>
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
            cur: None
        }
    }
    
    fn cur(&self) -> &ASMCode {
        self.cur.as_ref().unwrap()
    }
    
    fn cur_mut(&mut self) -> &mut ASMCode {
        self.cur.as_mut().unwrap()
    }
    
    fn line(&self) -> usize {
        self.cur().code.len()
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
    
    fn add_asm_block_label(&mut self, code: String, block_name: &'static str) {
        let l = self.line();
        self.cur_mut().code.push(ASM::symbolic(code));
        
        self.cur_mut().idx_to_blk.insert(l, block_name);
        self.cur_mut().blk_to_idx.insert(block_name, l);
    }
    
    fn add_asm_symbolic(&mut self, code: String){
        self.cur_mut().code.push(ASM::symbolic(code));
    }
    
    fn add_asm_call(&mut self, code: String) {
        self.cur_mut().code.push(ASM::call(code));
    }
    
    fn add_asm_ret(&mut self, code: String) {
        self.cur_mut().code.push(ASM::ret(code));
    }
    
    fn add_asm_branch(&mut self, code: String, target: &'static str) {
        let l = self.line();
        self.cur_mut().code.push(ASM::branch(code));
        
        self.cur_mut().branches.insert(l, target);
    }
    
    fn add_asm_branch2(&mut self, code: String, target: &'static str) {
        let l = self.line();
        self.cur_mut().code.push(ASM::branch(code));
        
        self.cur_mut().cond_branches.insert(l, target);
    }
    
    fn add_asm_inst(
        &mut self, 
        code: String, 
        defines: Vec<MuID>,
        mut define_locs: Vec<ASMLocation>, 
        uses: Vec<MuID>,
        mut use_locs: Vec<ASMLocation>) 
    {
        let line = self.line();
        
        trace!("asm: {}", code);
        trace!("     defines: {:?}, def_locs: {:?}", defines, define_locs);
        trace!("     uses: {:?}, use_locs: {:?}", uses, use_locs);
        let mc = self.cur_mut();
       
        // add locations of defined registers
        for i in 0..define_locs.len() {
            let id = defines[i];
            
            // update line in location
            let ref mut loc = define_locs[i];
            loc.line = line;
            
            if mc.reg_defines.contains_key(&id) {
                mc.reg_defines.get_mut(&id).unwrap().push(loc.clone());
            } else {
                mc.reg_defines.insert(id, vec![loc.clone()]);
            }
        }
       
        for i in 0..use_locs.len() {
            let id = uses[i];
            
            // update line in location
            let ref mut loc = use_locs[i];
            loc.line = line;
            
            if mc.reg_uses.contains_key(&id) {
                mc.reg_defines.get_mut(&id).unwrap().push(loc.clone());
            } else {
                mc.reg_defines.insert(id, vec![loc.clone()]);
            }
        }
       
        // put the instruction
        mc.code.push(ASM::inst(code, defines, uses));
    }
    
    fn define_reg(&mut self, reg: &P<Value>, loc: ASMLocation) {
        let id = reg.extract_ssa_id().unwrap();
        
        let code = self.cur_mut();
        if code.reg_defines.contains_key(&id) {
            let regs = code.reg_defines.get_mut(&id).unwrap();
            regs.push(loc);
        } else {
            code.reg_defines.insert(id, vec![loc]);
        } 
    }
    
    fn use_reg(&mut self, reg: &P<Value>, loc: ASMLocation) {
        let id = reg.extract_ssa_id().unwrap();
        
        let code = self.cur_mut();
        if code.reg_uses.contains_key(&id) {
            let reg_uses = code.reg_uses.get_mut(&id).unwrap();
            reg_uses.push(loc);
        } else {
            code.reg_uses.insert(id, vec![loc]);
        } 
    }
    
    fn prepare_op(&self, op: &P<Value>, loc: usize) -> (String, MuID, ASMLocation) {
        let str = self.asm_reg_op(op);
        let len = str.len();
        (str, op.extract_ssa_id().unwrap(), ASMLocation::new(loc, len)) 
    }
    
    fn prepare_machine_reg(&self, op: &P<Value>) -> MuID {
        op.extract_ssa_id().unwrap()
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
    
    fn control_flow_analysis(&mut self) {
        // control flow analysis
        let n_insts = self.line();
        
        let code = self.cur_mut();
        code.preds = vec![vec![]; n_insts];
        code.succs = vec![vec![]; n_insts];
        
        for i in 0..n_insts {
            // determine predecessor - if cur is not block start, its predecessor is previous insts
            let is_block_start = code.idx_to_blk.get(&i);
            if is_block_start.is_none() {
                if i > 0 {
                    code.preds[i].push(i - 1);
                }
            } else {
                // if cur is a branch target, we already set its predecessor
                // if cur is a fall-through block, we set it in a sanity check pass
            }
            
            // determine successor
            let is_branch = code.branches.get(&i);
            if is_branch.is_some() {
                // branch to target
                let target = is_branch.unwrap();
                let target_n = code.blk_to_idx.get(target).unwrap();
                
                // cur inst's succ is target
                code.succs[i].push(*target_n);
                
                // target's pred is cur
                code.preds[*target_n].push(i);
            } else {
                let is_cond_branch = code.cond_branches.get(&i);
                if is_cond_branch.is_some() {
                    // branch to target
                    let target = is_cond_branch.unwrap();
                    let target_n = code.blk_to_idx.get(target).unwrap();
                    
                    // cur insts' succ is target and next inst
                    code.succs[i].push(*target_n);
                    if i < n_insts - 1 {
                        code.succs[i].push(i + 1);
                    }
                    
                    // target's pred is cur
                    code.preds[*target_n].push(i);
                } else {
                    // not branch nor cond branch, succ is next inst
                    if i < n_insts - 1 {
                        code.succs[i].push(i + 1);
                    }
                }
            } 
        }
        
        // a sanity check for fallthrough blocks
        for i in 0..n_insts {
            if i != 0 && code.preds[i].len() == 0 {
                code.preds[i].push(i - 1);
            }
        }        
    }
}

impl CodeGenerator for ASMCodeGen {
    fn start_code(&mut self, func_name: MuTag) {
        self.cur = Some(Box::new(ASMCode {
                name: func_name,
                code: vec![],
                reg_defines: HashMap::new(),
                reg_uses: HashMap::new(),
                
                preds: vec![],
                succs: vec![],
                
                idx_to_blk: HashMap::new(),
                blk_to_idx: HashMap::new(),
                cond_branches: HashMap::new(),
                branches: HashMap::new()
            }));
        
        self.add_asm_symbolic(format!(".globl {}", func_name));
    }
    
    fn finish_code(&mut self) -> Box<MachineCode> {
        self.control_flow_analysis();
        self.cur.take().unwrap()
    }
    
    fn print_cur_code(&self) {
        println!("");
        
        if self.cur.is_some() {
            let code = self.cur.as_ref().unwrap();
            
            println!("code for {}: ", code.name);
            let n_insts = code.code.len();
            for i in 0..n_insts {
                let ref line = code.code[i];
                println!("#{}\t{}", i, line.code);
            }
        } else {
            println!("no current code");
        }
        
        println!("");
    }
    
    fn start_block(&mut self, block_name: MuTag) {
        let label = format!("{}:", self.asm_block_label(block_name));        
        self.add_asm_block_label(label, block_name);
    }
    
    fn emit_cmp_r64_r64(&mut self, op1: &P<Value>, op2: &P<Value>) {
        trace!("emit: cmp {} {}", op1, op2);
        
        let (reg1, id1, loc1) = self.prepare_op(op1, 4 + 1);
        let (reg2, id2, loc2) = self.prepare_op(op2, 4 + 1 + reg1.len() + 1);
        
        let asm = format!("cmpq {} {}", reg1, reg2);
        
        self.add_asm_inst(
            asm,
            vec![],
            vec![],
            vec![id1, id2],
            vec![loc1, loc2]
        );
    }
    
    fn emit_cmp_r64_imm32(&mut self, op1: &P<Value>, op2: u32) {
        trace!("emit: cmp {} {}", op1, op2);
        
        let (reg1, id1, loc1) = self.prepare_op(op1, 4 + 1);
        
        let asm = format!("cmpq {} ${}", reg1, op2);
        
        self.add_asm_inst(
            asm,
            vec![],
            vec![],
            vec![id1],
            vec![loc1]
        )
    }
    
    fn emit_cmp_r64_mem64(&mut self, op1: &P<Value>, op2: &P<Value>) {
        trace!("emit: cmp {} {}", op1, op2);
        unimplemented!()
    }
    
    fn emit_mov_r64_imm32(&mut self, dest: &P<Value>, src: u32) {
        trace!("emit: mov {} -> {}", src, dest);
        
        let (reg1, id1, loc1) = self.prepare_op(dest, 4 + 1);
        
        let asm = format!("movq {} ${}", src, reg1);
        
        self.add_asm_inst(
            asm,
            vec![],
            vec![],
            vec![id1],
            vec![loc1]
        )
    }
    
    fn emit_mov_r64_mem64(&mut self, dest: &P<Value>, src: &P<Value>) {
        trace!("emit: mov {} -> {}", src, dest);
        unimplemented!()
    }
    
    fn emit_mov_r64_r64(&mut self, dest: &P<Value>, src: &P<Value>) {
        trace!("emit: mov {} -> {}", src, dest);
        
        let (reg1, id1, loc1) = self.prepare_op(dest, 4 + 1);
        let (reg2, id2, loc2) = self.prepare_op(src, 4 + 1 + reg1.len() + 1);
        
        let asm = format!("movq {} {}", reg2, reg1);
        
        self.add_asm_inst(
            asm,
            vec![id1],
            vec![loc1],
            vec![id2],
            vec![loc2]
        )
    }
    
    fn emit_add_r64_r64(&mut self, dest: &P<Value>, src: &P<Value>) {
        trace!("emit: add {}, {} -> {}", dest, src, dest);
        
        let (reg1, id1, loc1) = self.prepare_op(dest, 4 + 1);
        let (reg2, id2, loc2) = self.prepare_op(src, 4 + 1 + reg1.len() + 1);
        
        let asm = format!("addq {} {}", reg2, reg1);
        
        self.add_asm_inst(
            asm,
            vec![id1],
            vec![loc1.clone()],
            vec![id1, id2],
            vec![loc1, loc2]
        )
    }
    
    fn emit_add_r64_mem64(&mut self, dest: &P<Value>, src: &P<Value>) {
        trace!("emit: add {}, {} -> {}", dest, src, dest);
        unimplemented!()
    }
    
    fn emit_add_r64_imm32(&mut self, dest: &P<Value>, src: u32) {
        trace!("emit: add {}, {} -> {}", dest, src, dest);
        
        let (reg1, id1, loc1) = self.prepare_op(dest, 4 + 1);
        
        let asm = format!("addq {} ${}", src, reg1);
        
        self.add_asm_inst(
            asm,
            vec![id1],
            vec![loc1.clone()],
            vec![id1],
            vec![loc1]
        )
    }
    
    fn emit_sub_r64_r64(&mut self, dest: &P<Value>, src: &P<Value>) {
        trace!("emit: sub {}, {} -> {}", dest, src, dest);
        
        let (reg1, id1, loc1) = self.prepare_op(dest, 4 + 1);
        let (reg2, id2, loc2) = self.prepare_op(src, 4 + 1 + reg1.len() + 1);
        
        let asm = format!("subq {} {}", reg2, reg1);
        
        self.add_asm_inst(
            asm,
            vec![id1],
            vec![loc1.clone()],
            vec![id1, id2],
            vec![loc1, loc2]
        )        
    }
    
    fn emit_sub_r64_mem64(&mut self, dest: &P<Value>, src: &P<Value>) {
        trace!("emit: sub {}, {} -> {}", dest, src, dest);
        unimplemented!()
    }
    
    fn emit_sub_r64_imm32(&mut self, dest: &P<Value>, src: u32) {
        trace!("emit: sub {}, {} -> {}", dest, src, dest);
        
        let (reg1, id1, loc1) = self.prepare_op(dest, 4 + 1);
        
        let asm = format!("subq {} ${}", src, reg1);
        
        self.add_asm_inst(
            asm,
            vec![id1],
            vec![loc1.clone()],
            vec![id1],
            vec![loc1]
        )        
    }
    
    fn emit_mul_r64(&mut self, src: &P<Value>) {
        trace!("emit: mul rax, {} -> (rdx, rax)", src);
        
        let (reg, id, loc) = self.prepare_op(src, 3 + 1);
        let rax = self.prepare_machine_reg(&x86_64::RAX);
        let rdx = self.prepare_machine_reg(&x86_64::RDX);
        
        let asm = format!("mul {}", reg);
        
        self.add_asm_inst(
            asm,
            vec![rax, rdx],
            vec![],
            vec![id, rax],
            vec![loc]
        )
    }
    
    fn emit_mul_mem64(&mut self, src: &P<Value>) {
        trace!("emit: mul rax, {} -> rax", src);
        unimplemented!()
    }
    
    fn emit_jmp(&mut self, dest: &Destination) {
        trace!("emit: jmp {}", dest.target);
        
        // symbolic label, we dont need to patch it
        let asm = format!("jmp {}", self.asm_block_label(dest.target));
        self.add_asm_branch(asm, dest.target)
    }
    
    fn emit_je(&mut self, dest: &Destination) {
        trace!("emit: je {}", dest.target);
        
        let asm = format!("je {}", self.asm_block_label(dest.target));
        self.add_asm_branch2(asm, dest.target);        
    }
    
    fn emit_jne(&mut self, dest: &Destination) {
        trace!("emit: jne {}", dest.target);
        
        let asm = format!("jne {}", self.asm_block_label(dest.target));
        self.add_asm_branch2(asm, dest.target);
    }
    
    fn emit_ja(&mut self, dest: &Destination) {
        trace!("emit: ja {}", dest.target);
        
        let asm = format!("ja {}", self.asm_block_label(dest.target));
        self.add_asm_branch2(asm, dest.target);
    }
    
    fn emit_jae(&mut self, dest: &Destination) {
        trace!("emit: jae {}", dest.target);
        
        let asm = format!("jae {}", self.asm_block_label(dest.target));
        self.add_asm_branch2(asm, dest.target);        
    }
    
    fn emit_jb(&mut self, dest: &Destination) {
        trace!("emit: jb {}", dest.target);
        
        let asm = format!("jb {}", self.asm_block_label(dest.target));
        self.add_asm_branch2(asm, dest.target);
    }
    
    fn emit_jbe(&mut self, dest: &Destination) {
        trace!("emit: jbe {}", dest.target);
        
        let asm = format!("jbe {}", self.asm_block_label(dest.target));
        self.add_asm_branch2(asm, dest.target);        
    }
    
    fn emit_jg(&mut self, dest: &Destination) {
        trace!("emit: jg {}", dest.target);
        
        let asm = format!("jg {}", self.asm_block_label(dest.target));
        self.add_asm_branch2(asm, dest.target);        
    }
    
    fn emit_jge(&mut self, dest: &Destination) {
        trace!("emit: jge {}", dest.target);
        
        let asm = format!("jge {}", self.asm_block_label(dest.target));
        self.add_asm_branch2(asm, dest.target);        
    }
    
    fn emit_jl(&mut self, dest: &Destination) {
        trace!("emit: jl {}", dest.target);
        
        let asm = format!("jl {}", self.asm_block_label(dest.target));
        self.add_asm_branch2(asm, dest.target);        
    }
    
    fn emit_jle(&mut self, dest: &Destination) {
        trace!("emit: jle {}", dest.target);
        
        let asm = format!("jle {}", self.asm_block_label(dest.target));
        self.add_asm_branch2(asm, dest.target);        
    }    
    
    fn emit_call_near_rel32(&mut self, func: MuTag) {
        trace!("emit: call {}", func);
        
        let asm = format!("call {}", func);
        self.add_asm_call(asm);
        
        // FIXME: call interferes with machine registers
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
        self.add_asm_ret(asm);
    }
    
    fn emit_push_r64(&mut self, src: &P<Value>) {
        trace!("emit: push {}", src);
        
        let (reg, id, loc) = self.prepare_op(src, 5 + 1);
        
        let asm = format!("pushq {}", reg);
        
        self.add_asm_inst(
            asm,
            vec![],
            vec![],
            vec![id],
            vec![loc]
        )
    }
    
    fn emit_pop_r64(&mut self, dest: &P<Value>) {
        trace!("emit: pop {}", dest);
        
        let (reg, id, loc) = self.prepare_op(dest, 4 + 1);
        
        let asm = format!("popq {}", reg);
        
        self.add_asm_inst(
            asm,
            vec![id],
            vec![loc.clone()],
            vec![id],
            vec![loc]
        )        
    }    
}