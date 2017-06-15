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

use ir::*;
use ptr::*;
use types::*;

use utils::LinkedHashMap;

pub struct MuBundle {
    pub id: MuID,
    
    pub type_defs: LinkedHashMap<MuID, P<MuType>>,
    pub func_sigs: LinkedHashMap<MuID, P<MuFuncSig>>,
    pub constants: LinkedHashMap<MuID, P<Value>>,
    pub globals  : LinkedHashMap<MuID, P<Value>>,
    pub func_defs: LinkedHashMap<MuID, MuFunction>,
    pub func_decls: LinkedHashMap<MuID, MuFunctionVersion>,
    
//    id_name_map: LinkedHashMap<MuID, MuName>,
//    name_id_map: LinkedHashMap<MuName, MuID>
}

impl MuBundle {
    pub fn new(id: MuID) -> MuBundle {
        MuBundle {
            id: id,
            
            type_defs: LinkedHashMap::new(),
            func_sigs: LinkedHashMap::new(),
            constants: LinkedHashMap::new(),
            globals: LinkedHashMap::new(),
            func_defs: LinkedHashMap::new(),
            func_decls: LinkedHashMap::new(),
            
//            id_name_map: LinkedHashMap::new(),
//            name_id_map: LinkedHashMap::new()
        }
    }
}