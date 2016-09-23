use std::ptr;
use std::os::raw::*;
use std::ffi::CString;
use std::slice;

use super::api_c::*;
use super::api_impl::*;
use super::deps::*;

// hand-written functions

// The following functions "from_*" convert low-level types to high-level types.

// Get the pointer to the high-level struct.
//
// The return value is a pointer. Usually the API function body do not change the ownership of the
// self parameter, so `&mut` should suffice. However, some API functions (such as
// `MuCtx.close_context`) will free the resources, it is probably better to call the methods on
// (*ptr) directly (such as `(*pmuctx).close_context()`), and offload the decision to the concrete
// implementation.
//
// Assume the pointer is the header.  For the sake of "implementation neutrality", the header
// defined in api_c.rs is encoded as *mut c_void rather than *mut MuVM or other concrete pointers.
// But it doesn't matter, because the header is only ever read in these three functions.
#[inline(always)]
fn from_MuVM_ptr(ptr: *mut CMuVM) -> *mut MuVM {
    debug_assert!(!ptr.is_null());
    unsafe {
        (*ptr).header as *mut MuVM
    }
}

#[inline(always)]
fn from_MuCtx_ptr(ptr: *mut CMuCtx) -> *mut MuCtx {
    debug_assert!(!ptr.is_null());
    unsafe {
        (*ptr).header as *mut MuCtx
    }
}

#[inline(always)]
fn from_MuIRBuilder_ptr(ptr: *mut CMuIRBuilder) -> *mut MuIRBuilder {
    debug_assert!(!ptr.is_null());
    unsafe {
        (*ptr).header as *mut MuIRBuilder
    }
}

#[inline(always)]
fn from_MuName(cname: CMuName) -> MuName {
    from_MuCString(cname)
}

#[inline(always)]
fn from_MuCString(cstring: CMuCString) -> String {
    debug_assert!(!cstring.is_null());
    let ffi_cstring = unsafe {
        CString::from_raw(cstring)
    };

    ffi_cstring.into_string().unwrap()
}

#[inline(always)]
fn from_MuCString_optional(cstring: CMuCString) -> Option<String> {
    if cstring.is_null() {
        None
    } else {
        Some(from_MuCString(cstring))
    }
}

#[inline(always)]
fn from_MuID(cmuid: CMuID) -> MuID {
    debug_assert!(cmuid != 0);
    cmuid as MuID   // just zero extend
}

#[inline(always)]
fn from_MuID_optional(cmuid: CMuID) -> Option<MuID> {
    if cmuid == 0 {
        None
    } else {
        Some(from_MuID(cmuid))
    }
}

#[inline(always)]
fn from_MuBool(cbool: CMuBool) -> bool {
    cbool != 0
}

// APIMuValue is immutable when used.
#[inline(always)]
fn from_handle<'a>(cmuvalue: CMuValue) -> &'a APIMuValue {
    debug_assert!(!cmuvalue.is_null());
    unsafe {
        &*(cmuvalue as *const APIMuValue)
    }
}

#[inline(always)]
fn from_handle_optional<'a>(cmuvalue: CMuValue) -> Option<&'a APIMuValue> {
    if cmuvalue.is_null() {
        None
    } else {
        Some(from_handle(cmuvalue))
    }
}

// The following functions "from_*_array" converts from C-level arrays to Rust-level slices.

#[inline(always)]
fn from_array_direct<'a, T>(ptr: *const T, len: usize) -> &'a [T] {
    if ptr.is_null() {
        unsafe {
            slice::from_raw_parts(ptr::null(), len)
        }
    } else {
        unsafe {
            slice::from_raw_parts(ptr, len)
        }
    }
}

#[inline(always)]
fn from_char_array<'a>(ptr: *const c_char, len: usize) -> &'a [c_char] {
    from_array_direct(ptr, len)
}

#[inline(always)]
fn from_uint64_t_array<'a>(ptr: *const u64, len: usize) -> &'a [u64] {
    from_array_direct(ptr, len)
}

#[inline(always)]
fn from_MuID_array<'a>(ptr: *const CMuID, len: usize) -> &'a [CMuID] {
    from_array_direct(ptr, len)
}

#[inline(always)]
fn from_MuFlag_array<'a>(ptr: *const CMuFlag, len: usize) -> &'a [CMuFlag] {
    from_array_direct(ptr, len)
}

/// Convert into a Vec of handle refs. It is not certain whether refs are represented in the same
/// way as raw pointers. So this function will convert each element. This function is only called
/// by `new_thread_nor`. As always, thread creation dominates the time.
#[inline(always)]
fn from_handle_array<'a>(ptr: *const CMuValue, len: usize) -> Vec<&'a APIMuValue> {
    let slc = from_array_direct(ptr, len);
    slc.iter().map(|&e| {
        debug_assert!(!e.is_null());
        unsafe { &*(e as *const APIMuValue) }
    }).collect::<Vec<_>>()
}

#[inline(always)]
fn from_MuCString_array<'a>(ptr: *const CMuCString, len: usize) -> Vec<String> {
    let slc = from_array_direct(ptr, len);
    slc.iter().map(|&e| from_MuCString(e)).collect::<Vec<_>>()
}

// GEN:BEGIN:Forwarders
extern fn _forwarder__MuVM__new_context(mvm: *mut CMuVM)-> *mut CMuCtx {
    let mut _arg_mvm = from_MuVM_ptr(mvm);
    panic!("not implemented")
}

extern fn _forwarder__MuVM__id_of(mvm: *mut CMuVM, name: CMuName)-> CMuID {
    let mut _arg_mvm = from_MuVM_ptr(mvm);
    let mut _arg_name = from_MuName(name);
    panic!("not implemented")
}

extern fn _forwarder__MuVM__name_of(mvm: *mut CMuVM, id: CMuID)-> CMuName {
    let mut _arg_mvm = from_MuVM_ptr(mvm);
    let mut _arg_id = from_MuID(id);
    panic!("not implemented")
}

