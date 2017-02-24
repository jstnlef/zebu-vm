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

    pub fn find_entry_for_temp(&self, temp: MuID) -> Option<&RegisterEntry> {
        for entry in self.inner.values() {
            if entry.match_temp(temp) {
                return Some(entry);
            }
        }

        None
    }

    pub fn find_entry_for_temp_mut(&mut self, temp: MuID) -> Option<&mut RegisterEntry> {
        for entry in self.inner.values_mut() {
            if entry.match_temp(temp) {
                return Some(entry)
            }
        }

        None
    }

    pub fn find_entry_for_reg(&self, reg: MuID) -> Option<&RegisterEntry> {
        for entry in self.inner.values() {
            if entry.match_reg(reg) {
                return Some(entry);
            }
        }

        None
    }

    pub fn find_entry_for_reg_mut(&mut self, reg: MuID) -> Option<&mut RegisterEntry> {
        for entry in self.inner.values_mut() {
            if entry.match_reg(reg) {
                return Some(entry);
            }
        }

        None
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

        let entry_exists = self.find_entry_for_temp(temp).is_some();

        if entry_exists {
            let mut entry = self.find_entry_for_temp_mut(temp).unwrap();
            entry.add_real_reg(reg);
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

    pub fn remove_reg(&mut self, reg: MuID) {
        debug!("removing alive reg: {}", reg);
        let mut indices = vec![];

        for (index, entry) in self.inner.iter() {
            if entry.match_reg(reg) {
                indices.push(*index);
            }
        }

        for index in indices {
            self.inner.remove(&index);
        }
    }

    pub fn remove_temp(&mut self, reg: MuID) {
        debug!("removing alive temp: {}", reg);

        let index = {
            let mut ret = 0;

            for (index, entry) in self.inner.iter() {
                if entry.match_temp(reg) {
                    ret = *index;
                }
            }

            ret
        };

        self.inner.remove(&index);
    }
}

pub struct RegisterEntry {
    temp  : Option<MuID>,
    real  : Vec<MuID>,
    stack : Vec<P<Value>>
}

impl RegisterEntry {
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

    pub fn add_real_reg(&mut self, reg: MuID) {
        if vec_utils::find_value(&mut self.real, reg).is_none() {
            self.real.push(reg);
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