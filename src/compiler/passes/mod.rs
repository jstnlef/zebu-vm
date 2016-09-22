use ast::ir::*;
use vm::VM;

mod def_use;
mod tree_gen;
mod control_flow;
mod trace_gen;

pub use compiler::passes::def_use::DefUse;
pub use compiler::passes::tree_gen::TreeGen;
pub use compiler::passes::control_flow::ControlFlowAnalysis;
pub use compiler::passes::trace_gen::TraceGen;

pub const PASS_IR_CHECK  : usize = 0;
pub const PASS_DEF_USE   : usize = 1;
pub const PASS_TREE_GEN  : usize = 2;
pub const PASS_CFA       : usize = 3;
pub const PASS_TRACE_GEN : usize = 4;
pub const PASS_INST_SEL  : usize = 5;
pub const PASS_REG_ALLOC : usize = 6;
pub const PASS_PEEPHOLE  : usize = 7;
pub const PASS_CODE_EMIT : usize = 8;

pub enum PassExecutionResult {
    ProceedToNext,
    GoBackTo(usize)
}

#[allow(unused_variables)]
pub trait CompilerPass {
    fn name(&self) -> &'static str;

    fn execute(&mut self, vm: &VM, func: &mut MuFunctionVersion) -> PassExecutionResult {
        debug!("---CompilerPass {} for {}---", self.name(), func);

        self.start_function(vm, func);
        self.visit_function(vm, func);
        self.finish_function(vm, func);

        debug!("---finish---");

        PassExecutionResult::ProceedToNext
    }

    fn visit_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        for (label, ref mut block) in func.content.as_mut().unwrap().blocks.iter_mut() {
            debug!("block: {}", label);

            self.start_block(vm, &mut func.context, block);
            self.visit_block(vm, &mut func.context, block);
            self.finish_block(vm, &mut func.context, block);
        }
    }

    fn visit_block(&mut self, vm: &VM, func_context: &mut FunctionContext, block: &mut Block) {
        for inst in block.content.as_mut().unwrap().body.iter_mut() {
            debug!("{}", inst);

            self.visit_inst(vm, func_context, inst);
        }
    }

    fn start_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {}
    fn finish_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {}

    fn start_block(&mut self, vm: &VM, func_context: &mut FunctionContext, block: &mut Block) {}
    fn finish_block(&mut self, vm: &VM, func_context: &mut FunctionContext, block: &mut Block) {}

    fn visit_inst(&mut self, vm: &VM, func_context: &mut FunctionContext, node: &TreeNode) {}
}
