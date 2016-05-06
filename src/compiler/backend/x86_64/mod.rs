pub mod inst_sel;

mod codegen;
pub use compiler::backend::x86_64::codegen::CodeGenerator;

mod asm_backend;
pub use compiler::backend::x86_64::asm_backend::ASMCodeGen;

use ast::ptr::P;
use ast::ir::*;
use ast::types::*;
pub fn is_valid_x86_imm(op: &P<Value>) -> bool {
    let ty : &MuType_ = &op.ty;
    match ty {
        &MuType_::Int(len) if len <= 32 => true,
        _ => false
    }
}