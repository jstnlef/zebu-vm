// Copyright 2017 The Australian National University
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate petgraph;

mod liveness;
mod coloring;

use compiler::backend::reg_alloc::graph_coloring::liveness::build_interference_graph_chaitin_briggs
as build_inteference_graph;
use compiler::backend::reg_alloc::graph_coloring::coloring::GraphColoring;

use ast::ir::*;
use vm::VM;
use compiler::CompilerPass;
use compiler::backend::is_callee_saved;
use compiler::backend::init_machine_regs_for_func;
use compiler::backend::reg_alloc::validate;
use std::any::Any;

pub struct RegisterAllocation {
    name: &'static str
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

impl RegisterAllocation {
    pub fn new() -> RegisterAllocation {
        RegisterAllocation {
            name: "Register Allocation"
        }
    }

    fn coloring(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        // get compiled function
        let compiled_funcs = vm.compiled_funcs().read().unwrap();
        let mut cf = compiled_funcs.get(&func.id()).unwrap().write().unwrap();

        // initialize machine registers for the function context (we are gonna use them)
        init_machine_regs_for_func(&mut func.context);

        // do graph coloring
        let coloring = GraphColoring::start(func, &mut cf, vm);

        // if we need to validate the results
        if !vm.vm_options.flag_disable_regalloc_validate {
            // a map of register assignment (from temp to machine register)
            let reg_assignment = coloring.get_assignments();
            // a map of spilled temporaries (from spilled temp to scratch temp)
            // we use this to validate spilling correctness
            let spill_scratch_temps = coloring.get_spill_scratch_temps();

            validate::validate_regalloc(&coloring.cf, reg_assignment, spill_scratch_temps);
        }

        // use the result to replace temporaries with assigned regs
        trace!("Replacing Registers...");
        for (temp, machine_reg) in coloring.get_assignments() {
            trace!("replacing {} with {}", temp, machine_reg);
            coloring.cf.mc_mut().replace_reg(temp, machine_reg);
            coloring.cf.temps.insert(temp, machine_reg);
        }

        // find out what callee saved registers are used, so we can delete unnecessary savings.
        // Currently I am only deleting those unnecessary push/pops of callee saved regs, but
        // I am not deleting the frame slots for them (so the frame size is still larger than
        // it needs to be).
        // FIXME: should fix offsets of frame slots, and patch the code. See Issue #47
        {
            // all the used callee saved registers
            let used_callee_saved: Vec<MuID> = {
                use std::collections::HashSet;
                let used_callee_saved: HashSet<MuID> = coloring
                    .cf
                    .temps
                    .values()
                    .map(|x| *x)
                    .filter(|x| is_callee_saved(*x))
                    .collect();
                used_callee_saved.into_iter().collect()
            };

            // remove unused callee saved registers
            let removed_callee_saved = coloring
                .cf
                .mc_mut()
                .remove_unnecessary_callee_saved(used_callee_saved);
            for reg in removed_callee_saved {
                coloring.cf.frame.remove_record_for_callee_saved_reg(reg);
            }

            // patch frame size
            let frame_size = coloring.cf.frame.cur_size();
            trace!(
                "patching the code to grow/shrink size of {} bytes",
                frame_size
            );
            coloring.cf.mc_mut().patch_frame_size(frame_size);
        }

        coloring.cf.mc().trace_mc();
    }
}
