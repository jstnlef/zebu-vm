use ast::ir::*;
use std::ops;
use std::collections::HashMap;

pub struct CompiledFunction {
    pub func_id: MuID,
    pub func_ver_id: MuID,
    pub temps: HashMap<MuID, MuID>, // assumes one temperary maps to one register
    pub mc: Box<MachineCode>
}

pub trait MachineCode {
    fn trace_mc(&self);
    fn trace_inst(&self, index: usize);
    
    fn emit(&self) -> Vec<u8>;
    
    fn number_of_insts(&self) -> usize;
    
    fn is_move(&self, index: usize) -> bool;
    fn is_using_mem_op(&self, index: usize) -> bool;
    
    fn get_succs(&self, index: usize) -> &Vec<usize>;
    fn get_preds(&self, index: usize) -> &Vec<usize>;
    
    fn get_inst_reg_uses(&self, index: usize) -> &Vec<MuID>;
    fn get_inst_reg_defines(&self, index: usize) -> &Vec<MuID>;
    
    fn get_ir_block_livein(&self, block: &str) -> Option<&Vec<MuID>>;
    fn get_ir_block_liveout(&self, block: &str) -> Option<&Vec<MuID>>;
    
    fn get_all_blocks(&self) -> &Vec<MuName>;
    fn get_block_range(&self, block: &str) -> Option<ops::Range<usize>>;
    
    fn replace_reg(&mut self, from: MuID, to: MuID);
    fn set_inst_nop(&mut self, index: usize);
}
