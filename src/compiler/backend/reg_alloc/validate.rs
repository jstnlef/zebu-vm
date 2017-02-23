use utils::LinkedHashMap;
use utils::vec_utils;
use ast::ir::*;
use ast::ptr::*;
use compiler::machine_code::CompiledFunction;

use std::fmt;

pub fn validate_regalloc(cf: &CompiledFunction,
                         func: &MuFunctionVersion,
                         reg_assigned: LinkedHashMap<MuID, MuID>,
                         reg_spilled: LinkedHashMap<MuID, P<Value>>)
{
    debug!("---Validating register allocation results---");

    let mut alive = AliveEntries::new();

    debug!("initializing alive entries for arguments...");

    // start with arguments with real locations
    let ref frame = cf.frame;
    for (temp, reg) in frame.argument_by_reg.iter() {
        alive.new_alive_reg(reg.id());
    }
    for (temp, stack) in frame.argument_by_stack.iter() {
        alive.new_alive_mem(stack.clone());
    }

    debug!("alive entries in the beginning");
    debug!("{}", alive);
}

type EntryID = usize;

struct AliveEntries {
    index: EntryID,

    inner: LinkedHashMap<EntryID, RegisterEntry>
}

impl fmt::Display for AliveEntries {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "").unwrap();
        writeln!(f, "| {:20} | {:20} | {:20} |", "ssa", "registers", "stack slots").unwrap();
        for entry in self.inner.values() {
            writeln!(f, "{}", entry).unwrap()
        }

        Ok(())
    }
}

impl AliveEntries {
    fn new() -> AliveEntries {
        AliveEntries {
            index: 0,
            inner: LinkedHashMap::new()
        }
    }

    fn new_index(&mut self) -> EntryID {
        let ret = self.index;
        self.index += 1;

        ret
    }

    fn find_entry_for_reg(&self, reg: MuID) -> Option<&RegisterEntry> {
        for entry in self.inner.values() {
            if entry.match_reg(reg) {
                return Some(entry);
            }
        }

        None
    }

    fn find_entry_for_reg_mut(&mut self, reg: MuID) -> Option<&mut RegisterEntry> {
        for entry in self.inner.values_mut() {
            if entry.match_reg(reg) {
                return Some(entry);
            }
        }

        None
    }

    fn new_alive_reg(&mut self, reg: MuID) {
        debug!("adding alive reg: {}", reg);

        let id = self.new_index();
        let entry = RegisterEntry {
            temp : None,
            real : vec![reg],
            stack: vec![]
        };

        self.inner.insert(id, entry);
    }

    fn new_alive_mem(&mut self, mem: P<Value>) {
        debug!("adding alive mem: {}", mem);

        let id = self.new_index();
        let entry = RegisterEntry {
            temp : None,
            real : vec![],
            stack: vec![mem]
        };

        self.inner.insert(id, entry);
    }
}

struct RegisterEntry {
    temp  : Option<MuID>,
    real  : Vec<MuID>,
    stack : Vec<P<Value>>
}

impl RegisterEntry {
    fn match_reg(&self, reg: MuID) -> bool {
        vec_utils::find_value(&self.real, reg).is_some()
    }
}

impl fmt::Display for RegisterEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let temp = match self.temp {
            Some(id) => format!("{}", id),
            None     => "_".to_string()
        };

        let real = format!("{:?}", self.real);
        let stack = format!("{:?}", self.stack);

        write!(f, "| {:20} | {:20} | {:20} |", temp, real, stack)
    }
}