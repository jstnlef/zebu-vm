use ast::ir::*;
use compiler::backend::Temporary;

use std::collections::HashMap;

pub struct CompiledFunction {
    pub fn_name: MuTag,
    pub temps: HashMap<MuID, Temporary>
}