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

#![allow(dead_code)]

use utils::LinkedHashMap;
use utils::vec_utils;
use ast::ir::*;
use ast::ptr::*;
use std::fmt;

type EntryID = usize;

#[derive(Clone)]
pub struct AliveEntries {
    index: EntryID,

    inner: LinkedHashMap<EntryID, RegisterEntry>
}

impl fmt::Display for AliveEntries {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{} entries", self.inner.len()).unwrap();
        writeln!(
            f,
            "| {:20} | {:20} | {:20} |",
            "ssa",
            "registers",
            "stack slots"
        ).unwrap();
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
            temp: None,
            real: vec![reg],
            stack: vec![]
        };

        self.inner.insert(id, entry);
    }

    pub fn new_alive_mem(&mut self, mem: P<Value>) {
        debug!("adding alive mem: {}", mem);

        let id = self.new_index();
        let entry = RegisterEntry {
            temp: None,
            real: vec![],
            stack: vec![mem]
        };

        self.inner.insert(id, entry);
    }

    pub fn add_temp_in_reg(&mut self, temp: MuID, reg: MuID) {
        debug!("adding alive temp in reg: {} in {}", temp, reg);

        let entry_exists = self.has_entries_for_temp(temp);

        if entry_exists {
            let entries = self.find_entries_for_temp_mut(temp);
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
            let entries = self.find_entries_for_temp_mut(temp);
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

    pub fn intersect(&mut self, another: &Self) -> bool {
        let mut changed = false;

        for (_, entry) in self.inner.iter_mut() {
            if entry.has_temp() {
                let temp = entry.get_temp().unwrap();

                // find entry with the same temp in the other set, and do intersect
                for another_entry in another.find_entries_for_temp(temp) {
                    if entry.intersect(another_entry) {
                        changed = true;
                    }
                }
            } else {
                // find entry without a temp in the other set and do intersect
                for another_entry in another.inner.values() {
                    if !another_entry.has_temp() {
                        if entry.intersect(another_entry) {
                            changed = true;
                        }
                    }
                }
            }
        }

        changed
    }

    pub fn preserve_list(&mut self, list: &Vec<MuID>) {
        let mut indices_to_delete: Vec<EntryID> = vec![];

        for (index, entry) in self.inner.iter() {
            if !entry.has_temp() {
                if list.iter().any(|x| entry.match_reg(*x)) {

                } else {
                    indices_to_delete.push(*index);
                }
            } else {
                let temp = entry.get_temp().unwrap();
                if !vec_utils::find_value(list, temp).is_some() {
                    indices_to_delete.push(*index);
                }
            }
        }

        // always preserve entries with spill location
        let mut indices_to_really_delete: Vec<EntryID> = vec![];
        for index in indices_to_delete {
            let entry = self.inner.get(&index).unwrap();

            if !entry.has_stack_slots() {
                indices_to_really_delete.push(index);
            }
        }

        // delete
        for index in indices_to_really_delete {
            self.inner.remove(&index);
        }
    }
}

#[derive(Clone)]
pub struct RegisterEntry {
    temp: Option<MuID>,
    real: Vec<MuID>,
    stack: Vec<P<Value>>
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

    // two entries can intersect only when they have the same temp, or they do not have temps
    pub fn intersect(&mut self, another: &Self) -> bool {
        assert!(
            (!self.has_temp() && !another.has_temp() ||
                 (self.has_temp() && another.has_temp() &&
                      self.get_temp().unwrap() == another.get_temp().unwrap()))
        );

        let mut changed = false;

        // intersect real registers
        if vec_utils::intersect(&mut self.real, &another.real) {
            changed = true;
        }

        // intersect memory
        if vec_utils::intersect(&mut self.stack, &another.stack) {
            changed = true;
        }

        changed
    }
}

impl fmt::Display for RegisterEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let temp = match self.temp {
            Some(id) => format!("{}", id),
            None => "_".to_string()
        };

        let real = format!("{:?}", self.real);
        let stack = format!("{:?}", self.stack);

        write!(f, "| {:20} | {:20} | {:20} |", temp, real, stack)
    }
}
