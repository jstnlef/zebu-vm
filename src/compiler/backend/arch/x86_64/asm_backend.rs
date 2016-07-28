#![allow(unused_variables)]

use compiler::backend;
use utils::ByteSize;
use compiler::backend::x86_64;
use compiler::backend::x86_64::CodeGenerator;
use vm::MachineCode;
use vm::VM;

use utils::string_utils;

use ast::ptr::P;
use ast::ir::*;
use ast::inst::*;

use std::collections::HashMap;
use std::str;
use std::usize;
use std::slice::Iter;
use std::ops;

struct ASMCode {
    name: MuName, 
    code: Vec<ASM>,
    reg_defines: HashMap<MuID, Vec<ASMLocation>>,
    reg_uses: HashMap<MuID, Vec<ASMLocation>>,
    
    mem_op_used: HashMap<usize, bool>,
    
    preds: Vec<Vec<usize>>,
    succs: Vec<Vec<usize>>,
    
    idx_to_blk: HashMap<usize, MuName>,
    blk_to_idx: HashMap<MuName, usize>,
    cond_branches: HashMap<usize, MuName>,
    branches: HashMap<usize, MuName>,
    
    blocks: Vec<MuName>,
    block_start: HashMap<MuName, usize>,
    block_range: HashMap<MuName, ops::Range<usize>>,
    
    block_livein: HashMap<MuName, Vec<MuID>>,
    block_liveout: HashMap<MuName, Vec<MuID>>
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
    
