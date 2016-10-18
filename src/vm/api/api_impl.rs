#![allow(unused_imports)]   // work in progress
#![allow(unused_variables)] // stubs
#![allow(dead_code)]        // stubs

/*!
 * This file contains the high-level implementation of the Mu API.
 *
 * Structs are written in idiomatic Rust code. The internal structures of these structs are
 * implementation-specific. Methods are defined using `impl`. 
 */

use std::os::raw::*;
use std::ptr;
use std::slice;
use std::ffi::CStr;
use std::ffi::CString;

use std::collections::HashMap;
use std::collections::HashSet;

use std::sync::Mutex;
use std::sync::RwLock;

use super::super::vm::VM;

use super::api_c::*;
use super::api_bridge::*;
//use super::deps::*;   // maybe it is better to import * here.
use super::irnodes::*;

use ast::bundle::*;
use ast::ir::*;
use ast::ptr::*;
use ast::types::*;

/**
 * Create a micro VM instance, and expose it as a C-visible `*mut CMuVM` pointer.
 *
 * NOTE: When used as an API (such as in tests), please use `mu::vm::api::mu_fastimpl_new` instead.
 *
 * This method is not part of the API defined by the Mu spec. It is used **when the client starts
 * the process and creates the micor VM**. For example, it is used if the client wants to build
 * boot images, or if the client implements most of its parts in C and onlu uses the micro VM as
 * the JIT compiler.
 *
 * The boot image itself should use `VM::resume_vm` to restore the saved the micro VM. There is no
 * need in the boot image itself to expose the `MuVM` structure to the trap handler. Trap handlers
 * only see `MuCtx`, and it is enough for most of the works.
 */
#[no_mangle]
pub extern fn mu_fastimpl_new() -> *mut CMuVM {
    info!("Creating Mu micro VM fast implementation instance...");

    let mvm = Box::new(MuVM::new());
    let mvm_ptr = Box::into_raw(mvm);

    debug!("The MuVM instance address: {:?}", mvm_ptr);

    let c_mvm = make_new_MuVM(mvm_ptr as *mut c_void);

    debug!("The C-visible CMuVM struct address: {:?}", c_mvm);

    c_mvm
}

pub struct MuVM {
    // The actual VM
    vm: VM,

    // Cache C strings. The C client expects `char*` from `name_of`. We assume the client won't
    // call `name_of` very often, so that we don't need to initialise this hashmap on startup.
    name_cache: Mutex<HashMap<MuID, CString>>,
}

pub struct MuCtx<'v> {
    /// ref to MuVM
    mvm: &'v MuVM,

    /// Point to the C-visible CMuCtx so that `close_context` can deallocate itself.
    c_struct: *mut CMuCtx,
}

pub struct MuIRBuilder<'v> {
    /// ref to MuVM
    mvm: &'v MuVM,

    /// Point to the C-visible CMuIRBuilder so that `load` and `abort` can deallocate itself.
    c_struct: *mut CMuIRBuilder,

    /// Map IDs to names. Items are inserted during `gen_sym`. MuIRBuilder is supposed to be used
    /// by one thread, so there is no need for locking.
    id_name_map: HashMap<MuID, MuName>,

    /// The "trantient bundle" includes everything being built here.
    bundle: TrantientBundle,
}

/// A trantient bundle, i.e. the bundle being built, but not yet loaded into the MuVM.
#[derive(Default)]
pub struct TrantientBundle {
    types: Vec<Box<NodeType>>,
    sigs: Vec<Box<NodeFuncSig>>,
    consts: Vec<Box<NodeConst>>,
    globals: Vec<Box<NodeGlobalCell>>,
    funcs: Vec<Box<NodeFunc>>,
    expfuncs: Vec<Box<NodeExpFunc>>,
    funcvers: Vec<Box<NodeFuncVer>>,
    bbs: Vec<Box<NodeBB>>,
    insts: Vec<Box<NodeInst>>,
    dest_clauses: Vec<Box<NodeDestClause>>,
    exc_clauses: Vec<Box<NodeExcClause>>,
    ka_clauses: Vec<Box<NodeKeepaliveClause>>,
}

