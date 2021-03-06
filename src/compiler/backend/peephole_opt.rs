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

use compiler::CompilerPass;
use ast::ir::*;
use vm::VM;
use compiler::machine_code::CompiledFunction;
use compiler::backend;

use std::any::Any;

pub struct PeepholeOptimization {
    name: &'static str
}

impl CompilerPass for PeepholeOptimization {
    fn name(&self) -> &'static str {
        self.name
    }

    fn as_any(&self) -> &Any {
        self
    }

    fn visit_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        let compiled_funcs = vm.compiled_funcs().read().unwrap();
        let mut cf = compiled_funcs.get(&func.id()).unwrap().write().unwrap();

        // remove redundant move first
        for i in 0..cf.mc().number_of_insts() {
            cf.mc().trace_inst(i);

            // if two sides of a move instruction are the same,
            // it is redundant, and can be eliminated
            trace!("trying to remove redundant move");
            self.remove_redundant_move(i, &mut cf);
        }

        // then remove jumps (because removing movs will affect this)
        for i in 0..cf.mc().number_of_insts() {
            cf.mc().trace_inst(i);

            // if a branch jumps a label that contains another jump, such as
            // ..
            //   jmp L1
            // ..
            // L1:
            //   jmp L2
            // ..
            // we can rewrite first branch to jump to L2 directly

            // the order matters: we need to run this first, then remove_unnecessary_jump()
            // as this will give us more chances to remove unnecessary jumps
            trace!("trying to remove jump-to-jump");
            self.remove_jump_to_jump(i, &mut cf);

            // if a branch targets a block that immediately follow it, it can be eliminated
            trace!("trying to remove unnecessary jmp");
            self.remove_unnecessary_jump(i, &mut cf);
        }

        trace!("after peephole optimization:");
        cf.mc().trace_mc();
    }
}

impl PeepholeOptimization {
    pub fn new() -> PeepholeOptimization {
        PeepholeOptimization {
            name: "Peephole Optimization"
        }
    }

    fn remove_redundant_move(&mut self, inst: usize, cf: &mut CompiledFunction) {
        // if this instruction is a move, and move from register to register (no memory operands)
        if cf.mc().is_move(inst) && !cf.mc().is_using_mem_op(inst) {
            // get source reg/temp ID
            let src: MuID = {
                let uses = cf.mc().get_inst_reg_uses(inst);
                if uses.len() == 0 {
                    // moving immediate to register, its not redundant
                    return;
                }
                uses[0]
            };

            // get dest reg/temp ID
            let dst: MuID = cf.mc().get_inst_reg_defines(inst)[0];

            // turning temp into machine reg
            let src_machine_reg: MuID = {
                match cf.temps.get(&src) {
                    Some(reg) => *reg,
                    None => src
                }
            };
            let dst_machine_reg: MuID = {
                match cf.temps.get(&dst) {
                    Some(reg) => *reg,
                    None => dst
                }
            };

            // check if two registers are aliased
            if backend::is_aliased(src_machine_reg, dst_machine_reg) {
                info!(
                    "move between {} and {} is redundant! removed",
                    src_machine_reg,
                    dst_machine_reg
                );
                // redundant, remove this move
                cf.mc_mut().set_inst_nop(inst);
            } else {

            }
        }
    }

    fn remove_unnecessary_jump(&mut self, inst: usize, cf: &mut CompiledFunction) {
        let mc = cf.mc_mut();

        // if this is last instruction, return
        if inst == mc.number_of_insts() - 1 {
            return;
        }

        // if this inst jumps to a label that directly follows it, we can set it to nop
        let opt_dest = mc.is_jmp(inst);

        match opt_dest {
            Some(ref dest) => {
                let opt_label = mc.is_label(inst + 1);
                match opt_label {
                    Some(ref label) if dest == label => {
                        info!("inst {}'s jmp to {} is unnecessary! removed", inst, label);
                        mc.set_inst_nop(inst);
                    }
                    _ => {
                        // do nothing
                    }
                }
            }
            None => {
                // do nothing
            }
        }
    }

    fn remove_jump_to_jump(&mut self, inst: usize, cf: &mut CompiledFunction) {
        let mc = cf.mc_mut();

        // the instruction that we may rewrite
        let orig_inst = inst;
        // the destination we will rewrite the instruction to branch to
        let dests: Option<(MuName, MuName)> = {
            use std::collections::HashSet;

            let mut cur_inst = inst;
            let mut last_dest = None;

            let mut visited_labels = HashSet::new();

            loop {
                let opt_dest = mc.is_jmp(cur_inst);
                match opt_dest {
                    Some(ref dest) => {
                        trace!("current instruction {} jumps to {}", cur_inst, dest);
                        // if we have already visited this instruction
                        // this means we met an infinite loop, we need to break
                        if visited_labels.contains(dest) {
                            warn!("met an infinite loop in removing jump-to-jump");
                            warn!("we are not optimizing this case");
                            return;
                        } else {
                            visited_labels.insert(dest.clone());
                            debug!("visited {}", dest);
                        }

                        // get the block for destination
                        let first_inst = {
                            let start = mc.get_block_range(dest).unwrap().start;
                            let last = mc.number_of_insts();

                            let mut first = start;
                            for i in start..last {
                                if mc.is_label(i).is_some() || mc.is_nop(i) {
                                    continue;
                                } else {
                                    first = i;
                                    break;
                                }
                            }

                            first
                        };

                        trace!(
                            "examining first valid inst {} from block {}",
                            first_inst,
                            dest
                        );

                        // if first instruction is jump
                        match mc.is_jmp(first_inst) {
                            Some(ref dest2) => {
                                // its a jump-to-jump case
                                cur_inst = first_inst;
                                last_dest = Some((dest.clone(), dest2.clone()));
                            }
                            None => break
                        }
                    }
                    None => break
                }
            }
            last_dest
        };

        if let Some((old_dest, final_dest)) = dests {
            let first_inst = mc.get_block_range(&final_dest).unwrap().start;
            let old_first_inst = mc.get_block_range(&old_dest).unwrap().start;

            info!(
                "inst {} chain jumps to {}, rewrite as branching to {} (successor: {})",
                orig_inst,
                final_dest,
                final_dest,
                first_inst
            );
            mc.replace_branch_dest(inst, old_first_inst, &final_dest, first_inst);
        }
    }
}
