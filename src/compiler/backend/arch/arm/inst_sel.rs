use ast::ir::*;
use ast::inst::Instruction_::*;
use vm::context::VMContext;

use compiler::CompilerPass;

pub struct InstructionSelection {
    name: &'static str
}

impl InstructionSelection {
    pub fn new() -> InstructionSelection {
        InstructionSelection{name: "Instruction Selection (ARM)"}
    }
}

impl CompilerPass for InstructionSelection {
    fn name(&self) -> &'static str {
        self.name
    }
    
    #[allow(unused_variables)]
    fn start_function(&mut self, vm_context: &VMContext, func: &mut MuFunction) {
        println!("{}", self.name());
    }
}