/**
 * Implement the methods of MuVM. Most methods implement the C-level methods, and others are
 * rust-level helpers. Most methods are forwarded to the underlying `VM.*` methods.
 */
impl MuVM {
    /**
     * Create a new micro VM instance from scratch.
     */
    pub fn new() -> MuVM {
        MuVM {
            vm: VM::new(),
            // Cache C strings. The C client expects `char*` from `name_of`. We assume the client
            // won't call `name_of` very often, so that we don't need to initialise this hashmap on
            // startup.
            //
            // RwLock won't work because Rust will not let me release the lock after reading
            // because other threads will remove that element from the cache, even though I only
            // monotonically add elements into the `name_cache`. I can't upgrade the lock from read
            // lock to write lock, otherwise it will deadlock.
            name_cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn new_context(&self) -> *mut CMuCtx {
        info!("Creating MuCtx...");

        let ctx = Box::new(MuCtx {
            mvm: self,
            c_struct: ptr::null_mut(),
        });

        let ctx_ptr = Box::into_raw(ctx);

        debug!("The MuCtx address: {:?}", ctx_ptr);

        let cctx = make_new_MuCtx(ctx_ptr as *mut c_void);

        debug!("The C-visible CMuCtx struct address: {:?}", cctx);

        unsafe{ (*ctx_ptr).c_struct = cctx; }

        cctx
    }

    pub fn id_of(&self, name: MuName) -> MuID {
        self.vm.id_of_by_refstring(&name)
    }

    pub fn name_of(&self, id: MuID) -> CMuCString {
        let mut map = self.name_cache.lock().unwrap();

        let cname = map.entry(id).or_insert_with(|| {
            let rustname = self.vm.name_of(id);
            CString::new(rustname).unwrap()
        });

        cname.as_ptr()
    }

    pub fn set_trap_handler(&self, trap_handler: CMuTrapHandler, userdata: CMuCPtr) {
        panic!("Not implemented")
    }

}

impl<'v> MuCtx<'v> {
    #[inline(always)]
    fn get_mvm(&mut self) -> &MuVM {
        self.mvm
        //unsafe { &mut *self.mvm }
    }

    pub fn id_of(&mut self, name: MuName) -> MuID {
        self.get_mvm().id_of(name)
    }

    pub fn name_of(&mut self, id: MuID) -> CMuCString {
        self.get_mvm().name_of(id)
    }

    fn deallocate(&mut self) {
        let c_struct = self.c_struct;
        let ctx_ptr = self as *mut MuCtx;
        debug!("Deallocating MuCtx {:?} and CMuCtx {:?}...", ctx_ptr, c_struct);
        unsafe {
            Box::from_raw(c_struct);
            Box::from_raw(ctx_ptr);
        }
    }

    pub fn close_context(&mut self) {
        info!("Closing MuCtx...");
        self.deallocate();
    }

    pub fn load_bundle(&mut self, buf: &[c_char]) {
        panic!("The fast implementation does not support the text form.")
    }

    pub fn load_hail(&mut self, buf: &[c_char]) {
        panic!("The fast implementation does not support the text form.")
    }

    pub fn handle_from_sint8(&mut self, num: i8, len: c_int) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn handle_from_uint8(&mut self, num: u8, len: c_int) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn handle_from_sint16(&mut self, num: i16, len: c_int) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn handle_from_uint16(&mut self, num: u16, len: c_int) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn handle_from_sint32(&mut self, num: i32, len: c_int) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn handle_from_uint32(&mut self, num: u32, len: c_int) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn handle_from_sint64(&mut self, num: i64, len: c_int) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn handle_from_uint64(&mut self, num: u64, len: c_int) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn handle_from_uint64s(&mut self, nums: &[u64], len: c_int) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn handle_from_float(&mut self, num: f32) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn handle_from_double(&mut self, num: f64) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn handle_from_ptr(&mut self, mu_type: MuID, ptr: CMuCPtr) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn handle_from_fp(&mut self, mu_type: MuID, fp: CMuCFP) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn handle_to_sint8(&mut self, opnd: &APIMuValue) -> i8 {
        panic!("Not implemented")
    }

    pub fn handle_to_uint8(&mut self, opnd: &APIMuValue) -> u8 {
        panic!("Not implemented")
    }

    pub fn handle_to_sint16(&mut self, opnd: &APIMuValue) -> i16 {
        panic!("Not implemented")
    }

    pub fn handle_to_uint16(&mut self, opnd: &APIMuValue) -> u16 {
        panic!("Not implemented")
    }

    pub fn handle_to_sint32(&mut self, opnd: &APIMuValue) -> i32 {
        panic!("Not implemented")
    }

    pub fn handle_to_uint32(&mut self, opnd: &APIMuValue) -> u32 {
        panic!("Not implemented")
    }

    pub fn handle_to_sint64(&mut self, opnd: &APIMuValue) -> i64 {
        panic!("Not implemented")
    }

    pub fn handle_to_uint64(&mut self, opnd: &APIMuValue) -> u64 {
        panic!("Not implemented")
    }

    pub fn handle_to_float(&mut self, opnd: &APIMuValue) -> f32 {
        panic!("Not implemented")
    }

    pub fn handle_to_double(&mut self, opnd: &APIMuValue) -> f64 {
        panic!("Not implemented")
    }

    pub fn handle_to_ptr(&mut self, opnd: &APIMuValue) -> CMuCPtr {
        panic!("Not implemented")
    }

    pub fn handle_to_fp(&mut self, opnd: &APIMuValue) -> CMuCFP {
        panic!("Not implemented")
    }

    pub fn handle_from_const(&mut self, id: MuID) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn handle_from_global(&mut self, id: MuID) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn handle_from_func(&mut self, id: MuID) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn handle_from_expose(&mut self, id: MuID) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn delete_value(&mut self, opnd: &APIMuValue) {
        panic!("Not implemented")
    }

    pub fn ref_eq(&mut self, lhs: &APIMuValue, rhs: &APIMuValue) -> bool {
        panic!("Not implemented")
    }

    pub fn ref_ult(&mut self, lhs: &APIMuValue, rhs: &APIMuValue) -> bool {
        panic!("Not implemented")
    }

    pub fn extract_value(&mut self, str: &APIMuValue, index: c_int) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn insert_value(&mut self, str: &APIMuValue, index: c_int, newval: &APIMuValue) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn extract_element(&mut self, str: &APIMuValue, index: &APIMuValue) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn insert_element(&mut self, str: &APIMuValue, index: &APIMuValue, newval: &APIMuValue) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn new_fixed(&mut self, mu_type: MuID) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn new_hybrid(&mut self, mu_type: MuID, length: &APIMuValue) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn refcast(&mut self, opnd: &APIMuValue, new_type: MuID) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn get_iref(&mut self, opnd: &APIMuValue) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn get_field_iref(&mut self, opnd: &APIMuValue, field: c_int) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn get_elem_iref(&mut self, opnd: &APIMuValue, index: &APIMuValue) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn shift_iref(&mut self, opnd: &APIMuValue, offset: &APIMuValue) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn get_var_part_iref(&mut self, opnd: &APIMuValue) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn load(&mut self, ord: CMuMemOrd, loc: &APIMuValue) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn store(&mut self, ord: CMuMemOrd, loc: &APIMuValue, newval: &APIMuValue) {
        panic!("Not implemented")
    }

    pub fn cmpxchg(&mut self, ord_succ: CMuMemOrd, ord_fail: CMuMemOrd, weak: bool, loc: &APIMuValue, expected: &APIMuValue, desired: &APIMuValue, is_succ: *mut CMuBool) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn atomicrmw(&mut self, ord: CMuMemOrd, op: CMuAtomicRMWOptr, loc: &APIMuValue, opnd: &APIMuValue) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn fence(&mut self, ord: CMuMemOrd) {
        panic!("Not implemented")
    }

    pub fn new_stack(&mut self, func: &APIMuValue) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn new_thread_nor(&mut self, stack: &APIMuValue, threadlocal: Option<&APIMuValue>, vals: Vec<&APIMuValue>) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn new_thread_exc(&mut self, stack: &APIMuValue, threadlocal: Option<&APIMuValue>, exc: &APIMuValue) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn kill_stack(&mut self, stack: &APIMuValue) {
        panic!("Not implemented")
    }

    pub fn set_threadlocal(&mut self, thread: &APIMuValue, threadlocal: &APIMuValue) {
        panic!("Not implemented")
    }

    pub fn get_threadlocal(&mut self, thread: &APIMuValue) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn new_cursor(&mut self, stack: &APIMuValue) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn next_frame(&mut self, cursor: &APIMuValue) {
        panic!("Not implemented")
    }

    pub fn copy_cursor(&mut self, cursor: &APIMuValue) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn close_cursor(&mut self, cursor: &APIMuValue) {
        panic!("Not implemented")
    }

    pub fn cur_func(&mut self, cursor: &APIMuValue) -> MuID {
        panic!("Not implemented")
    }

    pub fn cur_func_ver(&mut self, cursor: &APIMuValue) -> MuID {
        panic!("Not implemented")
    }

    pub fn cur_inst(&mut self, cursor: &APIMuValue) -> MuID {
        panic!("Not implemented")
    }

    pub fn dump_keepalives(&mut self, cursor: &APIMuValue, results: *mut CMuValue) {
        panic!("Not implemented")
    }

    pub fn pop_frames_to(&mut self, cursor: &APIMuValue) {
        panic!("Not implemented")
    }

    pub fn push_frame(&mut self, stack: &APIMuValue, func: &APIMuValue) {
        panic!("Not implemented")
    }

    pub fn tr64_is_fp(&mut self, value: &APIMuValue) -> bool {
        panic!("Not implemented")
    }

    pub fn tr64_is_int(&mut self, value: &APIMuValue) -> bool {
        panic!("Not implemented")
    }

    pub fn tr64_is_ref(&mut self, value: &APIMuValue) -> bool {
        panic!("Not implemented")
    }

    pub fn tr64_to_fp(&mut self, value: &APIMuValue) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn tr64_to_int(&mut self, value: &APIMuValue) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn tr64_to_ref(&mut self, value: &APIMuValue) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn tr64_to_tag(&mut self, value: &APIMuValue) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn tr64_from_fp(&mut self, value: &APIMuValue) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn tr64_from_int(&mut self, value: &APIMuValue) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn tr64_from_ref(&mut self, reff: &APIMuValue, tag: &APIMuValue) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn enable_watchpoint(&mut self, wpid: CMuWPID) {
        panic!("Not implemented")
    }

    pub fn disable_watchpoint(&mut self, wpid: CMuWPID) {
        panic!("Not implemented")
    }

    pub fn pin(&mut self, loc: &APIMuValue) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn unpin(&mut self, loc: &APIMuValue) {
        panic!("Not implemented")
    }

    pub fn get_addr(&mut self, loc: &APIMuValue) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn expose(&mut self, func: &APIMuValue, call_conv: CMuCallConv, cookie: &APIMuValue) -> *const APIMuValue {
        panic!("Not implemented")
    }

    pub fn unexpose(&mut self, call_conv: CMuCallConv, value: &APIMuValue) {
        panic!("Not implemented")
    }

    pub fn new_ir_builder(&mut self) -> *mut CMuIRBuilder {
        info!("Creating MuIRBuilder...");

        let b: Box<MuIRBuilder> = Box::new(MuIRBuilder {
            mvm: self.mvm,
            c_struct: ptr::null_mut(),
            id_name_map: Default::default(),
            bundle: Default::default(),
        });

        let b_ptr = Box::into_raw(b);

        debug!("The MuIRBuilder address: {:?}", b_ptr);

        let cb = make_new_MuIRBuilder(b_ptr as *mut c_void);

        debug!("The C-visible CMuIRBuilder struct address: {:?}", cb);

        unsafe{ (*b_ptr).c_struct = cb; }

        cb
    }

    pub fn make_boot_image(&mut self, whitelist: Vec<MuID>, primordial_func: Option<&APIMuValue>, primordial_stack: Option<&APIMuValue>, primordial_threadlocal: Option<&APIMuValue>, sym_fields: Vec<&APIMuValue>, sym_strings: Vec<String>, reloc_fields: Vec<&APIMuValue>, reloc_strings: Vec<String>, output_file: String) {
        panic!("Not implemented")
    }

}

impl<'v> MuIRBuilder<'v> {
    #[inline(always)]
    fn get_mvm(&mut self) -> &MuVM {
        self.mvm
    }

    #[inline(always)]
    fn get_vm(&mut self) -> &VM {
        &self.get_mvm().vm
    }

    #[inline(always)]
    fn next_id(&mut self) -> MuID {
        self.get_vm().next_id()
    }

    fn deallocate(&mut self) {
        let c_struct = self.c_struct;
        let b_ptr = self as *mut MuIRBuilder;
        debug!("Deallocating MuIRBuilder {:?} and CMuIRBuilder {:?}...", b_ptr, c_struct);
        unsafe {
            Box::from_raw(c_struct);
            Box::from_raw(b_ptr);
        }
    }

    /// Get the Mu name of the `id`. This will consume the entry in the `id_name_map`. For this
    /// reason, this function is only called when the actual MuEntity that has this ID is created
    /// (such as `new_type_int`).
    fn consume_name_of(&mut self, id: MuID) -> Option<MuName> {
        self.id_name_map.remove(&id)
    }

    pub fn load(&mut self) {
        panic!("Please implement bundle loading before deallocating itself.");
        self.deallocate();
    }

    pub fn abort(&mut self) {
        info!("Aborting boot image building...");
        self.deallocate();
    }

    pub fn gen_sym(&mut self, name: Option<String>) -> MuID {
        let my_id = self.next_id();

        debug!("gen_sym({:?}) -> {}", name, my_id);

        match name {
            None => {},
            Some(the_name) => {
                let old = self.id_name_map.insert(my_id, the_name);
                debug_assert!(old.is_none(), "ID already exists: {}, new name: {}, old name: {}",
                my_id, self.id_name_map.get(&my_id).unwrap(), old.unwrap());
            },
        };

        my_id
    }

    pub fn new_type_int(&mut self, id: MuID, len: c_int) {
        self.bundle.types.push(Box::new(NodeType::TypeInt { id: id, len: len }));


//        let maybe_name = self.consume_name_of(id);
//        let pty = P(MuType {
//            hdr: MuEntityHeader {
//                id: id,
//                name: RwLock::new(maybe_name),
//            },
//            v: MuType_::Int(len as usize),
//        });
//
//        self.bundle.types.push(pty);
    }

    pub fn new_type_float(&mut self, id: MuID) {
        panic!("Not implemented")
    }

    pub fn new_type_double(&mut self, id: MuID) {
        panic!("Not implemented")
    }

    pub fn new_type_uptr(&mut self, id: MuID, ty: MuID) {
        self.bundle.types.push(Box::new(NodeType::TypeUPtr{ id: id,
            ty: ty }));
    }

    pub fn new_type_ufuncptr(&mut self, id: MuID, sig: MuID) {
        panic!("Not implemented")
    }

    pub fn new_type_struct(&mut self, id: MuID, fieldtys: Vec<MuID>) {
        self.bundle.types.push(Box::new(NodeType::TypeStruct { id: id,
            fieldtys: fieldtys }));
    }

    pub fn new_type_hybrid(&mut self, id: MuID, fixedtys: Vec<MuID>, varty: MuID) {
        panic!("Not implemented")
    }

    pub fn new_type_array(&mut self, id: MuID, elemty: MuID, len: u64) {
        panic!("Not implemented")
    }

    pub fn new_type_vector(&mut self, id: MuID, elemty: MuID, len: u64) {
        panic!("Not implemented")
    }

    pub fn new_type_void(&mut self, id: MuID) {
        panic!("Not implemented")
    }

    pub fn new_type_ref(&mut self, id: MuID, ty: MuID) {
        panic!("Not implemented")
    }

    pub fn new_type_iref(&mut self, id: MuID, ty: MuID) {
        panic!("Not implemented")
    }

    pub fn new_type_weakref(&mut self, id: MuID, ty: MuID) {
        panic!("Not implemented")
    }

    pub fn new_type_funcref(&mut self, id: MuID, sig: MuID) {
        panic!("Not implemented")
    }

    pub fn new_type_tagref64(&mut self, id: MuID) {
        panic!("Not implemented")
    }

    pub fn new_type_threadref(&mut self, id: MuID) {
        panic!("Not implemented")
    }

    pub fn new_type_stackref(&mut self, id: MuID) {
        panic!("Not implemented")
    }

    pub fn new_type_framecursorref(&mut self, id: MuID) {
        panic!("Not implemented")
    }

    pub fn new_type_irbuilderref(&mut self, id: MuID) {
        panic!("Not implemented")
    }

    pub fn new_funcsig(&mut self, id: MuID, paramtys: Vec<MuID>, rettys: Vec<MuID>) {
        self.bundle.sigs.push(Box::new(NodeFuncSig { id: id,
            paramtys: paramtys, rettys: rettys }));
    }

    pub fn new_const_int(&mut self, id: MuID, ty: MuID, value: u64) {
        self.bundle.consts.push(Box::new(NodeConst::ConstInt { id: id,
            ty: ty, value: value }));
    }

    pub fn new_const_int_ex(&mut self, id: MuID, ty: MuID, values: &[u64]) {
        panic!("Not implemented")
    }

    pub fn new_const_float(&mut self, id: MuID, ty: MuID, value: f32) {
        panic!("Not implemented")
    }

    pub fn new_const_double(&mut self, id: MuID, ty: MuID, value: f64) {
        panic!("Not implemented")
    }

    pub fn new_const_null(&mut self, id: MuID, ty: MuID) {
        panic!("Not implemented")
    }

    pub fn new_const_seq(&mut self, id: MuID, ty: MuID, elems: Vec<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_const_extern(&mut self, id: MuID, ty: MuID, symbol: String) {
        panic!("Not implemented")
    }

    pub fn new_global_cell(&mut self, id: MuID, ty: MuID) {
        self.bundle.globals.push(Box::new(NodeGlobalCell { id: id,
            ty: ty }));
    }

    pub fn new_func(&mut self, id: MuID, sig: MuID) {
        self.bundle.funcs.push(Box::new(NodeFunc { id: id,
            sig: sig }));
    }

    pub fn new_exp_func(&mut self, id: MuID, func: MuID, callconv: CMuCallConv, cookie: MuID) {
        panic!("Not implemented")
    }

    pub fn new_func_ver(&mut self, id: MuID, func: MuID, bbs: Vec<MuID>) {
        self.bundle.funcvers.push(Box::new(NodeFuncVer { id: id,
            func: func, bbs: bbs }));
    }

    pub fn new_bb(&mut self, id: MuID, nor_param_ids: Vec<MuID>, nor_param_types: Vec<MuID>, exc_param_id: Option<MuID>, insts: Vec<MuID>) {
        self.bundle.bbs.push(Box::new(NodeBB { id: id,
            norParamIDs: nor_param_ids, norParamTys: nor_param_types,
            excParamID: exc_param_id, insts: insts }));
    }

    pub fn new_dest_clause(&mut self, id: MuID, dest: MuID, vars: Vec<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_exc_clause(&mut self, id: MuID, nor: MuID, exc: MuID) {
        panic!("Not implemented")
    }

    pub fn new_keepalive_clause(&mut self, id: MuID, vars: Vec<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_csc_ret_with(&mut self, id: MuID, rettys: Vec<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_csc_kill_old(&mut self, id: MuID) {
        panic!("Not implemented")
    }

    pub fn new_nsc_pass_values(&mut self, id: MuID, tys: Vec<MuID>, vars: Vec<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_nsc_throw_exc(&mut self, id: MuID, exc: MuID) {
        panic!("Not implemented")
    }

    pub fn new_binop(&mut self, id: MuID, result_id: MuID, optr: CMuBinOptr, ty: MuID, opnd1: MuID, opnd2: MuID, exc_clause: Option<MuID>) {
        self.bundle.insts.push(Box::new(NodeInst::NodeBinOp {
            id: id, resultID: result_id, statusResultIDs: vec![],
            optr: optr, flags: 0, ty: ty, opnd1: opnd1, opnd2: opnd2,
            excClause: exc_clause}))
    }

    pub fn new_binop_with_status(&mut self, id: MuID, result_id: MuID, status_result_ids: Vec<MuID>, optr: CMuBinOptr, status_flags: CMuBinOpStatus, ty: MuID, opnd1: MuID, opnd2: MuID, exc_clause: Option<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_cmp(&mut self, id: MuID, result_id: MuID, optr: CMuCmpOptr, ty: MuID, opnd1: MuID, opnd2: MuID) {
        panic!("Not implemented")
    }

    pub fn new_conv(&mut self, id: MuID, result_id: MuID, optr: CMuConvOptr, from_ty: MuID, to_ty: MuID, opnd: MuID) {
        panic!("Not implemented")
    }

    pub fn new_select(&mut self, id: MuID, result_id: MuID, cond_ty: MuID, opnd_ty: MuID, cond: MuID, if_true: MuID, if_false: MuID) {
        panic!("Not implemented")
    }

    pub fn new_branch(&mut self, id: MuID, dest: MuID) {
        panic!("Not implemented")
    }

    pub fn new_branch2(&mut self, id: MuID, cond: MuID, if_true: MuID, if_false: MuID) {
        panic!("Not implemented")
    }

    pub fn new_switch(&mut self, id: MuID, opnd_ty: MuID, opnd: MuID, default_dest: MuID, cases: Vec<MuID>, dests: Vec<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_call(&mut self, id: MuID, result_ids: Vec<MuID>, sig: MuID, callee: MuID, args: Vec<MuID>, exc_clause: Option<MuID>, keepalive_clause: Option<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_tailcall(&mut self, id: MuID, sig: MuID, callee: MuID, args: Vec<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_ret(&mut self, id: MuID, rvs: Vec<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_throw(&mut self, id: MuID, exc: MuID) {
        panic!("Not implemented")
    }

    pub fn new_extractvalue(&mut self, id: MuID, result_id: MuID, strty: MuID, index: c_int, opnd: MuID) {
        panic!("Not implemented")
    }

    pub fn new_insertvalue(&mut self, id: MuID, result_id: MuID, strty: MuID, index: c_int, opnd: MuID, newval: MuID) {
        panic!("Not implemented")
    }

    pub fn new_extractelement(&mut self, id: MuID, result_id: MuID, seqty: MuID, indty: MuID, opnd: MuID, index: MuID) {
        panic!("Not implemented")
    }

    pub fn new_insertelement(&mut self, id: MuID, result_id: MuID, seqty: MuID, indty: MuID, opnd: MuID, index: MuID, newval: MuID) {
        panic!("Not implemented")
    }

    pub fn new_shufflevector(&mut self, id: MuID, result_id: MuID, vecty: MuID, maskty: MuID, vec1: MuID, vec2: MuID, mask: MuID) {
        panic!("Not implemented")
    }

    pub fn new_new(&mut self, id: MuID, result_id: MuID, allocty: MuID, exc_clause: Option<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_newhybrid(&mut self, id: MuID, result_id: MuID, allocty: MuID, lenty: MuID, length: MuID, exc_clause: Option<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_alloca(&mut self, id: MuID, result_id: MuID, allocty: MuID, exc_clause: Option<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_allocahybrid(&mut self, id: MuID, result_id: MuID, allocty: MuID, lenty: MuID, length: MuID, exc_clause: Option<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_getiref(&mut self, id: MuID, result_id: MuID, refty: MuID, opnd: MuID) {
        panic!("Not implemented")
    }

    pub fn new_getfieldiref(&mut self, id: MuID, result_id: MuID, is_ptr: bool, refty: MuID, index: c_int, opnd: MuID) {
        panic!("Not implemented")
    }

    pub fn new_getelemiref(&mut self, id: MuID, result_id: MuID, is_ptr: bool, refty: MuID, indty: MuID, opnd: MuID, index: MuID) {
        panic!("Not implemented")
    }

    pub fn new_shiftiref(&mut self, id: MuID, result_id: MuID, is_ptr: bool, refty: MuID, offty: MuID, opnd: MuID, offset: MuID) {
        panic!("Not implemented")
    }

    pub fn new_getvarpartiref(&mut self, id: MuID, result_id: MuID, is_ptr: bool, refty: MuID, opnd: MuID) {
        panic!("Not implemented")
    }

    pub fn new_load(&mut self, id: MuID, result_id: MuID, is_ptr: bool, ord: CMuMemOrd, refty: MuID, loc: MuID, exc_clause: Option<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_store(&mut self, id: MuID, is_ptr: bool, ord: CMuMemOrd, refty: MuID, loc: MuID, newval: MuID, exc_clause: Option<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_cmpxchg(&mut self, id: MuID, value_result_id: MuID, succ_result_id: MuID, is_ptr: bool, is_weak: bool, ord_succ: CMuMemOrd, ord_fail: CMuMemOrd, refty: MuID, loc: MuID, expected: MuID, desired: MuID, exc_clause: Option<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_atomicrmw(&mut self, id: MuID, result_id: MuID, is_ptr: bool, ord: CMuMemOrd, optr: CMuAtomicRMWOptr, ref_ty: MuID, loc: MuID, opnd: MuID, exc_clause: Option<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_fence(&mut self, id: MuID, ord: CMuMemOrd) {
        panic!("Not implemented")
    }

    pub fn new_trap(&mut self, id: MuID, result_ids: Vec<MuID>, rettys: Vec<MuID>, exc_clause: Option<MuID>, keepalive_clause: Option<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_watchpoint(&mut self, id: MuID, wpid: CMuWPID, result_ids: Vec<MuID>, rettys: Vec<MuID>, dis: MuID, ena: MuID, exc: Option<MuID>, keepalive_clause: Option<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_wpbranch(&mut self, id: MuID, wpid: CMuWPID, dis: MuID, ena: MuID) {
        panic!("Not implemented")
    }

    pub fn new_ccall(&mut self, id: MuID, result_ids: Vec<MuID>, callconv: CMuCallConv, callee_ty: MuID, sig: MuID, callee: MuID, args: Vec<MuID>, exc_clause: Option<MuID>, keepalive_clause: Option<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_newthread(&mut self, id: MuID, result_id: MuID, stack: MuID, threadlocal: Option<MuID>, new_stack_clause: MuID, exc_clause: Option<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_swapstack(&mut self, id: MuID, result_ids: Vec<MuID>, swappee: MuID, cur_stack_clause: MuID, new_stack_clause: MuID, exc_clause: Option<MuID>, keepalive_clause: Option<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_comminst(&mut self, id: MuID, result_ids: Vec<MuID>, opcode: CMuCommInst, flags: &[CMuFlag], tys: Vec<MuID>, sigs: Vec<MuID>, args: Vec<MuID>, exc_clause: Option<MuID>, keepalive_clause: Option<MuID>) {
        panic!("Not implemented")
    }

}
