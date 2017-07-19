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
use compiler::frame::*;
use runtime::ValueLocation;

use rodal;
use utils::Address;
use std::sync::Arc;
use runtime::resolve_symbol;
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
    pub temps : HashMap<MuID, MuID>,

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
    pub end: ValueLocation
}
unsafe impl rodal::Dump for CompiledFunction {
    fn dump<D: ?Sized + rodal::Dumper>(&self, dumper: &mut D) {
        dumper.debug_record("CompiledFunction", "dump");
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
    pub fn new(func_id: MuID, fv_id: MuID, mc: Box<MachineCode + Send + Sync>,
               constants: HashMap<MuID, P<Value>>, constant_locs: HashMap<MuID, P<Value>>,
               frame: Frame, start_loc: ValueLocation, end_loc: ValueLocation) -> CompiledFunction {
        CompiledFunction {
            func_id: func_id,
            func_ver_id: fv_id,
            temps:  HashMap::new(),
            consts: constants,
            const_mem: constant_locs,
            mc: Some(mc),
            frame: frame,
            start: start_loc,
            end: end_loc
        }
    }

    /// gets a reference to the machine code representation of this compiled function
    pub fn mc(&self) -> &Box<MachineCode + Send + Sync> {
        match self.mc {
            Some(ref mc) => mc,
            None => panic!("trying to get mc from a compiled function. 
                But machine code is None (probably this compiled function is restored from
                boot image and mc is thrown away)")
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
pub struct CompiledCallsite {
    pub exceptional_destination: Option<Address>,
    pub stack_args_size: usize,
    pub callee_saved_registers: Arc<HashMap<isize, isize>>,
    pub function_version: MuID
}
impl CompiledCallsite {
    pub fn new(callsite: &Callsite, fv: MuID, callee_saved_registers: Arc<HashMap<isize, isize>>) -> CompiledCallsite {
        CompiledCallsite {
            exceptional_destination: match &callsite.exception_destination {
                &Some(ref name) => Some(resolve_symbol(name.clone())),
                &None => None
            },
            stack_args_size: callsite.stack_arg_size,
            callee_saved_registers: callee_saved_registers,
            function_version: fv,
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
    fn replace_branch_dest(&mut self, inst: usize, new_dest: &str, succ: usize);
    /// set an instruction as nop
    fn set_inst_nop(&mut self, index: usize);
    /// remove unnecessary push/pop if the callee saved register is not used
    /// returns what registers push/pop have been deleted, and the number of callee saved registers
    /// that weren't deleted
    fn remove_unnecessary_callee_saved(&mut self, used_callee_saved: Vec<MuID>) -> HashSet<MuID>;
    /// patch frame size
    fn patch_frame_size(&mut self, size: usize);

    fn as_any(&self) -> &Any;
}