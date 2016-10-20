use ast::ir::*;
use vm::VM;

mod def_use;
mod tree_gen;
mod control_flow;
mod trace_gen;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct PassID(usize);
impl PassID {pub fn get(&self) -> usize{self.0}}

pub use compiler::passes::def_use::DefUse;
pub use compiler::passes::tree_gen::TreeGen;
pub use compiler::passes::control_flow::ControlFlowAnalysis;
pub use compiler::passes::trace_gen::TraceGen;

// make sure the pass IDs are sequential
pub const PASS_IR_CHECK  : PassID = PassID(0);
pub const PASS_DEF_USE   : PassID = PassID(1);
pub const PASS_TREE_GEN  : PassID = PassID(2);
pub const PASS_CFA       : PassID = PassID(3);
pub const PASS_TRACE_GEN : PassID = PassID(4);
pub const PASS_INST_SEL  : PassID = PassID(5);
pub const PASS_REG_ALLOC : PassID = PassID(6);
pub const PASS_PEEPHOLE  : PassID = PassID(7);
pub const PASS_CODE_EMIT : PassID = PassID(8);

pub enum PassExecutionResult {
    ProceedToNext,
    ProceedTo(PassID),
    GoBackTo(PassID)
}

use std::any::Any;

#[allow(unused_variables)]
pub trait CompilerPass {
    fn name(&self) -> &'static str;
    fn as_any(&self) -> &Any;

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
