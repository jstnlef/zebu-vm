#![allow(dead_code)]

use compiler;
use compiler::CompilerPass;
use compiler::PassExecutionResult;
use ast::ir::*;
use vm::VM;

use compiler::backend::init_machine_regs_for_func;

mod graph_coloring;

pub enum RegAllocFailure {
    FailedForSpilling,
    FailedForUsingCallerSaved
}

pub struct RegisterAllocation {
    name: &'static str,
    is_fastpath: bool
}

impl RegisterAllocation {
    pub fn new(is_fastpath: bool) -> RegisterAllocation {
        RegisterAllocation {
            name: "Register Allcoation",
            is_fastpath: is_fastpath
        }
    }
    
    #[allow(unused_variables)]
    // returns true if we spill registers (which requires another instruction selection)
    fn coloring(&mut self, vm: &VM, func: &mut MuFunctionVersion) -> Result<(), RegAllocFailure> {
        let compiled_funcs = vm.compiled_funcs().read().unwrap();
        let mut cf = compiled_funcs.get(&func.id()).unwrap().write().unwrap();
        
        cf.mc().trace_mc();
        
        // initialize machine registers for the function context
        init_machine_regs_for_func(&mut func.context);
        
        let liveness = graph_coloring::build_inteference_graph(&mut cf, func);
        liveness.print();

        let coloring = match graph_coloring::GraphColoring::start(liveness) {
            Ok(coloring) => coloring,
            Err(err) => {
                return Err(err);
            }
        };

        let spills = coloring.spills();
        
        if !spills.is_empty() {
            return Err(RegAllocFailure::FailedForSpilling);
        }
        
        // replace regs
        trace!("Replacing Registers...");
        for node in coloring.ig.nodes() {
            let temp = coloring.ig.get_temp_of(node);
            
            // skip machine registers
            if temp < MACHINE_ID_END {
                continue;
            } else {
                let alias = coloring.get_alias(node);
                let machine_reg = coloring.ig.get_color_of(alias).unwrap();
                
                trace!("replacing {} with {}", temp, machine_reg);
                cf.mc_mut().replace_reg(temp, machine_reg);
                
                cf.temps.insert(temp, machine_reg);
            }
        }
        
        cf.mc().trace_mc();
        
        Ok(())
    }    
}

impl CompilerPass for RegisterAllocation {
    fn name(&self) -> &'static str {
        self.name
    }
    
    fn execute(&mut self, vm: &VM, func: &mut MuFunctionVersion) -> PassExecutionResult {
        debug!("---CompilerPass {} for {}---", self.name(), func);

        if !self.is_fastpath {
            unimplemented!()
        }
        
        match self.coloring(vm, func) {
            // skip slow path
            Ok(_) => PassExecutionResult::ProceedTo(compiler::PASS_PEEPHOLE),

            // go back to instruction selection for spilled operands
            Err(RegAllocFailure::FailedForSpilling) => PassExecutionResult::GoBackTo(compiler::PASS_FAST_INST_SEL),

            // proceed to slow path
            Err(RegAllocFailure::FailedForUsingCallerSaved) => PassExecutionResult::ProceedToNext
        }
    }
}
