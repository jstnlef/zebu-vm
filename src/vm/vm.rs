use std::collections::HashMap;

use ast::ptr::P;
use ast::ir::*;
use ast::types;
use ast::types::*;
use compiler::backend;
use compiler::backend::BackendTypeInfo;
use compiler::machine_code::CompiledFunction;
use vm::vm_options::VMOptions;
use runtime::thread::*;
use runtime::ValueLocation;
use utils::Address;
use runtime::mm as gc;

use rustc_serialize::{Encodable, Encoder, Decodable, Decoder};

use std::path;
use std::sync::RwLock;
use std::sync::atomic::{AtomicUsize, AtomicBool, ATOMIC_BOOL_INIT, ATOMIC_USIZE_INIT, Ordering};
use std::thread::JoinHandle;

pub struct VM {
    // serialize
    // 0
    next_id: AtomicUsize,
    // 1
    id_name_map: RwLock<HashMap<MuID, MuName>>,
    // 2
    name_id_map: RwLock<HashMap<MuName, MuID>>,
    // 3
    types: RwLock<HashMap<MuID, P<MuType>>>,
    // 4
    backend_type_info: RwLock<HashMap<MuID, Box<BackendTypeInfo>>>,
    // 5
    constants: RwLock<HashMap<MuID, P<Value>>>,
    // 6
    globals: RwLock<HashMap<MuID, P<Value>>>,
    // 7
    func_sigs: RwLock<HashMap<MuID, P<MuFuncSig>>>,
    // 8
    funcs: RwLock<HashMap<MuID, RwLock<MuFunction>>>,
    // 9
    func_vers: RwLock<HashMap<MuID, RwLock<MuFunctionVersion>>>,
    // 10
    pub primordial: RwLock<Option<MuPrimordialThread>>,
    // 11
    is_running: AtomicBool,
    
    // partially serialize
    // 12
    compiled_funcs: RwLock<HashMap<MuID, RwLock<CompiledFunction>>>,    
}

const VM_SERIALIZE_FIELDS : usize = 12;

impl Encodable for VM {
    fn encode<S: Encoder> (&self, s: &mut S) -> Result<(), S::Error> {
        // serialize VM_SERIALIZE_FIELDS fields
        // PLUS ONE extra global STRUCT_TAG_MAP
        s.emit_struct("VM", VM_SERIALIZE_FIELDS + 1, |s| {
            // next_id
            try!(s.emit_struct_field("next_id", 0, |s| {
                s.emit_usize(self.next_id.load(Ordering::SeqCst))
            }));
                
            // id_name_map
            {
                let map : &HashMap<MuID, MuName> = &self.id_name_map.read().unwrap();            
                try!(s.emit_struct_field("id_name_map", 1, |s| map.encode(s)));
            }
            
            // name_id_map
            {
                let map : &HashMap<MuName, MuID> = &self.name_id_map.read().unwrap(); 
                try!(s.emit_struct_field("name_id_map", 2, |s| map.encode(s)));
            }
            
            // types
            {
                let types = &self.types.read().unwrap();
                try!(s.emit_struct_field("types", 3, |s| types.encode(s)));
            }
            // STRUCT_TAG_MAP
            {
                let struct_tag_map = types::STRUCT_TAG_MAP.read().unwrap();
                try!(s.emit_struct_field("struct_tag_map", 4, |s| struct_tag_map.encode(s)));
            }
            
            // backend_type_info
            {
                let backend_type_info : &HashMap<_, _> = &self.backend_type_info.read().unwrap();
                try!(s.emit_struct_field("backend_type_info", 5, |s| backend_type_info.encode(s)));
            }
            
            // constants
            {
                let constants : &HashMap<_, _> = &self.constants.read().unwrap();
                try!(s.emit_struct_field("constants", 6, |s| constants.encode(s)));
            }
            
            // globals
            {
                let globals: &HashMap<_, _> = &self.globals.read().unwrap();
                try!(s.emit_struct_field("globals", 7, |s| globals.encode(s)));
            }
            
            // func sigs
            {
                let func_sigs: &HashMap<_, _> = &self.func_sigs.read().unwrap();
                try!(s.emit_struct_field("func_sigs", 8, |s| func_sigs.encode(s)));
            }
            
            // funcs
            {
                let funcs : &HashMap<_, _> = &self.funcs.read().unwrap();
                try!(s.emit_struct_field("funcs", 9, |s| {
                    s.emit_map(funcs.len(), |s| {
                        let mut i = 0;
                        for (k,v) in funcs.iter() {
                            s.emit_map_elt_key(i, |s| k.encode(s)).ok();
                            let func : &MuFunction = &v.read().unwrap();
                            s.emit_map_elt_val(i, |s| func.encode(s)).ok();
                            i += 1;
                        }
                        Ok(())
                    })
                }));
            }
            
            // func_vers
            {
                let func_vers : &HashMap<_, _> = &self.func_vers.read().unwrap();
                try!(s.emit_struct_field("func_vers", 10, |s| {
                    s.emit_map(func_vers.len(), |s| {
                        let mut i = 0;
                        for (k, v) in func_vers.iter() {
                            try!(s.emit_map_elt_key(i, |s| k.encode(s)));
                            let func_ver : &MuFunctionVersion = &v.read().unwrap();
                            try!(s.emit_map_elt_val(i, |s| func_ver.encode(s)));
                            i += 1;
                        }
                        Ok(())
                    })
                }));
            }
            
            // primordial
            {
                let primordial = &self.primordial.read().unwrap();
                try!(s.emit_struct_field("primordial", 11, |s| primordial.encode(s)));
            }
            
            // is_running
            {
                try!(s.emit_struct_field("is_running", 12, |s| self.is_running.load(Ordering::SeqCst).encode(s)));
            }
            
            Ok(())
        })
    }
}

