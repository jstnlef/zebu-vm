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

//! This module is for validating register allocation
//! However it is disabled for now due to bugs. It is uncertain
//! how important this is to Zebu. More description and discussion
//! can be found as Issue #19.

#![allow(dead_code)]

use utils::LinkedHashMap;
use ast::ir::*;
use ast::ptr::*;
use compiler::machine_code::CompiledFunction;
use compiler::backend::get_color_for_precolored as alias;
use compiler::backend::PROLOGUE_BLOCK_NAME;

mod alive_entry;
use compiler::backend::reg_alloc::validate::alive_entry::*;

mod exact_liveness;
use compiler::backend::reg_alloc::validate::exact_liveness::*;

const VERIFY_SPILLING : bool = false;

#[allow(unused_variables)]
#[allow(unreachable_code)]
pub fn validate_regalloc(cf: &CompiledFunction,
                         reg_assigned: LinkedHashMap<MuID, MuID>,
                         spill_scratch_regs: LinkedHashMap<MuID, MuID>)
{
    debug!("---Validating register allocation results---");

    debug!("liveness analysis...");
    let liveness = ExactLiveness::new(cf);
    for i in 0..cf.mc().number_of_insts() {
        cf.mc().trace_inst(i);

        liveness.trace(i);
    }

    let mut alive = AliveEntries::new();

    debug!("initializing alive entries for arguments...");

    // set up initial states

    // machine specific regs, such as program counter, stack pointer, etc
    add_machine_specific_regs_at_func_start(&mut alive);

    // arguments with real locations
    let ref frame = cf.frame;
    for (_, reg) in frame.argument_by_reg.iter() {
        alive.new_alive_reg(alias(reg.id()));
    }

    debug!("---alive entries in the beginning---");
    debug!("{}", alive);

    let mc = cf.mc();

    let mut work_queue : LinkedHashMap<MuName, AliveEntries> = LinkedHashMap::new();
    let mut visited    : LinkedHashMap<MuName, AliveEntries> = LinkedHashMap::new();
    // push entry block
    work_queue.insert(PROLOGUE_BLOCK_NAME.to_string(), alive.clone());

    while !work_queue.is_empty() {
        // fetch next block
        let (block, mut alive) = work_queue.pop_front().unwrap();

        debug!("---working on block {}---", block);
        debug!("{}", alive);

        // check inst sequentially
        let range = match mc.get_block_range(&block) {
            Some(range) => range,
            None => panic!("cannot find range for block {}", block)
        };
        let last_inst = mc.get_last_inst(range.end - 1).unwrap();
        for i in range {
            mc.trace_inst(i);

            // validate spill
            if VERIFY_SPILLING {
                panic!("the code doesnt work");

                if let Some(spill_loc) = mc.is_spill_load(i) {
                    // spill load is a move from spill location (mem) to temp

                    // its define is the scratch temp
                    let scratch_temp = mc.get_inst_reg_defines(i)[0];
                    let source_temp = get_source_temp_for_scratch(scratch_temp, &spill_scratch_regs);

                    // we check if source_temp are alive, and if it is alive in the designated location
                    validate_spill_load(scratch_temp, source_temp, spill_loc, &mut alive);
                } else if let Some(spill_loc) = mc.is_spill_store(i) {
                    // spill store is a move from scratch temp to mem

                    // it uses scratch temp as well as stack pointer (to refer to mem)
                    // we try to find the scratch temp
                    let scratch_temp = {
                        let uses = mc.get_inst_reg_uses(i);
                        let mut use_temps = vec![];
                        for reg in uses {
                            if reg >= MACHINE_ID_END {
                                use_temps.push(reg)
                            }
                        };

                        assert!(use_temps.len() == 1);

                        use_temps[0]
                    };
                    let source_temp = get_source_temp_for_scratch(scratch_temp, &spill_scratch_regs);

                    // we add both scratch_temp, and source_temp as alive
                    add_spill_store(scratch_temp, source_temp, spill_loc, &mut alive);
                }
            }

            // validate uses of registers
            for reg_use in mc.get_inst_reg_uses(i) {
                validate_use(reg_use, &reg_assigned, &alive);
            }

            // remove registers that die at this instruction from alive entries
            if let Some(kills) = liveness.get_kills(i) {
                for reg in kills.iter() {
                    debug!("Temp/Reg{} is killed", reg);
                    kill_reg(*reg, &mut alive);
                }
            }

            for reg_def in mc.get_inst_reg_defines(i) {
                let liveout = liveness.get_liveout(i).unwrap();

                // if reg is in the liveout set, we add a define to it
                if liveout.contains(&reg_def) {
                    // add a define
                    // modify existing alive entries (e.g. kill existing registers)
                    add_def(reg_def, &reg_assigned, mc.is_move(i), &mut alive);
                } else {
                    // we need to kill the reg, so that other temps cannot use it
                    // (its value has been defined)
                    if !mc.is_move(i) {
                        debug!("Temp/Reg{} is not liveout, will be killed", reg_def);
                        kill_reg(reg_def, &mut alive);
                    }
                }
            }

            debug!("{}", alive);
            debug!("---");
        }

        // find liveout of the block, and only preserve what is in the liveout
        let liveout = match mc.get_ir_block_liveout(&block) {
            Some(liveout) => liveout,
            None => panic!("cannot find liveout for block {}", block)
        };
        alive.preserve_list(liveout);
        debug!("liveout is {:?}", liveout);
        debug!("preserve only entries in liveout, we get:");
        debug!("{}", alive);

        // find succeeding blocks
        let succeeding_blocks : Vec<MuName> = mc.get_succs(last_inst).iter()
                                              .map(|x| match mc.is_label(*x - 1) {
                                                  Some(label) => label,
                                                  None => panic!("cannot find label for inst {}", *x - 1)
                                              }).collect();

        // 1) if we have visited this block before, we need to merge (intersect alive entries)
        // alive entries is changed, we need to push successors
        // 2) if this is our first time visit the block, we push successors
        let mut should_push_successors = false;

        if visited.contains_key(&block) {
            // if current block exists in visited, intersect with current
            let mut old = visited.get_mut(&block).unwrap();
            let changed = old.intersect(&alive);

            if changed {
                debug!("we have visted this block before, but intersection made changes. we need to push its sucessors again. ");
                should_push_successors = true;
            }
        } else {
            debug!("first time we visited this block, push its successors");
            visited.insert(block.clone(), alive.clone());
            should_push_successors = true;
        }

        // push successors to work list
        if should_push_successors {
            if succeeding_blocks.len() == 1 {
                // nothing special, just push next block to work list
                work_queue.insert(succeeding_blocks[0].clone(), alive.clone());
                debug!("push block {} to work list", succeeding_blocks[0]);
            } else if succeeding_blocks.len() == 2 {
                // conditional branch

                // it is possible that a variable is alive at the end of a BB, and used
                // only in one of its successors

                // 1st branch
                {
                    let block1 = succeeding_blocks[0].clone();
                    let block1_livein = match mc.get_ir_block_livein(&block1) {
                        Some(livein) => livein,
                        None => panic!("cannot find livein for block {}", block1)
                    };
                    let mut block1_alive = alive.clone();
                    block1_alive.preserve_list(block1_livein);

                    work_queue.insert(block1, block1_alive);
                    debug!("push block {} to work list", succeeding_blocks[0]);
                }

                // 2nd branch
                {
                    let block2 = succeeding_blocks[1].clone();
                    let block2_livein = match mc.get_ir_block_livein(&block2) {
                        Some(livein) => livein,
                        None => panic!("cannot find livein for block {}", block2)
                    };
                    let mut block2_alive = alive.clone();
                    block2_alive.preserve_list(block2_livein);

                    work_queue.insert(block2, block2_alive);
                    debug!("push block {} to work list", succeeding_blocks[1]);
                }
            }
        }

        //
    }
}

