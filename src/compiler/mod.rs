use ast::ir::*;
use std::cell::RefCell;

pub mod passes;

pub struct Compiler {
    policy: RefCell<CompilerPolicy>
}

impl Compiler {
    pub fn new(policy: CompilerPolicy) -> Compiler {
        Compiler{policy: RefCell::new(policy)}
    }
    
    pub fn compile(&self, func: &mut MuFunction) {
        for pass in self.policy.borrow_mut().passes.iter_mut() {
            pass.execute(func);
        }
    }
}

pub struct CompilerPolicy {
    passes: Vec<Box<CompilerPass>>
}

impl CompilerPolicy {
    pub fn default() -> CompilerPolicy {
        let mut passes : Vec<Box<CompilerPass>> = vec![];
        passes.push(Box::new(passes::tree_gen::TreeGenerationPass::new()));
        
        CompilerPolicy{passes: passes}
    }
}

pub trait CompilerPass {
    fn execute(&mut self, func: &mut MuFunction);
}
