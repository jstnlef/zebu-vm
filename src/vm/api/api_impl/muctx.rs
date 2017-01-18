use super::common::*;
use utils::Address;
//use std::os::raw::c_void;

pub struct MuCtx {
    /// ref to MuVM
    mvm: *const MuVM,

    /// Point to the C-visible CMuCtx so that `close_context` can deallocate itself.
    pub c_struct: *mut CMuCtx,
}

impl MuCtx {
    pub fn new(mvm: *const MuVM) -> Box<MuCtx> {
        Box::new(MuCtx {
            mvm: mvm,
            c_struct: ptr::null_mut(),
        })
    }

    #[inline(always)]
    fn get_mvm(&mut self) -> &MuVM {
        //self.mvm
        unsafe { & *self.mvm }
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

    pub fn handle_from_sint8(&mut self, num: i8, len: c_int) -> *const APIHandle {
        prepare_handle((self.get_mvm().vm.handle_from_sint8(num, len as usize)))
    }

    pub fn handle_from_uint8(&mut self, num: u8, len: c_int) -> *const APIHandle {
        prepare_handle((self.get_mvm().vm.handle_from_uint8(num, len as usize)))
    }

    pub fn handle_from_sint16(&mut self, num: i16, len: c_int) -> *const APIHandle {
        prepare_handle((self.get_mvm().vm.handle_from_sint16(num, len as usize)))
    }

    pub fn handle_from_uint16(&mut self, num: u16, len: c_int) -> *const APIHandle {
        prepare_handle((self.get_mvm().vm.handle_from_uint16(num, len as usize)))
    }

    pub fn handle_from_sint32(&mut self, num: i32, len: c_int) -> *const APIHandle {
        prepare_handle((self.get_mvm().vm.handle_from_sint32(num, len as usize)))
    }

    pub fn handle_from_uint32(&mut self, num: u32, len: c_int) -> *const APIHandle {
        prepare_handle((self.get_mvm().vm.handle_from_uint32(num, len as usize)))
    }

    pub fn handle_from_sint64(&mut self, num: i64, len: c_int) -> *const APIHandle {
        prepare_handle((self.get_mvm().vm.handle_from_sint64(num, len as usize)))
    }

    pub fn handle_from_uint64(&mut self, num: u64, len: c_int) -> *const APIHandle {
        prepare_handle((self.get_mvm().vm.handle_from_uint64(num, len as usize)))
    }

    pub fn handle_from_uint64s(&mut self, nums: &[u64], len: c_int) -> *const APIHandle {
        panic!("Zebu doesnt implement int with length larger than 64bits")
    }

    pub fn handle_from_float(&mut self, num: f32) -> *const APIHandle {
        prepare_handle((self.get_mvm().vm.handle_from_float(num)))
    }

    pub fn handle_from_double(&mut self, num: f64) -> *const APIHandle {
        prepare_handle((self.get_mvm().vm.handle_from_double(num)))
    }

    pub fn handle_from_ptr(&mut self, mu_type: MuID, ptr: CMuCPtr) -> *const APIHandle {
        let addr = Address::from_mut_ptr(ptr);

        prepare_handle((self.get_mvm().vm.handle_from_uptr(mu_type, addr)))
    }

    pub fn handle_from_fp(&mut self, mu_type: MuID, fp: CMuCFP) -> *const APIHandle {
        let addr = Address::from_mut_ptr(fp);

        prepare_handle((self.get_mvm().vm.handle_from_ufp(mu_type, addr)))
    }

    pub fn handle_to_sint8(&mut self, opnd: &APIHandle) -> i8 {
        self.get_mvm().vm.handle_to_sint8(opnd)
    }

    pub fn handle_to_uint8(&mut self, opnd: &APIHandle) -> u8 {
        self.get_mvm().vm.handle_to_uint8(opnd)
    }

    pub fn handle_to_sint16(&mut self, opnd: &APIHandle) -> i16 {
        self.get_mvm().vm.handle_to_sint16(opnd)
    }

    pub fn handle_to_uint16(&mut self, opnd: &APIHandle) -> u16 {
        self.get_mvm().vm.handle_to_uint16(opnd)
    }

    pub fn handle_to_sint32(&mut self, opnd: &APIHandle) -> i32 {
        self.get_mvm().vm.handle_to_sint32(opnd)
    }

    pub fn handle_to_uint32(&mut self, opnd: &APIHandle) -> u32 {
        self.get_mvm().vm.handle_to_uint32(opnd)
    }

    pub fn handle_to_sint64(&mut self, opnd: &APIHandle) -> i64 {
        self.get_mvm().vm.handle_to_sint64(opnd)
    }

    pub fn handle_to_uint64(&mut self, opnd: &APIHandle) -> u64 {
        self.get_mvm().vm.handle_to_uint64(opnd)
    }

    pub fn handle_to_float(&mut self, opnd: &APIHandle) -> f32 {
        self.get_mvm().vm.handle_to_float(opnd)
    }

    pub fn handle_to_double(&mut self, opnd: &APIHandle) -> f64 {
        self.get_mvm().vm.handle_to_double(opnd)
    }

    pub fn handle_to_ptr(&mut self, opnd: &APIHandle) -> CMuCPtr {
        self.get_mvm().vm.handle_to_uptr(opnd).to_ptr_mut()
    }

    pub fn handle_to_fp(&mut self, opnd: &APIHandle) -> CMuCFP {
        self.get_mvm().vm.handle_to_ufp(opnd).to_ptr_mut()
    }

    pub fn handle_from_const(&mut self, id: MuID) -> *const APIHandle {
        prepare_handle(self.get_mvm().vm.handle_from_const(id))
    }

    pub fn handle_from_global(&mut self, id: MuID) -> *const APIHandle {
        prepare_handle(self.get_mvm().vm.handle_from_global(id))
    }

    pub fn handle_from_func(&mut self, id: MuID) -> *const APIHandle {
        panic!("Not implemented")
    }

    pub fn handle_from_expose(&mut self, id: MuID) -> *const APIHandle {
        panic!("Not implemented")
    }

    pub fn delete_value(&mut self, opnd: &APIHandle) {
        delete_handle(opnd);
    }

    pub fn ref_eq(&mut self, lhs: &APIHandle, rhs: &APIHandle) -> bool {
        panic!("Not implemented")
    }

    pub fn ref_ult(&mut self, lhs: &APIHandle, rhs: &APIHandle) -> bool {
        panic!("Not implemented")
    }

    pub fn extract_value(&mut self, str: &APIHandle, index: c_int) -> *const APIHandle {
        panic!("Not implemented")
    }

    pub fn insert_value(&mut self, str: &APIHandle, index: c_int, newval: &APIHandle) -> *const APIHandle {
        panic!("Not implemented")
    }

    pub fn extract_element(&mut self, str: &APIHandle, index: &APIHandle) -> *const APIHandle {
        panic!("Not implemented")
    }

    pub fn insert_element(&mut self, str: &APIHandle, index: &APIHandle, newval: &APIHandle) -> *const APIHandle {
        panic!("Not implemented")
    }

    pub fn new_fixed(&mut self, mu_type: MuID) -> *const APIHandle {
        prepare_handle(self.get_mvm().vm.new_fixed(mu_type))
    }

    pub fn new_hybrid(&mut self, mu_type: MuID, length: &APIHandle) -> *const APIHandle {
        prepare_handle(self.get_mvm().vm.new_hybrid(mu_type, length))
    }

    pub fn refcast(&mut self, opnd: &APIHandle, new_type: MuID) -> *const APIHandle {
        panic!("Not implemented")
    }

    pub fn get_iref(&mut self, opnd: &APIHandle) -> *const APIHandle {
        prepare_handle(self.get_mvm().vm.handle_get_iref(opnd))
    }

    pub fn get_field_iref(&mut self, opnd: &APIHandle, field: c_int) -> *const APIHandle {
        prepare_handle(self.get_mvm().vm.handle_get_field_iref(opnd, field as usize))
    }

    pub fn get_elem_iref(&mut self, opnd: &APIHandle, index: &APIHandle) -> *const APIHandle {
        prepare_handle(self.get_mvm().vm.handle_get_elem_iref(opnd, index))
    }

    pub fn shift_iref(&mut self, opnd: &APIHandle, offset: &APIHandle) -> *const APIHandle {
        prepare_handle(self.get_mvm().vm.handle_shift_iref(opnd, offset))
    }

    pub fn get_var_part_iref(&mut self, opnd: &APIHandle) -> *const APIHandle {
        prepare_handle(self.get_mvm().vm.handle_get_var_part_iref(opnd))
    }

    pub fn load(&mut self, ord: CMuMemOrd, loc: &APIHandle) -> *const APIHandle {
        prepare_handle(self.get_mvm().vm.handle_load(impl_memorder(ord), loc))
    }

    pub fn store(&mut self, ord: CMuMemOrd, loc: &APIHandle, newval: &APIHandle) {
        self.get_mvm().vm.handle_store(impl_memorder(ord), loc, newval);
    }

    pub fn cmpxchg(&mut self, ord_succ: CMuMemOrd, ord_fail: CMuMemOrd, weak: bool, loc: &APIHandle, expected: &APIHandle, desired: &APIHandle, is_succ: *mut CMuBool) -> *const APIHandle {
        panic!("Not implemented")
    }

    pub fn atomicrmw(&mut self, ord: CMuMemOrd, op: CMuAtomicRMWOptr, loc: &APIHandle, opnd: &APIHandle) -> *const APIHandle {
        panic!("Not implemented")
    }

    pub fn fence(&mut self, ord: CMuMemOrd) {
        panic!("Not implemented")
    }

    pub fn new_stack(&mut self, func: &APIHandle) -> *const APIHandle {
        panic!("Not implemented")
    }

    pub fn new_thread_nor(&mut self, stack: &APIHandle, threadlocal: Option<&APIHandle>, vals: Vec<&APIHandle>) -> *const APIHandle {
        panic!("Not implemented")
    }

    pub fn new_thread_exc(&mut self, stack: &APIHandle, threadlocal: Option<&APIHandle>, exc: &APIHandle) -> *const APIHandle {
        panic!("Not implemented")
    }

    pub fn kill_stack(&mut self, stack: &APIHandle) {
        panic!("Not implemented")
    }

    pub fn set_threadlocal(&mut self, thread: &APIHandle, threadlocal: &APIHandle) {
        panic!("Not implemented")
    }

    pub fn get_threadlocal(&mut self, thread: &APIHandle) -> *const APIHandle {
        panic!("Not implemented")
    }

    pub fn new_cursor(&mut self, stack: &APIHandle) -> *const APIHandle {
        panic!("Not implemented")
    }

    pub fn next_frame(&mut self, cursor: &APIHandle) {
        panic!("Not implemented")
    }

    pub fn copy_cursor(&mut self, cursor: &APIHandle) -> *const APIHandle {
        panic!("Not implemented")
    }

    pub fn close_cursor(&mut self, cursor: &APIHandle) {
        panic!("Not implemented")
    }

    pub fn cur_func(&mut self, cursor: &APIHandle) -> MuID {
        panic!("Not implemented")
    }

    pub fn cur_func_ver(&mut self, cursor: &APIHandle) -> MuID {
        panic!("Not implemented")
    }

    pub fn cur_inst(&mut self, cursor: &APIHandle) -> MuID {
        panic!("Not implemented")
    }

    pub fn dump_keepalives(&mut self, cursor: &APIHandle, results: *mut CMuValue) {
        panic!("Not implemented")
    }

    pub fn pop_frames_to(&mut self, cursor: &APIHandle) {
        panic!("Not implemented")
    }

    pub fn push_frame(&mut self, stack: &APIHandle, func: &APIHandle) {
        panic!("Not implemented")
    }

    pub fn tr64_is_fp(&mut self, value: &APIHandle) -> bool {
        panic!("Not implemented")
    }

    pub fn tr64_is_int(&mut self, value: &APIHandle) -> bool {
        panic!("Not implemented")
    }

    pub fn tr64_is_ref(&mut self, value: &APIHandle) -> bool {
        panic!("Not implemented")
    }

    pub fn tr64_to_fp(&mut self, value: &APIHandle) -> *const APIHandle {
        panic!("Not implemented")
    }

    pub fn tr64_to_int(&mut self, value: &APIHandle) -> *const APIHandle {
        panic!("Not implemented")
    }

    pub fn tr64_to_ref(&mut self, value: &APIHandle) -> *const APIHandle {
        panic!("Not implemented")
    }

    pub fn tr64_to_tag(&mut self, value: &APIHandle) -> *const APIHandle {
        panic!("Not implemented")
    }

    pub fn tr64_from_fp(&mut self, value: &APIHandle) -> *const APIHandle {
        panic!("Not implemented")
    }

    pub fn tr64_from_int(&mut self, value: &APIHandle) -> *const APIHandle {
        panic!("Not implemented")
    }

    pub fn tr64_from_ref(&mut self, reff: &APIHandle, tag: &APIHandle) -> *const APIHandle {
        panic!("Not implemented")
    }

    pub fn enable_watchpoint(&mut self, wpid: CMuWPID) {
        panic!("Not implemented")
    }

    pub fn disable_watchpoint(&mut self, wpid: CMuWPID) {
        panic!("Not implemented")
    }

    pub fn pin(&mut self, loc: &APIHandle) -> *const APIHandle {
        prepare_handle(self.get_mvm().vm.handle_pin_object(loc))
    }

    pub fn unpin(&mut self, loc: &APIHandle) {
        self.get_mvm().vm.handle_unpin_object(loc)
    }

    pub fn get_addr(&mut self, loc: &APIHandle) -> *const APIHandle {
        prepare_handle(self.get_mvm().vm.handle_get_addr(loc))
    }

    pub fn expose(&mut self, func: &APIHandle, call_conv: CMuCallConv, cookie: &APIHandle) -> *const APIHandle {
        panic!("Not implemented")
    }

    pub fn unexpose(&mut self, call_conv: CMuCallConv, value: &APIHandle) {
        panic!("Not implemented")
    }

    pub fn new_ir_builder(&mut self) -> *mut CMuIRBuilder {
        info!("Creating MuIRBuilder...");

        let b: Box<MuIRBuilder> = MuIRBuilder::new(self.mvm);

        let b_ptr = Box::into_raw(b);

        debug!("The MuIRBuilder address: {:?}", b_ptr);

        let cb = make_new_MuIRBuilder(b_ptr as *mut c_void);

        debug!("The C-visible CMuIRBuilder struct address: {:?}", cb);

        unsafe{ (*b_ptr).c_struct = cb; }

        cb
    }

    pub fn make_boot_image(&mut self, whitelist: Vec<MuID>, primordial_func: Option<&APIHandle>, primordial_stack: Option<&APIHandle>, primordial_threadlocal: Option<&APIHandle>, sym_fields: Vec<&APIHandle>, sym_strings: Vec<String>, reloc_fields: Vec<&APIHandle>, reloc_strings: Vec<String>, output_file: String) {
        panic!("Not implemented")
    }

}

fn prepare_handle(handle: APIHandleResult) -> *const APIHandle {
    Box::into_raw(handle)
}

fn delete_handle(handle: APIHandleArg) {
    unsafe {Box::from_raw((handle as *const APIHandle) as *mut APIHandle);}
}

use ast::inst::MemoryOrder;
fn impl_memorder(order: CMuMemOrd) -> MemoryOrder {
    match order {
        CMU_ORD_NOT_ATOMIC => MemoryOrder::NotAtomic,
        CMU_ORD_RELAXED    => MemoryOrder::Relaxed,
        CMU_ORD_CONSUME    => MemoryOrder::Consume,
        CMU_ORD_ACQUIRE    => MemoryOrder::Acquire,
        CMU_ORD_RELEASE    => MemoryOrder::Release,
        CMU_ORD_ACQ_REL    => MemoryOrder::AcqRel,
        CMU_ORD_SEQ_CST    => MemoryOrder::SeqCst,
        _ => panic!("invalid CMuMemOrd flag: {}", order)
    }
}