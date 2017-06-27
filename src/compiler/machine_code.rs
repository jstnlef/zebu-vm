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
use std::ops;
use std::collections::HashMap;
use std::collections::HashSet;

pub struct CompiledFunction {
    pub func_id: MuID,
    pub func_ver_id: MuID,

    // assumes one temporary maps to one register
    pub temps : HashMap<MuID, MuID>,

    pub consts: HashMap<MuID, P<Value>>,
    pub const_mem: HashMap<MuID, P<Value>>,
    
    // not emitting this
    pub mc: Option<Box<MachineCode + Send + Sync>>,
    
    pub frame: Frame,
    pub start: ValueLocation,
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

    pub fn mc(&self) -> &Box<MachineCode + Send + Sync> {
        match self.mc {
            Some(ref mc) => mc,
            None => panic!("trying to get mc from a compiled function. 
                But machine code is None (probably this compiled function is restored from
                boot image and mc is thrown away)")
        }
    }
    
    pub fn mc_mut(&mut self) -> &mut Box<MachineCode + Send + Sync> {
        match self.mc {
            Some(ref mut mc) => mc,
            None => panic!("no mc found from a compiled function")
        }
    }
}

use std::any::Any;

pub trait MachineCode {
    fn trace_mc(&self);
    fn trace_inst(&self, index: usize);
    
    fn emit(&self) -> Vec<u8>;
    fn emit_inst(&self, index: usize) -> Vec<u8>;
    
    fn number_of_insts(&self) -> usize;
    
    fn is_move(&self, index: usize) -> bool;
    fn is_using_mem_op(&self, index: usize) -> bool;
    fn is_jmp(&self, index: usize) -> Option<MuName>;
    fn is_label(&self, index: usize) -> Option<MuName>;

    fn is_spill_load(&self, index: usize) -> Option<P<Value>>;
    fn is_spill_store(&self, index: usize) -> Option<P<Value>>;
    
    fn get_succs(&self, index: usize) -> &Vec<usize>;
    fn get_preds(&self, index: usize) -> &Vec<usize>;

    fn get_next_inst(&self, index: usize) -> Option<usize>;
    fn get_last_inst(&self, index: usize) -> Option<usize>;
    
    fn get_inst_reg_uses(&self, index: usize) -> Vec<MuID>;
    fn get_inst_reg_defines(&self, index: usize) -> Vec<MuID>;
    
    fn get_ir_block_livein(&self, block: &str) -> Option<&Vec<MuID>>;
    fn get_ir_block_liveout(&self, block: &str) -> Option<&Vec<MuID>>;
    fn set_ir_block_livein(&mut self, block: &str, set: Vec<MuID>);
    fn set_ir_block_liveout(&mut self, block: &str, set: Vec<MuID>);
    
    fn get_all_blocks(&self) -> Vec<MuName>;
    fn get_entry_block(&self) -> MuName;
    // returns [start_inst, end_inst) // end_inst not included
    fn get_block_range(&self, block: &str) -> Option<ops::Range<usize>>;
    fn get_block_for_inst(&self, index: usize) -> Option<MuName>;

    // functions for rewrite

    /// replace a temp with a machine register (to_reg must be a machine register)
    fn replace_reg(&mut self, from: MuID, to: MuID);
    /// replace a temp that is defined in the inst with another temp
    fn replace_define_tmp_for_inst(&mut self, from: MuID, to: MuID, inst: usize);
    /// replace a temp that is used in the inst with another temp
    fn replace_use_tmp_for_inst(&mut self, from: MuID, to: MuID, inst: usize);
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

pub trait MachineInst {

}
