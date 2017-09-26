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

use rodal;
use ast::ptr::*;
use ast::ir::*;
use ast::inst::*;
use ast::types;
use ast::types::*;
use compiler::{Compiler, CompilerPolicy};
use compiler::backend;
use compiler::backend::BackendType;
use compiler::machine_code::{CompiledFunction, CompiledCallsite};

use runtime::thread::*;
use runtime::*;
use utils::ByteSize;
use utils::BitSize;
use utils::Address;
use runtime::mm as gc;
use vm::handle::*;
use vm::vm_options::VMOptions;
use vm::vm_options::MuLogLevel;

use log::LogLevel;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::Mutex;
use std::sync::RwLockWriteGuard;
use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};
use std::thread::JoinHandle;
use std::collections::LinkedList;
use std;
use utils::bit_utils::{bits_ones, u64_asr};

/// The VM struct. This stores metadata for the currently running Zebu instance.
/// This struct gets persisted in the boot image, and when the boot image is loaded,
/// everything should be back to the same status as before persisting.
///
/// This struct is usually used as Arc<VM> so it can be shared among threads. The
/// Arc<VM> is stored in every thread local of a Mu thread, so that they can refer
/// to the VM easily.
///
/// We are using fine-grained lock on VM to allow mutability on different fields in VM.
/// Also we use two-level locks for some data structures such as MuFunction/
/// MuFunctionVersion/CompiledFunction so that we can mutate on two
/// different functions/funcvers/etc at the same time.

//  FIXME: However, there are problems with fine-grained lock design,
//  and we will need to rethink. See Issue #2.
//  TODO: besides fields in VM, there are some 'globals' we need to persist
//  such as STRUCT_TAG_MAP, INTERNAL_ID and internal types from ir crate. The point is
//  ir crate should be independent and self-contained. But when persisting the 'world',
//  besides persisting VM struct (containing most of the 'world'), we also need to
//  specifically persist those globals.
pub struct VM {
    // The comments are the offset into the struct
    // ---serialize---
    /// next MuID to assign
    next_id: AtomicUsize, // +0
    /// a map from MuID to MuName (for client to query)
    id_name_map: RwLock<HashMap<MuID, MuName>>, // +8
    /// a map from MuName to ID (for client to query)
    name_id_map: RwLock<HashMap<MuName, MuID>>, // +64
    /// types declared to the VM
    types: RwLock<HashMap<MuID, P<MuType>>>, // +120
    /// types that are resolved as BackendType
    backend_type_info: RwLock<HashMap<MuID, Box<BackendType>>>,
    /// constants declared to the VM
    constants: RwLock<HashMap<MuID, P<Value>>>,
    /// globals declared to the VM
    globals: RwLock<HashMap<MuID, P<Value>>>,
    /// function signatures declared
    func_sigs: RwLock<HashMap<MuID, P<MuFuncSig>>>,
    /// functions declared to the VM
    funcs: RwLock<HashMap<MuID, RwLock<MuFunction>>>,
    /// primordial function that is set to make boot image
    primordial: RwLock<Option<PrimordialThreadInfo>>,

    /// current options for this VM
    pub vm_options: VMOptions, // +624

    // ---partially serialize---
    /// compiled functions
    /// (we are not persisting generated code with compiled function)
    compiled_funcs: RwLock<HashMap<MuID, RwLock<CompiledFunction>>>, // +728

    /// match each functions version to a map, mapping each of it's containing callsites
    /// to the name of the catch block
    callsite_table: RwLock<HashMap<MuID, Vec<Callsite>>>, // +784

    // ---do not serialize---
    /// global cell locations. We use this map to create handles for global cells,
    /// or dump globals into boot image. (this map does not get persisted because
    /// the location is changed in different runs)
    global_locations: RwLock<HashMap<MuID, ValueLocation>>,
    func_vers: RwLock<HashMap<MuID, RwLock<MuFunctionVersion>>>,

    /// all the funcref that clients want to store for AOT which are pending stores
    /// For AOT scenario, when client tries to store funcref to the heap, the store
    /// happens before we have an actual address for the function so we store a fake
    /// funcref and when generating boot image, we fix the funcref with a relocatable symbol
    aot_pending_funcref_store: RwLock<HashMap<Address, ValueLocation>>,

    /// runtime callsite table for exception handling
    /// a map from callsite address to CompiledCallsite
    compiled_callsite_table: RwLock<HashMap<Address, CompiledCallsite>>, // 896

    /// Nnmber of callsites in the callsite tables
    callsite_count: AtomicUsize,

    /// A list of all threads currently waiting to be joined
    pub pending_joins: Mutex<LinkedList<JoinHandle<()>>>
}

rodal_named!(VM);
unsafe impl rodal::Dump for VM {
    fn dump<D: ?Sized + rodal::Dumper>(&self, dumper: &mut D) {
        dumper.debug_record::<Self>("dump");

        dumper.dump_object(&self.next_id);
        dumper.dump_object(&self.id_name_map);
        dumper.dump_object(&self.name_id_map);
        dumper.dump_object(&self.types);
        dumper.dump_object(&self.backend_type_info);
        dumper.dump_object(&self.constants);
        dumper.dump_object(&self.globals);
        dumper.dump_object(&self.func_sigs);
        dumper.dump_object(&self.funcs);
        dumper.dump_object(&self.primordial);
        dumper.dump_object(&self.vm_options);
        dumper.dump_object(&self.compiled_funcs);
        dumper.dump_object(&self.callsite_table);

        // Dump empty maps so that we can safely read and modify them once loaded
        dumper.dump_padding(&self.global_locations);
        let global_locations = RwLock::new(rodal::EmptyHashMap::<MuID, ValueLocation>::new());
        dumper.dump_object_here(&global_locations);

        dumper.dump_padding(&self.func_vers);
        let func_vers = RwLock::new(
            rodal::EmptyHashMap::<MuID, RwLock<MuFunctionVersion>>::new()
        );
        dumper.dump_object_here(&func_vers);

        dumper.dump_padding(&self.aot_pending_funcref_store);
        let aot_pending_funcref_store =
            RwLock::new(rodal::EmptyHashMap::<Address, ValueLocation>::new());
        dumper.dump_object_here(&aot_pending_funcref_store);

        dumper.dump_padding(&self.compiled_callsite_table);
        let compiled_callsite_table =
            RwLock::new(rodal::EmptyHashMap::<Address, CompiledCallsite>::new());
        dumper.dump_object_here(&compiled_callsite_table);

        dumper.dump_object(&self.callsite_count);

        dumper.dump_padding(&self.pending_joins);
        let pending_joins = Mutex::new(rodal::EmptyLinkedList::<JoinHandle<()>>::new());
        dumper.dump_object_here(&pending_joins);
    }
}

/// a fake funcref to store for AOT when client tries to store a funcref via API
//  For AOT scenario, when client tries to store funcref to the heap, the store
//  happens before we have an actual address for the function so we store a fake
//  funcref and when generating boot image, we fix the funcref with a relocatable symbol
const PENDING_FUNCREF: u64 = {
    use std::u64;
    u64::MAX
};

/// a macro to generate int8/16/32/64 from/to API calls
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

impl Drop for VM {
    fn drop(&mut self) {
        self.destroy();
    }
}

