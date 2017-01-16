#![allow(unused_variables)]

use compiler::backend::AOT_EMIT_CONTEXT_FILE;
use compiler::backend::RegGroup;
use utils::ByteSize;
use utils::Address;
use utils::POINTER_SIZE;
use compiler::backend::x86_64;
use compiler::backend::x86_64::CodeGenerator;
use compiler::backend::{Reg, Mem};
use compiler::backend::x86_64::check_op_len;
use compiler::machine_code::MachineCode;
use vm::VM;
use runtime::ValueLocation;

use utils::vec_utils;
use utils::string_utils;
use utils::LinkedHashMap;

use ast::ptr::P;
use ast::ir::*;
use ast::types::*;

use std::collections::HashMap;
use std::str;
use std::usize;
use std::slice::Iter;
use std::ops;

struct ASMCode {
    name: MuName, 
    code: Vec<ASMInst>,

    blocks: LinkedHashMap<MuName, ASMBlock>,

    frame_size_patchpoints: Vec<ASMLocation>
}

unsafe impl Send for ASMCode {} 
unsafe impl Sync for ASMCode {}

impl ASMCode {
    fn get_use_locations(&self, reg: MuID) -> Vec<ASMLocation> {
        let mut ret = vec![];

        for inst in self.code.iter() {
            match inst.uses.get(&reg) {
                Some(ref locs) => {
                    ret.append(&mut locs.to_vec());
                },
                None => {}
            }
        }

        ret
    }

    fn get_define_locations(&self, reg: MuID) -> Vec<ASMLocation> {
        let mut ret = vec![];

        for inst in self.code.iter() {
            match inst.defines.get(&reg) {
                Some(ref locs) => {
                    ret.append(&mut locs.to_vec());
                },
                None => {}
            }
        }

        ret
    }

    fn is_block_start(&self, inst: usize) -> bool {
        for block in self.blocks.values() {
            if block.start_inst == inst {
                return true;
            }
        }
        false
    }

    fn is_last_inst_in_block(&self, inst: usize) -> bool {
        for block in self.blocks.values() {
            if block.end_inst == inst + 1 {
                return true;
            }
        }
        false
    }

    fn get_block_by_inst(&self, inst: usize) -> (&String, &ASMBlock) {
        for (name, block) in self.blocks.iter() {
            if inst >= block.start_inst && inst < block.end_inst {
                return (name, block);
            }
        }

        panic!("didnt find any block for inst {}", inst)
    }

    fn get_block_by_start_inst(&self, inst: usize) -> Option<&ASMBlock> {
        for block in self.blocks.values() {
            if block.start_inst == inst {
                return Some(block);
            }
        }

        None
    }

    fn rewrite_insert(
        &self,
        insert_before: HashMap<usize, Vec<Box<ASMCode>>>,
        insert_after: HashMap<usize, Vec<Box<ASMCode>>>) -> Box<ASMCode>
    {
        trace!("insert spilling code");
        let mut ret = ASMCode {
            name: self.name.clone(),
            code: vec![],
            blocks: linked_hashmap!{},
            frame_size_patchpoints: vec![]
        };

        // iterate through old machine code
        let mut inst_offset = 0;    // how many instructions has been inserted
        let mut cur_block_start = usize::MAX;

        // inst N in old machine code is N' in new machine code
        // this map stores the relationship
        let mut location_map : HashMap<usize, usize> = HashMap::new();

        for i in 0..self.number_of_insts() {
            trace!("Inst{}", i);

            if self.is_block_start(i) {
                cur_block_start = i + inst_offset;
                trace!("  block start is shifted to {}", cur_block_start);
            }

            // insert code before this instruction
            if insert_before.contains_key(&i) {
                for insert in insert_before.get(&i).unwrap() {
                    ret.append_code_sequence_all(insert);
                    inst_offset += insert.number_of_insts();
                    trace!("  inserted {} insts before", insert.number_of_insts());
                }
            }

            // copy this instruction
            let mut inst = self.code[i].clone();

            // old ith inst is now the (i + inst_offset)th instruction
            location_map.insert(i, i + inst_offset);
            trace!("  Inst{} is now Inst{}", i, i + inst_offset);

            // this instruction has been offset by several instructions('inst_offset')
            // update its info
            // 1. fix defines and uses
            for locs in inst.defines.values_mut() {
                for loc in locs {
                    debug_assert!(loc.line == i);
                    loc.line += inst_offset;
                }
            }
            for locs in inst.uses.values_mut() {
                for loc in locs {
                    debug_assert!(loc.line == i);
                    loc.line += inst_offset;
                }
            }
            // 2. we need to delete existing preds/succs - CFA is required later
            inst.preds.clear();
            inst.succs.clear();
            // 3. add the inst
            ret.code.push(inst);


            // insert code after this instruction
            if insert_after.contains_key(&i) {
                for insert in insert_after.get(&i).unwrap() {
                    ret.append_code_sequence_all(insert);
                    inst_offset += insert.number_of_insts();
                    trace!("  inserted {} insts after", insert.number_of_insts());
                }
            }

            if self.is_last_inst_in_block(i) {
                let cur_block_end = i + 1 + inst_offset;

                // copy the block
                let (name, block) = self.get_block_by_inst(i);

                let new_block = ASMBlock{
                    start_inst: cur_block_start,
                    end_inst: cur_block_end,

                    livein: vec![],
                    liveout: vec![]
                };

                trace!("  old block: {:?}", block);
                trace!("  new block: {:?}", new_block);

                cur_block_start = usize::MAX;

                // add to the new code
                ret.blocks.insert(name.clone(), new_block);
            }
        }

        // fix patchpoint
        for patchpoint in self.frame_size_patchpoints.iter() {
            let new_patchpoint = ASMLocation {
                line: *location_map.get(&patchpoint.line).unwrap(),
                index: patchpoint.index,
                len: patchpoint.len,
                oplen: patchpoint.oplen
            };

            ret.frame_size_patchpoints.push(new_patchpoint);
        }

        ret.control_flow_analysis();

        Box::new(ret)
    }

    fn append_code_sequence(
        &mut self,
        another: &Box<ASMCode>,
        start_inst: usize,
        n_insts: usize)
    {
        let base_line = self.number_of_insts();

        for i in 0..n_insts {
            let cur_line_in_self = base_line + i;
            let cur_line_from_copy = start_inst + i;

            let mut inst = another.code[cur_line_from_copy].clone();

            // fix info
            for locs in inst.defines.values_mut() {
                for loc in locs {
                    debug_assert!(loc.line == i);
                    loc.line = cur_line_in_self;
                }
            }
            for locs in inst.uses.values_mut() {
                for loc in locs {
                    debug_assert!(loc.line == i);
                    loc.line = cur_line_in_self;
                }
            }
            // ignore preds/succs

            // add to self
            self.code.push(inst);
        }
    }

    fn append_code_sequence_all(&mut self, another: &Box<ASMCode>) {
        let n_insts = another.number_of_insts();
        self.append_code_sequence(another, 0, n_insts)
    }

    fn control_flow_analysis(&mut self) {
        const TRACE_CFA : bool = false;

        // control flow analysis
        let n_insts = self.number_of_insts();

        let ref blocks = self.blocks;
        let ref mut asm = self.code;

        let block_start = {
            let mut ret = vec![];
            for block in blocks.values() {
                ret.push(block.start_inst);
            }
            ret
        };

        for i in 0..n_insts {
            if TRACE_CFA {
                trace!("---inst {}---", i);
            }
            // determine predecessor - if cur is not block start, its predecessor is previous insts
            let is_block_start = block_start.contains(&i);
            if !is_block_start {
                if i > 0 {
                    if TRACE_CFA {
                        trace!("inst {}: not a block start", i);
                        trace!("inst {}: set PREDS as previous inst {}", i, i - 1);
                    }
                    asm[i].preds.push(i - 1);
                }
            } else {
                // if cur is a branch target, we already set its predecessor
                // if cur is a fall-through block, we set it in a sanity check pass
            }

            // determine successor
            let branch = asm[i].branch.clone();
            match branch {
                ASMBranchTarget::Unconditional(ref target) => {
                    // branch to target
                    let target_n = self.blocks.get(target).unwrap().start_inst;

                    // cur inst's succ is target
                    asm[i].succs.push(target_n);

                    // target's pred is cur
                    asm[target_n].preds.push(i);

                    if TRACE_CFA {
                        trace!("inst {}: is a branch to {}", i, target);
                        trace!("inst {}: branch target index is {}", i, target_n);
                        trace!("inst {}: set SUCCS as branch target {}", i, target_n);
                        trace!("inst {}: set PREDS as branch source {}", target_n, i);
                    }
                },
                ASMBranchTarget::Conditional(ref target) => {
                    // branch to target
                    let target_n = self.blocks.get(target).unwrap().start_inst;

                    // cur insts' succ is target and next inst
                    asm[i].succs.push(target_n);

                    if TRACE_CFA {
                        trace!("inst {}: is a cond branch to {}", i, target);
                        trace!("inst {}: branch target index is {}", i, target_n);
                        trace!("inst {}: set SUCCS as branch target {}", i, target_n);
                    }

                    if i < n_insts - 1 {
                        if TRACE_CFA {
                            trace!("inst {}: set SUCCS as next inst", i + 1);
                        }
                        asm[i].succs.push(i + 1);
                    }

                    // target's pred is cur
                    asm[target_n].preds.push(i);
                    if TRACE_CFA {
                        trace!("inst {}: set PREDS as {}", target_n, i);
                    }
                },
                ASMBranchTarget::Return => {
                    if TRACE_CFA {
                        trace!("inst {}: is a return", i);
                        trace!("inst {}: has no successor", i);
                    }
                }
                ASMBranchTarget::None => {
                    // not branch nor cond branch, succ is next inst
                    if TRACE_CFA {
                        trace!("inst {}: not a branch inst", i);
                    }
                    if i < n_insts - 1 {
                        if TRACE_CFA {
                            trace!("inst {}: set SUCCS as next inst {}", i, i + 1);
                        }
                        asm[i].succs.push(i + 1);
                    }
                }
            }
        }

        // a sanity check for fallthrough blocks
        for i in 0..n_insts {
            if i != 0 && asm[i].preds.len() == 0 {
                asm[i].preds.push(i - 1);
            }
        }
    }

    fn add_frame_size_patchpoint(&mut self, patchpoint: ASMLocation) {
        self.frame_size_patchpoints.push(patchpoint);
    }
}

use std::any::Any;

impl MachineCode for ASMCode {
    fn as_any(&self) -> &Any {
        self
    }
    fn number_of_insts(&self) -> usize {
        self.code.len()
    }
    
    fn is_move(&self, index: usize) -> bool {
        let inst = self.code.get(index);
        match inst {
            Some(inst) => {
                let ref inst = inst.code;

                if inst.starts_with("movsd") || inst.starts_with("movss") {
                    // floating point move
                    true
                } else if inst.starts_with("movs") || inst.starts_with("movz") {
                    // sign extend, zero extend
                    false
                } else if inst.starts_with("mov") {
                    // normal mov
                    true
                } else {
                    false
                }
            },
            None => false
        }
    }
    
    fn is_using_mem_op(&self, index: usize) -> bool {
        self.code[index].is_mem_op_used
    }

