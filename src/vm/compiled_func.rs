use ast::ir::*;

use std::marker::Send;
use std::marker::Sync;

pub struct CompiledFunction {
    pub fn_name: MuTag,
    pub mc: Box<MachineCode + Send + Sync>
}

pub trait MachineCode {
    fn number_of_insts(&self) -> usize;
    fn is_move(&self, index: usize) -> bool;
    
    fn get_inst_reg_uses(&self, index: usize) -> Vec<MuID>;
    fn get_inst_reg_defines(&self, index: usize) -> Vec<MuID>;
    
    fn get_reg_uses(&self, id: MuID) -> Vec<MuID>;
    fn get_reg_defines(&self, id: MuID) -> Vec<MuID>;
}