#![allow(dead_code)]

use ast::ir::*;
use vm::VM;
use compiler;
use compiler::CompilerPass;
use compiler::PassExecutionResult;
use compiler::backend::init_machine_regs_for_func;
use compiler::backend;

use std::collections::HashMap;

mod graph_coloring;

pub enum RegAllocFailure {
    FailedForSpilling,
}

pub struct RegisterAllocation {
    name: &'static str,
}

impl RegisterAllocation {
    pub fn new() -> RegisterAllocation {
        RegisterAllocation {
            name: "Register Allcoation",
        }
    }
    
    #[allow(unused_variables)]
    fn coloring(&mut self, vm: &VM, func: &mut MuFunctionVersion) -> Result<(), RegAllocFailure> {
        let compiled_funcs = vm.compiled_funcs().read().unwrap();
        let mut cf = compiled_funcs.get(&func.id()).unwrap().write().unwrap();
        
        cf.mc().trace_mc();
        
        // initialize machine registers for the function context
        init_machine_regs_for_func(&mut func.context);
        
        let coloring = match graph_coloring::GraphColoring::start(func, &mut cf, vm) {
            Ok(coloring) => coloring,
            Err(_) => panic!("error during coloring - unexpected")
        };
        
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
        
        match self.coloring(vm, func) {
            // skip slow path
            Ok(_) => PassExecutionResult::ProceedTo(compiler::PASS_PEEPHOLE),

            // go back to instruction selection for spilled operands
            Err(RegAllocFailure::FailedForSpilling) => PassExecutionResult::GoBackTo(compiler::PASS_INST_SEL),
        }
    }
}
