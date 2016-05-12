pub mod inst_sel;

mod codegen;
pub use compiler::backend::x86_64::codegen::CodeGenerator;

mod asm_backend;
pub use compiler::backend::x86_64::asm_backend::ASMCodeGen;

use ast::ptr::P;
use ast::ir::*;

pub fn is_valid_x86_imm(op: &P<Value>) -> bool {
    use std::u32;
    match op.v {
        Value_::Constant(Constant::Int(val)) if val <= u32::MAX as usize => {
            true
        },
        _ => false
    }
}