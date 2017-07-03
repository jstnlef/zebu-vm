// Copyright 2017 The Australian National University
// 
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// 
//     http://www.apache.org/licenses/LICENSE-2.0
// 
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::HashMap;

use ast::ptr::*;
use ast::ir::*;
use ast::inst::*;
use ast::types;
use ast::types::*;
use compiler::{Compiler, CompilerPolicy};
use compiler::backend;
use compiler::backend::BackendType;
use compiler::machine_code::CompiledFunction;

use runtime::thread::*;
use runtime::*;
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
use std::sync::RwLock;
use std::sync::RwLockWriteGuard;
use std::sync::atomic::{AtomicUsize, AtomicBool, ATOMIC_BOOL_INIT, ATOMIC_USIZE_INIT, Ordering};

// FIXME:
// besides fields in VM, there are some 'globals' we need to persist
// such as STRUCT_TAG_MAP
// possibly INTERNAL_ID in ir.rs, internal types, etc

/// The VM struct. This stores metadata for the currently running Zebu instance.
/// This struct gets persisted in the boot image, and when the boot image is loaded,
/// everything should be back to the same status as before persisting.
/// This struct is usually used as Arc<VM> so it can be shared among threads. The
/// Arc<VM> is stored in every thread local of a Mu thread, so that they can refer
/// to the VM easily.
/// We are using fine-grained lock on VM to allow mutability on different fields in VM.
/// Also we use two-level locks for some data structures such as MuFunction/
/// MuFunctionVersion/CompiledFunction so that we can mutate on two
/// different functions/funcvers/etc at the same time.
//  FIXME: However, there are problems with this design, and we will need to rethink.
//  See Issue #2.
//  FIXME: besides fields in VM, there are some 'globals' we need to persist
//  such as STRUCT_TAG_MAP, INTERNAL_ID and internal types from ir crate. The point is
//  ir crate should be independent and self-contained. But when persisting the 'world',
//  besides persisting VM struct (containing most of the 'world'), we also need to
//  specifically persist those globals.
pub struct VM {
    // ---serialize these fields---
    // 0
    next_id: AtomicUsize,
    // 1
    id_name_map: RwLock<HashMap<MuID, MuName>>,
    // 2
    name_id_map: RwLock<HashMap<MuName, MuID>>,
    // 3
    types: RwLock<HashMap<MuID, P<MuType>>>,
    // 4
    backend_type_info: RwLock<HashMap<MuID, Box<BackendType>>>,
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
    pub primordial: RwLock<Option<PrimordialThreadInfo>>,
    // 11
    is_running: AtomicBool,
    // 12
    pub vm_options: VMOptions,
    
    // ---partially serialize---
    // 13
    compiled_funcs: RwLock<HashMap<MuID, RwLock<CompiledFunction>>>,

    // Maps each callsite to a tuple of the corresponding catch blocks label (or ""_
    // and the id of the containing function-version
    // 14
    exception_table: RwLock<HashMap<MuID, HashMap<MuName, MuName>>>,

    // ---do not serialize---

    // client may try to store funcref to the heap, so that they can load it later, and call it
    // however the store may happen before we have an actual address to the func (in AOT scenario)
    aot_pending_funcref_store: RwLock<HashMap<Address, ValueLocation>>,

    // Table for excetions
    // TODO: Have a table of these tables (one per bundle?)

    // The exception table (before it has been written to disk)


    // Same as above but once the everything have been resolved to addreses

    // TODO: What should the function version refer to? (It has to refer to something that has callee saved registers...)
    // TODO: probably we should remove the pointer (its unsafe), thats why we need Sync/Send for VM
    //       we can make a copy of callee_saved_register location
    pub compiled_exception_table: RwLock<HashMap<Address, (Address, *const CompiledFunction)>>
}
unsafe impl Sync for VM {}
unsafe impl Send for VM {}

