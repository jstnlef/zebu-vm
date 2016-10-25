mod liveness;
mod coloring;

pub use compiler::backend::reg_alloc::graph_coloring::liveness::InterferenceGraph;
//pub use compiler::backend::reg_alloc::graph_coloring::liveness::build as build_inteference_graph;
pub use compiler::backend::reg_alloc::graph_coloring::liveness::build_chaitin_briggs as build_inteference_graph;
pub use compiler::backend::reg_alloc::graph_coloring::coloring::GraphColoring;

use ast::ir::*;
use vm::VM;
use compiler::CompilerPass;
use compiler::backend::is_callee_saved;
use compiler::backend::init_machine_regs_for_func;
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
    fn coloring(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        let compiled_funcs = vm.compiled_funcs().read().unwrap();
        let mut cf = compiled_funcs.get(&func.id()).unwrap().write().unwrap();

        // initialize machine registers for the function context
        init_machine_regs_for_func(&mut func.context);

        let coloring = GraphColoring::start(func, &mut cf, vm);

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

        // find out what callee saved registers are used
        {
            use std::collections::HashSet;

            let used_callee_saved: HashSet<MuID> =
                coloring.cf.temps.values()
                    .map(|x| *x)
                    .filter(|x| is_callee_saved(*x))
                    .collect();

            let used_callee_saved: Vec<MuID> = used_callee_saved.into_iter().collect();

            let removed_callee_saved = coloring.cf.mc_mut().remove_unnecessary_callee_saved(used_callee_saved);
            for reg in removed_callee_saved {
                coloring.cf.frame.remove_record_for_callee_saved_reg(reg);
            }
        }

        coloring.cf.mc().trace_mc();
    }
}

impl CompilerPass for RegisterAllocation {
    fn name(&self) -> &'static str {
        self.name
    }

    fn as_any(&self) -> &Any {
        self
    }

    fn visit_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        self.coloring(vm, func);
    }
}
