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

use utils::LinkedHashMap;
use utils::LinkedHashSet;
use ast::ir::*;
use compiler::machine_code::CompiledFunction;

pub struct ExactLiveness {
    livein: LinkedHashMap<usize, LinkedHashSet<MuID>>,
    liveout: LinkedHashMap<usize, LinkedHashSet<MuID>>,
    kill: LinkedHashMap<usize, LinkedHashSet<MuID>>,
}

impl ExactLiveness {
    pub fn new(cf: &CompiledFunction) -> ExactLiveness {
        let mut ret = ExactLiveness {
            livein: LinkedHashMap::new(),
            liveout: LinkedHashMap::new(),
            kill: LinkedHashMap::new(),
        };

        ret.liveness_analysis(cf);

        ret
    }

    fn liveness_analysis(&mut self, cf: &CompiledFunction) {
        let mc = cf.mc();

        for block in mc.get_all_blocks().iter() {
            let range = mc.get_block_range(block).unwrap();

            let mut liveout: LinkedHashSet<MuID> =
                LinkedHashSet::from_vec(mc.get_ir_block_liveout(block).unwrap().clone());

            for i in range.rev() {
                // set liveout
                self.liveout.insert(i, liveout.clone());

                // compute livein: in[n] <- use[n] + (out[n] - def[n])
                for reg_def in mc.get_inst_reg_defines(i) {
                    liveout.remove(&reg_def);
                }
                for reg_use in mc.get_inst_reg_uses(i) {
                    liveout.insert(reg_use);
                }
                // liveout is livein now
                self.livein.insert(i, liveout.clone());

                // liveout for prev inst is livein for current inst
            }
        }

        // liveness analysis done
        // compute 'kill': if a reg is in livein of an inst, but not liveout, it kills in the inst
        for i in self.livein.keys() {
            let mut kill: LinkedHashSet<MuID> = LinkedHashSet::new();

            let livein = self.livein.get(i).unwrap();
            let liveout = self.liveout.get(i).unwrap();

            for reg in livein.iter() {
                if !liveout.contains(reg) {
                    kill.insert(*reg);
                }
            }

            self.kill.insert(*i, kill);
        }
    }

    pub fn get_liveout(&self, index: usize) -> Option<LinkedHashSet<MuID>> {
        match self.liveout.get(&index) {
            Some(s) => Some(s.clone()),
            None => None,
        }
    }

    pub fn get_kills(&self, index: usize) -> Option<LinkedHashSet<MuID>> {
        match self.kill.get(&index) {
            Some(s) => Some(s.clone()),
            None => None,
        }
    }

    pub fn trace(&self, index: usize) {
        if self.livein.contains_key(&index) {
            trace!("in  : {:?}", self.livein.get(&index).unwrap());
            trace!("out : {:?}", self.liveout.get(&index).unwrap());
            trace!("kill: {:?}", self.kill.get(&index).unwrap());
        }
    }
}
