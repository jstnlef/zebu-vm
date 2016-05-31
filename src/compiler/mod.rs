use ast::ir::*;
use vm::context::VMContext;

use std::cell::RefCell;
use std::sync::Arc;

pub mod passes;
pub mod backend;

pub struct Compiler {
    policy: RefCell<CompilerPolicy>,
    vm: Arc<VMContext>
}

impl Compiler {
    pub fn new(policy: CompilerPolicy, vm: Arc<VMContext>) -> Compiler {
        Compiler{
            policy: RefCell::new(policy),
            vm: vm
        }
    }
    
    pub fn compile(&self, func: &mut MuFunction) {
        for pass in self.policy.borrow_mut().passes.iter_mut() {
            pass.execute(&self.vm, func);
        }
    }
}

pub struct CompilerPolicy {
    passes: Vec<Box<CompilerPass>>
}

impl CompilerPolicy {
    pub fn default() -> CompilerPolicy {
        let mut passes : Vec<Box<CompilerPass>> = vec![];
        passes.push(Box::new(passes::DefUse::new()));
        passes.push(Box::new(passes::TreeGen::new()));
        passes.push(Box::new(passes::ControlFlowAnalysis::new()));
        passes.push(Box::new(passes::TraceGen::new()));
        passes.push(Box::new(backend::inst_sel::InstructionSelection::new()));
        
        CompilerPolicy{passes: passes}
    }
    
    pub fn new(passes: Vec<Box<CompilerPass>>) -> CompilerPolicy {
        CompilerPolicy{passes: passes}
    }
}

#[allow(unused_variables)]
pub trait CompilerPass {
    fn name(&self) -> &'static str;
    
    fn execute(&mut self, vm_context: &VMContext, func: &mut MuFunction) {
        debug!("---CompilerPass {} for {}---", self.name(), func.fn_name);
        
        self.start_function(vm_context, func);
        self.visit_function(vm_context, func);
        self.finish_function(vm_context, func);        
        
        debug!("---finish---");
    }
    
    fn visit_function(&mut self, vm_context: &VMContext, func: &mut MuFunction) {
        for (label, ref mut block) in func.content.as_mut().unwrap().blocks.iter_mut() {
            debug!("block: {}", label);
            
            self.start_block(vm_context, &mut func.context, block);
            self.visit_block(vm_context, &mut func.context, block);
            self.finish_block(vm_context, &mut func.context, block);
        }
    }
    
    fn visit_block(&mut self, vm_context: &VMContext, func_context: &mut FunctionContext, block: &mut Block) {
        for inst in block.content.as_mut().unwrap().body.iter_mut() {
            debug!("{}", inst);
            
            self.visit_inst(vm_context, func_context, inst);
        }    
    }
    
    fn start_function(&mut self, vm_context: &VMContext, func: &mut MuFunction) {}
    fn finish_function(&mut self, vm_context: &VMContext, func: &mut MuFunction) {}
    
    fn start_block(&mut self, vm_context: &VMContext, func_context: &mut FunctionContext, block: &mut Block) {}
    fn finish_block(&mut self, vm_context: &VMContext, func_context: &mut FunctionContext, block: &mut Block) {}
    
    fn visit_inst(&mut self, vm_context: &VMContext, func_context: &mut FunctionContext, node: &mut TreeNode) {}
}
