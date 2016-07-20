use ast::ir::*;
use vm::VMContext;

mod def_use;
mod tree_gen;
mod control_flow;
mod trace_gen;

pub use compiler::passes::def_use::DefUse;
pub use compiler::passes::tree_gen::TreeGen;
pub use compiler::passes::control_flow::ControlFlowAnalysis;
pub use compiler::passes::trace_gen::TraceGen;

pub const PASS0_DEF_USE   : usize = 0;
pub const PASS1_TREE_GEN  : usize = 1;
pub const PASS2_CFA       : usize = 2;
pub const PASS3_TRACE_GEN : usize = 3;
pub const PASS4_INST_SEL  : usize = 4;
pub const PASS5_REG_ALLOC : usize = 5;
pub const PASS6_PEEPHOLE  : usize = 6;
pub const PASS7_CODE_EMIT : usize = 7;

pub enum PassExecutionResult {
    ProceedToNext,
    GoBackTo(usize)
}

#[allow(unused_variables)]
pub trait CompilerPass {
    fn name(&self) -> &'static str;

    fn execute(&mut self, vm_context: &VMContext, func: &mut MuFunctionVersion) -> PassExecutionResult {
        debug!("---CompilerPass {} for {}---", self.name(), func.fn_name);

        self.start_function(vm_context, func);
        self.visit_function(vm_context, func);
        self.finish_function(vm_context, func);

        debug!("---finish---");

        PassExecutionResult::ProceedToNext
    }

    fn visit_function(&mut self, vm_context: &VMContext, func: &mut MuFunctionVersion) {
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

    fn start_function(&mut self, vm_context: &VMContext, func: &mut MuFunctionVersion) {}
    fn finish_function(&mut self, vm_context: &VMContext, func: &mut MuFunctionVersion) {}

    fn start_block(&mut self, vm_context: &VMContext, func_context: &mut FunctionContext, block: &mut Block) {}
    fn finish_block(&mut self, vm_context: &VMContext, func_context: &mut FunctionContext, block: &mut Block) {}

    fn visit_inst(&mut self, vm_context: &VMContext, func_context: &mut FunctionContext, node: &mut TreeNode) {}
}
