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

use ast::ir::*;
use ast::ptr::*;
use compiler;
use compiler::frame::*;
use compiler::backend::mc_loopanalysis::MCLoopAnalysisResult;
use runtime::ValueLocation;
use utils::Address;
use utils::{LinkedHashMap, LinkedHashSet};
use runtime::resolve_symbol;
use rodal;
use std::sync::Arc;
use std;
use std::ops;
use std::collections::HashMap;
use std::collections::HashSet;

/// CompiledFunction store all information (including code) for a function that is compiled
pub struct CompiledFunction {
    /// Mu function ID
    pub func_id: MuID,
    /// Mu function version ID
    pub func_ver_id: MuID,

    /// a map between temporaries and their assigned machine registers
    // FIXME: assumes one temporary maps to one register
    pub temps: HashMap<MuID, MuID>,

    /// constants used in this function
    pub consts: HashMap<MuID, P<Value>>,
    /// if the constants needs to be put in memory, this stores their location
    pub const_mem: HashMap<MuID, P<Value>>,

    /// the machine code representation
    /// when making boot image, this field does not get persisted
    pub mc: Option<Box<MachineCode + Send + Sync>>,

    /// frame info for this compiled function
    pub frame: Frame,

    /// start location of this compiled function
    pub start: ValueLocation,
    /// end location of this compiled function
    pub end: ValueLocation,

    /// results of machine code loop analysis
    pub loop_analysis: Option<Box<MCLoopAnalysisResult>>
}
rodal_named!(CompiledFunction);
unsafe impl rodal::Dump for CompiledFunction {
    fn dump<D: ?Sized + rodal::Dumper>(&self, dumper: &mut D) {
        dumper.debug_record::<Self>("dump");
        dumper.dump_object(&self.func_id);
        dumper.dump_object(&self.func_ver_id);
        dumper.dump_object(&self.temps);
        dumper.dump_object(&self.consts);
        dumper.dump_object(&self.const_mem);
        dumper.dump_object(&self.frame);
        dumper.dump_object(&self.start);
        dumper.dump_object(&self.end);;
    }
}

impl CompiledFunction {
    /// creates a new compiled function
    pub fn new(
        func_id: MuID,
        fv_id: MuID,
        mc: Box<MachineCode + Send + Sync>,
        constants: HashMap<MuID, P<Value>>,
        constant_locs: HashMap<MuID, P<Value>>,
        frame: Frame,
        start_loc: ValueLocation,
        end_loc: ValueLocation
    ) -> CompiledFunction {
        CompiledFunction {
            func_id: func_id,
            func_ver_id: fv_id,
            temps: HashMap::new(),
            consts: constants,
            const_mem: constant_locs,
            mc: Some(mc),
            frame: frame,
            start: start_loc,
            end: end_loc,
            loop_analysis: None
        }
    }

    /// gets a reference to the machine code representation of this compiled function
    pub fn mc(&self) -> &Box<MachineCode + Send + Sync> {
        match self.mc {
            Some(ref mc) => mc,
            None => {
                panic!(
                    "trying to get mc from a compiled function.
                    But machine code is None (probably this compiled function is restored from
                    boot image and mc is thrown away)"
                )
            }
        }
    }

    /// gets a mutable reference to the machine code representation of this compiled function
    pub fn mc_mut(&mut self) -> &mut Box<MachineCode + Send + Sync> {
        match self.mc {
            Some(ref mut mc) => mc,
            None => panic!("no mc found from a compiled function")
        }
    }
}

// Contains information about a callsite (needed for exception handling)
rodal_named!(CompiledCallsite);
pub struct CompiledCallsite {
    pub exceptional_destination: Option<Address>,
    pub stack_args_size: usize,
    pub callee_saved_registers: Arc<HashMap<isize, isize>>,
    pub function_version: MuID
}
impl CompiledCallsite {
    pub fn new(
        callsite: &Callsite,
        fv: MuID,
        callee_saved_registers: Arc<HashMap<isize, isize>>
    ) -> CompiledCallsite {
        CompiledCallsite {
            exceptional_destination: match &callsite.exception_destination {
                &Some(ref name) => Some(resolve_symbol(name.clone())),
                &None => None
            },
            stack_args_size: callsite.stack_arg_size,
            callee_saved_registers: callee_saved_registers,
            function_version: fv
        }
    }
}

use std::any::Any;

