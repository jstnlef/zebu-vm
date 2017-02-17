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