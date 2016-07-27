use ast::ir::*;
use ast::ptr::*;
use ast::types::*;

use std::collections::HashMap;

pub struct MuBundle {
    pub type_defs: HashMap<MuID, P<MuType>>,
    pub func_sigs: HashMap<MuID, P<MuFuncSig>>,
    pub constants: HashMap<MuID, P<Value>>,
    pub globals  : HashMap<MuID, P<GlobalCell>>,
    pub func_defs: HashMap<MuID, MuFunction>,
    pub func_decls: HashMap<MuID, MuFunctionVersion>,
    
//    id_name_map: HashMap<MuID, MuName>,
//    name_id_map: HashMap<MuName, MuID>
}

impl MuBundle {
    pub fn new() -> MuBundle {
        MuBundle {
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

pub struct MuIRNode {
    pub id: MuID,
    pub v: MuIRNodeKind
}

impl MuIRNode {
    pub fn new(id: MuID, v: MuIRNodeKind) -> MuIRNode {
        MuIRNode {
            id: id,
            v: v
        }
    }
}

pub enum MuIRNodeKind {
    Type,
    FuncSig,
    Var(MuVarNodeKind),
    FuncVer,
    BB,
    Inst
}

pub enum MuVarNodeKind {
    Global(MuGlobalVarNodeKind),
    Local(MuLocalVarNodeKind)
}

pub enum MuGlobalVarNodeKind {
    Const,
    Global,
    Func,
    ExpFunc
}

pub enum MuLocalVarNodeKind {
    NorParam,
    ExcParam,
    MuInstRes
}