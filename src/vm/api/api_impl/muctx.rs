use super::common::*;

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

    pub fn handle_from_sint8(&mut self, num: i8, len: c_int) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn handle_from_uint8(&mut self, num: u8, len: c_int) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn handle_from_sint16(&mut self, num: i16, len: c_int) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn handle_from_uint16(&mut self, num: u16, len: c_int) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn handle_from_sint32(&mut self, num: i32, len: c_int) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn handle_from_uint32(&mut self, num: u32, len: c_int) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn handle_from_sint64(&mut self, num: i64, len: c_int) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn handle_from_uint64(&mut self, num: u64, len: c_int) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn handle_from_uint64s(&mut self, nums: &[u64], len: c_int) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn handle_from_float(&mut self, num: f32) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn handle_from_double(&mut self, num: f64) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn handle_from_ptr(&mut self, mu_type: MuID, ptr: CMuCPtr) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn handle_from_fp(&mut self, mu_type: MuID, fp: CMuCFP) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn handle_to_sint8(&mut self, opnd: &APIHandleKey) -> i8 {
        panic!("Not implemented")
    }

    pub fn handle_to_uint8(&mut self, opnd: &APIHandleKey) -> u8 {
        panic!("Not implemented")
    }

    pub fn handle_to_sint16(&mut self, opnd: &APIHandleKey) -> i16 {
        panic!("Not implemented")
    }

    pub fn handle_to_uint16(&mut self, opnd: &APIHandleKey) -> u16 {
        panic!("Not implemented")
    }

    pub fn handle_to_sint32(&mut self, opnd: &APIHandleKey) -> i32 {
        panic!("Not implemented")
    }

    pub fn handle_to_uint32(&mut self, opnd: &APIHandleKey) -> u32 {
        panic!("Not implemented")
    }

    pub fn handle_to_sint64(&mut self, opnd: &APIHandleKey) -> i64 {
        panic!("Not implemented")
    }

    pub fn handle_to_uint64(&mut self, opnd: &APIHandleKey) -> u64 {
        panic!("Not implemented")
    }

    pub fn handle_to_float(&mut self, opnd: &APIHandleKey) -> f32 {
        panic!("Not implemented")
    }

    pub fn handle_to_double(&mut self, opnd: &APIHandleKey) -> f64 {
        panic!("Not implemented")
    }

    pub fn handle_to_ptr(&mut self, opnd: &APIHandleKey) -> CMuCPtr {
        panic!("Not implemented")
    }

    pub fn handle_to_fp(&mut self, opnd: &APIHandleKey) -> CMuCFP {
        panic!("Not implemented")
    }

    pub fn handle_from_const(&mut self, id: MuID) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn handle_from_global(&mut self, id: MuID) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn handle_from_func(&mut self, id: MuID) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn handle_from_expose(&mut self, id: MuID) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn delete_value(&mut self, opnd: &APIHandleKey) {
        panic!("Not implemented")
    }

    pub fn ref_eq(&mut self, lhs: &APIHandleKey, rhs: &APIHandleKey) -> bool {
        panic!("Not implemented")
    }

    pub fn ref_ult(&mut self, lhs: &APIHandleKey, rhs: &APIHandleKey) -> bool {
        panic!("Not implemented")
    }

    pub fn extract_value(&mut self, str: &APIHandleKey, index: c_int) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn insert_value(&mut self, str: &APIHandleKey, index: c_int, newval: &APIHandleKey) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn extract_element(&mut self, str: &APIHandleKey, index: &APIHandleKey) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn insert_element(&mut self, str: &APIHandleKey, index: &APIHandleKey, newval: &APIHandleKey) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn new_fixed(&mut self, mu_type: MuID) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn new_hybrid(&mut self, mu_type: MuID, length: &APIHandleKey) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn refcast(&mut self, opnd: &APIHandleKey, new_type: MuID) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn get_iref(&mut self, opnd: &APIHandleKey) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn get_field_iref(&mut self, opnd: &APIHandleKey, field: c_int) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn get_elem_iref(&mut self, opnd: &APIHandleKey, index: &APIHandleKey) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn shift_iref(&mut self, opnd: &APIHandleKey, offset: &APIHandleKey) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn get_var_part_iref(&mut self, opnd: &APIHandleKey) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn load(&mut self, ord: CMuMemOrd, loc: &APIHandleKey) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn store(&mut self, ord: CMuMemOrd, loc: &APIHandleKey, newval: &APIHandleKey) {
        panic!("Not implemented")
    }

    pub fn cmpxchg(&mut self, ord_succ: CMuMemOrd, ord_fail: CMuMemOrd, weak: bool, loc: &APIHandleKey, expected: &APIHandleKey, desired: &APIHandleKey, is_succ: *mut CMuBool) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn atomicrmw(&mut self, ord: CMuMemOrd, op: CMuAtomicRMWOptr, loc: &APIHandleKey, opnd: &APIHandleKey) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn fence(&mut self, ord: CMuMemOrd) {
        panic!("Not implemented")
    }

    pub fn new_stack(&mut self, func: &APIHandleKey) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn new_thread_nor(&mut self, stack: &APIHandleKey, threadlocal: Option<&APIHandleKey>, vals: Vec<&APIHandleKey>) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn new_thread_exc(&mut self, stack: &APIHandleKey, threadlocal: Option<&APIHandleKey>, exc: &APIHandleKey) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn kill_stack(&mut self, stack: &APIHandleKey) {
        panic!("Not implemented")
    }

    pub fn set_threadlocal(&mut self, thread: &APIHandleKey, threadlocal: &APIHandleKey) {
        panic!("Not implemented")
    }

    pub fn get_threadlocal(&mut self, thread: &APIHandleKey) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn new_cursor(&mut self, stack: &APIHandleKey) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn next_frame(&mut self, cursor: &APIHandleKey) {
        panic!("Not implemented")
    }

    pub fn copy_cursor(&mut self, cursor: &APIHandleKey) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn close_cursor(&mut self, cursor: &APIHandleKey) {
        panic!("Not implemented")
    }

    pub fn cur_func(&mut self, cursor: &APIHandleKey) -> MuID {
        panic!("Not implemented")
    }

    pub fn cur_func_ver(&mut self, cursor: &APIHandleKey) -> MuID {
        panic!("Not implemented")
    }

    pub fn cur_inst(&mut self, cursor: &APIHandleKey) -> MuID {
        panic!("Not implemented")
    }

    pub fn dump_keepalives(&mut self, cursor: &APIHandleKey, results: *mut CMuValue) {
        panic!("Not implemented")
    }

    pub fn pop_frames_to(&mut self, cursor: &APIHandleKey) {
        panic!("Not implemented")
    }

    pub fn push_frame(&mut self, stack: &APIHandleKey, func: &APIHandleKey) {
        panic!("Not implemented")
    }

    pub fn tr64_is_fp(&mut self, value: &APIHandleKey) -> bool {
        panic!("Not implemented")
    }

    pub fn tr64_is_int(&mut self, value: &APIHandleKey) -> bool {
        panic!("Not implemented")
    }

    pub fn tr64_is_ref(&mut self, value: &APIHandleKey) -> bool {
        panic!("Not implemented")
    }

    pub fn tr64_to_fp(&mut self, value: &APIHandleKey) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn tr64_to_int(&mut self, value: &APIHandleKey) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn tr64_to_ref(&mut self, value: &APIHandleKey) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn tr64_to_tag(&mut self, value: &APIHandleKey) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn tr64_from_fp(&mut self, value: &APIHandleKey) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn tr64_from_int(&mut self, value: &APIHandleKey) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn tr64_from_ref(&mut self, reff: &APIHandleKey, tag: &APIHandleKey) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn enable_watchpoint(&mut self, wpid: CMuWPID) {
        panic!("Not implemented")
    }

    pub fn disable_watchpoint(&mut self, wpid: CMuWPID) {
        panic!("Not implemented")
    }

    pub fn pin(&mut self, loc: &APIHandleKey) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn unpin(&mut self, loc: &APIHandleKey) {
        panic!("Not implemented")
    }

    pub fn get_addr(&mut self, loc: &APIHandleKey) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn expose(&mut self, func: &APIHandleKey, call_conv: CMuCallConv, cookie: &APIHandleKey) -> *const APIHandleKey {
        panic!("Not implemented")
    }

    pub fn unexpose(&mut self, call_conv: CMuCallConv, value: &APIHandleKey) {
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

    pub fn make_boot_image(&mut self, whitelist: Vec<MuID>, primordial_func: Option<&APIHandleKey>, primordial_stack: Option<&APIHandleKey>, primordial_threadlocal: Option<&APIHandleKey>, sym_fields: Vec<&APIHandleKey>, sym_strings: Vec<String>, reloc_fields: Vec<&APIHandleKey>, reloc_strings: Vec<String>, output_file: String) {
        panic!("Not implemented")
    }

}
