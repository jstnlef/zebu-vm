use std::collections::HashMap;

use ast::ptr::*;
use ast::ir::*;
use ast::inst::*;
use ast::types;
use ast::types::*;
use compiler::{Compiler, CompilerPolicy};
use compiler::backend;
use compiler::backend::BackendTypeInfo;
use compiler::machine_code::CompiledFunction;
use runtime::thread::*;
use runtime::ValueLocation;
use utils::ByteSize;
use utils::BitSize;
use utils::Address;
use runtime::mm as gc;
use vm::handle::*;
use vm::vm_options::VMOptions;
use vm::vm_options::MuLogLevel;

use rustc_serialize::{Encodable, Encoder, Decodable, Decoder};
use log::LogLevel;
use std::sync::Arc;
use std::path;
use std::sync::RwLock;
use std::sync::RwLockWriteGuard;
use std::sync::atomic::{AtomicUsize, AtomicBool, ATOMIC_BOOL_INIT, ATOMIC_USIZE_INIT, Ordering};

// FIXME:
// besides fields in VM, there are some 'globals' we need to persist
// such as STRUCT_TAG_MAP
// possibly INTERNAL_ID in ir.rs, internal types, etc

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
    pub global_locations: RwLock<HashMap<MuID, ValueLocation>>,
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
    // 12
    pub vm_options: VMOptions,
    
    // partially serialize
    // 13
    compiled_funcs: RwLock<HashMap<MuID, RwLock<CompiledFunction>>>,
}

const VM_SERIALIZE_FIELDS : usize = 14;

