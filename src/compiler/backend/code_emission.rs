#![allow(dead_code)]

use compiler::CompilerPass;
use ast::ir::*;
use vm::VM;
use compiler::backend::emit_code;

pub struct CodeEmission {
    name: &'static str
}

impl CodeEmission {
    pub fn new() -> CodeEmission {
        CodeEmission {
            name: "Code Emission"
        }
    }
}

impl CompilerPass for CodeEmission {
    fn name(&self) -> &'static str {
        self.name
    }

    fn visit_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        emit_code(func, vm);
    }
}