fn get_source_temp_for_scratch(scratch: MuID, spill_scratch_temps: &LinkedHashMap<MuID, MuID>) -> MuID {
    match spill_scratch_temps.get(&scratch) {
        Some(src) => get_source_temp_for_scratch(*src, spill_scratch_temps),
        None => scratch
    }
}

fn get_machine_reg(reg: MuID, reg_assigned: &LinkedHashMap<MuID, MuID>) -> MuID {
    // find machine regs
    if reg < MACHINE_ID_END {
        reg
    } else {
        match reg_assigned.get(&reg) {
            Some(reg) => *reg,
            None => panic!("Temp {} is not assigned to any machine register", reg)
        }
    }
}

fn validate_use(reg: MuID, reg_assigned: &LinkedHashMap<MuID, MuID>, alive: &AliveEntries) {
    if reg < MACHINE_ID_END {
        // machine register

        // instruction selector choose to use machine registers
        // it is not about the correctness of register allocation, we do not verify it here
    } else {
        let machine_reg = get_machine_reg(reg, reg_assigned);
        let temp = reg;

        // ensure temp is assigned to the same machine reg in alive entries
        if alive.has_entries_for_temp(temp) {
            for entry in alive.find_entries_for_temp(temp).iter() {
                if !entry.match_reg(machine_reg) {
                    error!("Temp{}/MachineReg{} does not match at this point. ", temp, machine_reg);
                    error!("Temp{} is assigned as {}", temp, entry);

                    panic!("validation failed: temp-reg pair doesnt match")
                }
            }
        } else {
            error!("Temp{} is not alive at this point. ", temp);

            panic!("validation failed: use a temp that is not alive");
        }
    }
}

fn kill_reg(reg: MuID, alive: &mut AliveEntries) {
    if reg < MACHINE_ID_END {
        if alive.has_entries_for_reg(reg) {
            alive.remove_reg(reg);
        }
    } else {
        let temp = reg;

        alive.remove_temp(temp);
    }
}

