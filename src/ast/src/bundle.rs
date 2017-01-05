use ir::*;
use ptr::*;
use types::*;
use utils::Address;

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

#[derive(Copy, Clone, Debug)]
pub struct APIHandleKey {
    pub id: MuID,
    pub v: APIHandleKeyKind
} 

#[derive(Copy, Clone, Debug)]
pub enum APIHandleKeyKind {
    Int,
    Float,
    Double,
    UPtr,
    UFP,
    
    // SeqValue
    Struct,
    Array,
    Vector,
    
    // GenRef
    Ref,
    IRef,
    TagRef64,
    FuncRef,
    ThreadRef,
    StackRef,
    FCRef, // frame cursor ref        
    
    // GenRef->IR
    Bundle,
    
    // GenRef->IR->Child
    Type,
    FuncSig,
    FuncVer,
    BB,
    Inst,
    
    // GenRef->IR->Child->Var->Global
    Const,
    Global,
    Func,
    ExpFunc,
    
    // GenRef->IR->Child->Var->Local
    NorParam,
    ExcParam,
    InstRes,    
}

macro_rules! handle_constructor {
    ($fn_name: ident, $kind: ident) => {
        pub fn $fn_name(id: MuID) -> APIHandleKey {
            APIHandleKey{
                id: id, v: APIHandleKeyKind::$kind
            }
        }
    }
}

handle_constructor!(handle_int, Int);
handle_constructor!(handle_float, Float);
handle_constructor!(handle_double, Double);
handle_constructor!(handle_uptr, UPtr);
handle_constructor!(handle_ufp, UFP);

handle_constructor!(handle_struct, Struct);
handle_constructor!(handle_array, Array);
handle_constructor!(handle_vector, Vector);

handle_constructor!(handle_ref, Ref);
handle_constructor!(handle_iref, IRef);
handle_constructor!(handle_tagref64, TagRef64);
handle_constructor!(handle_funcref, FuncRef);
handle_constructor!(handle_threadref, ThreadRef);
handle_constructor!(handle_stackref, StackRef);
handle_constructor!(handle_fcref, FCRef);

handle_constructor!(handle_bundle, Bundle);

handle_constructor!(handle_type, Type);
handle_constructor!(handle_funcsig, FuncSig);
handle_constructor!(handle_funcver, FuncVer);
handle_constructor!(handle_bb, BB);
handle_constructor!(handle_inst, Inst);

handle_constructor!(handle_const, Const);
handle_constructor!(handle_global, Global);
handle_constructor!(handle_func, Func);
handle_constructor!(handle_expfunc, ExpFunc);

handle_constructor!(handle_norparam, NorParam);
handle_constructor!(handle_excparam, ExcParam);
handle_constructor!(handle_instres, InstRes);

#[derive(Clone)]
pub enum APIHandleValue {
    Int(u64),
    Float(f32),
    Double(f64),
    UPtr(Address),
    UFP(Address),

    Struct(Vec<APIHandleValue>),
    Array(Vec<APIHandleValue>),
    Vector(Vec<APIHandleValue>),

    Ref(Address),
    IRef(Address),
    TagRef64(u64),
    FuncRef(MuID),
    ThreadRef,
    StackRef,
    FCRef,
    IBRef
}