// Copyright 2017 The Australian National University
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
use ast::types;

use std::str;
use std::usize;
use std::slice::Iter;
use std::ops;
use std::collections::HashSet;
use std::sync::{RwLock, Arc};
use std::any::Any;

/// ASMCode represents a segment of assembly machine code. Usually it is machine code for
/// a Mu function, but it could simply be a sequence of machine code.
/// This data structure implements MachineCode trait which allows compilation passes to
/// operate on the machine code in a machine independent way.
/// This data structure is also designed in a way to support in-place code generation. Though
/// in-place code generation is mostly irrelevant for ahead-of-time compilation, I tried
/// test the idea with this AOT backend.
struct ASMCode {
    /// function name for the code
    name: MuName,
    /// a list of all the assembly instructions
    code: Vec<ASMInst>,
    /// entry block name
    entry: MuName,
    /// all the blocks
    blocks: LinkedHashMap<MuName, ASMBlock>,
    /// the patch location for frame size growth/shrink
    /// we only know the exact frame size after register allocation, but we need to insert
    /// frame adjust code beforehand, so we insert adjust code with an empty frame size, and
    /// patch it later
    frame_size_patchpoints: Vec<ASMLocation>
}

unsafe impl Send for ASMCode {}
unsafe impl Sync for ASMCode {}

/// ASMInst represents an assembly instruction.
/// This data structure contains enough information to implement MachineCode trait on ASMCode,
/// and it also supports in-place code generation.
#[derive(Clone, Debug)]
struct ASMInst {
    /// actual asm code
    code: String,
    /// defines of this instruction. a map from temporary/register ID to its location
    /// (where it appears in the code string)
    defines: LinkedHashMap<MuID, Vec<ASMLocation>>,
    /// uses of this instruction
    uses: LinkedHashMap<MuID, Vec<ASMLocation>>,
    /// is this instruction using memory operand?
    is_mem_op_used: bool,
    /// is this assembly code a symbol? (not an actual instruction)
    is_symbol: bool,
    /// is this instruction an inserted spill instruction (load from/store to memory)?
    spill_info: Option<SpillMemInfo>,
    /// predecessors of this instruction
    preds: Vec<usize>,
    /// successors of this instruction
    succs: Vec<usize>,
    /// branch target of this instruction
    branch: ASMBranchTarget
}

/// ASMLocation represents the location of a register/temporary in assembly code.
/// It contains enough information so that we can later patch the register.
#[derive(Clone, Debug, PartialEq, Eq)]
struct ASMLocation {
    /// which row it is in the assembly code vector
    line: usize,
    /// which column
    index: usize,
    /// length of spaces reserved for the register/temporary
    len: usize,
    /// bit-length of the register/temporary
    oplen: usize
}

/// ASMBlock represents information about a basic block in assembly.
#[derive(Clone, Debug)]
struct ASMBlock {
    /// [start_inst, end_inst) (includes start_inst)
    start_inst: usize,
    /// [start_inst, end_inst) (excludes end_inst)
    end_inst: usize,
    /// livein reg/temp
    livein: Vec<MuID>,
    /// liveout reg/temp
    liveout: Vec<MuID>
}

/// ASMBranchTarget represents branching control flow of machine instructions.
#[derive(Clone, Debug)]
enum ASMBranchTarget {
    /// not a branching instruction
    None,
    /// a conditional branch to target
    Conditional(MuName),
    /// an unconditional branch to target
    Unconditional(MuName),
    /// this instruction may throw exception to target
    PotentiallyExcepting(MuName),
    /// this instruction is a return
    Return
}

/// SpillMemInfo represents inserted spilling instructions for loading/storing values
#[derive(Clone, Debug)]
enum SpillMemInfo {
    Load(P<Value>),
    Store(P<Value>),
    CalleeSaved // Callee saved record
}

impl ASMCode {
    /// returns a vector of ASMLocation for all the uses of the given reg/temp
    fn get_use_locations(&self, reg: MuID) -> Vec<ASMLocation> {
        let mut ret = vec![];

        for inst in self.code.iter() {
            match inst.uses.get(&reg) {
                Some(ref locs) => {
                    ret.append(&mut locs.to_vec());
                }
                None => {}
            }
        }

        ret
    }

    /// returns a vector of ASMLocation for all the defines of the given reg/temp
    fn get_define_locations(&self, reg: MuID) -> Vec<ASMLocation> {
        let mut ret = vec![];

        for inst in self.code.iter() {
            match inst.defines.get(&reg) {
                Some(ref locs) => {
                    ret.append(&mut locs.to_vec());
                }
                None => {}
            }
        }

        ret
    }

    /// is the given instruction the starting instruction of a block?
    fn is_block_start(&self, inst: usize) -> bool {
        for block in self.blocks.values() {
            if block.start_inst == inst {
                return true;
            }
        }
        false
    }

    /// is the given instruction the ending instruction of a block?
    fn is_last_inst_in_block(&self, inst: usize) -> bool {
        for block in self.blocks.values() {
            if block.end_inst == inst + 1 {
                return true;
            }
        }
        false
    }

    /// finds block for a given instruction and returns the block
    fn get_block_by_inst(&self, inst: usize) -> (&MuName, &ASMBlock) {
        for (name, block) in self.blocks.iter() {
            if inst >= block.start_inst && inst < block.end_inst {
                return (name, block);
            }
        }
        panic!("didnt find any block for inst {}", inst)
    }

    /// finds block that starts with the given instruction
    /// returns None if we cannot find such block
    fn get_block_by_start_inst(&self, inst: usize) -> Option<&ASMBlock> {
        for block in self.blocks.values() {
            if block.start_inst == inst {
                return Some(block);
            }
        }
        None
    }

