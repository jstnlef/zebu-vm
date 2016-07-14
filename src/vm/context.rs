use std::collections::HashMap;

use ast::ptr::P;
use ast::ir::*;
use ast::types::*;
use vm::machine_code::CompiledFunction;

use std::sync::RwLock;
use std::cell::RefCell;

pub struct VMContext {
    constants: RwLock<HashMap<MuTag, P<Value>>>,
    types: RwLock<HashMap<MuTag, P<MuType>>>,
    func_sigs: RwLock<HashMap<MuTag, P<MuFuncSig>>>,
    funcs: RwLock<HashMap<MuTag, RefCell<MuFunctionVersion>>>,
    
    compiled_funcs: RwLock<HashMap<MuTag, RefCell<CompiledFunction>>>
}

impl <'a> VMContext {
    pub fn new() -> VMContext {
        VMContext {
            constants: RwLock::new(HashMap::new()),
            types: RwLock::new(HashMap::new()),
            func_sigs: RwLock::new(HashMap::new()),
            funcs: RwLock::new(HashMap::new()),
            compiled_funcs: RwLock::new(HashMap::new())
        }
    }
    
    pub fn declare_const(&self, const_name: MuTag, ty: P<MuType>, val: Constant) -> P<Value> {
        let mut constants = self.constants.write().unwrap();
        debug_assert!(!constants.contains_key(const_name));
        
        let ret = P(Value{tag: const_name, ty: ty, v: Value_::Constant(val)});
        constants.insert(const_name, ret.clone());
        
        ret
    }
    
    pub fn declare_type(&self, type_name: MuTag, ty: P<MuType>) -> P<MuType> {
        let mut types = self.types.write().unwrap();
        debug_assert!(!types.contains_key(type_name));
        
        types.insert(type_name, ty.clone());
        
        ty
    }
    
    pub fn declare_func_sig(&self, sig_name: MuTag, ret_tys: Vec<P<MuType>>, arg_tys: Vec<P<MuType>>) -> P<MuFuncSig> {
        let mut func_sigs = self.func_sigs.write().unwrap();
        debug_assert!(!func_sigs.contains_key(sig_name));
        
        let ret = P(MuFuncSig{ret_tys: ret_tys, arg_tys: arg_tys});
        func_sigs.insert(sig_name, ret.clone());
        
        ret
    }
    
    pub fn declare_func (&self, func: MuFunctionVersion) {
        let mut funcs = self.funcs.write().unwrap();
        debug_assert!(!funcs.contains_key(func.fn_name));
        
        funcs.insert(func.fn_name, RefCell::new(func));
    }
    
    pub fn add_compiled_func (&self, func: CompiledFunction) {
        debug_assert!(self.funcs.read().unwrap().contains_key(func.fn_name));

        self.compiled_funcs.write().unwrap().insert(func.fn_name, RefCell::new(func));
    }
    
    pub fn funcs(&self) -> &RwLock<HashMap<MuTag, RefCell<MuFunctionVersion>>> {
        &self.funcs
    }
    
    pub fn compiled_funcs(&self) -> &RwLock<HashMap<MuTag, RefCell<CompiledFunction>>> {
        &self.compiled_funcs
    }
}
