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

pub struct Compiler<'vm> {
    policy: RefCell<CompilerPolicy>,
    vm: &'vm VM
}

impl <'vm> Compiler<'vm> {
    pub fn new(policy: CompilerPolicy, vm: &VM) -> Compiler {
        Compiler{
            policy: RefCell::new(policy),
            vm: vm
        }
    }

    pub fn compile(&self, func: &mut MuFunctionVersion) {
        trace!("{:?}", func);
        
        // FIXME: should use function name here (however hprof::enter only accept &'static str)
        let _p = hprof::enter("Function Compilation");

        let ref mut passes = self.policy.borrow_mut().passes;

        for pass in passes.iter_mut() {
            let _p = hprof::enter(pass.name());

            pass.execute(self.vm, func);

            drop(_p);
        }

        drop(_p);
        hprof::profiler().print_timing();

        func.set_compiled();
    }

    pub fn get_policy(&self) -> &RefCell<CompilerPolicy> {
        &self.policy
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
        passes.push(Box::new(passes::Inlining::new()));
        // ir level passes
        passes.push(Box::new(passes::DefUse::new()));
        passes.push(Box::new(passes::TreeGen::new()));
        passes.push(Box::new(passes::GenMovPhi::new()));
        passes.push(Box::new(passes::ControlFlowAnalysis::new()));
        passes.push(Box::new(passes::TraceGen::new()));

        // compilation
        passes.push(Box::new(backend::inst_sel::InstructionSelection::new()));
        passes.push(Box::new(backend::reg_alloc::RegisterAllocation::new()));

        // machine code level passes
        passes.push(Box::new(backend::peephole_opt::PeepholeOptimization::new()));
        passes.push(Box::new(backend::code_emission::CodeEmission::new()));

        CompilerPolicy{passes: passes}
    }
}