/// MachineCode allows the compiler manipulate machine code in a target independent way
///
/// In this trait:
/// * the machine instructions/labels/asm directives are referred by an index
/// * block are referred by &str
/// * temporaries and registers are referred by MuID
///
/// This trait is designed greatly to favor the idea of in-place code generation (once
/// the machine code is generated, it will not get moved). However, as discussed in Issue#13,
/// we are uncertain whether in-place code generation is feasible. This trait may change
/// once we decide.
pub trait MachineCode {
    /// print the whole machine code by trace level log
    fn trace_mc(&self);
    /// print an inst for the given index
    fn trace_inst(&self, index: usize);

    /// emit the machine code as a byte array
    fn emit(&self) -> Vec<u8>;
    /// emit the machine instruction at the given index as a byte array
    fn emit_inst(&self, index: usize) -> Vec<u8>;
    /// returns the count of instructions in this machine code
    fn number_of_insts(&self) -> usize;

    /// is the specified index a move instruction?
    fn is_move(&self, index: usize) -> bool;
    /// is the specified index using memory operands?
    fn is_using_mem_op(&self, index: usize) -> bool;
    /// is the specified index is a nop?
    fn is_nop(&self, index: usize) -> bool;
    /// is the specified index a jump instruction? (unconditional jump)
    /// returns an Option for target block
    fn is_jmp(&self, index: usize) -> Option<MuName>;
    /// is the specified index a label? returns an Option for the label
    fn is_label(&self, index: usize) -> Option<MuName>;
    /// is the specified index loading a spilled register?
    /// returns an Option for the register that is loaded into
    fn is_spill_load(&self, index: usize) -> Option<P<Value>>;
    /// is the specified index storing a spilled register?
    /// returns an Option for the register that is stored
    fn is_spill_store(&self, index: usize) -> Option<P<Value>>;

    /// gets successors of a specified index
    fn get_succs(&self, index: usize) -> &Vec<usize>;
    /// gets predecessors of a specified index
    fn get_preds(&self, index: usize) -> &Vec<usize>;

    /// gets the next instruction of a specified index (labels are not instructions)
    fn get_next_inst(&self, index: usize) -> Option<usize>;
    /// gets the previous instruction of a specified index (labels are not instructions)
    fn get_last_inst(&self, index: usize) -> Option<usize>;

    /// gets the register uses of a specified index
    fn get_inst_reg_uses(&self, index: usize) -> Vec<MuID>;
    /// gets the register defines of a specified index
    fn get_inst_reg_defines(&self, index: usize) -> Vec<MuID>;

    /// gets block livein
    fn get_ir_block_livein(&self, block: &str) -> Option<&Vec<MuID>>;
    /// gets block liveout
    fn get_ir_block_liveout(&self, block: &str) -> Option<&Vec<MuID>>;
    /// sets block livein
    fn set_ir_block_livein(&mut self, block: &str, set: Vec<MuID>);
    /// sets block liveout
    fn set_ir_block_liveout(&mut self, block: &str, set: Vec<MuID>);

    /// gets all the blocks
    fn get_all_blocks(&self) -> Vec<MuName>;
    /// gets the entry block
    fn get_entry_block(&self) -> MuName;
    /// gets the prologue block
    fn get_prologue_block(&self) -> MuName {
        for name in self.get_all_blocks() {
            if name.contains(compiler::PROLOGUE_BLOCK_NAME) {
                return name;
            }
        }
        unreachable!()
    }
    /// gets the range of a given block, returns [start_inst, end_inst) (end_inst not included)
    fn get_block_range(&self, block: &str) -> Option<ops::Range<usize>>;
    /// gets the block for a given index, returns an Option for the block
    fn get_block_for_inst(&self, index: usize) -> Option<MuName>;

    // functions for rewrite

    /// replace a temp with a machine register (to_reg must be a machine register)
    fn replace_reg(&mut self, from: MuID, to: MuID);
    /// replace a temp that is defined in the inst with another temp
    fn replace_define_tmp_for_inst(&mut self, from: MuID, to: MuID, inst: usize);
    /// replace a temp that is used in the inst with another temp
    fn replace_use_tmp_for_inst(&mut self, from: MuID, to: MuID, inst: usize);
    /// replace destination for an unconditional branch instruction
    fn replace_branch_dest(&mut self, inst: usize, old_succ: usize, new_dest: &str, succ: usize);
    /// set an instruction as nop
    fn set_inst_nop(&mut self, index: usize);
    /// remove unnecessary push/pop if the callee saved register is not used
    /// returns what registers push/pop have been deleted, and the number of callee saved registers
    /// that weren't deleted
    fn remove_unnecessary_callee_saved(&mut self, used_callee_saved: Vec<MuID>) -> HashSet<MuID>;
    /// patch frame size
    fn patch_frame_size(&mut self, size: usize);

    fn as_any(&self) -> &Any;

