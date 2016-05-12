use std::collections::HashMap;

use ast::ptr::P;
use ast::ir::*;
use ast::types::*;
use vm::CompiledFunction;

use std::cell::RefCell;

pub struct VMContext {
    constants: HashMap<MuTag, P<Value>>,
    types: HashMap<MuTag, P<MuType>>,
    func_sigs: HashMap<MuTag, P<MuFuncSig>>,
    funcs: HashMap<MuTag, RefCell<MuFunction>>,
    
    compiled_funcs: HashMap<MuTag, RefCell<CompiledFunction>>
}

impl VMContext {
    pub fn new() -> VMContext {
        VMContext {
            constants: HashMap::new(),
            types: HashMap::new(),
            func_sigs: HashMap::new(),
            funcs: HashMap::new(),
            compiled_funcs: HashMap::new()
        }
    }
    
    pub fn declare_const(&mut self, const_name: MuTag, ty: P<MuType>, val: Constant) -> P<Value> {
        debug_assert!(!self.constants.contains_key(const_name));
        
        let ret = P(Value{tag: const_name, ty: ty, v: Value_::Constant(val)});
        self.constants.insert(const_name, ret.clone());
        
        ret
    }
    
    pub fn declare_type(&mut self, type_name: MuTag, ty: P<MuType>) -> P<MuType> {
        debug_assert!(!self.types.contains_key(type_name));
        
        self.types.insert(type_name, ty.clone());
        
        ty
    }
    
    pub fn declare_func_sig(&mut self, sig_name: MuTag, ret_tys: Vec<P<MuType>>, arg_tys: Vec<P<MuType>>) -> P<MuFuncSig> {
        debug_assert!(!self.func_sigs.contains_key(sig_name));
        
        let ret = P(MuFuncSig{ret_tys: ret_tys, arg_tys: arg_tys});
        self.func_sigs.insert(sig_name, ret.clone());
        
        ret
    }
    
    pub fn declare_func (&mut self, func: MuFunction) {
        debug_assert!(!self.funcs.contains_key(func.fn_name));
        
        self.funcs.insert(func.fn_name, RefCell::new(func));
    }
    
    pub fn add_compiled_func (&mut self, func: CompiledFunction) {
        debug_assert!(self.funcs.contains_key(func.fn_name));

        self.compiled_funcs.insert(func.fn_name, RefCell::new(func));
    }
    
    pub fn get_func(&self, fn_name: MuTag) -> Option<&RefCell<MuFunction>> {
        self.funcs.get(fn_name)
    }
}