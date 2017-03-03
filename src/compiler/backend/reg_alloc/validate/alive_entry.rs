#![allow(dead_code)]

use utils::LinkedHashMap;
use utils::vec_utils;
use ast::ir::*;
use ast::ptr::*;
use std::fmt;

type EntryID = usize;

pub struct AliveEntries {
    index: EntryID,

    inner: LinkedHashMap<EntryID, RegisterEntry>
}

impl fmt::Display for AliveEntries {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{} entries", self.inner.len()).unwrap();
        writeln!(f, "| {:20} | {:20} | {:20} |", "ssa", "registers", "stack slots").unwrap();
        for entry in self.inner.values() {
            writeln!(f, "{}", entry).unwrap()
        }

        Ok(())
    }
}

impl AliveEntries {
    pub fn new() -> AliveEntries {
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

    pub fn has_entries_for_temp(&self, temp: MuID) -> bool {
        for entry in self.inner.values() {
            if entry.match_temp(temp) {
                return true;
            }
        }

        false
    }
    pub fn find_entries_for_temp(&self, temp: MuID) -> Vec<&RegisterEntry> {
        let mut ret = vec![];
        for entry in self.inner.values() {
            if entry.match_temp(temp) {
                ret.push(entry);
            }
        }
        ret
    }
    pub fn find_entries_for_temp_mut(&mut self, temp: MuID) -> Vec<&mut RegisterEntry> {
        let mut ret = vec![];
        for entry in self.inner.values_mut() {
            if entry.match_temp(temp) {
                ret.push(entry);
            }
        }
        ret
    }

    pub fn has_entries_for_reg(&self, reg: MuID) -> bool {
        for entry in self.inner.values() {
            if entry.match_reg(reg) {
                return true;
            }
        }
        false
    }
    pub fn find_entries_for_reg(&self, reg: MuID) -> Vec<&RegisterEntry> {
        let mut ret = vec![];
        for entry in self.inner.values() {
            if entry.match_reg(reg) {
                ret.push(entry);
            }
        }
        ret
    }
    pub fn find_entries_for_reg_mut(&mut self, reg: MuID) -> Vec<&mut RegisterEntry> {
        let mut ret = vec![];
        for entry in self.inner.values_mut() {
            if entry.match_reg(reg) {
                ret.push(entry)
            }
        }
        ret
    }

    pub fn has_entries_for_mem(&self, mem: P<Value>) -> bool {
        for entry in self.inner.values() {
            if entry.match_stack_loc(mem.clone()) {
                return true;
            }
        }
        false
    }
    pub fn find_entries_for_mem(&self, mem: P<Value>) -> Vec<&RegisterEntry> {
        let mut ret = vec![];
        for entry in self.inner.values() {
            if entry.match_stack_loc(mem.clone()) {
                ret.push(entry)
            }
        }
        ret
    }
    pub fn find_entries_for_mem_mut(&mut self, mem: P<Value>) -> Vec<&mut RegisterEntry> {
        let mut ret = vec![];
        for entry in self.inner.values_mut() {
            if entry.match_stack_loc(mem.clone()) {
                ret.push(entry)
            }
        }
        ret
    }


    pub fn new_alive_reg(&mut self, reg: MuID) {
        debug!("adding alive reg: {}", reg);

        let id = self.new_index();
        let entry = RegisterEntry {
            temp : None,
            real : vec![reg],
            stack: vec![]
        };

        self.inner.insert(id, entry);
    }

    pub fn new_alive_mem(&mut self, mem: P<Value>) {
        debug!("adding alive mem: {}", mem);

        let id = self.new_index();
        let entry = RegisterEntry {
            temp : None,
            real : vec![],
            stack: vec![mem]
        };

        self.inner.insert(id, entry);
    }

    pub fn add_temp_in_reg(&mut self, temp: MuID, reg: MuID) {
        debug!("adding alive temp in reg: {} in {}", temp, reg);

        let entry_exists = self.has_entries_for_temp(temp);

        if entry_exists {
            let mut entries = self.find_entries_for_temp_mut(temp);
            for entry in entries {
                entry.add_real_reg(reg);
            }
        } else {
            let id = self.new_index();
            let entry = RegisterEntry {
                temp: Some(temp),
                real: vec![reg],
                stack: vec![]
            };

            self.inner.insert(id, entry);
        }
    }

    pub fn add_temp_in_mem(&mut self, temp: MuID, mem: P<Value>) {
        debug!("adding alive temp in mem: {} in {}", temp, mem);

        let entry_exists = self.has_entries_for_temp(temp);

        if entry_exists {
            let mut entries = self.find_entries_for_temp_mut(temp);
            for entry in entries {
                entry.add_stack_loc(mem.clone());
            }
        } else {
            let id = self.new_index();
            let entry = RegisterEntry {
                temp: Some(temp),
                real: vec![],
                stack: vec![mem]
            };

            self.inner.insert(id, entry);
        }
    }

    pub fn remove_reg(&mut self, reg: MuID) {
        debug!("removing alive reg: {}", reg);
        let mut indices = vec![];

        for (index, entry) in self.inner.iter_mut() {
            if entry.match_reg(reg) {
                entry.remove_real(reg);

                if entry.is_empty() {
                    indices.push(*index);
                }
            }
        }

        for index in indices {
            self.inner.remove(&index);
        }
    }

    pub fn remove_temp(&mut self, temp: MuID) {
        debug!("removing alive temp: {}", temp);

        let mut ret = vec![];

        for (index, entry) in self.inner.iter() {
            if entry.match_temp(temp) {
                ret.push(*index);
            }
        }

        if ret.len() == 0 {
            return;
        } else if ret.len() == 1 {
            self.inner.remove(&ret[0]);
        } else {
            panic!("Temp{} has more than one entry in AliveEntries");
        }
    }
}

pub struct RegisterEntry {
    temp  : Option<MuID>,
    real  : Vec<MuID>,
    stack : Vec<P<Value>>
}

impl RegisterEntry {
    pub fn is_empty(&self) -> bool {
        !self.has_real() && !self.has_stack_slots()
    }