impl Encodable for VM {
    fn encode<S: Encoder> (&self, s: &mut S) -> Result<(), S::Error> {
        let mut field_i = 0;

        // serialize VM_SERIALIZE_FIELDS fields
        // PLUS ONE extra global STRUCT_TAG_MAP
        s.emit_struct("VM", VM_SERIALIZE_FIELDS + 2, |s| {
            // next_id
            trace!("...serializing next_id");
            try!(s.emit_struct_field("next_id", field_i, |s| {
                s.emit_usize(self.next_id.load(Ordering::SeqCst))
            }));
            field_i += 1;
                
            // id_name_map
            trace!("...serializing id_name_map");
            {
                let map : &HashMap<MuID, MuName> = &self.id_name_map.read().unwrap();            
                try!(s.emit_struct_field("id_name_map", field_i, |s| map.encode(s)));
            }
            field_i += 1;
            
            // name_id_map
            trace!("...serializing name_id_map");
            {
                let map : &HashMap<MuName, MuID> = &self.name_id_map.read().unwrap(); 
                try!(s.emit_struct_field("name_id_map", field_i, |s| map.encode(s)));
            }
            field_i += 1;
            
            // types
            trace!("...serializing types");
            {
                let types = &self.types.read().unwrap();
                try!(s.emit_struct_field("types", field_i, |s| types.encode(s)));
            }
            field_i += 1;

            // STRUCT_TAG_MAP
            trace!("...serializing struct_tag_map");
            {
                let struct_tag_map = types::STRUCT_TAG_MAP.read().unwrap();
                try!(s.emit_struct_field("struct_tag_map", field_i, |s| struct_tag_map.encode(s)));
            }
            field_i += 1;

            // HYBRID_TAG_MAP
            trace!("...serializing hybrid_tag_map");
            {
                let hybrid_tag_map = types::HYBRID_TAG_MAP.read().unwrap();
                try!(s.emit_struct_field("hybrid_tag_map", field_i, |s| hybrid_tag_map.encode(s)));
            }
            field_i += 1;
            
            // backend_type_info
            trace!("...serializing backend_type_info");
            {
                let backend_type_info : &HashMap<_, _> = &self.backend_type_info.read().unwrap();
                try!(s.emit_struct_field("backend_type_info", field_i, |s| backend_type_info.encode(s)));
            }
            field_i += 1;
            
            // constants
            trace!("...serializing constants");
            {
                let constants : &HashMap<_, _> = &self.constants.read().unwrap();
                try!(s.emit_struct_field("constants", field_i, |s| constants.encode(s)));
            }
            field_i += 1;
            
            // globals
            trace!("...serializing globals");
            {
                let globals: &HashMap<_, _> = &self.globals.read().unwrap();
                try!(s.emit_struct_field("globals", field_i, |s| globals.encode(s)));
            }
            field_i += 1;
            
            // func sigs
            trace!("...serializing func_sigs");
            {
                let func_sigs: &HashMap<_, _> = &self.func_sigs.read().unwrap();
                try!(s.emit_struct_field("func_sigs", field_i, |s| func_sigs.encode(s)));
            }
            field_i += 1;
            
            // funcs
            trace!("...serializing funcs");
            {
                let funcs : &HashMap<_, _> = &self.funcs.read().unwrap();
                try!(s.emit_struct_field("funcs", field_i, |s| {
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
            field_i += 1;
            
            // func_vers
            trace!("...serializing func_vers");
            {
                let func_vers : &HashMap<_, _> = &self.func_vers.read().unwrap();
                try!(s.emit_struct_field("func_vers", field_i, |s| {
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
            field_i += 1;

            // primordial
            trace!("...serializing primordial");
            {
                let primordial = &self.primordial.read().unwrap();
                try!(s.emit_struct_field("primordial", field_i, |s| primordial.encode(s)));
            }
            field_i += 1;
            
            // is_running
            trace!("...serializing is_running");
            {
                try!(s.emit_struct_field("is_running", field_i, |s| self.is_running.load(Ordering::SeqCst).encode(s)));
            }
            field_i += 1;

            // options
            trace!("...serializing vm_options");
            {
                try!(s.emit_struct_field("vm_options", field_i, |s| self.vm_options.encode(s)));
            }
            field_i += 1;
            
            // compiled_funcs
            trace!("...serializing compiled_funcs");
            {
                let compiled_funcs : &HashMap<_, _> = &self.compiled_funcs.read().unwrap();
                try!(s.emit_struct_field("compiled_funcs", field_i, |s| {
                    s.emit_map(compiled_funcs.len(), |s| {
                        let mut i = 0;
                        for (k, v) in compiled_funcs.iter() {
                            try!(s.emit_map_elt_key(i, |s| k.encode(s)));
                            let compiled_func : &CompiledFunction = &v.read().unwrap();
                            try!(s.emit_map_elt_val(i, |s| compiled_func.encode(s)));
                            i += 1;
                        }
                        Ok(())
                    })
                }));
            }
            field_i += 1;
            
            trace!("serializing finished");
            Ok(())
        })
    }
}

impl Decodable for VM {
    fn decode<D: Decoder>(d: &mut D) -> Result<VM, D::Error> {
        let mut field_i = 0;

        d.read_struct("VM", VM_SERIALIZE_FIELDS + 2, |d| {
            // next_id
            let next_id = try!(d.read_struct_field("next_id", field_i, |d| {
                d.read_usize()
            }));
            field_i += 1;
            
            // id_name_map
            let id_name_map = try!(d.read_struct_field("id_name_map", field_i, |d| Decodable::decode(d)));
            field_i += 1;

            // name_id_map
            let name_id_map = try!(d.read_struct_field("name_id_map", field_i, |d| Decodable::decode(d)));
            field_i += 1;
            
            // types
            let types = try!(d.read_struct_field("types", field_i, |d| Decodable::decode(d)));
            field_i += 1;

            // struct tag map
            {
                let mut struct_tag_map : HashMap<MuName, StructType_> = try!(d.read_struct_field("struct_tag_map", field_i, |d| Decodable::decode(d)));
                
                let mut map_guard = types::STRUCT_TAG_MAP.write().unwrap();
                map_guard.clear();
                for (k, v) in struct_tag_map.drain() {
                    map_guard.insert(k, v);
                }
                field_i += 1;
            }

            // hybrid tag map
            {
                let mut hybrid_tag_map : HashMap<MuName, HybridType_> = try!(d.read_struct_field("hybrid_tag_map", field_i, |d| Decodable::decode(d)));

                let mut map_guard = types::HYBRID_TAG_MAP.write().unwrap();
                map_guard.clear();
                for (k, v) in hybrid_tag_map.drain() {
                    map_guard.insert(k, v);
                }
                field_i += 1;
            }
            
            // backend_type_info
            let backend_type_info = try!(d.read_struct_field("backend_type_info", field_i, |d| Decodable::decode(d)));
            field_i += 1;
            
            // constants
            let constants = try!(d.read_struct_field("constants", field_i, |d| Decodable::decode(d)));
            field_i += 1;
            
            // globals
            let globals = try!(d.read_struct_field("globals", field_i, |d| Decodable::decode(d)));
            field_i += 1;
            
            // func sigs
            let func_sigs = try!(d.read_struct_field("func_sigs", field_i, |d| Decodable::decode(d)));
            field_i += 1;
            
            // funcs
            let funcs = try!(d.read_struct_field("funcs", field_i, |d| {
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
            field_i += 1;
            
            // func_vers
            let func_vers = try!(d.read_struct_field("func_vers", field_i, |d| {
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
            field_i += 1;
            
            // primordial
            let primordial = try!(d.read_struct_field("primordial", field_i, |d| Decodable::decode(d)));
            field_i += 1;

            // is_running
            let is_running = try!(d.read_struct_field("is_running", field_i, |d| Decodable::decode(d)));
            field_i += 1;

            // vm_options
            let vm_options = try!(d.read_struct_field("vm_options", field_i, |d| Decodable::decode(d)));
            field_i += 1;
            
            // compiled funcs
            let compiled_funcs = try!(d.read_struct_field("compiled_funcs", field_i, |d| {
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
            field_i += 1;
            
            let vm = VM{
                next_id: ATOMIC_USIZE_INIT,
                id_name_map: RwLock::new(id_name_map),
                name_id_map: RwLock::new(name_id_map),
                types: RwLock::new(types),
                backend_type_info: RwLock::new(backend_type_info),
                constants: RwLock::new(constants),
                globals: RwLock::new(globals),
                global_locations: RwLock::new(hashmap!{}),
                func_sigs: RwLock::new(func_sigs),
                funcs: RwLock::new(funcs),
                func_vers: RwLock::new(func_vers),
                primordial: RwLock::new(primordial),
                is_running: ATOMIC_BOOL_INIT,
                vm_options: vm_options,
                compiled_funcs: RwLock::new(compiled_funcs)
            };
            
            vm.next_id.store(next_id, Ordering::SeqCst);
            vm.is_running.store(is_running, Ordering::SeqCst);
            
            Ok(vm)
        })
    }
}

macro_rules! gen_handle_int {
    ($fn_from: ident, $fn_to: ident, $int_ty: ty) => {
        pub fn $fn_from (&self, num: $int_ty, len: BitSize) -> APIHandleResult {
            let handle_id = self.next_id();
            self.new_handle (APIHandle {
                id: handle_id,
                v: APIHandleValue::Int(num as u64, len)
            })
        }

        pub fn $fn_to (&self, handle: APIHandleArg) -> $int_ty {
            handle.v.as_int() as $int_ty
        }
    }
}

impl <'a> VM {
    pub fn new() -> VM {
        VM::new_internal(VMOptions::default())
    }

    pub fn new_with_opts(str: &str) -> VM {
        VM::new_internal(VMOptions::init(str))
    }

    fn new_internal(options: VMOptions) -> VM {
        VM::start_logging(options.flag_log_level);

        let ret = VM {
            next_id: ATOMIC_USIZE_INIT,
            is_running: ATOMIC_BOOL_INIT,
            vm_options: options,

            id_name_map: RwLock::new(HashMap::new()),
            name_id_map: RwLock::new(HashMap::new()),

            constants: RwLock::new(HashMap::new()),

            types: RwLock::new(HashMap::new()),
            backend_type_info: RwLock::new(HashMap::new()),

            globals: RwLock::new(HashMap::new()),
            global_locations: RwLock::new(hashmap!{}),

            func_sigs: RwLock::new(HashMap::new()),
            func_vers: RwLock::new(HashMap::new()),
            funcs: RwLock::new(HashMap::new()),
            compiled_funcs: RwLock::new(HashMap::new()),

            primordial: RwLock::new(None)
        };

        // insert all intenral types
        {
            let mut types = ret.types.write().unwrap();
            for ty in INTERNAL_TYPES.iter() {
                types.insert(ty.id(), ty.clone());
            }
        }

        ret.is_running.store(false, Ordering::SeqCst);

        // Does not need SeqCst.
        //
        // If VM creates Mu threads and Mu threads calls traps, the trap handler still "happens
        // after" the creation of the VM itself. Rust does not have a proper memory model, but this
        // is how C++ works.
        //
        // If the client needs to create client-level threads, however, the client should properly
        // synchronise at the time of inter-thread communication, rather than creation of the VM.
        ret.next_id.store(USER_ID_START, Ordering::Relaxed);

        // init types
        types::init_types();

        ret.init_runtime();

        ret
    }

    fn init_runtime(&self) {
        // init log
        VM::start_logging(self.vm_options.flag_log_level);

        // init gc
        {
            let ref options = self.vm_options;
            gc::gc_init(options.flag_gc_immixspace_size, options.flag_gc_lospace_size, options.flag_gc_nthreads);
        }
    }

    fn start_logging(level: MuLogLevel) {
        match level {
            MuLogLevel::None  => {},
            MuLogLevel::Error => VM::start_logging_internal(LogLevel::Error),
            MuLogLevel::Warn  => VM::start_logging_internal(LogLevel::Warn),
            MuLogLevel::Info  => VM::start_logging_internal(LogLevel::Info),
            MuLogLevel::Debug => VM::start_logging_internal(LogLevel::Debug),
            MuLogLevel::Trace => VM::start_logging_internal(LogLevel::Trace),
        }
    }

    pub fn start_logging_trace() {
        VM::start_logging_internal(LogLevel::Trace)
    }

    fn start_logging_internal(level: LogLevel) {
        use simple_logger;

        match simple_logger::init_with_level(level) {
            Ok(_) => {},
            Err(_) => {}
        }
    }
    
    pub fn resume_vm(serialized_vm: &str) -> VM {
        use rustc_serialize::json;
        
        let vm : VM = json::decode(serialized_vm).unwrap();
        
        vm.init_runtime();

        // restore gc types
        {
            let type_info_guard = vm.backend_type_info.read().unwrap();
            let mut type_info_vec: Vec<Box<BackendTypeInfo>> = type_info_guard.values().map(|x| x.clone()).collect();
            type_info_vec.sort_by(|a, b| a.gc_type.id.cmp(&b.gc_type.id));

            let mut expect_id = 0;
            for ty_info in type_info_vec.iter() {
                use runtime::mm;

                let ref gc_type = ty_info.gc_type;

                if gc_type.id != expect_id {
                    debug_assert!(expect_id < gc_type.id);

                    while expect_id < gc_type.id {
                        use runtime::mm::common::gctype::GCType;

                        mm::add_gc_type(GCType::new_noreftype(0, 8));
                        expect_id += 1;
                    }
                }

                // now expect_id == gc_type.id
                debug_assert!(expect_id == gc_type.id);

                mm::add_gc_type(gc_type.as_ref().clone());
                expect_id += 1;
            }
        }
        
        vm
    }
    
    pub fn next_id(&self) -> MuID {
        // This only needs to be atomic, and does not need to be a synchronisation operation. The
        // only requirement for IDs is that all IDs obtained from `next_id()` are different. So
        // `Ordering::Relaxed` is sufficient.
        self.next_id.fetch_add(1, Ordering::Relaxed)
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
    
    pub fn id_of_by_refstring(&self, name: &String) -> MuID {
        let map = self.name_id_map.read().unwrap();
        *map.get(name).unwrap()
    }
    
    pub fn id_of(&self, name: &str) -> MuID {
        self.id_of_by_refstring(&name.to_string())
    }
    
    pub fn name_of(&self, id: MuID) -> MuName {
        let map = self.id_name_map.read().unwrap();
        map.get(&id).unwrap().clone()
    }
    
    pub fn declare_const(&self, id: MuID, ty: P<MuType>, val: Constant) -> P<Value> {
        let mut constants = self.constants.write().unwrap();
        let ret = P(Value{hdr: MuEntityHeader::unnamed(id), ty: ty, v: Value_::Constant(val)});

        self.declare_const_internal(&mut constants, id, ret.clone());
        
        ret
    }

    fn declare_const_internal(&self, map: &mut RwLockWriteGuard<HashMap<MuID, P<Value>>>, id: MuID, val: P<Value>) {
        debug_assert!(!map.contains_key(&id));

        info!("declare const #{} = {}", id, val);
        map.insert(id, val);
    }
    
    pub fn get_const(&self, id: MuID) -> P<Value> {
        let const_lock = self.constants.read().unwrap();
        match const_lock.get(&id) {
            Some(ret) => ret.clone(),
            None => panic!("cannot find const #{}", id)
        }
    }

    pub fn get_const_nocheck(&self, id: MuID) -> Option<P<Value>> {
        let const_lock = self.constants.read().unwrap();
        match const_lock.get(&id) {
            Some(ret) => Some(ret.clone()),
            None => None
        }
    }

    #[cfg(feature = "aot")]
    pub fn allocate_const(&self, val: P<Value>) -> ValueLocation {
        let id = val.id();
        let name = match val.name() {
            Some(name) => format!("CONST_{}_{}", id, name),
            None => format!("CONST_{}", id)
        };

        ValueLocation::Relocatable(backend::RegGroup::GPR, name)
    }
    
    pub fn declare_global(&self, id: MuID, ty: P<MuType>) -> P<Value> {
        let global = P(Value{
            hdr: MuEntityHeader::unnamed(id),
            ty: self.declare_type(self.next_id(), MuType_::iref(ty.clone())),
            v: Value_::Global(ty)
        });
        
        let mut globals = self.globals.write().unwrap();
        let mut global_locs = self.global_locations.write().unwrap();

        self.declare_global_internal(&mut globals, &mut global_locs, id, global.clone());
        
        global
    }

    fn declare_global_internal(
        &self,
        globals: &mut RwLockWriteGuard<HashMap<MuID, P<Value>>>,
        global_locs: &mut RwLockWriteGuard<HashMap<MuID, ValueLocation>>,
        id: MuID, val: P<Value>
    ) {
        self.declare_global_internal_no_alloc(globals, id, val.clone());
        self.alloc_global(global_locs, id, val);
    }

    // when bulk declaring, we hold locks for everything, we cannot resolve backend type, and do alloc
    fn declare_global_internal_no_alloc(
        &self,
        globals: &mut RwLockWriteGuard<HashMap<MuID, P<Value>>>,
        id: MuID, val: P<Value>
    ) {
        debug_assert!(!globals.contains_key(&id));

        info!("declare global #{} = {}", id, val);
        globals.insert(id, val.clone());
    }

    fn alloc_global(
        &self,
        global_locs: &mut RwLockWriteGuard<HashMap<MuID, ValueLocation>>,
        id: MuID, val: P<Value>
    ) {
        let backend_ty = self.get_backend_type_info(val.ty.get_referenced_ty().unwrap().id());
        let loc = gc::allocate_global(val, backend_ty);
        info!("allocate global #{} as {}", id, loc);
        global_locs.insert(id, loc);
    }
    
    pub fn declare_type(&self, id: MuID, ty: MuType_) -> P<MuType> {
        let ty = P(MuType{hdr: MuEntityHeader::unnamed(id), v: ty});
        
        let mut types = self.types.write().unwrap();

        self.declare_type_internal(&mut types, id, ty.clone());
        
        ty
    }

    fn declare_type_internal(&self, types: &mut RwLockWriteGuard<HashMap<MuID, P<MuType>>>, id: MuID, ty: P<MuType>) {
        debug_assert!(!types.contains_key(&id));

        info!("declare type #{} = {}", id, ty);
        types.insert(id, ty.clone());
    }
    
    pub fn get_type(&self, id: MuID) -> P<MuType> {
        let type_lock = self.types.read().unwrap();
        match type_lock.get(&id) {
            Some(ret) => ret.clone(),
            None => panic!("cannot find type #{}", id)
        }
    }    
    
    pub fn declare_func_sig(&self, id: MuID, ret_tys: Vec<P<MuType>>, arg_tys: Vec<P<MuType>>) -> P<MuFuncSig> {
        let ret = P(MuFuncSig{hdr: MuEntityHeader::unnamed(id), ret_tys: ret_tys, arg_tys: arg_tys});

        let mut func_sigs = self.func_sigs.write().unwrap();
        self.declare_func_sig_internal(&mut func_sigs, id, ret.clone());
        
        ret
    }

    fn declare_func_sig_internal(&self, sigs: &mut RwLockWriteGuard<HashMap<MuID, P<MuFuncSig>>>, id: MuID, sig: P<MuFuncSig>) {
        debug_assert!(!sigs.contains_key(&id));

        info!("declare func sig #{} = {}", id, sig);
        sigs.insert(id, sig);
    }
    
    pub fn get_func_sig(&self, id: MuID) -> P<MuFuncSig> {
        let func_sig_lock = self.func_sigs.read().unwrap();
        match func_sig_lock.get(&id) {
            Some(ret) => ret.clone(),
            None => panic!("cannot find func sig #{}", id)
        }
    }
    
    pub fn declare_func (&self, func: MuFunction) {
        let mut funcs = self.funcs.write().unwrap();

        self.declare_func_internal(&mut funcs, func.id(), func);
    }

    fn declare_func_internal(&self, funcs: &mut RwLockWriteGuard<HashMap<MuID, RwLock<MuFunction>>>, id: MuID, func: MuFunction) {
        debug_assert!(!funcs.contains_key(&id));

        info!("declare func #{} = {}", id, func);
        funcs.insert(id, RwLock::new(func));
    }
    
    /// The IR builder needs to look-up the function signature from the existing function ID.
    pub fn get_func_sig_for_func(&self, id: MuID) -> P<MuFuncSig> {
        let funcs_lock = self.funcs.read().unwrap();
        match funcs_lock.get(&id) {
            Some(func) => func.read().unwrap().sig.clone(),
            None => panic!("cannot find Mu function #{}", id)
        }
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

    /// Add a new bundle into VM.
    ///
    /// This function will drain the contents of all arguments.
    ///
    /// Ideally, this function should happen atomically. e.g. The client should not see a new type
    /// added without also seeing a new function added.
    pub fn declare_many(&self,
                        new_id_name_map: &mut HashMap<MuID, MuName>,
                        new_types: &mut HashMap<MuID, P<MuType>>,
                        new_func_sigs: &mut HashMap<MuID, P<MuFuncSig>>,
                        new_constants: &mut HashMap<MuID, P<Value>>,
                        new_globals: &mut HashMap<MuID, P<Value>>,
                        new_funcs: &mut HashMap<MuID, Box<MuFunction>>,
                        new_func_vers: &mut HashMap<MuID, Box<MuFunctionVersion>>,
                        arc_vm: Arc<VM>
                        ) {
        // Make sure other components, if ever acquiring multiple locks at the same time, acquire
        // them in this order, to prevent deadlock.
        {
            let mut id_name_map = self.id_name_map.write().unwrap();
            let mut name_id_map = self.name_id_map.write().unwrap();
            let mut types = self.types.write().unwrap();
            let mut constants = self.constants.write().unwrap();
            let mut globals = self.globals.write().unwrap();
            let mut func_sigs = self.func_sigs.write().unwrap();
            let mut funcs = self.funcs.write().unwrap();
            let mut func_vers = self.func_vers.write().unwrap();

            for (id, name) in new_id_name_map.drain() {
                id_name_map.insert(id, name.clone());
                name_id_map.insert(name, id);
            }

            for (id, obj) in new_types.drain() {
                self.declare_type_internal(&mut types, id, obj);
            }

            for (id, obj) in new_constants.drain() {
                self.declare_const_internal(&mut constants, id, obj);
            }

            for (id, obj) in new_globals.drain() {
                // we bulk allocate later (since we are holding all the locks, we cannot find ty info)
                self.declare_global_internal_no_alloc(&mut globals, id, obj);
            }

            for (id, obj) in new_func_sigs.drain() {
                self.declare_func_sig_internal(&mut func_sigs, id, obj);
            }

            for (id, obj) in new_funcs.drain() {
                self.declare_func_internal(&mut funcs, id, *obj);
            }

            for (id, obj) in new_func_vers.drain() {
                let func_id = obj.func_id;
                func_vers.insert(id, RwLock::new(*obj));

                {
                    trace!("Adding funcver {} as a version of {}...", id, func_id);
                    let func = funcs.get_mut(&func_id).unwrap();
                    func.write().unwrap().new_version(id);
                    trace!("Added funcver {} as a version of {} {:?}.", id, func_id, func);
                }
            }
        }
        // Locks released here

        // allocate all the globals defined
        {
            let globals = self.globals.read().unwrap();
            let mut global_locs = self.global_locations.write().unwrap();

            // make sure current thread has allocator
            let created = unsafe {MuThread::current_thread_as_mu_thread(Address::zero(), arc_vm.clone())};

            for (id, global) in globals.iter() {
                self.alloc_global(&mut global_locs, *id, global.clone());
            }

            if created {
                unsafe {MuThread::cleanup_current_mu_thread()};
            }
        }
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
        let ty = match types.get(&tyid) {
            Some(ty) => ty,
            None => panic!("invalid type id during get_backend_type_info(): {}", tyid)
        };
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

    pub fn get_cur_version_of(&self, fid: MuID) -> Option<MuID> {
        let funcs_guard = self.funcs.read().unwrap();
        match funcs_guard.get(&fid) {
            Some(rwlock_func) => {
                let func_guard = rwlock_func.read().unwrap();
                func_guard.cur_ver
            },
            None => None
        }
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
    
    pub fn make_primordial_thread(&self, func_id: MuID, has_const_args: bool, args: Vec<Constant>) {
        let mut guard = self.primordial.write().unwrap();
        *guard = Some(MuPrimordialThread{func_id: func_id, has_const_args: has_const_args, args: args});
    }
    
    #[allow(unused_variables)]
    pub fn make_boot_image(&mut self,
                           whitelist: Vec<MuID>,
                           primordial_func: Option<&APIHandle>, primordial_stack: Option<&APIHandle>,
                           primordial_threadlocal: Option<&APIHandle>,
                           sym_fields: Vec<&APIHandle>, sym_strings: Vec<String>,
                           reloc_fields: Vec<&APIHandle>, reloc_strings: Vec<String>,
                           output_file: String) {
        use rustc_serialize::json;

        let compiler = Compiler::new(CompilerPolicy::default(), self);
        let funcs = self.funcs().write().unwrap();
        let func_vers = self.func_vers().write().unwrap();

        // make sure all functions in whitelist are compiled
        for &id in whitelist.iter() {
            if let Some(f) = funcs.get(&id) {
                let f : &MuFunction = &f.read().unwrap();
                match f.cur_ver {
                    Some(fv_id) => {
                        let mut func_ver = func_vers.get(&fv_id).unwrap().write().unwrap();

                        if !func_ver.is_compiled() {
                            compiler.compile(&mut func_ver);
                        }
                    }
                    None => panic!("whitelist function {} has no version defined", f)
                }
            }
        }

        // make sure only one of primordial_func or primoridial_stack is set
        assert!(
            (primordial_func.is_some() && primordial_stack.is_none())
            || (primordial_func.is_none() && primordial_stack.is_some())
        );
        
        let serialized = json::encode(self).unwrap();
        
        unimplemented!() 
    }

    // -- API ---
    fn new_handle(&self, handle: APIHandle) -> APIHandleResult {
        let ret = Box::new(handle);

        ret
    }

    pub fn new_fixed(&self, tyid: MuID) -> APIHandleResult {
        let ty = self.get_type(tyid);

        let backend_ty = self.get_backend_type_info(tyid);
        let addr = gc::allocate_fixed(ty.clone(), backend_ty);
        trace!("API: allocated fixed type {} at {}", ty, addr);

        self.new_handle(APIHandle {
            id: self.next_id(),
            v : APIHandleValue::Ref(ty, addr)
        })
    }

    pub fn new_hybrid(&self, tyid: MuID, length: APIHandleArg) -> APIHandleResult {
        let ty  = self.get_type(tyid);
        let len = self.handle_to_uint64(length);

        let backend_ty = self.get_backend_type_info(tyid);
        let addr = gc::allocate_hybrid(ty.clone(), len, backend_ty);
        trace!("API: allocated hybrid type {} of length {} at {}", ty, len, addr);

        self.new_handle(APIHandle {
            id: self.next_id(),
            v: APIHandleValue::Ref(ty, addr)
        })
    }

    pub fn handle_get_iref(&self, handle_ref: APIHandleArg) -> APIHandleResult {
        let (ty, addr) = handle_ref.v.as_ref();

        /// FIXME: iref/ref share the same address - this actually depends on GC
        // iref has the same address as ref

        trace!("API: get iref from {:?}", handle_ref);
        trace!("API: result {} {:?}", ty, addr);

        self.new_handle(APIHandle {
            id: self.next_id(),
            v : APIHandleValue::IRef(ty, addr)
        })
    }

    pub fn handle_shift_iref(&self, handle_iref: APIHandleArg, offset: APIHandleArg) -> APIHandleResult {
        let (ty, addr) = handle_iref.v.as_iref();
        let offset = self.handle_to_uint64(offset);

        let offset_addr = {
            let backend_ty = self.get_backend_type_info(ty.id());
            addr.plus(backend_ty.size * (offset as usize))
        };

        self.new_handle(APIHandle {
            id: self.next_id(),
            v : APIHandleValue::IRef(ty, offset_addr)
        })
    }

    pub fn handle_get_elem_iref(&self, handle_iref: APIHandleArg, index: APIHandleArg) -> APIHandleResult {
        let (ty, addr) = handle_iref.v.as_iref();
        let index = self.handle_to_uint64(index);

        let ele_ty = match ty.get_elem_ty() {
            Some(ty) => ty,
            None => panic!("cannot get element ty from {}", ty)
        };
        let elem_addr = {
            let backend_ty = self.get_backend_type_info(ele_ty.id());
            addr.plus(backend_ty.size * (index as usize))
        };

        self.new_handle(APIHandle {
            id: self.next_id(),
            v : APIHandleValue::IRef(ele_ty, elem_addr)
        })
    }

    pub fn handle_get_var_part_iref(&self, handle_iref: APIHandleArg) -> APIHandleResult {
        let (ty, addr) = handle_iref.v.as_iref();

        let varpart_addr = {
            let backend_ty = self.get_backend_type_info(ty.id());
            addr.plus(backend_ty.size)
        };

        let varpart_ty = match ty.get_hybrid_varpart_ty() {
            Some(ty) => ty,
            None => panic!("cannot get varpart ty from {}", ty)
        };

        self.new_handle(APIHandle {
            id: self.next_id(),
            v : APIHandleValue::IRef(varpart_ty, varpart_addr)
        })
    }

    pub fn handle_get_field_iref(&self, handle_iref: APIHandleArg, field: usize) -> APIHandleResult {
        let (ty, addr) = handle_iref.v.as_iref();

        let field_ty = match ty.get_field_ty(field) {
            Some(ty) => ty,
            None => panic!("ty is not struct ty: {}", ty)
        };

        let field_addr = {
            let backend_ty = self.get_backend_type_info(ty.id());
            let field_offset = backend_ty.get_field_offset(field);
            addr.plus(field_offset)
        };

        self.new_handle(APIHandle {
            id: self.next_id(),
            v : APIHandleValue::IRef(field_ty, field_addr)
        })
    }

    pub fn handle_load(&self, ord: MemoryOrder, loc: APIHandleArg) -> APIHandleResult {
        let (ty, addr) = loc.v.as_iref();

        let handle_id = self.next_id();
        let handle_value = {
            match ty.v {
                MuType_::Int(len)     => APIHandleValue::Int(unsafe {addr.load::<u64>()}, len),
                MuType_::Float        => APIHandleValue::Float(unsafe {addr.load::<f32>()}),
                MuType_::Double       => APIHandleValue::Double(unsafe {addr.load::<f64>()}),
                MuType_::Ref(ref ty)  => APIHandleValue::Ref(ty.clone(), unsafe {addr.load::<Address>()}),
                MuType_::IRef(ref ty) => APIHandleValue::IRef(ty.clone(), unsafe {addr.load::<Address>()}),
                MuType_::UPtr(ref ty) => APIHandleValue::UPtr(ty.clone(), unsafe {addr.load::<Address>()}),

                _ => unimplemented!()
            }
        };

        self.new_handle(APIHandle {
            id: handle_id,
            v : handle_value
        })
    }

    pub fn handle_store(&self, ord: MemoryOrder, loc: APIHandleArg, val: APIHandleArg) {
        // FIXME: take memory order into consideration

        // get address
        let (_, addr) = loc.v.as_iref();

        // get value and store
        // we will store here (its unsafe)
        unsafe {
            match val.v {
                APIHandleValue::Int(ival, bits) => {
                    match bits {
                        8 => addr.store::<u8>(ival as u8),
                        16 => addr.store::<u16>(ival as u16),
                        32 => addr.store::<u32>(ival as u32),
                        64 => addr.store::<u64>(ival),
                        _ => panic!("unimplemented int length")
                    }
                },
                APIHandleValue::Float(fval) => addr.store::<f32>(fval),
                APIHandleValue::Double(fval) => addr.store::<f64>(fval),
                APIHandleValue::UPtr(_, aval) => addr.store::<Address>(aval),
                APIHandleValue::UFP(_, aval) => addr.store::<Address>(aval),

                APIHandleValue::Struct(_)
                | APIHandleValue::Array(_)
                | APIHandleValue::Vector(_) => panic!("cannot store an aggregated value to an address"),

                APIHandleValue::Ref(_, aval)
                | APIHandleValue::IRef(_, aval) => addr.store::<Address>(aval),

                _ => unimplemented!()
            }
        }
    }

    // this function and the following two make assumption that GC will not move object
    // they need to be reimplemented if we have a moving GC
    pub fn handle_pin_object(&self, loc: APIHandleArg) -> APIHandleResult {
        assert!(!gc::GC_MOVES_OBJECT);
        // gc will not move, so we just put ref into uptr

        let (ty, addr) = loc.v.as_ref_or_iref();
        self.new_handle(APIHandle {
            id: self.next_id(),
            v : APIHandleValue::UPtr(ty, addr)
        })
    }

    #[allow(unused_variables)]
    pub fn handle_unpin_object(&self, loc: APIHandleArg) {
        assert!(!gc::GC_MOVES_OBJECT);
        // gc will not move, do no need to unpin
        // do nothing
    }

    pub fn handle_get_addr(&self, loc: APIHandleArg) -> APIHandleResult {
        assert!(!gc::GC_MOVES_OBJECT);
        // loc needs to be already pinned - we don't check since we don't pin

        let (ty, addr) = loc.v.as_ref_or_iref();
        self.new_handle(APIHandle {
            id: self.next_id(),
            v : APIHandleValue::UPtr(ty, addr)
        })
    }

    pub fn handle_from_global(&self, id: MuID) -> APIHandleResult {
        let global_iref = {
            let global_locs = self.global_locations.read().unwrap();
            global_locs.get(&id).unwrap().to_address()
        };

        let global_inner_ty = {
            let global_lock = self.globals.read().unwrap();
            global_lock.get(&id).unwrap().ty.get_referenced_ty().unwrap()
        };

        let handle_id = self.next_id();

        self.new_handle(APIHandle {
            id: handle_id,
            v : APIHandleValue::IRef(global_inner_ty, global_iref)
        })
    }

    pub fn handle_from_const(&self, id: MuID) -> APIHandleResult {
        let constant = {
            let lock = self.constants.read().unwrap();
            lock.get(&id).unwrap().clone()
        };

        let ref const_ty = constant.ty;

        let handle_id = self.next_id();
        let handle = match constant.v {
            Value_::Constant(Constant::Int(val)) => {
                let len = match const_ty.get_int_length() {
                    Some(len) => len,
                    None => panic!("expected ty to be Int for a Constant::Int, found {}", const_ty)
                };

                APIHandle {
                    id: handle_id,
                    v : APIHandleValue::Int(val, len)
                }
            }
            Value_::Constant(Constant::Float(val)) => {
                APIHandle {
                    id: handle_id,
                    v : APIHandleValue::Float(val)
                }
            }
            Value_::Constant(Constant::Double(val)) => {
                APIHandle {
                    id: handle_id,
                    v : APIHandleValue::Double(val)
                }
            }
            Value_::Constant(Constant::FuncRef(_)) => {
                unimplemented!()
            }
            Value_::Constant(Constant::NullRef) => {
                APIHandle {
                    id: handle_id,
                    v : APIHandleValue::Ref(types::VOID_TYPE.clone(), unsafe {Address::zero()})
                }
            }
            _ => unimplemented!()
        };

        self.new_handle(handle)
    }

    gen_handle_int!(handle_from_uint64, handle_to_uint64, u64);
    gen_handle_int!(handle_from_uint32, handle_to_uint32, u32);
    gen_handle_int!(handle_from_uint16, handle_to_uint16, u16);
    gen_handle_int!(handle_from_uint8 , handle_to_uint8 , u8 );
    gen_handle_int!(handle_from_sint64, handle_to_sint64, i64);
    gen_handle_int!(handle_from_sint32, handle_to_sint32, i32);
    gen_handle_int!(handle_from_sint16, handle_to_sint16, i16);
    gen_handle_int!(handle_from_sint8 , handle_to_sint8 , i8 );

    pub fn handle_from_float(&self, num: f32) -> APIHandleResult {
        let handle_id = self.next_id();
        self.new_handle (APIHandle {
            id: handle_id,
            v: APIHandleValue::Float(num)
        })
    }

    pub fn handle_to_float (&self, handle: APIHandleArg) -> f32 {
        handle.v.as_float()
    }

    pub fn handle_from_double(&self, num: f64) -> APIHandleResult {
        let handle_id = self.next_id();
        self.new_handle (APIHandle {
            id: handle_id,
            v: APIHandleValue::Double(num)
        })
    }

    pub fn handle_to_double (&self, handle: APIHandleArg) -> f64 {
        handle.v.as_double()
    }

    pub fn handle_from_uptr(&self, tyid: MuID, ptr: Address) -> APIHandleResult {
        let ty = self.get_type(tyid);

        let handle_id = self.next_id();
        self.new_handle (APIHandle {
            id: handle_id,
            v : APIHandleValue::UPtr(ty, ptr)
        })
    }

    pub fn handle_to_uptr(&self, handle: APIHandleArg) -> Address {
        handle.v.as_uptr().1
    }

    pub fn handle_from_ufp(&self, tyid: MuID, ptr: Address) -> APIHandleResult {
        let ty = self.get_type(tyid);

        let handle_id = self.next_id();
        self.new_handle (APIHandle {
            id: handle_id,
            v : APIHandleValue::UFP(ty, ptr)
        })
    }

    pub fn handle_to_ufp(&self, handle: APIHandleArg) -> Address {
        handle.v.as_ufp().1
    }
}