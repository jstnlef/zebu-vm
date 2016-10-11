extern crate hprof;

use ast::ir::*;
use vm::VM;

use std::cell::RefCell;
use std::sync::Arc;

pub mod passes;
pub mod backend;
pub mod frame;
pub mod machine_code;

pub use compiler::passes::CompilerPass;
pub use compiler::passes::PassExecutionResult;
pub use compiler::passes::PASS_IR_CHECK;
pub use compiler::passes::PASS_DEF_USE;
pub use compiler::passes::PASS_TREE_GEN;
pub use compiler::passes::PASS_CFA;
pub use compiler::passes::PASS_TRACE_GEN;
pub use compiler::passes::PASS_FAST_INST_SEL;
pub use compiler::passes::PASS_FAST_REG_ALLOC;
pub use compiler::passes::PASS_SLOW_INST_SEL;
pub use compiler::passes::PASS_SLOW_REG_ALLOC;
pub use compiler::passes::PASS_PEEPHOLE;
pub use compiler::passes::PASS_CODE_EMIT;

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
        trace!("{:?}", func);
        
        // FIXME: should use function name here (however hprof::enter only accept &'static str)
        let _p = hprof::enter("Function Compilation");

        let mut cur_pass = 0;
        let n_passes = self.policy.borrow().passes.len();

        let ref mut passes = self.policy.borrow_mut().passes;

        while cur_pass < n_passes {
            let _p = hprof::enter(passes[cur_pass].name());
            let result = passes[cur_pass].execute(&self.vm, func);

            match result {
                PassExecutionResult::ProceedToNext => cur_pass += 1,
                PassExecutionResult::ProceedTo(next)
                | PassExecutionResult::GoBackTo(next) => cur_pass = next.get()
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
    pub fn new(passes: Vec<Box<CompilerPass>>) -> CompilerPolicy {
        CompilerPolicy{passes: passes}
    }
}

impl Default for CompilerPolicy {
    fn default() -> Self {
        let mut passes : Vec<Box<CompilerPass>> = vec![];
        // ir level passes
        passes.push(Box::new(passes::DefUse::new()));
        passes.push(Box::new(passes::TreeGen::new()));
        passes.push(Box::new(passes::ControlFlowAnalysis::new()));
        passes.push(Box::new(passes::TraceGen::new()));

        // fast path compilation - use callee saved registers only
        passes.push(Box::new(backend::inst_sel::InstructionSelection::new(true)));
        passes.push(Box::new(backend::reg_alloc::RegisterAllocation::new(true)));
        // slow path compilation - use all registers
        passes.push(Box::new(backend::inst_sel::InstructionSelection::new(false)));
        passes.push(Box::new(backend::reg_alloc::RegisterAllocation::new(false)));

        // machine code level passes
        passes.push(Box::new(backend::peephole_opt::PeepholeOptimization::new()));
        passes.push(Box::new(backend::code_emission::CodeEmission::new()));

        CompilerPolicy{passes: passes}
    }
}