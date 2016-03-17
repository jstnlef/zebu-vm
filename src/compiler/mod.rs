use ast::ir::*;
use ast::ptr::*;
use vm::context::VMContext;

use std::cell::RefCell;

pub mod passes;

pub struct Compiler {
    policy: RefCell<CompilerPolicy>
}

impl Compiler {
    pub fn new(policy: CompilerPolicy) -> Compiler {
        Compiler{policy: RefCell::new(policy)}
    }
    
    pub fn compile(&self, vm: &VMContext, func: &mut MuFunction) {
        for pass in self.policy.borrow_mut().passes.iter_mut() {
            pass.execute(vm, func);
        }
    }
}

pub struct CompilerPolicy {
    passes: Vec<Box<CompilerPass>>
}

impl CompilerPolicy {
    pub fn default() -> CompilerPolicy {
        let mut passes : Vec<Box<CompilerPass>> = vec![];
        passes.push(Box::new(passes::def_use::DefUsePass::new("DefUse")));
        passes.push(Box::new(passes::tree_gen::TreeGenerationPass::new("Tree Generation")));
        
        CompilerPolicy{passes: passes}
    }
}

pub trait CompilerPass {
    fn name(&self) -> &'static str;
    
    fn execute(&mut self, vm_context: &VMContext, func: &mut MuFunction) {
        debug!("---CompilerPass {} for {}---", self.name(), func.fn_name);
        
        self.visit_function(vm_context, func);
        
        for entry in func.blocks.iter_mut() {
            let label : MuTag = entry.0;
            let ref mut block : &mut Block = &mut entry.1;
            
            debug!("block: {}", label);
            
            for inst in block.content.as_mut().unwrap().body.iter_mut() {
                debug!("{:?}", inst);
                
                self.visit_inst(vm_context, inst);
            }
            
            debug!("---finish---");
        }
    }
    
    fn visit_function(&mut self, vm_context: &VMContext, func: &mut MuFunction) {}
    fn visit_block(&mut self, vm_context: &VMContext, block: &mut Block) {}
    fn visit_inst(&mut self, vm_context: &VMContext, node: &mut TreeNode) {}
}