    fn is_using_mem_op(&self, index: usize) -> bool {
        *self.mem_op_used.get(&index).unwrap()
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
    
    fn replace_reg(&mut self, from: MuID, to: MuID) {
        let to_reg_tag : MuName = backend::all_regs()[to].name.unwrap();
        let to_reg_string = "%".to_string() + to_reg_tag;
        
        match self.reg_defines.get(&from) {
            Some(defines) => {
                for loc in defines {
                    let ref mut inst_to_patch = self.code[loc.line];
                    for i in 0..loc.len {
                        string_utils::replace(&mut inst_to_patch.code, loc.index, &to_reg_string, to_reg_string.len());
                    }
                }
            },
            None => {}
        }
        
        match self.reg_uses.get(&from) {
            Some(uses) => {
                for loc in uses {
                    let ref mut inst_to_patch = self.code[loc.line];
                    for i in 0..loc.len {
                        string_utils::replace(&mut inst_to_patch.code, loc.index, &to_reg_string, to_reg_string.len());
                    }   
                }
            },
            None => {}
        }
    }
    
    fn set_inst_nop(&mut self, index: usize) {
        self.code.remove(index);
        self.code.insert(index, ASM::nop());
    }
    
    fn emit(&self) -> Vec<u8> {
        let mut ret = vec![];
        
        for inst in self.code.iter() {
            ret.append(&mut inst.code.clone().into_bytes());
            ret.append(&mut "\n".to_string().into_bytes());
        }
        
        ret
    }
    
    fn trace_mc(&self) {
        trace!("");

        trace!("code for {}: \n", self.name);
        
        let n_insts = self.code.len();
        for i in 0..n_insts {
            self.trace_inst(i);
        }
        
        trace!("")      
    }
    
    fn trace_inst(&self, i: usize) {
        trace!("#{}\t{:30}\t\tdefine: {:?}\tuses: {:?}\tpred: {:?}\tsucc: {:?}", 
            i, self.code[i].code, self.get_inst_reg_defines(i), self.get_inst_reg_uses(i),
            self.preds[i], self.succs[i]);
    }
    
    fn get_ir_block_livein(&self, block: MuName) -> Option<&Vec<MuID>> {
        self.block_livein.get(&block)
    }
    
    fn get_ir_block_liveout(&self, block: MuName) -> Option<&Vec<MuID>> {
        self.block_liveout.get(&block)
    }
    
    fn get_all_blocks(&self) -> &Vec<MuName> {
        &self.blocks
    }
    
    fn get_block_range(&self, block: MuName) -> Option<ops::Range<usize>> {
        match self.block_range.get(&block) {
            Some(r) => Some(r.clone()),
            None => None
        }
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
    
    fn nop() -> ASM {
        ASM {
            code: "".to_string(),
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

const REG_PLACEHOLDER_LEN : usize = 5;
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
    
    fn add_asm_block_label(&mut self, code: String, block_name: &'static str) {
        let l = self.line();
        self.cur_mut().code.push(ASM::symbolic(code));
        
        self.cur_mut().idx_to_blk.insert(l, block_name);
        self.cur_mut().blk_to_idx.insert(block_name, l);
    }
    
    fn add_asm_symbolic(&mut self, code: String){
        self.cur_mut().code.push(ASM::symbolic(code));
    }
    
    fn prepare_machine_regs(&self, regs: Iter<P<Value>>) -> Vec<MuID> {
        regs.map(|x| self.prepare_machine_reg(x)).collect()
    } 
    
    fn add_asm_call(&mut self, code: String) {
        let mut uses : Vec<MuID> = self.prepare_machine_regs(x86_64::ARGUMENT_GPRs.iter());
        uses.append(&mut self.prepare_machine_regs(x86_64::ARGUMENT_FPRs.iter()));
        
        let mut defines : Vec<MuID> = self.prepare_machine_regs(x86_64::RETURN_GPRs.iter());
        defines.append(&mut self.prepare_machine_regs(x86_64::RETURN_FPRs.iter()));
          
        self.add_asm_inst(code, defines, vec![], uses, vec![], false);
    }
    
    fn add_asm_ret(&mut self, code: String) {
        let mut uses : Vec<MuID> = self.prepare_machine_regs(x86_64::RETURN_GPRs.iter());
        uses.append(&mut self.prepare_machine_regs(x86_64::RETURN_FPRs.iter()));
        
        self.add_asm_inst(code, vec![], vec![], uses, vec![], false);
    }
    
    fn add_asm_branch(&mut self, code: String, target: &'static str) {
        let l = self.line();
        self.cur_mut().branches.insert(l, target);
        
        self.add_asm_inst(code, vec![], vec![], vec![], vec![], false);
    }
    
    fn add_asm_branch2(&mut self, code: String, target: &'static str) {
        let l = self.line();
        self.cur_mut().cond_branches.insert(l, target);
        
        self.add_asm_inst(code, vec![], vec![], vec![], vec![], false);
    }
    
    fn add_asm_inst(
        &mut self, 
        code: String, 
        defines: Vec<MuID>,
        mut define_locs: Vec<ASMLocation>, 
        uses: Vec<MuID>,
        mut use_locs: Vec<ASMLocation>,
        is_using_mem_op: bool) 
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
                mc.reg_uses.get_mut(&id).unwrap().push(loc.clone());
            } else {
                mc.reg_uses.insert(id, vec![loc.clone()]);
            }
        }
       
        // put the instruction
        mc.code.push(ASM::inst(code, defines, uses));
        mc.mem_op_used.insert(line, is_using_mem_op);
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
    
    fn prepare_reg(&self, op: &P<Value>, loc: usize) -> (String, MuID, ASMLocation) {
        let str = self.asm_reg_op(op);
        let len = str.len();
        (str, op.extract_ssa_id().unwrap(), ASMLocation::new(loc, len)) 
    }
    
    fn prepare_machine_reg(&self, op: &P<Value>) -> MuID {
        op.extract_ssa_id().unwrap()
    }
    
    #[allow(unused_assignments)]
    fn prepare_mem(&self, op: &P<Value>, loc: usize) -> (String, Vec<MuID>, Vec<ASMLocation>) {
        let mut ids : Vec<MuID> = vec![];
        let mut locs : Vec<ASMLocation> = vec![];
        let mut result_str : String = "".to_string();
        
        let mut loc_cursor : usize = 0;
        
        match op.v {
            // offset(base,index,scale)
            Value_::Memory(MemoryLocation::Address{ref base, ref offset, ref index, scale}) => {
                // deal with offset
                if offset.is_some() {
                    let offset = offset.as_ref().unwrap();
                    
                    match offset.v {
                        Value_::SSAVar(id) => {
                            // temp as offset
                            let (str, id, loc) = self.prepare_reg(offset, 0);
                            
                            result_str.push_str(&str);
                            ids.push(id);
                            locs.push(loc);
                            
                            loc_cursor += str.len();
                        },
                        Value_::Constant(Constant::Int(val)) => {
                            let str = val.to_string();
                            
                            result_str.push_str(&str);
                            loc_cursor += str.len();
                        },
                        _ => panic!("unexpected offset type: {:?}", offset)
                    }
                }
                
                result_str.push('(');
                loc_cursor += 1; 
                
                // deal with base, base is ssa
                let (str, id, loc) = self.prepare_reg(base, loc_cursor);
                result_str.push_str(&str);
                ids.push(id);
                locs.push(loc);
                loc_cursor += str.len();
                
                // deal with index (ssa or constant)
                if index.is_some() {
                    result_str.push(',');
                    loc_cursor += 1; // plus 1 for ,                    
                    
                    let index = index.as_ref().unwrap();
                    
                    match index.v {
                        Value_::SSAVar(id) => {
                            // temp as offset
                            let (str, id, loc) = self.prepare_reg(index, loc_cursor);
                            
                            result_str.push_str(&str);
                            ids.push(id);
                            locs.push(loc);
                            
                            loc_cursor += str.len();
                        },
                        Value_::Constant(Constant::Int(val)) => {
                            let str = val.to_string();
                            
                            result_str.push_str(&str);
                            loc_cursor += str.len();
                        },
                        _ => panic!("unexpected index type: {:?}", index)
                    }
                    
                    // scale
                    if scale.is_some() {
                        result_str.push(',');
                        loc_cursor += 1;
                        
                        let scale = scale.unwrap();
                        let str = scale.to_string();
                        
                        result_str.push_str(&str);
                        loc_cursor += str.len();
                    }
                }
                
                result_str.push(')');
                loc_cursor += 1;
            },
            Value_::Memory(MemoryLocation::Symbolic{ref base, label}) => {
                result_str.push_str(&symbol(label));
                loc_cursor += label.len();
                
                if base.is_some() {
                    result_str.push('(');
                    loc_cursor += 1;
                    
                    let (str, id, loc) = self.prepare_reg(base.as_ref().unwrap(), loc_cursor);
                    result_str.push_str(&str);
                    ids.push(id);
                    locs.push(loc);
                    loc_cursor += str.len();
                    
                    result_str.push(')');
                    loc_cursor += 1;                    
                }
            },
            _ => panic!("expect mem location as value")
        }
        
        (result_str, ids, locs)
    }
    
    fn asm_reg_op(&self, op: &P<Value>) -> String {
        let id = op.extract_ssa_id().unwrap();
        if id < MACHINE_ID_END {
            // machine reg
            format!("%{}", op.name.unwrap())
        } else {
            // virtual register, use place holder
            REG_PLACEHOLDER.clone()
        }
    }
    
    fn asm_block_label(&self, label: MuName) -> String {
        symbol(&format!("{}_{}", self.cur().name, label))
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
    fn start_code(&mut self, func_name: MuName) {
        self.cur = Some(Box::new(ASMCode {
                name: func_name,
                code: vec![],
                reg_defines: HashMap::new(),
                reg_uses: HashMap::new(),
                
                mem_op_used: HashMap::new(),
                
                preds: vec![],
                succs: vec![],
                
                idx_to_blk: HashMap::new(),
                blk_to_idx: HashMap::new(),
                cond_branches: HashMap::new(),
                branches: HashMap::new(),
                
                blocks: vec![],
                block_start: HashMap::new(),
                block_range: HashMap::new(),
                
                block_livein: HashMap::new(),
                block_liveout: HashMap::new()
            }));
        
        // to link with C sources via gcc
        self.add_asm_symbolic(directive_globl(symbol(func_name)));
        self.add_asm_symbolic(format!("{}:", symbol(func_name)));
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
    
    fn start_block(&mut self, block_name: MuName) {
        let label = format!("{}:", self.asm_block_label(block_name));        
        self.add_asm_block_label(label, block_name);
        self.cur_mut().blocks.push(block_name);
        
        let start = self.line();
        self.cur_mut().block_start.insert(block_name, start);
    }
    
    fn end_block(&mut self, block_name: MuName) {
        let start : usize = *self.cur().block_start.get(&block_name).unwrap();
        let end : usize = self.line();
        
        self.cur_mut().block_range.insert(block_name, (start..end));
    }
    
    fn set_block_livein(&mut self, block_name: MuName, live_in: &Vec<P<Value>>) {
        let cur = self.cur_mut();
        
        let mut res = {
            if !cur.block_livein.contains_key(&block_name) {
                cur.block_livein.insert(block_name, vec![]);
            } else {
                panic!("seems we are inserting livein to block {} twice", block_name);
            }
            
            cur.block_livein.get_mut(&block_name).unwrap()
        };
        
        for value in live_in {
            res.push(value.extract_ssa_id().unwrap());
        }
    }
    
    fn set_block_liveout(&mut self, block_name: MuName, live_out: &Vec<P<Value>>) {
        let cur = self.cur_mut();
        
        let mut res = {
            if !cur.block_liveout.contains_key(&block_name) {
                cur.block_liveout.insert(block_name, vec![]);
            } else {
                panic!("seems we are inserting livein to block {} twice", block_name);
            }
            
            cur.block_liveout.get_mut(&block_name).unwrap()
        };
        
        for value in live_out {
            match value.extract_ssa_id() {
                Some(id) => res.push(id),
                None => {}
            }
        }        
    }
    
    fn emit_cmp_r64_r64(&mut self, op1: &P<Value>, op2: &P<Value>) {
        trace!("emit: cmp {} {}", op1, op2);
        
        let (reg1, id1, loc1) = self.prepare_reg(op1, 4 + 1);
        let (reg2, id2, loc2) = self.prepare_reg(op2, 4 + 1 + reg1.len() + 1);
        
        let asm = format!("cmpq {},{}", reg1, reg2);
        
        self.add_asm_inst(
            asm,
            vec![],
            vec![],
            vec![id1, id2],
            vec![loc1, loc2],
            false
        );
    }
    
    fn emit_cmp_r64_imm32(&mut self, op1: &P<Value>, op2: u32) {
        trace!("emit: cmp {} {}", op1, op2);
        
        let (reg1, id1, loc1) = self.prepare_reg(op1, 4 + 1 + 1 + op2.to_string().len() + 1);
        
        let asm = format!("cmpq ${},{}", op2, reg1);
        
        self.add_asm_inst(
            asm,
            vec![],
            vec![],
            vec![id1],
            vec![loc1],
            false
        )
    }
    
    fn emit_cmp_r64_mem64(&mut self, op1: &P<Value>, op2: &P<Value>) {
        trace!("emit: cmp {} {}", op1, op2);
        unimplemented!()
    }
    
    fn emit_mov_r64_imm32(&mut self, dest: &P<Value>, src: u32) {
        trace!("emit: mov {} -> {}", src, dest);
        
        let (reg1, id1, loc1) = self.prepare_reg(dest, 4 + 1 + 1 + src.to_string().len() + 1);
        
        let asm = format!("movq ${},{}", src, reg1);
        
        self.add_asm_inst(
            asm,
            vec![id1],
            vec![loc1],
            vec![],
            vec![],
            false
        )
    }
    
    fn emit_mov_r64_mem64(&mut self, dest: &P<Value>, src: &P<Value>) {
        trace!("emit: mov {} -> {}", src, dest);
        
        let (mem, id1, loc1) = self.prepare_mem(src, 4 + 1);
        let (reg, id2, loc2) = self.prepare_reg(dest, 4 + 1 + mem.len() + 1);
        
        let asm = format!("movq {},{}", mem, reg);
        
        self.add_asm_inst(
            asm,
            vec![id2],
            vec![loc2],
            id1,
            loc1,
            true
        )
    }
    
    fn emit_mov_mem64_r64(&mut self, dest: &P<Value>, src: &P<Value>) {
        trace!("emit: mov {} -> {}", src, dest);
        
        let (reg, id1, loc1) = self.prepare_reg(src, 4 + 1);
        let (mem, mut id2, mut loc2) = self.prepare_mem(dest, 4 + 1 + reg.len() + 1);
        
        // the register we used for the memory location is counted as 'use'
        id2.push(id1);
        loc2.push(loc1);
        
        let asm = format!("movq {},{}", reg, mem);
        
        self.add_asm_inst(
            asm,
            vec![], // not defining anything (write to memory)
            vec![],
            id2,
            loc2,
            true
        )
    }
    
    fn emit_mov_mem64_imm32(&mut self, dest: &P<Value>, src: u32) {
        trace!("emit: mov {} -> {}", src, dest);
        
        let (mem, id, loc) = self.prepare_mem(dest, 4 + 1 + 1 + src.to_string().len() + 1);
        
        let asm = format!("movq ${},{}", src, mem);
        
        self.add_asm_inst(
            asm,
            vec![],
            vec![],
            id,
            loc,
            true
        )
    }
    
    fn emit_mov_r64_r64(&mut self, dest: &P<Value>, src: &P<Value>) {
        trace!("emit: mov {} -> {}", src, dest);
        
        let (reg1, id1, loc1) = self.prepare_reg(src, 4 + 1);
        let (reg2, id2, loc2) = self.prepare_reg(dest, 4 + 1 + reg1.len() + 1);
        
        let asm = format!("movq {},{}", reg1, reg2);
        
        self.add_asm_inst(
            asm,
            vec![id2],
            vec![loc2],
            vec![id1],
            vec![loc1],
            false
        )
    }
    
    fn emit_add_r64_r64(&mut self, dest: &P<Value>, src: &P<Value>) {
        trace!("emit: add {}, {} -> {}", dest, src, dest);
        
        let (reg1, id1, loc1) = self.prepare_reg(src, 4 + 1);
        let (reg2, id2, loc2) = self.prepare_reg(dest, 4 + 1 + reg1.len() + 1);
        
        let asm = format!("addq {},{}", reg1, reg2);
        
        self.add_asm_inst(
            asm,
            vec![id2],
            vec![loc2.clone()],
            vec![id1, id2],
            vec![loc1, loc2],
            false
        )
    }
    
    fn emit_add_r64_mem64(&mut self, dest: &P<Value>, src: &P<Value>) {
        trace!("emit: add {}, {} -> {}", dest, src, dest);
        unimplemented!()
    }
    
    fn emit_add_r64_imm32(&mut self, dest: &P<Value>, src: u32) {
        trace!("emit: add {}, {} -> {}", dest, src, dest);
        
        let (reg1, id1, loc1) = self.prepare_reg(dest, 4 + 1);
        
        let asm = format!("addq {},${}", src, reg1);
        
        self.add_asm_inst(
            asm,
            vec![id1],
            vec![loc1.clone()],
            vec![id1],
            vec![loc1],
            false
        )
    }
    
    fn emit_sub_r64_r64(&mut self, dest: &P<Value>, src: &P<Value>) {
        trace!("emit: sub {}, {} -> {}", dest, src, dest);
        
        let (reg1, id1, loc1) = self.prepare_reg(src, 4 + 1);
        let (reg2, id2, loc2) = self.prepare_reg(dest, 4 + 1 + reg1.len() + 1);
        
        let asm = format!("subq {},{}", reg1, reg2);
        
        self.add_asm_inst(
            asm,
            vec![id2],
            vec![loc2.clone()],
            vec![id1, id2],
            vec![loc1, loc2],
            false
        )        
    }
    
    fn emit_sub_r64_mem64(&mut self, dest: &P<Value>, src: &P<Value>) {
        trace!("emit: sub {}, {} -> {}", dest, src, dest);
        unimplemented!()
    }
    
    fn emit_sub_r64_imm32(&mut self, dest: &P<Value>, src: u32) {
        trace!("emit: sub {}, {} -> {}", dest, src, dest);
        
        let (reg1, id1, loc1) = self.prepare_reg(dest, 4 + 1 + 1 + src.to_string().len() + 1);
        
        let asm = format!("subq ${},{}", src, reg1);
        
        self.add_asm_inst(
            asm,
            vec![id1],
            vec![loc1.clone()],
            vec![id1],
            vec![loc1],
            false
        )        
    }
    
    fn emit_mul_r64(&mut self, src: &P<Value>) {
        trace!("emit: mul rax, {} -> (rdx, rax)", src);
        
        let (reg, id, loc) = self.prepare_reg(src, 3 + 1);
        let rax = self.prepare_machine_reg(&x86_64::RAX);
        let rdx = self.prepare_machine_reg(&x86_64::RDX);
        
        let asm = format!("mul {}", reg);
        
        self.add_asm_inst(
            asm,
            vec![rax, rdx],
            vec![],
            vec![id, rax],
            vec![loc],
            false
        )
    }
    
    fn emit_mul_mem64(&mut self, src: &P<Value>) {
        trace!("emit: mul rax, {} -> rax", src);
        unimplemented!()
    }
    
    fn emit_jmp(&mut self, dest: &Block) {
        let dest_name = dest.name.unwrap();
        trace!("emit: jmp {}", dest_name);
        
        // symbolic label, we dont need to patch it
        let asm = format!("jmp {}", self.asm_block_label(dest_name));
        self.add_asm_branch(asm, dest_name)
    }
    
    fn emit_je(&mut self, dest: &Block) {
        let dest_name = dest.name.unwrap();
        trace!("emit: je {}", dest_name);
        
        let asm = format!("je {}", self.asm_block_label(dest_name));
        self.add_asm_branch2(asm, dest_name);        
    }
    
    fn emit_jne(&mut self, dest: &Block) {
        let dest_name = dest.name.unwrap();
        trace!("emit: jne {}", dest_name);
        
        let asm = format!("jne {}", self.asm_block_label(dest_name));
        self.add_asm_branch2(asm, dest_name);
    }
    
    fn emit_ja(&mut self, dest: &Block) {
        let dest_name = dest.name.unwrap();
        trace!("emit: ja {}", dest_name);
        
        let asm = format!("ja {}", self.asm_block_label(dest_name));
        self.add_asm_branch2(asm, dest_name);
    }
    
    fn emit_jae(&mut self, dest: &Block) {
        let dest_name = dest.name.unwrap();
        trace!("emit: jae {}", dest_name);
        
        let asm = format!("jae {}", self.asm_block_label(dest_name));
        self.add_asm_branch2(asm, dest_name);        
    }
    
    fn emit_jb(&mut self, dest: &Block) {
        let dest_name = dest.name.unwrap();
        trace!("emit: jb {}", dest_name);
        
        let asm = format!("jb {}", self.asm_block_label(dest_name));
        self.add_asm_branch2(asm, dest_name);
    }
    
    fn emit_jbe(&mut self, dest: &Block) {
        let dest_name = dest.name.unwrap();
        trace!("emit: jbe {}", dest_name);
        
        let asm = format!("jbe {}", self.asm_block_label(dest_name));
        self.add_asm_branch2(asm, dest_name);        
    }
    
    fn emit_jg(&mut self, dest: &Block) {
        let dest_name = dest.name.unwrap();
        trace!("emit: jg {}", dest_name);
        
        let asm = format!("jg {}", self.asm_block_label(dest_name));
        self.add_asm_branch2(asm, dest_name);        
    }
    
    fn emit_jge(&mut self, dest: &Block) {
        let dest_name = dest.name.unwrap();
        trace!("emit: jge {}", dest_name);
        
        let asm = format!("jge {}", self.asm_block_label(dest_name));
        self.add_asm_branch2(asm, dest_name);        
    }
    
    fn emit_jl(&mut self, dest: &Block) {
        let dest_name = dest.name.unwrap();
        trace!("emit: jl {}", dest_name);
        
        let asm = format!("jl {}", self.asm_block_label(dest_name));
        self.add_asm_branch2(asm, dest_name);        
    }
    
    fn emit_jle(&mut self, dest: &Block) {
        let dest_name = dest.name.unwrap();
        trace!("emit: jle {}", dest_name);
        
        let asm = format!("jle {}", self.asm_block_label(dest_name));
        self.add_asm_branch2(asm, dest_name);        
    }    
    
    fn emit_call_near_rel32(&mut self, func: MuName) {
        trace!("emit: call {}", func);
        
        let asm = format!("call {}", symbol(func));
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
        
        let (reg, id, loc) = self.prepare_reg(src, 5 + 1);
        let rsp = self.prepare_machine_reg(&x86_64::RSP);
        
        let asm = format!("pushq {}", reg);
        
        self.add_asm_inst(
            asm,
            vec![rsp],
            vec![],
            vec![id, rsp],
            vec![loc],
            false
        )
    }
    
    fn emit_pop_r64(&mut self, dest: &P<Value>) {
        trace!("emit: pop {}", dest);
        
        let (reg, id, loc) = self.prepare_reg(dest, 4 + 1);
        let rsp = self.prepare_machine_reg(&x86_64::RSP);
        
        let asm = format!("popq {}", reg);
        
        self.add_asm_inst(
            asm,
            vec![id, rsp],
            vec![loc.clone()],
            vec![rsp],
            vec![],
            false
        )        
    }    
}

const EMIT_DIR : &'static str = "emit";

fn create_emit_directory() {
    use std::fs;    
    match fs::create_dir(EMIT_DIR) {
        Ok(_) => {},
        Err(_) => {}
    }    
}

pub fn emit_code(func: &mut MuFunctionVersion, vm: &VM) {
    use std::io::prelude::*;
    use std::fs::File;
    use std::path;

    let compiled_funcs = vm.compiled_funcs().read().unwrap();
    let cf = compiled_funcs.get(&func.id).unwrap().borrow();

    let code = cf.mc.emit();

    // create 'emit' directory
    create_emit_directory();

    let mut file_path = path::PathBuf::new();
    file_path.push(EMIT_DIR);
    file_path.push(func.name.unwrap().to_string() + ".s");
    let mut file = match File::create(file_path.as_path()) {
        Err(why) => panic!("couldn't create emission file {}: {}", file_path.to_str().unwrap(), why),
        Ok(file) => file
    };

    match file.write_all(code.as_slice()) {
        Err(why) => panic!("couldn'd write to file {}: {}", file_path.to_str().unwrap(), why),
        Ok(_) => println!("emit code to {}", file_path.to_str().unwrap())
    }
}

const CONTEXT_FILE : &'static str = "context.s";
pub fn emit_context(vm: &VM) {
    use std::path;
    use std::fs::File;
    use std::io::prelude::*;
    
    debug!("---Emit VM Context---");
    create_emit_directory();
    
    let mut file_path = path::PathBuf::new();
    file_path.push(EMIT_DIR);
    file_path.push(CONTEXT_FILE);
    
    let mut file = match File::create(file_path.as_path()) {
        Err(why) => panic!("couldn't create context file {}: {}", file_path.to_str().unwrap(), why),
        Ok(file) => file
    };
    
    // put globals into bss section
    file.write_fmt(format_args!("\t.bss\n")).unwrap();
    
    let globals = vm.globals().read().unwrap();
    for cell in globals.values() {
        let (size, align) = {
            let ty_info = vm.get_backend_type_info(&cell.ty);
            (ty_info.size, ty_info.alignment)
        };
        
        file.write_fmt(format_args!("\t{}\n", directive_globl(symbol(cell.tag)))).unwrap();
        file.write_fmt(format_args!("\t{}\n", directive_comm(symbol(cell.tag), size, align))).unwrap();
        file.write("\n".as_bytes()).unwrap();
    }
    
    debug!("---finish---");
}

fn directive_globl(name: String) -> String {
    format!(".globl {}", name)
}

fn directive_comm(name: String, size: ByteSize, align: ByteSize) -> String {
    format!(".comm {},{},{}", name, size, align)
}

#[cfg(target_os = "linux")]
fn symbol(name: &str) -> String {
    name.to_string()
}

#[cfg(target_os = "macos")]
fn symbol(name: &str) -> String {
    format!("_{}", name)
}