    pub fn has_temp(&self) -> bool {
        self.temp.is_some()
    }
    pub fn has_real(&self) -> bool {
        !self.real.is_empty()
    }
    pub fn has_stack_slots(&self) -> bool {
        !self.stack.is_empty()
    }

    pub fn set_temp(&mut self, temp: MuID) {
        self.temp = Some(temp);
    }
    pub fn get_temp(&self) -> Option<MuID> {
        self.temp.clone()
    }

    pub fn remove_real(&mut self, reg: MuID) {
        if let Some(index) = vec_utils::find_value(&self.real, reg) {
            self.real.remove(index);
        }
    }

    pub fn remove_stack_loc(&mut self, mem: P<Value>) {
        if let Some(index) = vec_utils::find_value(&self.stack, mem) {
            self.stack.remove(index);
        }
    }

    pub fn match_temp(&self, temp: MuID) -> bool {
        if self.temp.is_some() && self.temp.unwrap() == temp {
            true
        } else {
            false
        }
    }

    pub fn match_reg(&self, reg: MuID) -> bool {
        vec_utils::find_value(&self.real, reg).is_some()
    }

    pub fn match_stack_loc(&self, mem: P<Value>) -> bool {
        vec_utils::find_value(&self.stack, mem).is_some()
    }

    pub fn add_real_reg(&mut self, reg: MuID) {
        if vec_utils::find_value(&mut self.real, reg).is_none() {
            self.real.push(reg);
        }
    }

    pub fn add_stack_loc(&mut self, mem: P<Value>) {
        if vec_utils::find_value(&mut self.stack, mem.clone()).is_none() {
            self.stack.push(mem)
        }
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