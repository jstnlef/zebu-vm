extern crate immix_rust as gc;

use std::collections::HashMap;

use ast::ptr::P;
use ast::ir::*;
use ast::types::*;
use compiler::backend;
use compiler::backend::BackendTypeInfo;
use vm::machine_code::CompiledFunction;
use vm::vm_options::VMOptions;

use std::sync::RwLock;
use std::cell::RefCell;
use std::sync::atomic::{AtomicUsize, AtomicBool, ATOMIC_BOOL_INIT, ATOMIC_USIZE_INIT, Ordering};

pub struct VM {
    next_id: AtomicUsize,
    is_running: AtomicBool,
    
    id_name_map: RwLock<HashMap<MuID, MuName>>,
    name_id_map: RwLock<HashMap<MuName, MuID>>,
    
    constants: RwLock<HashMap<MuName, P<Value>>>,
    
    types: RwLock<HashMap<MuName, P<MuType>>>,
    backend_type_info: RwLock<HashMap<P<MuType>, BackendTypeInfo>>,
    
    globals: RwLock<HashMap<MuName, P<GlobalCell>>>,
    
    func_sigs: RwLock<HashMap<MuName, P<MuFuncSig>>>,
    func_vers: RwLock<HashMap<(MuID, MuID), RefCell<MuFunctionVersion>>>,
    funcs: RwLock<HashMap<MuID, RefCell<MuFunction>>>,
    
    compiled_funcs: RwLock<HashMap<MuID, RefCell<CompiledFunction>>>
}

impl <'a> VM {
    pub fn new() -> VM {
        let ret = VM {
            next_id: ATOMIC_USIZE_INIT,
            is_running: ATOMIC_BOOL_INIT,
            
            id_name_map: RwLock::new(HashMap::new()),
            name_id_map: RwLock::new(HashMap::new()),
            
            constants: RwLock::new(HashMap::new()),
            
            types: RwLock::new(HashMap::new()),
            backend_type_info: RwLock::new(HashMap::new()),
            
            globals: RwLock::new(HashMap::new()),
            
            func_sigs: RwLock::new(HashMap::new()),
            func_vers: RwLock::new(HashMap::new()),
            funcs: RwLock::new(HashMap::new()),
            compiled_funcs: RwLock::new(HashMap::new())
        };
        
        ret.is_running.store(false, Ordering::SeqCst);
        ret.next_id.store(RESERVED_NODE_IDS_FOR_MACHINE, Ordering::SeqCst);
        
        let options = VMOptions::default();
        gc::gc_init(options.immix_size, options.lo_size, options.n_gcthreads);
        
        ret
    }
    
    pub fn next_id(&self) -> MuID {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }
    
    pub fn run_vm(&self) {
        self.is_running.store(true, Ordering::SeqCst);
    }
    
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::Relaxed)
    }
    
    pub fn declare_const(&self, const_name: MuName, ty: P<MuType>, val: Constant) -> P<Value> {
        let mut constants = self.constants.write().unwrap();
        debug_assert!(!constants.contains_key(const_name));
        
        let ret = P(Value{tag: const_name, ty: ty, v: Value_::Constant(val)});
        constants.insert(const_name, ret.clone());
        
        ret
    }
    
    pub fn declare_global(&self, global_name: MuName, ty: P<MuType>) -> P<Value> {
        let global = P(GlobalCell{tag: global_name, ty: ty.clone()});
        
        let mut globals = self.globals.write().unwrap();
        globals.insert(global_name, global.clone());
        
        P(Value{
            tag: "",
            ty: P(MuType::iref(ty)),
            v: Value_::Global(global.clone())
        })
    }
    
    pub fn declare_type(&self, type_name: MuName, ty: P<MuType>) -> P<MuType> {
        let mut types = self.types.write().unwrap();
        debug_assert!(!types.contains_key(type_name));
        
        types.insert(type_name, ty.clone());
        
        ty
    }
    
    pub fn declare_func_sig(&self, sig_name: MuName, ret_tys: Vec<P<MuType>>, arg_tys: Vec<P<MuType>>) -> P<MuFuncSig> {
        let mut func_sigs = self.func_sigs.write().unwrap();
        debug_assert!(!func_sigs.contains_key(&sig_name));
        
        let ret = P(MuFuncSig{ret_tys: ret_tys, arg_tys: arg_tys});
        func_sigs.insert(sig_name, ret.clone());
        
        ret
    }
    
    pub fn declare_func (&self, func: MuFunction) {
        info!("declare function {:?}", func);
        let mut funcs = self.funcs.write().unwrap();
        funcs.insert(func.id, RefCell::new(func));
    }
    
    pub fn define_func_version (&self, func_ver: MuFunctionVersion) {
        info!("define function {} with version {}", func_ver.func_id, func_ver.id);
        // record this version
        let func_ver_key = (func_ver.func_id, func_ver.id);
        {
            let mut func_vers = self.func_vers.write().unwrap();
            func_vers.insert(func_ver_key, RefCell::new(func_ver));
        }
        
        // acquire a reference to the func_ver
        let func_vers = self.func_vers.read().unwrap();
        let func_ver = func_vers.get(&func_ver_key).unwrap().borrow();
        
        // change current version to this (obsolete old versions)
        let funcs = self.funcs.read().unwrap();
        debug_assert!(funcs.contains_key(&func_ver.func_id)); // it should be declared before defining
        let mut func = funcs.get(&func_ver.func_id).unwrap().borrow_mut();
        
        if func.cur_ver.is_some() {
            let obsolete_ver = func.cur_ver.unwrap();
            func.all_vers.push(obsolete_ver);
            
            // redefinition happens here
            // do stuff
        }
        func.cur_ver = Some(func_ver.id);        
    }
    
    pub fn add_compiled_func (&self, func: CompiledFunction) {
        debug_assert!(self.funcs.read().unwrap().contains_key(&func.func_ver_id));

        self.compiled_funcs.write().unwrap().insert(func.func_ver_id, RefCell::new(func));
    }
    
    pub fn get_backend_type_info(&self, ty: &P<MuType>) -> BackendTypeInfo {
        {
            let read_lock = self.backend_type_info.read().unwrap();
        
            match read_lock.get(ty) {
                Some(info) => {return info.clone();},
                None => {}
            }
        }
        
        let resolved = backend::resolve_backend_type_info(ty, self);
        
        let mut write_lock = self.backend_type_info.write().unwrap();
        write_lock.insert(ty.clone(), resolved.clone());
        
        resolved        
    }
    
    pub fn get_id_of(&self, name: MuName) -> MuID {
        *self.name_id_map.read().unwrap().get(&name).unwrap()
    }
    
    pub fn get_name_of(&self, id: MuID) -> MuName {
        *self.id_name_map.read().unwrap().get(&id).unwrap()
    }
    
    pub fn globals(&self) -> &RwLock<HashMap<MuName, P<GlobalCell>>> {
        &self.globals
    }
    
    pub fn funcs(&self) -> &RwLock<HashMap<MuID, RefCell<MuFunction>>> {
        &self.funcs
    }
    
    pub fn func_vers(&self) -> &RwLock<HashMap<(MuID, MuID), RefCell<MuFunctionVersion>>> {
        &self.func_vers
    }
    
    pub fn compiled_funcs(&self) -> &RwLock<HashMap<MuID, RefCell<CompiledFunction>>> {
        &self.compiled_funcs
    }
}