extern fn _forwarder__MuVM__set_trap_handler(mvm: *mut CMuVM, trap_handler: CMuTrapHandler, userdata: CMuCPtr) {
    let mut _arg_mvm = from_MuVM_ptr(mvm);
    let mut _arg_trap_handler = trap_handler;
    let mut _arg_userdata = userdata;
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__id_of(ctx: *mut CMuCtx, name: CMuName)-> CMuID {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_name = from_MuName(name);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__name_of(ctx: *mut CMuCtx, id: CMuID)-> CMuName {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_id = from_MuID(id);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__close_context(ctx: *mut CMuCtx) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__load_bundle(ctx: *mut CMuCtx, buf: *mut c_char, sz: CMuArraySize) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_buf = from_char_array(buf, sz);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__load_hail(ctx: *mut CMuCtx, buf: *mut c_char, sz: CMuArraySize) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_buf = from_char_array(buf, sz);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__handle_from_sint8(ctx: *mut CMuCtx, num: i8, len: c_int)-> CMuIntValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_num = num;
    let mut _arg_len = len;
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__handle_from_uint8(ctx: *mut CMuCtx, num: u8, len: c_int)-> CMuIntValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_num = num;
    let mut _arg_len = len;
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__handle_from_sint16(ctx: *mut CMuCtx, num: i16, len: c_int)-> CMuIntValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_num = num;
    let mut _arg_len = len;
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__handle_from_uint16(ctx: *mut CMuCtx, num: u16, len: c_int)-> CMuIntValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_num = num;
    let mut _arg_len = len;
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__handle_from_sint32(ctx: *mut CMuCtx, num: i32, len: c_int)-> CMuIntValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_num = num;
    let mut _arg_len = len;
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__handle_from_uint32(ctx: *mut CMuCtx, num: u32, len: c_int)-> CMuIntValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_num = num;
    let mut _arg_len = len;
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__handle_from_sint64(ctx: *mut CMuCtx, num: i64, len: c_int)-> CMuIntValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_num = num;
    let mut _arg_len = len;
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__handle_from_uint64(ctx: *mut CMuCtx, num: u64, len: c_int)-> CMuIntValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_num = num;
    let mut _arg_len = len;
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__handle_from_uint64s(ctx: *mut CMuCtx, nums: *mut u64, nnums: CMuArraySize, len: c_int)-> CMuIntValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_nums = from_uint64_t_array(nums, nnums);
    let mut _arg_len = len;
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__handle_from_float(ctx: *mut CMuCtx, num: f32)-> CMuFloatValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_num = num;
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__handle_from_double(ctx: *mut CMuCtx, num: f64)-> CMuDoubleValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_num = num;
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__handle_from_ptr(ctx: *mut CMuCtx, mu_type: CMuID, ptr: CMuCPtr)-> CMuUPtrValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_mu_type = from_MuID(mu_type);
    let mut _arg_ptr = ptr;
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__handle_from_fp(ctx: *mut CMuCtx, mu_type: CMuID, fp: CMuCFP)-> CMuUFPValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_mu_type = from_MuID(mu_type);
    let mut _arg_fp = fp;
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__handle_to_sint8(ctx: *mut CMuCtx, opnd: CMuIntValue)-> i8 {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_handle(opnd);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__handle_to_uint8(ctx: *mut CMuCtx, opnd: CMuIntValue)-> u8 {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_handle(opnd);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__handle_to_sint16(ctx: *mut CMuCtx, opnd: CMuIntValue)-> i16 {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_handle(opnd);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__handle_to_uint16(ctx: *mut CMuCtx, opnd: CMuIntValue)-> u16 {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_handle(opnd);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__handle_to_sint32(ctx: *mut CMuCtx, opnd: CMuIntValue)-> i32 {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_handle(opnd);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__handle_to_uint32(ctx: *mut CMuCtx, opnd: CMuIntValue)-> u32 {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_handle(opnd);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__handle_to_sint64(ctx: *mut CMuCtx, opnd: CMuIntValue)-> i64 {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_handle(opnd);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__handle_to_uint64(ctx: *mut CMuCtx, opnd: CMuIntValue)-> u64 {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_handle(opnd);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__handle_to_float(ctx: *mut CMuCtx, opnd: CMuFloatValue)-> f32 {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_handle(opnd);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__handle_to_double(ctx: *mut CMuCtx, opnd: CMuDoubleValue)-> f64 {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_handle(opnd);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__handle_to_ptr(ctx: *mut CMuCtx, opnd: CMuUPtrValue)-> CMuCPtr {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_handle(opnd);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__handle_to_fp(ctx: *mut CMuCtx, opnd: CMuUFPValue)-> CMuCFP {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_handle(opnd);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__handle_from_const(ctx: *mut CMuCtx, id: CMuID)-> CMuValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_id = from_MuID(id);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__handle_from_global(ctx: *mut CMuCtx, id: CMuID)-> CMuIRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_id = from_MuID(id);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__handle_from_func(ctx: *mut CMuCtx, id: CMuID)-> CMuFuncRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_id = from_MuID(id);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__handle_from_expose(ctx: *mut CMuCtx, id: CMuID)-> CMuValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_id = from_MuID(id);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__delete_value(ctx: *mut CMuCtx, opnd: CMuValue) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_handle(opnd);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__ref_eq(ctx: *mut CMuCtx, lhs: CMuGenRefValue, rhs: CMuGenRefValue)-> CMuBool {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_lhs = from_handle(lhs);
    let mut _arg_rhs = from_handle(rhs);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__ref_ult(ctx: *mut CMuCtx, lhs: CMuIRefValue, rhs: CMuIRefValue)-> CMuBool {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_lhs = from_handle(lhs);
    let mut _arg_rhs = from_handle(rhs);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__extract_value(ctx: *mut CMuCtx, str: CMuStructValue, index: c_int)-> CMuValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_str = from_handle(str);
    let mut _arg_index = index;
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__insert_value(ctx: *mut CMuCtx, str: CMuStructValue, index: c_int, newval: CMuValue)-> CMuStructValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_str = from_handle(str);
    let mut _arg_index = index;
    let mut _arg_newval = from_handle(newval);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__extract_element(ctx: *mut CMuCtx, str: CMuSeqValue, index: CMuIntValue)-> CMuValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_str = from_handle(str);
    let mut _arg_index = from_handle(index);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__insert_element(ctx: *mut CMuCtx, str: CMuSeqValue, index: CMuIntValue, newval: CMuValue)-> CMuSeqValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_str = from_handle(str);
    let mut _arg_index = from_handle(index);
    let mut _arg_newval = from_handle(newval);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__new_fixed(ctx: *mut CMuCtx, mu_type: CMuID)-> CMuRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_mu_type = from_MuID(mu_type);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__new_hybrid(ctx: *mut CMuCtx, mu_type: CMuID, length: CMuIntValue)-> CMuRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_mu_type = from_MuID(mu_type);
    let mut _arg_length = from_handle(length);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__refcast(ctx: *mut CMuCtx, opnd: CMuGenRefValue, new_type: CMuID)-> CMuGenRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_handle(opnd);
    let mut _arg_new_type = from_MuID(new_type);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__get_iref(ctx: *mut CMuCtx, opnd: CMuRefValue)-> CMuIRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_handle(opnd);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__get_field_iref(ctx: *mut CMuCtx, opnd: CMuIRefValue, field: c_int)-> CMuIRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_handle(opnd);
    let mut _arg_field = field;
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__get_elem_iref(ctx: *mut CMuCtx, opnd: CMuIRefValue, index: CMuIntValue)-> CMuIRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_handle(opnd);
    let mut _arg_index = from_handle(index);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__shift_iref(ctx: *mut CMuCtx, opnd: CMuIRefValue, offset: CMuIntValue)-> CMuIRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_handle(opnd);
    let mut _arg_offset = from_handle(offset);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__get_var_part_iref(ctx: *mut CMuCtx, opnd: CMuIRefValue)-> CMuIRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_handle(opnd);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__load(ctx: *mut CMuCtx, ord: CMuMemOrd, loc: CMuIRefValue)-> CMuValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_ord = ord;
    let mut _arg_loc = from_handle(loc);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__store(ctx: *mut CMuCtx, ord: CMuMemOrd, loc: CMuIRefValue, newval: CMuValue) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_ord = ord;
    let mut _arg_loc = from_handle(loc);
    let mut _arg_newval = from_handle(newval);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__cmpxchg(ctx: *mut CMuCtx, ord_succ: CMuMemOrd, ord_fail: CMuMemOrd, weak: CMuBool, loc: CMuIRefValue, expected: CMuValue, desired: CMuValue, is_succ: *mut CMuBool)-> CMuValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_ord_succ = ord_succ;
    let mut _arg_ord_fail = ord_fail;
    let mut _arg_weak = from_MuBool(weak);
    let mut _arg_loc = from_handle(loc);
    let mut _arg_expected = from_handle(expected);
    let mut _arg_desired = from_handle(desired);
    let mut _arg_is_succ = is_succ;
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__atomicrmw(ctx: *mut CMuCtx, ord: CMuMemOrd, op: CMuAtomicRMWOptr, loc: CMuIRefValue, opnd: CMuValue)-> CMuValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_ord = ord;
    let mut _arg_op = op;
    let mut _arg_loc = from_handle(loc);
    let mut _arg_opnd = from_handle(opnd);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__fence(ctx: *mut CMuCtx, ord: CMuMemOrd) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_ord = ord;
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__new_stack(ctx: *mut CMuCtx, func: CMuFuncRefValue)-> CMuStackRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_func = from_handle(func);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__new_thread_nor(ctx: *mut CMuCtx, stack: CMuStackRefValue, threadlocal: CMuRefValue, vals: *mut CMuValue, nvals: CMuArraySize)-> CMuThreadRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_stack = from_handle(stack);
    let mut _arg_threadlocal = from_handle_optional(threadlocal);
    let mut _arg_vals = from_handle_array(vals, nvals);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__new_thread_exc(ctx: *mut CMuCtx, stack: CMuStackRefValue, threadlocal: CMuRefValue, exc: CMuRefValue)-> CMuThreadRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_stack = from_handle(stack);
    let mut _arg_threadlocal = from_handle_optional(threadlocal);
    let mut _arg_exc = from_handle(exc);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__kill_stack(ctx: *mut CMuCtx, stack: CMuStackRefValue) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_stack = from_handle(stack);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__set_threadlocal(ctx: *mut CMuCtx, thread: CMuThreadRefValue, threadlocal: CMuRefValue) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_thread = from_handle(thread);
    let mut _arg_threadlocal = from_handle(threadlocal);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__get_threadlocal(ctx: *mut CMuCtx, thread: CMuThreadRefValue)-> CMuRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_thread = from_handle(thread);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__new_cursor(ctx: *mut CMuCtx, stack: CMuStackRefValue)-> CMuFCRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_stack = from_handle(stack);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__next_frame(ctx: *mut CMuCtx, cursor: CMuFCRefValue) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_cursor = from_handle(cursor);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__copy_cursor(ctx: *mut CMuCtx, cursor: CMuFCRefValue)-> CMuFCRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_cursor = from_handle(cursor);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__close_cursor(ctx: *mut CMuCtx, cursor: CMuFCRefValue) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_cursor = from_handle(cursor);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__cur_func(ctx: *mut CMuCtx, cursor: CMuFCRefValue)-> CMuID {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_cursor = from_handle(cursor);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__cur_func_ver(ctx: *mut CMuCtx, cursor: CMuFCRefValue)-> CMuID {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_cursor = from_handle(cursor);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__cur_inst(ctx: *mut CMuCtx, cursor: CMuFCRefValue)-> CMuID {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_cursor = from_handle(cursor);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__dump_keepalives(ctx: *mut CMuCtx, cursor: CMuFCRefValue, results: *mut CMuValue) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_cursor = from_handle(cursor);
    let mut _arg_results = results;
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__pop_frames_to(ctx: *mut CMuCtx, cursor: CMuFCRefValue) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_cursor = from_handle(cursor);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__push_frame(ctx: *mut CMuCtx, stack: CMuStackRefValue, func: CMuFuncRefValue) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_stack = from_handle(stack);
    let mut _arg_func = from_handle(func);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__tr64_is_fp(ctx: *mut CMuCtx, value: CMuTagRef64Value)-> CMuBool {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_value = from_handle(value);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__tr64_is_int(ctx: *mut CMuCtx, value: CMuTagRef64Value)-> CMuBool {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_value = from_handle(value);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__tr64_is_ref(ctx: *mut CMuCtx, value: CMuTagRef64Value)-> CMuBool {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_value = from_handle(value);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__tr64_to_fp(ctx: *mut CMuCtx, value: CMuTagRef64Value)-> CMuDoubleValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_value = from_handle(value);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__tr64_to_int(ctx: *mut CMuCtx, value: CMuTagRef64Value)-> CMuIntValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_value = from_handle(value);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__tr64_to_ref(ctx: *mut CMuCtx, value: CMuTagRef64Value)-> CMuRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_value = from_handle(value);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__tr64_to_tag(ctx: *mut CMuCtx, value: CMuTagRef64Value)-> CMuIntValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_value = from_handle(value);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__tr64_from_fp(ctx: *mut CMuCtx, value: CMuDoubleValue)-> CMuTagRef64Value {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_value = from_handle(value);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__tr64_from_int(ctx: *mut CMuCtx, value: CMuIntValue)-> CMuTagRef64Value {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_value = from_handle(value);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__tr64_from_ref(ctx: *mut CMuCtx, reff: CMuRefValue, tag: CMuIntValue)-> CMuTagRef64Value {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_reff = from_handle(reff);
    let mut _arg_tag = from_handle(tag);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__enable_watchpoint(ctx: *mut CMuCtx, wpid: CMuWPID) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_wpid = wpid;
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__disable_watchpoint(ctx: *mut CMuCtx, wpid: CMuWPID) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_wpid = wpid;
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__pin(ctx: *mut CMuCtx, loc: CMuValue)-> CMuUPtrValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_loc = from_handle(loc);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__unpin(ctx: *mut CMuCtx, loc: CMuValue) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_loc = from_handle(loc);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__get_addr(ctx: *mut CMuCtx, loc: CMuValue)-> CMuUPtrValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_loc = from_handle(loc);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__expose(ctx: *mut CMuCtx, func: CMuFuncRefValue, call_conv: CMuCallConv, cookie: CMuIntValue)-> CMuValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_func = from_handle(func);
    let mut _arg_call_conv = call_conv;
    let mut _arg_cookie = from_handle(cookie);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__unexpose(ctx: *mut CMuCtx, call_conv: CMuCallConv, value: CMuValue) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_call_conv = call_conv;
    let mut _arg_value = from_handle(value);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__new_ir_builder(ctx: *mut CMuCtx)-> *mut CMuIRBuilder {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    panic!("not implemented")
}

extern fn _forwarder__MuCtx__make_boot_image(ctx: *mut CMuCtx, whitelist: *mut CMuID, whitelist_sz: CMuArraySize, primordial_func: CMuFuncRefValue, primordial_stack: CMuStackRefValue, primordial_threadlocal: CMuRefValue, sym_fields: *mut CMuIRefValue, sym_strings: *mut CMuCString, nsyms: CMuArraySize, reloc_fields: *mut CMuIRefValue, reloc_strings: *mut CMuCString, nrelocs: CMuArraySize, output_file: CMuCString) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_whitelist = from_MuID_array(whitelist, whitelist_sz);
    let mut _arg_primordial_func = from_handle_optional(primordial_func);
    let mut _arg_primordial_stack = from_handle_optional(primordial_stack);
    let mut _arg_primordial_threadlocal = from_handle_optional(primordial_threadlocal);
    let mut _arg_sym_fields = from_handle_array(sym_fields, nsyms);
    let mut _arg_sym_strings = from_MuCString_array(sym_strings, nsyms);
    let mut _arg_reloc_fields = from_handle_array(reloc_fields, nrelocs);
    let mut _arg_reloc_strings = from_MuCString_array(reloc_strings, nrelocs);
    let mut _arg_output_file = from_MuCString(output_file);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__load(b: *mut CMuIRBuilder) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__abort(b: *mut CMuIRBuilder) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__gen_sym(b: *mut CMuIRBuilder, name: CMuCString)-> CMuID {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_name = from_MuCString_optional(name);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_type_int(b: *mut CMuIRBuilder, id: CMuID, len: c_int) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_len = len;
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_type_float(b: *mut CMuIRBuilder, id: CMuID) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_type_double(b: *mut CMuIRBuilder, id: CMuID) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_type_uptr(b: *mut CMuIRBuilder, id: CMuID, ty: CMuTypeNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_ty = from_MuID(ty);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_type_ufuncptr(b: *mut CMuIRBuilder, id: CMuID, sig: CMuFuncSigNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_sig = from_MuID(sig);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_type_struct(b: *mut CMuIRBuilder, id: CMuID, fieldtys: *mut CMuTypeNode, nfieldtys: CMuArraySize) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_fieldtys = from_MuID_array(fieldtys, nfieldtys);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_type_hybrid(b: *mut CMuIRBuilder, id: CMuID, fixedtys: *mut CMuTypeNode, nfixedtys: CMuArraySize, varty: CMuTypeNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_fixedtys = from_MuID_array(fixedtys, nfixedtys);
    let mut _arg_varty = from_MuID(varty);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_type_array(b: *mut CMuIRBuilder, id: CMuID, elemty: CMuTypeNode, len: u64) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_elemty = from_MuID(elemty);
    let mut _arg_len = len;
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_type_vector(b: *mut CMuIRBuilder, id: CMuID, elemty: CMuTypeNode, len: u64) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_elemty = from_MuID(elemty);
    let mut _arg_len = len;
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_type_void(b: *mut CMuIRBuilder, id: CMuID) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_type_ref(b: *mut CMuIRBuilder, id: CMuID, ty: CMuTypeNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_ty = from_MuID(ty);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_type_iref(b: *mut CMuIRBuilder, id: CMuID, ty: CMuTypeNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_ty = from_MuID(ty);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_type_weakref(b: *mut CMuIRBuilder, id: CMuID, ty: CMuTypeNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_ty = from_MuID(ty);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_type_funcref(b: *mut CMuIRBuilder, id: CMuID, sig: CMuFuncSigNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_sig = from_MuID(sig);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_type_tagref64(b: *mut CMuIRBuilder, id: CMuID) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_type_threadref(b: *mut CMuIRBuilder, id: CMuID) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_type_stackref(b: *mut CMuIRBuilder, id: CMuID) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_type_framecursorref(b: *mut CMuIRBuilder, id: CMuID) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_type_irbuilderref(b: *mut CMuIRBuilder, id: CMuID) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_funcsig(b: *mut CMuIRBuilder, id: CMuID, paramtys: *mut CMuTypeNode, nparamtys: CMuArraySize, rettys: *mut CMuTypeNode, nrettys: CMuArraySize) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_paramtys = from_MuID_array(paramtys, nparamtys);
    let mut _arg_rettys = from_MuID_array(rettys, nrettys);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_const_int(b: *mut CMuIRBuilder, id: CMuID, ty: CMuTypeNode, value: u64) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_ty = from_MuID(ty);
    let mut _arg_value = value;
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_const_int_ex(b: *mut CMuIRBuilder, id: CMuID, ty: CMuTypeNode, values: *mut u64, nvalues: CMuArraySize) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_ty = from_MuID(ty);
    let mut _arg_values = from_uint64_t_array(values, nvalues);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_const_float(b: *mut CMuIRBuilder, id: CMuID, ty: CMuTypeNode, value: f32) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_ty = from_MuID(ty);
    let mut _arg_value = value;
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_const_double(b: *mut CMuIRBuilder, id: CMuID, ty: CMuTypeNode, value: f64) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_ty = from_MuID(ty);
    let mut _arg_value = value;
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_const_null(b: *mut CMuIRBuilder, id: CMuID, ty: CMuTypeNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_ty = from_MuID(ty);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_const_seq(b: *mut CMuIRBuilder, id: CMuID, ty: CMuTypeNode, elems: *mut CMuGlobalVarNode, nelems: CMuArraySize) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_ty = from_MuID(ty);
    let mut _arg_elems = from_MuID_array(elems, nelems);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_const_extern(b: *mut CMuIRBuilder, id: CMuID, ty: CMuTypeNode, symbol: CMuCString) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_ty = from_MuID(ty);
    let mut _arg_symbol = from_MuCString(symbol);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_global_cell(b: *mut CMuIRBuilder, id: CMuID, ty: CMuTypeNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_ty = from_MuID(ty);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_func(b: *mut CMuIRBuilder, id: CMuID, sig: CMuFuncSigNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_sig = from_MuID(sig);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_exp_func(b: *mut CMuIRBuilder, id: CMuID, func: CMuFuncNode, callconv: CMuCallConv, cookie: CMuConstNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_func = from_MuID(func);
    let mut _arg_callconv = callconv;
    let mut _arg_cookie = from_MuID(cookie);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_func_ver(b: *mut CMuIRBuilder, id: CMuID, func: CMuFuncNode, bbs: *mut CMuBBNode, nbbs: CMuArraySize) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_func = from_MuID(func);
    let mut _arg_bbs = from_MuID_array(bbs, nbbs);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_bb(b: *mut CMuIRBuilder, id: CMuID, nor_param_ids: *mut CMuID, nor_param_types: *mut CMuTypeNode, n_nor_params: CMuArraySize, exc_param_id: CMuID, insts: *mut CMuInstNode, ninsts: CMuArraySize) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_nor_param_ids = from_MuID_array(nor_param_ids, n_nor_params);
    let mut _arg_nor_param_types = from_MuID_array(nor_param_types, n_nor_params);
    let mut _arg_exc_param_id = from_MuID_optional(exc_param_id);
    let mut _arg_insts = from_MuID_array(insts, ninsts);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_dest_clause(b: *mut CMuIRBuilder, id: CMuID, dest: CMuBBNode, vars: *mut CMuVarNode, nvars: CMuArraySize) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_dest = from_MuID(dest);
    let mut _arg_vars = from_MuID_array(vars, nvars);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_exc_clause(b: *mut CMuIRBuilder, id: CMuID, nor: CMuDestClause, exc: CMuDestClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_nor = from_MuID(nor);
    let mut _arg_exc = from_MuID(exc);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_keepalive_clause(b: *mut CMuIRBuilder, id: CMuID, vars: *mut CMuLocalVarNode, nvars: CMuArraySize) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_vars = from_MuID_array(vars, nvars);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_csc_ret_with(b: *mut CMuIRBuilder, id: CMuID, rettys: *mut CMuTypeNode, nrettys: CMuArraySize) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_rettys = from_MuID_array(rettys, nrettys);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_csc_kill_old(b: *mut CMuIRBuilder, id: CMuID) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_nsc_pass_values(b: *mut CMuIRBuilder, id: CMuID, tys: *mut CMuTypeNode, vars: *mut CMuVarNode, ntysvars: CMuArraySize) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_tys = from_MuID_array(tys, ntysvars);
    let mut _arg_vars = from_MuID_array(vars, ntysvars);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_nsc_throw_exc(b: *mut CMuIRBuilder, id: CMuID, exc: CMuVarNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_exc = from_MuID(exc);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_binop(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, optr: CMuBinOptr, ty: CMuTypeNode, opnd1: CMuVarNode, opnd2: CMuVarNode, exc_clause: CMuExcClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_optr = optr;
    let mut _arg_ty = from_MuID(ty);
    let mut _arg_opnd1 = from_MuID(opnd1);
    let mut _arg_opnd2 = from_MuID(opnd2);
    let mut _arg_exc_clause = from_MuID_optional(exc_clause);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_binop_with_status(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, status_result_ids: *mut CMuID, n_status_result_ids: CMuArraySize, optr: CMuBinOptr, status_flags: CMuBinOpStatus, ty: CMuTypeNode, opnd1: CMuVarNode, opnd2: CMuVarNode, exc_clause: CMuExcClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_status_result_ids = from_MuID_array(status_result_ids, n_status_result_ids);
    let mut _arg_optr = optr;
    let mut _arg_status_flags = status_flags;
    let mut _arg_ty = from_MuID(ty);
    let mut _arg_opnd1 = from_MuID(opnd1);
    let mut _arg_opnd2 = from_MuID(opnd2);
    let mut _arg_exc_clause = from_MuID_optional(exc_clause);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_cmp(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, optr: CMuCmpOptr, ty: CMuTypeNode, opnd1: CMuVarNode, opnd2: CMuVarNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_optr = optr;
    let mut _arg_ty = from_MuID(ty);
    let mut _arg_opnd1 = from_MuID(opnd1);
    let mut _arg_opnd2 = from_MuID(opnd2);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_conv(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, optr: CMuConvOptr, from_ty: CMuTypeNode, to_ty: CMuTypeNode, opnd: CMuVarNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_optr = optr;
    let mut _arg_from_ty = from_MuID(from_ty);
    let mut _arg_to_ty = from_MuID(to_ty);
    let mut _arg_opnd = from_MuID(opnd);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_select(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, cond_ty: CMuTypeNode, opnd_ty: CMuTypeNode, cond: CMuVarNode, if_true: CMuVarNode, if_false: CMuVarNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_cond_ty = from_MuID(cond_ty);
    let mut _arg_opnd_ty = from_MuID(opnd_ty);
    let mut _arg_cond = from_MuID(cond);
    let mut _arg_if_true = from_MuID(if_true);
    let mut _arg_if_false = from_MuID(if_false);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_branch(b: *mut CMuIRBuilder, id: CMuID, dest: CMuDestClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_dest = from_MuID(dest);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_branch2(b: *mut CMuIRBuilder, id: CMuID, cond: CMuVarNode, if_true: CMuDestClause, if_false: CMuDestClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_cond = from_MuID(cond);
    let mut _arg_if_true = from_MuID(if_true);
    let mut _arg_if_false = from_MuID(if_false);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_switch(b: *mut CMuIRBuilder, id: CMuID, opnd_ty: CMuTypeNode, opnd: CMuVarNode, default_dest: CMuDestClause, cases: *mut CMuConstNode, dests: *mut CMuDestClause, ncasesdests: CMuArraySize) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_opnd_ty = from_MuID(opnd_ty);
    let mut _arg_opnd = from_MuID(opnd);
    let mut _arg_default_dest = from_MuID(default_dest);
    let mut _arg_cases = from_MuID_array(cases, ncasesdests);
    let mut _arg_dests = from_MuID_array(dests, ncasesdests);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_call(b: *mut CMuIRBuilder, id: CMuID, result_ids: *mut CMuID, n_result_ids: CMuArraySize, sig: CMuFuncSigNode, callee: CMuVarNode, args: *mut CMuVarNode, nargs: CMuArraySize, exc_clause: CMuExcClause, keepalive_clause: CMuKeepaliveClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_ids = from_MuID_array(result_ids, n_result_ids);
    let mut _arg_sig = from_MuID(sig);
    let mut _arg_callee = from_MuID(callee);
    let mut _arg_args = from_MuID_array(args, nargs);
    let mut _arg_exc_clause = from_MuID_optional(exc_clause);
    let mut _arg_keepalive_clause = from_MuID_optional(keepalive_clause);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_tailcall(b: *mut CMuIRBuilder, id: CMuID, sig: CMuFuncSigNode, callee: CMuVarNode, args: *mut CMuVarNode, nargs: CMuArraySize) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_sig = from_MuID(sig);
    let mut _arg_callee = from_MuID(callee);
    let mut _arg_args = from_MuID_array(args, nargs);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_ret(b: *mut CMuIRBuilder, id: CMuID, rvs: *mut CMuVarNode, nrvs: CMuArraySize) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_rvs = from_MuID_array(rvs, nrvs);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_throw(b: *mut CMuIRBuilder, id: CMuID, exc: CMuVarNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_exc = from_MuID(exc);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_extractvalue(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, strty: CMuTypeNode, index: c_int, opnd: CMuVarNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_strty = from_MuID(strty);
    let mut _arg_index = index;
    let mut _arg_opnd = from_MuID(opnd);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_insertvalue(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, strty: CMuTypeNode, index: c_int, opnd: CMuVarNode, newval: CMuVarNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_strty = from_MuID(strty);
    let mut _arg_index = index;
    let mut _arg_opnd = from_MuID(opnd);
    let mut _arg_newval = from_MuID(newval);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_extractelement(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, seqty: CMuTypeNode, indty: CMuTypeNode, opnd: CMuVarNode, index: CMuVarNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_seqty = from_MuID(seqty);
    let mut _arg_indty = from_MuID(indty);
    let mut _arg_opnd = from_MuID(opnd);
    let mut _arg_index = from_MuID(index);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_insertelement(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, seqty: CMuTypeNode, indty: CMuTypeNode, opnd: CMuVarNode, index: CMuVarNode, newval: CMuVarNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_seqty = from_MuID(seqty);
    let mut _arg_indty = from_MuID(indty);
    let mut _arg_opnd = from_MuID(opnd);
    let mut _arg_index = from_MuID(index);
    let mut _arg_newval = from_MuID(newval);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_shufflevector(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, vecty: CMuTypeNode, maskty: CMuTypeNode, vec1: CMuVarNode, vec2: CMuVarNode, mask: CMuVarNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_vecty = from_MuID(vecty);
    let mut _arg_maskty = from_MuID(maskty);
    let mut _arg_vec1 = from_MuID(vec1);
    let mut _arg_vec2 = from_MuID(vec2);
    let mut _arg_mask = from_MuID(mask);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_new(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, allocty: CMuTypeNode, exc_clause: CMuExcClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_allocty = from_MuID(allocty);
    let mut _arg_exc_clause = from_MuID_optional(exc_clause);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_newhybrid(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, allocty: CMuTypeNode, lenty: CMuTypeNode, length: CMuVarNode, exc_clause: CMuExcClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_allocty = from_MuID(allocty);
    let mut _arg_lenty = from_MuID(lenty);
    let mut _arg_length = from_MuID(length);
    let mut _arg_exc_clause = from_MuID_optional(exc_clause);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_alloca(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, allocty: CMuTypeNode, exc_clause: CMuExcClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_allocty = from_MuID(allocty);
    let mut _arg_exc_clause = from_MuID_optional(exc_clause);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_allocahybrid(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, allocty: CMuTypeNode, lenty: CMuTypeNode, length: CMuVarNode, exc_clause: CMuExcClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_allocty = from_MuID(allocty);
    let mut _arg_lenty = from_MuID(lenty);
    let mut _arg_length = from_MuID(length);
    let mut _arg_exc_clause = from_MuID_optional(exc_clause);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_getiref(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, refty: CMuTypeNode, opnd: CMuVarNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_refty = from_MuID(refty);
    let mut _arg_opnd = from_MuID(opnd);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_getfieldiref(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, is_ptr: CMuBool, refty: CMuTypeNode, index: c_int, opnd: CMuVarNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_is_ptr = from_MuBool(is_ptr);
    let mut _arg_refty = from_MuID(refty);
    let mut _arg_index = index;
    let mut _arg_opnd = from_MuID(opnd);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_getelemiref(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, is_ptr: CMuBool, refty: CMuTypeNode, indty: CMuTypeNode, opnd: CMuVarNode, index: CMuVarNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_is_ptr = from_MuBool(is_ptr);
    let mut _arg_refty = from_MuID(refty);
    let mut _arg_indty = from_MuID(indty);
    let mut _arg_opnd = from_MuID(opnd);
    let mut _arg_index = from_MuID(index);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_shiftiref(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, is_ptr: CMuBool, refty: CMuTypeNode, offty: CMuTypeNode, opnd: CMuVarNode, offset: CMuVarNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_is_ptr = from_MuBool(is_ptr);
    let mut _arg_refty = from_MuID(refty);
    let mut _arg_offty = from_MuID(offty);
    let mut _arg_opnd = from_MuID(opnd);
    let mut _arg_offset = from_MuID(offset);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_getvarpartiref(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, is_ptr: CMuBool, refty: CMuTypeNode, opnd: CMuVarNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_is_ptr = from_MuBool(is_ptr);
    let mut _arg_refty = from_MuID(refty);
    let mut _arg_opnd = from_MuID(opnd);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_load(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, is_ptr: CMuBool, ord: CMuMemOrd, refty: CMuTypeNode, loc: CMuVarNode, exc_clause: CMuExcClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_is_ptr = from_MuBool(is_ptr);
    let mut _arg_ord = ord;
    let mut _arg_refty = from_MuID(refty);
    let mut _arg_loc = from_MuID(loc);
    let mut _arg_exc_clause = from_MuID_optional(exc_clause);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_store(b: *mut CMuIRBuilder, id: CMuID, is_ptr: CMuBool, ord: CMuMemOrd, refty: CMuTypeNode, loc: CMuVarNode, newval: CMuVarNode, exc_clause: CMuExcClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_is_ptr = from_MuBool(is_ptr);
    let mut _arg_ord = ord;
    let mut _arg_refty = from_MuID(refty);
    let mut _arg_loc = from_MuID(loc);
    let mut _arg_newval = from_MuID(newval);
    let mut _arg_exc_clause = from_MuID_optional(exc_clause);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_cmpxchg(b: *mut CMuIRBuilder, id: CMuID, value_result_id: CMuID, succ_result_id: CMuID, is_ptr: CMuBool, is_weak: CMuBool, ord_succ: CMuMemOrd, ord_fail: CMuMemOrd, refty: CMuTypeNode, loc: CMuVarNode, expected: CMuVarNode, desired: CMuVarNode, exc_clause: CMuExcClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_value_result_id = from_MuID(value_result_id);
    let mut _arg_succ_result_id = from_MuID(succ_result_id);
    let mut _arg_is_ptr = from_MuBool(is_ptr);
    let mut _arg_is_weak = from_MuBool(is_weak);
    let mut _arg_ord_succ = ord_succ;
    let mut _arg_ord_fail = ord_fail;
    let mut _arg_refty = from_MuID(refty);
    let mut _arg_loc = from_MuID(loc);
    let mut _arg_expected = from_MuID(expected);
    let mut _arg_desired = from_MuID(desired);
    let mut _arg_exc_clause = from_MuID_optional(exc_clause);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_atomicrmw(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, is_ptr: CMuBool, ord: CMuMemOrd, optr: CMuAtomicRMWOptr, refTy: CMuTypeNode, loc: CMuVarNode, opnd: CMuVarNode, exc_clause: CMuExcClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_is_ptr = from_MuBool(is_ptr);
    let mut _arg_ord = ord;
    let mut _arg_optr = optr;
    let mut _arg_refTy = from_MuID(refTy);
    let mut _arg_loc = from_MuID(loc);
    let mut _arg_opnd = from_MuID(opnd);
    let mut _arg_exc_clause = from_MuID_optional(exc_clause);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_fence(b: *mut CMuIRBuilder, id: CMuID, ord: CMuMemOrd) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_ord = ord;
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_trap(b: *mut CMuIRBuilder, id: CMuID, result_ids: *mut CMuID, rettys: *mut CMuTypeNode, nretvals: CMuArraySize, exc_clause: CMuExcClause, keepalive_clause: CMuKeepaliveClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_ids = from_MuID_array(result_ids, nretvals);
    let mut _arg_rettys = from_MuID_array(rettys, nretvals);
    let mut _arg_exc_clause = from_MuID_optional(exc_clause);
    let mut _arg_keepalive_clause = from_MuID_optional(keepalive_clause);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_watchpoint(b: *mut CMuIRBuilder, id: CMuID, wpid: CMuWPID, result_ids: *mut CMuID, rettys: *mut CMuTypeNode, nretvals: CMuArraySize, dis: CMuDestClause, ena: CMuDestClause, exc: CMuDestClause, keepalive_clause: CMuKeepaliveClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_wpid = wpid;
    let mut _arg_result_ids = from_MuID_array(result_ids, nretvals);
    let mut _arg_rettys = from_MuID_array(rettys, nretvals);
    let mut _arg_dis = from_MuID(dis);
    let mut _arg_ena = from_MuID(ena);
    let mut _arg_exc = from_MuID_optional(exc);
    let mut _arg_keepalive_clause = from_MuID_optional(keepalive_clause);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_wpbranch(b: *mut CMuIRBuilder, id: CMuID, wpid: CMuWPID, dis: CMuDestClause, ena: CMuDestClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_wpid = wpid;
    let mut _arg_dis = from_MuID(dis);
    let mut _arg_ena = from_MuID(ena);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_ccall(b: *mut CMuIRBuilder, id: CMuID, result_ids: *mut CMuID, n_result_ids: CMuArraySize, callconv: CMuCallConv, callee_ty: CMuTypeNode, sig: CMuFuncSigNode, callee: CMuVarNode, args: *mut CMuVarNode, nargs: CMuArraySize, exc_clause: CMuExcClause, keepalive_clause: CMuKeepaliveClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_ids = from_MuID_array(result_ids, n_result_ids);
    let mut _arg_callconv = callconv;
    let mut _arg_callee_ty = from_MuID(callee_ty);
    let mut _arg_sig = from_MuID(sig);
    let mut _arg_callee = from_MuID(callee);
    let mut _arg_args = from_MuID_array(args, nargs);
    let mut _arg_exc_clause = from_MuID_optional(exc_clause);
    let mut _arg_keepalive_clause = from_MuID_optional(keepalive_clause);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_newthread(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, stack: CMuVarNode, threadlocal: CMuVarNode, new_stack_clause: CMuNewStackClause, exc_clause: CMuExcClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_stack = from_MuID(stack);
    let mut _arg_threadlocal = from_MuID_optional(threadlocal);
    let mut _arg_new_stack_clause = from_MuID(new_stack_clause);
    let mut _arg_exc_clause = from_MuID_optional(exc_clause);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_swapstack(b: *mut CMuIRBuilder, id: CMuID, result_ids: *mut CMuID, n_result_ids: CMuArraySize, swappee: CMuVarNode, cur_stack_clause: CMuCurStackClause, new_stack_clause: CMuNewStackClause, exc_clause: CMuExcClause, keepalive_clause: CMuKeepaliveClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_ids = from_MuID_array(result_ids, n_result_ids);
    let mut _arg_swappee = from_MuID(swappee);
    let mut _arg_cur_stack_clause = from_MuID(cur_stack_clause);
    let mut _arg_new_stack_clause = from_MuID(new_stack_clause);
    let mut _arg_exc_clause = from_MuID_optional(exc_clause);
    let mut _arg_keepalive_clause = from_MuID_optional(keepalive_clause);
    panic!("not implemented")
}

extern fn _forwarder__MuIRBuilder__new_comminst(b: *mut CMuIRBuilder, id: CMuID, result_ids: *mut CMuID, n_result_ids: CMuArraySize, opcode: CMuCommInst, flags: *mut CMuFlag, nflags: CMuArraySize, tys: *mut CMuTypeNode, ntys: CMuArraySize, sigs: *mut CMuFuncSigNode, nsigs: CMuArraySize, args: *mut CMuVarNode, nargs: CMuArraySize, exc_clause: CMuExcClause, keepalive_clause: CMuKeepaliveClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_ids = from_MuID_array(result_ids, n_result_ids);
    let mut _arg_opcode = opcode;
    let mut _arg_flags = from_MuFlag_array(flags, nflags);
    let mut _arg_tys = from_MuID_array(tys, ntys);
    let mut _arg_sigs = from_MuID_array(sigs, nsigs);
    let mut _arg_args = from_MuID_array(args, nargs);
    let mut _arg_exc_clause = from_MuID_optional(exc_clause);
    let mut _arg_keepalive_clause = from_MuID_optional(keepalive_clause);
    panic!("not implemented")
}
// GEN:END:Forwarders

// GEN:BEGIN:Fillers
pub fn make_new_MuVM(header: *mut c_void) -> *mut CMuVM {
    let box = Box::new(CMuVM {
        header: header,
        new_context: _forwarder__MuVM__new_context,
        id_of: _forwarder__MuVM__id_of,
        name_of: _forwarder__MuVM__name_of,
        set_trap_handler: _forwarder__MuVM__set_trap_handler,
    });

    Box::into_raw(box)
}

pub fn make_new_MuCtx(header: *mut c_void) -> *mut CMuCtx {
    let box = Box::new(CMuCtx {
        header: header,
        id_of: _forwarder__MuCtx__id_of,
        name_of: _forwarder__MuCtx__name_of,
        close_context: _forwarder__MuCtx__close_context,
        load_bundle: _forwarder__MuCtx__load_bundle,
        load_hail: _forwarder__MuCtx__load_hail,
        handle_from_sint8: _forwarder__MuCtx__handle_from_sint8,
        handle_from_uint8: _forwarder__MuCtx__handle_from_uint8,
        handle_from_sint16: _forwarder__MuCtx__handle_from_sint16,
        handle_from_uint16: _forwarder__MuCtx__handle_from_uint16,
        handle_from_sint32: _forwarder__MuCtx__handle_from_sint32,
        handle_from_uint32: _forwarder__MuCtx__handle_from_uint32,
        handle_from_sint64: _forwarder__MuCtx__handle_from_sint64,
        handle_from_uint64: _forwarder__MuCtx__handle_from_uint64,
        handle_from_uint64s: _forwarder__MuCtx__handle_from_uint64s,
        handle_from_float: _forwarder__MuCtx__handle_from_float,
        handle_from_double: _forwarder__MuCtx__handle_from_double,
        handle_from_ptr: _forwarder__MuCtx__handle_from_ptr,
        handle_from_fp: _forwarder__MuCtx__handle_from_fp,
        handle_to_sint8: _forwarder__MuCtx__handle_to_sint8,
        handle_to_uint8: _forwarder__MuCtx__handle_to_uint8,
        handle_to_sint16: _forwarder__MuCtx__handle_to_sint16,
        handle_to_uint16: _forwarder__MuCtx__handle_to_uint16,
        handle_to_sint32: _forwarder__MuCtx__handle_to_sint32,
        handle_to_uint32: _forwarder__MuCtx__handle_to_uint32,
        handle_to_sint64: _forwarder__MuCtx__handle_to_sint64,
        handle_to_uint64: _forwarder__MuCtx__handle_to_uint64,
        handle_to_float: _forwarder__MuCtx__handle_to_float,
        handle_to_double: _forwarder__MuCtx__handle_to_double,
        handle_to_ptr: _forwarder__MuCtx__handle_to_ptr,
        handle_to_fp: _forwarder__MuCtx__handle_to_fp,
        handle_from_const: _forwarder__MuCtx__handle_from_const,
        handle_from_global: _forwarder__MuCtx__handle_from_global,
        handle_from_func: _forwarder__MuCtx__handle_from_func,
        handle_from_expose: _forwarder__MuCtx__handle_from_expose,
        delete_value: _forwarder__MuCtx__delete_value,
        ref_eq: _forwarder__MuCtx__ref_eq,
        ref_ult: _forwarder__MuCtx__ref_ult,
        extract_value: _forwarder__MuCtx__extract_value,
        insert_value: _forwarder__MuCtx__insert_value,
        extract_element: _forwarder__MuCtx__extract_element,
        insert_element: _forwarder__MuCtx__insert_element,
        new_fixed: _forwarder__MuCtx__new_fixed,
        new_hybrid: _forwarder__MuCtx__new_hybrid,
        refcast: _forwarder__MuCtx__refcast,
        get_iref: _forwarder__MuCtx__get_iref,
        get_field_iref: _forwarder__MuCtx__get_field_iref,
        get_elem_iref: _forwarder__MuCtx__get_elem_iref,
        shift_iref: _forwarder__MuCtx__shift_iref,
        get_var_part_iref: _forwarder__MuCtx__get_var_part_iref,
        load: _forwarder__MuCtx__load,
        store: _forwarder__MuCtx__store,
        cmpxchg: _forwarder__MuCtx__cmpxchg,
        atomicrmw: _forwarder__MuCtx__atomicrmw,
        fence: _forwarder__MuCtx__fence,
        new_stack: _forwarder__MuCtx__new_stack,
        new_thread_nor: _forwarder__MuCtx__new_thread_nor,
        new_thread_exc: _forwarder__MuCtx__new_thread_exc,
        kill_stack: _forwarder__MuCtx__kill_stack,
        set_threadlocal: _forwarder__MuCtx__set_threadlocal,
        get_threadlocal: _forwarder__MuCtx__get_threadlocal,
        new_cursor: _forwarder__MuCtx__new_cursor,
        next_frame: _forwarder__MuCtx__next_frame,
        copy_cursor: _forwarder__MuCtx__copy_cursor,
        close_cursor: _forwarder__MuCtx__close_cursor,
        cur_func: _forwarder__MuCtx__cur_func,
        cur_func_ver: _forwarder__MuCtx__cur_func_ver,
        cur_inst: _forwarder__MuCtx__cur_inst,
        dump_keepalives: _forwarder__MuCtx__dump_keepalives,
        pop_frames_to: _forwarder__MuCtx__pop_frames_to,
        push_frame: _forwarder__MuCtx__push_frame,
        tr64_is_fp: _forwarder__MuCtx__tr64_is_fp,
        tr64_is_int: _forwarder__MuCtx__tr64_is_int,
        tr64_is_ref: _forwarder__MuCtx__tr64_is_ref,
        tr64_to_fp: _forwarder__MuCtx__tr64_to_fp,
        tr64_to_int: _forwarder__MuCtx__tr64_to_int,
        tr64_to_ref: _forwarder__MuCtx__tr64_to_ref,
        tr64_to_tag: _forwarder__MuCtx__tr64_to_tag,
        tr64_from_fp: _forwarder__MuCtx__tr64_from_fp,
        tr64_from_int: _forwarder__MuCtx__tr64_from_int,
        tr64_from_ref: _forwarder__MuCtx__tr64_from_ref,
        enable_watchpoint: _forwarder__MuCtx__enable_watchpoint,
        disable_watchpoint: _forwarder__MuCtx__disable_watchpoint,
        pin: _forwarder__MuCtx__pin,
        unpin: _forwarder__MuCtx__unpin,
        get_addr: _forwarder__MuCtx__get_addr,
        expose: _forwarder__MuCtx__expose,
        unexpose: _forwarder__MuCtx__unexpose,
        new_ir_builder: _forwarder__MuCtx__new_ir_builder,
        make_boot_image: _forwarder__MuCtx__make_boot_image,
    });

    Box::into_raw(box)
}

pub fn make_new_MuIRBuilder(header: *mut c_void) -> *mut CMuIRBuilder {
    let box = Box::new(CMuIRBuilder {
        header: header,
        load: _forwarder__MuIRBuilder__load,
        abort: _forwarder__MuIRBuilder__abort,
        gen_sym: _forwarder__MuIRBuilder__gen_sym,
        new_type_int: _forwarder__MuIRBuilder__new_type_int,
        new_type_float: _forwarder__MuIRBuilder__new_type_float,
        new_type_double: _forwarder__MuIRBuilder__new_type_double,
        new_type_uptr: _forwarder__MuIRBuilder__new_type_uptr,
        new_type_ufuncptr: _forwarder__MuIRBuilder__new_type_ufuncptr,
        new_type_struct: _forwarder__MuIRBuilder__new_type_struct,
        new_type_hybrid: _forwarder__MuIRBuilder__new_type_hybrid,
        new_type_array: _forwarder__MuIRBuilder__new_type_array,
        new_type_vector: _forwarder__MuIRBuilder__new_type_vector,
        new_type_void: _forwarder__MuIRBuilder__new_type_void,
        new_type_ref: _forwarder__MuIRBuilder__new_type_ref,
        new_type_iref: _forwarder__MuIRBuilder__new_type_iref,
        new_type_weakref: _forwarder__MuIRBuilder__new_type_weakref,
        new_type_funcref: _forwarder__MuIRBuilder__new_type_funcref,
        new_type_tagref64: _forwarder__MuIRBuilder__new_type_tagref64,
        new_type_threadref: _forwarder__MuIRBuilder__new_type_threadref,
        new_type_stackref: _forwarder__MuIRBuilder__new_type_stackref,
        new_type_framecursorref: _forwarder__MuIRBuilder__new_type_framecursorref,
        new_type_irbuilderref: _forwarder__MuIRBuilder__new_type_irbuilderref,
        new_funcsig: _forwarder__MuIRBuilder__new_funcsig,
        new_const_int: _forwarder__MuIRBuilder__new_const_int,
        new_const_int_ex: _forwarder__MuIRBuilder__new_const_int_ex,
        new_const_float: _forwarder__MuIRBuilder__new_const_float,
        new_const_double: _forwarder__MuIRBuilder__new_const_double,
        new_const_null: _forwarder__MuIRBuilder__new_const_null,
        new_const_seq: _forwarder__MuIRBuilder__new_const_seq,
        new_const_extern: _forwarder__MuIRBuilder__new_const_extern,
        new_global_cell: _forwarder__MuIRBuilder__new_global_cell,
        new_func: _forwarder__MuIRBuilder__new_func,
        new_exp_func: _forwarder__MuIRBuilder__new_exp_func,
        new_func_ver: _forwarder__MuIRBuilder__new_func_ver,
        new_bb: _forwarder__MuIRBuilder__new_bb,
        new_dest_clause: _forwarder__MuIRBuilder__new_dest_clause,
        new_exc_clause: _forwarder__MuIRBuilder__new_exc_clause,
        new_keepalive_clause: _forwarder__MuIRBuilder__new_keepalive_clause,
        new_csc_ret_with: _forwarder__MuIRBuilder__new_csc_ret_with,
        new_csc_kill_old: _forwarder__MuIRBuilder__new_csc_kill_old,
        new_nsc_pass_values: _forwarder__MuIRBuilder__new_nsc_pass_values,
        new_nsc_throw_exc: _forwarder__MuIRBuilder__new_nsc_throw_exc,
        new_binop: _forwarder__MuIRBuilder__new_binop,
        new_binop_with_status: _forwarder__MuIRBuilder__new_binop_with_status,
        new_cmp: _forwarder__MuIRBuilder__new_cmp,
        new_conv: _forwarder__MuIRBuilder__new_conv,
        new_select: _forwarder__MuIRBuilder__new_select,
        new_branch: _forwarder__MuIRBuilder__new_branch,
        new_branch2: _forwarder__MuIRBuilder__new_branch2,
        new_switch: _forwarder__MuIRBuilder__new_switch,
        new_call: _forwarder__MuIRBuilder__new_call,
        new_tailcall: _forwarder__MuIRBuilder__new_tailcall,
        new_ret: _forwarder__MuIRBuilder__new_ret,
        new_throw: _forwarder__MuIRBuilder__new_throw,
        new_extractvalue: _forwarder__MuIRBuilder__new_extractvalue,
        new_insertvalue: _forwarder__MuIRBuilder__new_insertvalue,
        new_extractelement: _forwarder__MuIRBuilder__new_extractelement,
        new_insertelement: _forwarder__MuIRBuilder__new_insertelement,
        new_shufflevector: _forwarder__MuIRBuilder__new_shufflevector,
        new_new: _forwarder__MuIRBuilder__new_new,
        new_newhybrid: _forwarder__MuIRBuilder__new_newhybrid,
        new_alloca: _forwarder__MuIRBuilder__new_alloca,
        new_allocahybrid: _forwarder__MuIRBuilder__new_allocahybrid,
        new_getiref: _forwarder__MuIRBuilder__new_getiref,
        new_getfieldiref: _forwarder__MuIRBuilder__new_getfieldiref,
        new_getelemiref: _forwarder__MuIRBuilder__new_getelemiref,
        new_shiftiref: _forwarder__MuIRBuilder__new_shiftiref,
        new_getvarpartiref: _forwarder__MuIRBuilder__new_getvarpartiref,
        new_load: _forwarder__MuIRBuilder__new_load,
        new_store: _forwarder__MuIRBuilder__new_store,
        new_cmpxchg: _forwarder__MuIRBuilder__new_cmpxchg,
        new_atomicrmw: _forwarder__MuIRBuilder__new_atomicrmw,
        new_fence: _forwarder__MuIRBuilder__new_fence,
        new_trap: _forwarder__MuIRBuilder__new_trap,
        new_watchpoint: _forwarder__MuIRBuilder__new_watchpoint,
        new_wpbranch: _forwarder__MuIRBuilder__new_wpbranch,
        new_ccall: _forwarder__MuIRBuilder__new_ccall,
        new_newthread: _forwarder__MuIRBuilder__new_newthread,
        new_swapstack: _forwarder__MuIRBuilder__new_swapstack,
        new_comminst: _forwarder__MuIRBuilder__new_comminst,
    });

    Box::into_raw(box)
}
// GEN:END:Fillers