    fn is_jmp(&self, index: usize) -> Option<MuName> {
        let inst = self.code.get(index);
        match inst {
            Some(inst) if inst.code.starts_with("jmp") => {
                let split : Vec<&str> = inst.code.split(' ').collect();

                Some(String::from(split[1]))
            }
            _ => None
        }
    }

    fn is_label(&self, index: usize) -> Option<MuName> {
        let inst = self.code.get(index);
        match inst {
            Some(inst) if inst.code.ends_with(':') => {
                let split : Vec<&str> = inst.code.split(':').collect();

                Some(String::from(split[0]))
            }
            _ => None
        }
    }
    
    fn get_succs(&self, index: usize) -> &Vec<usize> {
        &self.code[index].succs
    }
    
    fn get_preds(&self, index: usize) -> &Vec<usize> {
        &self.code[index].preds
    }
    
    fn get_inst_reg_uses(&self, index: usize) -> Vec<MuID> {
        self.code[index].uses.keys().map(|x| *x).collect()
    }
    
    fn get_inst_reg_defines(&self, index: usize) -> Vec<MuID> {
        self.code[index].defines.keys().map(|x| *x).collect()
    }
    
    fn replace_reg(&mut self, from: MuID, to: MuID) {
        for loc in self.get_define_locations(from) {
            let ref mut inst_to_patch = self.code[loc.line];

            // pick the right reg based on length
            let to_reg = x86_64::get_alias_for_length(to, loc.oplen);
            let to_reg_tag = to_reg.name().unwrap();
            let to_reg_string = "%".to_string() + &to_reg_tag;

            string_utils::replace(&mut inst_to_patch.code, loc.index, &to_reg_string, to_reg_string.len());
        }

        for loc in self.get_use_locations(from) {
            let ref mut inst_to_patch = self.code[loc.line];

            // pick the right reg based on length
            let to_reg = x86_64::get_alias_for_length(to, loc.oplen);
            let to_reg_tag = to_reg.name().unwrap();
            let to_reg_string = "%".to_string() + &to_reg_tag;

            string_utils::replace(&mut inst_to_patch.code, loc.index, &to_reg_string, to_reg_string.len());
        }
    }

    fn replace_define_tmp_for_inst(&mut self, from: MuID, to: MuID, inst: usize) {
        let to_reg_string : MuName = REG_PLACEHOLDER.clone();

        let asm = &mut self.code[inst];
        // if this reg is defined, replace the define
        if asm.defines.contains_key(&from) {
            let define_locs = asm.defines.get(&from).unwrap().to_vec();
            // replace temps
            for loc in define_locs.iter() {
                string_utils::replace(&mut asm.code, loc.index, &to_reg_string, to_reg_string.len());
            }

            // remove old key, insert new one
            asm.defines.remove(&from);
            asm.defines.insert(to, define_locs);
        }
    }

    fn replace_use_tmp_for_inst(&mut self, from: MuID, to: MuID, inst: usize) {
        let to_reg_string : MuName = REG_PLACEHOLDER.clone();

        let asm = &mut self.code[inst];

        // if this reg is used, replace the use
        if asm.uses.contains_key(&from) {
            let use_locs = asm.uses.get(&from).unwrap().to_vec();
            // replace temps
            for loc in use_locs.iter() {
                string_utils::replace(&mut asm.code, loc.index, &to_reg_string, to_reg_string.len());
            }

            // remove old key, insert new one
            asm.uses.remove(&from);
            asm.uses.insert(to, use_locs);
        }
    }
    
    fn set_inst_nop(&mut self, index: usize) {
        self.code.remove(index);
        self.code.insert(index, ASMInst::nop());
    }

    fn remove_unnecessary_callee_saved(&mut self, used_callee_saved: Vec<MuID>) -> Vec<MuID> {
        // we always save rbp
        let rbp = x86_64::RBP.extract_ssa_id().unwrap();
        // every push/pop will use/define rsp
        let rsp = x86_64::RSP.extract_ssa_id().unwrap();

        let find_op_other_than_rsp = |inst: &ASMInst| -> Option<MuID> {
            for id in inst.defines.keys() {
                if *id != rsp && *id != rbp {
                    return Some(*id);
                }
            }
            for id in inst.uses.keys() {
                if *id != rsp && *id != rbp {
                    return Some(*id);
                }
            }

            None
        };

        let mut inst_to_remove = vec![];
        let mut regs_to_remove = vec![];

        for i in 0..self.number_of_insts() {
            let ref inst = self.code[i];

            if inst.code.contains("push") || inst.code.contains("pop") {
                match find_op_other_than_rsp(inst) {
                    Some(op) => {
                        // if this push/pop instruction is about a callee saved register
                        // and the register is not used, we set the instruction as nop
                        if x86_64::is_callee_saved(op) && !used_callee_saved.contains(&op) {
                            trace!("removing instruction {:?} for save/restore unnecessary callee saved regs", inst);
                            regs_to_remove.push(op);
                            inst_to_remove.push(i);
                        }
                    }
                    None => {}
                }
            }
        }

        for i in inst_to_remove {
            self.set_inst_nop(i);
        }

        regs_to_remove
    }

    fn patch_frame_size(&mut self, size: usize) {
        let size = size.to_string();

        debug_assert!(size.len() <= FRAME_SIZE_PLACEHOLDER_LEN);

        for loc in self.frame_size_patchpoints.iter() {
            let ref mut inst = self.code[loc.line];

            string_utils::replace(&mut inst.code, loc.index, &size, size.len());
        }
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
            self.code[i].preds, self.code[i].succs);
    }
    
    fn get_ir_block_livein(&self, block: &str) -> Option<&Vec<MuID>> {
        match self.blocks.get(block) {
            Some(ref block) => Some(&block.livein),
            None => None
        }
    }
    
    fn get_ir_block_liveout(&self, block: &str) -> Option<&Vec<MuID>> {
        match self.blocks.get(block) {
            Some(ref block) => Some(&block.liveout),
            None => None
        }
    }
    
    fn set_ir_block_livein(&mut self, block: &str, set: Vec<MuID>) {
        let block = self.blocks.get_mut(block).unwrap();
        block.livein = set;
    }
    
    fn set_ir_block_liveout(&mut self, block: &str, set: Vec<MuID>) {
        let block = self.blocks.get_mut(block).unwrap();
        block.liveout = set;
    }
    
    fn get_all_blocks(&self) -> Vec<MuName> {
        self.blocks.keys().map(|x| x.clone()).collect()
    }
    
    fn get_block_range(&self, block: &str) -> Option<ops::Range<usize>> {
        match self.blocks.get(block) {
            Some(ref block) => Some(block.start_inst..block.end_inst),
            None => None
        }
    }
}

#[derive(Clone, Debug)]
enum ASMBranchTarget {
    None,
    Conditional(MuName),
    Unconditional(MuName),
    Return
}

#[derive(Clone, Debug)]
struct ASMInst {
    code: String,

    defines: LinkedHashMap<MuID, Vec<ASMLocation>>,
    uses: LinkedHashMap<MuID, Vec<ASMLocation>>,

    is_mem_op_used: bool,
    preds: Vec<usize>,
    succs: Vec<usize>,
    branch: ASMBranchTarget
}

impl ASMInst {
    fn symbolic(line: String) -> ASMInst {
        ASMInst {
            code: line,
            defines: LinkedHashMap::new(),
            uses: LinkedHashMap::new(),
            is_mem_op_used: false,
            preds: vec![],
            succs: vec![],
            branch: ASMBranchTarget::None
        }
    }
    
    fn inst(
        inst: String,
        defines: LinkedHashMap<MuID, Vec<ASMLocation>>,
        uses: LinkedHashMap<MuID, Vec<ASMLocation>>,
        is_mem_op_used: bool,
        target: ASMBranchTarget
    ) -> ASMInst
    {
        ASMInst {
            code: inst,
            defines: defines,
            uses: uses,
            is_mem_op_used: is_mem_op_used,
            preds: vec![],
            succs: vec![],
            branch: target
        }
    }
    