    /// rewrites code by inserting instructions in certain locations
    /// This function is used for inserting spilling instructions. It takes
    /// two hashmaps as arguments, the keys of which are line numbers where
    /// we should insert code, and the values are the code to be inserted.
    /// We need to carefully ensure the metadata for existing code is
    /// still correct after insertion. This function returns the resulting code.
    fn rewrite_insert(
        &self,
        insert_before: LinkedHashMap<usize, Vec<Box<ASMCode>>>,
        insert_after: LinkedHashMap<usize, Vec<Box<ASMCode>>>
    ) -> Box<ASMCode> {
        trace!("insert spilling code");
        let mut ret = ASMCode {
            name: self.name.clone(),
            entry: self.entry.clone(),
            code: vec![],
            blocks: linked_hashmap!{},
            frame_size_patchpoints: vec![]
        };

        // how many instructions have been inserted
        let mut inst_offset = 0;
        let mut cur_block_start = usize::MAX;

        // inst N in old machine code is N' in new machine code
        // this map stores the relationship
        let mut location_map: LinkedHashMap<usize, usize> = LinkedHashMap::new();

        // iterate through old machine code
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

            // if we finish a block
            if self.is_last_inst_in_block(i) {
                let cur_block_end = i + 1 + inst_offset;

                // copy the block
                let (name, block) = self.get_block_by_inst(i);

                let new_block = ASMBlock {
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

        // fix patchpoints
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

    /// appends a given part of assembly code sequence at the end of current code
    /// During appending, we need to fix line number.
    fn append_code_sequence(&mut self, another: &Box<ASMCode>, start_inst: usize, n_insts: usize) {
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

    /// appends assembly sequence at the end of current code
    fn append_code_sequence_all(&mut self, another: &Box<ASMCode>) {
        let n_insts = another.number_of_insts();
        self.append_code_sequence(another, 0, n_insts)
    }

    /// control flow analysis on current code
    /// calculating branch targets, preds/succs for each instruction
    fn control_flow_analysis(&mut self) {
        const TRACE_CFA: bool = true;

        // control flow analysis
        let n_insts = self.number_of_insts();
        let ref mut asm = self.code;

        for i in 0..n_insts {
            trace_if!(TRACE_CFA, "---inst {}---", i);

            // skip symbol
            if asm[i].is_symbol {
                continue;
            }

            // determine predecessor:
            // * if last instruction falls through to current instruction,
            //   the predecessor is last instruction
            // * otherwise, we set predecessor when we deal with the instruction
            //   that branches to current instruction
            if i != 0 {
                let last_inst = ASMCode::find_prev_inst(i, asm);
                match last_inst {
                    Some(last_inst) => {
                        let last_inst_branch = asm[last_inst].branch.clone();
                        match last_inst_branch {
                            // if it is a fallthrough, we set its preds as last inst
                            ASMBranchTarget::None => {
                                if !asm[i].preds.contains(&last_inst) {
                                    asm[i].preds.push(last_inst);
                                    trace_if!(
                                        TRACE_CFA,
                                        "inst {}: set PREDS as previous inst - fallthrough {}",
                                        i,
                                        last_inst
                                    );
                                }
                            }
                            // otherwise do nothing
                            _ => {}
                        }
                    }
                    None => {}
                }
            }

            // determine successor
            // make a clone so that we are not borrowing anything
            let branch = asm[i].branch.clone();
            match branch {
                ASMBranchTarget::Unconditional(ref target) => {
                    // branch-to target
                    let target_n = self.blocks.get(target).unwrap().start_inst;

                    // cur inst's succ is target
                    asm[i].succs.push(target_n);

                    // target's pred is cur
                    asm[target_n].preds.push(i);

                    trace_if!(TRACE_CFA, "inst {}: is a branch to {}", i, target);
                    trace_if!(TRACE_CFA, "inst {}: branch target index is {}", i, target_n);
                    trace_if!(
                        TRACE_CFA,
                        "inst {}: set SUCCS as branch target {}",
                        i,
                        target_n
                    );
                    trace_if!(
                        TRACE_CFA,
                        "inst {}: set PREDS as branch source {}",
                        target_n,
                        i
                    );
                }
                ASMBranchTarget::Conditional(ref target) => {
                    // branch-to target
                    let target_n = self.blocks.get(target).unwrap().start_inst;

                    // cur insts' succ is target
                    asm[i].succs.push(target_n);

                    trace_if!(TRACE_CFA, "inst {}: is a cond branch to {}", i, target);
                    trace_if!(TRACE_CFA, "inst {}: branch target index is {}", i, target_n);
                    trace_if!(
                        TRACE_CFA,
                        "inst {}: set SUCCS as branch target {}",
                        i,
                        target_n
                    );

                    // target's pred is cur
                    asm[target_n].preds.push(i);
                    trace_if!(TRACE_CFA, "inst {}: set PREDS as {}", target_n, i);

                    if let Some(next_inst) = ASMCode::find_next_inst(i, asm) {
                        // cur succ is next inst
                        asm[i].succs.push(next_inst);

                        // next inst's pred is cur
                        asm[next_inst].preds.push(i);

                        trace_if!(
                            TRACE_CFA,
                            "inst {}: SET SUCCS as c-branch fallthrough target {}",
                            i,
                            next_inst
                        );
                    } else {
                        panic!("conditional branch does not have a fallthrough target");
                    }
                }
                ASMBranchTarget::PotentiallyExcepting(ref target) => {
                    // may trigger exception and jump to target - similar as conditional branch
                    let target_n = self.blocks.get(target).unwrap().start_inst;

                    // cur inst's succ is target
                    asm[i].succs.push(target_n);

                    trace_if!(
                        TRACE_CFA,
                        "inst {}: is potentially excepting to {}",
                        i,
                        target
                    );
                    trace_if!(
                        TRACE_CFA,
                        "inst {}: excepting target index is {}",
                        i,
                        target_n
                    );
                    trace_if!(
                        TRACE_CFA,
                        "inst {}: set SUCCS as excepting target {}",
                        i,
                        target_n
                    );

                    asm[target_n].preds.push(i);

                    if let Some(next_inst) = ASMCode::find_next_inst(i, asm) {
                        // cur succ is next inst
                        asm[i].succs.push(next_inst);

                        // next inst's pred is cur
                        asm[next_inst].preds.push(i);

                        trace_if!(
                            TRACE_CFA,
                            "inst {}: SET SUCCS as PEI fallthrough target {}",
                            i,
                            next_inst
                        );
                    } else {
                        panic!("PEI does not have a fallthrough target");
                    }
                }
                ASMBranchTarget::Return => {
                    trace_if!(TRACE_CFA, "inst {}: is a return", i);
                    trace_if!(TRACE_CFA, "inst {}: has no successor", i);
                }
                ASMBranchTarget::None => {
                    // not branch nor cond branch, succ is next inst
                    trace_if!(TRACE_CFA, "inst {}: not a branch inst", i);
                    if let Some(next_inst) = ASMCode::find_next_inst(i, asm) {
                        trace_if!(
                            TRACE_CFA,
                            "inst {}: set SUCCS as next inst {}",
                            i,
                            next_inst
                        );
                        asm[i].succs.push(next_inst);
                    }
                }
            }
        }
    }

    /// finds the previous instruction (skip non-instruction assembly)
    fn find_prev_inst(i: usize, asm: &Vec<ASMInst>) -> Option<usize> {
        if i == 0 {
            None
        } else {
            let mut cur = i - 1;
            while cur != 0 {
                if !asm[cur].is_symbol {
                    return Some(cur);
                }

                if cur == 0 {
                    return None;
                } else {
                    cur -= 1;
                }
            }

            None
        }
    }

    /// finds the next instruction (skip non-instruction assembly)
    fn find_next_inst(i: usize, asm: &Vec<ASMInst>) -> Option<usize> {
        if i >= asm.len() - 1 {
            None
        } else {
            let mut cur = i + 1;
            while cur < asm.len() {
                if !asm[cur].is_symbol {
                    return Some(cur);
                }

                cur += 1;
            }

            None
        }
    }

    /// finds the last instruction that appears on or before the given index
    /// (skip non-instruction assembly)
    fn find_last_inst(i: usize, asm: &Vec<ASMInst>) -> Option<usize> {
        if i == 0 {
            None
        } else {
            let mut cur = i;
            loop {
                if !asm[cur].is_symbol {
                    return Some(cur);
                }

                if cur == 0 {
                    return None;
                } else {
                    cur -= 1;
                }
            }
        }
    }

    fn add_frame_size_patchpoint(&mut self, patchpoint: ASMLocation) {
        self.frame_size_patchpoints.push(patchpoint);
    }
}

impl MachineCode for ASMCode {
    fn as_any(&self) -> &Any {
        self
    }

    /// returns the count of instructions in this machine code
    fn number_of_insts(&self) -> usize {
        self.code.len()
    }

    /// is the specified index a move instruction?
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
            }
            None => false
        }
    }

    /// is the specified index using memory operands?
    fn is_using_mem_op(&self, index: usize) -> bool {
        self.code[index].is_mem_op_used
    }

    /// is the specified index a jump instruction? (unconditional jump)
    /// returns an Option for target block
    fn is_jmp(&self, index: usize) -> Option<MuName> {
        let inst = self.code.get(index);
        match inst {
            Some(inst) if inst.code.starts_with("jmp") => {
                let split: Vec<&str> = inst.code.split(' ').collect();

                Some(demangle_name(String::from(split[1])))
            }
            _ => None
        }
    }

    /// is the specified index a label? returns an Option for the label
    fn is_label(&self, index: usize) -> Option<MuName> {
        let inst = self.code.get(index);
        match inst {
            Some(inst) if inst.code.ends_with(':') => {
                let split: Vec<&str> = inst.code.split(':').collect();

                Some(demangle_name(String::from(split[0])))
            }
            _ => None
        }
    }

    /// is the specified index loading a spilled register?
    /// returns an Option for the register that is loaded into
    fn is_spill_load(&self, index: usize) -> Option<P<Value>> {
        if let Some(inst) = self.code.get(index) {
            match inst.spill_info {
                Some(SpillMemInfo::Load(ref p)) => Some(p.clone()),
                _ => None
            }
        } else {
            None
        }
    }

    /// is the specified index storing a spilled register?
    /// returns an Option for the register that is stored
    fn is_spill_store(&self, index: usize) -> Option<P<Value>> {
        if let Some(inst) = self.code.get(index) {
            match inst.spill_info {
                Some(SpillMemInfo::Store(ref p)) => Some(p.clone()),
                _ => None
            }
        } else {
            None
        }
    }

    /// gets successors of a specified index
    fn get_succs(&self, index: usize) -> &Vec<usize> {
        &self.code[index].succs
    }

    /// gets predecessors of a specified index
    fn get_preds(&self, index: usize) -> &Vec<usize> {
        &self.code[index].preds
    }

    /// gets the register uses of a specified index
    fn get_inst_reg_uses(&self, index: usize) -> Vec<MuID> {
        self.code[index].uses.keys().map(|x| *x).collect()
    }

    /// gets the register defines of a specified index
    fn get_inst_reg_defines(&self, index: usize) -> Vec<MuID> {
        self.code[index].defines.keys().map(|x| *x).collect()
    }

    /// replace a temp with a machine register (to_reg must be a machine register)
    fn replace_reg(&mut self, from: MuID, to: MuID) {
        // replace defines
        for loc in self.get_define_locations(from) {
            let ref mut inst_to_patch = self.code[loc.line];

            // pick the right reg based on length
            let to_reg = x86_64::get_alias_for_length(to, loc.oplen);
            let to_reg_tag = to_reg.name();
            let to_reg_string = "%".to_string() + &to_reg_tag;

            string_utils::replace(
                &mut inst_to_patch.code,
                loc.index,
                &to_reg_string,
                to_reg_string.len()
            );
        }

        // replace uses
        for loc in self.get_use_locations(from) {
            let ref mut inst_to_patch = self.code[loc.line];

            // pick the right reg based on length
            let to_reg = x86_64::get_alias_for_length(to, loc.oplen);
            let to_reg_tag = to_reg.name();
            let to_reg_string = "%".to_string() + &to_reg_tag;

            string_utils::replace(
                &mut inst_to_patch.code,
                loc.index,
                &to_reg_string,
                to_reg_string.len()
            );
        }
    }

    /// replace a temp that is defined in the inst with another temp
    fn replace_define_tmp_for_inst(&mut self, from: MuID, to: MuID, inst: usize) {
        let to_reg_string: MuName = REG_PLACEHOLDER.clone();

        let asm = &mut self.code[inst];
        // if this reg is defined, replace the define
        if asm.defines.contains_key(&from) {
            let define_locs = asm.defines.get(&from).unwrap().to_vec();
            // replace temps
            for loc in define_locs.iter() {
                string_utils::replace(
                    &mut asm.code,
                    loc.index,
                    &to_reg_string,
                    to_reg_string.len()
                );
            }

            // remove old key, insert new one
            asm.defines.remove(&from);
            asm.defines.insert(to, define_locs);
        }
    }

    /// replace a temp that is used in the inst with another temp
    fn replace_use_tmp_for_inst(&mut self, from: MuID, to: MuID, inst: usize) {
        let to_reg_string: MuName = REG_PLACEHOLDER.clone();

        let asm = &mut self.code[inst];

        // if this reg is used, replace the use
        if asm.uses.contains_key(&from) {
            let use_locs = asm.uses.get(&from).unwrap().to_vec();
            // replace temps
            for loc in use_locs.iter() {
                string_utils::replace(
                    &mut asm.code,
                    loc.index,
                    &to_reg_string,
                    to_reg_string.len()
                );
            }

            // remove old key, insert new one
            asm.uses.remove(&from);
            asm.uses.insert(to, use_locs);
        }
    }

    /// replace destination for a jump instruction
    fn replace_branch_dest(&mut self, inst: usize, new_dest: &str, succ: MuID) {
        {
            let asm = &mut self.code[inst];

            asm.code = format!(
                "jmp {}",
                symbol(&mangle_name(Arc::new(new_dest.to_string())))
            );
            asm.succs.clear();
            asm.succs.push(succ);
        }
        {
            let asm = &mut self.code[succ];

            if !asm.preds.contains(&inst) {
                asm.preds.push(inst);
            }
        }
    }

    /// set an instruction as nop
    fn set_inst_nop(&mut self, index: usize) {
        self.code[index].code.clear();
    }

    /// is the specified index is a nop?
    fn is_nop(&self, index: usize) -> bool {
        let ref inst = self.code[index];
        if inst.code == "" || inst.code == "nop" {
            true
        } else {
            false
        }
    }

    /// remove unnecessary push/pop if the callee saved register is not used
    /// returns what registers push/pop have been deleted, and the number of callee saved registers
    /// that weren't deleted
    fn remove_unnecessary_callee_saved(&mut self, used_callee_saved: Vec<MuID>) -> HashSet<MuID> {
        // we always save rbp
        let rbp = x86_64::RBP.extract_ssa_id().unwrap();

        let find_op_other_than_rbp = |inst: &ASMInst| -> MuID {
            for id in inst.defines.keys() {
                if *id != rbp {
                    return *id;
                }
            }
            for id in inst.uses.keys() {
                if *id != rbp {
                    return *id;
                }
            }
            panic!("Expected to find a used register other than the rbp");
        };

        let mut inst_to_remove = vec![];
        let mut regs_to_remove = HashSet::new();

        for i in 0..self.number_of_insts() {
            let ref inst = self.code[i];
            match inst.spill_info {
                Some(SpillMemInfo::CalleeSaved) => {
                    let reg = find_op_other_than_rbp(inst);
                    if !used_callee_saved.contains(&reg) {
                        trace!(
                            "removing instruction {:?} for save/restore \
                             unnecessary callee saved regs",
                            inst
                        );
                        regs_to_remove.insert(reg);
                        inst_to_remove.push(i);
                    }
                }
                _ => {}
            }
        }

        for i in inst_to_remove {
            self.set_inst_nop(i);
        }

        regs_to_remove
    }

    /// patch frame size
    fn patch_frame_size(&mut self, size: usize) {
        let size = size.to_string();
        assert!(size.len() <= FRAME_SIZE_PLACEHOLDER_LEN);

        for loc in self.frame_size_patchpoints.iter() {
            let ref mut inst = self.code[loc.line];
            string_utils::replace(&mut inst.code, loc.index, &size, size.len());
        }
    }

    /// emit the machine code as a byte array
    fn emit(&self) -> Vec<u8> {
        let mut ret = vec![];

        for inst in self.code.iter() {
            if !inst.is_symbol {
                ret.append(&mut "\t".to_string().into_bytes());
            }

            ret.append(&mut inst.code.clone().into_bytes());
            ret.append(&mut "\n".to_string().into_bytes());
        }

        ret
    }

    /// emit the machine instruction at the given index as a byte array
    fn emit_inst(&self, index: usize) -> Vec<u8> {
        let mut ret = vec![];

        let ref inst = self.code[index];

        if !inst.is_symbol {
            ret.append(&mut "\t".to_string().into_bytes());
        }

        ret.append(&mut inst.code.clone().into_bytes());

        ret
    }

    /// print the whole machine code by trace level log
    fn trace_mc(&self) {
        trace!("");
        trace!("code for {}: \n", self.name);

        let n_insts = self.code.len();
        for i in 0..n_insts {
            self.trace_inst(i);
        }

        trace!("")
    }

    /// print an inst for the given index
    fn trace_inst(&self, i: usize) {
        trace!(
            "#{}\t{:60}\t\tdefine: {:?}\tuses: {:?}\tpred: {:?}\tsucc: {:?}",
            i,
            demangle_text(&self.code[i].code),
            self.get_inst_reg_defines(i),
            self.get_inst_reg_uses(i),
            self.code[i].preds,
            self.code[i].succs
        );
    }

    /// gets block livein
    fn get_ir_block_livein(&self, block: &str) -> Option<&Vec<MuID>> {
        match self.blocks.get(&block.to_string()) {
            Some(ref block) => Some(&block.livein),
            None => None
        }
    }

    /// gets block liveout
    fn get_ir_block_liveout(&self, block: &str) -> Option<&Vec<MuID>> {
        match self.blocks.get(&block.to_string()) {
            Some(ref block) => Some(&block.liveout),
            None => None
        }
    }

    /// sets block livein
    fn set_ir_block_livein(&mut self, block: &str, set: Vec<MuID>) {
        let block = self.blocks.get_mut(&block.to_string()).unwrap();
        block.livein = set;
    }

    /// sets block liveout
    fn set_ir_block_liveout(&mut self, block: &str, set: Vec<MuID>) {
        let block = self.blocks.get_mut(&block.to_string()).unwrap();
        block.liveout = set;
    }

    /// gets all the blocks
    fn get_all_blocks(&self) -> Vec<MuName> {
        self.blocks.keys().map(|x| x.clone()).collect()
    }

    /// gets the entry block
    fn get_entry_block(&self) -> MuName {
        self.entry.clone()
    }

    /// gets the range of a given block, returns [start_inst, end_inst) (end_inst not included)
    fn get_block_range(&self, block: &str) -> Option<ops::Range<usize>> {
        match self.blocks.get(&block.to_string()) {
            Some(ref block) => Some(block.start_inst..block.end_inst),
            None => None
        }
    }

    /// gets the block for a given index, returns an Option for the block
    fn get_block_for_inst(&self, index: usize) -> Option<MuName> {
        for (name, block) in self.blocks.iter() {
            if index >= block.start_inst && index < block.end_inst {
                return Some(name.clone());
            }
        }
        None
    }

    /// gets the next instruction of a specified index (labels are not instructions)
    fn get_next_inst(&self, index: usize) -> Option<usize> {
        ASMCode::find_next_inst(index, &self.code)
    }

    /// gets the previous instruction of a specified index (labels are not instructions)
    fn get_last_inst(&self, index: usize) -> Option<usize> {
        ASMCode::find_last_inst(index, &self.code)
    }
}

impl ASMInst {
    /// creates a symbolic assembly code (not an instruction)
    fn symbolic(line: String) -> ASMInst {
        ASMInst {
            code: line,
            defines: LinkedHashMap::new(),
            uses: LinkedHashMap::new(),
            is_mem_op_used: false,
            is_symbol: true,
            preds: vec![],
            succs: vec![],
            branch: ASMBranchTarget::None,

            spill_info: None
        }
    }

    /// creates an instruction
    fn inst(
        inst: String,
        defines: LinkedHashMap<MuID, Vec<ASMLocation>>,
        uses: LinkedHashMap<MuID, Vec<ASMLocation>>,
        is_mem_op_used: bool,
        target: ASMBranchTarget,
        spill_info: Option<SpillMemInfo>
    ) -> ASMInst {
        ASMInst {
            code: inst,
            defines: defines,
            uses: uses,
            is_symbol: false,
            is_mem_op_used: is_mem_op_used,
            preds: vec![],
            succs: vec![],
            branch: target,

            spill_info: spill_info
        }
    }

    /// creates a nop instruction
    fn nop() -> ASMInst {
        ASMInst {
            code: "".to_string(),
            defines: LinkedHashMap::new(),
            uses: LinkedHashMap::new(),
            is_symbol: false,
            is_mem_op_used: false,
            preds: vec![],
            succs: vec![],
            branch: ASMBranchTarget::None,

            spill_info: None
        }
    }
}

impl ASMLocation {
    fn new(line: usize, index: usize, len: usize, oplen: usize) -> ASMLocation {
        ASMLocation {
            line: line,
            index: index,
            len: len,
            oplen: oplen
        }
    }
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

/// ASMCodeGen is the assembly backend that implements CodeGenerator.
pub struct ASMCodeGen {
    cur: Option<Box<ASMCode>>
}

/// placeholder in assembly code for a temporary
const REG_PLACEHOLDER_LEN: usize = 5;
lazy_static! {
    pub static ref REG_PLACEHOLDER : MuName = {
        let blank_spaces = [' ' as u8; REG_PLACEHOLDER_LEN];
        Arc::new(format!("%{}", str::from_utf8(&blank_spaces).unwrap()))
    };
}

/// placeholder in assembly code for a frame size
//  this is a fairly random number, but a frame is something smaller than 10^10
const FRAME_SIZE_PLACEHOLDER_LEN: usize = 10;
lazy_static! {
    pub static ref FRAME_SIZE_PLACEHOLDER : String = {
        let blank_spaces = [' ' as u8; FRAME_SIZE_PLACEHOLDER_LEN];
        format!("{}", str::from_utf8(&blank_spaces).unwrap())
    };
}

impl ASMCodeGen {
    pub fn new() -> ASMCodeGen {
        ASMCodeGen { cur: None }
    }

    /// returns a reference to current assembly code that is being constructed
    fn cur(&self) -> &ASMCode {
        self.cur.as_ref().unwrap()
    }

    /// returns a mutable reference to current assembly code that is being constructed
    fn cur_mut(&mut self) -> &mut ASMCode {
        self.cur.as_mut().unwrap()
    }

