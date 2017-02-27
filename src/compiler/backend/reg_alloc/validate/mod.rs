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
                         reg_spilled: LinkedHashMap<MuID, P<Value>>)
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
    for (_, stack) in frame.argument_by_stack.iter() {
        alive.new_alive_mem(stack.clone());
    }

    debug!("alive entries in the beginning");
    debug!("{}", alive);

    let mc = cf.mc();

    for i in 0..mc.number_of_insts() {
        mc.trace_inst(i);

        if mc.is_jmp(i).is_some() {
            // we need to flow-sensitive analysis
            unimplemented!();
        }

        for reg_use in mc.get_inst_reg_uses(i) {
            validate_use(reg_use, &reg_assigned, &alive);
        }

        // remove kills in the inst from alive entries
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
        if let Some(entry) = alive.find_entry_for_temp(temp) {
            if !entry.match_reg(machine_reg) {
                error!("Temp{}/MachineReg{} does not match at this point. ", temp, machine_reg);
                error!("Temp{} is assigned as {}", temp, entry);

                panic!("validation failed: temp-reg pair doesnt match")
            }
        } else {
            error!("Temp{} is not alive at this point. ", temp);

            panic!("validation failed: use a temp that is not alive");
        }
    }
}

fn kill_reg(reg: MuID, reg_assigned: &LinkedHashMap<MuID, MuID>, alive: &mut AliveEntries) {
    if reg < MACHINE_ID_END {
        if alive.find_entry_for_reg(reg).is_some() {
            alive.remove_reg(reg);
        }
    } else {
        let temp = reg;

        alive.remove_temp(temp);
    }
}

fn add_def(reg: MuID, reg_assigned: &LinkedHashMap<MuID, MuID>, reg_coalesced: &LinkedHashMap<MuID, MuID>, alive: &mut AliveEntries) {
    if reg < MACHINE_ID_END {
        // if it is a machine register
        // we require either it doesn't have an entry,
        // or its entry doesnt have a temp, so that we can safely overwrite it

        if alive.find_entry_for_reg(reg).is_none() {
            // add new machine register
            alive.new_alive_reg(reg);
        } else if !alive.find_entry_for_reg(reg).unwrap().has_temp() {
            // overwrite it
        } else {
            let old_temp = alive.find_entry_for_reg(reg).unwrap().get_temp().unwrap();

            error!("Register{}/Temp{} is alive at this point, defining a new value to Register{} is incorrect", reg, old_temp, reg);

            panic!("validation failed: define a register that is already alive (value overwritten)");
        }
    } else {
        let machine_reg = get_machine_reg(reg, reg_assigned);
        let temp = reg;

        if alive.find_entry_for_reg(machine_reg).is_none() {
            // if this register is not alive, we add an entry for it
            alive.add_temp_in_reg(temp, machine_reg);
        } else {
            // otherwise, this register contains some value
            {
                let entry = alive.find_entry_for_reg_mut(machine_reg).unwrap();

                if !entry.has_temp() {
                    debug!("adding temp {} to reg {}", temp, machine_reg);
                    entry.set_temp(temp);
                } else {
                    // if the register is holding a temporary, it needs to be coalesced with new temp
                    let old_temp: MuID = entry.get_temp().unwrap();

                    if (reg_coalesced.contains_key(&old_temp) && *reg_coalesced.get(&old_temp).unwrap() == temp)
                        || (reg_coalesced.contains_key(&temp) && *reg_coalesced.get(&temp).unwrap() == old_temp)
                    {
                        // coalesced, safe
                    } else {
                        // not coalesced, error
                        error!("Temp{} and Temp{} are not coalesced, but they use the same Register{}", temp, old_temp, machine_reg);

                        panic!("validation failed: define a register that is already alive, and their temps are not coalesced");
                    }
                }
            }

            // they are coalesced, it is valid
            alive.add_temp_in_reg(temp, machine_reg);
        }
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