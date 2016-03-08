use std::collections::HashMap;

use ast::ptr::P;
use ast::ir::*;
use ast::types::*;

pub struct VMContext {
    constants: HashMap<MuTag, P<Value>>,
    types: HashMap<MuTag, P<MuType_>>,
    func_sigs: HashMap<MuTag, P<MuFuncSig>>,
    funcs: HashMap<MuTag, MuFunction>
}

impl VMContext {
    pub fn new() -> VMContext {
        VMContext {
            constants: HashMap::new(),
            types: HashMap::new(),
            func_sigs: HashMap::new(),
            funcs: HashMap::new()
        }
    }
    
    pub fn declare_const(&mut self, const_name: MuTag, ty: P<MuType_>, val: Constant) -> P<Value> {
        debug_assert!(!self.constants.contains_key(const_name));
        
        let ret = P(Value::Constant(MuConstant{ty: ty, val: val}));
        self.constants.insert(const_name, ret.clone());
        
        ret
    }
    
    pub fn declare_type(&mut self, type_name: MuTag, ty: P<MuType_>) -> P<MuType_> {
        debug_assert!(!self.types.contains_key(type_name));
        
        self.types.insert(type_name, ty.clone());
        
        ty
    }
    
    pub fn declare_func_sig(&mut self, sig_name: MuTag, ret_tys: Vec<P<MuType_>>, arg_tys: Vec<P<MuType_>>) -> P<MuFuncSig> {
        debug_assert!(!self.func_sigs.contains_key(sig_name));
        
        let ret = P(MuFuncSig{ret_tys: ret_tys, arg_tys: arg_tys});
        self.func_sigs.insert(sig_name, ret.clone());
        
        ret
    }
    
    pub fn declare_func (&mut self, fn_name: MuTag, sig: P<MuFuncSig>, entry: MuTag, blocks: Vec<(MuTag, Block)>) {
        debug_assert!(!self.funcs.contains_key(fn_name));
        
        let ret = MuFunction{fn_name: fn_name, sig: sig, entry: entry, blocks: blocks};
        self.funcs.insert(fn_name, ret);
    } 
}