impl<'a> VM {
    /// creates a VM with default options
    pub fn new() -> VM {
        VM::new_internal(VMOptions::default())
    }

    /// creates a VM with specified options
    pub fn new_with_opts(str: &str) -> VM {
        VM::new_internal(VMOptions::init(str))
    }

    /// internal function to create a VM with options
    #[cfg(not(feature = "sel4-rumprun"))]
    fn new_internal(options: VMOptions) -> VM {
        VM::start_logging(options.flag_log_level);

        let ret = VM {
            next_id: ATOMIC_USIZE_INIT,
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
            callsite_table: RwLock::new(HashMap::new()),
            primordial: RwLock::new(None),
            aot_pending_funcref_store: RwLock::new(HashMap::new()),
            compiled_callsite_table: RwLock::new(HashMap::new()),
            callsite_count: ATOMIC_USIZE_INIT,
            pending_joins: Mutex::new(LinkedList::new())
        };

        // insert all internal types
        {
            let mut types = ret.types.write().unwrap();
            for ty in INTERNAL_TYPES.iter() {
                types.insert(ty.id(), ty.clone());
            }
        }

        // starts allocating ID from USER_ID_START
        ret.next_id.store(USER_ID_START, Ordering::Relaxed);

        // init types
        types::init_types();

        // init runtime
        ret.init_runtime();

        ret
    }

    /// internal function to create a VM with options for sel4-rumprun
    /// default memory sizes are different from other platforms
    #[cfg(feature = "sel4-rumprun")]
    fn new_internal(options: VMOptions) -> VM {
        VM::start_logging(options.flag_log_level);

        let mut ret = VM {
            next_id: ATOMIC_USIZE_INIT,
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
            callsite_table: RwLock::new(HashMap::new()),
            primordial: RwLock::new(None),
            aot_pending_funcref_store: RwLock::new(HashMap::new()),
            compiled_callsite_table: RwLock::new(HashMap::new()),
            callsite_count: ATOMIC_USIZE_INIT
        };

        // currently, the default sizes don't work on sel4-rumprun platform
        // this is due to memory allocation size limitations
        ret.vm_options.flag_gc_immixspace_size = 1 << 19;
        ret.vm_options.flag_gc_lospace_size = 1 << 19;

        // insert all internal types
        {
            let mut types = ret.types.write().unwrap();
            for ty in INTERNAL_TYPES.iter() {
                types.insert(ty.id(), ty.clone());
            }
        }

        // starts allocating ID from USER_ID_START
        ret.next_id.store(USER_ID_START, Ordering::Relaxed);

        // init types
        types::init_types();

        // init runtime
        ret.init_runtime();

        ret
    }

    /// initializes runtime
    fn init_runtime(&self) {
        // init gc
        {
            let ref options = self.vm_options;
            gc::gc_init(
                options.flag_gc_immixspace_size,
                options.flag_gc_lospace_size,
                options.flag_gc_nthreads,
                !options.flag_gc_disable_collection
            );
        }
    }

    /// starts logging based on MuLogLevel flag
    fn start_logging(level: MuLogLevel) {
        use std::env;
        match level {
            MuLogLevel::None => {}
            MuLogLevel::Error => VM::start_logging_internal(LogLevel::Error),
            MuLogLevel::Warn => VM::start_logging_internal(LogLevel::Warn),
            MuLogLevel::Info => VM::start_logging_internal(LogLevel::Info),
            MuLogLevel::Debug => VM::start_logging_internal(LogLevel::Debug),
            MuLogLevel::Trace => VM::start_logging_internal(LogLevel::Trace),
            MuLogLevel::Env => {
                match env::var("MU_LOG_LEVEL") {
                    Ok(s) => VM::start_logging(MuLogLevel::from_string(s)),
                    _ => {} // Don't log
                }
            }
        }
    }

    /// starts trace-level logging
    pub fn start_logging_trace() {
        VM::start_logging_internal(LogLevel::Trace)
    }

    /// starts logging based on MU_LOG_LEVEL environment variable
    pub fn start_logging_env() {
        VM::start_logging(MuLogLevel::Env)
    }

    /// starts logging based on Rust's LogLevel
    /// (this function actually initializes logger and deals with error)
    fn start_logging_internal(level: LogLevel) {
        use stderrlog;

        let verbose = match level {
            LogLevel::Error => 0,
            LogLevel::Warn => 1,
            LogLevel::Info => 2,
            LogLevel::Debug => 3,
            LogLevel::Trace => 4
        };

        match stderrlog::new().verbosity(verbose).init() {
            Ok(()) => { info!("logger initialized") }
            Err(e) => {
                error!(
                    "failed to init logger, probably already initialized: {:?}",
                    e
                )
            }
        }
    }

    /// cleans up currenet VM
    fn destroy(&mut self) {
        gc::gc_destoy();
    }

    /// adds an exception callsite and catch block
    /// (later we will use this info to build an exception table for unwinding use)
    pub fn add_exception_callsite(&self, callsite: Callsite, fv: MuID) {
        let mut table = self.callsite_table.write().unwrap();

        if table.contains_key(&fv) {
            table.get_mut(&fv).unwrap().push(callsite);
        } else {
            table.insert(fv, vec![callsite]);
        };
        // TODO: do wee need a stronger ordering??
        self.callsite_count.fetch_add(1, Ordering::Relaxed);
    }

    /// resumes persisted VM. Ideally the VM should be back to the status when we start
    /// persisting it except a few fields that we do not want to persist.
    pub fn resume_vm(dumped_vm: *mut Arc<VM>) -> Arc<VM> {
        // load the vm back
        let vm = unsafe { rodal::load_asm_pointer_move(dumped_vm) };

        // initialize runtime
        vm.init_runtime();

        // construct exception table
        vm.build_callsite_table();

        // restore gc types
        {
            let type_info_guard = vm.backend_type_info.read().unwrap();
            let mut type_info_vec: Vec<Box<BackendType>> =
                type_info_guard.values().map(|x| x.clone()).collect();
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
        vm.build_callsite_table();
        vm
    }

    /// builds a succinct exception table for fast query during exception unwinding
    /// We need this step because for AOT compilation, we do not know symbol address at compile,
    /// and resolving symbol address during exception handling is expensive. Thus when boot image
    /// gets executed, we first resolve symbols and store the results in another table for fast
    /// query.
    pub fn build_callsite_table(&self) {
        let callsite_table = self.callsite_table.read().unwrap();
        let compiled_funcs = self.compiled_funcs.read().unwrap();
        let mut compiled_callsite_table = self.compiled_callsite_table.write().unwrap();
        // TODO: Use a different ordering?
        compiled_callsite_table.reserve(self.callsite_count.load(Ordering::Relaxed));
        for (fv, callsite_list) in callsite_table.iter() {
            let compiled_func = compiled_funcs.get(fv).unwrap().read().unwrap();
            let callee_saved_table = Arc::new(compiled_func.frame.callee_saved.clone());
            for callsite in callsite_list.iter() {
                compiled_callsite_table.insert(
                    resolve_symbol(callsite.name.clone()),
                    CompiledCallsite::new(
                        &callsite,
                        compiled_func.func_ver_id,
                        callee_saved_table.clone()
                    )
                );
            }
        }
    }

    /// returns a valid ID for use next
    pub fn next_id(&self) -> MuID {
        // This only needs to be atomic, and does not need to be a synchronisation operation. The
        // only requirement for IDs is that all IDs obtained from `next_id()` are different. So
        // `Ordering::Relaxed` is sufficient.
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    /// are we doing AOT compilation? (feature = aot when building Zebu)
    pub fn is_doing_aot(&self) -> bool {
        return cfg!(feature = "aot");
    }

    /// are we doing JIT compilation? (feature = jit when building Zebu)
    pub fn is_doing_jit(&self) -> bool {
        return cfg!(feature = "jit");
    }

    /// informs VM about a client-supplied name
    pub fn set_name(&self, entity: &MuEntity) {
        let id = entity.id();
        let name = entity.name();

        let mut map = self.id_name_map.write().unwrap();
        map.insert(id, name.clone());

        let mut map2 = self.name_id_map.write().unwrap();
        map2.insert(name, id);
    }

    /// returns Mu ID for a client-supplied name
    /// This function should only used by client, 'name' used internally may be slightly different
    /// due to removal of some special symbols in the MuName. See name_check() in ir.rs
    pub fn id_of(&self, name: &str) -> MuID {
        let map = self.name_id_map.read().unwrap();
        match map.get(&name.to_string()) {
            Some(id) => *id,
            None => panic!("cannot find id for name: {}", name)
        }
    }

    /// returns the client-supplied Mu name for Mu ID
    /// This function should only used by client, 'name' used internally may be slightly different
    /// due to removal of some special symbols in the MuName. See name_check() in ir.rs
    pub fn name_of(&self, id: MuID) -> MuName {
        let map = self.id_name_map.read().unwrap();
        map.get(&id).unwrap().clone()
    }

    /// declares a constant
    pub fn declare_const(&self, entity: MuEntityHeader, ty: P<MuType>, val: Constant) -> P<Value> {
        let ret = P(Value {
            hdr: entity,
            ty: ty,
            v: Value_::Constant(val)
        });

        let mut constants = self.constants.write().unwrap();
        self.declare_const_internal(&mut constants, ret.id(), ret.clone());

        ret
    }

    /// adds a constant to the map (already acquired lock)
    fn declare_const_internal(
        &self,
        map: &mut RwLockWriteGuard<HashMap<MuID, P<Value>>>,
        id: MuID,
        val: P<Value>
    ) {
        debug_assert!(!map.contains_key(&id));

        info!("declare const #{} = {}", id, val);
        map.insert(id, val);
    }

    /// gets the constant P<Value> for a given Mu ID, panics if there is no type with the ID
    pub fn get_const(&self, id: MuID) -> P<Value> {
        let const_lock = self.constants.read().unwrap();
        match const_lock.get(&id) {
            Some(ret) => ret.clone(),
            None => panic!("cannot find const #{}", id)
        }
    }

    /// allocates memory for a constant that needs to be put in memory
    /// For AOT, we simply create a label for it, and let code emitter allocate the memory
    #[cfg(feature = "aot")]
    pub fn allocate_const(&self, val: &P<Value>) -> ValueLocation {
        let id = val.id();
        let name = format!("CONST_{}_{}", id, val.name());

        ValueLocation::Relocatable(backend::RegGroup::GPR, Arc::new(name))
    }

    /// declares a global
    pub fn declare_global(&self, entity: MuEntityHeader, ty: P<MuType>) -> P<Value> {
        // create iref value for the global
        let global = P(Value {
            hdr: entity,
            ty: self.declare_type(
                MuEntityHeader::unnamed(self.next_id()),
                MuType_::iref(ty.clone())
            ),
            v: Value_::Global(ty)
        });

        let mut globals = self.globals.write().unwrap();
        let mut global_locs = self.global_locations.write().unwrap();
        self.declare_global_internal(&mut globals, &mut global_locs, global.id(), global.clone());

        global
    }

    /// adds the global to the map (already acquired lock), and allocates memory for it
    fn declare_global_internal(
        &self,
        globals: &mut RwLockWriteGuard<HashMap<MuID, P<Value>>>,
        global_locs: &mut RwLockWriteGuard<HashMap<MuID, ValueLocation>>,
        id: MuID,
        val: P<Value>
    ) {
        self.declare_global_internal_no_alloc(globals, id, val.clone());
        self.alloc_global(global_locs, id, val);
    }

    /// adds the global to the map (already acquired lock)
    /// when bulk declaring, we hold locks for everything, we cannot resolve backend type
    /// and do alloc so we add globals to the map, and then allocate them later
    fn declare_global_internal_no_alloc(
        &self,
        globals: &mut RwLockWriteGuard<HashMap<MuID, P<Value>>>,
        id: MuID,
        val: P<Value>
    ) {
        debug_assert!(!globals.contains_key(&id));

        info!("declare global #{} = {}", id, val);
        globals.insert(id, val.clone());
    }

    /// allocates memory for a global cell
    fn alloc_global(
        &self,
        global_locs: &mut RwLockWriteGuard<HashMap<MuID, ValueLocation>>,
        id: MuID,
        val: P<Value>
    ) {
        let backend_ty = self.get_backend_type_info(val.ty.get_referent_ty().unwrap().id());
        let loc = gc::allocate_global(val, backend_ty);
        trace!("allocate global #{} as {}", id, loc);
        global_locs.insert(id, loc);
    }

    /// declares a type
    pub fn declare_type(&self, entity: MuEntityHeader, ty: MuType_) -> P<MuType> {
        let ty = P(MuType { hdr: entity, v: ty });

        let mut types = self.types.write().unwrap();
        self.declare_type_internal(&mut types, ty.id(), ty.clone());

        ty
    }

    /// adds the type to the map (already acquired lock)
    fn declare_type_internal(
        &self,
        types: &mut RwLockWriteGuard<HashMap<MuID, P<MuType>>>,
        id: MuID,
        ty: P<MuType>
    ) {
        debug_assert!(!types.contains_key(&id));

        types.insert(id, ty.clone());
        info!("declare type #{} = {}", id, ty);

        // for struct/hybrid, also adds to struct/hybrid tag map
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

    /// gets the type for a given Mu ID, panics if there is no type with the ID
    pub fn get_type(&self, id: MuID) -> P<MuType> {
        let type_lock = self.types.read().unwrap();
        match type_lock.get(&id) {
            Some(ret) => ret.clone(),
            None => panic!("cannot find type #{}", id)
        }
    }

    /// declares a function signature
    pub fn declare_func_sig(
        &self,
        entity: MuEntityHeader,
        ret_tys: Vec<P<MuType>>,
        arg_tys: Vec<P<MuType>>
    ) -> P<MuFuncSig> {
        let ret = P(MuFuncSig {
            hdr: entity,
            ret_tys: ret_tys,
            arg_tys: arg_tys
        });

        let mut func_sigs = self.func_sigs.write().unwrap();
        self.declare_func_sig_internal(&mut func_sigs, ret.id(), ret.clone());

        ret
    }

    /// adds a function signature to the map (already acquired lock)
    fn declare_func_sig_internal(
        &self,
        sigs: &mut RwLockWriteGuard<HashMap<MuID, P<MuFuncSig>>>,
        id: MuID,
        sig: P<MuFuncSig>
    ) {
        debug_assert!(!sigs.contains_key(&id));

        info!("declare func sig #{} = {}", id, sig);
        sigs.insert(id, sig);
    }

    /// gets the function signature for a given ID, panics if there is no func sig with the ID
    pub fn get_func_sig(&self, id: MuID) -> P<MuFuncSig> {
        let func_sig_lock = self.func_sigs.read().unwrap();
        match func_sig_lock.get(&id) {
            Some(ret) => ret.clone(),
            None => panic!("cannot find func sig #{}", id)
        }
    }

    /// declares a Mu function
    pub fn declare_func(&self, func: MuFunction) {
        let mut funcs = self.funcs.write().unwrap();

        self.declare_func_internal(&mut funcs, func.id(), func);
    }

    /// adds a Mu function to the map (already acquired lock)
    fn declare_func_internal(
        &self,
        funcs: &mut RwLockWriteGuard<HashMap<MuID, RwLock<MuFunction>>>,
        id: MuID,
        func: MuFunction
    ) {
        debug_assert!(!funcs.contains_key(&id));

        info!("declare func #{} = {}", id, func);
        funcs.insert(id, RwLock::new(func));
    }

    /// gets the function name for a function (by ID), panics if there is no function with the ID
    /// Note this name is the internal name, which is different than
    /// the client-supplied name from vm.name_of()
    pub fn get_name_for_func(&self, id: MuID) -> MuName {
        let funcs_lock = self.funcs.read().unwrap();
        match funcs_lock.get(&id) {
            Some(func) => func.read().unwrap().name(),
            None => panic!("cannot find name for Mu function #{}")
        }
    }

    /// gets the function signature for a function (by ID),
    /// panics if there is no function with the ID
    pub fn get_sig_for_func(&self, id: MuID) -> P<MuFuncSig> {
        let funcs_lock = self.funcs.read().unwrap();
        match funcs_lock.get(&id) {
            Some(func) => func.read().unwrap().sig.clone(),
            None => panic!("cannot find Mu function #{}", id)
        }
    }

    /// gets the current function version for a Mu function (by ID)
    /// returns None if the function does not exist, or no version is defined for the function
    pub fn get_cur_version_for_func(&self, fid: MuID) -> Option<MuID> {
        let funcs_guard = self.funcs.read().unwrap();
        match funcs_guard.get(&fid) {
            Some(rwlock_func) => {
                let func_guard = rwlock_func.read().unwrap();
                func_guard.cur_ver
            }
            None => None
        }
    }

    /// gets the address as ValueLocation of a Mu function (by ID)
    pub fn get_address_for_func(&self, func_id: MuID) -> ValueLocation {
        let funcs = self.funcs.read().unwrap();
        let func: &MuFunction = &funcs.get(&func_id).unwrap().read().unwrap();

        if self.is_doing_jit() {
            unimplemented!()
        } else {
            ValueLocation::Relocatable(backend::RegGroup::GPR, func.name())
        }
    }

    /// defines a function version
    pub fn define_func_version(&self, func_ver: MuFunctionVersion) {
        info!("define function version {}", func_ver);
        // add this funcver to map
        let func_ver_id = func_ver.id();
        {
            let mut func_vers = self.func_vers.write().unwrap();
            func_vers.insert(func_ver_id, RwLock::new(func_ver));
        }

        // acquire a reference to the func_ver
        let func_vers = self.func_vers.read().unwrap();
        let func_ver = func_vers.get(&func_ver_id).unwrap().write().unwrap();

        // change current version of the function to new version (obsolete old versions)
        let funcs = self.funcs.read().unwrap();
        // it should be declared before defining
        debug_assert!(funcs.contains_key(&func_ver.func_id));
        let mut func = funcs.get(&func_ver.func_id).unwrap().write().unwrap();

        func.new_version(func_ver.id());

        if self.is_doing_jit() {
            // redefinition may happen, we need to check
            unimplemented!()
        }
    }

    /// adds a new bundle into VM.
    /// This function will drain the contents of all arguments. Ideally, this function should
    /// happen atomically. e.g. The client should not see a new type added without also seeing
    /// a new function added.
    pub fn declare_many(
        &self,
        new_name_id_map: &mut HashMap<MuName, MuID>,
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

            for (name, id) in new_name_id_map.drain() {
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
                // we bulk allocate later
                // (since we are holding all the locks, we cannot find ty info)
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
                    trace!(
                        "Added funcver {} as a version of {} {:?}.",
                        id,
                        func_id,
                        func
                    );
                }
            }
        }
        // Locks released here

        // allocate all the globals defined
        {
            let globals = self.globals.read().unwrap();
            let mut global_locs = self.global_locations.write().unwrap();

            // make sure current thread has allocator
            let created =
                unsafe { MuThread::current_thread_as_mu_thread(Address::zero(), arc_vm.clone()) };

            for (id, global) in globals.iter() {
                self.alloc_global(&mut global_locs, *id, global.clone());
            }

            if created {
                unsafe { MuThread::cleanup_current_mu_thread() };
            }
        }
    }

    /// informs the VM of a newly compiled function
    /// (the function and funcver should already be declared before this call)
    pub fn add_compiled_func(&self, func: CompiledFunction) {
        debug_assert!(self.funcs.read().unwrap().contains_key(&func.func_id));
        debug_assert!(
            self.func_vers
                .read()
                .unwrap()
                .contains_key(&func.func_ver_id)
        );

        self.compiled_funcs
            .write()
            .unwrap()
            .insert(func.func_ver_id, RwLock::new(func));
    }

    /// gets the backend/storage type for a given Mu type (by ID)
    pub fn get_backend_type_info(&self, tyid: MuID) -> Box<BackendType> {
        // if we already resolved this type, return the BackendType
        {
            let read_lock = self.backend_type_info.read().unwrap();

            match read_lock.get(&tyid) {
                Some(info) => {
                    return info.clone();
                }
                None => {}
            }
        }

        // otherwise, we need to resolve the type now
        let types = self.types.read().unwrap();
        let ty = match types.get(&tyid) {
            Some(ty) => ty,
            None => panic!("invalid type id during get_backend_type_info(): {}", tyid)
        };
        let resolved = Box::new(backend::BackendType::resolve(ty, self));

        // insert the type so later we do not need to resolve it again
        let mut write_lock = self.backend_type_info.write().unwrap();
        write_lock.insert(tyid, resolved.clone());

        resolved
    }

    /// gets the backend/storage type size for a given Mu type (by ID)
    /// This is equivalent to vm.get_backend_type_info(id).size
    pub fn get_backend_type_size(&self, tyid: MuID) -> ByteSize {
        self.get_backend_type_info(tyid).size
    }

    /// returns the lock for globals
    pub fn globals(&self) -> &RwLock<HashMap<MuID, P<Value>>> {
        &self.globals
    }

    /// returns the lock for functions
    pub fn funcs(&self) -> &RwLock<HashMap<MuID, RwLock<MuFunction>>> {
        &self.funcs
    }

    /// returns the lock for function versions
    pub fn func_vers(&self) -> &RwLock<HashMap<MuID, RwLock<MuFunctionVersion>>> {
        &self.func_vers
    }

    /// returns the lock for compiled functions
    pub fn compiled_funcs(&self) -> &RwLock<HashMap<MuID, RwLock<CompiledFunction>>> {
        &self.compiled_funcs
    }

    /// returns the lock for types
    pub fn types(&self) -> &RwLock<HashMap<MuID, P<MuType>>> {
        &self.types
    }

    /// returns the lock for function signatures
    pub fn func_sigs(&self) -> &RwLock<HashMap<MuID, P<MuFuncSig>>> {
        &self.func_sigs
    }

    /// returns the lock for global locations
    pub fn global_locations(&self) -> &RwLock<HashMap<MuID, ValueLocation>> {
        &self.global_locations
    }

    /// returns the lock for primordial thread info
    pub fn primordial(&self) -> &RwLock<Option<PrimordialThreadInfo>> {
        &self.primordial
    }

    /// returns the lock for compiled callsite table
    pub fn compiled_callsite_table(&self) -> &RwLock<HashMap<Address, CompiledCallsite>> {
        &self.compiled_callsite_table
    }

    pub fn resolve_function_address(&self, func_id: MuID) -> ValueLocation {
        let funcs = self.funcs.read().unwrap();
        let func: &MuFunction = &funcs.get(&func_id).unwrap().read().unwrap();

        if self.is_doing_jit() {
            unimplemented!()
        } else {
            ValueLocation::Relocatable(backend::RegGroup::GPR, func.name())
        }
    }

    /// set info (entry function, arguments) for primordial thread for boot image
    pub fn set_primordial_thread(&self, func_id: MuID, has_const_args: bool, args: Vec<Constant>) {
        let mut guard = self.primordial.write().unwrap();
        *guard = Some(PrimordialThreadInfo {
            func_id: func_id,
            has_const_args: has_const_args,
            args: args
        });
    }

    /// makes a boot image
    /// We are basically following the spec for this API calls.
    /// However, there are a few differences:
    /// 1. we are not doing 'automagic' relocation for unsafe pointers, relocation of
    ///    unsafe pointers needs to be done via explicit sym_fields/strings, reloc_fields/strings
    /// 2. if the output name for the boot image has extension name for dynamic libraries
    ///    (.so or .dylib), we generate a dynamic library as boot image. Otherwise, we generate
    ///    an executable.
    /// 3. we do not support primordial stack (as Kunshan pointed out, making boot image with a
    ///    primordial stack may get deprecated)
    ///
    /// args:
    /// whitelist               : functions to be put into the boot image
    /// primordial_func         : starting function for the boot image
    /// primordial_stack        : starting stack for the boot image
    ///                           (client should name either primordial_func or stack,
    ///                            currently Zebu only supports func)
    /// primordial_threadlocal  : thread local for the starting thread
    /// sym_fields/strings      : declare an address with symbol
    /// reloc_fields/strings    : declare an field pointing to a symbol
    /// output_file             : path for the boot image
    pub fn make_boot_image(
        &self,
        whitelist: Vec<MuID>,
        primordial_func: Option<&APIHandle>,
        primordial_stack: Option<&APIHandle>,
        primordial_threadlocal: Option<&APIHandle>,
        sym_fields: Vec<&APIHandle>,
        sym_strings: Vec<MuName>,
        reloc_fields: Vec<&APIHandle>,
        reloc_strings: Vec<MuName>,
        output_file: String
    ) {
        self.make_boot_image_internal(
            whitelist,
            primordial_func,
            primordial_stack,
            primordial_threadlocal,
            sym_fields,
            sym_strings,
            reloc_fields,
            reloc_strings,
            vec![],
            output_file
        )
    }

    /// the actual function to make boot image
    /// One difference from the public one is that we allow linking extra source code during
    /// generating the boot image.
    #[allow(unused_variables)]
    pub fn make_boot_image_internal(
        &self,
        whitelist: Vec<MuID>,
        primordial_func: Option<&APIHandle>,
        primordial_stack: Option<&APIHandle>,
        primordial_threadlocal: Option<&APIHandle>,
        sym_fields: Vec<&APIHandle>,
        sym_strings: Vec<MuName>,
        reloc_fields: Vec<&APIHandle>,
        reloc_strings: Vec<MuName>,
        extra_sources_to_link: Vec<String>,
        output_file: String
    ) {
        info!("Making boot image...");

        // Only store name info for whitelisted entities
        {
            let mut new_id_name_map = HashMap::<MuID, MuName>::with_capacity(whitelist.len());
            let mut new_name_id_map = HashMap::<MuName, MuID>::with_capacity(whitelist.len());

            let mut id_name_map = self.id_name_map.write().unwrap();
            let mut name_id_map = self.name_id_map.write().unwrap();
            for &id in whitelist.iter() {
                match id_name_map.get(&id) {
                    Some(name) => {
                        new_id_name_map.insert(id, name.clone());
                        new_name_id_map.insert(name.clone(), id);
                    }
                    None => {}
                }
            }
            *id_name_map = new_id_name_map;
            *name_id_map = new_name_id_map;
        }

        // compile the whitelist functions
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

        let has_primordial_func = primordial_func.is_some();
        let has_primordial_stack = primordial_stack.is_some();

        // we assume client will start with a function (instead of a stack)
        if has_primordial_stack {
            panic!(
                "Zebu doesnt support creating primordial thread from a stack, \
                 name a entry function instead"
            )
        } else {
            if has_primordial_func {
                // extract func id
                let func_id = primordial_func.unwrap().v.as_funcref();

                // make primordial thread in vm
                // no const args are passed, we will use argc/argv
                self.set_primordial_thread(func_id, false, vec![]);
            } else {
                warn!("no entry function is passed");
            }

            // deal with relocation symbols, zip the two vectors into a hashmap
            assert_eq!(sym_fields.len(), sym_strings.len());
            let symbols: HashMap<Address, MuName> = sym_fields
                .into_iter()
                .map(|handle| handle.v.as_address())
                .zip(sym_strings.into_iter())
                .collect();

            // deal with relocation fields
            // zip the two vectors into a hashmap, and add fields for pending funcref stores
            assert_eq!(reloc_fields.len(), reloc_strings.len());
            let fields = {
                // init reloc fields with client-supplied field/symbol pair
                let mut reloc_fields: HashMap<Address, MuName> = reloc_fields
                    .into_iter()
                    .map(|handle| handle.v.as_address())
                    .zip(reloc_strings.into_iter())
                    .collect();

                // pending funcrefs - we want to replace them as symbol
                {
                    let mut pending_funcref = self.aot_pending_funcref_store.write().unwrap();
                    for (addr, vl) in pending_funcref.drain() {
                        reloc_fields.insert(addr, vl.to_relocatable());
                    }
                }

                reloc_fields
            };

            // emit context (persist vm, etc)
            backend::emit_context_with_reloc(self, symbols, fields);

            // link
            self.link_boot_image(whitelist_funcs, extra_sources_to_link, output_file);
        }
    }

    /// links boot image (generates a dynamic library is the specified output file
    /// has dylib extension, otherwise generates an executable)
    #[cfg(feature = "aot")]
    fn link_boot_image(&self, funcs: Vec<MuID>, extra_srcs: Vec<String>, output_file: String) {
        use linkutils;

        info!("Linking boot image...");

        let func_names = {
            let funcs_guard = self.funcs().read().unwrap();
            funcs
                .iter()
                .map(|x| funcs_guard.get(x).unwrap().read().unwrap().name())
                .collect()
        };

        trace!("functions: {:?}", func_names);
        trace!("extern sources: {:?}", extra_srcs);
        trace!("output   : {}", output_file);

        if output_file.ends_with("dylib") || output_file.ends_with("so") {
            // compile as dynamic library
            linkutils::aot::link_dylib_with_extra_srcs(func_names, extra_srcs, &output_file, self);
        } else {
            if extra_srcs.len() != 0 {
                panic!("trying to create an executable with linking extern sources, unimplemented");
            }
            // compile as executable
            linkutils::aot::link_primordial(func_names, &output_file, self);
        }

        trace!("Done!");
    }

    // the following functions are implementing corresponding APIs

    /// creates a new stack with the given entry function
    pub fn new_stack(&self, func_id: MuID) -> Box<MuStack> {
        let funcs = self.funcs.read().unwrap();
        let func: &MuFunction = &funcs.get(&func_id).unwrap().read().unwrap();

        let func_addr = resolve_symbol(self.get_name_for_func(func_id));
        let stack_arg_size = backend::call_stack_size(func.sig.clone(), self);

        Box::new(MuStack::new(self.next_id(), func_addr, stack_arg_size))
    }

    /// creates a handle that we can return to the client
    fn new_handle(&self, handle: APIHandle) -> APIHandleResult {
        let ret = Box::new(handle);

        ret
    }

    /// creates a fix sized object in the heap, and returns a reference handle
    pub fn new_fixed(&self, tyid: MuID) -> APIHandleResult {
        let ty = self.get_type(tyid);
        assert!(!ty.is_hybrid());

        let backend_ty = self.get_backend_type_info(tyid);
        let addr = gc::allocate_fixed(ty.clone(), backend_ty);
        trace!("API: allocated fixed type {} at {}", ty, addr);

        self.new_handle(APIHandle {
            id: self.next_id(),
            v: APIHandleValue::Ref(ty, addr)
        })
    }

    /// creates a hybrid type object in the heap, and returns a reference handle
    pub fn new_hybrid(&self, tyid: MuID, length: APIHandleArg) -> APIHandleResult {
        let ty = self.get_type(tyid);
        assert!(ty.is_hybrid());

        let len = self.handle_to_uint64(length);

        let backend_ty = self.get_backend_type_info(tyid);
        let addr = gc::allocate_hybrid(ty.clone(), len, backend_ty);
        trace!(
            "API: allocated hybrid type {} of length {} at {}",
            ty,
            len,
            addr
        );

        self.new_handle(APIHandle {
            id: self.next_id(),
            v: APIHandleValue::Ref(ty, addr)
        })
    }

    /// performs REFCAST
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
            }
            APIHandleValue::IRef(_, addr) => {
                assert!(to_ty.is_iref());
                let inner_ty = to_ty.get_referent_ty().unwrap();

                self.new_handle(APIHandle {
                    id: handle_id,
                    v: APIHandleValue::IRef(inner_ty, addr)
                })
            }
            APIHandleValue::FuncRef(_) => unimplemented!(),

            _ => panic!("unexpected operand for refcast: {:?}", from_op)
        }
    }

    /// performs GETIREF
    pub fn handle_get_iref(&self, handle_ref: APIHandleArg) -> APIHandleResult {
        let (ty, addr) = handle_ref.v.as_ref();

        // assume iref has the same address as ref
        let ret = self.new_handle(APIHandle {
            id: self.next_id(),
            v: APIHandleValue::IRef(ty, addr)
        });

        trace!("API: get iref from {:?}", handle_ref);
        trace!("API: result {:?}", ret);

        ret
    }

    /// performs SHIFTIREF
    pub fn handle_shift_iref(
        &self,
        handle_iref: APIHandleArg,
        offset: APIHandleArg
    ) -> APIHandleResult {
        let (ty, addr) = handle_iref.v.as_iref();
        let offset = self.handle_to_uint64(offset);

        let offset_addr = {
            use utils::math::align_up;

            let backend_ty = self.get_backend_type_info(ty.id());
            let aligned_size = align_up(backend_ty.size, backend_ty.alignment);
            addr + (aligned_size * (offset as usize))
        };

        let ret = self.new_handle(APIHandle {
            id: self.next_id(),
            v: APIHandleValue::IRef(ty, offset_addr)
        });

        trace!("API: shift iref from {:?}", handle_iref);
        trace!("API: result {:?}", ret);

        ret
    }

    /// performs GETELEMIREF
    pub fn handle_get_elem_iref(
        &self,
        handle_iref: APIHandleArg,
        index: APIHandleArg
    ) -> APIHandleResult {
        let (ty, addr) = handle_iref.v.as_iref();
        let index = self.handle_to_uint64(index);

        let ele_ty = match ty.get_elem_ty() {
            Some(ty) => ty,
            None => panic!("cannot get element ty from {}", ty)
        };
        let elem_addr = {
            use utils::math::align_up;

            let backend_ty = self.get_backend_type_info(ele_ty.id());
            let aligned_size = align_up(backend_ty.size, backend_ty.alignment);
            addr + (aligned_size * (index as usize))
        };

        let ret = self.new_handle(APIHandle {
            id: self.next_id(),
            v: APIHandleValue::IRef(ele_ty, elem_addr)
        });

        trace!(
            "API: get element iref from {:?} at index {:?}",
            handle_iref,
            index
        );
        trace!("API: result {:?}", ret);

        ret
    }

    /// performs GETVARPARTIREF
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
            v: APIHandleValue::IRef(varpart_ty, varpart_addr)
        });

        trace!("API: get var part iref from {:?}", handle_iref);
        trace!("API: result {:?}", ret);

        ret
    }

    /// performs GETFIELDIREF
    pub fn handle_get_field_iref(
        &self,
        handle_iref: APIHandleArg,
        field: usize
    ) -> APIHandleResult {
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
            v: APIHandleValue::IRef(field_ty, field_addr)
        });

        trace!(
            "API: get field iref from {:?}, field: {}",
            handle_iref,
            field
        );
        trace!("API: result {:?}", ret);

        ret
    }

    /// performs LOAD
    #[allow(unused_variables)]
    pub fn handle_load(&self, ord: MemoryOrder, loc: APIHandleArg) -> APIHandleResult {
        let (ty, addr) = loc.v.as_iref();

        // FIXME: not using memory order for load at the moment - See Issue #51
        let rust_memord = match ord {
            MemoryOrder::Relaxed => Ordering::Relaxed,
            MemoryOrder::Acquire => Ordering::Acquire,
            MemoryOrder::SeqCst => Ordering::SeqCst,
            MemoryOrder::NotAtomic => Ordering::Relaxed,    // use relax for not atomic
            MemoryOrder::Consume => Ordering::Acquire,    // use acquire for consume
            _ => panic!("unsupported order {:?} for LOAD", ord)
        };

        let handle_id = self.next_id();
        let handle_value = unsafe {
            match ty.v {
                MuType_::Int(len) => APIHandleValue::Int(addr.load::<u64>(), len),
                MuType_::Float => APIHandleValue::Float(addr.load::<f32>()),
                MuType_::Double => APIHandleValue::Double(addr.load::<f64>()),
                MuType_::Ref(ref ty) => APIHandleValue::Ref(ty.clone(), addr.load::<Address>()),
                MuType_::IRef(ref ty) => APIHandleValue::IRef(ty.clone(), addr.load::<Address>()),
                MuType_::UPtr(ref ty) => APIHandleValue::UPtr(ty.clone(), addr.load::<Address>()),
                MuType_::Tagref64 => APIHandleValue::TagRef64(addr.load::<u64>()),

                _ => unimplemented!()
            }
        };

        let ret = self.new_handle(APIHandle {
            id: handle_id,
            v: handle_value
        });

        trace!("API: load from {:?}", loc);
        trace!("API: result {:?}", ret);

        ret
    }

    /// performs STORE
    #[allow(unused_variables)]
    pub fn handle_store(&self, ord: MemoryOrder, loc: APIHandleArg, val: APIHandleArg) {
        // get address
        let (_, addr) = loc.v.as_iref();

        // FIXME: not using memory order for store at the moment - See Issue #51
        let rust_memord = match ord {
            MemoryOrder::Relaxed => Ordering::Relaxed,
            MemoryOrder::Release => Ordering::Release,
            MemoryOrder::SeqCst => Ordering::SeqCst,
            MemoryOrder::NotAtomic => Ordering::Relaxed,    // use relaxed for not atomic
            _ => panic!("unsupported order {:?} for STORE", ord)
        };

        // get value and store
        // we will store here (its unsafe)
        unsafe {
            match val.v {
                APIHandleValue::Int(ival, bits) => {
                    let trunc: u64 = ival & bits_ones(bits);
                    match bits {
                        1...8 => addr.store::<u8>(trunc as u8),
                        9...16 => addr.store::<u16>(trunc as u16),
                        17...32 => addr.store::<u32>(trunc as u32),
                        33...64 => addr.store::<u64>(trunc as u64),
                        _ => panic!("unimplemented int length")
                    }
                }
                APIHandleValue::TagRef64(val) => addr.store::<u64>(val),
                APIHandleValue::Float(fval) => addr.store::<f32>(fval),
                APIHandleValue::Double(fval) => addr.store::<f64>(fval),
                APIHandleValue::UPtr(_, aval) => addr.store::<Address>(aval),
                APIHandleValue::UFP(_, aval) => addr.store::<Address>(aval),

                APIHandleValue::Struct(_) | APIHandleValue::Array(_) |
                APIHandleValue::Vector(_) => unimplemented!(),

                APIHandleValue::Ref(_, aval) | APIHandleValue::IRef(_, aval) => {
                    addr.store::<Address>(aval)
                }

                // if we are JITing, we can store the address of the function
                // but if we are doing AOT, we pend the store, and resolve the store
                // when making boot image
                APIHandleValue::FuncRef(id) => {
                    if self.is_doing_jit() {
                        unimplemented!()
                    } else {
                        self.store_funcref(addr, id)
                    }
                }

                _ => unimplemented!()
            }
        }

        trace!("API: store value {:?} to location {:?}", val, loc);
    }

    #[cfg(feature = "aot")]
    fn store_funcref(&self, addr: Address, func_id: MuID) {
        // put a pending funcref in the address
        unsafe { addr.store::<u64>(PENDING_FUNCREF) };

        // and record this funcref
        let symbol = self.get_name_for_func(func_id);

        let mut pending_funcref_guard = self.aot_pending_funcref_store.write().unwrap();
        pending_funcref_guard.insert(
            addr,
            ValueLocation::Relocatable(backend::RegGroup::GPR, symbol)
        );
    }

    /// performs CommonInst_Pin
    //  This function and the following two make assumption that GC will not move object.
    //  They need to be reimplemented if we have a moving GC
    //  FIXME: The pin/unpin semantic (here and in instruction selection) is different from Mu spec
    //  See Issue #33
    pub fn handle_pin_object(&self, loc: APIHandleArg) -> APIHandleResult {
        debug_assert!(!gc::GC_MOVES_OBJECT);
        // gc will not move, so we just put ref into uptr

        let (ty, addr) = loc.v.as_ref_or_iref();
        self.new_handle(APIHandle {
            id: self.next_id(),
            v: APIHandleValue::UPtr(ty, addr)
        })
    }

    /// performs CommonInst_Unpin
    #[allow(unused_variables)]
    pub fn handle_unpin_object(&self, loc: APIHandleArg) {
        debug_assert!(!gc::GC_MOVES_OBJECT);
        // gc will not move, do no need to unpin
        // do nothing
    }

    /// performs CommonInst_GetAddr
    pub fn handle_get_addr(&self, loc: APIHandleArg) -> APIHandleResult {
        assert!(!gc::GC_MOVES_OBJECT);
        // loc needs to be already pinned - we don't check since we don't pin

        let (ty, addr) = loc.v.as_ref_or_iref();
        self.new_handle(APIHandle {
            id: self.next_id(),
            v: APIHandleValue::UPtr(ty, addr)
        })
    }

    /// creates a handle for a function (by ID)
    pub fn handle_from_func(&self, id: MuID) -> APIHandleResult {
        let handle_id = self.next_id();

        self.new_handle(APIHandle {
            id: handle_id,
            v: APIHandleValue::FuncRef(id)
        })
    }

    /// creates a handle for a global (by ID)
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
            v: APIHandleValue::IRef(global_inner_ty, global_iref)
        })
    }

    /// creates a handle for a constant (by ID)
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
                    None => {
                        panic!(
                            "expected ty to be Int for a Constant::Int, found {}",
                            const_ty
                        )
                    }
                };

                APIHandle {
                    id: handle_id,
                    v: APIHandleValue::Int(val, len)
                }
            }
            Value_::Constant(Constant::Float(val)) => {
                APIHandle {
                    id: handle_id,
                    v: APIHandleValue::Float(val)
                }
            }
            Value_::Constant(Constant::Double(val)) => {
                APIHandle {
                    id: handle_id,
                    v: APIHandleValue::Double(val)
                }
            }
            Value_::Constant(Constant::FuncRef(_)) => unimplemented!(),
            Value_::Constant(Constant::NullRef) => {
                APIHandle {
                    id: handle_id,
                    v: APIHandleValue::Ref(types::VOID_TYPE.clone(), unsafe { Address::zero() })
                }
            }
            _ => unimplemented!()
        };

        self.new_handle(handle)
    }

    /// creates a handle for unsigned int 64
    gen_handle_int!(handle_from_uint64, handle_to_uint64, u64);
    /// creates a handle for unsigned int 32
    gen_handle_int!(handle_from_uint32, handle_to_uint32, u32);
    /// creates a handle for unsigned int 16
    gen_handle_int!(handle_from_uint16, handle_to_uint16, u16);
    /// creates a handle for unsigned int 8
    gen_handle_int!(handle_from_uint8, handle_to_uint8, u8);
    /// creates a handle for signed int 64
    gen_handle_int!(handle_from_sint64, handle_to_sint64, i64);
    /// creates a handle for signed int 32
    gen_handle_int!(handle_from_sint32, handle_to_sint32, i32);
    /// creates a handle for signed int 16
    gen_handle_int!(handle_from_sint16, handle_to_sint16, i16);
    /// creates a handle for signed int 8
    gen_handle_int!(handle_from_sint8, handle_to_sint8, i8);

    /// creates a handle for float
    pub fn handle_from_float(&self, num: f32) -> APIHandleResult {
        let handle_id = self.next_id();
        self.new_handle(APIHandle {
            id: handle_id,
            v: APIHandleValue::Float(num)
        })
    }

    pub fn push_join_handle(&self, join_handle: JoinHandle<()>) {
        self.pending_joins.lock().unwrap().push_front(join_handle);
    }
    pub fn pop_join_handle(&self) -> Option<JoinHandle<()>> {
        self.pending_joins.lock().unwrap().pop_front()
    }
    /// unwraps a handle to float
    pub fn handle_to_float(&self, handle: APIHandleArg) -> f32 {
        handle.v.as_float()
    }

    /// creates a handle for double
    pub fn handle_from_double(&self, num: f64) -> APIHandleResult {
        let handle_id = self.next_id();
        self.new_handle(APIHandle {
            id: handle_id,
            v: APIHandleValue::Double(num)
        })
    }

    /// unwraps a handle to double
    pub fn handle_to_double(&self, handle: APIHandleArg) -> f64 {
        handle.v.as_double()
    }

    /// creates a handle for unsafe pointer
    pub fn handle_from_uptr(&self, tyid: MuID, ptr: Address) -> APIHandleResult {
        let ty = self.get_type(tyid);

        let handle_id = self.next_id();
        self.new_handle(APIHandle {
            id: handle_id,
            v: APIHandleValue::UPtr(ty, ptr)
        })
    }

    /// unwraps a handle to unsafe pointer
    pub fn handle_to_uptr(&self, handle: APIHandleArg) -> Address {
        handle.v.as_uptr().1
    }

    /// creates a handle for unsafe function pointer
    pub fn handle_from_ufp(&self, tyid: MuID, ptr: Address) -> APIHandleResult {
        let ty = self.get_type(tyid);

        let handle_id = self.next_id();
        self.new_handle(APIHandle {
            id: handle_id,
            v: APIHandleValue::UFP(ty, ptr)
        })
    }

    /// unwraps a handle to unsafe function pointer
    pub fn handle_to_ufp(&self, handle: APIHandleArg) -> Address {
        handle.v.as_ufp().1
    }

    // Functions for handling TagRef64-related API calls are taken from:
    // https://gitlab.anu.edu.au/mu/mu-impl-ref2/blob/master/src/main/scala/uvm/refimpl/
    // itpr/operationHelpers.scala

    /// checks if a Tagref64 is a floating point value
    pub fn handle_tr64_is_fp(&self, value: APIHandleArg) -> bool {
        (!self.handle_tr64_is_int(value)) && (!self.handle_tr64_is_ref(value))
    }

    /// checks if a Tagref64 is an integer value
    pub fn handle_tr64_is_int(&self, value: APIHandleArg) -> bool {
        let opnd = value.v.as_tr64();
        (opnd & 0x7ff0000000000001u64) == 0x7ff0000000000001u64
    }

    /// checks if a Tagref64 is a reference value
    pub fn handle_tr64_is_ref(&self, value: APIHandleArg) -> bool {
        let opnd = value.v.as_tr64();
        (opnd & 0x7ff0000000000003u64) == 0x7ff0000000000002u64
    }

    /// unwraps a Tagref64 to a floating point value
    pub fn handle_tr64_to_fp(&self, value: APIHandleArg) -> APIHandleResult {
        let handle_id = self.next_id();
        self.new_handle(APIHandle {
            id: handle_id,
            v: APIHandleValue::Double(unsafe { std::mem::transmute(value.v.as_tr64()) })
        })
    }

    /// unwraps a Tagref64 to an integer value
    pub fn handle_tr64_to_int(&self, value: APIHandleArg) -> APIHandleResult {
        let handle_id = self.next_id();
        let opnd = value.v.as_tr64();
        self.new_handle(APIHandle {
            id: handle_id,
            v: APIHandleValue::Int(
                ((opnd & 0xffffffffffffeu64) >> 1) | ((opnd & 0x8000000000000000u64) >> 12),
                52
            )
        })
    }

    /// unwraps a Tagref64 to a reference value
    pub fn handle_tr64_to_ref(&self, value: APIHandleArg) -> APIHandleResult {
        let handle_id = self.next_id();
        let opnd = value.v.as_tr64();
        self.new_handle(APIHandle {
            id: handle_id,
            v: APIHandleValue::Ref(types::REF_VOID_TYPE.clone(), unsafe {
                Address::from_usize(
                    ((opnd & 0x7ffffffffff8u64) | u64_asr((opnd & 0x8000000000000000u64), 16)) as
                        usize
                )
            })
        })
    }

    /// unwraps a Tagref64 to its reference tag value
    pub fn handle_tr64_to_tag(&self, value: APIHandleArg) -> APIHandleResult {
        let handle_id = self.next_id();
        let opnd = value.v.as_tr64();
        self.new_handle(APIHandle {
            id: handle_id,
            v: APIHandleValue::Int(
                u64_asr((opnd & 0x000f800000000000u64), 46) | (u64_asr((opnd & 0x4), 2)),
                6
            )
        })
    }

    /// creates a Tagref64 handle from a floating point value
    pub fn handle_tr64_from_fp(&self, value: APIHandleArg) -> APIHandleResult {
        let handle_id = self.next_id();
        let double_bits = unsafe { std::mem::transmute(value.v.as_double()) };

        let result_bits = if value.v.as_double().is_nan() {
            double_bits & 0xfff8000000000000u64 | 0x0000000000000008u64
        } else {
            double_bits
        };

        self.new_handle(APIHandle {
            id: handle_id,
            v: APIHandleValue::TagRef64(result_bits)
        })
    }

    /// creates a Tagref64 handle from an integer value
    pub fn handle_tr64_from_int(&self, value: APIHandleArg) -> APIHandleResult {
        let handle_id = self.next_id();
        let opnd = value.v.as_int();
        self.new_handle(APIHandle {
            id: handle_id,
            v: APIHandleValue::TagRef64(
                0x7ff0000000000001u64 | ((opnd & 0x7ffffffffffffu64) << 1) |
                    ((opnd & 0x8000000000000u64) << 12)
            )
        })
    }

    /// creates a Tagref64 handle from a reference value and a tag
    pub fn handle_tr64_from_ref(&self, reff: APIHandleArg, tag: APIHandleArg) -> APIHandleResult {
        let handle_id = self.next_id();
        let (_, addr) = reff.v.as_ref();
        let addr_ = addr.as_usize() as u64;
        let tag_ = tag.v.as_int();
        self.new_handle(APIHandle {
            id: handle_id,
            v: APIHandleValue::TagRef64(
                0x7ff0000000000002u64 | (addr_ & 0x7ffffffffff8u64) |
                    ((addr_ & 0x800000000000u64) << 16) |
                    ((tag_ & 0x3eu64) << 46) | ((tag_ & 0x1) << 2)
            )
        })
    }
}