impl Decodable for VM {
    fn decode<D: Decoder>(d: &mut D) -> Result<VM, D::Error> {
        d.read_struct("VM", VM_SERIALIZE_FIELDS + 1, |d| {
            // next_id
            let next_id = try!(d.read_struct_field("next_id", 0, |d| {
                d.read_usize()
            }));
            
            // id_name_map
            let id_name_map = try!(d.read_struct_field("id_name_map", 1, |d| Decodable::decode(d)));
            
            // name_id_map
            let name_id_map = try!(d.read_struct_field("name_id_map", 2, |d| Decodable::decode(d)));
            
            // types
            let types = try!(d.read_struct_field("types", 3, |d| Decodable::decode(d)));
            {
                // struct tag map
                let mut struct_tag_map : HashMap<MuName, StructType_> = try!(d.read_struct_field("struct_tag_map", 4, |d| Decodable::decode(d)));
                
                let mut map_guard = types::STRUCT_TAG_MAP.write().unwrap();
                map_guard.clear();
                for (k, v) in struct_tag_map.drain() {
                    map_guard.insert(k, v);
                }
            }
            
            // backend_type_info
            let backend_type_info = try!(d.read_struct_field("backend_type_info", 5, |d| Decodable::decode(d)));
            
            // constants
            let constants = try!(d.read_struct_field("constants", 6, |d| Decodable::decode(d)));
            
            // globals
            let globals = try!(d.read_struct_field("globals", 7, |d| Decodable::decode(d)));
            
            // func sigs
            let func_sigs = try!(d.read_struct_field("func_sigs", 8, |d| Decodable::decode(d)));
            
            // funcs
            let funcs = try!(d.read_struct_field("funcs", 9, |d| {
                d.read_map(|d, len| {
                    let mut map = HashMap::new();
                    for i in 0..len {
                        let key = try!(d.read_map_elt_key(i, |d| Decodable::decode(d)));
                        let val = RwLock::new(try!(d.read_map_elt_val(i, |d| Decodable::decode(d))));
                        map.insert(key, val);
                    }
                    Ok(map)
                })
            }));
            
            // func_vers
            let func_vers = try!(d.read_struct_field("func_vers", 10, |d| {
                d.read_map(|d, len| {
                    let mut map = HashMap::new();
                    for i in 0..len {
                        let key = try!(d.read_map_elt_key(i, |d| Decodable::decode(d)));
                        let val = RwLock::new(try!(d.read_map_elt_val(i, |d| Decodable::decode(d))));
                        map.insert(key, val);
                    }
                    Ok(map)
                })
            }));
            
            // primordial
            let primordial = try!(d.read_struct_field("primordial", 11, |d| Decodable::decode(d)));
            
            let is_running = try!(d.read_struct_field("is_running", 12, |d| Decodable::decode(d)));
            
            let vm = VM{
                next_id: ATOMIC_USIZE_INIT,
                id_name_map: RwLock::new(id_name_map),
                name_id_map: RwLock::new(name_id_map),
                types: RwLock::new(types),
                backend_type_info: RwLock::new(backend_type_info),
                constants: RwLock::new(constants),
                globals: RwLock::new(globals),
                func_sigs: RwLock::new(func_sigs),
                funcs: RwLock::new(funcs),
                func_vers: RwLock::new(func_vers),
                primordial: RwLock::new(primordial),
                is_running: ATOMIC_BOOL_INIT,
                
                // not serialized
                compiled_funcs: RwLock::new(HashMap::new()),
            };
            
            vm.next_id.store(next_id, Ordering::SeqCst);
            vm.is_running.store(is_running, Ordering::SeqCst);
            
            Ok(vm)
        })
    }
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
            compiled_funcs: RwLock::new(HashMap::new()),
            