    fn build_cfg(&self) -> MachineCFG {
        let mut ret = MachineCFG::empty();
        let all_blocks = self.get_all_blocks();

        let (start_inst_map, end_inst_map) = {
            let mut start_inst_map: LinkedHashMap<usize, MuName> = LinkedHashMap::new();
            let mut end_inst_map: LinkedHashMap<usize, MuName> = LinkedHashMap::new();
            for block in all_blocks.iter() {
                let range = match self.get_block_range(block) {
                    Some(range) => range,
                    None => panic!("cannot find range for block {}", block)
                };

                // start inst
                let first_inst = range.start;
                // last inst (we need to skip symbols)
                let last_inst = match self.get_last_inst(range.end) {
                    Some(last) => last,
                    None => {
                        panic!(
                            "cannot find last instruction in block {}, \
                             this block contains no instruction?",
                            block
                        )
                    }
                };
                trace!(
                    "Block {}: start_inst={}, end_inst(inclusive)={}",
                    block,
                    first_inst,
                    last_inst
                );

                start_inst_map.insert(first_inst, block.clone());
                end_inst_map.insert(last_inst, block.clone());
            }

            (start_inst_map, end_inst_map)
        };

        // collect info for each basic block
        for block in self.get_all_blocks().iter() {
            let range = self.get_block_range(block).unwrap();
            let start_inst = range.start;
            let end = range.end;

            let preds: Vec<MuName> = {
                let mut ret = vec![];

                // predecessors of the first instruction is the predecessors of this block
                for pred in self.get_preds(start_inst).into_iter() {
                    match end_inst_map.get(pred) {
                        Some(block) => ret.push(block.clone()),
                        None => {}
                    }
                }

                ret
            };

            let succs: Vec<MuName> = {
                let mut ret = vec![];

                // successors of the last instruction is the successors of this block
                for succ in self.get_succs(self.get_last_inst(end).unwrap()).into_iter() {
                    match start_inst_map.get(succ) {
                        Some(block) => ret.push(block.clone()),
                        None => {}
                    }
                }

                ret
            };

            let node = MachineCFGNode {
                block: block.clone(),
                preds: preds,
                succs: succs
            };

            trace!("{:?}", node);
            ret.inner.insert(block.clone(), node);
        }

        ret
    }
}

pub struct MachineCFG {
    inner: LinkedHashMap<MuName, MachineCFGNode>
}

impl MachineCFG {
    fn empty() -> Self {
        MachineCFG {
            inner: LinkedHashMap::new()
        }
    }

    pub fn get_blocks(&self) -> Vec<MuName> {
        self.inner.keys().map(|x| x.clone()).collect()
    }

    pub fn get_preds(&self, block: &MuName) -> &Vec<MuName> {
        &self.inner.get(block).unwrap().preds
    }

    pub fn get_succs(&self, block: &MuName) -> &Vec<MuName> {
        &self.inner.get(block).unwrap().succs
    }

    pub fn has_edge(&self, from: &MuName, to: &MuName) -> bool {
        if self.inner.contains_key(from) {
            let ref node = self.inner.get(from).unwrap();
            for succ in node.succs.iter() {
                if succ == to {
                    return true;
                }
            }
        }
        false
    }

    /// checks if there exists a path between from and to, without excluded node
    pub fn has_path_with_node_excluded(
        &self,
        from: &MuName,
        to: &MuName,
        exclude_node: &MuName
    ) -> bool {
        // we cannot exclude start and end of the path
        assert!(exclude_node != from && exclude_node != to);

        if from == to {
            true
        } else {
            // we are doing BFS

            // visited nodes
            let mut visited: LinkedHashSet<&MuName> = LinkedHashSet::new();
            // work queue
            let mut work_list: Vec<&MuName> = vec![];
            // initialize visited nodes, and work queue
            visited.insert(from);
            work_list.push(from);

            while !work_list.is_empty() {
                let n = work_list.pop().unwrap();
                for succ in self.get_succs(n) {
                    if succ == exclude_node {
                        // we are not going to follow a path with the excluded node
                        continue;
                    } else {
                        // if we are reaching destination, return true
                        if succ == to {
                            return true;
                        }

                        // push succ to work list so we will traverse them later
                        if !visited.contains(succ) {
                            visited.insert(succ);
                            work_list.push(succ);
                        }
                    }
                }
            }

            false
        }
    }
}

/// MachineCFGNode represents a block in machine code control flow graph
#[derive(Clone, Debug)]
pub struct MachineCFGNode {
    block: MuName,
    preds: Vec<MuName>,
    succs: Vec<MuName>
}
