use ast::ir::*;
use ast::ptr::*;
use compiler::frame::*;
use runtime::ValueLocation;

use std::ops;
use std::collections::HashMap;

use rustc_serialize::{Encodable, Encoder, Decodable, Decoder};

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

const CF_SERIALIZE_FIELDS : usize = 6;

impl Encodable for CompiledFunction {
    fn encode<S: Encoder> (&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_struct("CompiledFunction", CF_SERIALIZE_FIELDS, |s| {
            let mut i = 0;

            try!(s.emit_struct_field("func_id",     i, |s| self.func_id.encode(s)));
            i += 1;

            try!(s.emit_struct_field("func_ver_id", i, |s| self.func_ver_id.encode(s)));
            i += 1;

            try!(s.emit_struct_field("temps",       i, |s| self.temps.encode(s)));
            i += 1;

            try!(s.emit_struct_field("consts",      i, |s| self.consts.encode(s)));
            i += 1;

            try!(s.emit_struct_field("const_mem",   i, |s| self.const_mem.encode(s)));
            i += 1;

            try!(s.emit_struct_field("frame",       i, |s| self.frame.encode(s)));
            i += 1;

            try!(s.emit_struct_field("start",       i, |s| self.start.encode(s)));
            i += 1;

            try!(s.emit_struct_field("end",         i, |s| self.end.encode(s)));
            
            Ok(())
        })
    }
}

impl Decodable for CompiledFunction {
    fn decode<D: Decoder>(d: &mut D) -> Result<CompiledFunction, D::Error> {
        d.read_struct("CompiledFunction", CF_SERIALIZE_FIELDS, |d| {
            let mut i = 0;

            let func_id = 
                try!(d.read_struct_field("func_id",     i, |d| Decodable::decode(d)));
            i += 1;
            let func_ver_id = 
                try!(d.read_struct_field("func_ver_id", i, |d| Decodable::decode(d)));
            i += 1;
            let temps = 
                try!(d.read_struct_field("temps",       i, |d| Decodable::decode(d)));
            i += 1;
            let consts =
                try!(d.read_struct_field("consts",      i, |d| Decodable::decode(d)));
            i += 1;
            let const_mem =
                try!(d.read_struct_field("const_mem",   i, |d| Decodable::decode(d)));
            i += 1;
            let frame = 
                try!(d.read_struct_field("frame",       i, |d| Decodable::decode(d)));
            i += 1;
            let start = 
                try!(d.read_struct_field("start",       i, |d| Decodable::decode(d)));
            i += 1;
            let end =
                try!(d.read_struct_field("end",         i, |d| Decodable::decode(d)));
            
            Ok(CompiledFunction{
                func_id: func_id,
                func_ver_id: func_ver_id,
                temps: temps,
                consts: consts,
                const_mem: const_mem,
                mc: None,
                frame: frame,
                start: start,
                end: end
            })
        })
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
    /// returns what registers push/pop have been deleted
    fn remove_unnecessary_callee_saved(&mut self, used_callee_saved: Vec<MuID>) -> Vec<MuID>;
    /// patch frame size
    fn patch_frame_size(&mut self, size: usize, size_used: usize);

    fn as_any(&self) -> &Any;
}

pub trait MachineInst {

}
