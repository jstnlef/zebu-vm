use ast::ir::*;
use ast::ptr::*;
use ast::inst::*;
use vm::VM;

use compiler::CompilerPass;
use std::any::Any;
use std::sync::RwLock;
use std::collections::HashMap;

pub struct Inlining {
    name: &'static str,

    // whether a function version should be inlined
    should_inline: HashMap<MuID, bool>
}

impl Inlining {
    pub fn new() -> Inlining {
        Inlining{
            name: "Inlining",
            should_inline: HashMap::new()
        }
    }

    fn check(&mut self, vm: &VM, func: &mut MuFunctionVersion) -> bool {
        unimplemented!()
    }

    fn inline(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        unimplemented!()
    }
}

impl CompilerPass for Inlining {
    fn name(&self) -> &'static str {
        self.name
    }

    fn as_any(&self) -> &Any {
        self
    }

    fn visit_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        if self.check(vm, func) {
            self.inline(vm, func);
        }
    }
}