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

use compiler::backend::AOT_EMIT_CONTEXT_FILE;
use compiler::backend::RegGroup;
use utils::ByteSize;
use utils::Address;
use utils::POINTER_SIZE;
use compiler::backend::aarch64::*;

use compiler::backend::{Reg, Mem};
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
use std::ops;
use std::collections::HashSet;
use std::sync::RwLock;

macro_rules! trace_emit {
    ($arg1:tt $($arg:tt)*) => {
        trace!(concat!("emit: ", $arg1) $($arg)*)
    }
}
struct ASMCode {
    name: MuName,
    code: Vec<ASMInst>,

    entry: MuName,
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
                }
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
                }
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

        // iterate through old machine code
        let mut inst_offset = 0; // how many instructions has been inserted
        let mut cur_block_start = usize::MAX;

        // inst N in old machine code is N' in new machine code
        // this map stores the relationship
        let mut location_map: LinkedHashMap<usize, usize> = LinkedHashMap::new();

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

    fn append_code_sequence_all(&mut self, another: &Box<ASMCode>) {
        let n_insts = another.number_of_insts();
        self.append_code_sequence(another, 0, n_insts)
    }

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

            // determine predecessor

            // we check if it is a fallthrough block
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
            let branch = asm[i].branch.clone();
            match branch {
                ASMBranchTarget::Unconditional(ref target) => {
                    // branch to target
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
                    // branch to target
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
                ASMBranchTarget::UnconditionalReg(id) => {
                    trace_if!(
                        TRACE_CFA,
                        "inst {}: is an unconditional branch to reg {}",
                        i,
                        id
                    );
                    trace_if!(TRACE_CFA, "inst {}: has no successor", i);
                }
            }
        }
    }

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

                if inst.starts_with("MOV ") || inst.starts_with("FMOV ") {
                    // normal mov
                    true
                } else {
                    false
                }
            }
            None => false
        }
    }

    fn is_using_mem_op(&self, index: usize) -> bool {
        self.code[index].is_mem_op_used
    }

    fn is_jmp(&self, index: usize) -> Option<MuName> {
        let inst = self.code.get(index);
        match inst {
            Some(inst) if inst.code.starts_with("B.") || inst.code.starts_with("B ") => {
                // Destination is the first argument
                let split: Vec<&str> = inst.code.split(' ').collect();
                Some(demangle_name(String::from(split[1])))
            }
            _ => None
        }
    }

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
            let to_reg = get_alias_for_length(to, loc.oplen);
            let to_reg_string = to_reg.name();

            string_utils::replace(
                &mut inst_to_patch.code,
                loc.index,
                &to_reg_string,
                to_reg_string.len()
            );
        }

        for loc in self.get_use_locations(from) {
            let ref mut inst_to_patch = self.code[loc.line];

            // pick the right reg based on length
            let to_reg = get_alias_for_length(to, loc.oplen);
            let to_reg_string = to_reg.name();

            string_utils::replace(
                &mut inst_to_patch.code,
                loc.index,
                &to_reg_string,
                to_reg_string.len()
            );
        }
    }

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

    fn replace_branch_dest(&mut self, inst: usize, new_dest: &str, succ: usize) {
        {
            let asm = &mut self.code[inst];

            let inst = String::from(asm.code.split_whitespace().next().unwrap());
            asm.code = format!("{} {}", inst, mangle_name(String::from(new_dest)));
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

    fn set_inst_nop(&mut self, index: usize) {
        self.code[index].code.clear();
        //        self.code.remove(index);
        //        self.code.insert(index, ASMInst::nop());
    }

    fn remove_unnecessary_callee_saved(&mut self, used_callee_saved: Vec<MuID>) -> HashSet<MuID> {
        // every push pair (STP)/and pop pair (LDP) will use/define SP
        let fp = FP.extract_ssa_id().unwrap();

        // Note: this version assumes only 1 callee is pushed or poped
        let find_op_other_than_fp = |inst: &ASMInst| -> MuID {
            for id in inst.defines.keys() {
                if *id != fp {
                    return *id;
                }
            }
            for id in inst.uses.keys() {
                if *id != fp {
                    return *id;
                }
            }

            panic!("Expected to find a used register other than the FP");
        };

        let mut inst_to_remove = vec![];
        let mut regs_to_remove = HashSet::new();

        for i in 0..self.number_of_insts() {
            let ref inst = self.code[i];

            match inst.spill_info {
                Some(SpillMemInfo::CalleeSaved) => {
                    let reg = find_op_other_than_fp(inst);
                    if !used_callee_saved.contains(&reg) {
                        inst_to_remove.push(i);
                        regs_to_remove.insert(reg);
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

    fn patch_frame_size(&mut self, size: usize) {
        debug_assert!(size % 16 == 0);

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
            if !inst.is_symbol {
                ret.append(&mut "\t".to_string().into_bytes());
            }

            ret.append(&mut inst.code.clone().into_bytes());
            ret.append(&mut "\n".to_string().into_bytes());
        }

        ret
    }

    fn emit_inst(&self, index: usize) -> Vec<u8> {
        let mut ret = vec![];

        let ref inst = self.code[index];

        if !inst.is_symbol {
            ret.append(&mut "\t".to_string().into_bytes());
        }

        ret.append(&mut inst.code.clone().into_bytes());

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
        trace!(
            "#{}\t{:30}\t\tdefine: {:?}\tuses: {:?}\tpred: {:?}\tsucc: {:?}",
            i,
            demangle_text(self.code[i].code.clone()),
            self.get_inst_reg_defines(i),
            self.get_inst_reg_uses(i),
            self.code[i].preds,
            self.code[i].succs
        );
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

    fn get_entry_block(&self) -> MuName {
        self.entry.clone()
    }

    fn get_block_range(&self, block: &str) -> Option<ops::Range<usize>> {
        match self.blocks.get(block) {
            Some(ref block) => Some(block.start_inst..block.end_inst),
            None => None
        }
    }

    fn get_block_for_inst(&self, index: usize) -> Option<MuName> {
        for (name, block) in self.blocks.iter() {
            if index >= block.start_inst && index < block.end_inst {
                return Some(name.clone());
            }
        }
        None
    }

    fn get_next_inst(&self, index: usize) -> Option<usize> {
        ASMCode::find_next_inst(index, &self.code)
    }

    fn get_last_inst(&self, index: usize) -> Option<usize> {
        ASMCode::find_last_inst(index, &self.code)
    }
}

#[derive(Clone, Debug)]
enum ASMBranchTarget {
    None,
    Conditional(MuName),
    Unconditional(MuName),
    PotentiallyExcepting(MuName),
    Return,
    UnconditionalReg(MuID)
}

#[derive(Clone, Debug)]
enum SpillMemInfo {
    Load(P<Value>),
    Store(P<Value>),
    CalleeSaved // Callee saved record
}

#[derive(Clone, Debug)]
struct ASMInst {
    code: String,

    defines: LinkedHashMap<MuID, Vec<ASMLocation>>,
    uses: LinkedHashMap<MuID, Vec<ASMLocation>>,

    is_mem_op_used: bool,
    is_symbol: bool,

    preds: Vec<usize>,
    succs: Vec<usize>,
    branch: ASMBranchTarget,

    spill_info: Option<SpillMemInfo>
}

impl ASMInst {
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
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ASMLocation {
    line: usize,
    index: usize,
    len: usize,
    oplen: usize
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

const REG_PLACEHOLDER_LEN: usize = 5;
lazy_static! {
    pub static ref REG_PLACEHOLDER : String = {
        let blank_spaces = [' ' as u8; REG_PLACEHOLDER_LEN];

        format!("{}", str::from_utf8(&blank_spaces).unwrap())
    };
}

const FRAME_SIZE_PLACEHOLDER_LEN: usize = 10; // a frame is smaller than 1 << 10
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

    fn cur(&self) -> &ASMCode {
        self.cur.as_ref().unwrap()
    }

    fn cur_mut(&mut self) -> &mut ASMCode {
        self.cur.as_mut().unwrap()
    }

    fn line(&self) -> usize {
        self.cur().code.len()
    }

    fn add_asm_symbolic(&mut self, code: String) {
        trace_emit!("{}", demangle_text(code.clone()));
        self.cur_mut().code.push(ASMInst::symbolic(code));
    }

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

    fn add_asm_inst_internal(
        &mut self,
        code: String,
        defines: LinkedHashMap<MuID, Vec<ASMLocation>>,
        uses: LinkedHashMap<MuID, Vec<ASMLocation>>,
        is_using_mem_op: bool,
        target: ASMBranchTarget,
        spill_info: Option<SpillMemInfo>
    ) {
        trace!("asm: {}", demangle_text(code.clone()));
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

    fn prepare_reg(&self, op: &P<Value>, loc: usize) -> (String, MuID, ASMLocation) {
        if cfg!(debug_assertions) {
            match op.v {
                Value_::SSAVar(_) => {}
                _ => panic!("expecting register op")
            }
        }

        let str = self.asm_reg_op(op);
        let len = str.len();
        (
            str,
            op.extract_ssa_id().unwrap(),
            ASMLocation::new(self.line(), loc, len, check_op_len(&op.ty))
        )
    }

    fn prepare_mem(
        &self,
        op: &P<Value>,
        loc: usize
    ) -> (String, LinkedHashMap<MuID, Vec<ASMLocation>>) {
        if cfg!(debug_assertions) {
            match op.v {
                Value_::Memory(_) => {}
                _ => panic!("expecting memory op")
            }
        }

        let mut ids: Vec<MuID> = vec![];
        let mut locs: Vec<ASMLocation> = vec![];
        let mut result_str: String = "".to_string();

        let mut loc_cursor: usize = loc;
        match op.v {
            // offset(base,index,scale)
            Value_::Memory(MemoryLocation::Address {
                ref base,
                ref offset,
                shift,
                signed
            }) => {
                result_str.push('[');
                loc_cursor += 1;
                // deal with base, base is ssa
                let (str, id, loc) = self.prepare_reg(base, loc_cursor);
                result_str.push_str(&str);
                ids.push(id);
                locs.push(loc);
                loc_cursor += str.len();

                // deal with offset
                if offset.is_some() {
                    result_str.push(',');
                    loc_cursor += 1;

                    let offset = offset.as_ref().unwrap();
                    match offset.v {
                        Value_::SSAVar(_) => {
                            // temp as offset
                            let (str, id, loc) = self.prepare_reg(offset, loc_cursor);

                            result_str.push_str(&str);
                            ids.push(id);
                            locs.push(loc);

                            result_str.push_str(",");
                            let n = offset.ty.get_int_length().unwrap();
                            let shift_type = if n == 64 {
                                if signed {
                                    "SXTX"
                                } else {
                                    "LSL"
                                }
                            } else if n == 32 {
                                if signed {
                                    "SXTW"
                                } else {
                                    "UXTW"
                                }
                            } else {
                                panic!("Unexpected size for offset register")
                            };

                            result_str.push_str(&shift_type);
                            result_str.push_str(" #");
                            let shift_str = shift.to_string();
                            result_str.push_str(&shift_str);
                        }
                        Value_::Constant(Constant::Int(val)) => {
                            let str = (val as i32).to_string();

                            result_str.push('#');
                            result_str.push_str(&str);
                        }
                        Value_::Constant(Constant::ExternSym(ref name)) => {
                            result_str.push('#');
                            result_str.push_str(name.as_str());
                        }
                        _ => panic!("unexpected offset type: {:?}", offset)
                    }
                }

                // scale (for LSL type)
                if shift != 0 {}

                result_str.push(']');
            }

            Value_::Memory(MemoryLocation::Symbolic {
                ref label,
                is_global,
                is_native
            }) => {
                let label = if is_native {
                    "/*C*/".to_string() + label.as_str()
                } else {
                    mangle_name(label.clone())
                };
                let label = if is_global {
                    format!(":got:{}", label.clone())
                } else {
                    label.clone()
                };
                result_str.push_str(label.as_str());
            }

            Value_::Memory(MemoryLocation::VirtualAddress { .. }) => {
                panic!("Can't directly use a virtual adress (try calling emit_mem first)");
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

    fn asm_reg_op(&self, op: &P<Value>) -> String {
        let id = op.extract_ssa_id().unwrap();
        if id < MACHINE_ID_END {
            // machine reg
            format!("{}", op.name())
        } else {
            // virtual register, use place holder
            REG_PLACEHOLDER.clone()
        }
    }

    fn finish_code_sequence_asm(&mut self) -> Box<ASMCode> {
        self.cur.take().unwrap()
    }

    fn internal_simple(&mut self, inst: &str) {
        let inst = inst.to_string();
        trace_emit!("\t{}", inst);

        let asm = inst;

        self.add_asm_inst(asm, linked_hashmap!{}, linked_hashmap!{}, false)
    }

    fn internal_simple_imm(&mut self, inst: &str, val: u64) {
        let inst = inst.to_string();
        trace_emit!("\t{} {}", inst, val);

        let asm = format!("{} #{}", inst, val);

        self.add_asm_inst(asm, linked_hashmap!{}, linked_hashmap!{}, false)
    }

    fn internal_simple_str(&mut self, inst: &str, option: &str) {
        let inst = inst.to_string();
        let option = option.to_string();
        trace_emit!("\t{} {}", inst, option);

        let asm = format!("{} {}", inst, option);

        self.add_asm_inst(asm, linked_hashmap!{}, linked_hashmap!{}, false)
    }

    // A system instruction
    fn internal_system(&mut self, inst: &str, option: &str, src: &P<Value>) {
        let inst = inst.to_string();
        let option = option.to_string();
        trace_emit!("\t{} {} {}", inst, option, src);

        let (reg1, id1, loc1) = self.prepare_reg(src, inst.len() + 1 + option.len() + 1);

        let asm = format!("{} {},{}", inst, option, reg1);

        self.add_asm_inst(
            asm,
            linked_hashmap!{},
            ignore_zero_register(id1, vec![loc1]),
            false
        )
    }

    fn internal_branch_op(&mut self, inst: &str, src: &P<Value>, dest_name: MuName) {
        trace_emit!("\t{} {}, {}", inst, src, dest_name);

        let (reg1, id1, loc1) = self.prepare_reg(src, inst.len() + 1);
        // symbolic label, we dont need to patch it
        let asm = format!("{} {},{}", inst, reg1, mangle_name(dest_name.clone()));
        self.add_asm_inst_internal(
            asm,
            linked_hashmap!{},
            linked_hashmap! { id1 => vec![loc1]},
            false,
            ASMBranchTarget::Conditional(dest_name),
            None
        );
    }

    fn internal_branch_op_imm(&mut self, inst: &str, src1: &P<Value>, src2: u8, dest_name: MuName) {
        trace_emit!("\t{} {},{},{}", inst, src1, src2, dest_name);

        let (reg1, id1, loc1) = self.prepare_reg(src1, inst.len() + 1);
        // symbolic label, we dont need to patch it
        let asm = format!(
            "{} {},#{},{}",
            inst,
            reg1,
            src2,
            mangle_name(dest_name.clone())
        );
        self.add_asm_inst_internal(
            asm,
            linked_hashmap!{},
            linked_hashmap! { id1 => vec![loc1]},
            false,
            ASMBranchTarget::Conditional(dest_name),
            None
        );
    }

    // Same as inetnral_binop except extends the second source register
    fn internal_binop_ext(
        &mut self,
        inst: &str,
        dest: &P<Value>,
        src1: &P<Value>,
        src2: &P<Value>,
        signed: bool,
        shift: u8
    ) {
        let inst = inst.to_string();
        let ext_s = if signed { "S" } else { "U" };
        let ext_p = match src2.ty.get_int_length() {
            Some(8) => "B",
            Some(16) => "H",
            Some(32) => "W",
            Some(64) => "X",
            _ => {
                panic!(
                    "op size: {} dose not support extension",
                    src2.ty.get_int_length().unwrap()
                )
            }
        };
        let ext = ext_s.to_string() + "XT" + ext_p;

        trace_emit!(
            "\t{} {}, {} {} {} -> {}",
            inst,
            src1,
            src2,
            ext,
            shift,
            dest
        );


        let (reg1, id1, loc1) = self.prepare_reg(dest, inst.len() + 1);
        let (reg2, id2, loc2) = self.prepare_reg(src1, inst.len() + 1 + reg1.len() + 1);
        let (reg3, id3, loc3) =
            self.prepare_reg(src2, inst.len() + 1 + reg1.len() + 1 + reg2.len() + 1);

        let asm = if shift == 0 {
            format!("{} {},{},{},{}", inst, reg1, reg2, reg3, ext)
        } else {
            format!("{} {},{},{},{} #{}", inst, reg1, reg2, reg3, ext, shift)
        };


        self.add_asm_inst(
            asm,
            ignore_zero_register(id1, vec![loc1]),
            create_hash_map(vec![(id2, loc2), (id3, loc3)]),
            false
        )
    }

    fn internal_binop_imm(
        &mut self,
        inst: &str,
        dest: &P<Value>,
        src1: &P<Value>,
        src2: u64,
        shift: u8
    ) {
        let inst = inst.to_string();
        trace_emit!("\t{} {}, {} LSL {} -> {}", inst, src1, src2, shift, dest);

        let (reg1, id1, loc1) = self.prepare_reg(dest, inst.len() + 1);
        let (reg2, id2, loc2) = self.prepare_reg(src1, inst.len() + 1 + reg1.len() + 1);

        let asm = if shift == 0 {
            format!("{} {},{},#{}", inst, reg1, reg2, src2)
        } else {
            format!("{} {},{},#{},LSL #{}", inst, reg1, reg2, src2, shift)
        };

        self.add_asm_inst(
            asm,
            ignore_zero_register(id1, vec![loc1]),
            ignore_zero_register(id2, vec![loc2]),
            false
        )
    }

    // dest <= inst(src1, src2)
    fn internal_unop_shift(
        &mut self,
        inst: &str,
        dest: &P<Value>,
        src: &P<Value>,
        shift: &str,
        amount: u8
    ) {
        let inst = inst.to_string();
        trace_emit!("\t{} {}, {} {} -> {}", inst, src, shift, amount, dest);

        let (reg1, id1, loc1) = self.prepare_reg(dest, inst.len() + 1);
        let (reg2, id2, loc2) = self.prepare_reg(src, inst.len() + 1 + reg1.len() + 1);

        let asm = format!("{} {},{},{} #{}", inst, reg1, reg2, shift, amount);

        self.add_asm_inst(
            asm,
            ignore_zero_register(id1, vec![loc1]),
            ignore_zero_register(id2, vec![loc2]),
            false
        )
    }

    // dest <= inst(src)
    fn internal_unop(&mut self, inst: &str, dest: &P<Value>, src: &P<Value>) {
        let inst = inst.to_string();
        trace_emit!("\t{} {} -> {}", inst, src, dest);

        let (reg1, id1, loc1) = self.prepare_reg(dest, inst.len() + 1);
        let (reg2, id2, loc2) = self.prepare_reg(src, inst.len() + 1 + reg1.len() + 1);

        let asm = format!("{} {},{}", inst, reg1, reg2);

        self.add_asm_inst(
            asm,
            ignore_zero_register(id1, vec![loc1]),
            ignore_zero_register(id2, vec![loc2]),
            false
        )
    }

    // Note: different instructions have different allowed src values
    fn internal_unop_imm(&mut self, inst: &str, dest: &P<Value>, src: u64, shift: u8) {
        debug_assert!(shift == 0 || shift == 16 || shift == 32 || shift == 48);
        let inst = inst.to_string();
        trace_emit!("\t{} {} LSL {} -> {}", inst, src, shift, dest);

        let (reg1, id1, loc1) = self.prepare_reg(dest, inst.len() + 1);
        let asm = if shift == 0 {
            format!("{} {},#{}", inst, reg1, src)
        } else {
            format!("{} {},#{},LSL #{}", inst, reg1, src, shift)
        };

        self.add_asm_inst(
            asm,
            ignore_zero_register(id1, vec![loc1]),
            linked_hashmap!{},
            false
        )
    }


    // dest <= inst(src1, src2)
    fn internal_binop(&mut self, inst: &str, dest: &P<Value>, src1: &P<Value>, src2: &P<Value>) {
        let inst = inst.to_string();
        trace_emit!("\t{} {}, {} -> {}", inst, src1, src2, dest);

        let (reg1, id1, loc1) = self.prepare_reg(dest, inst.len() + 1);
        let (reg2, id2, loc2) = self.prepare_reg(src1, inst.len() + 1 + reg1.len() + 1);
        let (reg3, id3, loc3) =
            self.prepare_reg(src2, inst.len() + 1 + reg1.len() + 1 + reg2.len() + 1);

        let asm = format!("{} {},{},{}", inst, reg1, reg2, reg3);

        self.add_asm_inst(
            asm,
            ignore_zero_register(id1, vec![loc1]),
            create_hash_map(vec![(id2, loc2), (id3, loc3)]),
            false
        )
    }

    // dest <= inst(src1, src2)
    fn internal_binop_shift(
        &mut self,
        inst: &str,
        dest: &P<Value>,
        src1: &P<Value>,
        src2: &P<Value>,
        shift: &str,
        amount: u8
    ) {
        let inst = inst.to_string();
        trace_emit!(
            "\t{} {}, {}, {} {} -> {}",
            inst,
            src1,
            src2,
            shift,
            amount,
            dest
        );

        let (reg1, id1, loc1) = self.prepare_reg(dest, inst.len() + 1);
        let (reg2, id2, loc2) = self.prepare_reg(src1, inst.len() + 1 + reg1.len() + 1);
        let (reg3, id3, loc3) =
            self.prepare_reg(src2, inst.len() + 1 + reg1.len() + 1 + reg2.len() + 1);

        let asm = format!("{} {},{},{},{} #{}", inst, reg1, reg2, reg3, shift, amount);

        self.add_asm_inst(
            asm,
            ignore_zero_register(id1, vec![loc1]),
            create_hash_map(vec![(id2, loc2), (id3, loc3)]),
            false
        )
    }

    // dest <= inst(src1, src2, src3)
    fn internal_ternop(
        &mut self,
        inst: &str,
        dest: &P<Value>,
        src1: &P<Value>,
        src2: &P<Value>,
        src3: &P<Value>
    ) {
        let inst = inst.to_string();
        trace_emit!("\t{} {}, {}, {} -> {}", inst, src3, src1, src2, dest);

        let (reg1, id1, loc1) = self.prepare_reg(dest, inst.len() + 1);
        let (reg2, id2, loc2) = self.prepare_reg(src1, inst.len() + 1 + reg1.len() + 1);
        let (reg3, id3, loc3) =
            self.prepare_reg(src2, inst.len() + 1 + reg1.len() + 1 + reg2.len() + 1);
        let (reg4, id4, loc4) = self.prepare_reg(
            src3,
            inst.len() + 1 + reg1.len() + 1 + reg2.len() + 1 + reg3.len() + 1
        );

        let asm = format!("{} {},{},{},{}", inst, reg1, reg2, reg3, reg4);

        self.add_asm_inst(
            asm,
            ignore_zero_register(id1, vec![loc1]),
            create_hash_map(vec![(id2, loc2), (id3, loc3), (id4, loc4)]),
            false
        )
    }

    fn internal_ternop_imm(
        &mut self,
        inst: &str,
        dest: &P<Value>,
        src1: &P<Value>,
        src2: u64,
        src3: u64
    ) {
        let inst = inst.to_string();
        trace_emit!("\t{} {}, {}, {} -> {}", inst, src1, src2, src3, dest);

        let (reg1, id1, loc1) = self.prepare_reg(dest, inst.len() + 1);
        let (reg2, id2, loc2) = self.prepare_reg(src1, inst.len() + 1 + reg1.len() + 1);

        let asm = format!("{} {},{},#{},#{}", inst, reg1, reg2, src2, src3);

        self.add_asm_inst(
            asm,
            ignore_zero_register(id1, vec![loc1]),
            ignore_zero_register(id2, vec![loc2]),
            false
        )
    }

    // PSTATE.<NZCV> = inst(src1, src2)
    fn internal_cmpop(&mut self, inst: &str, src1: &P<Value>, src2: &P<Value>) {
        let inst = inst.to_string();
        trace_emit!("\t{} {}, {}", inst, src1, src2);

        let (reg1, id1, loc1) = self.prepare_reg(src1, inst.len() + 1);
        let (reg2, id2, loc2) = self.prepare_reg(src2, inst.len() + 1 + reg1.len() + 1);

        let asm = format!("{} {},{}", inst, reg1, reg2);

        self.add_asm_inst(
            asm,
            linked_hashmap!{},
            create_hash_map(vec![(id1, loc1), (id2, loc2)]),
            false
        )
    }

    // dest <= inst(src1, src2)
    fn internal_cmpop_shift(
        &mut self,
        inst: &str,
        src1: &P<Value>,
        src2: &P<Value>,
        shift: &str,
        amount: u8
    ) {
        let inst = inst.to_string();
        trace_emit!("\t{} {},{}, {} {}", inst, src1, src2, shift, amount);

        let (reg1, id1, loc1) = self.prepare_reg(src1, inst.len() + 1);
        let (reg2, id2, loc2) = self.prepare_reg(src2, inst.len() + 1 + reg1.len() + 1);

        let asm = format!("{} {},{},{} #{}", inst, reg1, reg2, shift, amount);

        self.add_asm_inst(
            asm,
            linked_hashmap!{},
            create_hash_map(vec![(id1, loc1), (id2, loc2)]),
            false
        )
    }

    // Same as inetnral_binop except extends the second source register
    fn internal_cmpop_ext(
        &mut self,
        inst: &str,
        src1: &P<Value>,
        src2: &P<Value>,
        signed: bool,
        shift: u8
    ) {
        let inst = inst.to_string();
        let ext_s = if signed { "S" } else { "U" };
        let ext_p = match src2.ty.get_int_length() {
            Some(8) => "B",
            Some(16) => "H",
            Some(32) => "W",
            Some(64) => "X",
            _ => {
                panic!(
                    "op size: {} dose not support extension",
                    src2.ty.get_int_length().unwrap()
                )
            }
        };
        let ext = ext_s.to_string() + "XT" + ext_p;

        trace_emit!("\t{} {}, {} {} {}", inst, src1, src2, ext, shift);


        let (reg1, id1, loc1) = self.prepare_reg(src1, inst.len() + 1);
        let (reg2, id2, loc2) = self.prepare_reg(src2, inst.len() + 1 + reg1.len() + 1);

        let asm = format!("{} {},{},{} #{}", inst, reg1, reg2, ext, shift);

        self.add_asm_inst(
            asm,
            linked_hashmap!{},
            create_hash_map(vec![(id1, loc1), (id2, loc2)]),
            false
        )
    }
    // PSTATE.<NZCV> = inst(src1, src2 [<< 12])
    fn internal_cmpop_imm(&mut self, inst: &str, src1: &P<Value>, src2: u64, shift: u8) {
        let inst = inst.to_string();
        trace_emit!("\t{} {}, {} LSL {}", inst, src1, src2, shift);

        let (reg1, id1, loc1) = self.prepare_reg(src1, inst.len() + 1);

        let asm = if shift == 0 {
            format!("{} {},#{}", inst, reg1, src2)
        } else {
            format!("{} {},#{},LSL #{}", inst, reg1, src2, shift)
        };

        self.add_asm_inst(
            asm,
            linked_hashmap!{},
            ignore_zero_register(id1, vec![loc1]),
            false
        )
    }

    // PSTATE.<NZCV> = inst(src1, 0.0)
    fn internal_cmpop_f0(&mut self, inst: &str, src1: &P<Value>) {
        let inst = inst.to_string();
        trace_emit!("\t{} {}, 0.0", inst, src1);

        let (reg1, id1, loc1) = self.prepare_reg(src1, inst.len() + 1);

        let asm = format!("{} {},#0.0", inst, reg1);

        self.add_asm_inst(
            asm,
            linked_hashmap!{},
            ignore_zero_register(id1, vec![loc1]),
            false
        )
    }

    // dest <= inst<cond>()
    fn internal_cond_op(&mut self, inst: &str, dest: &P<Value>, cond: &str) {
        let inst = inst.to_string();
        let cond = cond.to_string();
        trace_emit!("\t{} {} -> {}", inst, cond, dest);

        let (reg1, id1, loc1) = self.prepare_reg(dest, inst.len() + 1);

        let asm = format!("{} {},{}", inst, reg1, cond);

        self.add_asm_inst(
            asm,
            ignore_zero_register(id1, vec![loc1]),
            linked_hashmap!{},
            false
        )
    }

    // dest <= inst<cond>(src)
    fn internal_cond_unop(&mut self, inst: &str, dest: &P<Value>, src: &P<Value>, cond: &str) {
        let inst = inst.to_string();
        let cond = cond.to_string();
        trace_emit!("\t{} {} {} -> {}", inst, cond, src, dest);

        let (reg1, id1, loc1) = self.prepare_reg(dest, inst.len() + 1);
        let (reg2, id2, loc2) = self.prepare_reg(src, inst.len() + 1 + reg1.len() + 1);

        let asm = format!("{} {},{},{}", inst, reg1, reg2, cond);

        self.add_asm_inst(
            asm,
            ignore_zero_register(id1, vec![loc1]),
            ignore_zero_register(id2, vec![loc2]),
            false
        )
    }

    // dest <= inst<cond>(src1, src2)
    fn internal_cond_binop(
        &mut self,
        inst: &str,
        dest: &P<Value>,
        src1: &P<Value>,
        src2: &P<Value>,
        cond: &str
    ) {
        let inst = inst.to_string();
        let cond = cond.to_string();
        trace_emit!("\t{} {}, {}, {} -> {}", inst, cond, src1, src2, dest);

        let (reg1, id1, loc1) = self.prepare_reg(dest, inst.len() + 1);
        let (reg2, id2, loc2) = self.prepare_reg(src1, inst.len() + 1 + reg1.len() + 1);
        let (reg3, id3, loc3) =
            self.prepare_reg(src2, inst.len() + 1 + reg1.len() + 1 + reg2.len() + 1);

        let asm = format!("{} {},{},{},{}", inst, reg1, reg2, reg3, cond);

        self.add_asm_inst(
            asm,
            ignore_zero_register(id1, vec![loc1]),
            create_hash_map(vec![(id2, loc2), (id3, loc3)]),
            false
        )
    }

    // PSTATE.<NZCV> = inst<cond>(src1, src2, flags)
    fn internal_cond_cmpop(
        &mut self,
        inst: &str,
        src1: &P<Value>,
        src2: &P<Value>,
        flags: u8,
        cond: &str
    ) {
        let inst = inst.to_string();
        let cond = cond.to_string();
        trace_emit!("\t{} {}, {}, {}, {}", inst, src1, src2, flags, cond);

        let (reg1, id1, loc1) = self.prepare_reg(src1, inst.len() + 1);
        let (reg2, id2, loc2) = self.prepare_reg(src2, inst.len() + 1 + reg1.len() + 1);

        let asm = format!("{} {},{},#{},{}", inst, reg1, reg2, flags, cond);

        self.add_asm_inst(
            asm,
            linked_hashmap!{},
            create_hash_map(vec![(id1, loc1), (id2, loc2)]),
            false
        )
    }

    // PSTATE.<NZCV> = inst<cond>(src1, src2, flags)
    fn internal_cond_cmpop_imm(
        &mut self,
        inst: &str,
        src1: &P<Value>,
        src2: u8,
        flags: u8,
        cond: &str
    ) {
        let inst = inst.to_string();
        let cond = cond.to_string();
        trace_emit!("\t{} {}, {}, {}, {}", inst, src1, src2, flags, cond);

        let (reg1, id1, loc1) = self.prepare_reg(src1, inst.len() + 1);

        let asm = format!("{} {},#{},#{},{}", inst, reg1, src2, flags, cond);

        self.add_asm_inst(
            asm,
            linked_hashmap!{},
            ignore_zero_register(id1, vec![loc1]),
            false
        )
    }

    fn internal_load(
        &mut self,
        inst: &str,
        dest: &P<Value>,
        src: Mem,
        signed: bool,
        is_spill_related: bool,
        is_callee_saved: bool
    ) {
        let op_len = primitive_byte_size(&dest.ty);
        let inst = inst.to_string() +
            if signed {
                match op_len {
                    1 => "SB",
                    2 => "SH",
                    4 => "SW",
                    8 => "",
                    _ => panic!("unexpected op size: {}", op_len)
                }
            } else {
                match op_len {
                    1 => "B",
                    2 => "H",
                    4 => "",
                    8 => "",
                    _ => panic!("unexpected op size: {}", op_len)
                }
            };

        trace_emit!("\t{} {} -> {}", inst, src, dest);

        let (reg, id, loc) = self.prepare_reg(dest, inst.len() + 1);
        let (mem, uses) = self.prepare_mem(src, inst.len() + 1 + reg.len() + 1);

        let asm = format!("{} {},{}", inst, reg, mem);

        if is_callee_saved {
            self.add_asm_inst_with_callee_saved(
                asm,
                ignore_zero_register(id, vec![loc]),
                uses,
                true
            )
        } else if is_spill_related {
            self.add_asm_inst_with_spill(
                asm,
                ignore_zero_register(id, vec![loc]),
                uses,
                true,
                SpillMemInfo::Load(src.clone())
            )
        } else {
            self.add_asm_inst(asm, ignore_zero_register(id, vec![loc]), uses, true)
        }
    }

    fn internal_load_pair(
        &mut self,
        inst: &str,
        dest1: &P<Value>,
        dest2: &P<Value>,
        src: &P<Value>
    ) {
        let inst = inst.to_string();
        trace_emit!("\t{} {} -> {}, {}", inst, src, dest1, dest2);

        let (reg1, id1, loc1) = self.prepare_reg(dest1, inst.len() + 1);
        let (reg2, id2, loc2) = self.prepare_reg(dest2, inst.len() + 1 + reg1.len() + 1);
        let (mem, uses) = self.prepare_mem(src, inst.len() + 1 + reg1.len() + 1 + reg2.len() + 1);

        let asm = format!("{} {},{},{}", inst, reg1, reg2, mem);

        self.add_asm_inst(
            asm,
            create_hash_map(vec![(id1, loc1), (id2, loc2)]),
            uses,
            true
        )
    }

    fn internal_store(
        &mut self,
        inst: &str,
        dest: Mem,
        src: &P<Value>,
        is_spill_related: bool,
        is_callee_saved: bool
    ) {
        let op_len = primitive_byte_size(&src.ty);
        let inst = inst.to_string() +
            match op_len {
                1 => "B",
                2 => "H",
                4 => "",
                8 => "",
                _ => panic!("unexpected op size: {}", op_len)
            };

        trace_emit!("\t{} {} -> {}", inst, src, dest);

        let (reg, id1, loc1) = self.prepare_reg(src, inst.len() + 1);
        let (mem, mut uses) = self.prepare_mem(dest, inst.len() + 1 + reg.len() + 1);

        // the register we used for the memory location is counted as 'use'
        // use the vec from mem as 'use' (push use reg from src to it)
        if is_zero_register_id(id1) {
            // zero register, ignore
        } else if uses.contains_key(&id1) {
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

    fn internal_store_exclusive(
        &mut self,
        inst: &str,
        dest: Mem,
        status: &P<Value>,
        src: &P<Value>
    ) {
        let inst = inst.to_string();
        let op_len = primitive_byte_size(&src.ty);
        let suffix = match op_len {
            1 => "B",
            2 => "H",
            4 => "",
            8 => "",
            _ => panic!("unexpected op size: {}", op_len)
        }.to_string();

        trace_emit!("\t{}-{} {} -> {},{}", inst, suffix, src, dest, status);

        let (reg1, id1, loc1) = self.prepare_reg(status, inst.len() + 1);
        let (reg2, id2, loc2) = self.prepare_reg(src, inst.len() + 1 + reg1.len() + 1);
        let (mem, mut uses) =
            self.prepare_mem(dest, inst.len() + 1 + reg1.len() + 1 + reg2.len() + 1);

        // the register we used for the memory location is counted as 'use'
        // use the vec from mem as 'use' (push use reg from src to it)
        if is_zero_register_id(id2) {
            // zero register, ignore
        } else if uses.contains_key(&id2) {
            let mut locs = uses.get_mut(&id2).unwrap();
            vec_utils::add_unique(locs, loc2);
        } else {
            uses.insert(id2, vec![loc2]);
        }

        let asm = format!("{}{} {},{},{}", inst, suffix, reg1, reg2, mem);

        self.add_asm_inst(asm, ignore_zero_register(id1, vec![loc1]), uses, true)
    }

    fn internal_store_pair(
        &mut self,
        inst: &str,
        dest: &P<Value>,
        src1: &P<Value>,
        src2: &P<Value>
    ) {
        let inst = inst.to_string();
        trace_emit!("\t{} {},{} -> {}", inst, src1, src2, dest);

        let (reg1, id1, loc1) = self.prepare_reg(src1, inst.len() + 1);
        let (reg2, id2, loc2) = self.prepare_reg(src2, inst.len() + 1 + reg1.len() + 1);
        let (mem, mut uses) =
            self.prepare_mem(dest, inst.len() + 1 + reg1.len() + 1 + reg2.len() + 1);

        if is_zero_register_id(id1) {
            // zero register, ignore
        } else if uses.contains_key(&id1) {
            let mut locs = uses.get_mut(&id1).unwrap();
            vec_utils::add_unique(locs, loc1);
        } else {
            uses.insert(id1, vec![loc1]);
        }

        if is_zero_register_id(id2) {
            // zero register, ignore
        } else if uses.contains_key(&id2) {
            let mut locs = uses.get_mut(&id2).unwrap();
            vec_utils::add_unique(locs, loc2);
        } else {
            uses.insert(id2, vec![loc2]);
        }

        let asm = format!("{} {},{},{}", inst, reg1, reg2, mem);

        self.add_asm_inst(asm, linked_hashmap!{}, uses, false)
    }

    fn internal_store_pair_exclusive(
        &mut self,
        inst: &str,
        dest: &P<Value>,
        status: &P<Value>,
        src1: &P<Value>,
        src2: &P<Value>
    ) {
        let inst = inst.to_string();
        trace_emit!("\t{} {},{} -> {},{}", inst, src1, src2, dest, status);

        let (reg1, id1, loc1) = self.prepare_reg(status, inst.len() + 1);
        let (reg2, id2, loc2) = self.prepare_reg(src1, inst.len() + 1 + reg1.len() + 1);
        let (reg3, id3, loc3) =
            self.prepare_reg(src2, inst.len() + 1 + reg1.len() + 1 + reg2.len() + 1);
        let (mem, mut uses) = self.prepare_mem(
            dest,
            inst.len() + 1 + reg1.len() + 1 + reg2.len() + 1 + reg3.len() + 1
        );

        if is_zero_register_id(id2) {
            // zero register, ignore
        } else if uses.contains_key(&id2) {
            let mut locs = uses.get_mut(&id2).unwrap();
            vec_utils::add_unique(locs, loc2);
        } else {
            uses.insert(id2, vec![loc2]);
        }

        if is_zero_register_id(id3) {
            // zero register, ignore
        } else if uses.contains_key(&id3) {
            let mut locs = uses.get_mut(&id3).unwrap();
            vec_utils::add_unique(locs, loc3);
        } else {
            uses.insert(id3, vec![loc3]);
        }

        let asm = format!("{} {},{},{},{}", inst, reg1, reg2, reg3, mem);

        self.add_asm_inst(asm, ignore_zero_register(id1, vec![loc1]), uses, false)
    }

    fn internal_call(&mut self, callsite: Option<String>, code: String, pe: Option<MuName>, args: Vec<P<Value>>, ret: Vec<P<Value>>, target: Option<(MuID, ASMLocation)>, may_return: bool) -> Option<ValueLocation> {
        let mut uses: LinkedHashMap<MuID, Vec<ASMLocation>> = LinkedHashMap::new();
        if target.is_some() {
            let (id, loc) = target.unwrap();
            uses.insert(id, vec![loc]);
        }
        for arg in args {
            uses.insert(arg.id(), vec![]);
        }

        let mut defines: LinkedHashMap<MuID, Vec<ASMLocation>> = LinkedHashMap::new();
        for ret in ret.iter() {
            defines.insert(ret.id(), vec![]);
        }

        self.add_asm_inst_internal(
            code,
            defines,
            uses,
            false,
            {
                if pe.is_some() {
                    ASMBranchTarget::PotentiallyExcepting(pe.unwrap())
                } else if may_return {
                    ASMBranchTarget::None
                } else {
                    ASMBranchTarget::Return
                }
            },
            None
        );

        if callsite.is_some() {
            let callsite_symbol = mangle_name(callsite.as_ref().unwrap().clone());
            self.add_asm_symbolic(directive_globl(callsite_symbol.clone()));
            self.add_asm_symbolic(format!("{}:", callsite_symbol.clone()));
            Some(ValueLocation::Relocatable(RegGroup::GPR, callsite.unwrap()))
        } else {
            None
        }
    }

    fn emit_ldr_spill(&mut self, dest: Reg, src: Mem) {
        self.internal_load("LDR", dest, src, false, true, false);
    }
    fn emit_str_spill(&mut self, dest: Mem, src: Reg) {
        self.internal_store("STR", dest, src, true, false);
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
        let func_symbol = mangle_name(func_name.clone());
        self.add_asm_symbolic(directive_globl(func_symbol.clone()));
        self.add_asm_symbolic(format!(".type {}, @function", func_symbol.clone()));
        self.add_asm_symbolic(format!("{}:", func_symbol.clone()));
        if is_valid_c_identifier(&func_name) {
            self.add_asm_symbolic(directive_globl(func_name.clone()));
            self.add_asm_symbolic(directive_equiv(func_name.clone(), func_symbol.clone()));
        }

        ValueLocation::Relocatable(RegGroup::GPR, func_name)
    }

    fn finish_code(
        &mut self,
        func_name: MuName
    ) -> (Box<MachineCode + Sync + Send>, ValueLocation) {
        let func_end = {
            let mut symbol = func_name.clone();
            symbol.push_str(":end");
            symbol
        };
        let func_symbol = mangle_name(func_name.clone());
        let func_end_sym = mangle_name(func_end.clone());
        self.add_asm_symbolic(directive_globl(func_end_sym.clone()));
        self.add_asm_symbolic(format!("{}:", func_end_sym.clone()));
        self.add_asm_symbolic(format!(
            ".size {}, {}-{}",
            func_symbol.clone(),
            func_end_sym.clone(),
            func_symbol.clone()
        ));

        self.cur.as_mut().unwrap().control_flow_analysis();

        (
            self.cur.take().unwrap(),
            ValueLocation::Relocatable(RegGroup::GPR, func_end)
        )
    }

    fn start_code_sequence(&mut self) {
        self.cur = Some(Box::new(ASMCode {
            name: "snippet".to_string(),
            entry: "none".to_string(),
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
        trace_emit!("{}:", block_name.clone());
        let label = format!("{}:", mangle_name(block_name.clone()));
        self.cur_mut().code.push(ASMInst::symbolic(label));

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

    fn start_exception_block(&mut self, block_name: MuName) -> ValueLocation {
        self.add_asm_symbolic(directive_globl(mangle_name(block_name.clone())));

        self.start_block(block_name.clone());

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

    fn block_exists(&self, block_name: MuName) -> bool {
        self.cur().blocks.contains_key(&block_name)
    }

    fn add_cfi_sections(&mut self, arg: &str) {
        self.add_asm_symbolic(format!(".cfi_sections {}", arg));
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
    fn add_cfi_def_cfa(&mut self, reg: Reg, offset: i32) {
        let reg = self.asm_reg_op(reg);
        self.add_asm_symbolic(format!(".cfi_def_cfa {}, {}", reg, offset));
    }
    fn add_cfi_offset(&mut self, reg: Reg, offset: i32) {
        let reg = self.asm_reg_op(reg);
        self.add_asm_symbolic(format!(".cfi_offset {}, {}", reg, offset));
    }

    fn emit_frame_grow(&mut self) {
        trace_emit!("\tSUB SP, SP, #FRAME_SIZE_PLACEHOLDER");
        let asm = format!("SUB SP,SP,#{}", FRAME_SIZE_PLACEHOLDER.clone());

        let line = self.line();
        self.cur_mut()
            .add_frame_size_patchpoint(ASMLocation::new(line, 11, FRAME_SIZE_PLACEHOLDER_LEN, 0));

        self.add_asm_inst(
            asm,
            linked_hashmap!{}, // let reg alloc ignore this instruction
            linked_hashmap!{},
            false
        )
    }

    // Pushes a pair of registers on the givne stack (uses the STP instruction)
    fn emit_push_pair(&mut self, src1: &P<Value>, src2: &P<Value>, stack: &P<Value>) {
        trace_emit!("\tpush_pair {}, {} -> {}[-8,-16]", src1, src2, stack);

        let (reg1, id1, loc1) = self.prepare_reg(src2, 3 + 1);
        let (reg2, id2, loc2) = self.prepare_reg(src1, 3 + 1 + reg1.len() + 1);
        let (reg3, id3, loc3) =
            self.prepare_reg(stack, 3 + 1 + reg1.len() + 1 + reg2.len() + 1 + 1);

        let asm = format!("STP {},{},[{},#-16]!", reg1, reg2, reg3);

        self.add_asm_inst(
            asm,
            ignore_zero_register(id3, vec![loc3.clone()]),
            create_hash_map(vec![(id1, loc1), (id2, loc2), (id3, loc3)]),
            false
        )
    }


    // TODO: What to do when src1/src2/stack are the same???
    fn emit_pop_pair(&mut self, dest1: &P<Value>, dest2: &P<Value>, stack: &P<Value>) {
        trace_emit!("\tpop_pair {} [+0,+8] -> {}, {}", stack, dest1, dest2);

        let (reg1, id1, loc1) = self.prepare_reg(dest1, 3 + 1);
        let (reg2, id2, loc2) = self.prepare_reg(dest2, 3 + 1 + reg1.len() + 1);
        let (reg3, id3, loc3) =
            self.prepare_reg(stack, 3 + 1 + reg1.len() + 1 + reg2.len() + 1 + 1);

        let asm = format!("LDP {},{},[{}],#16", reg1, reg2, reg3);

        self.add_asm_inst(
            asm,
            create_hash_map(vec![(id1, loc1), (id2, loc2), (id3, loc3.clone())]),
            ignore_zero_register(id3, vec![loc3]),
            false
        )
    }

    fn emit_ret(&mut self, src: Reg) {
        trace_emit!("\tRET {}", src);

        let (reg1, id1, loc1) = self.prepare_reg(src, 3 + 1);
        let asm = format!("RET {}", reg1);
        self.add_asm_inst_internal(
            asm,
            linked_hashmap!{},
            linked_hashmap!{id1 => vec![loc1]},
            false,
            ASMBranchTarget::Return,
            None
        );
    }

    fn emit_bl(
        &mut self,
        callsite: Option<String>,
        func: MuName,
        pe: Option<MuName>,
        args: Vec<P<Value>>,
        ret: Vec<P<Value>>,
        is_native: bool
    ) -> Option<ValueLocation> {
        if is_native {
            trace_emit!("\tBL /*C*/ {}({:?})", func, args);
        } else {
            trace_emit!("\tBL {}({:?})", func, args);
        }

        let func = if is_native {
            "/*C*/".to_string() + func.as_str()
        } else {
            mangle_name(func)
        };

        let mut ret = ret;
        ret.push(LR.clone());
        let asm = format!("BL {}", func);
        self.internal_call(callsite, asm, pe, args, ret, None, true)
    }

    fn emit_blr(
        &mut self,
        callsite: Option<String>,
        func: Reg,
        pe: Option<MuName>,
        args: Vec<P<Value>>,
        ret: Vec<P<Value>>
    ) -> Option<ValueLocation> {
        trace_emit!("\tBLR {}({:?})", func, args);
        let mut ret = ret;
        ret.push(LR.clone());

        let (reg1, id1, loc1) = self.prepare_reg(func, 3 + 1);
        let asm = format!("BLR {}", reg1);
        self.internal_call(callsite, asm, pe, args, ret, Some((id1, loc1)), true)
    }


    fn emit_b(&mut self, dest_name: MuName) {
        trace_emit!("\tB {}", dest_name);

        // symbolic label, we dont need to patch it
        let asm = format!("B {}", mangle_name(dest_name.clone()));
        self.add_asm_inst_internal(
            asm,
            linked_hashmap!{},
            linked_hashmap!{},
            false,
            ASMBranchTarget::Unconditional(dest_name),
            None
        );
    }

    fn emit_b_call(
        &mut self,
        callsite: Option<String>,
        func: MuName,
        pe: Option<MuName>,
        args: Vec<P<Value>>,
        ret: Vec<P<Value>>,
        is_native: bool,
        may_return: bool
    ) -> Option<ValueLocation> {
        if is_native {
            trace_emit!("\tB /*C*/ {}({:?})", func, args);
        } else {
            trace_emit!("\tB {}({:?})", func, args);
        }

        let func = if is_native {
            "/*C*/".to_string() + func.as_str()
        } else {
            mangle_name(func)
        };

        let mut ret = ret;
        ret.push(LR.clone());
        let asm = format!("/*CALL*/ B {}", func);
        self.internal_call(callsite, asm, pe, args, ret, None, may_return)
    }

    fn emit_b_cond(&mut self, cond: &str, dest_name: MuName) {
        trace_emit!("\tB.{} {}", cond, dest_name);

        // symbolic label, we dont need to patch it
        let asm = format!("B.{} {}", cond, mangle_name(dest_name.clone()));
        self.add_asm_inst_internal(
            asm,
            linked_hashmap!{},
            linked_hashmap!{},
            false,
            ASMBranchTarget::Conditional(dest_name),
            None
        );
    }
    fn emit_br(&mut self, dest_address: Reg) {
        trace_emit!("\tBR {}", dest_address);

        let (reg1, id1, loc1) = self.prepare_reg(dest_address, 2 + 1);
        let asm = format!("BR {}", reg1);
        self.add_asm_inst_internal(
            asm,
            linked_hashmap!{},
            linked_hashmap!{id1 => vec![loc1]},
            false,
            ASMBranchTarget::UnconditionalReg(id1),
            None
        );
    }

    fn emit_br_call(
        &mut self,
        callsite: Option<String>,
        func: Reg,
        pe: Option<MuName>,
        args: Vec<P<Value>>,
        ret: Vec<P<Value>>,
        may_return: bool
    ) -> Option<ValueLocation> {
        trace_emit!("\tBR {}({:?})", func, args);
        let mut ret = ret;
        ret.push(LR.clone());

        let (reg1, id1, loc1) = self.prepare_reg(func, 3 + 1);
        let asm = format!("/*CALL*/ BR {}", reg1);
        self.internal_call(callsite, asm, pe, args, ret, Some((id1, loc1)), may_return)
    }

    fn emit_cbnz(&mut self, src: Reg, dest_name: MuName) {
        self.internal_branch_op("CBNZ", src, dest_name);
    }
    fn emit_cbz(&mut self, src: Reg, dest_name: MuName) {
        self.internal_branch_op("CBZ", src, dest_name);
    }
    fn emit_tbnz(&mut self, src1: Reg, src2: u8, dest_name: MuName) {
        self.internal_branch_op_imm("TBNZ", src1, src2, dest_name);
    }
    fn emit_tbz(&mut self, src1: Reg, src2: u8, dest_name: MuName) {
        self.internal_branch_op_imm("TBZ", src1, src2, dest_name);
    }

    fn emit_msr(&mut self, dest: &str, src: Reg) {
        let dest = dest.to_string();
        trace_emit!("\tMSR {} -> {}", src, dest);

        let (reg1, id1, loc1) = self.prepare_reg(src, 3 + 1 + 4 + 1);

        let asm = format!("MSR {},{}", dest, reg1);

        self.add_asm_inst(
            asm,
            linked_hashmap!{},
            ignore_zero_register(id1, vec![loc1]),
            false
        )
    }
    fn emit_mrs(&mut self, dest: Reg, src: &str) {
        let src = src.to_string();
        trace_emit!("\tMRS {} -> {}", src, dest);

        let (reg1, id1, loc1) = self.prepare_reg(dest, 3 + 1);

        let asm = format!("MRS {},{}", reg1, src);

        self.add_asm_inst(
            asm,
            ignore_zero_register(id1, vec![loc1]),
            linked_hashmap!{},
            false
        )
    }


    // Address calculation
    fn emit_adr(&mut self, dest: Reg, src: Mem) {
        self.internal_load("ADR", dest, src, false, false, false)
    }
    fn emit_adrp(&mut self, dest: Reg, src: Mem) {
        self.internal_load("ADRP", dest, src, false, false, false)
    }

    // Unary operators
    fn emit_mov(&mut self, dest: Reg, src: Reg) {
        self.internal_unop("MOV", dest, src)
    }
    fn emit_mvn(&mut self, dest: Reg, src: Reg) {
        self.internal_unop("MVN", dest, src)
    }
    fn emit_neg(&mut self, dest: Reg, src: Reg) {
        self.internal_unop("NEG", dest, src)
    }
    fn emit_negs(&mut self, dest: Reg, src: Reg) {
        self.internal_unop("NEGS", dest, src)
    }
    fn emit_ngc(&mut self, dest: Reg, src: Reg) {
        self.internal_unop("NGC", dest, src)
    }
    fn emit_ngcs(&mut self, dest: Reg, src: Reg) {
        self.internal_unop("NGCS", dest, src)
    }
    fn emit_sxtb(&mut self, dest: Reg, src: Reg) {
        self.internal_unop("SXTB", dest, src)
    }
    fn emit_sxth(&mut self, dest: Reg, src: Reg) {
        self.internal_unop("SXTH", dest, src)
    }
    fn emit_sxtw(&mut self, dest: Reg, src: Reg) {
        self.internal_unop("SXTW", dest, src)
    }
    fn emit_uxtb(&mut self, dest: Reg, src: Reg) {
        self.internal_unop("UXTB", dest, src)
    }
    fn emit_cls(&mut self, dest: Reg, src: Reg) {
        self.internal_unop("CLS", dest, src)
    }
    fn emit_clz(&mut self, dest: Reg, src: Reg) {
        self.internal_unop("CLZ", dest, src)
    }
    fn emit_uxth(&mut self, dest: Reg, src: Reg) {
        self.internal_unop("UXTH", dest, src)
    }
    fn emit_rbit(&mut self, dest: Reg, src: Reg) {
        self.internal_unop("RBIT", dest, src)
    }
    fn emit_rev(&mut self, dest: Reg, src: Reg) {
        self.internal_unop("REV", dest, src)
    }
    fn emit_rev16(&mut self, dest: Reg, src: Reg) {
        self.internal_unop("REV16", dest, src)
    }
    fn emit_rev32(&mut self, dest: Reg /*64*/, src: Reg) {
        self.internal_unop("REV32", dest, src)
    }
    fn emit_rev64(&mut self, dest: Reg /*64*/, src: Reg) {
        self.internal_unop("REV64", dest, src)
    }
    fn emit_fabs(&mut self, dest: Reg, src: Reg) {
        self.internal_unop("FABS", dest, src)
    }
    fn emit_fcvt(&mut self, dest: Reg, src: Reg) {
        self.internal_unop("FCVT", dest, src)
    }
    fn emit_fcvtas(&mut self, dest: Reg, src: Reg) {
        self.internal_unop("FCVTAS", dest, src)
    }
    fn emit_fcvtau(&mut self, dest: Reg, src: Reg) {
        self.internal_unop("FCVTAU", dest, src)
    }
    fn emit_fcvtms(&mut self, dest: Reg, src: Reg) {
        self.internal_unop("FCVTMS", dest, src)
    }
    fn emit_fcvtmu(&mut self, dest: Reg, src: Reg) {
        self.internal_unop("FCVTMU", dest, src)
    }
    fn emit_fcvtns(&mut self, dest: Reg, src: Reg) {
        self.internal_unop("FCVTNS", dest, src)
    }
    fn emit_fcvtnu(&mut self, dest: Reg, src: Reg) {
        self.internal_unop("FCVTNU", dest, src)
    }
    fn emit_fcvtps(&mut self, dest: Reg, src: Reg) {
        self.internal_unop("FCVTPS", dest, src)
    }
    fn emit_fcvtpu(&mut self, dest: Reg, src: Reg) {
        self.internal_unop("FCVTPU", dest, src)
    }
    fn emit_fcvtzs(&mut self, dest: Reg, src: Reg) {
        self.internal_unop("FCVTZS", dest, src)
    }
    fn emit_fcvtzu(&mut self, dest: Reg, src: Reg) {
        self.internal_unop("FCVTZU", dest, src)
    }
    fn emit_fmov(&mut self, dest: Reg, src: Reg) {
        self.internal_unop("FMOV", dest, src)
    }
    fn emit_fneg(&mut self, dest: Reg, src: Reg) {
        self.internal_unop("FNEG", dest, src)
    }
    fn emit_frinta(&mut self, dest: Reg, src: Reg) {
        self.internal_unop("FRINTA", dest, src)
    }
    fn emit_frinti(&mut self, dest: Reg, src: Reg) {
        self.internal_unop("FRINTI", dest, src)
    }
    fn emit_frintm(&mut self, dest: Reg, src: Reg) {
        self.internal_unop("FRINTM", dest, src)
    }
    fn emit_frintn(&mut self, dest: Reg, src: Reg) {
        self.internal_unop("FRINTN", dest, src)
    }
    fn emit_frintp(&mut self, dest: Reg, src: Reg) {
        self.internal_unop("FRINTP", dest, src)
    }
    fn emit_frintx(&mut self, dest: Reg, src: Reg) {
        self.internal_unop("FRINTX", dest, src)
    }
    fn emit_frintz(&mut self, dest: Reg, src: Reg) {
        self.internal_unop("FRINTZ", dest, src)
    }
    fn emit_fsqrt(&mut self, dest: Reg, src: Reg) {
        self.internal_unop("FSQRT", dest, src)
    }
    fn emit_scvtf(&mut self, dest: Reg, src: Reg) {
        self.internal_unop("SCVTF", dest, src)
    }
    fn emit_ucvtf(&mut self, dest: Reg, src: Reg) {
        self.internal_unop("UCVTF", dest, src)
    }

    // Unary operations with shift
    fn emit_mov_shift(&mut self, dest: Reg, src: Reg, shift: &str, amount: u8) {
        self.internal_unop_shift("MOV", dest, src, shift, amount)
    }
    fn emit_mvn_shift(&mut self, dest: Reg, src: Reg, shift: &str, amount: u8) {
        self.internal_unop_shift("MVN", dest, src, shift, amount)
    }
    fn emit_neg_shift(&mut self, dest: Reg, src: Reg, shift: &str, amount: u8) {
        self.internal_unop_shift("NEG", dest, src, shift, amount)
    }
    fn emit_negs_shift(&mut self, dest: Reg, src: Reg, shift: &str, amount: u8) {
        self.internal_unop_shift("NEGS", dest, src, shift, amount)
    }

    // Unary operations with moves
    fn emit_mov_imm(&mut self, dest: &P<Value>, src: u64) {
        self.internal_unop_imm("MOV", dest, src as u64, 0)
    }

    fn emit_movz(&mut self, dest: &P<Value>, src: u16, shift: u8) {
        self.internal_unop_imm("MOVZ", dest, src as u64, shift)
    }
    fn emit_movk(&mut self, dest: &P<Value>, src: u16, shift: u8) {
        self.internal_unop_imm("MOVK", dest, src as u64, shift)
    }
    fn emit_movn(&mut self, dest: &P<Value>, src: u16, shift: u8) {
        self.internal_unop_imm("MOVN", dest, src as u64, shift)
    }
    fn emit_movi(&mut self, dest: &P<Value>, src: u64) {
        self.internal_unop_imm("MOVI", dest, src as u64, 0)
    }

    fn emit_fmov_imm(&mut self, dest: &P<Value>, src: f32) {
        trace_emit!("\tFMOV {} -> {}", src, dest);

        let (reg1, id1, loc1) = self.prepare_reg(dest, 4 + 1);
        // GCC complains if the immediate argument has no decimal part
        // (it will treat it as an integer)
        // (e.g. #1 is an error, but #1.0 is not)
        let asm = if src == src.trunc() {
            // src is an integer, append '.0'
            format!("FMOV {},#{}.0", reg1, src)
        } else {
            format!("FMOV {},#{}", reg1, src)
        };

        self.add_asm_inst(
            asm,
            linked_hashmap!{id1 => vec![loc1]},
            linked_hashmap!{},
            false
        )
    }

    // Binary operations with immediates
    fn emit_add_imm(&mut self, dest: Reg, src1: Reg, src2: u16, shift: bool) {
        self.internal_binop_imm("ADD", dest, src1, src2 as u64, if shift { 12 } else { 0 })
    }
    fn emit_adds_imm(&mut self, dest: Reg, src1: Reg, src2: u16, shift: bool) {
        self.internal_binop_imm("ADDS", dest, src1, src2 as u64, if shift { 12 } else { 0 })
    }
    fn emit_sub_imm(&mut self, dest: Reg, src1: Reg, src2: u16, shift: bool) {
        self.internal_binop_imm("SUB", dest, src1, src2 as u64, if shift { 12 } else { 0 })
    }
    fn emit_subs_imm(&mut self, dest: Reg, src1: Reg, src2: u16, shift: bool) {
        self.internal_binop_imm("SUBS", dest, src1, src2 as u64, if shift { 12 } else { 0 })
    }
    fn emit_and_imm(&mut self, dest: Reg, src1: Reg, src2: u64) {
        self.internal_binop_imm("AND", dest, src1, src2, 0)
    }
    fn emit_ands_imm(&mut self, dest: Reg, src1: Reg, src2: u64) {
        self.internal_binop_imm("ANDS", dest, src1, src2, 0)
    }
    fn emit_eor_imm(&mut self, dest: Reg, src1: Reg, src2: u64) {
        self.internal_binop_imm("EOR", dest, src1, src2, 0)
    }
    fn emit_orr_imm(&mut self, dest: Reg, src1: Reg, src2: u64) {
        self.internal_binop_imm("ORR", dest, src1, src2, 0)
    }
    fn emit_asr_imm(&mut self, dest: Reg, src1: Reg, src2: u8) {
        self.internal_binop_imm("ASR", dest, src1, src2 as u64, 0)
    }
    fn emit_lsr_imm(&mut self, dest: Reg, src1: Reg, src2: u8) {
        self.internal_binop_imm("LSR", dest, src1, src2 as u64, 0)
    }
    fn emit_lsl_imm(&mut self, dest: Reg, src1: Reg, src2: u8) {
        self.internal_binop_imm("LSL", dest, src1, src2 as u64, 0)
    }
    fn emit_ror_imm(&mut self, dest: Reg, src1: Reg, src2: u8) {
        self.internal_binop_imm("ROR", dest, src1, src2 as u64, 0)
    }

    // Binary operations with extension
    fn emit_add_ext(&mut self, dest: Reg, src1: Reg, src2: Reg, signed: bool, shift: u8) {
        self.internal_binop_ext("ADD", dest, src1, src2, signed, shift)
    }
    fn emit_adds_ext(&mut self, dest: Reg, src1: Reg, src2: Reg, signed: bool, shift: u8) {
        self.internal_binop_ext("ADDS", dest, src1, src2, signed, shift)
    }
    fn emit_sub_ext(&mut self, dest: Reg, src1: Reg, src2: Reg, signed: bool, shift: u8) {
        self.internal_binop_ext("SUB", dest, src1, src2, signed, shift)
    }
    fn emit_subs_ext(&mut self, dest: Reg, src1: Reg, src2: Reg, signed: bool, shift: u8) {
        self.internal_binop_ext("SUBS", dest, src1, src2, signed, shift)
    }

    // Normal Binary operations
    fn emit_mul(&mut self, dest: Reg, src1: Reg, src2: Reg) {
        self.internal_binop("MUL", dest, src1, src2)
    }
    fn emit_mneg(&mut self, dest: Reg, src1: Reg, src2: Reg) {
        self.internal_binop("MNEG", dest, src1, src2)
    }
    fn emit_smulh(&mut self, dest: Reg /*64*/, src1: Reg /*64*/, src2: Reg /*64*/) {
        self.internal_binop("SMULH", dest, src1, src2)
    }
    fn emit_umulh(&mut self, dest: Reg /*64*/, src1: Reg /*64*/, src2: Reg /*64*/) {
        self.internal_binop("UMULH", dest, src1, src2)
    }
    fn emit_smnegl(&mut self, dest: Reg /*64*/, src1: Reg /*32*/, src2: Reg /*32*/) {
        self.internal_binop("SMNEGL", dest, src1, src2)
    }
    fn emit_smull(&mut self, dest: Reg /*64*/, src1: Reg /*32*/, src2: Reg /*32*/) {
        self.internal_binop("SMULL", dest, src1, src2)
    }
    fn emit_umnegl(&mut self, dest: Reg /*64*/, src1: Reg /*32*/, src2: Reg /*32*/) {
        self.internal_binop("UMNEGL", dest, src1, src2)
    }
    fn emit_umull(&mut self, dest: Reg /*64*/, src1: Reg /*32*/, src2: Reg /*32*/) {
        self.internal_binop("UMULL", dest, src1, src2)
    }
    fn emit_adc(&mut self, dest: Reg, src1: Reg, src2: Reg) {
        self.internal_binop("ADC", dest, src1, src2)
    }
    fn emit_adcs(&mut self, dest: Reg, src1: Reg, src2: Reg) {
        self.internal_binop("ADCS", dest, src1, src2)
    }
    fn emit_add(&mut self, dest: Reg, src1: Reg, src2: Reg) {
        self.internal_binop("ADD", dest, src1, src2)
    }
    fn emit_adds(&mut self, dest: Reg, src1: Reg, src2: Reg) {
        self.internal_binop("ADDS", dest, src1, src2)
    }
    fn emit_sbc(&mut self, dest: Reg, src1: Reg, src2: Reg) {
        self.internal_binop("SBC", dest, src1, src2)
    }
    fn emit_sbcs(&mut self, dest: Reg, src1: Reg, src2: Reg) {
        self.internal_binop("SBCS", dest, src1, src2)
    }
    fn emit_sub(&mut self, dest: Reg, src1: Reg, src2: Reg) {
        self.internal_binop("SUB", dest, src1, src2)
    }
    fn emit_subs(&mut self, dest: Reg, src1: Reg, src2: Reg) {
        self.internal_binop("SUBS", dest, src1, src2)
    }
    fn emit_sdiv(&mut self, dest: Reg, src1: Reg, src2: Reg) {
        self.internal_binop("SDIV", dest, src1, src2)
    }
    fn emit_udiv(&mut self, dest: Reg, src1: Reg, src2: Reg) {
        self.internal_binop("UDIV", dest, src1, src2)
    }
    fn emit_asr(&mut self, dest: Reg, src1: Reg, src2: Reg) {
        self.internal_binop("ASR", dest, src1, src2)
    }
    fn emit_asrv(&mut self, dest: Reg, src1: Reg, src2: Reg) {
        self.internal_binop("ASRV", dest, src1, src2)
    }
    fn emit_lsl(&mut self, dest: Reg, src1: Reg, src2: Reg) {
        self.internal_binop("LSL", dest, src1, src2)
    }
    fn emit_lslv(&mut self, dest: Reg, src1: Reg, src2: Reg) {
        self.internal_binop("LSLV", dest, src1, src2)
    }
    fn emit_lsr(&mut self, dest: Reg, src1: Reg, src2: Reg) {
        self.internal_binop("LSR", dest, src1, src2)
    }
    fn emit_lsrv(&mut self, dest: Reg, src1: Reg, src2: Reg) {
        self.internal_binop("LSRV", dest, src1, src2)
    }
    fn emit_ror(&mut self, dest: Reg, src1: Reg, src2: Reg) {
        self.internal_binop("ROR", dest, src1, src2)
    }
    fn emit_bic(&mut self, dest: Reg, src1: Reg, src2: Reg) {
        self.internal_binop("BIC", dest, src1, src2)
    }
    fn emit_bics(&mut self, dest: Reg, src1: Reg, src2: Reg) {
        self.internal_binop("BICS", dest, src1, src2)
    }
    fn emit_and(&mut self, dest: Reg, src1: Reg, src2: Reg) {
        self.internal_binop("AND", dest, src1, src2)
    }
    fn emit_ands(&mut self, dest: Reg, src1: Reg, src2: Reg) {
        self.internal_binop("ANDS", dest, src1, src2)
    }
    fn emit_eon(&mut self, dest: Reg, src1: Reg, src2: Reg) {
        self.internal_binop("EON", dest, src1, src2)
    }
    fn emit_eor(&mut self, dest: Reg, src1: Reg, src2: Reg) {
        self.internal_binop("EOR", dest, src1, src2)
    }
    fn emit_orn(&mut self, dest: Reg, src1: Reg, src2: Reg) {
        self.internal_binop("ORN", dest, src1, src2)
    }
    fn emit_orr(&mut self, dest: Reg, src1: Reg, src2: Reg) {
        self.internal_binop("ORR", dest, src1, src2)
    }
    fn emit_fadd(&mut self, dest: Reg, src1: Reg, src2: Reg) {
        self.internal_binop("FADD", dest, src1, src2)
    }
    fn emit_fdiv(&mut self, dest: Reg, src1: Reg, src2: Reg) {
        self.internal_binop("FDIV", dest, src1, src2)
    }
    fn emit_fmax(&mut self, dest: Reg, src1: Reg, src2: Reg) {
        self.internal_binop("FMAX", dest, src1, src2)
    }
    fn emit_fmaxnm(&mut self, dest: Reg, src1: Reg, src2: Reg) {
        self.internal_binop("FMAXNM", dest, src1, src2)
    }
    fn emit_fmin(&mut self, dest: Reg, src1: Reg, src2: Reg) {
        self.internal_binop("FMIN", dest, src1, src2)
    }
    fn emit_fminm(&mut self, dest: Reg, src1: Reg, src2: Reg) {
        self.internal_binop("FMINM", dest, src1, src2)
    }
    fn emit_fmul(&mut self, dest: Reg, src1: Reg, src2: Reg) {
        self.internal_binop("FMUL", dest, src1, src2)
    }
    fn emit_fnmul(&mut self, dest: Reg, src1: Reg, src2: Reg) {
        self.internal_binop("FNMUL", dest, src1, src2)
    }
    fn emit_fsub(&mut self, dest: Reg, src1: Reg, src2: Reg) {
        self.internal_binop("FSUB", dest, src1, src2)
    }

    // Binary operations with shift
    fn emit_add_shift(&mut self, dest: Reg, src1: Reg, src2: Reg, shift: &str, amount: u8) {
        self.internal_binop_shift("ADD", dest, src1, src2, shift, amount)
    }
    fn emit_adds_shift(&mut self, dest: Reg, src1: Reg, src2: Reg, shift: &str, amount: u8) {
        self.internal_binop_shift("ADDS", dest, src1, src2, shift, amount)
    }
    fn emit_sub_shift(&mut self, dest: Reg, src1: Reg, src2: Reg, shift: &str, amount: u8) {
        self.internal_binop_shift("SUB", dest, src1, src2, shift, amount)
    }
    fn emit_subs_shift(&mut self, dest: Reg, src1: Reg, src2: Reg, shift: &str, amount: u8) {
        self.internal_binop_shift("SUBS", dest, src1, src2, shift, amount)
    }
    fn emit_bic_shift(&mut self, dest: Reg, src1: Reg, src2: Reg, shift: &str, amount: u8) {
        self.internal_binop_shift("BIC", dest, src1, src2, shift, amount)
    }
    fn emit_bics_shift(&mut self, dest: Reg, src1: Reg, src2: Reg, shift: &str, amount: u8) {
        self.internal_binop_shift("BICS", dest, src1, src2, shift, amount)
    }
    fn emit_and_shift(&mut self, dest: Reg, src1: Reg, src2: Reg, shift: &str, amount: u8) {
        self.internal_binop_shift("AND", dest, src1, src2, shift, amount)
    }
    fn emit_ands_shift(&mut self, dest: Reg, src1: Reg, src2: Reg, shift: &str, amount: u8) {
        self.internal_binop_shift("ANDS", dest, src1, src2, shift, amount)
    }
    fn emit_eon_shift(&mut self, dest: Reg, src1: Reg, src2: Reg, shift: &str, amount: u8) {
        self.internal_binop_shift("EON", dest, src1, src2, shift, amount)
    }
    fn emit_eor_shift(&mut self, dest: Reg, src1: Reg, src2: Reg, shift: &str, amount: u8) {
        self.internal_binop_shift("EOR", dest, src1, src2, shift, amount)
    }
    fn emit_orn_shift(&mut self, dest: Reg, src1: Reg, src2: Reg, shift: &str, amount: u8) {
        self.internal_binop_shift("ORN", dest, src1, src2, shift, amount)
    }
    fn emit_orr_shift(&mut self, dest: Reg, src1: Reg, src2: Reg, shift: &str, amount: u8) {
        self.internal_binop_shift("ORR", dest, src1, src2, shift, amount)
    }

    // ternarry operations
    fn emit_madd(&mut self, dest: Reg, src1: Reg, src2: Reg, src3: Reg) {
        self.internal_ternop("MADD", dest, src1, src2, src3)
    }
    fn emit_msub(&mut self, dest: Reg, src1: Reg, src2: Reg, src3: Reg) {
        self.internal_ternop("MSUB", dest, src1, src2, src3)
    }
    fn emit_smaddl(&mut self, dest: Reg, src1: Reg, src2: Reg, src3: Reg) {
        self.internal_ternop("SMADDL", dest, src1, src2, src3)
    }
    fn emit_smsubl(&mut self, dest: Reg, src1: Reg, src2: Reg, src3: Reg) {
        self.internal_ternop("SMSUBL", dest, src1, src2, src3)
    }
    fn emit_umaddl(&mut self, dest: Reg, src1: Reg, src2: Reg, src3: Reg) {
        self.internal_ternop("UMADDL", dest, src1, src2, src3)
    }
    fn emit_umsubl(&mut self, dest: Reg, src1: Reg, src2: Reg, src3: Reg) {
        self.internal_ternop("UMSUBL", dest, src1, src2, src3)
    }
    fn emit_fmadd(&mut self, dest: Reg, src1: Reg, src2: Reg, src3: Reg) {
        self.internal_ternop("FMADD", dest, src1, src2, src3)
    }
    fn emit_fmsub(&mut self, dest: Reg, src1: Reg, src2: Reg, src3: Reg) {
        self.internal_ternop("FMSUB", dest, src1, src2, src3)
    }
    fn emit_fnmadd(&mut self, dest: Reg, src1: Reg, src2: Reg, src3: Reg) {
        self.internal_ternop("FNMADD", dest, src1, src2, src3)
    }
    fn emit_fnmsub(&mut self, dest: Reg, src1: Reg, src2: Reg, src3: Reg) {
        self.internal_ternop("FNMSUB", dest, src1, src2, src3)
    }

    // Ternary operations with immediates
    fn emit_bfm(&mut self, dest: Reg, src1: Reg, src2: u8, src3: u8) {
        self.internal_ternop_imm("BFM", dest, src1, src2 as u64, src3 as u64)
    }
    fn emit_bfi(&mut self, dest: Reg, src1: Reg, src2: u8, src3: u8) {
        self.internal_ternop_imm("BFI", dest, src1, src2 as u64, src3 as u64)
    }
    fn emit_bfxil(&mut self, dest: Reg, src1: Reg, src2: u8, src3: u8) {
        self.internal_ternop_imm("BFXIL", dest, src1, src2 as u64, src3 as u64)
    }
    fn emit_ubfm(&mut self, dest: Reg, src1: Reg, src2: u8, src3: u8) {
        self.internal_ternop_imm("UBFM", dest, src1, src2 as u64, src3 as u64)
    }
    fn emit_ubfx(&mut self, dest: Reg, src1: Reg, src2: u8, src3: u8) {
        self.internal_ternop_imm("UBFX", dest, src1, src2 as u64, src3 as u64)
    }
    fn emit_ubfiz(&mut self, dest: Reg, src1: Reg, src2: u8, src3: u8) {
        self.internal_ternop_imm("UBFIZ", dest, src1, src2 as u64, src3 as u64)
    }
    fn emit_sbfm(&mut self, dest: Reg, src1: Reg, src2: u8, src3: u8) {
        self.internal_ternop_imm("SBFM", dest, src1, src2 as u64, src3 as u64)
    }
    fn emit_sbfx(&mut self, dest: Reg, src1: Reg, src2: u8, src3: u8) {
        self.internal_ternop_imm("SBFX", dest, src1, src2 as u64, src3 as u64)
    }
    fn emit_sbfiz(&mut self, dest: Reg, src1: Reg, src2: u8, src3: u8) {
        self.internal_ternop_imm("SBFIZ", dest, src1, src2 as u64, src3 as u64)
    }

    // Comparisons
    fn emit_tst(&mut self, src1: Reg, src2: Reg) {
        self.internal_cmpop("TST", src1, src2)
    }
    fn emit_cmn(&mut self, src1: Reg, src2: Reg) {
        self.internal_cmpop("CMN", src1, src2)
    }
    fn emit_cmp(&mut self, src1: Reg, src2: Reg) {
        self.internal_cmpop("CMP", src1, src2)
    }
    fn emit_fcmp(&mut self, src1: Reg, src2: Reg) {
        self.internal_cmpop("FCMP", src1, src2)
    }
    fn emit_fcmpe(&mut self, src1: Reg, src2: Reg) {
        self.internal_cmpop("CMPE", src1, src2)
    }

    // Comparisons with extension
    fn emit_cmn_ext(&mut self, src1: Reg, src2: Reg, signed: bool, shift: u8) {
        self.internal_cmpop_ext("CMN", src1, src2, signed, shift)
    }
    fn emit_cmp_ext(&mut self, src1: Reg, src2: Reg, signed: bool, shift: u8) {
        self.internal_cmpop_ext("CMP", src1, src2, signed, shift)
    }

    // Comparisons with shift
    fn emit_tst_shift(&mut self, src1: Reg, src2: Reg, shift: &str, amount: u8) {
        self.internal_cmpop_shift("TST", src1, src2, shift, amount)
    }
    fn emit_cmn_shift(&mut self, src1: Reg, src2: Reg, shift: &str, amount: u8) {
        self.internal_cmpop_shift("CMN", src1, src2, shift, amount)
    }
    fn emit_cmp_shift(&mut self, src1: Reg, src2: Reg, shift: &str, amount: u8) {
        self.internal_cmpop_shift("CMP", src1, src2, shift, amount)
    }

    // Comparisons with immediates
    fn emit_tst_imm(&mut self, src1: Reg, src2: u64) {
        self.internal_cmpop_imm("TST", src1, src2, 0)
    }
    fn emit_cmn_imm(&mut self, src1: Reg, src2: u16, shift: bool) {
        self.internal_cmpop_imm("CMN", src1, src2 as u64, if shift { 12 } else { 0 })
    }
    fn emit_cmp_imm(&mut self, src1: Reg, src2: u16, shift: bool) {
        self.internal_cmpop_imm("CMP", src1, src2 as u64, if shift { 12 } else { 0 })
    }

    // Comparisons agains #0.0
    fn emit_fcmp_0(&mut self, src: Reg) {
        self.internal_cmpop_f0("FCMP", src)
    }
    fn emit_fcmpe_0(&mut self, src: Reg) {
        self.internal_cmpop_f0("CMPE", src)
    }

    // Conditional ops
    fn emit_cset(&mut self, dest: Reg, cond: &str) {
        self.internal_cond_op("CSET", dest, cond)
    }
    fn emit_csetm(&mut self, dest: Reg, cond: &str) {
        self.internal_cond_op("CSETM", dest, cond)
    }

    // Conditional unary ops
    fn emit_cinc(&mut self, dest: Reg, src: Reg, cond: &str) {
        self.internal_cond_unop("CINC", dest, src, cond)
    }
    fn emit_cneg(&mut self, dest: Reg, src: Reg, cond: &str) {
        self.internal_cond_unop("CNEG", dest, src, cond)
    }
    fn emit_cinv(&mut self, dest: Reg, src: Reg, cond: &str) {
        self.internal_cond_unop("CINB", dest, src, cond)
    }

    // Conditional binary ops
    fn emit_csel(&mut self, dest: Reg, src1: Reg, src2: Reg, cond: &str) {
        self.internal_cond_binop("CSEL", dest, src1, src2, cond)
    }
    fn emit_csinc(&mut self, dest: Reg, src1: Reg, src2: Reg, cond: &str) {
        self.internal_cond_binop("CSINC", dest, src1, src2, cond)
    }
    fn emit_csinv(&mut self, dest: Reg, src1: Reg, src2: Reg, cond: &str) {
        self.internal_cond_binop("CSINV", dest, src1, src2, cond)
    }
    fn emit_csneg(&mut self, dest: Reg, src1: Reg, src2: Reg, cond: &str) {
        self.internal_cond_binop("CSNEG", dest, src1, src2, cond)
    }
    fn emit_fcsel(&mut self, dest: Reg, src1: Reg, src2: Reg, cond: &str) {
        self.internal_cond_binop("FCSEL", dest, src1, src2, cond)
    }

    // Conditional comparisons
    fn emit_ccmn(&mut self, src1: Reg, src2: Reg, flags: u8, cond: &str) {
        self.internal_cond_cmpop("CCMN", src1, src2, flags, cond)
    }
    fn emit_ccmp(&mut self, src1: Reg, src2: Reg, flags: u8, cond: &str) {
        self.internal_cond_cmpop("CCMP", src1, src2, flags, cond)
    }
    fn emit_fccmp(&mut self, src1: Reg, src2: Reg, flags: u8, cond: &str) {
        self.internal_cond_cmpop("FCCMP", src1, src2, flags, cond)
    }
    fn emit_fccmpe(&mut self, src1: Reg, src2: Reg, flags: u8, cond: &str) {
        self.internal_cond_cmpop("FCCMPE", src1, src2, flags, cond)
    }

    // Conditional comparisons (with immediate)
    fn emit_ccmn_imm(&mut self, src1: Reg, src2: u8, flags: u8, cond: &str) {
        self.internal_cond_cmpop_imm("CCMN", src1, src2, flags, cond)
    }
    fn emit_ccmp_imm(&mut self, src1: Reg, src2: u8, flags: u8, cond: &str) {
        self.internal_cond_cmpop_imm("CCMP", src1, src2, flags, cond)
    }

    fn emit_bfc(&mut self, dest: Reg, src1: u8, src2: u8) {
        trace_emit!("\tBFC {}, {} -> {}", src1, src2, dest);

        let (reg1, id1, loc1) = self.prepare_reg(dest, 3 + 1);

        let asm = format!("BFC {},#{},#{}", reg1, src1, src2);

        self.add_asm_inst(
            asm,
            ignore_zero_register(id1, vec![loc1]),
            linked_hashmap!{},
            false
        )
    }

    fn emit_extr(&mut self, dest: Reg, src1: Reg, src2: Reg, src3: u8) {
        trace_emit!("\tEXTR {}, {}, {} -> {}", src1, src2, src3, dest);

        let (reg1, id1, loc1) = self.prepare_reg(dest, 4 + 1);
        let (reg2, id2, loc2) = self.prepare_reg(src1, 4 + 1 + reg1.len() + 1);
        let (reg3, id3, loc3) = self.prepare_reg(src2, 4 + 1 + reg1.len() + 1 + reg2.len() + 1);

        let asm = format!("EXTR {},{},{},#{}", reg1, reg2, reg3, src3);

        self.add_asm_inst(
            asm,
            ignore_zero_register(id1, vec![loc1]),
            create_hash_map(vec![(id2, loc2), (id3, loc3)]),
            false
        )
    }

    fn emit_ldr_callee_saved(&mut self, dest: Reg, src: Mem) {
        self.internal_load("LDR", dest, src, false, false, true);
    }
    fn emit_str_callee_saved(&mut self, dest: Mem, src: Reg) {
        self.internal_store("STR", dest, src, false, true)
    }

    // Loads
    fn emit_ldr(&mut self, dest: Reg, src: Mem, signed: bool) {
        self.internal_load("LDR", dest, src, signed, false, false);
    }
    fn emit_ldtr(&mut self, dest: Reg, src: Mem, signed: bool) {
        self.internal_load("LDTR", dest, src, signed, false, false);
    }
    fn emit_ldur(&mut self, dest: Reg, src: Mem, signed: bool) {
        self.internal_load("LDUR", dest, src, signed, false, false);
    }
    fn emit_ldxr(&mut self, dest: Reg, src: Mem) {
        self.internal_load("LDXR", dest, src, false, false, false);
    }
    fn emit_ldaxr(&mut self, dest: Reg, src: Mem) {
        self.internal_load("LDAXR", dest, src, false, false, false);
    }
    fn emit_ldar(&mut self, dest: Reg, src: Mem) {
        self.internal_load("LDAR", dest, src, false, false, false);
    }

    // Load pair
    fn emit_ldp(&mut self, dest1: Reg, dest2: Reg, src: Mem) {
        self.internal_load_pair("LDP", dest1, dest2, src)
    }
    fn emit_ldxp(&mut self, dest1: Reg, dest2: Reg, src: Mem) {
        self.internal_load_pair("LDXP", dest1, dest2, src)
    }
    fn emit_ldaxp(&mut self, dest1: Reg, dest2: Reg, src: Mem) {
        self.internal_load_pair("LDAXP", dest1, dest2, src)
    }
    fn emit_ldnp(&mut self, dest1: Reg, dest2: Reg, src: Mem) {
        self.internal_load_pair("LDNP", dest1, dest2, src)
    }

    // Stores
    fn emit_str(&mut self, dest: Mem, src: Reg) {
        self.internal_store("STR", dest, src, false, false)
    }
    fn emit_sttr(&mut self, dest: Mem, src: Reg) {
        self.internal_store("STTR", dest, src, false, false)
    }
    fn emit_stur(&mut self, dest: Mem, src: Reg) {
        self.internal_store("STUR", dest, src, false, false)
    }
    fn emit_stxr(&mut self, dest: Mem, status: Reg, src: Reg) {
        self.internal_store_exclusive("STXR", dest, status, src)
    }
    fn emit_stlxr(&mut self, dest: Mem, status: Reg, src: Reg) {
        self.internal_store_exclusive("STLXR", dest, status, src)
    }
    fn emit_stlr(&mut self, dest: Mem, src: Reg) {
        self.internal_store("STLR", dest, src, false, false)
    }

    // Store Pairs
    fn emit_stp(&mut self, dest: Mem, src1: Reg, src2: Reg) {
        self.internal_store_pair("STP", dest, src1, src2)
    }
    fn emit_stxp(&mut self, dest: Mem, status: Reg, src1: Reg, src2: Reg) {
        self.internal_store_pair_exclusive("STXP", dest, status, src1, src2)
    }
    fn emit_stlxp(&mut self, dest: Mem, status: Reg, src1: Reg, src2: Reg) {
        self.internal_store_pair_exclusive("STLXP", dest, status, src1, src2)
    }
    fn emit_stnp(&mut self, dest: Mem, src1: Reg, src2: Reg) {
        self.internal_store_pair("STNP", dest, src1, src2)
    }

    // Synchronisation
    fn emit_dsb(&mut self, option: &str) {
        self.internal_simple_str("DSB", option)
    }
    fn emit_dmb(&mut self, option: &str) {
        self.internal_simple_str("DMB", option)
    }
    fn emit_isb(&mut self, option: &str) {
        self.internal_simple_str("ISB", option)
    }
    fn emit_clrex(&mut self) {
        self.internal_simple("CLREX")
    }

    // Hint instructions
    fn emit_sevl(&mut self) {
        self.internal_simple("SEVL")
    }
    fn emit_sev(&mut self) {
        self.internal_simple("SEV")
    }
    fn emit_wfe(&mut self) {
        self.internal_simple("WFE")
    }
    fn emit_wfi(&mut self) {
        self.internal_simple("WFI")
    }
    fn emit_yield(&mut self) {
        self.internal_simple("YIELD")
    }
    fn emit_nop(&mut self) {
        self.internal_simple("NOP")
    }
    fn emit_hint(&mut self, val: u8) {
        self.internal_simple_imm("HINT", val as u64)
    }

    // Debug instructions
    fn emit_drps(&mut self) {
        self.internal_simple("DRPS")
    }
    fn emit_dcps1(&mut self, val: u16) {
        self.internal_simple_imm("DCPS1", val as u64)
    }
    fn emit_dcps2(&mut self, val: u16) {
        self.internal_simple_imm("DCPS2", val as u64)
    }
    fn emit_dcps3(&mut self, val: u16) {
        self.internal_simple_imm("DCPS3", val as u64)
    }

    // System instruction
    fn emit_dc(&mut self, option: &str, src: Reg) {
        self.internal_system("DC", option, src)
    }
    fn emit_at(&mut self, option: &str, src: Reg) {
        self.internal_system("AT", option, src)
    }
    fn emit_ic(&mut self, option: &str, src: Reg) {
        self.internal_system("IC", option, src)
    }
    fn emit_tlbi(&mut self, option: &str, src: Reg) {
        self.internal_system("TLBI", option, src)
    }

    fn emit_sys(&mut self, imm1: u8, cn: u8, cm: u8, imm2: u8, src: Reg) {
        trace_emit!("\tSYS {}, C{}, C{}, {}, {}", imm1, cn, cm, imm2, src);

        let start = format!("SYS #{},C{},C{},#{},", imm1, cn, cm, imm2);
        let (reg1, id1, loc1) = self.prepare_reg(src, start.len());

        let asm = format!("{},{}", start, reg1);

        self.add_asm_inst(
            asm,
            linked_hashmap!{},
            ignore_zero_register(id1, vec![loc1]),
            false
        )
    }

    fn emit_sysl(&mut self, dest: Reg, imm1: u8, cn: u8, cm: u8, imm2: u8) {
        trace_emit!("\tSYSL {}, C{}, C{}, {} -> {}", imm1, cn, cm, imm2, dest);

        let (reg1, id1, loc1) = self.prepare_reg(dest, 4 + 1);

        let asm = format!("SYSL {},#{},C{},C{},#{}", reg1, imm1, cn, cm, imm2);

        self.add_asm_inst(
            asm,
            ignore_zero_register(id1, vec![loc1]),
            linked_hashmap!{},
            false
        )
    }


    // Exceptiuon instructions (NOTE: these will alter the PC)
    fn emit_brk(&mut self, val: u16) {
        self.internal_simple_imm("BRK", val as u64)
    }
    fn emit_hlt(&mut self, val: u16) {
        self.internal_simple_imm("HLT", val as u64)
    }
    fn emit_hvc(&mut self, val: u16) {
        self.internal_simple_imm("HVC", val as u64)
    }
    fn emit_smc(&mut self, val: u16) {
        self.internal_simple_imm("SMC", val as u64)
    }
    fn emit_svc(&mut self, val: u16) {
        self.internal_simple_imm("SVC", val as u64)
    }
    fn emit_eret(&mut self) {
        self.internal_simple("ERET")
    }
}

use compiler::backend::code_emission::create_emit_directory;
use std::fs::File;

pub fn emit_code(fv: &mut MuFunctionVersion, vm: &VM) {
    use std::io::prelude::*;
    use std::path;

    let funcs = vm.funcs().read().unwrap();
    let func = funcs.get(&fv.func_id).unwrap().read().unwrap();

    let compiled_funcs = vm.compiled_funcs().read().unwrap();
    let cf = compiled_funcs.get(&fv.id()).unwrap().read().unwrap();

    // create 'emit' directory
    create_emit_directory(vm);

    let mut file_path = path::PathBuf::new();
    file_path.push(&vm.vm_options.flag_aot_emit_dir);
    file_path.push(func.name().to_string() + ".S");
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

        writeln!(file, ".arch armv8-a").unwrap();

        // constants in text section
        writeln!(file, ".text").unwrap();

        write_const_min_align(&mut file);

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

    // Read the file we just wrote above an demangle it
    {
        let mut demangled_path = path::PathBuf::new();
        demangled_path.push(&vm.vm_options.flag_aot_emit_dir);
        demangled_path.push(func.name() + ".demangled.S");

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
        let d = demangle_text(f);
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

// min alignment as 4 bytes
const MIN_ALIGN: ByteSize = 4;

fn check_min_align(align: ByteSize) -> ByteSize {
    if align > MIN_ALIGN {
        MIN_ALIGN
    } else {
        align
    }
}

fn write_const_min_align(f: &mut File) {
    write_align(f, MIN_ALIGN);
}

#[cfg(target_os = "linux")]
fn write_align(f: &mut File, align: ByteSize) {
    use std::io::Write;
    writeln!(f, ".balign {}", check_min_align(align)).unwrap();
}

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
    writeln!(f, "{}:", mangle_name(label)).unwrap();

    write_const_value(f, constant);
}

fn write_const_value(f: &mut File, constant: P<Value>) {
    use std::mem;
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
                1...8 => writeln!(f, ".byte {}", get_unsigned_value(val, len) as u8).unwrap(),
                9...16 => writeln!(f, ".word {}", get_unsigned_value(val, len) as u16).unwrap(),
                17...32 => writeln!(f, ".long {}", get_unsigned_value(val, len) as u32).unwrap(),
                33...64 => writeln!(f, ".xword {}", get_unsigned_value(val, len) as u64).unwrap(),
                _ => panic!("unimplemented int length: {}", len)
            }
        }
        &Constant::Float(val) => {
            let bytes: [u8; 4] = unsafe { mem::transmute(val) };
            write!(f, ".long ").unwrap();
            f.write(&bytes).unwrap();
            writeln!(f).unwrap();
        }
        &Constant::Double(val) => {
            let bytes: [u8; 8] = unsafe { mem::transmute(val) };
            write!(f, ".xword ").unwrap();
            f.write(&bytes).unwrap();
            writeln!(f).unwrap();
        }
        &Constant::NullRef => writeln!(f, ".xword 0").unwrap(),
        &Constant::ExternSym(ref name) => writeln!(f, ".xword {}", name).unwrap(),
        &Constant::List(ref vals) => {
            for val in vals {
                write_const_value(f, val.clone())
            }
        }
        _ => unimplemented!()
    }
}


use std::collections::HashMap;

pub fn emit_context_with_reloc(
    vm: &VM,
    symbols: HashMap<Address, String>,
    fields: HashMap<Address, String>
) {
    use std::path;
    use std::io::prelude::*;

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

    // data
    writeln!(file, ".data").unwrap();

    {
        use runtime::mm;

        // persist globals
        let global_locs_lock = vm.global_locations().read().unwrap();
        let global_lock = vm.globals().read().unwrap();

        let global_addr_id_map = {
            let mut map: LinkedHashMap<Address, MuID> = LinkedHashMap::new();

            for (id, global_loc) in global_locs_lock.iter() {
                map.insert(global_loc.to_address(), *id);
            }

            map
        };

        // dump heap from globals
        let global_addrs: Vec<Address> =
            global_locs_lock.values().map(|x| x.to_address()).collect();
        debug!("going to dump these globals: {:?}", global_addrs);
        let mut global_dump = mm::persist_heap(global_addrs);
        debug!("Heap Dump from GC: {:?}", global_dump);

        let ref objects = global_dump.objects;
        let ref mut relocatable_refs = global_dump.relocatable_refs;

        // merge symbols with relocatable_refs
        for (addr, str) in symbols {
            relocatable_refs.insert(addr, mangle_name(str));
        }

        for obj_dump in objects.values() {
            write_align(&mut file, 8);

            // .bytes xx,xx,xx,xx (between mem_start to reference_addr)
            write_data_bytes(&mut file, obj_dump.mem_start, obj_dump.reference_addr);

            if global_addr_id_map.contains_key(&obj_dump.reference_addr) {
                let global_id = global_addr_id_map.get(&obj_dump.reference_addr).unwrap();

                let global_value = global_lock.get(global_id).unwrap();

                // .globl global_cell_name
                // global_cell_name:
                let demangled_name = global_value.name().clone();
                let global_cell_name = mangle_name(demangled_name.clone());
                writeln!(file, "\t{}", directive_globl(global_cell_name.clone())).unwrap();
                writeln!(file, "{}:", global_cell_name.clone()).unwrap();

                if is_valid_c_identifier(&demangled_name) {
                    writeln!(file, "\t{}", directive_globl(demangled_name.clone())).unwrap();
                    writeln!(
                        file,
                        "\t{}",
                        directive_equiv(demangled_name, global_cell_name.clone())
                    ).unwrap();
                }
            }

            // dump_label:
            let dump_label = relocatable_refs
                .get(&obj_dump.reference_addr)
                .unwrap()
                .clone();
            writeln!(file, "{}:", dump_label).unwrap();

            let base = obj_dump.reference_addr;
            let end = obj_dump.mem_start + obj_dump.mem_size;
            assert!(base.is_aligned_to(POINTER_SIZE));

            let mut offset = 0;

            while offset < obj_dump.mem_size {
                let cur_addr = base + offset;

                if obj_dump.reference_offsets.contains(&offset) {
                    // write ref with label
                    let load_ref = unsafe { cur_addr.load::<Address>() };
                    if load_ref.is_zero() {
                        // write 0
                        writeln!(file, ".xword 0").unwrap();
                    } else {
                        let label = match relocatable_refs.get(&load_ref) {
                            Some(label) => label,
                            None => {
                                panic!(
                                    "cannot find label for address {}, \
                                     it is not dumped by GC (why GC didn't trace to it)",
                                    load_ref
                                )
                            }
                        };

                        writeln!(file, ".xword {}", label.clone()).unwrap();
                    }
                } else if fields.contains_key(&cur_addr) {
                    // write uptr (or other relocatable value) with label
                    let label = fields.get(&cur_addr).unwrap();

                    writeln!(file, ".xword {}", mangle_name(label.clone())).unwrap();
                } else {
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

    // serialize vm
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

    dumper.finish(); // Dump everything the previously dumped objects referenced

    // main_thread
    //    let primordial = vm.primordial.read().unwrap();
    //    if primordial.is_some() {
    //        let primordial = primordial.as_ref().unwrap();
    //    }

    debug!("---finish---");
}

pub fn emit_context(vm: &VM) {
    emit_context_with_reloc(vm, hashmap!{}, hashmap!{});
}

fn write_data_bytes(f: &mut File, from: Address, to: Address) {
    use std::io::Write;

    if from < to {
        f.write(".byte ".as_bytes()).unwrap();

        let mut cursor = from;
        while cursor < to {
            let byte = unsafe { cursor.load::<u8>() };
            write!(f, "0x{:x}", byte).unwrap();

            cursor = cursor + 1 as ByteSize;

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
fn directive_equiv(name: String, target: String) -> String {
    format!(".equiv {}, {}", name, target)
}

use compiler::machine_code::CompiledFunction;

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

                        let spill_mem = emit_mem(
                            &mut codegen,
                            &spill_mem,
                            get_type_alignment(&temp.ty, vm),
                            &mut func.context,
                            vm
                        );
                        codegen.emit_ldr_spill(&temp, &spill_mem);

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

                    // generate a store
                    let code = {
                        let mut codegen = ASMCodeGen::new();
                        codegen.start_code_sequence();

                        let spill_mem = emit_mem(
                            &mut codegen,
                            &spill_mem,
                            get_type_alignment(&temp.ty, vm),
                            &mut func.context,
                            vm
                        );
                        codegen.emit_str_spill(&spill_mem, &temp);

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

#[inline(always)]
// This function is used so that the register allocator will ignore the zero register
// (some instructions don't use this function as they don't support the zero regester,
// or the use of the zero register would make no sense (such as branching to it))
fn ignore_zero_register(id: MuID, locs: Vec<ASMLocation>) -> LinkedHashMap<MuID, Vec<ASMLocation>> {
    if is_zero_register_id(id) {
        linked_hashmap!{}
    } else {
        linked_hashmap!{id => locs}
    }
}


#[inline(always)]
// Creates a hashmap from the given vector (ignoring the zero register)
fn create_hash_map(data: Vec<(MuID, ASMLocation)>) -> LinkedHashMap<MuID, Vec<ASMLocation>> {
    let mut map: LinkedHashMap<MuID, Vec<ASMLocation>> = LinkedHashMap::new();

    for (id, loc) in data {
        if is_zero_register_id(id) {
            // ignore the zero register
        } else if map.contains_key(&id) {
            map.get_mut(&id).unwrap().push(loc);
        } else {
            map.insert(id, vec![loc]);
        }
    }
    map
}