    /// returns current line number (also the index for next instruction)
    fn line(&self) -> usize {
        self.cur().code.len()
    }

    /// starst a block
    fn start_block_internal(&mut self, block_name: MuName) {
        self.cur_mut()
            .blocks
            .insert(block_name.clone(), ASMBlock::new());
        let start = self.line();
        self.cur_mut()
            .blocks
            .get_mut(&block_name)
            .unwrap()
            .start_inst = start;
    }

    /// appends .global to current code
    fn add_asm_global_label(&mut self, label: String) {
        self.add_asm_symbolic(directive_globl(label.clone()));
        self.add_asm_label(label);
    }

    /// appends .equiv to current code
    fn add_asm_global_equiv(&mut self, name: String, target: String) {
        self.add_asm_symbolic(directive_globl(name.clone()));
        self.add_asm_symbolic(directive_equiv(name, target));
    }

    /// appends an label to current code
    fn add_asm_label(&mut self, label: String) {
        self.add_asm_symbolic(format!("{}:", label));
    }

    /// appends a symbolic assembly to current node
    fn add_asm_symbolic(&mut self, code: String) {
        self.cur_mut().code.push(ASMInst::symbolic(code));
    }

    /// appends a call instruction. In this instruction:
    /// * return registers are defined
    /// * caller saved registers are defined
    /// * user supplied registers
    fn add_asm_call(
        &mut self,
        code: String,
        potentially_excepting: Option<MuName>,
        use_vec: Vec<P<Value>>,
        def_vec: Vec<P<Value>>,
        target: Option<(MuID, ASMLocation)>
    ) {
        let mut uses: LinkedHashMap<MuID, Vec<ASMLocation>> = LinkedHashMap::new();
        if target.is_some() {
            let (id, loc) = target.unwrap();
            uses.insert(id, vec![loc]);
        }
        for u in use_vec {
            uses.insert(u.id(), vec![]);
        }

        let mut defines: LinkedHashMap<MuID, Vec<ASMLocation>> = LinkedHashMap::new();
        for d in def_vec {
            defines.insert(d.id(), vec![]);
        }

        self.add_asm_inst_internal(
            code,
            defines,
            uses,
            false,
            {
                if potentially_excepting.is_some() {
                    ASMBranchTarget::PotentiallyExcepting(potentially_excepting.unwrap())
                } else {
                    ASMBranchTarget::None
                }
            },
            None
        )
    }


    /// appends a return instruction
    fn add_asm_ret(&mut self, code: String) {
        // return instruction does not use anything (not RETURN REGS)
        // otherwise it will keep RETURN REGS alive
        // and if there is no actual move into RETURN REGS, it will keep RETURN REGS for alive
        // for very long and prevents anything using those registers
        self.add_asm_inst_internal(
            code,
            linked_hashmap!{},
            linked_hashmap!{},
            false,
            ASMBranchTarget::Return,
            None
        );
    }

    /// appends an unconditional branch instruction
    fn add_asm_branch(&mut self, code: String, target: MuName) {
        self.add_asm_inst_internal(
            code,
            linked_hashmap!{},
            linked_hashmap!{},
            false,
            ASMBranchTarget::Unconditional(target),
            None
        );
    }

    /// appends a conditional branch instruction
    fn add_asm_branch2(&mut self, code: String, target: MuName) {
        self.add_asm_inst_internal(
            code,
            linked_hashmap!{},
            linked_hashmap!{},
            false,
            ASMBranchTarget::Conditional(target),
            None
        );
    }

    /// appends a general non-branching instruction
    fn add_asm_inst(
        &mut self,
        code: String,
        defines: LinkedHashMap<MuID, Vec<ASMLocation>>,
        uses: LinkedHashMap<MuID, Vec<ASMLocation>>,
        is_using_mem_op: bool
    ) {
        self.add_asm_inst_internal(
            code,
            defines,
            uses,
            is_using_mem_op,
            ASMBranchTarget::None,
            None
        )
    }

    /// appends an instruction that stores/loads callee saved registers
    fn add_asm_inst_with_callee_saved(
        &mut self,
        code: String,
        defines: LinkedHashMap<MuID, Vec<ASMLocation>>,
        uses: LinkedHashMap<MuID, Vec<ASMLocation>>,
        is_using_mem_op: bool
    ) {
        self.add_asm_inst_internal(
            code,
            defines,
            uses,
            is_using_mem_op,
            ASMBranchTarget::None,
            Some(SpillMemInfo::CalleeSaved)
        )
    }

    /// appends an instruction that stores/loads spilled registers
    fn add_asm_inst_with_spill(
        &mut self,
        code: String,
        defines: LinkedHashMap<MuID, Vec<ASMLocation>>,
        uses: LinkedHashMap<MuID, Vec<ASMLocation>>,
        is_using_mem_op: bool,
        spill_info: SpillMemInfo
    ) {
        self.add_asm_inst_internal(
            code,
            defines,
            uses,
            is_using_mem_op,
            ASMBranchTarget::None,
            Some(spill_info)
        )
    }

    /// internal function to append any instruction
    fn add_asm_inst_internal(
        &mut self,
        code: String,
        defines: LinkedHashMap<MuID, Vec<ASMLocation>>,
        uses: LinkedHashMap<MuID, Vec<ASMLocation>>,
        is_using_mem_op: bool,
        target: ASMBranchTarget,
        spill_info: Option<SpillMemInfo>
    ) {
        let line = self.line();
        trace!("asm: {}", demangle_text(&code));
        trace!("     defines: {:?}", defines);
        trace!("     uses: {:?}", uses);
        let mc = self.cur_mut();

        // put the instruction
        mc.code.push(ASMInst::inst(
            code,
            defines,
            uses,
            is_using_mem_op,
            target,
            spill_info
        ));
    }

    /// prepares information for a temporary/register, returns (name, ID, location)
    fn prepare_reg(&self, op: &P<Value>, loc: usize) -> (String, MuID, ASMLocation) {
        debug_assert!(op.is_reg());
        let str = self.asm_reg_op(op);
        let len = str.len();
        (
            str,
            op.extract_ssa_id().unwrap(),
            ASMLocation::new(self.line(), loc, len, check_op_len(op))
        )
    }

    /// prepares information for a floatingpoint temporary/register, returns (name, ID, location)
    fn prepare_fpreg(&self, op: &P<Value>, loc: usize) -> (String, MuID, ASMLocation) {
        debug_assert!(op.is_reg());
        let str = self.asm_reg_op(op);
        let len = str.len();
        (
            str,
            op.extract_ssa_id().unwrap(),
            ASMLocation::new(self.line(), loc, len, 64)
        )
    }

    /// prepares information for a machine register, returns ID
    fn prepare_machine_reg(&self, op: &P<Value>) -> MuID {
        debug_assert!(op.is_reg());
        op.extract_ssa_id().unwrap()
    }

    /// prepares information for a collection of machine registers, returns IDs
    fn prepare_machine_regs(&self, regs: Iter<P<Value>>) -> Vec<MuID> {
        regs.map(|x| self.prepare_machine_reg(x)).collect()
    }

