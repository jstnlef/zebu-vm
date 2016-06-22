use ast::ir::*;

pub struct CompiledFunction {
    pub fn_name: MuTag,
    pub mc: Box<MachineCode>
}

pub trait MachineCode {
    fn print(&self);
    
    fn number_of_insts(&self) -> usize;
    
    fn is_move(&self, index: usize) -> bool;
    fn get_succs(&self, index: usize) -> &Vec<usize>;
    fn get_preds(&self, index: usize) -> &Vec<usize>;
    
    fn get_inst_reg_uses(&self, index: usize) -> &Vec<MuID>;
    fn get_inst_reg_defines(&self, index: usize) -> &Vec<MuID>;
    
    fn replace_reg(&mut self, from: MuID, to: MuID);
}