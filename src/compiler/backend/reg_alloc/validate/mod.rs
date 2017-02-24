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
                         reg_spilled: LinkedHashMap<MuID, P<Value>>)
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
    for (_, stack) in frame.argument_by_stack.iter() {
        alive.new_alive_mem(stack.clone());
    }

    debug!("alive entries in the beginning");
    debug!("{}", alive);

    let mc = cf.mc();

    for i in 0..mc.number_of_insts() {
        mc.trace_inst(i);

        if mc.is_jump(i) {
            // we need to flow-sensitive analysis
            unimplemented!();
        }

        for reg_use in mc.get_inst_reg_uses(i) {
            validate_use(reg_use, &reg_assigned, &alive);
        }

        // remove kills in the inst from alive entries
        if let Some(kills) = liveness.get_kills(i) {
            for reg in kills.iter() {
                remove_kill(*reg, &reg_assigned, &mut alive);
            }
        }

        // add defines to alive entries
        for reg_def in mc.get_inst_reg_defines(i) {
            add_def(reg_def, &reg_assigned, &mut alive);
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

fn remove_kill(reg: MuID, reg_assigned: &LinkedHashMap<MuID, MuID>, alive: &mut AliveEntries) {
    if reg < MACHINE_ID_END {
        if alive.find_entry_for_reg(reg).is_some() {
            alive.remove_reg(reg);
        }
    } else {
        let temp = reg;

        alive.remove_temp(temp);
    }
}

fn add_def(reg: MuID, reg_assigned: &LinkedHashMap<MuID, MuID>, alive: &mut AliveEntries) {
    if reg < MACHINE_ID_END {
        if alive.find_entry_for_reg(reg).is_none() {
            // add new machine register
            alive.new_alive_reg(reg);
        }
    } else {
        let machine_reg = get_machine_reg(reg, reg_assigned);
        let temp = reg;

        alive.add_temp_in_reg(temp, machine_reg);
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