use std::u64;
const PENDING_FUNCREF : u64 = u64::MAX;

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
            trace!("...serializing exception_table");
            {
                let map : &HashMap<MuID, HashMap<MuName, MuName>> = &self.exception_table.read().unwrap();
                try!(s.emit_struct_field("exception_table", field_i, |s| map.encode(s)));
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

            trace!("Deserialising exception table");
            let exception_table = try!(d.read_struct_field("exception_table", field_i, |d| Decodable::decode(d)));
            field_i += 1;


            let vm = VM {
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
                func_vers: RwLock::new(hashmap!{}),
                primordial: RwLock::new(primordial),
                is_running: ATOMIC_BOOL_INIT,
                vm_options: vm_options,
                compiled_funcs: RwLock::new(compiled_funcs),
                exception_table: RwLock::new(exception_table),
                aot_pending_funcref_store: RwLock::new(HashMap::new()),
                compiled_exception_table: RwLock::new(HashMap::new()),
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
            exception_table: RwLock::new(HashMap::new()),
            primordial: RwLock::new(None),

            aot_pending_funcref_store: RwLock::new(HashMap::new()),
            compiled_exception_table: RwLock::new(HashMap::new()),
        };

        // insert all internal types
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
            gc::gc_init(options.flag_gc_immixspace_size, options.flag_gc_lospace_size, options.flag_gc_nthreads, !options.flag_gc_disable_collection);
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
        use stderrlog;

        let verbose = match level {
            LogLevel::Error => 0,
            LogLevel::Warn  => 1,
            LogLevel::Info  => 2,
            LogLevel::Debug => 3,
            LogLevel::Trace => 4,
        };

        match stderrlog::new().verbosity(verbose).init() {
            Ok(()) => info!("logger initialized"),
            Err(e) => error!("failed to init logger, probably already initialized: {:?}", e)
        }
    }

    pub fn add_exception_callsite(&self, callsite: MuName, catch: MuName, fv: MuID) {
        let mut table = self.exception_table.write().unwrap();

        if table.contains_key(&fv) {
            let mut map = table.get_mut(&fv).unwrap();
            map.insert(callsite, catch);
        } else {
            let mut new_map = HashMap::new();
            new_map.insert(callsite, catch);
            table.insert(fv, new_map);
        };
    }

    pub fn resume_vm(serialized_vm: &str) -> VM {
        use rustc_serialize::json;
        
        let vm : VM = json::decode(serialized_vm).unwrap();
        
        vm.init_runtime();

        // restore gc types
        {
            let type_info_guard = vm.backend_type_info.read().unwrap();
            let mut type_info_vec: Vec<Box<BackendType>> = type_info_guard.values().map(|x| x.clone()).collect();
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

        // construct exception table
        vm.build_exception_table();

        vm
    }

    pub fn build_exception_table(&self) {
        let exception_table = self.exception_table.read().unwrap();
        let compiled_funcs = self.compiled_funcs.read().unwrap();
        let mut compiled_exception_table = self.compiled_exception_table.write().unwrap();

        for (fv, map) in exception_table.iter() {
            let ref compiled_func = *compiled_funcs.get(fv).unwrap().read().unwrap();

            for (callsite, catch_block) in map.iter() {
                let catch_addr = if catch_block.is_empty() {
                    unsafe {Address::zero()}
                } else {
                    resolve_symbol(catch_block.clone())
                };

                compiled_exception_table.insert(resolve_symbol(callsite.clone()), (catch_addr, &*compiled_func));
            }
        }
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
    
    pub fn set_name(&self, entity: &MuEntity) {
        let id = entity.id();
        let name = entity.name().unwrap();
        
        let mut map = self.id_name_map.write().unwrap();
        map.insert(id, name.clone());
        
        let mut map2 = self.name_id_map.write().unwrap();
        map2.insert(name, id);
    }
    
    pub fn id_of_by_refstring(&self, name: &String) -> MuID {
        let map = self.name_id_map.read().unwrap();
        match map.get(name) {
            Some(id) => *id,
            None => panic!("cannot find id for name: {}", name)
        }
    }

    /// should only used by client
    /// 'name' used internally may be slightly different to remove some special symbols
    pub fn id_of(&self, name: &str) -> MuID {
        self.id_of_by_refstring(&name.to_string())
    }

    /// should only used by client
    /// 'name' used internally may be slightly different to remove some special symbols
    pub fn name_of(&self, id: MuID) -> MuName {
        let map = self.id_name_map.read().unwrap();
        map.get(&id).unwrap().clone()
    }
    
    pub fn declare_const(&self, entity: MuEntityHeader, ty: P<MuType>, val: Constant) -> P<Value> {
        let mut constants = self.constants.write().unwrap();
        let ret = P(Value{hdr: entity, ty: ty, v: Value_::Constant(val)});

        self.declare_const_internal(&mut constants, ret.id(), ret.clone());
        
        ret
    }

    fn declare_const_internal(&self, map: &mut RwLockWriteGuard<HashMap<MuID, P<Value>>>, id: MuID, val: P<Value>) {
        debug_assert!(!map.contains_key(&id));

        trace!("declare const #{} = {}", id, val);
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
    
    pub fn declare_global(&self, entity: MuEntityHeader, ty: P<MuType>) -> P<Value> {
        let global = P(Value{
            hdr: entity,
            ty: self.declare_type(MuEntityHeader::unnamed(self.next_id()), MuType_::iref(ty.clone())),
            v: Value_::Global(ty)
        });
        
        let mut globals = self.globals.write().unwrap();
        let mut global_locs = self.global_locations.write().unwrap();

        self.declare_global_internal(&mut globals, &mut global_locs, global.id(), global.clone());
        
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
        let backend_ty = self.get_backend_type_info(val.ty.get_referent_ty().unwrap().id());
        let loc = gc::allocate_global(val, backend_ty);
        info!("allocate global #{} as {}", id, loc);
        global_locs.insert(id, loc);
    }
    
    pub fn declare_type(&self, entity: MuEntityHeader, ty: MuType_) -> P<MuType> {
        let ty = P(MuType{hdr: entity, v: ty});
        
        let mut types = self.types.write().unwrap();

        self.declare_type_internal(&mut types, ty.id(), ty.clone());
        
        ty
    }

    fn declare_type_internal(&self, types: &mut RwLockWriteGuard<HashMap<MuID, P<MuType>>>, id: MuID, ty: P<MuType>) {
        debug_assert!(!types.contains_key(&id));

        types.insert(id, ty.clone());

        trace!("declare type #{} = {}", id, ty);
        if ty.is_struct() {
            let tag = ty.get_struct_hybrid_tag().unwrap();
            let struct_map_guard = STRUCT_TAG_MAP.read().unwrap();
            let struct_inner = struct_map_guard.get(&tag).unwrap();
            trace!("  {}", struct_inner);
        } else if ty.is_hybrid() {
            let tag = ty.get_struct_hybrid_tag().unwrap();
            let hybrid_map_guard = HYBRID_TAG_MAP.read().unwrap();
            let hybrid_inner = hybrid_map_guard.get(&tag).unwrap();
            trace!("  {}", hybrid_inner);
        }
    }
    
    pub fn get_type(&self, id: MuID) -> P<MuType> {
        let type_lock = self.types.read().unwrap();
        match type_lock.get(&id) {
            Some(ret) => ret.clone(),
            None => panic!("cannot find type #{}", id)
        }
    }    
    
    pub fn declare_func_sig(&self, entity: MuEntityHeader, ret_tys: Vec<P<MuType>>, arg_tys: Vec<P<MuType>>) -> P<MuFuncSig> {
        let ret = P(MuFuncSig{hdr: entity, ret_tys: ret_tys, arg_tys: arg_tys});

        let mut func_sigs = self.func_sigs.write().unwrap();
        self.declare_func_sig_internal(&mut func_sigs, ret.id(), ret.clone());
        
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

    /// this is different than vm.name_of()
    pub fn get_func_name(&self, id: MuID) -> MuName {
        let funcs_lock = self.funcs.read().unwrap();
        match funcs_lock.get(&id) {
            Some(func) => func.read().unwrap().name().unwrap(),
            None => panic!("cannot find name for Mu function #{}")
        }
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
    
    pub fn get_backend_type_info(&self, tyid: MuID) -> Box<BackendType> {
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
        let resolved = Box::new(backend::BackendType::resolve(ty, self));
        
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
    
    pub fn set_primordial_thread(&self, func_id: MuID, has_const_args: bool, args: Vec<Constant>) {
        let mut guard = self.primordial.write().unwrap();
        *guard = Some(PrimordialThreadInfo {func_id: func_id, has_const_args: has_const_args, args: args});
    }

    pub fn make_boot_image(&self,
                            whitelist: Vec<MuID>,
                            primordial_func: Option<&APIHandle>, primordial_stack: Option<&APIHandle>,
                            primordial_threadlocal: Option<&APIHandle>,
                            sym_fields: Vec<&APIHandle>, sym_strings: Vec<String>,
                            reloc_fields: Vec<&APIHandle>, reloc_strings: Vec<String>,
                            output_file: String) {
        self.make_boot_image_internal(
            whitelist,
            primordial_func, primordial_stack,
            primordial_threadlocal,
            sym_fields, sym_strings,
            reloc_fields, reloc_strings,
            vec![],
            output_file
        )
    }
    
    #[allow(unused_variables)]
    pub fn make_boot_image_internal(&self,
                                   whitelist: Vec<MuID>,
                                   primordial_func: Option<&APIHandle>, primordial_stack: Option<&APIHandle>,
                                   primordial_threadlocal: Option<&APIHandle>,
                                   sym_fields: Vec<&APIHandle>, sym_strings: Vec<String>,
                                   reloc_fields: Vec<&APIHandle>, reloc_strings: Vec<String>,
                                   extra_sources_to_link: Vec<String>,
                                   output_file: String) {
        trace!("Making boot image...");

        let whitelist_funcs = {
            let compiler = Compiler::new(CompilerPolicy::default(), self);
            let funcs = self.funcs().read().unwrap();
            let func_vers = self.func_vers().read().unwrap();

            // make sure all functions in whitelist are compiled
            let mut whitelist_funcs: Vec<MuID> = vec![];
            for &id in whitelist.iter() {
                if let Some(f) = funcs.get(&id) {
                    whitelist_funcs.push(id);

                    let f: &MuFunction = &f.read().unwrap();
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

            whitelist_funcs
        };

        if primordial_threadlocal.is_some() {
            // we are going to need to persist this threadlocal
            unimplemented!()
        }

        // make sure only one of primordial_func or primoridial_stack is set
        let has_primordial_func  = primordial_func.is_some();
        let has_primordial_stack = primordial_stack.is_some();

        // we assume client will start with a function (instead of a stack)
        if has_primordial_stack {
            panic!("Zebu doesnt support creating primordial thread through a stack, name a entry function instead")
        } else {
            if has_primordial_func {
                // extract func id
                let func_id = primordial_func.unwrap().v.as_funcref();

                // make primordial thread in vm
                self.set_primordial_thread(func_id, false, vec![]);    // do not pass const args, use argc/argv
            } else {
                warn!("no entry function is passed");
            }

            // deal with relocation symbols
            assert_eq!(sym_fields.len(), sym_strings.len());
            let symbols = {
                let mut ret = hashmap!{};
                for i in 0..sym_fields.len() {
                    let addr = sym_fields[i].v.as_address();
                    ret.insert(addr, name_check(sym_strings[i].clone()));
                }
                ret
            };

            assert_eq!(reloc_fields.len(), reloc_strings.len());
            let fields = {
                let mut ret = hashmap!{};

                // client supplied relocation fields
                for i in 0..reloc_fields.len() {
                    let addr = reloc_fields[i].v.as_address();
                    ret.insert(addr, name_check(reloc_strings[i].clone()));
                }

                // pending funcrefs - we want to replace them as symbol
                {
                    let mut pending_funcref = self.aot_pending_funcref_store.write().unwrap();
                    for (addr, vl) in pending_funcref.drain() {
                        ret.insert(addr, name_check(vl.to_relocatable()));
                    }
                }

                ret
            };

            // emit context (serialized vm, etc)
            backend::emit_context_with_reloc(self, symbols, fields);

            // link
            self.link_boot_image(whitelist_funcs, extra_sources_to_link, output_file);
        }
    }

    #[cfg(feature = "aot")]
    fn link_boot_image(&self, funcs: Vec<MuID>, extra_srcs: Vec<String>, output_file: String) {
        use testutil;

        trace!("Linking boot image...");

        let func_names = {
            let funcs_guard = self.funcs().read().unwrap();
            funcs.iter().map(|x| funcs_guard.get(x).unwrap().read().unwrap().name().unwrap()).collect()
        };

        trace!("functions: {:?}", func_names);
        trace!("extern sources: {:?}", extra_srcs);
        trace!("output   : {}", output_file);

        if output_file.ends_with("dylib") || output_file.ends_with("so") {
            // compile as dynamic library
            testutil::aot::link_dylib_with_extra_srcs(func_names, extra_srcs, &output_file, self);
        } else {
            assert!(extra_srcs.len() == 0, "trying to create an executable with linking extern sources, unimplemented");
            // compile as executable
            testutil::aot::link_primordial(func_names, &output_file, self);
        }

        trace!("Done!");
    }

    // -- API ---
    fn new_handle(&self, handle: APIHandle) -> APIHandleResult {
        let ret = Box::new(handle);

        ret
    }

    pub fn new_fixed(&self, tyid: MuID) -> APIHandleResult {
        let ty = self.get_type(tyid);
        assert!(!ty.is_hybrid());

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
        assert!(ty.is_hybrid());

        let len = self.handle_to_uint64(length);

        let backend_ty = self.get_backend_type_info(tyid);
        let addr = gc::allocate_hybrid(ty.clone(), len, backend_ty);
        trace!("API: allocated hybrid type {} of length {} at {}", ty, len, addr);

        self.new_handle(APIHandle {
            id: self.next_id(),
            v: APIHandleValue::Ref(ty, addr)
        })
    }

    pub fn handle_refcast(&self, from_op: APIHandleArg, to_ty: MuID) -> APIHandleResult {
        let handle_id = self.next_id();
        let to_ty = self.get_type(to_ty);

        trace!("API: refcast {} into type {}", from_op, to_ty);

        match from_op.v {
            APIHandleValue::Ref(_, addr) => {
                assert!(to_ty.is_ref());
                let inner_ty = to_ty.get_referent_ty().unwrap();

                self.new_handle(APIHandle {
                    id: handle_id,
                    v: APIHandleValue::Ref(inner_ty, addr)
                })
            },
            APIHandleValue::IRef(_, addr) => {
                assert!(to_ty.is_iref());
                let inner_ty = to_ty.get_referent_ty().unwrap();

                self.new_handle(APIHandle {
                    id: handle_id,
                    v : APIHandleValue::IRef(inner_ty, addr)
                })
            },
            APIHandleValue::FuncRef(_) => unimplemented!(),

            _ => panic!("unexpected operand for refcast: {:?}", from_op)
        }
    }

    pub fn handle_get_iref(&self, handle_ref: APIHandleArg) -> APIHandleResult {
        let (ty, addr) = handle_ref.v.as_ref();

        /// FIXME: iref/ref share the same address - this actually depends on GC
        // iref has the same address as ref
        let ret = self.new_handle(APIHandle {
            id: self.next_id(),
            v : APIHandleValue::IRef(ty, addr)
        });

        trace!("API: get iref from {:?}", handle_ref);
        trace!("API: result {:?}", ret);

        ret
    }

    pub fn handle_shift_iref(&self, handle_iref: APIHandleArg, offset: APIHandleArg) -> APIHandleResult {
        let (ty, addr) = handle_iref.v.as_iref();
        let offset = self.handle_to_uint64(offset);

        let offset_addr = {
            let backend_ty = self.get_backend_type_info(ty.id());
            addr + (backend_ty.size * (offset as usize))
        };

        let ret = self.new_handle(APIHandle {
            id: self.next_id(),
            v : APIHandleValue::IRef(ty, offset_addr)
        });

        trace!("API: shift iref from {:?}", handle_iref);
        trace!("API: result {:?}", ret);

        ret
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
            addr + (backend_ty.size * (index as usize))
        };

        let ret = self.new_handle(APIHandle {
            id: self.next_id(),
            v : APIHandleValue::IRef(ele_ty, elem_addr)
        });

        trace!("API: get element iref from {:?} at index {:?}", handle_iref, index);
        trace!("API: result {:?}", ret);

        ret
    }

    pub fn handle_get_var_part_iref(&self, handle_iref: APIHandleArg) -> APIHandleResult {
        let (ty, addr) = handle_iref.v.as_iref();

        let varpart_addr = {
            let backend_ty = self.get_backend_type_info(ty.id());
            addr + backend_ty.size
        };

        let varpart_ty = match ty.get_hybrid_varpart_ty() {
            Some(ty) => ty,
            None => panic!("cannot get varpart ty from {}", ty)
        };

        let ret = self.new_handle(APIHandle {
            id: self.next_id(),
            v : APIHandleValue::IRef(varpart_ty, varpart_addr)
        });

        trace!("API: get var part iref from {:?}", handle_iref);
        trace!("API: result {:?}", ret);

        ret
    }

    pub fn handle_get_field_iref(&self, handle_iref: APIHandleArg, field: usize) -> APIHandleResult {
        trace!("API: get field iref from {:?}", handle_iref);

        let (ty, addr) = handle_iref.v.as_iref();

        let field_ty = match ty.get_field_ty(field) {
            Some(ty) => ty,
            None => panic!("ty is not struct ty: {}", ty)
        };

        let field_addr = {
            let backend_ty = self.get_backend_type_info(ty.id());
            let field_offset = backend_ty.get_field_offset(field);
            addr + field_offset
        };

        let ret = self.new_handle(APIHandle {
            id: self.next_id(),
            v : APIHandleValue::IRef(field_ty, field_addr)
        });

        trace!("API: get field iref from {:?}, field: {}", handle_iref, field);
        trace!("API: result {:?}", ret);

        ret
    }

    #[allow(unused_variables)]
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
                MuType_::Tagref64     => APIHandleValue::TagRef64(unsafe {addr.load::<u64>()}),

                _ => unimplemented!()
            }
        };

        let ret = self.new_handle(APIHandle {
            id: handle_id,
            v : handle_value
        });

        trace!("API: load from {:?}", loc);
        trace!("API: result {:?}", ret);

        ret
    }

    #[allow(unused_variables)]
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
                        1  => addr.store::<u8>((ival as u8) & 0b1u8),
                        6  => addr.store::<u8>((ival as u8) & 0b111111u8),
                        8  => addr.store::<u8>(ival as u8),
                        16 => addr.store::<u16>(ival as u16),
                        32 => addr.store::<u32>(ival as u32),
                        52 => addr.store::<u64>(ival & ((1 << 51)-1)),
                        64 => addr.store::<u64>(ival),
                        _  => panic!("unimplemented int length")
                    }
                },
                APIHandleValue::TagRef64(val) => addr.store::<u64>(val),
                APIHandleValue::Float(fval) => addr.store::<f32>(fval),
                APIHandleValue::Double(fval) => addr.store::<f64>(fval),
                APIHandleValue::UPtr(_, aval) => addr.store::<Address>(aval),
                APIHandleValue::UFP(_, aval) => addr.store::<Address>(aval),

                APIHandleValue::Struct(_)
                | APIHandleValue::Array(_)
                | APIHandleValue::Vector(_) => panic!("cannot store an aggregated value to an address"),

                APIHandleValue::Ref(_, aval)
                | APIHandleValue::IRef(_, aval) => addr.store::<Address>(aval),

                // if we are JITing, we can store the address of the function
                // but if we are doing AOT, we pend the store, and resolve the store when making boot image
                APIHandleValue::FuncRef(id) => self.store_funcref(addr, id),

                _ => panic!("unimplemented store for handle {}", val.v)
            }
        }

        trace!("API: store value {:?} to location {:?}", val, loc);
    }

    #[cfg(feature = "aot")]
    fn store_funcref(&self, addr: Address, func_id: MuID) {
        // put a pending funcref in the address
        unsafe {addr.store::<u64>(PENDING_FUNCREF)};

        // and record this funcref
        let symbol = self.name_of(func_id);

        let mut pending_funcref_guard = self.aot_pending_funcref_store.write().unwrap();
        pending_funcref_guard.insert(addr, ValueLocation::Relocatable(backend::RegGroup::GPR, symbol));
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

    pub fn handle_from_func(&self, id: MuID) -> APIHandleResult {
        let handle_id = self.next_id();

        self.new_handle(APIHandle {
            id: handle_id,
            v : APIHandleValue::FuncRef(id)
        })
    }

    pub fn handle_from_global(&self, id: MuID) -> APIHandleResult {
        let global_iref = {
            let global_locs = self.global_locations.read().unwrap();
            global_locs.get(&id).unwrap().to_address()
        };

        let global_inner_ty = {
            let global_lock = self.globals.read().unwrap();
            global_lock.get(&id).unwrap().ty.get_referent_ty().unwrap()
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
    
   /**
    * Functions for handling TagRef64-related API calls are taken from:
    * https://gitlab.anu.edu.au/mu/mu-impl-ref2/blob/master/src/main/scala/uvm/refimpl/itpr/operationHelpers.scala
    */
    
    // See: `tr64IsFP`
    pub fn handle_tr64_is_fp(&self, value:APIHandleArg) -> bool {
        let opnd = value.v.as_tr64();
        (opnd & 0x7ff0000000000001u64) != 0x7ff0000000000001u64 &&
           (opnd & 0x7ff0000000000003u64) != 0x7ff0000000000002u64
    }

    // See: `tr64IsInt`
    pub fn handle_tr64_is_int(&self, value: APIHandleArg) -> bool {
        let opnd = value.v.as_tr64();
        (opnd & 0x7ff0000000000001u64) == 0x7ff0000000000001u64
    }

    // See: `tr64IsRef`
    pub fn handle_tr64_is_ref(&self, value: APIHandleArg) -> bool {
        let opnd = value.v.as_tr64();
        (opnd & 0x7ff0000000000003u64) == 0x7ff0000000000002u64
    }
    
    // See: `tr64ToFP`
    pub fn handle_tr64_to_fp(&self, value: APIHandleArg) -> APIHandleResult {
        let handle_id = self.next_id();
        self.new_handle(APIHandle {
            id: handle_id,
            v : APIHandleValue::Double(
                value.v.as_tr64() as f64
            )
        })
    }

    // See: `tr64ToInt`
    pub fn handle_tr64_to_int(&self, value: APIHandleArg) -> APIHandleResult {
        let handle_id = self.next_id();
        let opnd = value.v.as_tr64();
        self.new_handle(APIHandle {
            id: handle_id,
            v : APIHandleValue::Int(
                (((opnd & 0xffffffffffffeu64) >> 1) | ((opnd & 0x8000000000000000u64) >> 12)),
                52
            )
        })
    }

    // See: `tr64ToRef`
    pub fn handle_tr64_to_ref(&self, value: APIHandleArg) -> APIHandleResult {
        let handle_id = self.next_id();
        let opnd = value.v.as_tr64();
        self.new_handle(APIHandle {
            id: handle_id,
            v : APIHandleValue::Ref(types::REF_VOID_TYPE.clone(),
                unsafe { Address::from_usize(
                        ((opnd & 0x7ffffffffff8u64) |
                               (((!(((opnd & 0x8000000000000000u64) << 1) - 1)) >> 17) &
                                    0xffff800000000000u64)) as usize
                ) })
        })
    }

    // See: `tr64ToTag`
    pub fn handle_tr64_to_tag(&self, value: APIHandleArg) -> APIHandleResult {
        let handle_id = self.next_id();
        let opnd = value.v.as_tr64();
        self.new_handle(APIHandle {
            id: handle_id,
            v : APIHandleValue::Int(
                    (((opnd & 0x000f800000000000u64) >> 46) | ((opnd & 0x4) >> 2)),
                6
            )
        })
    }

    // See: `fpToTr64`
    pub fn handle_tr64_from_fp(&self, value: APIHandleArg) -> APIHandleResult {
        let handle_id = self.next_id();
        let mut bits = value.v.as_double() as u64;
        if value.v.as_double().is_nan() {
            bits = bits & 0xfff8000000000000u64 | 0x0000000000000008u64;
        }
        self.new_handle(APIHandle {
            id: handle_id,
            v : APIHandleValue::TagRef64(bits)
        })
    }

    // See: `intToTr64`
    pub fn handle_tr64_from_int(&self, value: APIHandleArg) -> APIHandleResult {
        let handle_id = self.next_id();
        let opnd = value.v.as_int();
        self.new_handle(APIHandle {
            id: handle_id,
            v : APIHandleValue::TagRef64(
                (0x7ff0000000000001u64 | ((opnd & 0x7ffffffffffffu64) << 1) |
                    ((opnd & 0x8000000000000u64) << 12))
            )
        })
    }
    
    // See: `refToTr64`
    pub fn handle_tr64_from_ref(&self, reff: APIHandleArg, tag: APIHandleArg) -> APIHandleResult {
        let handle_id = self.next_id();
        let (_, addr) = reff.v.as_ref();
        let addr_ = addr.as_usize() as u64;
        let tag_  = tag.v.as_int();
        self.new_handle (APIHandle {
            id: handle_id,
            v : APIHandleValue::TagRef64( 
                (0x7ff0000000000002u64 | (addr_ & 0x7ffffffffff8u64) | ((addr_ & 0x800000000000u64) << 16)
                    | ((tag_ & 0x3eu64) << 46) | ((tag_ & 0x1) << 2))
            )
        })
    }
}
