extern crate petgraph;

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
use compiler::backend::reg_alloc::validate;
use utils::POINTER_SIZE;
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

        if !vm.vm_options.flag_disable_regalloc_validate {
            let reg_assignment = coloring.get_assignments();
            let reg_spilled    = coloring.get_spill_history();
            let reg_coalesced  = coloring.get_coalesced();
            let spill_scratch_temps = coloring.get_spill_scratch_temps();

            validate::validate_regalloc(&coloring.cf, &coloring.func, reg_assignment, reg_coalesced, reg_spilled, spill_scratch_temps);
        }

        // replace regs
        trace!("Replacing Registers...");
        for (temp, machine_reg) in coloring.get_assignments() {
            trace!("replacing {} with {}", temp, machine_reg);

            coloring.cf.mc_mut().replace_reg(temp, machine_reg);
            coloring.cf.temps.insert(temp, machine_reg);
        }

        // find out what callee saved registers are used
        // FIXME: current not doing this
        // reason: we generated frame slots for callee saved registers, then generated slots for spills
        // if we delete some callee saved registers, the slots for spills are not correct
        {
            use std::collections::HashSet;

            let used_callee_saved: HashSet<MuID> =
                coloring.cf.temps.values()
                    .map(|x| *x)
                    .filter(|x| is_callee_saved(*x))
                    .collect();

            let used_callee_saved: Vec<MuID> = used_callee_saved.into_iter().collect();
            let n_used_callee_saved = used_callee_saved.len();

            let removed_callee_saved = coloring.cf.mc_mut().remove_unnecessary_callee_saved(used_callee_saved);
            for reg in removed_callee_saved {
                coloring.cf.frame.remove_record_for_callee_saved_reg(reg);
            }

            // patch frame size

            // size for callee saved regs
            let size_for_callee_saved_regs = n_used_callee_saved * POINTER_SIZE;
            trace!("callee saved registers used {} bytes", size_for_callee_saved_regs);

            let total_frame_size = coloring.cf.frame.cur_size();
            trace!("frame reserved for {} bytes", total_frame_size);

            let size_to_patch = total_frame_size - size_for_callee_saved_regs;

            trace!("patching the code to grow/shrink size of {} bytes", size_to_patch);
            coloring.cf.mc_mut().patch_frame_size(size_to_patch, size_for_callee_saved_regs);
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
