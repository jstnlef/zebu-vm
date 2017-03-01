use utils::LinkedHashMap;
use ast::ir::*;
use ast::ptr::*;
use compiler::machine_code::CompiledFunction;
use compiler::backend::get_color_for_precolored as alias;

mod alive_entry;
use compiler::backend::reg_alloc::validate::alive_entry::*;

mod exact_liveness;
use compiler::backend::reg_alloc::validate::exact_liveness::*;

pub fn validate_regalloc(cf: &CompiledFunction,
                         func: &MuFunctionVersion,
                         reg_assigned: LinkedHashMap<MuID, MuID>,
                         reg_coalesced:LinkedHashMap<MuID, MuID>,
                         reg_spilled: LinkedHashMap<MuID, P<Value>>,
                         spill_scratch_regs: LinkedHashMap<MuID, MuID>)
{
    debug!("---Validating register allocation results---");

    debug!("coalesced registers: ");
    for (a, b) in reg_coalesced.iter() {
        debug!("{} -> {}", a, b);
    }

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
    // we do not consider mem loc for arguments - we do consider mem loc for spilling
//    for (_, stack) in frame.argument_by_stack.iter() {
//        alive.new_alive_mem(stack.clone());
//    }

    debug!("alive entries in the beginning");
    debug!("{}", alive);

    let mc = cf.mc();

    for i in 0..mc.number_of_insts() {
        mc.trace_inst(i);

        if mc.is_jmp(i).is_some() {
            // we need to do flow-sensitive analysis
            unimplemented!();
        }

        // validate spill
        if let Some(spill_loc) = mc.is_spill_load(i) {
            // spill load is a move from spill location (mem) to temp
            // its define is the scratch temp
            let scratch_temp = mc.get_inst_reg_defines(i)[0];
            let source_temp  = get_source_temp_for_scratch(scratch_temp, &spill_scratch_regs);

            add_spill_load(scratch_temp, source_temp, spill_loc, &reg_spilled, &mut alive);
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

            add_spill_store(scratch_temp, source_temp, spill_loc, &reg_spilled, &reg_coalesced, &mut alive);
        }

        // validate uses of registers
        for reg_use in mc.get_inst_reg_uses(i) {
            validate_use(reg_use, &reg_assigned, &alive);
        }

        // remove registers that die at this instruction from alive entries
        if let Some(kills) = liveness.get_kills(i) {
            for reg in kills.iter() {
                kill_reg(*reg, &reg_assigned, &mut alive);
            }
        }

        // add defines to alive entries
        for reg_def in mc.get_inst_reg_defines(i) {
            let liveout = liveness.get_liveout(i).unwrap();

            // if reg is in the liveout set, we add a define to it
            if liveout.contains(&reg_def) {
                add_def(reg_def, &reg_assigned, &reg_coalesced, &mut alive);
            } else {
                kill_reg(reg_def, &reg_assigned, &mut alive);
            }
        }

        debug!("{}", alive);
        trace!("---");
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
            alive.find_entries_for_temp(temp).iter().inspect(|entry| {
                if !entry.match_reg(machine_reg) {
                    error!("Temp{}/MachineReg{} does not match at this point. ", temp, machine_reg);
                    error!("Temp{} is assigned as {}", temp, entry);

                    panic!("validation failed: temp-reg pair doesnt match")
                }
            });
        } else {
            error!("Temp{} is not alive at this point. ", temp);

            panic!("validation failed: use a temp that is not alive");
        }
    }
}

fn kill_reg(reg: MuID, reg_assigned: &LinkedHashMap<MuID, MuID>, alive: &mut AliveEntries) {
    if reg < MACHINE_ID_END {
        if alive.has_entries_for_reg(reg) {
            alive.remove_reg(reg);
        }
    } else {
        let temp = reg;

        alive.remove_temp(temp);
    }
}

fn is_coalesced(reg1: MuID, reg2: MuID, reg_coalesced: &LinkedHashMap<MuID, MuID>) -> bool {
    if reg1 == reg2 {
        true
    } else if (reg_coalesced.contains_key(&reg1) && *reg_coalesced.get(&reg2).unwrap() == reg1)
        || (reg_coalesced.contains_key(&reg2) && *reg_coalesced.get(&reg1).unwrap() == reg2) {
        true
    } else {
        false
    }
}

fn add_def(reg: MuID, reg_assigned: &LinkedHashMap<MuID, MuID>, reg_coalesced: &LinkedHashMap<MuID, MuID>, alive: &mut AliveEntries) {
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
            alive.find_entries_for_reg(reg).iter().inspect(|entry| {
                let old_temp = entry.get_temp().unwrap();
                error!("Register{}/Temp{} is alive at this point, defining a new value to Register{} is incorrect", reg, old_temp, reg);
            });

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

                        if is_coalesced(old_temp, temp, reg_coalesced) {
                            // safe
                        } else {
                            // not coalesced, error
                            error!("Temp{} and Temp{} are not coalesced, but they use the same Register{}", temp, old_temp, machine_reg);

                            panic!("validation failed: define a register that is already alive, and their temps are not coalesced");
                        }
                    }
                }
            }

            // they are coalesced, it is valid
            alive.add_temp_in_reg(temp, machine_reg);
        }
    }

    // if other temp use the same register, remove the register from their entry
    for entry in alive.find_entries_for_reg_mut(machine_reg) {
        if let Some(other_temp) = entry.get_temp() {
            if is_coalesced(other_temp, temp, reg_coalesced) {
                // do nothing
            } else {
                entry.remove_real(reg);
            }
        }
    }
}

fn add_spill_store(scratch_temp: MuID, source_temp: MuID, spill_loc: P<Value>,
                   reg_spilled: &LinkedHashMap<MuID, P<Value>>,
                   reg_coalesced: &LinkedHashMap<MuID, MuID>,
                   alive: &mut AliveEntries) {
    // add source_temp with mem loc
    alive.add_temp_in_mem(source_temp, spill_loc.clone());

    // add scratch_temp
    alive.add_temp_in_mem(scratch_temp, spill_loc.clone());

    // trying to store into a spill location, it is always valid
    // but if other temp use the same mem location and it is not coalesced temp,
    // we need to delete the mem location from their entry
    for entry in alive.find_entries_for_mem_mut(spill_loc.clone()) {
        if let Some(temp) = entry.get_temp() {
            if is_coalesced(temp, source_temp, reg_coalesced) || is_coalesced(temp, scratch_temp, reg_coalesced) {
                // its okay, coalesced temps can have one spill location
            } else {
                entry.remove_stack_loc(spill_loc.clone())
            }
        }
    }
}

fn add_spill_load(scratch_temp: MuID, source_temp: MuID, spill_loc: P<Value>,
                  reg_spilled: &LinkedHashMap<MuID, P<Value>>,
                  alive: &mut AliveEntries) {
    // verify its correct: the source temp should be alive with the mem location
    if alive.has_entries_for_temp(source_temp) {
        alive.find_entries_for_temp(source_temp).iter().inspect(|entry| {
            if entry.match_stack_loc(spill_loc.clone()) {
                // valid
            } else {
                error!("SourceTemp{} is alive with the following entry, loading it from {} as ScratchTemp{} is not valid", source_temp, spill_loc, scratch_temp);
                debug!("{}", entry);

                panic!("validation failed: load a register from a spilled location that is incorrect");
            }
        });
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