    /// prepares information for a memory operand, returns (operand string (as in asm),
    /// reg/tmp locations) pair
    /// This function turns memory operands into something like "offset(base, scale, index)" or
    /// "label(base)"
    #[allow(unused_assignments)]
    // we keep updating loc_cursor to be valid, but we may not read value in the end
    fn prepare_mem(
        &self,
        op: &P<Value>,
        loc: usize
    ) -> (String, LinkedHashMap<MuID, Vec<ASMLocation>>) {
        debug_assert!(op.is_mem());

        // temps/regs used
        let mut ids: Vec<MuID> = vec![];
        // locations for temps/regs
        let mut locs: Vec<ASMLocation> = vec![];
        // resulting string for the memory operand
        let mut result_str: String = "".to_string();
        // column cursor
        let mut loc_cursor: usize = loc;

        match op.v {
            // offset(base,index,scale)
            Value_::Memory(MemoryLocation::Address {
                ref base,
                ref offset,
                ref index,
                scale
            }) => {
                // deal with offset
                if offset.is_some() {
                    let offset = offset.as_ref().unwrap();
                    match offset.v {
                        Value_::SSAVar(_) => {
                            // temp as offset
                            let (str, id, loc) = self.prepare_reg(offset, loc_cursor);

                            result_str.push_str(&str);
                            ids.push(id);
                            locs.push(loc);
                            loc_cursor += str.len();
                        }
                        Value_::Constant(Constant::Int(val)) => {
                            let str = (val as i32).to_string();

                            result_str.push_str(&str);
                            loc_cursor += str.len();
                        }
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
                        Value_::SSAVar(_) => {
                            // temp as offset
                            let (str, id, loc) = self.prepare_reg(index, loc_cursor);

                            result_str.push_str(&str);
                            ids.push(id);
                            locs.push(loc);
                            loc_cursor += str.len();
                        }
                        Value_::Constant(Constant::Int(val)) => {
                            let str = (val as i32).to_string();

                            result_str.push_str(&str);
                            loc_cursor += str.len();
                        }
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
            }
            Value_::Memory(MemoryLocation::Symbolic {
                ref base,
                ref label,
                is_global,
                is_native
            }) => {
                let label = if is_native {
                    "/*C*/".to_string() + label.as_str()
                } else {
                    mangle_name(label.clone())
                };
                if base.is_some() && base.as_ref().unwrap().id() == x86_64::RIP.id() && is_global {
                    // pc relative address
                    let pic_symbol = pic_symbol(&label.clone());
                    result_str.push_str(&pic_symbol);
                    loc_cursor += label.len();
                } else {
                    let symbol = symbol(&label.clone());
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
            }
            _ => panic!("expect mem location as value")
        }

        let uses: LinkedHashMap<MuID, Vec<ASMLocation>> = {
            let mut map: LinkedHashMap<MuID, Vec<ASMLocation>> = linked_hashmap!{};
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

    /// prepares information for an immediate number, returns i32 value
    fn prepare_imm(&self, op: i32, len: usize) -> i32 {
        match len {
            64 => op,
            32 => op,
            16 => op as i16 as i32, // truncate
            8 => op as i8 as i32,
            _ => unimplemented!()
        }
    }

    /// returns %NAME for a machine register, or blank placeholder for a temporary
    fn asm_reg_op(&self, op: &P<Value>) -> String {
        let id = op.extract_ssa_id().unwrap();
        if id < MACHINE_ID_END {
            // machine reg
            format!("%{}", op.name())
        } else {
            // virtual register, use place holder
            (**REG_PLACEHOLDER).clone()
        }
    }

    /// returns a unique block label from current function and label name
    fn mangle_block_label(&self, label: MuName) -> String {
        format!("{}_{}", self.cur().name, label)
    }

    /// returns label name from a mangled block label
    fn unmangle_block_label(fn_name: MuName, label: String) -> MuName {
        // input: _fn_name_BLOCK_NAME
        // return BLOCK_NAME
        let split: Vec<&str> = label.splitn(2, &((*fn_name).clone() + "_")).collect();
        Arc::new(String::from(split[1]))
    }

    /// finishes current code sequence, and returns Box<ASMCode>
    fn finish_code_sequence_asm(&mut self) -> Box<ASMCode> {
        self.cur.take().unwrap()
    }

    /// emits an instruction (use 1 reg, define none)
    fn internal_uniop_def_r(&mut self, inst: &str, op: &P<Value>) {
        trace!("emit: {} {}", inst, op);

        let (reg, id, loc) = self.prepare_reg(op, inst.len() + 1);

        let asm = format!("{} {}", inst, reg);

        self.add_asm_inst(
            asm,
            linked_hashmap!{
                id => vec![loc]
            },
            linked_hashmap!{},
            false
        )
    }

    /// emits an instruction (use 2 regs, define none)
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

    /// emits an instruction (use 1 imm 1 reg, define none)
    fn internal_binop_no_def_imm_r(&mut self, inst: &str, op1: i32, op2: &P<Value>) {
        let len = check_op_len(op2);

        let inst = inst.to_string() + &op_postfix(len);
        trace!("emit: {} {} {}", inst, op1, op2);

        let imm = self.prepare_imm(op1, len);
        let (reg2, id2, loc2) =
            self.prepare_reg(op2, inst.len() + 1 + 1 + imm.to_string().len() + 1);

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

    /// emits an instruction (use 1 mem 1 reg, define none)
    fn internal_binop_no_def_mem_r(&mut self, inst: &str, op1: &P<Value>, op2: &P<Value>) {
        let len = check_op_len(op2);

        let inst = inst.to_string() + &op_postfix(len);
        trace!("emit: {} {} {}", inst, op1, op2);

        let (mem, mut uses) = self.prepare_mem(op1, inst.len() + 1);
        let (reg, id1, loc1) = self.prepare_reg(op2, inst.len() + 1 + mem.len() + 1);

        let asm = format!("{} {},{}", inst, mem, reg);

        // merge use vec
        if uses.contains_key(&id1) {
            let mut locs = uses.get_mut(&id1).unwrap();
            vec_utils::add_unique(locs, loc1.clone());
        } else {
            uses.insert(id1, vec![loc1]);
        }

        self.add_asm_inst(asm, linked_hashmap!{}, uses, true)
    }

    /// emits an instruction (use 1 reg 1 mem, define none)
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

        self.add_asm_inst(asm, linked_hashmap!{}, uses, true)
    }

    /// emits an instruction (use 2 regs, define 1st reg)
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

    /// emits an instruction (use 1 reg 1 mreg, define 1st reg)
    fn internal_binop_def_r_mr(&mut self, inst: &str, dest: &P<Value>, src: &P<Value>) {
        let len = check_op_len(dest);

        let inst = inst.to_string() + &op_postfix(len);
        trace!("emit: {} {}, {} -> {}", inst, src, dest, dest);

        let mreg = self.prepare_machine_reg(src);
        let mreg_name = src.name();
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

    /// emits an instruction (use 1 reg 1 imm, define the reg)
    fn internal_binop_def_r_imm(&mut self, inst: &str, dest: &P<Value>, src: i32) {
        let len = check_op_len(dest);

        let inst = inst.to_string() + &op_postfix(len);
        trace!("emit: {} {}, {} -> {}", inst, src, dest, dest);

        let imm = self.prepare_imm(src, len);
        let (reg1, id1, loc1) =
            self.prepare_reg(dest, inst.len() + 1 + 1 + imm.to_string().len() + 1);

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

    /// emits an instruction (use 1 reg 1 mem, define the reg)
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

    /// emits an instruction (use 2 reg 1 mreg, define 1st reg)
    fn internal_triop_def_r_r_mr(&mut self, inst: &str, dest: Reg, src1: Reg, src2: Reg) {
        let len = check_op_len(dest);

        let inst = inst.to_string() + &op_postfix(len);
        trace!("emit: {} {}, {}, {} -> {}", inst, dest, src1, src2, dest);

        let mreg = self.prepare_machine_reg(src2);
        let mreg_name = src2.name();

        let (reg1, id1, loc1) = self.prepare_reg(src1, inst.len() + 1 + 1 + mreg_name.len() + 1);
        let (reg2, id2, loc2) = self.prepare_reg(
            dest,
            inst.len() + 1 + 1 + mreg_name.len() + 1 + reg1.len() + 1
        );

        let asm = format!("{} %{},{},{}", inst, mreg_name, reg1, reg2);

        self.add_asm_inst(
            asm,
            linked_hashmap!{
                id2 => vec![loc2.clone()]
            },
            {
                if id1 == id2 {
                    linked_hashmap! {
                        id1 => vec![loc1, loc2],
                        mreg => vec![]
                    }
                } else {
                    linked_hashmap! {
                        id1 => vec![loc1],
                        id2 => vec![loc2],
                        mreg => vec![]
                    }
                }
            },
            false
        )
    }

    /// emits a move instruction (imm64 -> reg64)
    fn internal_mov_r64_imm64(&mut self, inst: &str, dest: &P<Value>, src: i64) {
        let inst = inst.to_string() + &op_postfix(64);
        trace!("emit: {} {} -> {}", inst, src, dest);

        let (reg1, id1, loc1) =
            self.prepare_reg(dest, inst.len() + 1 + 1 + src.to_string().len() + 1);

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

    /// emits a move instruction (reg64/32 -> fpr)
    fn internal_mov_bitcast_fpr_r(&mut self, inst: &str, dest: &P<Value>, src: &P<Value>) {
        trace!("emit: {} {} -> {}", inst, src, dest);

        let (reg1, id1, loc1) = self.prepare_reg(src, inst.len() + 1);
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

    /// emits a move instruction (fpr -> reg64/32)
    fn internal_mov_bitcast_r_fpr(&mut self, inst: &str, dest: &P<Value>, src: &P<Value>) {
        trace!("emit: {} {} -> {}", inst, src, dest);

        let (reg1, id1, loc1) = self.prepare_fpreg(src, inst.len() + 1);
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

    /// emits a move instruction (reg -> reg)
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

    /// emits a move instruction (imm -> reg)
    fn internal_mov_r_imm(&mut self, inst: &str, dest: &P<Value>, src: i32) {
        let len = check_op_len(dest);

        let inst = inst.to_string() + &op_postfix(len);
        trace!("emit: {} {} -> {}", inst, src, dest);

        let imm = self.prepare_imm(src, len);
        let (reg1, id1, loc1) =
            self.prepare_reg(dest, inst.len() + 1 + 1 + imm.to_string().len() + 1);

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

    /// emits a move instruction (mem -> reg), i.e. load instruction
    fn internal_mov_r_mem(
        &mut self,
        inst: &str,
        dest: Reg,
        src: Mem,
        is_spill_related: bool,
        is_callee_saved: bool
    ) {
        let len = check_op_len(dest);

        let inst = inst.to_string() + &op_postfix(len);
        trace!("emit: {} {} -> {}", inst, src, dest);

        let (mem, uses) = self.prepare_mem(src, inst.len() + 1);
        let (reg, id2, loc2) = self.prepare_reg(dest, inst.len() + 1 + mem.len() + 1);

        let asm = format!("{} {},{}", inst, mem, reg);

        if is_callee_saved {
            self.add_asm_inst_with_callee_saved(
                asm,
                linked_hashmap!{
                    id2 => vec![loc2]
                },
                uses,
                true
            )
        } else if is_spill_related {
            self.add_asm_inst_with_spill(
                asm,
                linked_hashmap!{
                    id2 => vec![loc2]
                },
                uses,
                true,
                SpillMemInfo::Load(src.clone())
            )
        } else {
            self.add_asm_inst(
                asm,
                linked_hashmap! {
                id2 => vec![loc2]
            },
                uses,
                true
            )
        }
    }

    /// emits a move instruction (reg -> mem), i.e. store instruction
    fn internal_mov_mem_r(
        &mut self,
        inst: &str,
        dest: Mem,
        src: Reg,
        is_spill_related: bool,
        is_callee_saved: bool
    ) {
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

        if is_callee_saved {
            self.add_asm_inst_with_callee_saved(asm, linked_hashmap!{}, uses, true)
        } else if is_spill_related {
            self.add_asm_inst_with_spill(
                asm,
                linked_hashmap!{},
                uses,
                true,
                SpillMemInfo::Store(dest.clone())
            )
        } else {
            self.add_asm_inst(asm, linked_hashmap!{}, uses, true)
        }
    }

    /// emits a move instruction (imm -> mem), i.e. store instruction
    fn internal_mov_mem_imm(&mut self, inst: &str, dest: &P<Value>, src: i32, len: usize) {
        let inst = inst.to_string() + &op_postfix(len);
        trace!("emit: {} {} -> {}", inst, src, dest);

        let imm = self.prepare_imm(src, len);
        let (mem, uses) = self.prepare_mem(dest, inst.len() + 1 + 1 + imm.to_string().len() + 1);

        let asm = format!("{} ${},{}", inst, imm, mem);

        self.add_asm_inst(asm, linked_hashmap!{}, uses, true)
    }

    /// emits a move instruction (fpreg -> fpreg)
    fn internal_fp_mov_f_f(&mut self, inst: &str, dest: Reg, src: Reg) {
        trace!("emit: {} {} -> {}", inst, src, dest);

        let (reg1, id1, loc1) = self.prepare_fpreg(src, inst.len() + 1);
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

    /// emits a move instruction (mem -> fpreg), i.e. load instruction
    fn internal_fp_mov_f_mem(&mut self, inst: &str, dest: Reg, src: Mem, is_spill_related: bool) {
        trace!("emit: {} {} -> {}", inst, src, dest);

        let (mem, uses) = self.prepare_mem(src, inst.len() + 1);
        let (reg, id2, loc2) = self.prepare_fpreg(dest, inst.len() + 1 + mem.len() + 1);

        let asm = format!("{} {},{}", inst, mem, reg);

        if is_spill_related {
            self.add_asm_inst_with_spill(
                asm,
                linked_hashmap!{
                    id2 => vec![loc2]
                },
                uses,
                true,
                SpillMemInfo::Load(src.clone())
            )
        } else {
            self.add_asm_inst(
                asm,
                linked_hashmap! {
                id2 => vec![loc2]
            },
                uses,
                true
            )
        }
    }

    /// emits a move instruction (fpreg -> mem), i.e. store instruction
    fn internal_fp_mov_mem_f(&mut self, inst: &str, dest: Mem, src: Reg, is_spill_related: bool) {
        trace!("emit: {} {} -> {}", inst, src, dest);

        let (reg, id1, loc1) = self.prepare_fpreg(src, inst.len() + 1);
        let (mem, mut uses) = self.prepare_mem(dest, inst.len() + 1 + reg.len() + 1);

        // the register we used for the memory location is counted as 'use'
        // use the vec from mem as 'use' (push use reg from src to it)
        if uses.contains_key(&id1) {
            uses.get_mut(&id1).unwrap().push(loc1);
        } else {
            uses.insert(id1, vec![loc1]);
        }

        let asm = format!("{} {},{}", inst, reg, mem);

        if is_spill_related {
            self.add_asm_inst_with_spill(
                asm,
                linked_hashmap!{},
                uses,
                true,
                SpillMemInfo::Store(dest.clone())
            )
        } else {
            self.add_asm_inst(asm, linked_hashmap!{}, uses, true)
        }
    }

    /// emits an instruction (use 2 fpregs, define none)
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

    /// emits an instruction (use 2 fpregs, define 1st fpreg)
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

    /// emits an instruction (use 2 fpregs, define 1st fpreg)
    fn internal_fp_binop_def_r_mem(&mut self, inst: &str, dest: Reg, src: Reg) {
        trace!("emit: {} {}, {} -> {}", inst, src, dest, dest);
        unimplemented!()
    }

    /// emits a move instruction (reg -> fpreg)
    fn internal_gpr_to_fpr(&mut self, inst: &str, dest: Reg, src: Reg) {
        let len = check_op_len(src);

        let inst = inst.to_string() + &op_postfix(len);
        trace!("emit: {} {} -> {}", inst, src, dest);

        let (reg1, id1, loc1) = self.prepare_reg(src, inst.len() + 1);
        let (reg2, id2, loc2) = self.prepare_fpreg(dest, inst.len() + 1 + reg1.len() + 1);

        let asm = format!("{} {}, {}", inst, reg1, reg2);

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

    /// emits a move instruction (fpreg -> reg)
    fn internal_fpr_to_gpr(&mut self, inst: &str, dest: Reg, src: Reg) {
        let len = check_op_len(dest);

        let inst = inst.to_string() + &op_postfix(len);
        trace!("emit: {} {} -> {}", inst, src, dest);

        let (reg1, id1, loc1) = self.prepare_fpreg(src, inst.len() + 1);
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

    /// emits a truncate instruction (fpreg -> fpreg)
    fn internal_fp_trunc(&mut self, inst: &str, dest: Reg, src: Reg) {
        let inst = inst.to_string();
        trace!("emit: {} {} -> {}", inst, src, dest);

        let (reg1, id1, loc1) = self.prepare_fpreg(src, inst.len() + 1);
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

    /// emits a store instruction to store a spilled register
    fn emit_spill_store_gpr(&mut self, dest: Mem, src: Reg) {
        self.internal_mov_mem_r("mov", dest, src, true, false)
    }

    /// emits a load instruction to load a spilled register
    fn emit_spill_load_gpr(&mut self, dest: Reg, src: Mem) {
        self.internal_mov_r_mem("mov", dest, src, true, false)
    }

    /// emits a store instruction to store a spilled floating point register
    fn emit_spill_store_fpr(&mut self, dest: Mem, src: Reg) {
        self.internal_fp_mov_mem_f("movsd", dest, src, true)
    }

    /// emits a load instruction to load a spilled floating point register
    fn emit_spill_load_fpr(&mut self, dest: Reg, src: Mem) {
        self.internal_fp_mov_f_mem("movsd", dest, src, true)
    }
}

/// returns postfix for instruction based on operand length (b for 8 bits, w for 16 bits, etc.)
#[inline(always)]
fn op_postfix(op_len: usize) -> &'static str {
    match op_len {
        8 => "b",
        16 => "w",
        32 => "l",
        64 => "q",
        _ => panic!("unexpected op size: {}", op_len)
    }
}

impl CodeGenerator for ASMCodeGen {
    fn start_code(&mut self, func_name: MuName, entry: MuName) -> ValueLocation {
        self.cur = Some(Box::new(ASMCode {
            name: func_name.clone(),
            entry: entry,
            code: vec![],
            blocks: linked_hashmap!{},
            frame_size_patchpoints: vec![]
        }));

        // to link with C sources via gcc
        let func_symbol = symbol(&mangle_name(func_name.clone()));
        self.add_asm_global_label(func_symbol.clone());
        if is_valid_c_identifier(&func_name) {
            self.add_asm_global_equiv(symbol(&func_name.clone()), func_symbol);
        }

        ValueLocation::Relocatable(RegGroup::GPR, func_name)
    }

    fn finish_code(
        &mut self,
        func_name: MuName
    ) -> (Box<MachineCode + Sync + Send>, ValueLocation) {
        let func_end = {
            let mut symbol = (*func_name).clone();
            symbol.push_str(":end");
            Arc::new(symbol)
        };
        self.add_asm_global_label(symbol(&mangle_name(func_end.clone())));

        self.cur.as_mut().unwrap().control_flow_analysis();
        (
            self.cur.take().unwrap(),
            ValueLocation::Relocatable(RegGroup::GPR, func_end)
        )
    }

    fn start_code_sequence(&mut self) {
        self.cur = Some(Box::new(ASMCode {
            name: Arc::new("snippet".to_string()),
            entry: Arc::new("none".to_string()),
            code: vec![],
            blocks: linked_hashmap!{},
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
        self.add_asm_label(symbol(&mangle_name(block_name.clone())));
        self.start_block_internal(block_name);
    }

    fn start_exception_block(&mut self, block_name: MuName) -> ValueLocation {
        self.add_asm_global_label(symbol(&mangle_name(block_name.clone())));
        self.start_block_internal(block_name.clone());

        ValueLocation::Relocatable(RegGroup::GPR, block_name)
    }

    fn end_block(&mut self, block_name: MuName) {
        let line = self.line();
        match self.cur_mut().blocks.get_mut(&block_name) {
            Some(ref mut block) => {
                block.end_inst = line;
            }
            None => {
                panic!(
                    "trying to end block {} which hasnt been started",
                    block_name
                )
            }
        }
    }

    fn add_cfi_startproc(&mut self) {
        self.add_asm_symbolic(".cfi_startproc".to_string());
    }
    fn add_cfi_endproc(&mut self) {
        self.add_asm_symbolic(".cfi_endproc".to_string());
    }

    fn add_cfi_def_cfa_register(&mut self, reg: Reg) {
        let reg = self.asm_reg_op(reg);
        self.add_asm_symbolic(format!(".cfi_def_cfa_register {}", reg));
    }
    fn add_cfi_def_cfa_offset(&mut self, offset: i32) {
        self.add_asm_symbolic(format!(".cfi_def_cfa_offset {}", offset));
    }
    fn add_cfi_offset(&mut self, reg: Reg, offset: i32) {
        let reg = self.asm_reg_op(reg);
        self.add_asm_symbolic(format!(".cfi_offset {}, {}", reg, offset));
    }

    /// emits code to grow frame size (size is unknown at this point, use a placeholder)
    fn emit_frame_grow(&mut self) {
        trace!("emit frame grow");

        let asm = format!("addq $-{},%rsp", FRAME_SIZE_PLACEHOLDER.clone());

        // record the placeholder position so we can patch it later
        let line = self.line();
        self.cur_mut()
            .add_frame_size_patchpoint(ASMLocation::new(line, 7, FRAME_SIZE_PLACEHOLDER_LEN, 0));

        self.add_asm_inst(
            asm,
            linked_hashmap!{}, // let reg alloc ignore this instruction
            linked_hashmap!{},
            false
        )
    }

    fn emit_nop(&mut self, bytes: usize) {
        trace!("emit: nop ({} bytes)", bytes);

        let asm = String::from("nop");

        self.add_asm_inst(asm, linked_hashmap!{}, linked_hashmap!{}, false);
    }

    // cmp

    fn emit_cmp_r_r(&mut self, op1: &P<Value>, op2: &P<Value>) {
        self.internal_binop_no_def_r_r("cmp", op1, op2)
    }

    fn emit_cmp_imm_r(&mut self, op1: i32, op2: &P<Value>) {
        self.internal_binop_no_def_imm_r("cmp", op1, op2)
    }

    fn emit_cmp_mem_r(&mut self, op1: &P<Value>, op2: &P<Value>) {
        self.internal_binop_no_def_mem_r("cmp", op1, op2)
    }

    fn emit_test_r_r(&mut self, op1: &P<Value>, op2: &P<Value>) {
        self.internal_binop_no_def_r_r("test", op1, op2)
    }

    fn emit_test_imm_r(&mut self, op1: i32, op2: Reg) {
        self.internal_binop_no_def_imm_r("test", op1, op2)
    }

    // mov

    fn emit_mov_r64_imm64(&mut self, dest: &P<Value>, src: i64) {
        self.internal_mov_r64_imm64("mov", dest, src)
    }

    fn emit_mov_fpr_r64(&mut self, dest: Reg, src: Reg) {
        self.internal_mov_bitcast_fpr_r("movq", dest, src)
    }

    fn emit_mov_fpr_r32(&mut self, dest: Reg, src: Reg) {
        self.internal_mov_bitcast_fpr_r("movd", dest, src)
    }

    fn emit_mov_r64_fpr(&mut self, dest: Reg, src: Reg) {
        self.internal_mov_bitcast_r_fpr("movq", dest, src)
    }

    fn emit_mov_r32_fpr(&mut self, dest: Reg, src: Reg) {
        self.internal_mov_bitcast_r_fpr("movd", dest, src)
    }

    fn emit_mov_r_imm(&mut self, dest: &P<Value>, src: i32) {
        self.internal_mov_r_imm("mov", dest, src)
    }
    fn emit_mov_r_mem(&mut self, dest: &P<Value>, src: &P<Value>) {
        self.internal_mov_r_mem("mov", dest, src, false, false)
    }
    fn emit_mov_r_r(&mut self, dest: &P<Value>, src: &P<Value>) {
        self.internal_mov_r_r("mov", dest, src)
    }
    fn emit_mov_mem_r(&mut self, dest: &P<Value>, src: &P<Value>) {
        self.internal_mov_mem_r("mov", dest, src, false, false)
    }
    fn emit_mov_mem_imm(&mut self, dest: &P<Value>, src: i32, oplen: usize) {
        self.internal_mov_mem_imm("mov", dest, src, oplen)
    }

    fn emit_mov_r_mem_callee_saved(&mut self, dest: &P<Value>, src: &P<Value>) {
        self.internal_mov_r_mem("mov", dest, src, false, true)
    }
    fn emit_mov_mem_r_callee_saved(&mut self, dest: &P<Value>, src: &P<Value>) {
        self.internal_mov_mem_r("mov", dest, src, false, true)
    }

    // zero/sign extend mov

    fn emit_movs_r_r(&mut self, dest: Reg, src: Reg) {
        let dest_len = check_op_len(dest);
        let src_len = check_op_len(src);

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

    fn emit_movz_r_r(&mut self, dest: Reg, src: Reg) {
        let dest_len = check_op_len(dest);
        let src_len = check_op_len(src);

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

    // set byte
    fn emit_sets_r8(&mut self, dest: Reg) {
        self.internal_uniop_def_r("sets", dest)
    }
    fn emit_setz_r8(&mut self, dest: Reg) {
        self.internal_uniop_def_r("setz", dest)
    }
    fn emit_seto_r8(&mut self, dest: Reg) {
        self.internal_uniop_def_r("seto", dest)
    }
    fn emit_setb_r8(&mut self, dest: Reg) {
        self.internal_uniop_def_r("setb", dest)
    }
    fn emit_seta_r(&mut self, dest: Reg) {
        self.internal_uniop_def_r("seta", dest)
    }
    fn emit_setae_r(&mut self, dest: Reg) {
        self.internal_uniop_def_r("setae", dest)
    }
    fn emit_setb_r(&mut self, dest: Reg) {
        self.internal_uniop_def_r("setb", dest)
    }
    fn emit_setbe_r(&mut self, dest: Reg) {
        self.internal_uniop_def_r("setbe", dest)
    }
    fn emit_sete_r(&mut self, dest: Reg) {
        self.internal_uniop_def_r("sete", dest)
    }
    fn emit_setg_r(&mut self, dest: Reg) {
        self.internal_uniop_def_r("setg", dest)
    }
    fn emit_setge_r(&mut self, dest: Reg) {
        self.internal_uniop_def_r("setge", dest)
    }
    fn emit_setl_r(&mut self, dest: Reg) {
        self.internal_uniop_def_r("setl", dest)
    }
    fn emit_setle_r(&mut self, dest: Reg) {
        self.internal_uniop_def_r("setle", dest)
    }
    fn emit_setne_r(&mut self, dest: Reg) {
        self.internal_uniop_def_r("setne", dest)
    }

    // cmov src -> dest

    fn emit_cmova_r_r(&mut self, dest: &P<Value>, src: &P<Value>) {
        debug_assert!(check_op_len(dest) >= 16);
        self.internal_binop_no_def_r_r("cmova", src, dest)
    }
    fn emit_cmova_r_mem(&mut self, dest: &P<Value>, src: &P<Value>) {
        debug_assert!(check_op_len(dest) >= 16);
        self.internal_binop_no_def_r_mem("cmova", src, dest)
    }

    fn emit_cmovae_r_r(&mut self, dest: &P<Value>, src: &P<Value>) {
        debug_assert!(check_op_len(dest) >= 16);
        self.internal_binop_no_def_r_r("cmovae", src, dest)
    }
    fn emit_cmovae_r_mem(&mut self, dest: &P<Value>, src: &P<Value>) {
        debug_assert!(check_op_len(dest) >= 16);
        self.internal_binop_no_def_r_mem("cmovae", src, dest)
    }

    fn emit_cmovb_r_r(&mut self, dest: &P<Value>, src: &P<Value>) {
        debug_assert!(check_op_len(dest) >= 16);
        self.internal_binop_no_def_r_r("cmovb", src, dest)
    }
    fn emit_cmovb_r_mem(&mut self, dest: &P<Value>, src: &P<Value>) {
        debug_assert!(check_op_len(dest) >= 16);
        self.internal_binop_no_def_r_mem("cmovb", src, dest)
    }

    fn emit_cmovbe_r_r(&mut self, dest: &P<Value>, src: &P<Value>) {
        debug_assert!(check_op_len(dest) >= 16);
        self.internal_binop_no_def_r_r("cmovbe", src, dest)
    }
    fn emit_cmovbe_r_mem(&mut self, dest: &P<Value>, src: &P<Value>) {
        debug_assert!(check_op_len(dest) >= 16);
        self.internal_binop_no_def_r_mem("cmovbe", src, dest)
    }

    fn emit_cmove_r_r(&mut self, dest: &P<Value>, src: &P<Value>) {
        debug_assert!(check_op_len(dest) >= 16);
        self.internal_binop_no_def_r_r("cmove", src, dest)
    }
    fn emit_cmove_r_mem(&mut self, dest: &P<Value>, src: &P<Value>) {
        debug_assert!(check_op_len(dest) >= 16);
        self.internal_binop_no_def_r_mem("cmove", src, dest)
    }

    fn emit_cmovg_r_r(&mut self, dest: &P<Value>, src: &P<Value>) {
        debug_assert!(check_op_len(dest) >= 16);
        self.internal_binop_no_def_r_r("cmovg", src, dest)
    }
    fn emit_cmovg_r_mem(&mut self, dest: &P<Value>, src: &P<Value>) {
        debug_assert!(check_op_len(dest) >= 16);
        self.internal_binop_no_def_r_mem("cmovg", src, dest)
    }

    fn emit_cmovge_r_r(&mut self, dest: &P<Value>, src: &P<Value>) {
        debug_assert!(check_op_len(dest) >= 16);
        self.internal_binop_no_def_r_r("cmovge", src, dest)
    }
    fn emit_cmovge_r_mem(&mut self, dest: &P<Value>, src: &P<Value>) {
        debug_assert!(check_op_len(dest) >= 16);
        self.internal_binop_no_def_r_mem("cmovge", src, dest)
    }

    fn emit_cmovl_r_r(&mut self, dest: &P<Value>, src: &P<Value>) {
        debug_assert!(check_op_len(dest) >= 16);
        self.internal_binop_no_def_r_r("cmovl", src, dest)
    }
    fn emit_cmovl_r_mem(&mut self, dest: &P<Value>, src: &P<Value>) {
        debug_assert!(check_op_len(dest) >= 16);
        self.internal_binop_no_def_r_mem("cmovl", src, dest)
    }

    fn emit_cmovle_r_r(&mut self, dest: &P<Value>, src: &P<Value>) {
        debug_assert!(check_op_len(dest) >= 16);
        self.internal_binop_no_def_r_r("cmovle", src, dest)
    }
    fn emit_cmovle_r_mem(&mut self, dest: &P<Value>, src: &P<Value>) {
        debug_assert!(check_op_len(dest) >= 16);
        self.internal_binop_no_def_r_mem("cmovle", src, dest)
    }

    fn emit_cmovne_r_r(&mut self, dest: &P<Value>, src: &P<Value>) {
        debug_assert!(check_op_len(dest) >= 16);
        self.internal_binop_no_def_r_r("cmovne", src, dest)
    }
    fn emit_cmovne_r_mem(&mut self, dest: &P<Value>, src: &P<Value>) {
        debug_assert!(check_op_len(dest) >= 16);
        self.internal_binop_no_def_r_mem("cmovne", src, dest)
    }

    // lea
    fn emit_lea_r64(&mut self, dest: &P<Value>, src: &P<Value>) {
        self.internal_mov_r_mem("lea", dest, src, false, false)
    }

    // and
    fn emit_and_r_imm(&mut self, dest: Reg, src: i32) {
        self.internal_binop_def_r_imm("and", dest, src)
    }
    fn emit_and_r_r(&mut self, dest: Reg, src: Reg) {
        self.internal_binop_def_r_r("and", dest, src)
    }
    fn emit_and_r_mem(&mut self, dest: Reg, src: Mem) {
        self.internal_binop_def_r_mem("and", dest, src)
    }

    // or
    fn emit_or_r_imm(&mut self, dest: Reg, src: i32) {
        self.internal_binop_def_r_imm("or", dest, src)
    }
    fn emit_or_r_r(&mut self, dest: Reg, src: Reg) {
        self.internal_binop_def_r_r("or", dest, src)
    }
    fn emit_or_r_mem(&mut self, dest: Reg, src: Mem) {
        self.internal_binop_def_r_mem("or", dest, src)
    }

    // xor
    fn emit_xor_r_imm(&mut self, dest: Reg, src: i32) {
        self.internal_binop_def_r_imm("xor", dest, src)
    }
    fn emit_xor_r_r(&mut self, dest: Reg, src: Reg) {
        self.internal_binop_def_r_r("xor", dest, src)
    }
    fn emit_xor_r_mem(&mut self, dest: Reg, src: Mem) {
        self.internal_binop_def_r_mem("xor", dest, src)
    }

    // add
    fn emit_add_r_imm(&mut self, dest: Reg, src: i32) {
        self.internal_binop_def_r_imm("add", dest, src)
    }
    fn emit_add_r_r(&mut self, dest: Reg, src: Reg) {
        self.internal_binop_def_r_r("add", dest, src)
    }
    fn emit_add_r_mem(&mut self, dest: Reg, src: Mem) {
        self.internal_binop_def_r_mem("add", dest, src)
    }

    // adc
    fn emit_adc_r_r(&mut self, dest: Reg, src: Reg) {
        self.internal_binop_def_r_r("adc", dest, src)
    }
    fn emit_adc_r_mem(&mut self, dest: Reg, src: Mem) {
        self.internal_binop_def_r_mem("adc", dest, src)
    }
    fn emit_adc_r_imm(&mut self, dest: Reg, src: i32) {
        self.internal_binop_def_r_imm("adc", dest, src)
    }

    // sub
    fn emit_sub_r_imm(&mut self, dest: Reg, src: i32) {
        self.internal_binop_def_r_imm("sub", dest, src)
    }
    fn emit_sub_r_r(&mut self, dest: Reg, src: Reg) {
        self.internal_binop_def_r_r("sub", dest, src)
    }
    fn emit_sub_r_mem(&mut self, dest: Reg, src: Mem) {
        self.internal_binop_def_r_mem("sub", dest, src)
    }

    // sbb
    fn emit_sbb_r_r(&mut self, dest: Reg, src: Reg) {
        self.internal_binop_def_r_r("sbb", dest, src)
    }
    fn emit_sbb_r_mem(&mut self, dest: Reg, src: Mem) {
        self.internal_binop_def_r_mem("sbb", dest, src)
    }
    fn emit_sbb_r_imm(&mut self, dest: Reg, src: i32) {
        self.internal_binop_def_r_imm("sbb", dest, src)
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

    #[allow(unused_variables)]
    fn emit_mul_mem(&mut self, src: &P<Value>) {
        unimplemented!()
    }

    fn emit_imul_r_r(&mut self, dest: Reg, src: Reg) {
        self.internal_binop_def_r_r("imul", dest, src)
    }

    fn emit_div_r(&mut self, src: &P<Value>) {
        let len = check_op_len(src);

        let inst = "div".to_string() + &op_postfix(len);

        let rdx = self.prepare_machine_reg(&x86_64::RDX);
        let rax = self.prepare_machine_reg(&x86_64::RAX);
        let (reg, id, loc) = self.prepare_reg(src, inst.len() + 1);

        let asm = format!("{} {}", inst, reg);

        if len != 8 {
            trace!(
                "emit: {} rdx:rax, {} -> quotient: rax + remainder: rdx",
                inst,
                src
            );
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
            trace!(
                "emit: {} ah:al, {} -> quotient: al + remainder: ah",
                inst,
                src
            );
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

    fn emit_div_mem(&mut self, src: &P<Value>) {
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
            trace!(
                "emit: {} rdx:rax, {} -> quotient: rax + remainder: rdx",
                inst,
                src
            );
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
            trace!(
                "emit: {} ah:al, {} -> quotient: al + remainder: ah",
                inst,
                src
            );

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

    fn emit_idiv_r(&mut self, src: &P<Value>) {
        let len = check_op_len(src);
        let inst = "idiv".to_string() + &op_postfix(len);
        let (reg, id, loc) = self.prepare_reg(src, inst.len() + 1);

        let asm = format!("{} {}", inst, reg);

        if len != 8 {
            trace!(
                "emit: {} rdx:rax, {} -> quotient: rax + remainder: rdx",
                inst,
                src
            );

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
            trace!(
                "emit: {} ah:al, {} -> quotient: al + remainder: ah",
                inst,
                src
            );

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
            trace!(
                "emit: {} rdx:rax, {} -> quotient: rax + remainder: rdx",
                inst,
                src
            );

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
            trace!(
                "emit: {} ah:al, {} -> quotient: al + remainder: ah",
                inst,
                src
            );

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

    fn emit_shl_r_cl(&mut self, dest: &P<Value>) {
        self.internal_binop_def_r_mr("shl", dest, &x86_64::CL)
    }

    fn emit_shl_r_imm8(&mut self, dest: &P<Value>, src: i8) {
        self.internal_binop_def_r_imm("shl", dest, src as i32)
    }

    fn emit_shld_r_r_cl(&mut self, dest: Reg, src: Reg) {
        self.internal_triop_def_r_r_mr("shld", dest, src, &x86_64::CL);
    }

    fn emit_shr_r_cl(&mut self, dest: &P<Value>) {
        self.internal_binop_def_r_mr("shr", dest, &x86_64::CL)
    }

    fn emit_shr_r_imm8(&mut self, dest: &P<Value>, src: i8) {
        self.internal_binop_def_r_imm("shr", dest, src as i32)
    }

    fn emit_shrd_r_r_cl(&mut self, dest: Reg, src: Reg) {
        self.internal_triop_def_r_r_mr("shrd", dest, src, &x86_64::CL);
    }

    fn emit_sar_r_cl(&mut self, dest: &P<Value>) {
        self.internal_binop_def_r_mr("sar", dest, &x86_64::CL)
    }

    fn emit_sar_r_imm8(&mut self, dest: &P<Value>, src: i8) {
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
        let asm = format!("jmp {}", symbol(&mangle_name(dest_name.clone())));
        self.add_asm_branch(asm, dest_name)
    }

    fn emit_je(&mut self, dest_name: MuName) {
        trace!("emit: je {}", dest_name);

        let asm = format!("je {}", symbol(&mangle_name(dest_name.clone())));
        self.add_asm_branch2(asm, dest_name);
    }

    fn emit_jne(&mut self, dest_name: MuName) {
        trace!("emit: jne {}", dest_name);

        let asm = format!("jne {}", symbol(&mangle_name(dest_name.clone())));
        self.add_asm_branch2(asm, dest_name);
    }

    fn emit_ja(&mut self, dest_name: MuName) {
        trace!("emit: ja {}", dest_name);

        let asm = format!("ja {}", symbol(&mangle_name(dest_name.clone())));
        self.add_asm_branch2(asm, dest_name);
    }

    fn emit_jae(&mut self, dest_name: MuName) {
        trace!("emit: jae {}", dest_name);

        let asm = format!("jae {}", symbol(&mangle_name(dest_name.clone())));
        self.add_asm_branch2(asm, dest_name);
    }

    fn emit_jb(&mut self, dest_name: MuName) {
        trace!("emit: jb {}", dest_name);

        let asm = format!("jb {}", symbol(&mangle_name(dest_name.clone())));
        self.add_asm_branch2(asm, dest_name);
    }

    fn emit_jbe(&mut self, dest_name: MuName) {
        trace!("emit: jbe {}", dest_name);

        let asm = format!("jbe {}", symbol(&mangle_name(dest_name.clone())));
        self.add_asm_branch2(asm, dest_name);
    }

    fn emit_jg(&mut self, dest_name: MuName) {
        trace!("emit: jg {}", dest_name);

        let asm = format!("jg {}", symbol(&mangle_name(dest_name.clone())));
        self.add_asm_branch2(asm, dest_name);
    }

    fn emit_jge(&mut self, dest_name: MuName) {
        trace!("emit: jge {}", dest_name);

        let asm = format!("jge {}", symbol(&mangle_name(dest_name.clone())));
        self.add_asm_branch2(asm, dest_name);
    }

    fn emit_jl(&mut self, dest_name: MuName) {
        trace!("emit: jl {}", dest_name);

        let asm = format!("jl {}", symbol(&mangle_name(dest_name.clone())));
        self.add_asm_branch2(asm, dest_name);
    }

    fn emit_jle(&mut self, dest_name: MuName) {
        trace!("emit: jle {}", dest_name);

        let asm = format!("jle {}", symbol(&mangle_name(dest_name.clone())));
        self.add_asm_branch2(asm, dest_name);
    }

    fn emit_js(&mut self, dest_name: MuName) {
        trace!("emit: js {}", dest_name);

        let asm = format!("js {}", symbol(&mangle_name(dest_name.clone())));
        self.add_asm_branch2(asm, dest_name);
    }

    fn emit_call_near_rel32(
        &mut self,
        callsite: MuName,
        func: MuName,
        pe: Option<MuName>,
        uses: Vec<P<Value>>,
        defs: Vec<P<Value>>,
        is_native: bool
    ) -> ValueLocation {
        let func = if is_native {
            trace!("emit: call /*C*/ {}({:?})", func, uses);
            "/*C*/".to_string() + symbol(&func).as_str()
        } else {
            trace!("emit: call {}({:?})", func, uses);
            symbol(&mangle_name(func))
        };

        let asm = if cfg!(target_os = "macos") {
            format!("call {}", func)
        } else {
            format!("call {}@PLT", func)
        };

        self.add_asm_call(asm, pe, uses, defs, None);

        self.add_asm_global_label(symbol(&mangle_name(callsite.clone())));
        ValueLocation::Relocatable(RegGroup::GPR, callsite)
    }

    fn emit_call_near_r64(
        &mut self,
        callsite: MuName,
        func: &P<Value>,
        pe: Option<MuName>,
        uses: Vec<P<Value>>,
        defs: Vec<P<Value>>
    ) -> ValueLocation {
        trace!("emit: call {}", func);
        let (reg, id, loc) = self.prepare_reg(func, 6);
        let asm = format!("call *{}", reg);

        // the call uses the register
        self.add_asm_call(asm, pe, uses, defs, Some((id, loc)));

        self.add_asm_global_label(symbol(&mangle_name(callsite.clone())));
        ValueLocation::Relocatable(RegGroup::GPR, callsite)
    }

    #[allow(unused_variables)]
    fn emit_call_near_mem64(
        &mut self,
        callsite: MuName,
        func: &P<Value>,
        pe: Option<MuName>,
        uses: Vec<P<Value>>,
        defs: Vec<P<Value>>
    ) -> ValueLocation {
        trace!("emit: call {}", func);
        unimplemented!()
    }

    fn emit_call_jmp(
        &mut self,
        callsite: MuName,
        func: MuName,
        pe: Option<MuName>,
        uses: Vec<P<Value>>,
        defs: Vec<P<Value>>,
        is_native: bool
    ) -> ValueLocation {
        let func = if is_native {
            trace!("emit: call/jmp /*C*/ {}({:?})", func, uses);
            "/*C*/".to_string() + symbol(&func).as_str()
        } else {
            trace!("emit: call/jmp {}({:?})", func, uses);
            symbol(&mangle_name(func))
        };

        let asm = if cfg!(target_os = "macos") {
            format!("/*CALL*/ jmp {}", func)
        } else {
            format!("/*CALL*/ jmp {}@PLT", func)
        };

        self.add_asm_call(asm, pe, uses, defs, None);

        self.add_asm_global_label(symbol(&mangle_name(callsite.clone())));
        ValueLocation::Relocatable(RegGroup::GPR, callsite)
    }

    fn emit_call_jmp_indirect(
        &mut self,
        callsite: MuName,
        func: &P<Value>,
        pe: Option<MuName>,
        uses: Vec<P<Value>>,
        defs: Vec<P<Value>>
    ) -> ValueLocation {
        trace!("emit: call/jmp {}", func);
        let (reg, id, loc) = self.prepare_reg(func, 6);
        let asm = format!("/*CALL*/ jmp *{}", reg);

        // the call uses the register
        self.add_asm_call(asm, pe, uses, defs, Some((id, loc)));

        self.add_asm_global_label(symbol(&mangle_name(callsite.clone())));
        ValueLocation::Relocatable(RegGroup::GPR, callsite)
    }

    fn emit_ret(&mut self) {
        trace!("emit: ret");

        let asm = format!("ret");
        self.add_asm_ret(asm);
    }

    fn emit_mfence(&mut self) {
        trace!("emit: mfence");

        let asm = format!("mfence");
        self.add_asm_inst(asm, linked_hashmap!{}, linked_hashmap!{}, false);
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

    // mov - double

    fn emit_movsd_f64_f64(&mut self, dest: &P<Value>, src: &P<Value>) {
        self.internal_fp_mov_f_f("movsd", dest, src)
    }
    fn emit_movapd_f64_f64(&mut self, dest: Reg, src: Reg) {
        self.internal_fp_mov_f_f("movapd", dest, src);
    }
    // load
    fn emit_movsd_f64_mem64(&mut self, dest: &P<Value>, src: &P<Value>) {
        self.internal_fp_mov_f_mem("movsd", dest, src, false)
    }
    // store
    fn emit_movsd_mem64_f64(&mut self, dest: &P<Value>, src: &P<Value>) {
        self.internal_fp_mov_mem_f("movsd", dest, src, false)
    }

    // mov - float

    fn emit_movss_f32_f32(&mut self, dest: &P<Value>, src: &P<Value>) {
        self.internal_fp_mov_f_f("movss", dest, src)
    }
    fn emit_movaps_f32_f32(&mut self, dest: Reg, src: Reg) {
        self.internal_fp_mov_f_f("movaps", dest, src);
    }
    // load
    fn emit_movss_f32_mem32(&mut self, dest: &P<Value>, src: &P<Value>) {
        self.internal_fp_mov_f_mem("movss", dest, src, false)
    }
    // store
    fn emit_movss_mem32_f32(&mut self, dest: &P<Value>, src: &P<Value>) {
        self.internal_fp_mov_mem_f("movss", dest, src, false)
    }

    // compare - double

    fn emit_comisd_f64_f64(&mut self, op1: Reg, op2: Reg) {
        self.internal_fp_binop_no_def_r_r("comisd", op1, op2);
    }
    fn emit_ucomisd_f64_f64(&mut self, op1: Reg, op2: Reg) {
        self.internal_fp_binop_no_def_r_r("ucomisd", op1, op2);
    }

    // compare - float

    fn emit_comiss_f32_f32(&mut self, op1: Reg, op2: Reg) {
        self.internal_fp_binop_no_def_r_r("comiss", op1, op2);
    }
    fn emit_ucomiss_f32_f32(&mut self, op1: Reg, op2: Reg) {
        self.internal_fp_binop_no_def_r_r("ucomiss", op1, op2);
    }

    // add - double

    fn emit_addsd_f64_f64(&mut self, dest: &P<Value>, src: &P<Value>) {
        self.internal_fp_binop_def_r_r("addsd", dest, src);
    }
    fn emit_addsd_f64_mem64(&mut self, dest: &P<Value>, src: &P<Value>) {
        self.internal_fp_binop_def_r_mem("addsd", dest, src);
    }

    // add - float

    fn emit_addss_f32_f32(&mut self, dest: &P<Value>, src: &P<Value>) {
        self.internal_fp_binop_def_r_r("addss", dest, src);
    }
    fn emit_addss_f32_mem32(&mut self, dest: &P<Value>, src: &P<Value>) {
        self.internal_fp_binop_def_r_mem("addss", dest, src);
    }

    // sub - double

    fn emit_subsd_f64_f64(&mut self, dest: Reg, src: Reg) {
        self.internal_fp_binop_def_r_r("subsd", dest, src);
    }
    fn emit_subsd_f64_mem64(&mut self, dest: Reg, src: Mem) {
        self.internal_fp_binop_def_r_mem("subsd", dest, src);
    }

    // sub - float

    fn emit_subss_f32_f32(&mut self, dest: Reg, src: Reg) {
        self.internal_fp_binop_def_r_r("subss", dest, src);
    }
    fn emit_subss_f32_mem32(&mut self, dest: Reg, src: Mem) {
        self.internal_fp_binop_def_r_mem("subss", dest, src);
    }

    // div - double

    fn emit_divsd_f64_f64(&mut self, dest: Reg, src: Reg) {
        self.internal_fp_binop_def_r_r("divsd", dest, src);
    }
    fn emit_divsd_f64_mem64(&mut self, dest: Reg, src: Mem) {
        self.internal_fp_binop_def_r_mem("divsd", dest, src);
    }

    // div - float

    fn emit_divss_f32_f32(&mut self, dest: Reg, src: Reg) {
        self.internal_fp_binop_def_r_r("divss", dest, src);
    }
    fn emit_divss_f32_mem32(&mut self, dest: Reg, src: Mem) {
        self.internal_fp_binop_def_r_mem("divss", dest, src);
    }

    // mul - double

    fn emit_mulsd_f64_f64(&mut self, dest: Reg, src: Reg) {
        self.internal_fp_binop_def_r_r("mulsd", dest, src);
    }
    fn emit_mulsd_f64_mem64(&mut self, dest: Reg, src: Mem) {
        self.internal_fp_binop_def_r_mem("mulsd", dest, src);
    }

    // mul - float

    fn emit_mulss_f32_f32(&mut self, dest: Reg, src: Reg) {
        self.internal_fp_binop_def_r_r("mulss", dest, src);
    }
    fn emit_mulss_f32_mem32(&mut self, dest: Reg, src: Mem) {
        self.internal_fp_binop_def_r_mem("mulss", dest, src);
    }

    // convert - double

    fn emit_cvtsi2sd_f64_r(&mut self, dest: Reg, src: Reg) {
        self.internal_gpr_to_fpr("cvtsi2sd", dest, src);
    }
    fn emit_cvtsd2si_r_f64(&mut self, dest: Reg, src: Reg) {
        self.internal_fpr_to_gpr("cvtsd2si", dest, src);
    }
    fn emit_cvttsd2si_r_f64(&mut self, dest: Reg, src: Reg) {
        self.internal_fpr_to_gpr("cvttsd2si", dest, src);
    }

    // convert - single

    fn emit_cvtsi2ss_f32_r(&mut self, dest: Reg, src: Reg) {
        self.internal_gpr_to_fpr("cvtsi2ss", dest, src);
    }
    fn emit_cvtss2si_r_f32(&mut self, dest: Reg, src: Reg) {
        self.internal_fpr_to_gpr("cvtss2si", dest, src);
    }
    fn emit_cvttss2si_r_f32(&mut self, dest: Reg, src: Reg) {
        self.internal_fpr_to_gpr("cvttss2si", dest, src);
    }

    // convert - fp trunc
    fn emit_cvtsd2ss_f32_f64(&mut self, dest: Reg, src: Reg) {
        self.internal_fp_trunc("cvtsd2ss", dest, src)
    }

    fn emit_cvtss2sd_f64_f32(&mut self, dest: Reg, src: Reg) {
        self.internal_fp_trunc("cvtss2sd", dest, src)
    }

    // unpack low data - interleave low byte
    fn emit_punpckldq_f64_mem128(&mut self, dest: Reg, src: Mem) {
        trace!("emit: punpckldq {} {} -> {}", src, dest, dest);

        let (mem, mut uses) = self.prepare_mem(src, 9 + 1);
        let (reg, id2, loc2) = self.prepare_fpreg(dest, 9 + 1 + mem.len() + 1);

        let asm = format!("punpckldq {},{}", mem, reg);

        // memory op won't use a fpreg, we insert the use of fpreg
        uses.insert(id2, vec![loc2.clone()]);

        self.add_asm_inst(
            asm,
            linked_hashmap!{
                id2 => vec![loc2]
            },
            uses,
            true
        )
    }
    // substract packed double-fp
    fn emit_subpd_f64_mem128(&mut self, dest: Reg, src: Mem) {
        trace!("emit: subpd {} {} -> {}", src, dest, dest);

        let (mem, mut uses) = self.prepare_mem(src, 5 + 1);
        let (reg, id2, loc2) = self.prepare_fpreg(dest, 5 + 1 + mem.len() + 1);

        let asm = format!("subpd {},{}", mem, reg);

        uses.insert(id2, vec![loc2.clone()]);

        self.add_asm_inst(
            asm,
            linked_hashmap!{
                id2 => vec![loc2]
            },
            uses,
            true
        )
    }
    // packed double-fp horizontal add
    fn emit_haddpd_f64_f64(&mut self, op1: Reg, op2: Reg) {
        trace!("emit: haddpd {} {} -> {}", op2, op1, op1);

        let (reg1, id1, loc1) = self.prepare_fpreg(op1, 6 + 1);
        let (reg2, id2, loc2) = self.prepare_fpreg(op2, 6 + 1 + reg1.len() + 1);

        let asm = format!("haddpd {},{}", reg1, reg2);

        self.add_asm_inst(
            asm,
            linked_hashmap!{
                id2 => vec![loc2.clone()]
            },
            {
                if id1 == id2 {
                    linked_hashmap!{id1 => vec![loc1, loc2]}
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

    // move aligned packed double-precision fp values
    fn emit_movapd_f64_mem128(&mut self, dest: Reg, src: Mem) {
        trace!("emit movapd {} -> {}", src, dest);

        let (mem, mut uses) = self.prepare_mem(src, 6 + 1);
        let (reg, id2, loc2) = self.prepare_fpreg(dest, 6 + 1 + mem.len() + 1);

        // memory op won't use a fpreg, we insert the use of fpreg
        uses.insert(id2, vec![loc2.clone()]);

        let asm = format!("movapd {},{}", mem, reg);

        self.add_asm_inst(
            asm,
            linked_hashmap!{
                id2 => vec![loc2.clone()]
            },
            uses,
            true
        )
    }
}

use compiler::backend::code_emission::create_emit_directory;
use std::fs::File;

/// emit assembly file for a function version
pub fn emit_code(fv: &mut MuFunctionVersion, vm: &VM) {
    use std::io::prelude::*;
    use std::path;

    // acquire lock and function
    let funcs = vm.funcs().read().unwrap();
    let func = funcs.get(&fv.func_id).unwrap().read().unwrap();

    // acquire lock and compiled function
    let compiled_funcs = vm.compiled_funcs().read().unwrap();
    let cf = compiled_funcs.get(&fv.id()).unwrap().read().unwrap();

    // create 'emit' directory
    create_emit_directory(vm);

    // create emit file
    let mut file_path = path::PathBuf::new();
    file_path.push(&vm.vm_options.flag_aot_emit_dir);
    file_path.push((*func.name()).clone() + ".S");
    {
        let mut file = match File::create(file_path.as_path()) {
            Err(why) => {
                panic!(
                    "couldn't create emission file {}: {}",
                    file_path.to_str().unwrap(),
                    why
                )
            }
            Ok(file) => file
        };
        // constants in text section
        file.write("\t.text\n".as_bytes()).unwrap();

        // write constants
        for (id, constant) in cf.consts.iter() {
            let mem = cf.const_mem.get(id).unwrap();
            write_const(&mut file, constant.clone(), mem.clone());
        }

        // write code
        let code = cf.mc.as_ref().unwrap().emit();
        match file.write_all(code.as_slice()) {
            Err(why) => {
                panic!(
                    "couldn'd write to file {}: {}",
                    file_path.to_str().unwrap(),
                    why
                )
            }
            Ok(_) => info!("emit code to {}", file_path.to_str().unwrap())
        }
    }
    info!("write demangled code...");
    // Read the file we just wrote above an demangle it
    {
        let mut demangled_path = path::PathBuf::new();
        demangled_path.push(&vm.vm_options.flag_aot_emit_dir);
        demangled_path.push((*func.name()).clone() + ".demangled.S");

        let mut demangled_file = match File::create(demangled_path.as_path()) {
            Err(why) => {
                panic!(
                    "couldn't create demangled emission file {}: {}",
                    demangled_path.to_str().unwrap(),
                    why
                )
            }
            Ok(file) => file
        };
        let mut mangled_file = match File::open(file_path.as_path()) {
            Err(why) => {
                panic!(
                    "couldn't create demangled emission file {}: {}",
                    demangled_path.to_str().unwrap(),
                    why
                )
            }
            Ok(file) => file
        };
        let mut f = String::new();
        mangled_file.read_to_string(&mut f).unwrap();
        let d = demangle_text(&f);
        match demangled_file.write_all(d.as_bytes()) {
            Err(why) => {
                panic!(
                    "couldn'd write to file {}: {}",
                    demangled_path.to_str().unwrap(),
                    why
                )
            }
            Ok(_) => {
                info!(
                    "emit demangled code to {}",
                    demangled_path.to_str().unwrap()
                )
            }
        }
    }
}

// max alignment as 16 byte (written as 4 (2^4) on macos)
const MAX_ALIGN: ByteSize = 16;

/// checks alignment (if it is larger than 16 bytes, use 16 bytes; otherwise use the alignment)
fn check_align(align: ByteSize) -> ByteSize {
    if align > MAX_ALIGN {
        MAX_ALIGN
    } else {
        align
    }
}

/// writes alignment in bytes for linux
#[cfg(not(feature = "sel4-rumprun"))]
#[cfg(target_os = "linux")]
fn write_align(f: &mut File, align: ByteSize) {
    use std::io::Write;
    f.write_fmt(format_args!("\t.align {}\n", check_align(align)))
        .unwrap();
}

/// writes alignment for macos. For macos, .align is followed by exponent
/// (e.g. 16 bytes is 2^4, writes .align 4 on macos)
#[cfg(not(feature = "sel4-rumprun"))]
#[cfg(target_os = "macos")]
fn write_align(f: &mut File, align: ByteSize) {
    use std::io::Write;
    use utils::math::is_power_of_two;

    let align = check_align(align);
    let n = match is_power_of_two(align) {
        Some(n) => n,
        _ => panic!("alignments needs to be power fo 2, alignment is {}", align)
    };

    f.write_fmt(format_args!("\t.align {}\n", n)).unwrap();
}

/// writes alignment in bytes for sel4-rumprun, which is exactly the same as Linux
#[cfg(feature = "sel4-rumprun")]
fn write_align(f: &mut File, align: ByteSize) {
    use std::io::Write;
    f.write_fmt(format_args!("\t.align {}\n", check_align(align)))
        .unwrap();
}

/// writes a constant to assembly output
fn write_const(f: &mut File, constant: P<Value>, loc: P<Value>) {
    use std::io::Write;

    // label
    let label = match loc.v {
        Value_::Memory(MemoryLocation::Symbolic { ref label, .. }) => label.clone(),
        _ => {
            panic!(
                "expecing a symbolic memory location for constant {}, found {}",
                constant,
                loc
            )
        }
    };
    write_align(f, MAX_ALIGN);
    writeln!(f, "{}:", symbol(&mangle_name(label))).unwrap();

    // actual value
    write_const_value(f, constant);
}

/// writes a constant value based on its type and value
fn write_const_value(f: &mut File, constant: P<Value>) {
    use std::io::Write;

    let ref ty = constant.ty;

    let inner = match constant.v {
        Value_::Constant(ref c) => c,
        _ => panic!("expected constant, found {}", constant)
    };

    match inner {
        &Constant::Int(val) => {
            let len = ty.get_int_length().unwrap();
            match len {
                8 => {
                    f.write_fmt(format_args!("\t.byte {}\n", val as u8))
                        .unwrap()
                }
                16 => {
                    f.write_fmt(format_args!("\t.word {}\n", val as u16))
                        .unwrap()
                }
                32 => {
                    f.write_fmt(format_args!("\t.long {}\n", val as u32))
                        .unwrap()
                }
                64 => {
                    f.write_fmt(format_args!("\t.quad {}\n", val as u64))
                        .unwrap()
                }
                _ => panic!("unimplemented int length: {}", len)
            }
        }
        &Constant::IntEx(ref val) => {
            assert!(val.len() == 2);
            f.write_fmt(format_args!("\t.quad {}\n", val[0] as u64))
                .unwrap();
            f.write_fmt(format_args!("\t.quad {}\n", val[1] as u64))
                .unwrap();
        }
        &Constant::Float(val) => {
            use utils::mem::f32_to_raw;
            f.write_fmt(format_args!("\t.long {}\n", f32_to_raw(val) as u32))
                .unwrap();
        }
        &Constant::Double(val) => {
            use utils::mem::f64_to_raw;
            f.write_fmt(format_args!("\t.quad {}\n", f64_to_raw(val) as u64))
                .unwrap();
        }
        &Constant::NullRef => f.write_fmt(format_args!("\t.quad 0\n")).unwrap(),
        &Constant::ExternSym(ref name) => f.write_fmt(format_args!("\t.quad {}\n", name)).unwrap(),
        &Constant::List(ref vals) => {
            for val in vals {
                write_const_value(f, val.clone())
            }
        }
        _ => unimplemented!()
    }
}

#[cfg(not(feature = "sel4-rumprun"))]
pub fn emit_sym_table(vm: &VM) {
    debug!("Currently nothing to emit for --!");
}

fn mangle_all(name_vec: &mut Vec<String>) {
    for i in 0..name_vec.len() {
        name_vec[i] = name_vec[i]
            .replace('.', "Zd")
            .replace('-', "Zh")
            .replace(':', "Zc")
            .replace('#', "Za");
        name_vec[i] = "__mu_".to_string() + &name_vec[i];
    }
}

#[cfg(feature = "sel4-rumprun")]
pub fn emit_sym_table(vm: &VM) {

    use std::path;
    use std::io::Write;

    // Here goes the code to generate an asm file to resolve symbol addresses at link time
    // in this stage, a single sym_file is generated for the test
    // these sym_files will be compiled in build.rs in the parent directory of sel4 side

    //**************************************************
    // first create the asm file in the correct path
    // _st added file name and path stands for _SymTable
    //*************************************************
    debug!("Going to emit Sym table for sel4-rumprun");
    let mut file_path_st = path::PathBuf::new();
    file_path_st.push(&vm.vm_options.flag_aot_emit_dir);

    // vm file name is: "mu_sym_table.s"
    file_path_st.push(format!("{}", AOT_EMIT_SYM_TABLE_FILE));

    let mut file_st = match File::create(file_path_st.as_path()) {
        Err(why) => {
            panic!(
                "couldn't create SYM TABLE file {}: {}",
                file_path_st.to_str().unwrap(),
                why
            )
        }
        Ok(file) => file
    };

    // **************************************************
    // mu_sym_table.s content generation
    // *************************************************
    // Code for exporting all of the required symbols \
    // in vm, using the following fields:
    // compiled_funcs.CompiledFunction.start
    // compiled_funcs.CompiledFunction.end
    // compiled_funcs.CompiledFunction.Frame. \
    // exception_callsites[iter](src,_)
    // compiled_funcs.CompiledFunction.Frame. \
    // exception_callsites[iter](_,dest)
    // ret
    // *************************************************

    let mut sym_vec: Vec<String> = Vec::new();
    let compiled_funcs: &HashMap<_, _> = &vm.compiled_funcs().read().unwrap();
    for (theID, theCFs) in compiled_funcs.iter() {
        let theCF: &CompiledFunction = &theCFs.read().unwrap();
        match theCF.start {
            // CF.start can only be relocatable , otherwise panic
            ValueLocation::Relocatable(_, ref symbol) => {
                //                debug!("theCF.start, symbol = {}\n", *symbol);
                sym_vec.push((*symbol).clone());
            }
            // CF.start can't reach this state
            _ => {
                panic!(
                    "Sym_Table_start: expecting Relocatable location, found {}",
                    theCF.start
                )
            }
        }
        match theCF.end {
            // CF.start can only be relocatable , otherwise panic
            ValueLocation::Relocatable(_, ref symbol) => {
                //                debug!("theCF.end, symbol = {}\n", *symbol);
                sym_vec.push((*symbol).clone());
            }
            // CF.end can't reach this state
            _ => {
                panic!(
                    "Sym_Table_end: expecting Relocatable location, found {}",
                    theCF.end
                )
            }
        }

        // for &(ref callsite, ref dest) in theCF.frame.get_exception_callsites().iter(){
        //     match *callsite {
        //         ValueLocation::Relocatable(_, ref symbol) => {
        //             sym_vec.push((*symbol).clone());
        //         },
        //         // can't reach this state
        //         _ => panic!("Sym_Table_callsite: expecting Relocatable location, found {}",
        //                callsite)
        //     }
        //     match *dest {
        //         ValueLocation::Relocatable(_, ref symbol) => {
        //             sym_vec.push((*symbol).clone());
        //         },
        //         // can't reach this state
        //         _ => panic!("Sym_Table_callsite: expecting Relocatable location, found {}",
        //                       dest)
        //     }
        // }
    }

    mangle_all(&mut sym_vec);

    file_st.write("\t.data\n".as_bytes()).unwrap();

    file_st
        .write_fmt(format_args!(
            "\t{}\n",
            directive_globl("mu_sym_table".to_string())
        ))
        .unwrap();
    file_st.write_fmt(format_args!("mu_sym_table:\n")).unwrap();
    file_st
        .write_fmt(format_args!(".quad {}\n", sym_vec.len()))
        .unwrap();
    for i in 0..sym_vec.len() {
        file_st
            .write_fmt(format_args!(".quad {}\n", sym_vec[i].len()))
            .unwrap();
        file_st
            .write_fmt(format_args!(".ascii \"{}\"\n", sym_vec[i]))
            .unwrap();
        file_st
            .write_fmt(format_args!(".quad {}\n", sym_vec[i]))
            .unwrap();
    }
}


use std::collections::HashMap;
use compiler::backend::code_emission::emit_mu_types;

/// emit vm context for current session, considering relocation symbols/fields from the client
pub fn emit_context_with_reloc(
    vm: &VM,
    symbols: HashMap<Address, MuName>,
    fields: HashMap<Address, MuName>
) {
    use std::path;
    use std::io::prelude::*;

    emit_mu_types("", vm);

    // creates emit directy, and file
    debug!("---Emit VM Context---");
    create_emit_directory(vm);
    let mut file_path = path::PathBuf::new();
    file_path.push(&vm.vm_options.flag_aot_emit_dir);
    file_path.push(AOT_EMIT_CONTEXT_FILE);
    let mut file = match File::create(file_path.as_path()) {
        Err(why) => {
            panic!(
                "couldn't create context file {}: {}",
                file_path.to_str().unwrap(),
                why
            )
        }
        Ok(file) => file
    };

    // --- bss section ---
    // not used for now
    file.write_fmt(format_args!("\t.bss\n")).unwrap();

    // --- data section ---
    file.write("\t.data\n".as_bytes()).unwrap();

    // persist heap - we traverse the heap from globals
    {
        use runtime::mm;

        let global_locs_lock = vm.global_locations().read().unwrap();
        let global_lock = vm.globals().read().unwrap();

        // a map from address to ID
        let global_addr_id_map = {
            let mut map: LinkedHashMap<Address, MuID> = LinkedHashMap::new();
            for (id, global_loc) in global_locs_lock.iter() {
                map.insert(global_loc.to_address(), *id);
            }
            map
        };

        // get address of all globals so we can traverse heap from them
        let global_addrs: Vec<Address> =
            global_locs_lock.values().map(|x| x.to_address()).collect();
        debug!("going to dump these globals: {:?}", global_addrs);

        // heap dump
        let mut global_dump = mm::persist_heap(global_addrs);
        debug!("Heap Dump from GC: {:?}", global_dump);
        let ref objects = global_dump.objects;
        let ref mut relocatable_refs = global_dump.relocatable_refs;

        // merge symbols with relocatable_refs
        for (addr, str) in symbols {
            relocatable_refs.insert(addr, mangle_name(str));
        }

        // for all the reachable object, we write them to the boot image
        for obj_dump in objects.values() {
            write_align(&mut file, 8);

            // write object metadata
            // .bytes xx,xx,xx,xx (between mem_start to reference_addr)
            write_data_bytes(&mut file, obj_dump.mem_start, obj_dump.reference_addr);

            // if this object is a global cell, we add labels so it can be accessed
            if global_addr_id_map.contains_key(&obj_dump.reference_addr) {
                let global_id = global_addr_id_map.get(&obj_dump.reference_addr).unwrap();
                let global_value = global_lock.get(global_id).unwrap();

                // .globl global_cell_name
                // global_cell_name:
                let demangled_name = global_value.name().clone();
                let global_cell_name = symbol(&mangle_name(demangled_name.clone()));
                writeln!(file, "\t{}", directive_globl(global_cell_name.clone())).unwrap();
                writeln!(file, "{}:", global_cell_name.clone()).unwrap();

                // .equiv global_cell_name_if_its_valid_c_ident
                if is_valid_c_identifier(&demangled_name) {
                    let demangled_name = symbol(&*demangled_name);
                    writeln!(file, "\t{}", directive_globl(demangled_name.clone())).unwrap();
                    writeln!(
                        file,
                        "\t{}",
                        directive_equiv(demangled_name, global_cell_name.clone())
                    ).unwrap();
                }
            }

            // put dump_label for this object (so it can be referred to from other dumped objects)
            let dump_label = symbol(&&relocatable_refs
                .get(&obj_dump.reference_addr)
                .unwrap()
                .clone());
            file.write_fmt(format_args!("{}:\n", dump_label)).unwrap();

            // get ready to go through from the object start (not mem_start) to the end
            let base = obj_dump.reference_addr;
            let end = obj_dump.mem_start + obj_dump.mem_size;
            assert!(base.is_aligned_to(POINTER_SIZE));

            // offset as cursor
            let mut offset = 0;
            while offset < obj_dump.mem_size {
                let cur_addr = base + offset;

                if obj_dump.reference_offsets.contains(&offset) {
                    // if this offset is a reference field, we put a relocatable label
                    // generated by the GC instead of address value

                    let load_ref = unsafe { cur_addr.load::<Address>() };
                    if load_ref.is_zero() {
                        // null reference, write 0
                        file.write("\t.quad 0\n".as_bytes()).unwrap();
                    } else {
                        // get the relocatable label
                        let label = match relocatable_refs.get(&load_ref) {
                            Some(label) => label,
                            None => {
                                panic!(
                                    "cannot find label for address {}, it is not dumped by GC \
                                     (why GC didn't trace to it?)",
                                    load_ref
                                )
                            }
                        };
                        file.write_fmt(format_args!("\t.quad {}\n", symbol(&label)))
                            .unwrap();
                    }
                } else if fields.contains_key(&cur_addr) {
                    // if this offset is a field named by the client to relocatable,
                    // we put the relocatable label given by the client

                    let label = fields.get(&cur_addr).unwrap();

                    file.write_fmt(format_args!(
                        "\t.quad {}\n",
                        symbol(&mangle_name(label.clone()))
                    )).unwrap();
                } else {
                    // otherwise this offset is plain data

                    // write plain word (as bytes)
                    let next_word_addr = cur_addr + POINTER_SIZE;
                    if next_word_addr <= end {
                        write_data_bytes(&mut file, cur_addr, next_word_addr);
                    } else {
                        write_data_bytes(&mut file, cur_addr, end);
                    }
                }

                offset += POINTER_SIZE;
            }
        }
    }

    // serialize vm, and put it to boot image
    // currently using rustc_serialize to persist vm as json string.
    // Deserializing from this is extremely slow, we need to fix this. See Issue #41
    trace!("start serializing vm");
    use rodal;
    let mut dumper = rodal::AsmDumper::new(file);

    // Dump an Arc to the vm
    let vm_arc = rodal::FakeArc::new(vm);
    dumper.dump("vm", &vm_arc);

    use std::ops::Deref;
    let struct_tag_map: &RwLock<HashMap<types::StructTag, types::StructType_>> =
        types::STRUCT_TAG_MAP.deref();
    dumper.dump("STRUCT_TAG_MAP", struct_tag_map);

    let hybrid_tag_map: &RwLock<HashMap<types::HybridTag, types::HybridType_>> =
        types::HYBRID_TAG_MAP.deref();
    dumper.dump("HYBRID_TAG_MAP", hybrid_tag_map);

    dumper.finish();

    emit_sym_table(vm);

    debug!("---finish---");
}

/// emit vm context for current session,
/// without consideration about relocation symbols/fields from the client
pub fn emit_context(vm: &VM) {
    emit_context_with_reloc(vm, hashmap!{}, hashmap!{});
}

/// writes raw bytes from memory between from_address (inclusive) to to_address (exclusive)
fn write_data_bytes(f: &mut File, from: Address, to: Address) {
    use std::io::Write;

    if from < to {
        f.write("\t.byte ".as_bytes()).unwrap();

        let mut cursor = from;
        while cursor < to {
            let byte = unsafe { cursor.load::<u8>() };
            f.write_fmt(format_args!("0x{:x}", byte)).unwrap();

            cursor += 1 as ByteSize;
            if cursor != to {
                f.write(",".as_bytes()).unwrap();
            }
        }

        f.write("\n".as_bytes()).unwrap();
    }
}

/// declares a global symbol with .global
fn directive_globl(name: String) -> String {
    format!(".globl {}", name)
}

/// declares a symbol to be equivalent to another symbol
fn directive_equiv(name: String, target: String) -> String {
    format!(".equiv {}, {}", name, target)
}

/// allocates storage with .comm
#[allow(dead_code)]
fn directive_comm(name: String, size: ByteSize, align: ByteSize) -> String {
    format!(".comm {},{},{}", name, size, align)
}

/// returns symbol for a string (on linux, returns the same string)
#[cfg(not(feature = "sel4-rumprun"))]
#[cfg(target_os = "linux")]
pub fn symbol(name: &String) -> String {
    name.clone()
}
/// returns symbol for a string (on macos, prefixes it with a understore (_))
#[cfg(not(feature = "sel4-rumprun"))]
#[cfg(target_os = "macos")]
pub fn symbol(name: &String) -> String {
    format!("_{}", name)
}

/// returns symbol for a string (on sel4-rumprun, returns the same string)
#[cfg(feature = "sel4-rumprun")]
pub fn symbol(name: &String) -> String {
    name.clone()
}

/// returns a position-indepdent symbol for a string (on linux, postfixes it with @GOTPCREL)
#[cfg(not(feature = "sel4-rumprun"))]
#[cfg(target_os = "linux")]
pub fn pic_symbol(name: &String) -> String {
    format!("{}@GOTPCREL", name)
}
/// returns a position-indepdent symbol for a string (on macos, returns the same string)
#[cfg(not(feature = "sel4-rumprun"))]
#[cfg(target_os = "macos")]
pub fn pic_symbol(name: &String) -> String {
    symbol(&name)
}

/// returns a position-indepdent symbol for a string (on sel4-rumprun, postfixes it with @GOTPCREL)
#[cfg(feature = "sel4-rumprun")]
pub fn pic_symbol(name: &String) -> String {
    format!("{}@GOTPCREL", name)
}

use compiler::machine_code::CompiledFunction;

/// rewrites the machine code of a function version for spilling.
/// spills: a map from temporary IDs that get spilled to memory operands of their spilling location
pub fn spill_rewrite(
    spills: &LinkedHashMap<MuID, P<Value>>,
    func: &mut MuFunctionVersion,
    cf: &mut CompiledFunction,
    vm: &VM
) -> LinkedHashMap<MuID, MuID> {
    trace!("spill rewrite for x86_64 asm backend");

    trace!("code before spilling");
    cf.mc().trace_mc();

    // if temp a gets spilled, all its uses and defines will become a use/def of a scratch temp
    // we maintain this mapping for later use
    let mut spilled_scratch_temps = LinkedHashMap::new();

    // record code and their insertion point, so we can do the copy/insertion all at once
    let mut spill_code_before: LinkedHashMap<usize, Vec<Box<ASMCode>>> = LinkedHashMap::new();
    let mut spill_code_after: LinkedHashMap<usize, Vec<Box<ASMCode>>> = LinkedHashMap::new();

    // map from old to new
    let mut temp_for_cur_inst: LinkedHashMap<MuID, P<Value>> = LinkedHashMap::new();

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
                    let temp = func.new_ssa(MuEntityHeader::unnamed(vm.next_id()), temp_ty.clone())
                        .clone_value();

                    // maintain mapping
                    trace!("reg {} used in Inst{} is replaced as {}", val_reg, i, temp);
                    spilled_scratch_temps.insert(temp.id(), reg);

                    // generate a load
                    let code = {
                        let mut codegen = ASMCodeGen::new();
                        codegen.start_code_sequence();

                        if RegGroup::get_from_ty(&temp_ty) == RegGroup::FPR {
                            codegen.emit_spill_load_fpr(&temp, spill_mem);
                        } else if RegGroup::get_from_ty(&temp_ty) == RegGroup::GPR {
                            codegen.emit_spill_load_gpr(&temp, spill_mem);
                        } else {
                            panic!("expected spilling a reg or freg, found {}", temp_ty);
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
                        let temp =
                            func.new_ssa(MuEntityHeader::unnamed(vm.next_id()), temp_ty.clone())
                                .clone_value();

                        spilled_scratch_temps.insert(temp.id(), reg);

                        temp
                    };
                    trace!(
                        "reg {} defined in Inst{} is replaced as {}",
                        val_reg,
                        i,
                        temp
                    );

                    let code = {
                        let mut codegen = ASMCodeGen::new();
                        codegen.start_code_sequence();

                        if RegGroup::get_from_ty(&temp.ty) == RegGroup::FPR {
                            codegen.emit_spill_store_fpr(spill_mem, &temp);
                        } else if RegGroup::get_from_ty(&temp.ty) == RegGroup::GPR {
                            codegen.emit_spill_store_gpr(spill_mem, &temp);
                        } else {
                            panic!("expected spilling a reg or freg, found {}", temp.ty);
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
        let old_mc_ref: &ASMCode = old_mc.as_any().downcast_ref().unwrap();
        old_mc_ref.rewrite_insert(spill_code_before, spill_code_after)
    };

    cf.mc = Some(new_mc);

    trace!("spill rewrite done");

    trace!("code after spilling");
    cf.mc().trace_mc();

    spilled_scratch_temps
}