fn add_def(reg: MuID, reg_assigned: &LinkedHashMap<MuID, MuID>, is_mov: bool, alive: &mut AliveEntries) {
    let machine_reg = get_machine_reg(reg, reg_assigned);
    let temp = reg;

    if reg < MACHINE_ID_END {
        // if it is a machine register
        // we require either it doesn't have an entry,
        // or its entry doesnt have a temp, so that we can safely overwrite it

        if !alive.has_entries_for_reg(reg) {
            // add new machine register
            alive.new_alive_reg(reg);
        } else if !alive.find_entries_for_reg(reg).iter().any(|entry| entry.has_temp()) {
            // overwrite the value that is not used
        } else {
            for entry in alive.find_entries_for_reg(reg).iter() {
                let old_temp = entry.get_temp().unwrap();
                error!("Register{}/Temp{} is alive at this point, defining a new value to Register{} is incorrect", reg, old_temp, reg);
            }

            panic!("validation failed: define a register that is already alive (value overwritten)");
        }
    } else {
        if !alive.has_entries_for_reg(machine_reg) {
            // if this register is not alive, we add an entry for it
            alive.add_temp_in_reg(temp, machine_reg);
        } else {
            // otherwise, this register contains some value
            {
                for entry in alive.find_entries_for_reg_mut(machine_reg) {
                    if !entry.has_temp() {
                        debug!("adding temp {} to reg {}", temp, machine_reg);
                        entry.set_temp(temp);
                    } else {
                        // if the register is holding a temporary, it needs to be coalesced with new temp
                        let old_temp: MuID = entry.get_temp().unwrap();

                        if old_temp == temp {
                            // overwrite value, safe
                        } else {
                            if is_mov {
                                debug!("Temp{} and Temp{} is using the same Register{}, possibly coalesced", temp, old_temp, machine_reg);
                            } else {
                                // trying to overwrite another value, error
                                error!("Temp{} and Temp{} try use the same Register{}", temp, old_temp, machine_reg);

                                panic!("validation failed: define a register that is already alive");
                            }
                        }
                    }
                }
            }

            // they are coalesced, it is valid
            alive.add_temp_in_reg(temp, machine_reg);
        }
    }
}

fn add_spill_store(scratch_temp: MuID, source_temp: MuID, spill_loc: P<Value>,
                   alive: &mut AliveEntries) {
    // add source_temp with mem loc
    alive.add_temp_in_mem(source_temp, spill_loc.clone());

    // add scratch_temp
    alive.add_temp_in_mem(scratch_temp, spill_loc.clone());
}

fn validate_spill_load(scratch_temp: MuID, source_temp: MuID, spill_loc: P<Value>,
                       alive: &mut AliveEntries) {
    // verify its correct: the source temp should be alive with the mem location
    if alive.has_entries_for_temp(source_temp) {
        for entry in alive.find_entries_for_temp(source_temp).iter() {
            if entry.match_stack_loc(spill_loc.clone()) {
                // valid
            } else {
                error!("SourceTemp{} is alive with the following entry, loading it from {} as ScratchTemp{} is not valid", source_temp, spill_loc, scratch_temp);
                debug!("{}", entry);

                panic!("validation failed: load a register from a spilled location that is incorrect");
            }
        }
    } else {
        error!("SourceTemp{} is not alive, loading it from {} as ScratchTemp{} is not valid", scratch_temp, spill_loc, scratch_temp);

        panic!("validation failed: load a register from a spilled location before storing into it")
    }
}

#[cfg(target_arch = "x86_64")]
fn add_machine_specific_regs_at_func_start(alive: &mut AliveEntries) {
    use compiler::backend::x86_64;

    // RIP, RSP, RBP always have valid values
    alive.new_alive_reg(x86_64::RIP.id());
    alive.new_alive_reg(x86_64::RSP.id());
    alive.new_alive_reg(x86_64::RBP.id());

    // callee saved regs are alive
    alive.new_alive_reg(x86_64::RBX.id());
    alive.new_alive_reg(x86_64::R12.id());
    alive.new_alive_reg(x86_64::R13.id());
    alive.new_alive_reg(x86_64::R14.id());
    alive.new_alive_reg(x86_64::R15.id());
}

#[cfg(target_arch = "aarch64")]
fn add_machine_specific_regs_at_func_start(alive: &mut AliveEntries) {
    use compiler::backend::aarch64;

    // the instruction pointer, stack pointer, link register and frame pointer, always have valid values
    alive.new_alive_reg(aarch64::SP.id());
    alive.new_alive_reg(aarch64::LR.id());
    alive.new_alive_reg(aarch64::FP.id());

    // callee saved regs are alive
    alive.new_alive_reg(aarch64::X28.id());
    alive.new_alive_reg(aarch64::X27.id());
    alive.new_alive_reg(aarch64::X26.id());
    alive.new_alive_reg(aarch64::X25.id());
    alive.new_alive_reg(aarch64::X24.id());
    alive.new_alive_reg(aarch64::X23.id());
    alive.new_alive_reg(aarch64::X22.id());
    alive.new_alive_reg(aarch64::X21.id());
    alive.new_alive_reg(aarch64::X20.id());
    alive.new_alive_reg(aarch64::X19.id());

    // platform register, reserved (never use it)
    alive.new_alive_reg(aarch64::PR.id());
}