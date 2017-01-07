use ir::*;
use ptr::*;
use types::*;

use std::collections::HashMap;

pub struct MuBundle {
    pub id: MuID,
    
    pub type_defs: HashMap<MuID, P<MuType>>,
    pub func_sigs: HashMap<MuID, P<MuFuncSig>>,
    pub constants: HashMap<MuID, P<Value>>,
    pub globals  : HashMap<MuID, P<Value>>,
    pub func_defs: HashMap<MuID, MuFunction>,
    pub func_decls: HashMap<MuID, MuFunctionVersion>,
    
//    id_name_map: HashMap<MuID, MuName>,
//    name_id_map: HashMap<MuName, MuID>
}

impl MuBundle {
    pub fn new(id: MuID) -> MuBundle {
        MuBundle {
            id: id,
            
            type_defs: HashMap::new(),
            func_sigs: HashMap::new(),
            constants: HashMap::new(),
            globals: HashMap::new(),
            func_defs: HashMap::new(),
            func_decls: HashMap::new(),
            
//            id_name_map: HashMap::new(),
//            name_id_map: HashMap::new()
        }
    }
}