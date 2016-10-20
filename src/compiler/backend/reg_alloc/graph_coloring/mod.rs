mod liveness;
mod coloring;

pub use compiler::backend::reg_alloc::graph_coloring::liveness::InterferenceGraph;
//pub use compiler::backend::reg_alloc::graph_coloring::liveness::build as build_inteference_graph;
pub use compiler::backend::reg_alloc::graph_coloring::liveness::build_chaitin_briggs as build_inteference_graph;
pub use compiler::backend::reg_alloc::graph_coloring::coloring::GraphColoring;

use ast::ir::*;
use vm::VM;
use compiler;
use compiler::CompilerPass;
use compiler::PassExecutionResult;
use compiler::backend::init_machine_regs_for_func;
use compiler::backend;
use compiler::backend::reg_alloc::RegAllocFailure;

use std::collections::HashMap;
use std::any::Any;

pub struct RegisterAllocation {
    name: &'static str,
}

impl RegisterAllocation {
    pub fn new() -> RegisterAllocation {
        RegisterAllocation {
            name: "Register Allocation",
        }
    }

    #[allow(unused_variables)]
    fn coloring(&mut self, vm: &VM, func: &mut MuFunctionVersion) -> Result<(), RegAllocFailure> {
        let compiled_funcs = vm.compiled_funcs().read().unwrap();
        let mut cf = compiled_funcs.get(&func.id()).unwrap().write().unwrap();

        // initialize machine registers for the function context
        init_machine_regs_for_func(&mut func.context);

        let coloring = match GraphColoring::start(func, &mut cf, vm) {
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
                let machine_reg = match coloring.ig.get_color_of(alias) {
                    Some(reg) => reg,
                    None => panic!(
                        "Reg{}/{:?} (aliased as Reg{}/{:?}) is not assigned with a color",
                        coloring.ig.get_temp_of(node), node,
                        coloring.ig.get_temp_of(alias), alias)
                };

                trace!("replacing {} with {}", temp, machine_reg);
                coloring.cf.mc_mut().replace_reg(temp, machine_reg);

                coloring.cf.temps.insert(temp, machine_reg);
            }
        }

        coloring.cf.mc().trace_mc();

        Ok(())
    }
}

impl CompilerPass for RegisterAllocation {
    fn name(&self) -> &'static str {
        self.name
    }

    fn as_any(&self) -> &Any {
        self
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
