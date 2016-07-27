extern crate hprof;

use ast::ir::*;
use vm::VM;

use std::cell::RefCell;
use std::sync::Arc;

pub mod passes;
pub mod backend;

pub use compiler::passes::CompilerPass;
pub use compiler::passes::PassExecutionResult;
pub use compiler::passes::PASS0_DEF_USE;
pub use compiler::passes::PASS1_TREE_GEN;
pub use compiler::passes::PASS2_CFA;
pub use compiler::passes::PASS3_TRACE_GEN;
pub use compiler::passes::PASS4_INST_SEL;
pub use compiler::passes::PASS5_REG_ALLOC;
pub use compiler::passes::PASS6_PEEPHOLE;
pub use compiler::passes::PASS7_CODE_EMIT;

pub struct Compiler {
    policy: RefCell<CompilerPolicy>,
    vm: Arc<VM>
}

impl Compiler {
    pub fn new(policy: CompilerPolicy, vm: Arc<VM>) -> Compiler {
        Compiler{
            policy: RefCell::new(policy),
            vm: vm
        }
    }

    pub fn compile(&self, func: &mut MuFunctionVersion) {
        let _p = hprof::enter(func.fn_name);

        let mut cur_pass = 0;
        let n_passes = self.policy.borrow().passes.len();

        let ref mut passes = self.policy.borrow_mut().passes;

        while cur_pass < n_passes {
            let _p = hprof::enter(passes[cur_pass].name());
            let result = passes[cur_pass].execute(&self.vm, func);

            match result {
                PassExecutionResult::ProceedToNext => cur_pass += 1,
                PassExecutionResult::GoBackTo(next) => cur_pass = next
            }

            drop(_p);
        }

        drop(_p);
        hprof::profiler().print_timing();
    }
}

pub struct CompilerPolicy {
    pub passes: Vec<Box<CompilerPass>>
}

impl CompilerPolicy {
    pub fn default() -> CompilerPolicy {
        let mut passes : Vec<Box<CompilerPass>> = vec![];
        passes.push(Box::new(passes::DefUse::new()));
        passes.push(Box::new(passes::TreeGen::new()));
        passes.push(Box::new(passes::ControlFlowAnalysis::new()));
        passes.push(Box::new(passes::TraceGen::new()));
        passes.push(Box::new(backend::inst_sel::InstructionSelection::new()));
        passes.push(Box::new(backend::reg_alloc::RegisterAllocation::new()));
        passes.push(Box::new(backend::peephole_opt::PeepholeOptimization::new()));
        passes.push(Box::new(backend::code_emission::CodeEmission::new()));

        CompilerPolicy{passes: passes}
    }

    pub fn new(passes: Vec<Box<CompilerPass>>) -> CompilerPolicy {
        CompilerPolicy{passes: passes}
    }
}
