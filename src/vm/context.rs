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
    
    types: RwLock<HashMap<MuID, P<MuType>>>,
    backend_type_info: RwLock<HashMap<MuID, P<BackendTypeInfo>>>,
    
    constants: RwLock<HashMap<MuID, P<Value>>>,
    globals: RwLock<HashMap<MuID, P<Value>>>,
    
    func_sigs: RwLock<HashMap<MuID, P<MuFuncSig>>>,
    // key: (func_id, func_ver_id)
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
        ret.next_id.store(USER_ID_START, Ordering::SeqCst);
        
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
    
    pub fn set_name(&self, entity: &MuEntity, name: MuName) {
        let id = entity.id();
        entity.set_name(name);
        
        let mut map = self.id_name_map.write().unwrap();
        map.insert(id, name);
        
        let mut map2 = self.name_id_map.write().unwrap();
        map2.insert(name, id);
    }
    
    pub fn id_of(&self, name: MuName) -> MuID {
        let map = self.name_id_map.read().unwrap();
        *map.get(name).unwrap()
    }
    
    pub fn name_of(&self, id: MuID) -> MuName {
        let map = self.id_name_map.read().unwrap();
        map.get(&id).unwrap()
    }
    
    pub fn declare_const(&self, id: MuID, ty: P<MuType>, val: Constant) -> P<Value> {
        let mut constants = self.constants.write().unwrap();
        debug_assert!(!constants.contains_key(&id));
        
        let ret = P(Value{hdr: MuEntityHeader::unnamed(id), ty: ty, v: Value_::Constant(val)});
        constants.insert(id, ret.clone());
        
        ret
    }
    
    pub fn declare_global(&self, id: MuID, ty: P<MuType>) -> P<Value> {
        let global = P(Value{
            hdr: MuEntityHeader::unnamed(id),
            ty: P(MuType::new(self.next_id(), MuType_::iref(ty.clone()))),
            v: Value_::Global(ty)
        });
        
        let mut globals = self.globals.write().unwrap();
        globals.insert(id, global.clone());
        
        global
    }
    
    pub fn declare_type(&self, id: MuID, ty: MuType_) -> P<MuType> {
        let ty = P(MuType{hdr: MuEntityHeader::unnamed(id), v: ty});
        
        let mut types = self.types.write().unwrap();
        debug_assert!(!types.contains_key(&id));
        
        types.insert(ty.id(), ty.clone());
        
        ty
    }
    
    pub fn declare_func_sig(&self, id: MuID, ret_tys: Vec<P<MuType>>, arg_tys: Vec<P<MuType>>) -> P<MuFuncSig> {
        let mut func_sigs = self.func_sigs.write().unwrap();
        debug_assert!(!func_sigs.contains_key(&id));
        
        let ret = P(MuFuncSig{hdr: MuEntityHeader::unnamed(id), ret_tys: ret_tys, arg_tys: arg_tys});
        func_sigs.insert(id, ret.clone());
        
        ret
    }
    
    pub fn declare_func (&self, func: MuFunction) {
        info!("declare function {}", func);
        let mut funcs = self.funcs.write().unwrap();
        funcs.insert(func.id(), RefCell::new(func));
    }
    
    pub fn define_func_version (&self, func_ver: MuFunctionVersion) {
        info!("define function version {}", func_ver);
        // record this version
        let func_ver_key = (func_ver.func_id, func_ver.id());
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
        func.cur_ver = Some(func_ver.id());        
    }
    
    pub fn add_compiled_func (&self, func: CompiledFunction) {
        debug_assert!(self.funcs.read().unwrap().contains_key(&func.func_id));
        debug_assert!(self.func_vers.read().unwrap().contains_key(&(func.func_id, func.func_ver_id)));

        self.compiled_funcs.write().unwrap().insert(func.func_ver_id, RefCell::new(func));
    }
    
    pub fn get_backend_type_info(&self, tyid: MuID) -> P<BackendTypeInfo> {        
        {
            let read_lock = self.backend_type_info.read().unwrap();
        
            match read_lock.get(&tyid) {
                Some(info) => {return info.clone();},
                None => {}
            }
        }

        let types = self.types.read().unwrap();
        let ty = types.get(&tyid).unwrap();
        let resolved = P(backend::resolve_backend_type_info(ty, self));
        
        let mut write_lock = self.backend_type_info.write().unwrap();
        write_lock.insert(tyid, resolved.clone());
        
        resolved        
    }
    
    pub fn get_id_of(&self, name: MuName) -> MuID {
        *self.name_id_map.read().unwrap().get(&name).unwrap()
    }
    
    pub fn get_name_of(&self, id: MuID) -> MuName {
        *self.id_name_map.read().unwrap().get(&id).unwrap()
    }
    
    pub fn globals(&self) -> &RwLock<HashMap<MuID, P<Value>>> {
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