    fn nop() -> ASMInst {
        ASMInst {
            code: "".to_string(),
            defines: LinkedHashMap::new(),
            uses: LinkedHashMap::new(),
            is_mem_op_used: false,
            preds: vec![],
            succs: vec![],
            branch: ASMBranchTarget::None
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ASMLocation {
    line: usize,
    index: usize,
    len: usize,
    oplen: usize,
}

impl ASMLocation {
    fn new(line: usize, index: usize, len: usize, oplen: usize) -> ASMLocation {
        ASMLocation{
            line: line,
            index: index,
            len: len,
            oplen: oplen
        }
    }
}

#[derive(Clone, Debug)]
/// [start_inst, end_inst)
struct ASMBlock {
    start_inst: usize,
    end_inst: usize,

    livein: Vec<MuID>,
    liveout: Vec<MuID>
}

impl ASMBlock {
    fn new() -> ASMBlock {
        ASMBlock {
            start_inst: usize::MAX,
            end_inst: usize::MAX,
            livein: vec![],
            liveout: vec![]
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

const FRAME_SIZE_PLACEHOLDER_LEN : usize = 10; // a frame is smaller than 1 << 10
lazy_static! {
    pub static ref FRAME_SIZE_PLACEHOLDER : String = {
        let blank_spaces = [' ' as u8; FRAME_SIZE_PLACEHOLDER_LEN];
        format!("{}", str::from_utf8(&blank_spaces).unwrap())
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
    
    fn add_asm_label(&mut self, code: String) {
        let l = self.line();
        self.cur_mut().code.push(ASMInst::symbolic(code));
    }
    
    fn add_asm_block_label(&mut self, code: String, block_name: MuName) {
        let l = self.line();
        self.cur_mut().code.push(ASMInst::symbolic(code));
    }
    
    fn add_asm_symbolic(&mut self, code: String){
        self.cur_mut().code.push(ASMInst::symbolic(code));
    }
    
    fn prepare_machine_regs(&self, regs: Iter<P<Value>>) -> Vec<MuID> {
        regs.map(|x| self.prepare_machine_reg(x)).collect()
    }
    
    fn add_asm_call(&mut self, code: String) {
        // a call instruction will use all the argument registers
        // do not need
        let uses : LinkedHashMap<MuID, Vec<ASMLocation>> = LinkedHashMap::new();
//        for reg in x86_64::ARGUMENT_GPRs.iter() {
//            uses.insert(reg.id(), vec![]);
//        }
//        for reg in x86_64::ARGUMENT_FPRs.iter() {
//            uses.insert(reg.id(), vec![]);
//        }

        // defines: return registers
        let mut defines : LinkedHashMap<MuID, Vec<ASMLocation>> = LinkedHashMap::new();
        for reg in x86_64::RETURN_GPRs.iter() {
            defines.insert(reg.id(), vec![]);
        }
        for reg in x86_64::RETURN_FPRs.iter() {
            defines.insert(reg.id(), vec![]);
        }
        for reg in x86_64::CALLER_SAVED_GPRs.iter() {
            if !defines.contains_key(&reg.id()) {
                defines.insert(reg.id(), vec![]);
            }
        }
        for reg in x86_64::CALLER_SAVED_FPRs.iter() {
            if !defines.contains_key(&reg.id()) {
                defines.insert(reg.id(), vec![]);
            }
        }
          
        self.add_asm_inst(code, defines, uses, false);
    }
    
    fn add_asm_ret(&mut self, code: String) {
        // return instruction does not use anything (not RETURN REGS)
        // otherwise it will keep RETURN REGS alive
        // and if there is no actual move into RETURN REGS, it will keep RETURN REGS for alive for very long
        // and prevents anything using those regsiters
        self.add_asm_inst_internal(code, linked_hashmap!{}, linked_hashmap!{}, false, ASMBranchTarget::Return);
    }
    
    fn add_asm_branch(&mut self, code: String, target: MuName) {
        self.add_asm_inst_internal(code, linked_hashmap!{}, linked_hashmap!{}, false, ASMBranchTarget::Unconditional(target));
    }
    
    fn add_asm_branch2(&mut self, code: String, target: MuName) {
        self.add_asm_inst_internal(code, linked_hashmap!{}, linked_hashmap!{}, false, ASMBranchTarget::Conditional(target));
    }
    
    fn add_asm_inst(
        &mut self, 
        code: String, 
        defines: LinkedHashMap<MuID, Vec<ASMLocation>>,
        uses: LinkedHashMap<MuID, Vec<ASMLocation>>,
        is_using_mem_op: bool)
    {
        self.add_asm_inst_internal(code, defines, uses, is_using_mem_op, ASMBranchTarget::None)
    }

    fn add_asm_inst_internal(
        &mut self,
        code: String,
        defines: LinkedHashMap<MuID, Vec<ASMLocation>>,
        uses: LinkedHashMap<MuID, Vec<ASMLocation>>,
        is_using_mem_op: bool,
        target: ASMBranchTarget)
    {
        let line = self.line();
        trace!("asm: {}", code);
        trace!("     defines: {:?}", defines);
        trace!("     uses: {:?}", uses);
        let mc = self.cur_mut();

        // put the instruction
        mc.code.push(ASMInst::inst(code, defines, uses, is_using_mem_op, target));
    }
    
    fn prepare_reg(&self, op: &P<Value>, loc: usize) -> (String, MuID, ASMLocation) {
        if cfg!(debug_assertions) {
            match op.v {
                Value_::SSAVar(_) => {},
                _ => panic!("expecting register op")
            }
        }
        
        let str = self.asm_reg_op(op);
        let len = str.len();
        (str, op.extract_ssa_id().unwrap(), ASMLocation::new(self.line(), loc, len, check_op_len(op)))
    }

    fn prepare_fpreg(&self, op: &P<Value>, loc: usize) -> (String, MuID, ASMLocation) {
        if cfg!(debug_assertions) {
            match op.v {
                Value_::SSAVar(_) => {},
                _ => panic!("expecting register op")
            }
        }

        let str = self.asm_reg_op(op);
        let len = str.len();
        (str, op.extract_ssa_id().unwrap(), ASMLocation::new(self.line(), loc, len, 64))
    }
    
    fn prepare_machine_reg(&self, op: &P<Value>) -> MuID {
        if cfg!(debug_assertions) {
            match op.v {
                Value_::SSAVar(_) => {},
                _ => panic!("expecting machine register op")
            }
        }        
        
        op.extract_ssa_id().unwrap()
    }
    
    #[allow(unused_assignments)]
    fn prepare_mem(&self, op: &P<Value>, loc: usize) -> (String, LinkedHashMap<MuID, Vec<ASMLocation>>) {
        if cfg!(debug_assertions) {
            match op.v {
                Value_::Memory(_) => {},
                _ => panic!("expecting memory op")
            }
        }        

        let mut ids : Vec<MuID> = vec![];
        let mut locs : Vec<ASMLocation> = vec![];
        let mut result_str : String = "".to_string();
        
        let mut loc_cursor : usize = loc;
        
        match op.v {
            // offset(base,index,scale)
            Value_::Memory(MemoryLocation::Address{ref base, ref offset, ref index, scale}) => {
                // deal with offset
                if offset.is_some() {
                    let offset = offset.as_ref().unwrap();
                    
                    match offset.v {
                        Value_::SSAVar(id) => {
                            // temp as offset
                            let (str, id, loc) = self.prepare_reg(offset, loc_cursor);
                            
                            result_str.push_str(&str);
                            ids.push(id);
                            locs.push(loc);
                            
                            loc_cursor += str.len();
                        },
                        Value_::Constant(Constant::Int(val)) => {
                            let str = (val as i32).to_string();
                            
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
                            let str = (val as i32).to_string();
                            
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
            Value_::Memory(MemoryLocation::Symbolic{ref base, ref label}) => {
                if base.is_some() && base.as_ref().unwrap().id() == x86_64::RIP.id() {
                    // pc relative address
                    let pic_symbol = pic_symbol(label.clone());
//                    let pic_symbol = symbol(label.clone()); // not sure if we need this
                    result_str.push_str(&pic_symbol);
                    loc_cursor += label.len();
                } else {
                    let symbol = symbol(label.clone());
                    result_str.push_str(&symbol);
                    loc_cursor += label.len();
                }
                
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

        let uses : LinkedHashMap<MuID, Vec<ASMLocation>> = {
            let mut map : LinkedHashMap<MuID, Vec<ASMLocation>> = linked_hashmap!{};
            for i in 0..ids.len() {
                let id = ids[i];
                let loc = locs[i].clone();

                if map.contains_key(&id) {
                    map.get_mut(&id).unwrap().push(loc);
                } else {
                    map.insert(id, vec![loc]);
                }
            }
            map
        };


        (result_str, uses)
    }

    fn prepare_imm(&self, op: i32, len: usize) -> i32 {
        match len {
            64 => op,
            32 => op,
            16 => op as i16 as i32,
            8  => op as i8  as i32,
            _ => unimplemented!()
        }
    }
    
    fn asm_reg_op(&self, op: &P<Value>) -> String {
        let id = op.extract_ssa_id().unwrap();
        if id < MACHINE_ID_END {
            // machine reg
            format!("%{}", op.name().unwrap())
        } else {
            // virtual register, use place holder
            REG_PLACEHOLDER.clone()
        }
    }
    
    fn mangle_block_label(&self, label: MuName) -> String {
        format!("{}_{}", self.cur().name, label)
    }

    fn finish_code_sequence_asm(&mut self) -> Box<ASMCode> {
        self.cur.take().unwrap()
    }

    fn internal_binop_no_def_r_r(&mut self, inst: &str, op1: &P<Value>, op2: &P<Value>) {
        let len = check_op_len(op1);

        // with postfix
        let inst = inst.to_string() + &op_postfix(len);
        trace!("emit: {} {} {}", inst, op1, op2);

        let (reg1, id1, loc1) = self.prepare_reg(op1, inst.len() + 1);
        let (reg2, id2, loc2) = self.prepare_reg(op2, inst.len() + 1 + reg1.len() + 1);

        let asm = format!("{} {},{}", inst, reg1, reg2);

        self.add_asm_inst(
            asm,
            linked_hashmap!{},
            {
                if id1 == id2 {
                    linked_hashmap!{
                        id1 => vec![loc1, loc2]
                    }
                } else {
                    linked_hashmap!{
                        id1 => vec![loc1],
                        id2 => vec![loc2]
                    }
                }
            },
            false
        )
    }

    fn internal_binop_no_def_imm_r(&mut self, inst: &str, op1: i32, op2: &P<Value>) {
        let len = check_op_len(op2);

        let inst = inst.to_string() + &op_postfix(len);
        trace!("emit: {} {} {}", inst, op1, op2);

        let imm = self.prepare_imm(op1, len);
        let (reg2, id2, loc2) = self.prepare_reg(op2, inst.len() + 1 + 1 + imm.to_string().len() + 1);

        let asm = format!("{} ${},{}", inst, imm, reg2);

        self.add_asm_inst(
            asm,
            linked_hashmap!{},
            linked_hashmap!{
                id2 => vec![loc2]
            },
            false
        )
    }

    fn internal_binop_no_def_mem_r(&mut self, inst: &str, op1: &P<Value>, op2: &P<Value>) {
        let len = check_op_len(op2);

        let inst = inst.to_string() + &op_postfix(len);
        trace!("emit: {} {} {}", inst, op1, op2);

        let (mem, mut uses)  = self.prepare_mem(op1, inst.len() + 1);
        let (reg, id1, loc1) = self.prepare_reg(op2, inst.len() + 1 + mem.len() + 1);

        let asm = format!("{} {},{}", inst, mem, reg);

        // merge use vec
        if uses.contains_key(&id1) {
            let mut locs = uses.get_mut(&id1).unwrap();
            vec_utils::add_unique(locs, loc1.clone());
        } else {
            uses.insert(id1, vec![loc1]);
        }

        self.add_asm_inst(
            asm,
            linked_hashmap!{},
            uses,
            true
        )
    }

    fn internal_binop_no_def_r_mem(&mut self, inst: &str, op1: &P<Value>, op2: &P<Value>) {
        let len = check_op_len(op1);

        let inst = inst.to_string() + &op_postfix(len);
        trace!("emit: {} {} {}", inst, op1, op2);

        let (mem, mut uses) = self.prepare_mem(op2, inst.len() + 1);
        let (reg, id1, loc1) = self.prepare_reg(op1, inst.len() + 1 + mem.len() + 1);

        if uses.contains_key(&id1) {
            let mut locs = uses.get_mut(&id1).unwrap();
            vec_utils::add_unique(locs, loc1.clone());
        } else {
            uses.insert(id1, vec![loc1.clone()]);
        }

        let asm = format!("{} {},{}", inst, mem, reg);

        self.add_asm_inst(
            asm,
            linked_hashmap!{},
            uses,
            true
        )
    }

    fn internal_binop_def_r_r(&mut self, inst: &str, dest: &P<Value>, src: &P<Value>) {
        let len = check_op_len(src);

        let inst = inst.to_string() + &op_postfix(len);
        trace!("emit: {} {}, {} -> {}", inst, src, dest, dest);

        let (reg1, id1, loc1) = self.prepare_reg(src, inst.len() + 1);
        let (reg2, id2, loc2) = self.prepare_reg(dest, inst.len() + 1 + reg1.len() + 1);

        let asm = format!("{} {},{}", inst, reg1, reg2);

        self.add_asm_inst(
            asm,
            linked_hashmap!{
                id2 => vec![loc2.clone()]
            },
            {
                if id1 == id2 {
                    linked_hashmap!{
                        id1 => vec![loc1, loc2]
                    }
                } else {
                    linked_hashmap!{
                        id1 => vec![loc1],
                        id2 => vec![loc2]
                    }
                }
            },
            false
        )
    }

    fn internal_binop_def_r_mr(&mut self, inst: &str, dest: &P<Value>, src: &P<Value>) {
        let len = check_op_len(dest);

        let inst = inst.to_string() + &op_postfix(len);
        trace!("emit: {} {}, {} -> {}", inst, src, dest, dest);

        let mreg = self.prepare_machine_reg(src);
        let mreg_name = src.name().unwrap();
        let (reg2, id2, loc2) = self.prepare_reg(dest, inst.len() + 1 + 1 + mreg_name.len() + 1);

        let asm = format!("{} %{},{}", inst, mreg_name, reg2);

        self.add_asm_inst(
            asm,
            linked_hashmap!{
                id2 => vec![loc2.clone()]
            },
            linked_hashmap!{
                id2 => vec![loc2],
                mreg => vec![]
            },
            false
        )
    }

    fn internal_binop_def_r_imm(&mut self, inst: &str, dest: &P<Value>, src: i32) {
        let len = check_op_len(dest);

        let inst = inst.to_string() + &op_postfix(len);
        trace!("emit: {} {}, {} -> {}", inst, src, dest, dest);

        let imm = self.prepare_imm(src, len);
        let (reg1, id1, loc1) = self.prepare_reg(dest, inst.len() + 1 + 1 + imm.to_string().len() + 1);

        let asm = format!("{} ${},{}", inst, imm, reg1);

        self.add_asm_inst(
            asm,
            linked_hashmap!{
                id1 => vec![loc1.clone()]
            },
            linked_hashmap!{
                id1 => vec![loc1]
            },
            false
        )
    }

    fn internal_binop_def_r_mem(&mut self, inst: &str, dest: &P<Value>, src: &P<Value>) {
        let len = match dest.ty.get_int_length() {
            Some(n) if n == 64 | 32 | 16 | 8 => n,
            _ => panic!("unimplemented int types: {}", dest.ty)
        };

        let inst = inst.to_string() + &op_postfix(len);
        trace!("emit: {} {}, {} -> {}", inst, src, dest, dest);

        let (mem, mut uses) = self.prepare_mem(src, inst.len() + 1);
        let (reg, id1, loc1) = self.prepare_reg(dest, inst.len() + 1 + mem.len() + 1);

        if uses.contains_key(&id1) {
            let mut locs = uses.get_mut(&id1).unwrap();
            vec_utils::add_unique(locs, loc1.clone());
        } else {
            uses.insert(id1, vec![loc1.clone()]);
        }

        let asm = format!("{} {},{}", inst, mem, reg);

        self.add_asm_inst(
            asm,
            linked_hashmap!{
                id1 => vec![loc1]
            },
            uses,
            true
        )
    }

    fn internal_mov_r64_imm64(&mut self, inst: &str, dest: &P<Value>, src: i64) {
        let inst = inst.to_string() + &op_postfix(64);
        trace!("emit: {} {} -> {}", inst, src, dest);

        let (reg1, id1, loc1) = self.prepare_reg(dest, inst.len() + 1 + 1 + src.to_string().len() + 1);

        let asm = format!("{} ${},{}", inst, src, reg1);

        self.add_asm_inst(
            asm,
            linked_hashmap!{
                id1 => vec![loc1]
            },
            linked_hashmap!{},
            false
        )
    }

    fn internal_mov_r_r(&mut self, inst: &str, dest: &P<Value>, src: &P<Value>) {
        let len = check_op_len(dest);

        let inst = inst.to_string() + &op_postfix(len);
        trace!("emit: {} {} -> {}", inst, src, dest);

        let (reg1, id1, loc1) = self.prepare_reg(src, inst.len() + 1);
        let (reg2, id2, loc2) = self.prepare_reg(dest, inst.len() + 1 + reg1.len() + 1);

        let asm = format!("{} {},{}", inst, reg1, reg2);

        self.add_asm_inst(
            asm,
            linked_hashmap!{
                id2 => vec![loc2]
            },
            linked_hashmap!{
                id1 => vec![loc1]
            },
            false
        )
    }

    fn internal_mov_r_imm(&mut self, inst: &str, dest: &P<Value>, src: i32) {
        let len = check_op_len(dest);

        let inst = inst.to_string() + &op_postfix(len);
        trace!("emit: {} {} -> {}", inst, src, dest);

        let imm = self.prepare_imm(src, len);
        let (reg1, id1, loc1) = self.prepare_reg(dest, inst.len() + 1 + 1 + imm.to_string().len() + 1);

        let asm = format!("{} ${},{}", inst, imm, reg1);

        self.add_asm_inst(
            asm,
            linked_hashmap!{
                id1 => vec![loc1]
            },
            linked_hashmap!{},
            false
        )
    }

    fn internal_mov_r_mem(&mut self, inst: &str, dest: &P<Value>, src: &P<Value>) {
        let len = check_op_len(dest);

        let inst = inst.to_string() + &op_postfix(len);
        trace!("emit: {} {} -> {}", inst, src, dest);

        let (mem, uses) = self.prepare_mem(src, inst.len() + 1);
        let (reg, id2, loc2) = self.prepare_reg(dest, inst.len() + 1 + mem.len() + 1);

        let asm = format!("{} {},{}", inst, mem, reg);

        self.add_asm_inst(
            asm,
            linked_hashmap!{
                id2 => vec![loc2]
            },
            uses,
            true
        )
    }

    fn internal_mov_mem_r(&mut self, inst: &str, dest: &P<Value>, src: &P<Value>) {
        let len = check_op_len(src);

        let inst = inst.to_string() + &op_postfix(len);
        trace!("emit: {} {} -> {}", inst, src, dest);

        let (reg, id1, loc1) = self.prepare_reg(src, inst.len() + 1);
        let (mem, mut uses) = self.prepare_mem(dest, inst.len() + 1 + reg.len() + 1);

        // the register we used for the memory location is counted as 'use'
        // use the vec from mem as 'use' (push use reg from src to it)
        if uses.contains_key(&id1) {
            let mut locs = uses.get_mut(&id1).unwrap();
            vec_utils::add_unique(locs, loc1);
        } else {
            uses.insert(id1, vec![loc1]);
        }

        let asm = format!("{} {},{}", inst, reg, mem);

        self.add_asm_inst(
            asm,
            linked_hashmap!{},
            uses,
            true
        )
    }

    fn internal_mov_mem_imm(&mut self, inst: &str, dest: &P<Value>, src: i32) {
        let len = check_op_len(dest);

        let inst = inst.to_string() + &op_postfix(len);
        trace!("emit: {} {} -> {}", inst, src, dest);

        let imm = self.prepare_imm(src, len);
        let (mem, uses) = self.prepare_mem(dest, inst.len() + 1 + 1 + imm.to_string().len() + 1);

        let asm = format!("{} ${},{}", inst, imm, mem);

        self.add_asm_inst(
            asm,
            linked_hashmap!{},
            uses,
            true
        )
    }

    fn internal_fp_binop_no_def_r_r(&mut self, inst: &str, op1: &P<Value>, op2: &P<Value>) {
        trace!("emit: {} {} {}", inst, op1, op2);

        let (reg1, id1, loc1) = self.prepare_fpreg(op1, inst.len() + 1);
        let (reg2, id2, loc2) = self.prepare_fpreg(op2, inst.len() + 1 + reg1.len() + 1);

        let asm = format!("{} {},{}", inst, reg1, reg2);

        self.add_asm_inst(
            asm,
            linked_hashmap!{},
            {
                if id1 == id2 {
                    linked_hashmap!{
                        id1 => vec![loc1, loc2]
                    }
                } else {
                    linked_hashmap!{
                        id1 => vec![loc1],
                        id2 => vec![loc2]
                    }
                }
            },
            false
        )
    }

    fn internal_fp_binop_def_r_r(&mut self, inst: &str, dest: Reg, src: Reg) {
        trace!("emit: {} {}, {} -> {}", inst, src, dest, dest);

        let (reg1, id1, loc1) = self.prepare_fpreg(src, inst.len() + 1);
        let (reg2, id2, loc2) = self.prepare_fpreg(dest, inst.len() + 1 + reg1.len() + 1);

        let asm = format!("{} {},{}", inst, reg1, reg2);

        self.add_asm_inst(
            asm,
            linked_hashmap!{
                id2 => vec![loc2.clone()]
            },
            {
                if id1 == id2 {
                    linked_hashmap!{id1 => vec![loc1, loc2]}
                } else {
                    linked_hashmap! {
                        id1 => vec![loc1],
                        id2 => vec![loc2]
                    }
                }
            },
            false
        )
    }

    fn internal_fp_binop_def_r_mem(&mut self, inst: &str, dest: Reg, src: Reg) {
        trace!("emit: {} {}, {} -> {}", inst, src, dest, dest);
        unimplemented!()
    }
}

#[inline(always)]
fn op_postfix(op_len: usize) -> &'static str {
    match op_len {
        8  => "b",
        16 => "w",
        32 => "l",
        64 => "q",
        _  => panic!("unexpected op size: {}", op_len)
    }
}

impl CodeGenerator for ASMCodeGen {
    fn start_code(&mut self, func_name: MuName) -> ValueLocation {
        self.cur = Some(Box::new(ASMCode {
            name: func_name.clone(),
            code: vec![],
            blocks: linked_hashmap! {},
            frame_size_patchpoints: vec![]
        }));

        // to link with C sources via gcc
        let func_symbol = symbol(func_name.clone());
        self.add_asm_symbolic(directive_globl(func_symbol.clone()));
        self.add_asm_symbolic(format!("{}:", func_symbol.clone()));

        ValueLocation::Relocatable(RegGroup::GPR, func_name)
    }

    fn finish_code(&mut self, func_name: MuName) -> (Box<MachineCode + Sync + Send>, ValueLocation) {
        let func_end = {
            let mut symbol = func_name.clone();
            symbol.push_str("_end");
            symbol
        };
        self.add_asm_symbolic(directive_globl(symbol(func_end.clone())));
        self.add_asm_symbolic(format!("{}:", symbol(func_end.clone())));

        self.cur.as_mut().unwrap().control_flow_analysis();

        (
            self.cur.take().unwrap(),
            ValueLocation::Relocatable(RegGroup::GPR, func_end)
        )
    }

    fn start_code_sequence(&mut self) {
        self.cur = Some(Box::new(ASMCode {
            name: "snippet".to_string(),
            code: vec![],
            blocks: linked_hashmap! {},
            frame_size_patchpoints: vec![]
        }));
    }

    fn finish_code_sequence(&mut self) -> Box<MachineCode + Sync + Send> {
        self.finish_code_sequence_asm()
    }

    fn print_cur_code(&self) {
        debug!("");

        if self.cur.is_some() {
            let code = self.cur.as_ref().unwrap();

            debug!("code for {}: ", code.name);
            let n_insts = code.code.len();
            for i in 0..n_insts {
                let ref line = code.code[i];
                debug!("#{}\t{}", i, line.code);
            }
        } else {
            debug!("no current code");
        }

        debug!("");
    }

    fn start_block(&mut self, block_name: MuName) {
        let label = format!("{}:", symbol(self.mangle_block_label(block_name.clone())));
        self.add_asm_block_label(label, block_name.clone());

        self.cur_mut().blocks.insert(block_name.clone(), ASMBlock::new());
        let start = self.line();
        self.cur_mut().blocks.get_mut(&block_name).unwrap().start_inst = start;
    }

    fn start_exception_block(&mut self, block_name: MuName) -> ValueLocation {
        let mangled_name = self.mangle_block_label(block_name.clone());
        self.add_asm_symbolic(directive_globl(symbol(mangled_name.clone())));

        self.start_block(block_name.clone());

        ValueLocation::Relocatable(RegGroup::GPR, mangled_name)
    }

    fn end_block(&mut self, block_name: MuName) {
        let line = self.line();
        match self.cur_mut().blocks.get_mut(&block_name) {
            Some(ref mut block) => {
                block.end_inst = line;
            }
            None => panic!("trying to end block {} which hasnt been started", block_name)
        }
    }

    fn set_block_livein(&mut self, block_name: MuName, live_in: &Vec<P<Value>>) {
        let cur = self.cur_mut();

        match cur.blocks.get_mut(&block_name) {
            Some(ref mut block) => {
                if block.livein.is_empty() {
                    let mut live_in = {
                        let mut ret = vec![];
                        for p in live_in {
                            match p.extract_ssa_id() {
                                Some(id) => ret.push(id),
                                // this should not happen
                                None => error!("{} as live-in of block {} is not SSA", p, block_name)
                            }
                        }
                        ret
                    };
                    block.livein.append(&mut live_in);
                } else {
                    panic!("seems we are inserting livein to block {} twice", block_name);
                }
            }
            None => panic!("haven't created ASMBlock for {}", block_name)
        }
    }

    fn set_block_liveout(&mut self, block_name: MuName, live_out: &Vec<P<Value>>) {
        let cur = self.cur_mut();

        match cur.blocks.get_mut(&block_name) {
            Some(ref mut block) => {
                if block.liveout.is_empty() {
                    let mut live_out = {
                        let mut ret = vec![];
                        for p in live_out {
                            match p.extract_ssa_id() {
                                Some(id) => ret.push(id),
                                // the liveout are actually args out of this block
                                // (they can be constants)
                                None => trace!("{} as live-out of block {} is not SSA", p, block_name)
                            }
                        }
                        ret
                    };
                    block.liveout.append(&mut live_out);
                } else {
                    panic!("seems we are inserting liveout to block {} twice", block_name);
                }
            }
            None => panic!("haven't created ASMBlock for {}", block_name)
        }
    }

    fn emit_frame_grow(&mut self) {
        trace!("emit frame grow");

        let asm = format!("addq $-{},%rsp", FRAME_SIZE_PLACEHOLDER.clone());

        let line = self.line();
        self.cur_mut().add_frame_size_patchpoint(ASMLocation::new(line, 7, FRAME_SIZE_PLACEHOLDER_LEN, 0));

        self.add_asm_inst(
            asm,
            linked_hashmap!{}, // let reg alloc ignore this instruction
            linked_hashmap!{},
            false
        )
    }

    fn emit_frame_shrink(&mut self) {
        trace!("emit frame shrink");

        let asm = format!("addq ${},%rsp", FRAME_SIZE_PLACEHOLDER.clone());

        let line = self.line();
        self.cur_mut().add_frame_size_patchpoint(ASMLocation::new(line, 6, FRAME_SIZE_PLACEHOLDER_LEN, 0));

        self.add_asm_inst(
            asm,
            linked_hashmap!{},
            linked_hashmap!{},
            false
        )
    }

    fn emit_nop(&mut self, bytes: usize) {
        trace!("emit: nop ({} bytes)", bytes);

        let asm = String::from("nop");

        self.add_asm_inst(
            asm,
            linked_hashmap! {},
            linked_hashmap! {},
            false
        );
    }

    // cmp

    fn emit_cmp_r_r (&mut self, op1: &P<Value>, op2: &P<Value>) {
        self.internal_binop_no_def_r_r("cmp", op1, op2)
    }

    fn emit_cmp_imm_r(&mut self, op1: i32, op2: &P<Value>) {
        self.internal_binop_no_def_imm_r("cmp", op1, op2)
    }

    fn emit_cmp_mem_r(&mut self, op1: &P<Value>, op2: &P<Value>) {
        self.internal_binop_no_def_mem_r("cmp", op1, op2)
    }

    // mov

    fn emit_mov_r64_imm64  (&mut self, dest: &P<Value>, src: i64) {
        self.internal_mov_r64_imm64("mov", dest, src)
    }

    fn emit_mov_fpr_r64 (&mut self, dest: Reg, src: Reg) {
        trace!("emit: movq {} -> {}", src, dest);

        let (reg1, id1, loc1) = self.prepare_reg(src, 5);
        let (reg2, id2, loc2) = self.prepare_fpreg(dest, 5 + reg1.len() + 1);

        let asm = format!("movq {},{}", reg1, reg2);

        self.add_asm_inst(
            asm,
            linked_hashmap!{
                id2 => vec![loc2]
            },
            linked_hashmap!{
                id1 => vec![loc1]
            },
            false
        )
    }

    fn emit_mov_r_imm  (&mut self, dest: &P<Value>, src: i32) {
        self.internal_mov_r_imm("mov", dest, src)
    }
    fn emit_mov_r_mem  (&mut self, dest: &P<Value>, src: &P<Value>) {
        self.internal_mov_r_mem("mov", dest, src)
    }
    fn emit_mov_r_r    (&mut self, dest: &P<Value>, src: &P<Value>) {
        self.internal_mov_r_r("mov", dest, src)
    }
    fn emit_mov_mem_r  (&mut self, dest: &P<Value>, src: &P<Value>) {
        self.internal_mov_mem_r("mov", dest, src)
    }
    fn emit_mov_mem_imm(&mut self, dest: &P<Value>, src: i32) {
        self.internal_mov_mem_imm("mov", dest, src)
    }

    // zero/sign extend mov
    
    fn emit_movs_r_r (&mut self, dest: Reg, src: Reg) {
        let dest_len = check_op_len(dest);
        let src_len  = check_op_len(src);

        let inst = "movs".to_string() + &op_postfix(src_len) + &op_postfix(dest_len);
        trace!("emit: {} {} -> {}", inst, src, dest);

        let (reg1, id1, loc1) = self.prepare_reg(src, inst.len() + 1);
        let (reg2, id2, loc2) = self.prepare_reg(dest, inst.len() + 1 + reg1.len() + 1);

        let asm = format!("{} {},{}", inst, reg1, reg2);

        self.add_asm_inst(
            asm,
            linked_hashmap!{
                id2 => vec![loc2]
            },
            linked_hashmap!{
                id1 => vec![loc1]
            },
            false
        )
    }

    fn emit_movz_r_r (&mut self, dest: Reg, src: Reg) {
        let dest_len = check_op_len(dest);
        let src_len  = check_op_len(src);

        let inst = "movz".to_string() + &op_postfix(src_len) + &op_postfix(dest_len);
        trace!("emit: {} {} -> {}", inst, src, dest);

        let (reg1, id1, loc1) = self.prepare_reg(src, inst.len() + 1);
        let (reg2, id2, loc2) = self.prepare_reg(dest, inst.len() + 1 + reg1.len() + 1);

        let asm = format!("{} {},{}", inst, reg1, reg2);

        self.add_asm_inst(
            asm,
            linked_hashmap!{
                id2 => vec![loc2]
            },
            linked_hashmap!{
                id1 => vec![loc1]
            },
            false
        )
    }

    // cmov src -> dest
    // binop op1, op2 (op2 is destination)

    fn emit_cmova_r_r  (&mut self, dest: &P<Value>, src: &P<Value>) {
        debug_assert!(check_op_len(dest) >= 16);
        self.internal_binop_no_def_r_r("cmova", src, dest)
    }
    fn emit_cmova_r_mem(&mut self, dest: &P<Value>, src: &P<Value>) {
        debug_assert!(check_op_len(dest) >= 16);
        self.internal_binop_no_def_r_mem("cmova", src, dest)
    }

    fn emit_cmovae_r_r  (&mut self, dest: &P<Value>, src: &P<Value>) {
        debug_assert!(check_op_len(dest) >= 16);
        self.internal_binop_no_def_r_r("cmovae", src, dest)
    }
    fn emit_cmovae_r_mem(&mut self, dest: &P<Value>, src: &P<Value>) {
        debug_assert!(check_op_len(dest) >= 16);
        self.internal_binop_no_def_r_mem("cmovae", src, dest)
    }

    fn emit_cmovb_r_r  (&mut self, dest: &P<Value>, src: &P<Value>) {
        debug_assert!(check_op_len(dest) >= 16);
        self.internal_binop_no_def_r_r("cmovb", src, dest)
    }
    fn emit_cmovb_r_mem(&mut self, dest: &P<Value>, src: &P<Value>) {
        debug_assert!(check_op_len(dest) >= 16);
        self.internal_binop_no_def_r_mem("cmovb", src, dest)
    }

    fn emit_cmovbe_r_r  (&mut self, dest: &P<Value>, src: &P<Value>) {
        debug_assert!(check_op_len(dest) >= 16);
        self.internal_binop_no_def_r_r("cmovbe", src, dest)
    }
    fn emit_cmovbe_r_mem(&mut self, dest: &P<Value>, src: &P<Value>) {
        debug_assert!(check_op_len(dest) >= 16);
        self.internal_binop_no_def_r_mem("cmovbe", src, dest)
    }

    fn emit_cmove_r_r  (&mut self, dest: &P<Value>, src: &P<Value>) {
        debug_assert!(check_op_len(dest) >= 16);
        self.internal_binop_no_def_r_r("cmove", src, dest)
    }
    fn emit_cmove_r_mem(&mut self, dest: &P<Value>, src: &P<Value>) {
        debug_assert!(check_op_len(dest) >= 16);
        self.internal_binop_no_def_r_mem("cmove", src, dest)
    }

    fn emit_cmovg_r_r  (&mut self, dest: &P<Value>, src: &P<Value>) {
        debug_assert!(check_op_len(dest) >= 16);
        self.internal_binop_no_def_r_r("cmovg", src, dest)
    }
    fn emit_cmovg_r_mem(&mut self, dest: &P<Value>, src: &P<Value>) {
        debug_assert!(check_op_len(dest) >= 16);
        self.internal_binop_no_def_r_mem("cmovg", src, dest)
    }

    fn emit_cmovge_r_r  (&mut self, dest: &P<Value>, src: &P<Value>) {
        debug_assert!(check_op_len(dest) >= 16);
        self.internal_binop_no_def_r_r("cmovge", src, dest)
    }
    fn emit_cmovge_r_mem(&mut self, dest: &P<Value>, src: &P<Value>) {
        debug_assert!(check_op_len(dest) >= 16);
        self.internal_binop_no_def_r_mem("cmovge", src, dest)
    }

    fn emit_cmovl_r_r  (&mut self, dest: &P<Value>, src: &P<Value>) {
        debug_assert!(check_op_len(dest) >= 16);
        self.internal_binop_no_def_r_r("cmovl", src, dest)
    }
    fn emit_cmovl_r_mem(&mut self, dest: &P<Value>, src: &P<Value>) {
        debug_assert!(check_op_len(dest) >= 16);
        self.internal_binop_no_def_r_mem("cmovl", src, dest)
    }

    fn emit_cmovle_r_r  (&mut self, dest: &P<Value>, src: &P<Value>) {
        debug_assert!(check_op_len(dest) >= 16);
        self.internal_binop_no_def_r_r("cmovle", src, dest)
    }
    fn emit_cmovle_r_mem(&mut self, dest: &P<Value>, src: &P<Value>) {
        debug_assert!(check_op_len(dest) >= 16);
        self.internal_binop_no_def_r_mem("cmovle", src, dest)
    }

    fn emit_cmovne_r_r  (&mut self, dest: &P<Value>, src: &P<Value>) {
        debug_assert!(check_op_len(dest) >= 16);
        self.internal_binop_no_def_r_r("cmovne", src, dest)
    }
    fn emit_cmovne_r_mem(&mut self, dest: &P<Value>, src: &P<Value>) {
        debug_assert!(check_op_len(dest) >= 16);
        self.internal_binop_no_def_r_mem("cmovne", src, dest)
    }

    // lea
    fn emit_lea_r64(&mut self, dest: &P<Value>, src: &P<Value>) {
        self.internal_mov_r_mem("lea", dest, src)
    }

    // and
    fn emit_and_r_imm(&mut self, dest: Reg, src: i32) {
        self.internal_binop_def_r_imm("and", dest, src)
    }
    fn emit_and_r_r  (&mut self, dest: Reg, src: Reg) {
        self.internal_binop_def_r_r("and", dest, src)
    }
    fn emit_and_r_mem(&mut self, dest: Reg, src: Mem) {
        self.internal_binop_def_r_mem("and", dest, src)
    }

    // or
    fn emit_or_r_imm(&mut self, dest: Reg, src: i32) {
        self.internal_binop_def_r_imm("or", dest, src)
    }
    fn emit_or_r_r  (&mut self, dest: Reg, src: Reg) {
        self.internal_binop_def_r_r("or", dest, src)
    }
    fn emit_or_r_mem(&mut self, dest: Reg, src: Mem) {
        self.internal_binop_def_r_mem("or", dest, src)
    }

    // xor
    fn emit_xor_r_imm(&mut self, dest: Reg, src: i32) {
        self.internal_binop_def_r_imm("xor", dest, src)
    }
    fn emit_xor_r_r  (&mut self, dest: Reg, src: Reg) {
        self.internal_binop_def_r_r("xor", dest, src)
    }
    fn emit_xor_r_mem(&mut self, dest: Reg, src: Mem) {
        self.internal_binop_def_r_mem("xor", dest, src)
    }

    // add
    fn emit_add_r_imm(&mut self, dest: Reg, src: i32) {
        self.internal_binop_def_r_imm("add", dest, src)
    }
    fn emit_add_r_r  (&mut self, dest: Reg, src: Reg) {
        self.internal_binop_def_r_r("add", dest, src)
    }
    fn emit_add_r_mem(&mut self, dest: Reg, src: Mem) {
        self.internal_binop_def_r_mem("add", dest, src)
    }

    // sub
    fn emit_sub_r_imm(&mut self, dest: Reg, src: i32) {
        self.internal_binop_def_r_imm("sub", dest, src)
    }
    fn emit_sub_r_r  (&mut self, dest: Reg, src: Reg) {
        self.internal_binop_def_r_r("sub", dest, src)
    }
    fn emit_sub_r_mem(&mut self, dest: Reg, src: Mem) {
        self.internal_binop_def_r_mem("sub", dest, src)
    }
    
    fn emit_mul_r(&mut self, src: &P<Value>) {
        let len = check_op_len(src);

        let inst = "mul".to_string() + &op_postfix(len);
        
        let (reg, id, loc) = self.prepare_reg(src, inst.len() + 1);
        let rax = self.prepare_machine_reg(&x86_64::RAX);
        let rdx = self.prepare_machine_reg(&x86_64::RDX);
        
        let asm = format!("{} {}", inst, reg);

        if len != 8 {
            trace!("emit: {} rax, {} -> (rdx, rax)", inst, src);

            self.add_asm_inst(
                asm,
                linked_hashmap! {
                    rax => vec![],
                    rdx => vec![]
                },
                linked_hashmap! {
                    id => vec![loc],
                    rax => vec![]
                },
                false
            )
        } else {
            trace!("emit: {} al, {} -> ax", inst, src);

            self.add_asm_inst(
                asm,
                linked_hashmap! {
                    rax => vec![]
                },
                linked_hashmap! {
                    id => vec![loc],
                    rax => vec![]
                },
                false
            )
        }
    }

    fn emit_mul_mem(&mut self, src: &P<Value>) {
        unimplemented!()
    }

    fn emit_div_r  (&mut self, src: &P<Value>) {
        let len = check_op_len(src);

        let inst = "div".to_string() + &op_postfix(len);

        let rdx = self.prepare_machine_reg(&x86_64::RDX);
        let rax = self.prepare_machine_reg(&x86_64::RAX);
        let (reg, id, loc) = self.prepare_reg(src, inst.len() + 1);

        let asm = format!("{} {}", inst, reg);

        if len != 8 {
            trace!("emit: {} rdx:rax, {} -> quotient: rax + remainder: rdx", inst, src);
            self.add_asm_inst(
                asm,
                linked_hashmap!{
                rdx => vec![],
                rax => vec![],
            },
                linked_hashmap!{
                id => vec![loc],
                rdx => vec![],
                rax => vec![]
            },
                false
            )
        } else {
            trace!("emit: {} ah:al, {} -> quotient: al + remainder: ah", inst, src);

            let ah = self.prepare_machine_reg(&x86_64::AH);
            let al = self.prepare_machine_reg(&x86_64::AL);

            self.add_asm_inst(
                asm,
                linked_hashmap!{
                    ah => vec![],
                    al => vec![]
                },
                linked_hashmap!{
                    id => vec![loc],
                    ah => vec![],
                    al => vec![]
                },
                false
            )
        }
    }

    fn emit_div_mem (&mut self, src: &P<Value>) {
        let len = check_op_len(src);

        let inst = "div".to_string() + &op_postfix(len);

        let rdx = self.prepare_machine_reg(&x86_64::RDX);
        let rax = self.prepare_machine_reg(&x86_64::RAX);
        let (mem, mut uses) = self.prepare_mem(src, inst.len() + 1);

        // merge use vec
        if !uses.contains_key(&rdx) {
            uses.insert(rdx, vec![]);
        }
        if !uses.contains_key(&rax) {
            uses.insert(rax, vec![]);
        }

        let asm = format!("{} {}", inst, mem);

        if len != 8 {
            trace!("emit: {} rdx:rax, {} -> quotient: rax + remainder: rdx", inst, src);
            self.add_asm_inst(
                asm,
                linked_hashmap! {
                    rdx => vec![],
                    rax => vec![]
                },
                uses,
                true
            )
        } else {
            trace!("emit: {} ah:al, {} -> quotient: al + remainder: ah", inst, src);

            let ah = self.prepare_machine_reg(&x86_64::AH);
            let al = self.prepare_machine_reg(&x86_64::AL);

            // merge use vec
            if !uses.contains_key(&ah) {
                uses.insert(ah, vec![]);
            }
            if !uses.contains_key(&al) {
                uses.insert(al, vec![]);
            }

            self.add_asm_inst(
                asm,
                linked_hashmap!{
                    ah => vec![],
                    al => vec![]
                },
                uses,
                false
            )
        }
    }

    fn emit_idiv_r  (&mut self, src: &P<Value>) {
        let len = check_op_len(src);

        let inst = "idiv".to_string() + &op_postfix(len);

        let (reg, id, loc) = self.prepare_reg(src, inst.len() + 1);

        let asm = format!("{} {}", inst, reg);

        if len != 8 {
            trace!("emit: {} rdx:rax, {} -> quotient: rax + remainder: rdx", inst, src);

            let rdx = self.prepare_machine_reg(&x86_64::RDX);
            let rax = self.prepare_machine_reg(&x86_64::RAX);

            self.add_asm_inst(
                asm,
                linked_hashmap!{
                    rdx => vec![],
                    rax => vec![],
                },
                linked_hashmap!{
                    id => vec![loc],
                    rdx => vec![],
                    rax => vec![]
                },
                false
            )
        } else {
            trace!("emit: {} ah:al, {} -> quotient: al + remainder: ah", inst, src);

            let ah = self.prepare_machine_reg(&x86_64::AH);
            let al = self.prepare_machine_reg(&x86_64::AL);

            self.add_asm_inst(
                asm,
                linked_hashmap!{
                    ah => vec![],
                    al => vec![]
                },
                linked_hashmap!{
                    id => vec![loc],
                    ah => vec![],
                    al => vec![]
                },
                false
            )
        }
    }

    fn emit_idiv_mem(&mut self, src: &P<Value>) {
        let len = check_op_len(src);

        let inst = "idiv".to_string() + &op_postfix(len);

        let (mem, mut uses) = self.prepare_mem(src, inst.len() + 1);

        let asm = format!("{} {}", inst, mem);

        if len != 8 {
            trace!("emit: {} rdx:rax, {} -> quotient: rax + remainder: rdx", inst, src);

            let rdx = self.prepare_machine_reg(&x86_64::RDX);
            let rax = self.prepare_machine_reg(&x86_64::RAX);

            // merge use vec
            if !uses.contains_key(&rdx) {
                uses.insert(rdx, vec![]);
            }
            if !uses.contains_key(&rax) {
                uses.insert(rax, vec![]);
            }

            self.add_asm_inst(
                asm,
                linked_hashmap! {
                    rdx => vec![],
                    rax => vec![]
                },
                uses,
                true
            )
        } else {
            trace!("emit: {} ah:al, {} -> quotient: al + remainder: ah", inst, src);

            let ah = self.prepare_machine_reg(&x86_64::AH);
            let al = self.prepare_machine_reg(&x86_64::AL);

            // merge use vec
            if !uses.contains_key(&ah) {
                uses.insert(ah, vec![]);
            }
            if !uses.contains_key(&al) {
                uses.insert(al, vec![]);
            }

            self.add_asm_inst(
                asm,
                linked_hashmap!{
                    ah => vec![],
                    al => vec![]
                },
                uses,
                false
            )
        }
    }

    fn emit_shl_r_cl    (&mut self, dest: &P<Value>) {
        self.internal_binop_def_r_mr("shl", dest, &x86_64::CL)
    }

    fn emit_shl_r_imm8  (&mut self, dest: &P<Value>, src: i8) {
        self.internal_binop_def_r_imm("shl", dest, src as i32)
    }

    fn emit_shr_r_cl    (&mut self, dest: &P<Value>) {
        self.internal_binop_def_r_mr("shr", dest, &x86_64::CL)
    }

    fn emit_shr_r_imm8  (&mut self, dest: &P<Value>, src: i8) {
        self.internal_binop_def_r_imm("shr", dest, src as i32)
    }

    fn emit_sar_r_cl    (&mut self, dest: &P<Value>) {
        self.internal_binop_def_r_mr("sar", dest, &x86_64::CL)
    }

    fn emit_sar_r_imm8  (&mut self, dest: &P<Value>, src: i8) {
        self.internal_binop_def_r_imm("sar", dest, src as i32)
    }

    fn emit_cqo(&mut self) {
        trace!("emit: cqo rax -> rdx:rax");

        let rax = self.prepare_machine_reg(&x86_64::RAX);
        let rdx = self.prepare_machine_reg(&x86_64::RDX);

        let asm = format!("cqto");

        self.add_asm_inst(
            asm,
            linked_hashmap!{
                rdx => vec![],
                rax => vec![]
            },
            linked_hashmap!{
                rax => vec![]
            },
            false
        )
    }

    fn emit_cdq(&mut self) {
        trace!("emit: cdq eax -> edx:eax");

        let eax = self.prepare_machine_reg(&x86_64::EAX);
        let edx = self.prepare_machine_reg(&x86_64::EDX);

        let asm = format!("cltd");

        self.add_asm_inst(
            asm,
            linked_hashmap!{
                edx => vec![],
                eax => vec![]
            },
            linked_hashmap!{
                eax => vec![],
            },
            false
        )
    }

    fn emit_cwd(&mut self) {
        trace!("emit: cwd ax -> dx:ax");

        let ax = self.prepare_machine_reg(&x86_64::AX);
        let dx = self.prepare_machine_reg(&x86_64::DX);

        let asm = format!("cwtd");

        self.add_asm_inst(
            asm,
            linked_hashmap!{
                dx => vec![],
                ax => vec![]
            },
            linked_hashmap!{
                ax => vec![],
            },
            false
        )
    }
    
    fn emit_jmp(&mut self, dest_name: MuName) {
        trace!("emit: jmp {}", dest_name);
        
        // symbolic label, we dont need to patch it
        let asm = format!("jmp {}", symbol(self.mangle_block_label(dest_name.clone())));
        self.add_asm_branch(asm, dest_name)
    }
    
    fn emit_je(&mut self, dest_name: MuName) {
        trace!("emit: je {}", dest_name);
        
        let asm = format!("je {}", symbol(self.mangle_block_label(dest_name.clone())));
        self.add_asm_branch2(asm, dest_name);        
    }
    
    fn emit_jne(&mut self, dest_name: MuName) {
        trace!("emit: jne {}", dest_name);
        
        let asm = format!("jne {}", symbol(self.mangle_block_label(dest_name.clone())));
        self.add_asm_branch2(asm, dest_name);
    }
    
    fn emit_ja(&mut self, dest_name: MuName) {
        trace!("emit: ja {}", dest_name);
        
        let asm = format!("ja {}", symbol(self.mangle_block_label(dest_name.clone())));
        self.add_asm_branch2(asm, dest_name);
    }
    
    fn emit_jae(&mut self, dest_name: MuName) {
        trace!("emit: jae {}", dest_name);
        
        let asm = format!("jae {}", symbol(self.mangle_block_label(dest_name.clone())));
        self.add_asm_branch2(asm, dest_name);        
    }
    
    fn emit_jb(&mut self, dest_name: MuName) {
        trace!("emit: jb {}", dest_name);
        
        let asm = format!("jb {}", symbol(self.mangle_block_label(dest_name.clone())));
        self.add_asm_branch2(asm, dest_name);
    }
    
    fn emit_jbe(&mut self, dest_name: MuName) {
        trace!("emit: jbe {}", dest_name);
        
        let asm = format!("jbe {}", symbol(self.mangle_block_label(dest_name.clone())));
        self.add_asm_branch2(asm, dest_name);        
    }
    
    fn emit_jg(&mut self, dest_name: MuName) {
        trace!("emit: jg {}", dest_name);
        
        let asm = format!("jg {}", symbol(self.mangle_block_label(dest_name.clone())));
        self.add_asm_branch2(asm, dest_name);        
    }
    
    fn emit_jge(&mut self, dest_name: MuName) {
        trace!("emit: jge {}", dest_name);
        
        let asm = format!("jge {}", symbol(self.mangle_block_label(dest_name.clone())));
        self.add_asm_branch2(asm, dest_name);        
    }
    
    fn emit_jl(&mut self, dest_name: MuName) {
        trace!("emit: jl {}", dest_name);
        
        let asm = format!("jl {}", symbol(self.mangle_block_label(dest_name.clone())));
        self.add_asm_branch2(asm, dest_name);        
    }
    
    fn emit_jle(&mut self, dest_name: MuName) {
        trace!("emit: jle {}", dest_name);
        
        let asm = format!("jle {}", symbol(self.mangle_block_label(dest_name.clone())));
        self.add_asm_branch2(asm, dest_name);        
    }    

    #[cfg(target_os = "macos")]
    fn emit_call_near_rel32(&mut self, callsite: String, func: MuName) -> ValueLocation {
        trace!("emit: call {}", func);
        
        let asm = format!("call {}", symbol(func));
        self.add_asm_call(asm);
        
        let callsite_symbol = symbol(callsite.clone());
        self.add_asm_symbolic(directive_globl(callsite_symbol.clone()));
        self.add_asm_symbolic(format!("{}:", callsite_symbol.clone()));            
        
        ValueLocation::Relocatable(RegGroup::GPR, callsite)
    }

    #[cfg(target_os = "linux")]
    // generating Position-Independent Code using PLT
    fn emit_call_near_rel32(&mut self, callsite: String, func: MuName) -> ValueLocation {
        trace!("emit: call {}", func);

        let func = func + "@PLT";

        let asm = format!("call {}", symbol(func));
        self.add_asm_call(asm);

        let callsite_symbol = symbol(callsite.clone());
        self.add_asm_symbolic(directive_globl(callsite_symbol.clone()));
        self.add_asm_symbolic(format!("{}:", callsite_symbol.clone()));

        ValueLocation::Relocatable(RegGroup::GPR, callsite)
    }
    
    fn emit_call_near_r64(&mut self, callsite: String, func: &P<Value>) -> ValueLocation {
        trace!("emit: call {}", func);
        unimplemented!()
    }
    
    fn emit_call_near_mem64(&mut self, callsite: String, func: &P<Value>) -> ValueLocation {
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
            linked_hashmap!{
                rsp => vec![]
            },
            linked_hashmap!{
                id => vec![loc],
                rsp => vec![]
            },
            false
        )
    }
    
    fn emit_push_imm32(&mut self, src: i32) {
        trace!("emit: push {}", src);
        
        let rsp = self.prepare_machine_reg(&x86_64::RSP);
        
        let asm = format!("pushq ${}", src);
        
        self.add_asm_inst(
            asm,
            linked_hashmap!{
                rsp => vec![]
            },
            linked_hashmap!{
                rsp => vec![]
            },
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
            linked_hashmap!{
                id => vec![loc.clone()],
                rsp => vec![]
            },
            linked_hashmap!{
                rsp => vec![]
            },
            false
        )        
    }

    fn emit_movsd_f64_f64  (&mut self, dest: &P<Value>, src: &P<Value>) {
        trace!("emit: movsd {} -> {}", src, dest);

        let (reg1, id1, loc1) = self.prepare_fpreg(src, 5 + 1);
        let (reg2, id2, loc2) = self.prepare_fpreg(dest, 5 + 1 + reg1.len() + 1);

        let asm = format!("movsd {},{}", reg1, reg2);

        self.add_asm_inst(
            asm,
            linked_hashmap!{
                id2 => vec![loc2]
            },
            linked_hashmap!{
                id1 => vec![loc1]
            },
            false
        )
    }

    // load
    fn emit_movsd_f64_mem64(&mut self, dest: &P<Value>, src: &P<Value>) {
        trace!("emit: movsd {} -> {}", src, dest);

        let (mem, uses) = self.prepare_mem(src, 5 + 1);
        let (reg, id2, loc2) = self.prepare_fpreg(dest, 5 + 1 + mem.len() + 1);

        let asm = format!("movsd {},{}", mem, reg);

        self.add_asm_inst(
            asm,
            linked_hashmap!{
                id2 => vec![loc2]
            },
            uses,
            true
        )
    }

    // store
    fn emit_movsd_mem64_f64(&mut self, dest: &P<Value>, src: &P<Value>) {
        trace!("emit: movsd {} -> {}", src, dest);

        let (reg, id1, loc1) = self.prepare_fpreg(src, 5 + 1);
        let (mem, mut uses) = self.prepare_mem(dest, 5 + 1 + reg.len() + 1);

        // the register we used for the memory location is counted as 'use'
        // use the vec from mem as 'use' (push use reg from src to it)
        if uses.contains_key(&id1) {
            uses.get_mut(&id1).unwrap().push(loc1);
        } else {
            uses.insert(id1, vec![loc1]);
        }

        let asm = format!("movsd {},{}", reg, mem);

        self.add_asm_inst(
            asm,
            linked_hashmap!{},
            uses,
            true
        )
    }

    fn emit_comisd_f64_f64  (&mut self, op1: Reg, op2: Reg) {
        self.internal_fp_binop_no_def_r_r("comisd", op1, op2);
    }
    fn emit_ucomisd_f64_f64 (&mut self, op1: Reg, op2: Reg) {
        self.internal_fp_binop_no_def_r_r("ucomisd", op1, op2);
    }

    fn emit_addsd_f64_f64  (&mut self, dest: &P<Value>, src: &P<Value>) {
        self.internal_fp_binop_def_r_r("addsd", dest, src);
    }

    fn emit_addsd_f64_mem64(&mut self, dest: &P<Value>, src: &P<Value>) {
        self.internal_fp_binop_def_r_mem("addsd", dest, src);
    }

    fn emit_subsd_f64_f64  (&mut self, dest: Reg, src: Reg) {
        self.internal_fp_binop_def_r_r("subsd", dest, src);
    }
    fn emit_subsd_f64_mem64(&mut self, dest: Reg, src: Mem) {
        self.internal_fp_binop_def_r_mem("subsd", dest, src);
    }

    fn emit_divsd_f64_f64  (&mut self, dest: Reg, src: Reg) {
        self.internal_fp_binop_def_r_r("divsd", dest, src);
    }
    fn emit_divsd_f64_mem64(&mut self, dest: Reg, src: Mem) {
        self.internal_fp_binop_def_r_mem("divsd", dest, src);
    }

    fn emit_mulsd_f64_f64  (&mut self, dest: Reg, src: Reg) {
        self.internal_fp_binop_def_r_r("mulsd", dest, src);
    }
    fn emit_mulsd_f64_mem64(&mut self, dest: Reg, src: Mem) {
        self.internal_fp_binop_def_r_mem("mulsd", dest, src);
    }

    fn emit_cvtsi2sd_f64_r  (&mut self, dest: Reg, src: Reg) {
        let len = check_op_len(src);

        let inst = "cvtsi2sd".to_string() + &op_postfix(len);
        trace!("emit: {} {} -> {}", inst, src, dest);

        let (reg1, id1, loc1) = self.prepare_reg  (src,  inst.len() + 1);
        let (reg2, id2, loc2) = self.prepare_fpreg(dest, inst.len() + 1 + reg1.len() + 1);

        let asm = format!("{} {},{}", inst, reg1, reg2);

        self.add_asm_inst(
            asm,
            linked_hashmap!{
                id2 => vec![loc2]
            },
            linked_hashmap!{
                id1 => vec![loc1]
            },
            false
        )
    }

    fn emit_cvtsd2si_r_f64  (&mut self, dest: Reg, src: Reg) {
        let len = check_op_len(dest);

        let inst = "cvtsd2si".to_string() + &op_postfix(len);
        trace!("emit: {} {} -> {}", inst, src, dest);

        let (reg1, id1, loc1) = self.prepare_fpreg(src,  inst.len() + 1);
        let (reg2, id2, loc2) = self.prepare_reg  (dest, inst.len() + 1 + reg1.len() + 1);

        let asm = format!("{} {},{}", inst, reg1, reg2);

        self.add_asm_inst(
            asm,
            linked_hashmap!{
                id2 => vec![loc2]
            },
            linked_hashmap!{
                id1 => vec![loc1]
            },
            false
        )
    }
}

fn create_emit_directory(vm: &VM) {
    use std::fs;    
    match fs::create_dir(&vm.vm_options.flag_aot_emit_dir) {
        Ok(_) => {},
        Err(_) => {}
    }    
}

use std::fs::File;

pub fn emit_code(fv: &mut MuFunctionVersion, vm: &VM) {
    use std::io::prelude::*;
    use std::path;
    
    let funcs = vm.funcs().read().unwrap();
    let func = funcs.get(&fv.func_id).unwrap().read().unwrap();

    let compiled_funcs = vm.compiled_funcs().read().unwrap();
    let cf = compiled_funcs.get(&fv.id()).unwrap().read().unwrap();

    let code = cf.mc.as_ref().unwrap().emit();

    // create 'emit' directory
    create_emit_directory(vm);

    let mut file_path = path::PathBuf::new();
    file_path.push(&vm.vm_options.flag_aot_emit_dir);
    file_path.push(func.name().unwrap().to_string() + ".s");
    let mut file = match File::create(file_path.as_path()) {
        Err(why) => panic!("couldn't create emission file {}: {}", file_path.to_str().unwrap(), why),
        Ok(file) => file
    };

    match file.write_all(code.as_slice()) {
        Err(why) => panic!("couldn'd write to file {}: {}", file_path.to_str().unwrap(), why),
        Ok(_) => info!("emit code to {}", file_path.to_str().unwrap())
    }
}

pub fn emit_context(vm: &VM) {
    use std::path;
    use std::io::prelude::*;
    use rustc_serialize::json;
    
    debug!("---Emit VM Context---");
    create_emit_directory(vm);
    
    let mut file_path = path::PathBuf::new();
    file_path.push(&vm.vm_options.flag_aot_emit_dir);
    file_path.push(AOT_EMIT_CONTEXT_FILE);
    
    let mut file = match File::create(file_path.as_path()) {
        Err(why) => panic!("couldn't create context file {}: {}", file_path.to_str().unwrap(), why),
        Ok(file) => file
    };
    
    // bss
    file.write_fmt(format_args!("\t.bss\n")).unwrap();

    {
        // put globals into bss section
//        let globals = vm.globals().read().unwrap();
//        for global in globals.values() {
//            debug!("emit global: {}", global);
//            let (size, align) = {
//                let alloc_ty = {
//                    match global.v {
//                        Value_::Global(ref ty) => ty,
//                        _ => panic!("expected a global")
//                    }
//                };
//
//                debug!("getting type: {:?}", alloc_ty);
//                let ty_info = vm.get_backend_type_info(alloc_ty.id());
//                (ty_info.size, ty_info.alignment)
//            };
//
//            file.write_fmt(format_args!("\t{}\n", directive_globl(symbol(global.name().unwrap())))).unwrap();
//            file.write_fmt(format_args!("\t{}\n", directive_comm(symbol(global.name().unwrap()), size, align))).unwrap();
//            file.write("\n".as_bytes()).unwrap();
//        }
    }
    
    // data
    file.write("\t.data\n".as_bytes()).unwrap();

    {
        use runtime::mm;

        // persist globals
        let global_locs_lock = vm.global_locations.read().unwrap();
        let global_lock      = vm.globals().read().unwrap();

        let global_addr_id_map = {
            let mut map : LinkedHashMap<Address, MuID> = LinkedHashMap::new();

            for (id, global_loc) in global_locs_lock.iter() {
                map.insert(global_loc.to_address(), *id);
            }

            map
        };

        // dump heap from globals
        let global_addrs : Vec<Address> = global_locs_lock.values().map(|x| x.to_address()).collect();
        debug!("going to dump these globals: {:?}", global_addrs);
        let global_dump = mm::persist_heap(global_addrs);
        debug!("Heap Dump from GC: {:?}", global_dump);

        let ref objects          = global_dump.objects;
        let ref relocatable_refs = global_dump.relocatable_refs;

        for obj_dump in objects.values() {
            // .bytes xx,xx,xx,xx (between mem_start to reference_addr)
            write_data_bytes(&mut file, obj_dump.mem_start, obj_dump.reference_addr);

            if global_addr_id_map.contains_key(&obj_dump.reference_addr) {
                let global_id = global_addr_id_map.get(&obj_dump.reference_addr).unwrap();

                let global_value = global_lock.get(global_id).unwrap();

                // .globl global_cell_name
                // global_cell_name:
                let global_cell_name = symbol(global_value.name().unwrap());
                file.write_fmt(format_args!("\t{}\n", directive_globl(global_cell_name.clone()))).unwrap();
                file.write_fmt(format_args!("{}:\n", global_cell_name)).unwrap();
            }

            // dump_label:
            let dump_label = symbol(global_dump.relocatable_refs.get(&obj_dump.reference_addr).unwrap().clone());
            file.write_fmt(format_args!("{}:\n", dump_label)).unwrap();

            let base = obj_dump.reference_addr;
            let mut cursor = obj_dump.reference_addr;
            for ref_offset in obj_dump.reference_offsets.iter() {
                let cur_ref_addr = base.plus(*ref_offset);

                if cursor < cur_ref_addr {
                    // write all non-ref data
                    write_data_bytes(&mut file, cursor, cur_ref_addr);
                }

                // write ref with label
                let load_ref = unsafe {cur_ref_addr.load::<Address>()};
                if load_ref.is_zero() {
                    file.write("\t.quad 0\n".as_bytes()).unwrap();
                } else {
                    let label = match global_dump.relocatable_refs.get(&load_ref) {
                        Some(label) => label,
                        None => panic!("cannot find label for address {}, it is not dumped by GC (why GC didn't trace to it)", load_ref)
                    };

                    file.write_fmt(format_args!("\t.quad {}\n", symbol(label.clone()))).unwrap();
                }

                cursor = cur_ref_addr.plus(POINTER_SIZE);
            }

            // write whatever is after the last ref
            write_data_bytes(&mut file, cursor, obj_dump.mem_start.plus(obj_dump.mem_size));
        }
    }

    // serialize vm
    trace!("start serializing vm");
    {
        let serialize_vm = json::encode(&vm).unwrap();
        
        let vm_symbol = symbol("vm".to_string());
        file.write_fmt(format_args!("{}\n", directive_globl(vm_symbol.clone()))).unwrap();
        let escape_serialize_vm = serialize_vm.replace("\"", "\\\"");
        file.write_fmt(format_args!("\t{}: .asciz \"{}\"", vm_symbol, escape_serialize_vm)).unwrap();
        file.write("\n".as_bytes()).unwrap();
    }
    
    // main_thread
//    let primordial = vm.primordial.read().unwrap();
//    if primordial.is_some() {
//        let primordial = primordial.as_ref().unwrap();
//    }
    
    debug!("---finish---");
}

fn write_data_bytes(f: &mut File, from: Address, to: Address) {
    use std::io::Write;

    if from < to {
        f.write("\t.byte ".as_bytes()).unwrap();

        let mut cursor = from;
        while cursor < to {
            let byte = unsafe {cursor.load::<u8>()};
            f.write_fmt(format_args!("0x{:x}", byte)).unwrap();

            cursor = cursor.plus(1);

            if cursor != to {
                f.write(",".as_bytes()).unwrap();
            }
        }

        f.write("\n".as_bytes()).unwrap();
    }
}

fn directive_globl(name: String) -> String {
    format!(".globl {}", name)
}

fn directive_comm(name: String, size: ByteSize, align: ByteSize) -> String {
    format!(".comm {},{},{}", name, size, align)
}

#[cfg(target_os = "linux")]
pub fn symbol(name: String) -> String {
    name
}
#[cfg(target_os = "macos")]
pub fn symbol(name: String) -> String {
    format!("_{}", name)
}

#[allow(dead_code)]
#[cfg(target_os = "linux")]
pub fn pic_symbol(name: String) -> String {
    format!("{}@GOTPCREL", name)
}
#[allow(dead_code)]
#[cfg(target_os = "macos")]
pub fn pic_symbol(name: String) -> String {
    symbol(name)
}

use compiler::machine_code::CompiledFunction;

pub fn spill_rewrite(
    spills: &HashMap<MuID, P<Value>>,
    func: &mut MuFunctionVersion,
    cf: &mut CompiledFunction,
    vm: &VM) -> Vec<P<Value>>
{
    trace!("spill rewrite for x86_64 asm backend");

    trace!("code before spilling");
    cf.mc().trace_mc();

    let mut new_nodes = vec![];

    // record code and their insertion point, so we can do the copy/insertion all at once
    let mut spill_code_before: HashMap<usize, Vec<Box<ASMCode>>> = HashMap::new();
    let mut spill_code_after: HashMap<usize, Vec<Box<ASMCode>>> = HashMap::new();

    // map from old to new
    let mut temp_for_cur_inst : HashMap<MuID, P<Value>> = HashMap::new();

    // iterate through all instructions
    for i in 0..cf.mc().number_of_insts() {
        temp_for_cur_inst.clear();

        trace!("---Inst {}---", i);
        // find use of any register that gets spilled
        {
            let reg_uses = cf.mc().get_inst_reg_uses(i).to_vec();
            for reg in reg_uses {
                if spills.contains_key(&reg) {
                    let val_reg = func.context.get_value(reg).unwrap().value().clone();

                    // a register used here is spilled
                    let spill_mem = spills.get(&reg).unwrap();

                    // generate a random new temporary
                    let temp_ty = val_reg.ty.clone();
                    let temp = func.new_ssa(vm.next_id(), temp_ty.clone()).clone_value();
                    vec_utils::add_unique(&mut new_nodes, temp.clone());
                    trace!("reg {} used in Inst{} is replaced as {}", val_reg, i, temp);

                    // generate a load
                    let code = {
                        let mut codegen = ASMCodeGen::new();
                        codegen.start_code_sequence();

                        if is_fp(&temp_ty) {
                            codegen.emit_movsd_f64_mem64(&temp, spill_mem);
                        } else {
                            codegen.emit_mov_r_mem(&temp, spill_mem);
                        }

                        codegen.finish_code_sequence_asm()
                    };
                    // record that this load will be inserted at i
                    trace!("insert before inst #{}", i);
                    if spill_code_before.contains_key(&i) {
                        spill_code_before.get_mut(&i).unwrap().push(code);
                    } else {
                        spill_code_before.insert(i, vec![code]);
                    }

                    // replace register reg with temp
                    cf.mc_mut().replace_use_tmp_for_inst(reg, temp.id(), i);

                    temp_for_cur_inst.insert(reg, temp.clone());
                }
            }
        }

        // find define of any register that gets spilled
        {
            let reg_defines = cf.mc().get_inst_reg_defines(i).to_vec();
            for reg in reg_defines {
                if spills.contains_key(&reg) {
                    let val_reg = func.context.get_value(reg).unwrap().value().clone();

                    let spill_mem = spills.get(&reg).unwrap();

                    let temp = if temp_for_cur_inst.contains_key(&reg) {
                        temp_for_cur_inst.get(&reg).unwrap().clone()
                    } else {
                        let temp_ty = val_reg.ty.clone();
                        let temp = func.new_ssa(vm.next_id(), temp_ty.clone()).clone_value();
                        vec_utils::add_unique(&mut new_nodes, temp.clone());

                        temp
                    };
                    trace!("reg {} defined in Inst{} is replaced as {}", val_reg, i, temp);

                    let code = {
                        let mut codegen = ASMCodeGen::new();
                        codegen.start_code_sequence();

                        if is_fp(&temp.ty) {
                            codegen.emit_movsd_mem64_f64(spill_mem, &temp);
                        } else {
                            codegen.emit_mov_mem_r(spill_mem, &temp);
                        }

                        codegen.finish_code_sequence_asm()
                    };

                    trace!("insert after inst #{}", i);
                    if spill_code_after.contains_key(&i) {
                        spill_code_after.get_mut(&i).unwrap().push(code);
                    } else {
                        spill_code_after.insert(i, vec![code]);
                    }

                    cf.mc_mut().replace_define_tmp_for_inst(reg, temp.id(), i);
                }
            }
        }
    }

    // copy and insert the code
    let new_mc = {
        let old_mc = cf.mc.take().unwrap();
        let old_mc_ref : &ASMCode = old_mc.as_any().downcast_ref().unwrap();
        old_mc_ref.rewrite_insert(spill_code_before, spill_code_after)
    };

    cf.mc = Some(new_mc);

    trace!("spill rewrite done");

    trace!("code after spilling");
    cf.mc().trace_mc();

    new_nodes
}