            primordial: RwLock::new(None)
        };
        
        ret.is_running.store(false, Ordering::SeqCst);
        ret.next_id.store(USER_ID_START, Ordering::SeqCst);
        
        let options = VMOptions::default();
        gc::gc_init(options.immix_size, options.lo_size, options.n_gcthreads);
        
        ret
    }
    
    pub fn resume_vm(serialized_vm: &str) -> VM {
        use rustc_serialize::json;
        
        let vm = json::decode(serialized_vm).unwrap();
        
        let options = VMOptions::default();
        gc::gc_init(options.immix_size, options.lo_size, options.n_gcthreads);
        
        vm
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
        entity.set_name(name.clone());
        
        let mut map = self.id_name_map.write().unwrap();
        map.insert(id, name.clone());
        
        let mut map2 = self.name_id_map.write().unwrap();
        map2.insert(name, id);
    }
    
    pub fn id_of(&self, name: &str) -> MuID {
        let map = self.name_id_map.read().unwrap();
        *map.get(&name.to_string()).unwrap()
    }
    
    pub fn name_of(&self, id: MuID) -> MuName {
        let map = self.id_name_map.read().unwrap();
        map.get(&id).unwrap().clone()
    }
    
    pub fn declare_const(&self, id: MuID, ty: P<MuType>, val: Constant) -> P<Value> {
        let mut constants = self.constants.write().unwrap();
        debug_assert!(!constants.contains_key(&id));
        
        let ret = P(Value{hdr: MuEntityHeader::unnamed(id), ty: ty, v: Value_::Constant(val)});
        constants.insert(id, ret.clone());
        
        ret
    }
    
    pub fn get_const(&self, id: MuID) -> P<Value> {
        let const_lock = self.constants.read().unwrap();
        match const_lock.get(&id) {
            Some(ret) => ret.clone(),
            None => panic!("cannot find const #{}", id)
        }
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
    
    pub fn get_type(&self, id: MuID) -> P<MuType> {
        let type_lock = self.types.read().unwrap();
        match type_lock.get(&id) {
            Some(ret) => ret.clone(),
            None => panic!("cannot find type #{}", id)
        }
    }    
    
    pub fn declare_func_sig(&self, id: MuID, ret_tys: Vec<P<MuType>>, arg_tys: Vec<P<MuType>>) -> P<MuFuncSig> {
        let mut func_sigs = self.func_sigs.write().unwrap();
        debug_assert!(!func_sigs.contains_key(&id));
        
        let ret = P(MuFuncSig{hdr: MuEntityHeader::unnamed(id), ret_tys: ret_tys, arg_tys: arg_tys});
        func_sigs.insert(id, ret.clone());
        
        ret
    }
    
    pub fn get_func_sig(&self, id: MuID) -> P<MuFuncSig> {
        let func_sig_lock = self.func_sigs.read().unwrap();
        match func_sig_lock.get(&id) {
            Some(ret) => ret.clone(),
            None => panic!("cannot find func sig #{}", id)
        }
    }
    
    pub fn declare_func (&self, func: MuFunction) {
        info!("declare function {}", func);
        let mut funcs = self.funcs.write().unwrap();
        funcs.insert(func.id(), RwLock::new(func));
    }
    
    pub fn define_func_version (&self, func_ver: MuFunctionVersion) {
        info!("define function version {}", func_ver);
        // record this version
        let func_ver_id = func_ver.id();
        {
            let mut func_vers = self.func_vers.write().unwrap();
            func_vers.insert(func_ver_id, RwLock::new(func_ver));
        }
        
        // acquire a reference to the func_ver
        let func_vers = self.func_vers.read().unwrap();
        let func_ver = func_vers.get(&func_ver_id).unwrap().write().unwrap();
        
        // change current version to this (obsolete old versions)
        let funcs = self.funcs.read().unwrap();
        debug_assert!(funcs.contains_key(&func_ver.func_id)); // it should be declared before defining
        let mut func = funcs.get(&func_ver.func_id).unwrap().write().unwrap();
        
        func.new_version(func_ver.id());
        
        // redefinition happens here
        // do stuff        
    }
    
    pub fn add_compiled_func (&self, func: CompiledFunction) {
        debug_assert!(self.funcs.read().unwrap().contains_key(&func.func_id));
        debug_assert!(self.func_vers.read().unwrap().contains_key(&func.func_ver_id));

        self.compiled_funcs.write().unwrap().insert(func.func_ver_id, RwLock::new(func));
    }
    
    pub fn get_backend_type_info(&self, tyid: MuID) -> Box<BackendTypeInfo> {        
        {
            let read_lock = self.backend_type_info.read().unwrap();
        
            match read_lock.get(&tyid) {
                Some(info) => {return info.clone();},
                None => {}
            }
        }

        let types = self.types.read().unwrap();
        let ty = types.get(&tyid).unwrap();
        let resolved = Box::new(backend::resolve_backend_type_info(ty, self));
        
        let mut write_lock = self.backend_type_info.write().unwrap();
        write_lock.insert(tyid, resolved.clone());
        
        resolved        
    }
    
    pub fn get_type_size(&self, tyid: MuID) -> ByteSize {
        self.get_backend_type_info(tyid).size
    }
    
    pub fn globals(&self) -> &RwLock<HashMap<MuID, P<Value>>> {
        &self.globals
    }
    
    pub fn funcs(&self) -> &RwLock<HashMap<MuID, RwLock<MuFunction>>> {
        &self.funcs
    }
    
    pub fn func_vers(&self) -> &RwLock<HashMap<MuID, RwLock<MuFunctionVersion>>> {
        &self.func_vers
    }
    
    pub fn compiled_funcs(&self) -> &RwLock<HashMap<MuID, RwLock<CompiledFunction>>> {
        &self.compiled_funcs
    }
    
    pub fn types(&self) -> &RwLock<HashMap<MuID, P<MuType>>> {
        &self.types
    }
    
    pub fn func_sigs(&self) -> &RwLock<HashMap<MuID, P<MuFuncSig>>> {
        &self.func_sigs
    }
    
    pub fn resolve_function_address(&self, func_id: MuID) -> ValueLocation {
        let funcs = self.funcs.read().unwrap();
        let func : &MuFunction = &funcs.get(&func_id).unwrap().read().unwrap();
                
        if self.is_running() {
            unimplemented!()
        } else {
            ValueLocation::Relocatable(backend::RegGroup::GPR, func.name().unwrap())
        }
    }
    
    pub fn new_stack(&self, func_id: MuID) -> Box<MuStack> {
        let funcs = self.funcs.read().unwrap();
        let func : &MuFunction = &funcs.get(&func_id).unwrap().read().unwrap();
        
        Box::new(MuStack::new(self.next_id(), self.resolve_function_address(func_id), func))
    }
    
    pub fn make_primordial_thread(&self, func_id: MuID, args: Vec<Constant>) {
        let mut guard = self.primordial.write().unwrap();
        *guard = Some(MuPrimordialThread{func_id: func_id, args: args});
    }
    
    pub fn new_thread_normal(&self, mut stack: Box<MuStack>, threadlocal: Address, vals: Vec<ValueLocation>) -> JoinHandle<()> {
        let user_tls = {
            if threadlocal.is_zero() {
                None
            } else {
                Some(threadlocal)
            }
        };
        
        // set up arguments on stack
        stack.runtime_load_args(vals);
        
        MuThread::mu_thread_launch(self.next_id(), stack, user_tls, self)
    }
    
    #[allow(unused_variables)]
    pub fn make_boot_image(self, output: &path::Path) {
        use rustc_serialize::json;
        
        let serialized = json::encode(&self).unwrap();
        
        unimplemented!() 
    }
}