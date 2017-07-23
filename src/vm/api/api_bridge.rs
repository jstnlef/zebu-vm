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

#![allow(non_snake_case)] // It's generated code.
#![allow(dead_code)] // Seems Rust do not consider "taking the function pointer" as "use"

//! This file contains the bridge between the C interface and the Rust implementation.
//! This file contains the the functions in the C-API's `MuVM`, `MuCtx` and `MuIRBuilder` structs.
//! These functions will convert the low-level C-style argument types to the high-level Rust-level
//! argument types, and attempt to call the methods of the same name on the corresponding Rust
//! structs in api_impl.
//!
//! NOTE: Parts of this file (between GEN:BEGIN:* and GEN:END:*) are automatically generated.
//! Do not edit those parts manually because they will be overwritten. Instead, edit the
//! muapi2rustapi.py script to generate the desired code.

use std::ptr;
use std::os::raw::*;
use std::ffi::CStr;
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
    unsafe { (*ptr).header as *mut MuVM }
}

#[inline(always)]
fn from_MuCtx_ptr<'v>(ptr: *mut CMuCtx) -> *mut MuCtx {
    debug_assert!(!ptr.is_null());
    unsafe { (*ptr).header as *mut MuCtx }
}

#[inline(always)]
fn from_MuIRBuilder_ptr<'c>(ptr: *mut CMuIRBuilder) -> *mut MuIRBuilder {
    debug_assert!(!ptr.is_null());
    unsafe { (*ptr).header as *mut MuIRBuilder }
}

#[inline(always)]
fn from_MuName(cname: CMuName) -> MuName {
    from_MuCString(cname)
}

#[inline(always)]
fn from_MuCString(cstring: CMuCString) -> String {
    debug_assert!(!cstring.is_null());
    let ffi_cstr = unsafe { CStr::from_ptr(cstring) };

    ffi_cstr.to_string_lossy().into_owned()
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
    cmuid as MuID // just zero extend
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

// APIHandle is immutable when used.
#[inline(always)]
fn from_handle<'a>(cmuvalue: CMuValue) -> &'a APIHandle {
    debug_assert!(!cmuvalue.is_null());
    unsafe { &*(cmuvalue as *const APIHandle) }
}

#[inline(always)]
fn from_handle_optional<'a>(cmuvalue: CMuValue) -> Option<&'a APIHandle> {
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
        unsafe { slice::from_raw_parts(ptr::null(), len) }
    } else {
        unsafe { slice::from_raw_parts(ptr, len) }
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
fn from_MuFlag_array<'a>(ptr: *const CMuFlag, len: usize) -> &'a [CMuFlag] {
    from_array_direct(ptr, len)
}

/// Convert into a Vec of handle refs. It is not certain whether refs are represented in the same
/// way as raw pointers. So this function will convert each element. This function is only called
/// by `new_thread_nor`. As always, thread creation dominates the time.
#[inline(always)]
fn from_handle_array<'a>(ptr: *const CMuValue, len: usize) -> Vec<&'a APIHandle> {
    let slc = from_array_direct(ptr, len);
    slc.iter()
        .map(|&e| {
            debug_assert!(!e.is_null());
            unsafe { &*(e as *const APIHandle) }
        })
        .collect::<Vec<_>>()
}

/// MuID is usize in this impl. Need conversion.
#[inline(always)]
fn from_MuID_array<'a>(ptr: *const CMuID, len: usize) -> Vec<MuID> {
    let slc = from_array_direct(ptr, len);
    slc.iter()
        .map(|&e| {
            debug_assert!(e != 0);
            e as MuID
        })
        .collect::<Vec<_>>()
}

#[inline(always)]
fn from_MuCString_array<'a>(ptr: *const CMuCString, len: usize) -> Vec<String> {
    let slc = from_array_direct(ptr, len);
    slc.iter().map(|&e| from_MuCString(e)).collect::<Vec<_>>()
}

// The following functions `to_*` converts high-level types to C-like types.

#[inline(always)]
fn to_MuID(value: MuID) -> CMuID {
    debug_assert!(value <= 0xFFFFFFFFusize);
    value as CMuID
}

#[inline(always)]
fn to_handle(muvalue: *const APIHandle) -> CMuValue {
    debug_assert!(!muvalue.is_null());
    muvalue as CMuValue
}

#[inline(always)]
fn to_MuBool(value: bool) -> CMuBool {
    if value {
        1
    } else {
        0
    }
}

// GEN:BEGIN:Forwarders
extern "C" fn _forwarder__MuVM__new_context(mvm: *mut CMuVM) -> *mut CMuCtx {
    let mut _arg_mvm = from_MuVM_ptr(mvm);
    let _rv = unsafe { (*_arg_mvm).new_context() };
    let _rv_prep = _rv;
    _rv_prep
}

extern "C" fn _forwarder__MuVM__id_of(mvm: *mut CMuVM, name: CMuName) -> CMuID {
    let mut _arg_mvm = from_MuVM_ptr(mvm);
    let mut _arg_name = from_MuName(name);
    let _rv = unsafe { (*_arg_mvm).id_of(_arg_name) };
    let _rv_prep = to_MuID(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuVM__name_of(mvm: *mut CMuVM, id: CMuID) -> CMuName {
    let mut _arg_mvm = from_MuVM_ptr(mvm);
    let mut _arg_id = from_MuID(id);
    let _rv = unsafe { (*_arg_mvm).name_of(_arg_id) };
    let _rv_prep = _rv;
    _rv_prep
}

extern "C" fn _forwarder__MuVM__set_trap_handler(
    mvm: *mut CMuVM,
    trap_handler: CMuTrapHandler,
    userdata: CMuCPtr,
) {
    let mut _arg_mvm = from_MuVM_ptr(mvm);
    let mut _arg_trap_handler = trap_handler;
    let mut _arg_userdata = userdata;
    unsafe { (*_arg_mvm).set_trap_handler(_arg_trap_handler, _arg_userdata) };
}

extern "C" fn _forwarder__MuVM__compile_to_sharedlib(
    mvm: *mut CMuVM,
    lib_name: CMuCString,
    extra_srcs: *mut CMuCString,
    n_extra_srcs: CMuArraySize,
) {
    let mut _arg_mvm = from_MuVM_ptr(mvm);
    let mut _arg_lib_name = from_MuCString(lib_name);
    let mut _arg_extra_srcs = from_MuCString_array(extra_srcs, n_extra_srcs);
    unsafe { (*_arg_mvm).compile_to_sharedlib(_arg_lib_name, _arg_extra_srcs) };
}

extern "C" fn _forwarder__MuVM__current_thread_as_mu_thread(mvm: *mut CMuVM, threadlocal: CMuCPtr) {
    let mut _arg_mvm = from_MuVM_ptr(mvm);
    let mut _arg_threadlocal = threadlocal;
    unsafe { (*_arg_mvm).current_thread_as_mu_thread(_arg_threadlocal) };
}

extern "C" fn _forwarder__MuCtx__id_of(ctx: *mut CMuCtx, name: CMuName) -> CMuID {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_name = from_MuName(name);
    let _rv = unsafe { (*_arg_ctx).id_of(_arg_name) };
    let _rv_prep = to_MuID(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__name_of(ctx: *mut CMuCtx, id: CMuID) -> CMuName {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_id = from_MuID(id);
    let _rv = unsafe { (*_arg_ctx).name_of(_arg_id) };
    let _rv_prep = _rv;
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__close_context(ctx: *mut CMuCtx) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    unsafe { (*_arg_ctx).close_context() };
}

extern "C" fn _forwarder__MuCtx__load_bundle(ctx: *mut CMuCtx, buf: *mut c_char, sz: CMuArraySize) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_buf = from_char_array(buf, sz);
    unsafe { (*_arg_ctx).load_bundle(_arg_buf) };
}

extern "C" fn _forwarder__MuCtx__load_hail(ctx: *mut CMuCtx, buf: *mut c_char, sz: CMuArraySize) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_buf = from_char_array(buf, sz);
    unsafe { (*_arg_ctx).load_hail(_arg_buf) };
}

extern "C" fn _forwarder__MuCtx__handle_from_sint8(
    ctx: *mut CMuCtx,
    num: i8,
    len: c_int,
) -> CMuIntValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_num = num;
    let mut _arg_len = len;
    let _rv = unsafe { (*_arg_ctx).handle_from_sint8(_arg_num, _arg_len) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__handle_from_uint8(
    ctx: *mut CMuCtx,
    num: u8,
    len: c_int,
) -> CMuIntValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_num = num;
    let mut _arg_len = len;
    let _rv = unsafe { (*_arg_ctx).handle_from_uint8(_arg_num, _arg_len) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__handle_from_sint16(
    ctx: *mut CMuCtx,
    num: i16,
    len: c_int,
) -> CMuIntValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_num = num;
    let mut _arg_len = len;
    let _rv = unsafe { (*_arg_ctx).handle_from_sint16(_arg_num, _arg_len) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__handle_from_uint16(
    ctx: *mut CMuCtx,
    num: u16,
    len: c_int,
) -> CMuIntValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_num = num;
    let mut _arg_len = len;
    let _rv = unsafe { (*_arg_ctx).handle_from_uint16(_arg_num, _arg_len) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__handle_from_sint32(
    ctx: *mut CMuCtx,
    num: i32,
    len: c_int,
) -> CMuIntValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_num = num;
    let mut _arg_len = len;
    let _rv = unsafe { (*_arg_ctx).handle_from_sint32(_arg_num, _arg_len) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__handle_from_uint32(
    ctx: *mut CMuCtx,
    num: u32,
    len: c_int,
) -> CMuIntValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_num = num;
    let mut _arg_len = len;
    let _rv = unsafe { (*_arg_ctx).handle_from_uint32(_arg_num, _arg_len) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__handle_from_sint64(
    ctx: *mut CMuCtx,
    num: i64,
    len: c_int,
) -> CMuIntValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_num = num;
    let mut _arg_len = len;
    let _rv = unsafe { (*_arg_ctx).handle_from_sint64(_arg_num, _arg_len) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__handle_from_uint64(
    ctx: *mut CMuCtx,
    num: u64,
    len: c_int,
) -> CMuIntValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_num = num;
    let mut _arg_len = len;
    let _rv = unsafe { (*_arg_ctx).handle_from_uint64(_arg_num, _arg_len) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__handle_from_uint64s(
    ctx: *mut CMuCtx,
    nums: *mut u64,
    nnums: CMuArraySize,
    len: c_int,
) -> CMuIntValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_nums = from_uint64_t_array(nums, nnums);
    let mut _arg_len = len;
    let _rv = unsafe { (*_arg_ctx).handle_from_uint64s(_arg_nums, _arg_len) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__handle_from_float(ctx: *mut CMuCtx, num: f32) -> CMuFloatValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_num = num;
    let _rv = unsafe { (*_arg_ctx).handle_from_float(_arg_num) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__handle_from_double(ctx: *mut CMuCtx, num: f64) -> CMuDoubleValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_num = num;
    let _rv = unsafe { (*_arg_ctx).handle_from_double(_arg_num) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__handle_from_ptr(
    ctx: *mut CMuCtx,
    mu_type: CMuID,
    ptr: CMuCPtr,
) -> CMuUPtrValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_mu_type = from_MuID(mu_type);
    let mut _arg_ptr = ptr;
    let _rv = unsafe { (*_arg_ctx).handle_from_ptr(_arg_mu_type, _arg_ptr) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__handle_from_fp(
    ctx: *mut CMuCtx,
    mu_type: CMuID,
    fp: CMuCFP,
) -> CMuUFPValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_mu_type = from_MuID(mu_type);
    let mut _arg_fp = fp;
    let _rv = unsafe { (*_arg_ctx).handle_from_fp(_arg_mu_type, _arg_fp) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__handle_to_sint8(ctx: *mut CMuCtx, opnd: CMuIntValue) -> i8 {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_handle(opnd);
    let _rv = unsafe { (*_arg_ctx).handle_to_sint8(_arg_opnd) };
    let _rv_prep = _rv;
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__handle_to_uint8(ctx: *mut CMuCtx, opnd: CMuIntValue) -> u8 {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_handle(opnd);
    let _rv = unsafe { (*_arg_ctx).handle_to_uint8(_arg_opnd) };
    let _rv_prep = _rv;
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__handle_to_sint16(ctx: *mut CMuCtx, opnd: CMuIntValue) -> i16 {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_handle(opnd);
    let _rv = unsafe { (*_arg_ctx).handle_to_sint16(_arg_opnd) };
    let _rv_prep = _rv;
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__handle_to_uint16(ctx: *mut CMuCtx, opnd: CMuIntValue) -> u16 {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_handle(opnd);
    let _rv = unsafe { (*_arg_ctx).handle_to_uint16(_arg_opnd) };
    let _rv_prep = _rv;
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__handle_to_sint32(ctx: *mut CMuCtx, opnd: CMuIntValue) -> i32 {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_handle(opnd);
    let _rv = unsafe { (*_arg_ctx).handle_to_sint32(_arg_opnd) };
    let _rv_prep = _rv;
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__handle_to_uint32(ctx: *mut CMuCtx, opnd: CMuIntValue) -> u32 {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_handle(opnd);
    let _rv = unsafe { (*_arg_ctx).handle_to_uint32(_arg_opnd) };
    let _rv_prep = _rv;
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__handle_to_sint64(ctx: *mut CMuCtx, opnd: CMuIntValue) -> i64 {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_handle(opnd);
    let _rv = unsafe { (*_arg_ctx).handle_to_sint64(_arg_opnd) };
    let _rv_prep = _rv;
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__handle_to_uint64(ctx: *mut CMuCtx, opnd: CMuIntValue) -> u64 {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_handle(opnd);
    let _rv = unsafe { (*_arg_ctx).handle_to_uint64(_arg_opnd) };
    let _rv_prep = _rv;
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__handle_to_float(ctx: *mut CMuCtx, opnd: CMuFloatValue) -> f32 {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_handle(opnd);
    let _rv = unsafe { (*_arg_ctx).handle_to_float(_arg_opnd) };
    let _rv_prep = _rv;
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__handle_to_double(ctx: *mut CMuCtx, opnd: CMuDoubleValue) -> f64 {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_handle(opnd);
    let _rv = unsafe { (*_arg_ctx).handle_to_double(_arg_opnd) };
    let _rv_prep = _rv;
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__handle_to_ptr(ctx: *mut CMuCtx, opnd: CMuUPtrValue) -> CMuCPtr {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_handle(opnd);
    let _rv = unsafe { (*_arg_ctx).handle_to_ptr(_arg_opnd) };
    let _rv_prep = _rv;
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__handle_to_fp(ctx: *mut CMuCtx, opnd: CMuUFPValue) -> CMuCFP {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_handle(opnd);
    let _rv = unsafe { (*_arg_ctx).handle_to_fp(_arg_opnd) };
    let _rv_prep = _rv;
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__handle_from_const(ctx: *mut CMuCtx, id: CMuID) -> CMuValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_id = from_MuID(id);
    let _rv = unsafe { (*_arg_ctx).handle_from_const(_arg_id) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__handle_from_global(ctx: *mut CMuCtx, id: CMuID) -> CMuIRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_id = from_MuID(id);
    let _rv = unsafe { (*_arg_ctx).handle_from_global(_arg_id) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__handle_from_func(ctx: *mut CMuCtx, id: CMuID) -> CMuFuncRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_id = from_MuID(id);
    let _rv = unsafe { (*_arg_ctx).handle_from_func(_arg_id) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__handle_from_expose(ctx: *mut CMuCtx, id: CMuID) -> CMuValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_id = from_MuID(id);
    let _rv = unsafe { (*_arg_ctx).handle_from_expose(_arg_id) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__delete_value(ctx: *mut CMuCtx, opnd: CMuValue) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_handle(opnd);
    unsafe { (*_arg_ctx).delete_value(_arg_opnd) };
}

extern "C" fn _forwarder__MuCtx__ref_eq(
    ctx: *mut CMuCtx,
    lhs: CMuGenRefValue,
    rhs: CMuGenRefValue,
) -> CMuBool {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_lhs = from_handle(lhs);
    let mut _arg_rhs = from_handle(rhs);
    let _rv = unsafe { (*_arg_ctx).ref_eq(_arg_lhs, _arg_rhs) };
    let _rv_prep = to_MuBool(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__ref_ult(
    ctx: *mut CMuCtx,
    lhs: CMuIRefValue,
    rhs: CMuIRefValue,
) -> CMuBool {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_lhs = from_handle(lhs);
    let mut _arg_rhs = from_handle(rhs);
    let _rv = unsafe { (*_arg_ctx).ref_ult(_arg_lhs, _arg_rhs) };
    let _rv_prep = to_MuBool(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__extract_value(
    ctx: *mut CMuCtx,
    str: CMuStructValue,
    index: c_int,
) -> CMuValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_str = from_handle(str);
    let mut _arg_index = index;
    let _rv = unsafe { (*_arg_ctx).extract_value(_arg_str, _arg_index) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__insert_value(
    ctx: *mut CMuCtx,
    str: CMuStructValue,
    index: c_int,
    newval: CMuValue,
) -> CMuStructValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_str = from_handle(str);
    let mut _arg_index = index;
    let mut _arg_newval = from_handle(newval);
    let _rv = unsafe { (*_arg_ctx).insert_value(_arg_str, _arg_index, _arg_newval) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__extract_element(
    ctx: *mut CMuCtx,
    str: CMuSeqValue,
    index: CMuIntValue,
) -> CMuValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_str = from_handle(str);
    let mut _arg_index = from_handle(index);
    let _rv = unsafe { (*_arg_ctx).extract_element(_arg_str, _arg_index) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__insert_element(
    ctx: *mut CMuCtx,
    str: CMuSeqValue,
    index: CMuIntValue,
    newval: CMuValue,
) -> CMuSeqValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_str = from_handle(str);
    let mut _arg_index = from_handle(index);
    let mut _arg_newval = from_handle(newval);
    let _rv = unsafe { (*_arg_ctx).insert_element(_arg_str, _arg_index, _arg_newval) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__new_fixed(ctx: *mut CMuCtx, mu_type: CMuID) -> CMuRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_mu_type = from_MuID(mu_type);
    let _rv = unsafe { (*_arg_ctx).new_fixed(_arg_mu_type) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__new_hybrid(
    ctx: *mut CMuCtx,
    mu_type: CMuID,
    length: CMuIntValue,
) -> CMuRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_mu_type = from_MuID(mu_type);
    let mut _arg_length = from_handle(length);
    let _rv = unsafe { (*_arg_ctx).new_hybrid(_arg_mu_type, _arg_length) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__refcast(
    ctx: *mut CMuCtx,
    opnd: CMuGenRefValue,
    new_type: CMuID,
) -> CMuGenRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_handle(opnd);
    let mut _arg_new_type = from_MuID(new_type);
    let _rv = unsafe { (*_arg_ctx).refcast(_arg_opnd, _arg_new_type) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__get_iref(ctx: *mut CMuCtx, opnd: CMuRefValue) -> CMuIRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_handle(opnd);
    let _rv = unsafe { (*_arg_ctx).get_iref(_arg_opnd) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__get_field_iref(
    ctx: *mut CMuCtx,
    opnd: CMuIRefValue,
    field: c_int,
) -> CMuIRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_handle(opnd);
    let mut _arg_field = field;
    let _rv = unsafe { (*_arg_ctx).get_field_iref(_arg_opnd, _arg_field) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__get_elem_iref(
    ctx: *mut CMuCtx,
    opnd: CMuIRefValue,
    index: CMuIntValue,
) -> CMuIRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_handle(opnd);
    let mut _arg_index = from_handle(index);
    let _rv = unsafe { (*_arg_ctx).get_elem_iref(_arg_opnd, _arg_index) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__shift_iref(
    ctx: *mut CMuCtx,
    opnd: CMuIRefValue,
    offset: CMuIntValue,
) -> CMuIRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_handle(opnd);
    let mut _arg_offset = from_handle(offset);
    let _rv = unsafe { (*_arg_ctx).shift_iref(_arg_opnd, _arg_offset) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__get_var_part_iref(
    ctx: *mut CMuCtx,
    opnd: CMuIRefValue,
) -> CMuIRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_handle(opnd);
    let _rv = unsafe { (*_arg_ctx).get_var_part_iref(_arg_opnd) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__load(
    ctx: *mut CMuCtx,
    ord: CMuMemOrd,
    loc: CMuIRefValue,
) -> CMuValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_ord = ord;
    let mut _arg_loc = from_handle(loc);
    let _rv = unsafe { (*_arg_ctx).load(_arg_ord, _arg_loc) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__store(
    ctx: *mut CMuCtx,
    ord: CMuMemOrd,
    loc: CMuIRefValue,
    newval: CMuValue,
) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_ord = ord;
    let mut _arg_loc = from_handle(loc);
    let mut _arg_newval = from_handle(newval);
    unsafe { (*_arg_ctx).store(_arg_ord, _arg_loc, _arg_newval) };
}

extern "C" fn _forwarder__MuCtx__cmpxchg(
    ctx: *mut CMuCtx,
    ord_succ: CMuMemOrd,
    ord_fail: CMuMemOrd,
    weak: CMuBool,
    loc: CMuIRefValue,
    expected: CMuValue,
    desired: CMuValue,
    is_succ: *mut CMuBool,
) -> CMuValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_ord_succ = ord_succ;
    let mut _arg_ord_fail = ord_fail;
    let mut _arg_weak = from_MuBool(weak);
    let mut _arg_loc = from_handle(loc);
    let mut _arg_expected = from_handle(expected);
    let mut _arg_desired = from_handle(desired);
    let mut _arg_is_succ = is_succ;
    let _rv = unsafe {
        (*_arg_ctx).cmpxchg(
            _arg_ord_succ,
            _arg_ord_fail,
            _arg_weak,
            _arg_loc,
            _arg_expected,
            _arg_desired,
            _arg_is_succ,
        )
    };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__atomicrmw(
    ctx: *mut CMuCtx,
    ord: CMuMemOrd,
    op: CMuAtomicRMWOptr,
    loc: CMuIRefValue,
    opnd: CMuValue,
) -> CMuValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_ord = ord;
    let mut _arg_op = op;
    let mut _arg_loc = from_handle(loc);
    let mut _arg_opnd = from_handle(opnd);
    let _rv = unsafe { (*_arg_ctx).atomicrmw(_arg_ord, _arg_op, _arg_loc, _arg_opnd) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__fence(ctx: *mut CMuCtx, ord: CMuMemOrd) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_ord = ord;
    unsafe { (*_arg_ctx).fence(_arg_ord) };
}

extern "C" fn _forwarder__MuCtx__new_stack(
    ctx: *mut CMuCtx,
    func: CMuFuncRefValue,
) -> CMuStackRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_func = from_handle(func);
    let _rv = unsafe { (*_arg_ctx).new_stack(_arg_func) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__new_thread_nor(
    ctx: *mut CMuCtx,
    stack: CMuStackRefValue,
    threadlocal: CMuRefValue,
    vals: *mut CMuValue,
    nvals: CMuArraySize,
) -> CMuThreadRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_stack = from_handle(stack);
    let mut _arg_threadlocal = from_handle_optional(threadlocal);
    let mut _arg_vals = from_handle_array(vals, nvals);
    let _rv = unsafe { (*_arg_ctx).new_thread_nor(_arg_stack, _arg_threadlocal, _arg_vals) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__new_thread_exc(
    ctx: *mut CMuCtx,
    stack: CMuStackRefValue,
    threadlocal: CMuRefValue,
    exc: CMuRefValue,
) -> CMuThreadRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_stack = from_handle(stack);
    let mut _arg_threadlocal = from_handle_optional(threadlocal);
    let mut _arg_exc = from_handle(exc);
    let _rv = unsafe { (*_arg_ctx).new_thread_exc(_arg_stack, _arg_threadlocal, _arg_exc) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__kill_stack(ctx: *mut CMuCtx, stack: CMuStackRefValue) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_stack = from_handle(stack);
    unsafe { (*_arg_ctx).kill_stack(_arg_stack) };
}

extern "C" fn _forwarder__MuCtx__set_threadlocal(
    ctx: *mut CMuCtx,
    thread: CMuThreadRefValue,
    threadlocal: CMuRefValue,
) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_thread = from_handle(thread);
    let mut _arg_threadlocal = from_handle(threadlocal);
    unsafe { (*_arg_ctx).set_threadlocal(_arg_thread, _arg_threadlocal) };
}

extern "C" fn _forwarder__MuCtx__get_threadlocal(
    ctx: *mut CMuCtx,
    thread: CMuThreadRefValue,
) -> CMuRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_thread = from_handle(thread);
    let _rv = unsafe { (*_arg_ctx).get_threadlocal(_arg_thread) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__new_cursor(
    ctx: *mut CMuCtx,
    stack: CMuStackRefValue,
) -> CMuFCRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_stack = from_handle(stack);
    let _rv = unsafe { (*_arg_ctx).new_cursor(_arg_stack) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__next_frame(ctx: *mut CMuCtx, cursor: CMuFCRefValue) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_cursor = from_handle(cursor);
    unsafe { (*_arg_ctx).next_frame(_arg_cursor) };
}

extern "C" fn _forwarder__MuCtx__copy_cursor(
    ctx: *mut CMuCtx,
    cursor: CMuFCRefValue,
) -> CMuFCRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_cursor = from_handle(cursor);
    let _rv = unsafe { (*_arg_ctx).copy_cursor(_arg_cursor) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__close_cursor(ctx: *mut CMuCtx, cursor: CMuFCRefValue) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_cursor = from_handle(cursor);
    unsafe { (*_arg_ctx).close_cursor(_arg_cursor) };
}

extern "C" fn _forwarder__MuCtx__cur_func(ctx: *mut CMuCtx, cursor: CMuFCRefValue) -> CMuID {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_cursor = from_handle(cursor);
    let _rv = unsafe { (*_arg_ctx).cur_func(_arg_cursor) };
    let _rv_prep = to_MuID(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__cur_func_ver(ctx: *mut CMuCtx, cursor: CMuFCRefValue) -> CMuID {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_cursor = from_handle(cursor);
    let _rv = unsafe { (*_arg_ctx).cur_func_ver(_arg_cursor) };
    let _rv_prep = to_MuID(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__cur_inst(ctx: *mut CMuCtx, cursor: CMuFCRefValue) -> CMuID {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_cursor = from_handle(cursor);
    let _rv = unsafe { (*_arg_ctx).cur_inst(_arg_cursor) };
    let _rv_prep = to_MuID(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__dump_keepalives(
    ctx: *mut CMuCtx,
    cursor: CMuFCRefValue,
    results: *mut CMuValue,
) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_cursor = from_handle(cursor);
    let mut _arg_results = results;
    unsafe { (*_arg_ctx).dump_keepalives(_arg_cursor, _arg_results) };
}

extern "C" fn _forwarder__MuCtx__pop_frames_to(ctx: *mut CMuCtx, cursor: CMuFCRefValue) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_cursor = from_handle(cursor);
    unsafe { (*_arg_ctx).pop_frames_to(_arg_cursor) };
}

extern "C" fn _forwarder__MuCtx__push_frame(
    ctx: *mut CMuCtx,
    stack: CMuStackRefValue,
    func: CMuFuncRefValue,
) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_stack = from_handle(stack);
    let mut _arg_func = from_handle(func);
    unsafe { (*_arg_ctx).push_frame(_arg_stack, _arg_func) };
}

extern "C" fn _forwarder__MuCtx__tr64_is_fp(ctx: *mut CMuCtx, value: CMuTagRef64Value) -> CMuBool {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_value = from_handle(value);
    let _rv = unsafe { (*_arg_ctx).tr64_is_fp(_arg_value) };
    let _rv_prep = to_MuBool(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__tr64_is_int(ctx: *mut CMuCtx, value: CMuTagRef64Value) -> CMuBool {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_value = from_handle(value);
    let _rv = unsafe { (*_arg_ctx).tr64_is_int(_arg_value) };
    let _rv_prep = to_MuBool(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__tr64_is_ref(ctx: *mut CMuCtx, value: CMuTagRef64Value) -> CMuBool {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_value = from_handle(value);
    let _rv = unsafe { (*_arg_ctx).tr64_is_ref(_arg_value) };
    let _rv_prep = to_MuBool(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__tr64_to_fp(
    ctx: *mut CMuCtx,
    value: CMuTagRef64Value,
) -> CMuDoubleValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_value = from_handle(value);
    let _rv = unsafe { (*_arg_ctx).tr64_to_fp(_arg_value) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__tr64_to_int(
    ctx: *mut CMuCtx,
    value: CMuTagRef64Value,
) -> CMuIntValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_value = from_handle(value);
    let _rv = unsafe { (*_arg_ctx).tr64_to_int(_arg_value) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__tr64_to_ref(
    ctx: *mut CMuCtx,
    value: CMuTagRef64Value,
) -> CMuRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_value = from_handle(value);
    let _rv = unsafe { (*_arg_ctx).tr64_to_ref(_arg_value) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__tr64_to_tag(
    ctx: *mut CMuCtx,
    value: CMuTagRef64Value,
) -> CMuIntValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_value = from_handle(value);
    let _rv = unsafe { (*_arg_ctx).tr64_to_tag(_arg_value) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__tr64_from_fp(
    ctx: *mut CMuCtx,
    value: CMuDoubleValue,
) -> CMuTagRef64Value {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_value = from_handle(value);
    let _rv = unsafe { (*_arg_ctx).tr64_from_fp(_arg_value) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__tr64_from_int(
    ctx: *mut CMuCtx,
    value: CMuIntValue,
) -> CMuTagRef64Value {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_value = from_handle(value);
    let _rv = unsafe { (*_arg_ctx).tr64_from_int(_arg_value) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__tr64_from_ref(
    ctx: *mut CMuCtx,
    reff: CMuRefValue,
    tag: CMuIntValue,
) -> CMuTagRef64Value {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_reff = from_handle(reff);
    let mut _arg_tag = from_handle(tag);
    let _rv = unsafe { (*_arg_ctx).tr64_from_ref(_arg_reff, _arg_tag) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__enable_watchpoint(ctx: *mut CMuCtx, wpid: CMuWPID) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_wpid = wpid;
    unsafe { (*_arg_ctx).enable_watchpoint(_arg_wpid) };
}

extern "C" fn _forwarder__MuCtx__disable_watchpoint(ctx: *mut CMuCtx, wpid: CMuWPID) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_wpid = wpid;
    unsafe { (*_arg_ctx).disable_watchpoint(_arg_wpid) };
}

extern "C" fn _forwarder__MuCtx__pin(ctx: *mut CMuCtx, loc: CMuValue) -> CMuUPtrValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_loc = from_handle(loc);
    let _rv = unsafe { (*_arg_ctx).pin(_arg_loc) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__unpin(ctx: *mut CMuCtx, loc: CMuValue) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_loc = from_handle(loc);
    unsafe { (*_arg_ctx).unpin(_arg_loc) };
}

extern "C" fn _forwarder__MuCtx__get_addr(ctx: *mut CMuCtx, loc: CMuValue) -> CMuUPtrValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_loc = from_handle(loc);
    let _rv = unsafe { (*_arg_ctx).get_addr(_arg_loc) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__expose(
    ctx: *mut CMuCtx,
    func: CMuFuncRefValue,
    call_conv: CMuCallConv,
    cookie: CMuIntValue,
) -> CMuValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_func = from_handle(func);
    let mut _arg_call_conv = call_conv;
    let mut _arg_cookie = from_handle(cookie);
    let _rv = unsafe { (*_arg_ctx).expose(_arg_func, _arg_call_conv, _arg_cookie) };
    let _rv_prep = to_handle(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__unexpose(
    ctx: *mut CMuCtx,
    call_conv: CMuCallConv,
    value: CMuValue,
) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_call_conv = call_conv;
    let mut _arg_value = from_handle(value);
    unsafe { (*_arg_ctx).unexpose(_arg_call_conv, _arg_value) };
}

extern "C" fn _forwarder__MuCtx__new_ir_builder(ctx: *mut CMuCtx) -> *mut CMuIRBuilder {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let _rv = unsafe { (*_arg_ctx).new_ir_builder() };
    let _rv_prep = _rv;
    _rv_prep
}

extern "C" fn _forwarder__MuCtx__make_boot_image(
    ctx: *mut CMuCtx,
    whitelist: *mut CMuID,
    whitelist_sz: CMuArraySize,
    primordial_func: CMuFuncRefValue,
    primordial_stack: CMuStackRefValue,
    primordial_threadlocal: CMuRefValue,
    sym_fields: *mut CMuIRefValue,
    sym_strings: *mut CMuCString,
    nsyms: CMuArraySize,
    reloc_fields: *mut CMuIRefValue,
    reloc_strings: *mut CMuCString,
    nrelocs: CMuArraySize,
    output_file: CMuCString,
) {
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
    unsafe {
        (*_arg_ctx).make_boot_image(
            _arg_whitelist,
            _arg_primordial_func,
            _arg_primordial_stack,
            _arg_primordial_threadlocal,
            _arg_sym_fields,
            _arg_sym_strings,
            _arg_reloc_fields,
            _arg_reloc_strings,
            _arg_output_file,
        )
    };
}

extern "C" fn _forwarder__MuIRBuilder__load(b: *mut CMuIRBuilder) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    unsafe { (*_arg_b).load() };
}

extern "C" fn _forwarder__MuIRBuilder__abort(b: *mut CMuIRBuilder) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    unsafe { (*_arg_b).abort() };
}

extern "C" fn _forwarder__MuIRBuilder__gen_sym(b: *mut CMuIRBuilder, name: CMuCString) -> CMuID {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_name = from_MuCString_optional(name);
    let _rv = unsafe { (*_arg_b).gen_sym(_arg_name) };
    let _rv_prep = to_MuID(_rv);
    _rv_prep
}

extern "C" fn _forwarder__MuIRBuilder__new_type_int(b: *mut CMuIRBuilder, id: CMuID, len: c_int) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_len = len;
    unsafe { (*_arg_b).new_type_int(_arg_id, _arg_len) };
}

extern "C" fn _forwarder__MuIRBuilder__new_type_float(b: *mut CMuIRBuilder, id: CMuID) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    unsafe { (*_arg_b).new_type_float(_arg_id) };
}

extern "C" fn _forwarder__MuIRBuilder__new_type_double(b: *mut CMuIRBuilder, id: CMuID) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    unsafe { (*_arg_b).new_type_double(_arg_id) };
}

extern "C" fn _forwarder__MuIRBuilder__new_type_uptr(
    b: *mut CMuIRBuilder,
    id: CMuID,
    ty: CMuTypeNode,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_ty = from_MuID(ty);
    unsafe { (*_arg_b).new_type_uptr(_arg_id, _arg_ty) };
}

extern "C" fn _forwarder__MuIRBuilder__new_type_ufuncptr(
    b: *mut CMuIRBuilder,
    id: CMuID,
    sig: CMuFuncSigNode,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_sig = from_MuID(sig);
    unsafe { (*_arg_b).new_type_ufuncptr(_arg_id, _arg_sig) };
}

extern "C" fn _forwarder__MuIRBuilder__new_type_struct(
    b: *mut CMuIRBuilder,
    id: CMuID,
    fieldtys: *mut CMuTypeNode,
    nfieldtys: CMuArraySize,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_fieldtys = from_MuID_array(fieldtys, nfieldtys);
    unsafe { (*_arg_b).new_type_struct(_arg_id, _arg_fieldtys) };
}

extern "C" fn _forwarder__MuIRBuilder__new_type_hybrid(
    b: *mut CMuIRBuilder,
    id: CMuID,
    fixedtys: *mut CMuTypeNode,
    nfixedtys: CMuArraySize,
    varty: CMuTypeNode,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_fixedtys = from_MuID_array(fixedtys, nfixedtys);
    let mut _arg_varty = from_MuID(varty);
    unsafe { (*_arg_b).new_type_hybrid(_arg_id, _arg_fixedtys, _arg_varty) };
}

extern "C" fn _forwarder__MuIRBuilder__new_type_array(
    b: *mut CMuIRBuilder,
    id: CMuID,
    elemty: CMuTypeNode,
    len: u64,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_elemty = from_MuID(elemty);
    let mut _arg_len = len;
    unsafe { (*_arg_b).new_type_array(_arg_id, _arg_elemty, _arg_len) };
}

extern "C" fn _forwarder__MuIRBuilder__new_type_vector(
    b: *mut CMuIRBuilder,
    id: CMuID,
    elemty: CMuTypeNode,
    len: u64,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_elemty = from_MuID(elemty);
    let mut _arg_len = len;
    unsafe { (*_arg_b).new_type_vector(_arg_id, _arg_elemty, _arg_len) };
}

extern "C" fn _forwarder__MuIRBuilder__new_type_void(b: *mut CMuIRBuilder, id: CMuID) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    unsafe { (*_arg_b).new_type_void(_arg_id) };
}

extern "C" fn _forwarder__MuIRBuilder__new_type_ref(
    b: *mut CMuIRBuilder,
    id: CMuID,
    ty: CMuTypeNode,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_ty = from_MuID(ty);
    unsafe { (*_arg_b).new_type_ref(_arg_id, _arg_ty) };
}

extern "C" fn _forwarder__MuIRBuilder__new_type_iref(
    b: *mut CMuIRBuilder,
    id: CMuID,
    ty: CMuTypeNode,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_ty = from_MuID(ty);
    unsafe { (*_arg_b).new_type_iref(_arg_id, _arg_ty) };
}

extern "C" fn _forwarder__MuIRBuilder__new_type_weakref(
    b: *mut CMuIRBuilder,
    id: CMuID,
    ty: CMuTypeNode,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_ty = from_MuID(ty);
    unsafe { (*_arg_b).new_type_weakref(_arg_id, _arg_ty) };
}

extern "C" fn _forwarder__MuIRBuilder__new_type_funcref(
    b: *mut CMuIRBuilder,
    id: CMuID,
    sig: CMuFuncSigNode,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_sig = from_MuID(sig);
    unsafe { (*_arg_b).new_type_funcref(_arg_id, _arg_sig) };
}

extern "C" fn _forwarder__MuIRBuilder__new_type_tagref64(b: *mut CMuIRBuilder, id: CMuID) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    unsafe { (*_arg_b).new_type_tagref64(_arg_id) };
}

extern "C" fn _forwarder__MuIRBuilder__new_type_threadref(b: *mut CMuIRBuilder, id: CMuID) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    unsafe { (*_arg_b).new_type_threadref(_arg_id) };
}

extern "C" fn _forwarder__MuIRBuilder__new_type_stackref(b: *mut CMuIRBuilder, id: CMuID) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    unsafe { (*_arg_b).new_type_stackref(_arg_id) };
}

extern "C" fn _forwarder__MuIRBuilder__new_type_framecursorref(b: *mut CMuIRBuilder, id: CMuID) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    unsafe { (*_arg_b).new_type_framecursorref(_arg_id) };
}

extern "C" fn _forwarder__MuIRBuilder__new_type_irbuilderref(b: *mut CMuIRBuilder, id: CMuID) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    unsafe { (*_arg_b).new_type_irbuilderref(_arg_id) };
}

extern "C" fn _forwarder__MuIRBuilder__new_funcsig(
    b: *mut CMuIRBuilder,
    id: CMuID,
    paramtys: *mut CMuTypeNode,
    nparamtys: CMuArraySize,
    rettys: *mut CMuTypeNode,
    nrettys: CMuArraySize,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_paramtys = from_MuID_array(paramtys, nparamtys);
    let mut _arg_rettys = from_MuID_array(rettys, nrettys);
    unsafe { (*_arg_b).new_funcsig(_arg_id, _arg_paramtys, _arg_rettys) };
}

extern "C" fn _forwarder__MuIRBuilder__new_const_int(
    b: *mut CMuIRBuilder,
    id: CMuID,
    ty: CMuTypeNode,
    value: u64,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_ty = from_MuID(ty);
    let mut _arg_value = value;
    unsafe { (*_arg_b).new_const_int(_arg_id, _arg_ty, _arg_value) };
}

extern "C" fn _forwarder__MuIRBuilder__new_const_int_ex(
    b: *mut CMuIRBuilder,
    id: CMuID,
    ty: CMuTypeNode,
    values: *mut u64,
    nvalues: CMuArraySize,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_ty = from_MuID(ty);
    let mut _arg_values = from_uint64_t_array(values, nvalues);
    unsafe { (*_arg_b).new_const_int_ex(_arg_id, _arg_ty, _arg_values) };
}

extern "C" fn _forwarder__MuIRBuilder__new_const_float(
    b: *mut CMuIRBuilder,
    id: CMuID,
    ty: CMuTypeNode,
    value: f32,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_ty = from_MuID(ty);
    let mut _arg_value = value;
    unsafe { (*_arg_b).new_const_float(_arg_id, _arg_ty, _arg_value) };
}

extern "C" fn _forwarder__MuIRBuilder__new_const_double(
    b: *mut CMuIRBuilder,
    id: CMuID,
    ty: CMuTypeNode,
    value: f64,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_ty = from_MuID(ty);
    let mut _arg_value = value;
    unsafe { (*_arg_b).new_const_double(_arg_id, _arg_ty, _arg_value) };
}

extern "C" fn _forwarder__MuIRBuilder__new_const_null(
    b: *mut CMuIRBuilder,
    id: CMuID,
    ty: CMuTypeNode,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_ty = from_MuID(ty);
    unsafe { (*_arg_b).new_const_null(_arg_id, _arg_ty) };
}

extern "C" fn _forwarder__MuIRBuilder__new_const_seq(
    b: *mut CMuIRBuilder,
    id: CMuID,
    ty: CMuTypeNode,
    elems: *mut CMuGlobalVarNode,
    nelems: CMuArraySize,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_ty = from_MuID(ty);
    let mut _arg_elems = from_MuID_array(elems, nelems);
    unsafe { (*_arg_b).new_const_seq(_arg_id, _arg_ty, _arg_elems) };
}

extern "C" fn _forwarder__MuIRBuilder__new_const_extern(
    b: *mut CMuIRBuilder,
    id: CMuID,
    ty: CMuTypeNode,
    symbol: CMuCString,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_ty = from_MuID(ty);
    let mut _arg_symbol = from_MuCString(symbol);
    unsafe { (*_arg_b).new_const_extern(_arg_id, _arg_ty, _arg_symbol) };
}

extern "C" fn _forwarder__MuIRBuilder__new_global_cell(
    b: *mut CMuIRBuilder,
    id: CMuID,
    ty: CMuTypeNode,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_ty = from_MuID(ty);
    unsafe { (*_arg_b).new_global_cell(_arg_id, _arg_ty) };
}

extern "C" fn _forwarder__MuIRBuilder__new_func(
    b: *mut CMuIRBuilder,
    id: CMuID,
    sig: CMuFuncSigNode,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_sig = from_MuID(sig);
    unsafe { (*_arg_b).new_func(_arg_id, _arg_sig) };
}

extern "C" fn _forwarder__MuIRBuilder__new_exp_func(
    b: *mut CMuIRBuilder,
    id: CMuID,
    func: CMuFuncNode,
    callconv: CMuCallConv,
    cookie: CMuConstNode,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_func = from_MuID(func);
    let mut _arg_callconv = callconv;
    let mut _arg_cookie = from_MuID(cookie);
    unsafe { (*_arg_b).new_exp_func(_arg_id, _arg_func, _arg_callconv, _arg_cookie) };
}

extern "C" fn _forwarder__MuIRBuilder__new_func_ver(
    b: *mut CMuIRBuilder,
    id: CMuID,
    func: CMuFuncNode,
    bbs: *mut CMuBBNode,
    nbbs: CMuArraySize,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_func = from_MuID(func);
    let mut _arg_bbs = from_MuID_array(bbs, nbbs);
    unsafe { (*_arg_b).new_func_ver(_arg_id, _arg_func, _arg_bbs) };
}

extern "C" fn _forwarder__MuIRBuilder__new_bb(
    b: *mut CMuIRBuilder,
    id: CMuID,
    nor_param_ids: *mut CMuID,
    nor_param_types: *mut CMuTypeNode,
    n_nor_params: CMuArraySize,
    exc_param_id: CMuID,
    insts: *mut CMuInstNode,
    ninsts: CMuArraySize,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_nor_param_ids = from_MuID_array(nor_param_ids, n_nor_params);
    let mut _arg_nor_param_types = from_MuID_array(nor_param_types, n_nor_params);
    let mut _arg_exc_param_id = from_MuID_optional(exc_param_id);
    let mut _arg_insts = from_MuID_array(insts, ninsts);
    unsafe {
        (*_arg_b).new_bb(
            _arg_id,
            _arg_nor_param_ids,
            _arg_nor_param_types,
            _arg_exc_param_id,
            _arg_insts,
        )
    };
}

extern "C" fn _forwarder__MuIRBuilder__new_dest_clause(
    b: *mut CMuIRBuilder,
    id: CMuID,
    dest: CMuBBNode,
    vars: *mut CMuVarNode,
    nvars: CMuArraySize,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_dest = from_MuID(dest);
    let mut _arg_vars = from_MuID_array(vars, nvars);
    unsafe { (*_arg_b).new_dest_clause(_arg_id, _arg_dest, _arg_vars) };
}

extern "C" fn _forwarder__MuIRBuilder__new_exc_clause(
    b: *mut CMuIRBuilder,
    id: CMuID,
    nor: CMuDestClause,
    exc: CMuDestClause,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_nor = from_MuID(nor);
    let mut _arg_exc = from_MuID(exc);
    unsafe { (*_arg_b).new_exc_clause(_arg_id, _arg_nor, _arg_exc) };
}

extern "C" fn _forwarder__MuIRBuilder__new_keepalive_clause(
    b: *mut CMuIRBuilder,
    id: CMuID,
    vars: *mut CMuLocalVarNode,
    nvars: CMuArraySize,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_vars = from_MuID_array(vars, nvars);
    unsafe { (*_arg_b).new_keepalive_clause(_arg_id, _arg_vars) };
}

extern "C" fn _forwarder__MuIRBuilder__new_csc_ret_with(
    b: *mut CMuIRBuilder,
    id: CMuID,
    rettys: *mut CMuTypeNode,
    nrettys: CMuArraySize,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_rettys = from_MuID_array(rettys, nrettys);
    unsafe { (*_arg_b).new_csc_ret_with(_arg_id, _arg_rettys) };
}

extern "C" fn _forwarder__MuIRBuilder__new_csc_kill_old(b: *mut CMuIRBuilder, id: CMuID) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    unsafe { (*_arg_b).new_csc_kill_old(_arg_id) };
}

extern "C" fn _forwarder__MuIRBuilder__new_nsc_pass_values(
    b: *mut CMuIRBuilder,
    id: CMuID,
    tys: *mut CMuTypeNode,
    vars: *mut CMuVarNode,
    ntysvars: CMuArraySize,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_tys = from_MuID_array(tys, ntysvars);
    let mut _arg_vars = from_MuID_array(vars, ntysvars);
    unsafe { (*_arg_b).new_nsc_pass_values(_arg_id, _arg_tys, _arg_vars) };
}

extern "C" fn _forwarder__MuIRBuilder__new_nsc_throw_exc(
    b: *mut CMuIRBuilder,
    id: CMuID,
    exc: CMuVarNode,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_exc = from_MuID(exc);
    unsafe { (*_arg_b).new_nsc_throw_exc(_arg_id, _arg_exc) };
}

extern "C" fn _forwarder__MuIRBuilder__new_binop(
    b: *mut CMuIRBuilder,
    id: CMuID,
    result_id: CMuID,
    optr: CMuBinOptr,
    ty: CMuTypeNode,
    opnd1: CMuVarNode,
    opnd2: CMuVarNode,
    exc_clause: CMuExcClause,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_optr = optr;
    let mut _arg_ty = from_MuID(ty);
    let mut _arg_opnd1 = from_MuID(opnd1);
    let mut _arg_opnd2 = from_MuID(opnd2);
    let mut _arg_exc_clause = from_MuID_optional(exc_clause);
    unsafe {
        (*_arg_b).new_binop(
            _arg_id,
            _arg_result_id,
            _arg_optr,
            _arg_ty,
            _arg_opnd1,
            _arg_opnd2,
            _arg_exc_clause,
        )
    };
}

extern "C" fn _forwarder__MuIRBuilder__new_binop_with_status(
    b: *mut CMuIRBuilder,
    id: CMuID,
    result_id: CMuID,
    status_result_ids: *mut CMuID,
    n_status_result_ids: CMuArraySize,
    optr: CMuBinOptr,
    status_flags: CMuBinOpStatus,
    ty: CMuTypeNode,
    opnd1: CMuVarNode,
    opnd2: CMuVarNode,
    exc_clause: CMuExcClause,
) {
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
    unsafe {
        (*_arg_b).new_binop_with_status(
            _arg_id,
            _arg_result_id,
            _arg_status_result_ids,
            _arg_optr,
            _arg_status_flags,
            _arg_ty,
            _arg_opnd1,
            _arg_opnd2,
            _arg_exc_clause,
        )
    };
}

extern "C" fn _forwarder__MuIRBuilder__new_cmp(
    b: *mut CMuIRBuilder,
    id: CMuID,
    result_id: CMuID,
    optr: CMuCmpOptr,
    ty: CMuTypeNode,
    opnd1: CMuVarNode,
    opnd2: CMuVarNode,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_optr = optr;
    let mut _arg_ty = from_MuID(ty);
    let mut _arg_opnd1 = from_MuID(opnd1);
    let mut _arg_opnd2 = from_MuID(opnd2);
    unsafe {
        (*_arg_b).new_cmp(
            _arg_id,
            _arg_result_id,
            _arg_optr,
            _arg_ty,
            _arg_opnd1,
            _arg_opnd2,
        )
    };
}

extern "C" fn _forwarder__MuIRBuilder__new_conv(
    b: *mut CMuIRBuilder,
    id: CMuID,
    result_id: CMuID,
    optr: CMuConvOptr,
    from_ty: CMuTypeNode,
    to_ty: CMuTypeNode,
    opnd: CMuVarNode,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_optr = optr;
    let mut _arg_from_ty = from_MuID(from_ty);
    let mut _arg_to_ty = from_MuID(to_ty);
    let mut _arg_opnd = from_MuID(opnd);
    unsafe {
        (*_arg_b).new_conv(
            _arg_id,
            _arg_result_id,
            _arg_optr,
            _arg_from_ty,
            _arg_to_ty,
            _arg_opnd,
        )
    };
}

extern "C" fn _forwarder__MuIRBuilder__new_select(
    b: *mut CMuIRBuilder,
    id: CMuID,
    result_id: CMuID,
    cond_ty: CMuTypeNode,
    opnd_ty: CMuTypeNode,
    cond: CMuVarNode,
    if_true: CMuVarNode,
    if_false: CMuVarNode,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_cond_ty = from_MuID(cond_ty);
    let mut _arg_opnd_ty = from_MuID(opnd_ty);
    let mut _arg_cond = from_MuID(cond);
    let mut _arg_if_true = from_MuID(if_true);
    let mut _arg_if_false = from_MuID(if_false);
    unsafe {
        (*_arg_b).new_select(
            _arg_id,
            _arg_result_id,
            _arg_cond_ty,
            _arg_opnd_ty,
            _arg_cond,
            _arg_if_true,
            _arg_if_false,
        )
    };
}

extern "C" fn _forwarder__MuIRBuilder__new_branch(
    b: *mut CMuIRBuilder,
    id: CMuID,
    dest: CMuDestClause,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_dest = from_MuID(dest);
    unsafe { (*_arg_b).new_branch(_arg_id, _arg_dest) };
}

extern "C" fn _forwarder__MuIRBuilder__new_branch2(
    b: *mut CMuIRBuilder,
    id: CMuID,
    cond: CMuVarNode,
    if_true: CMuDestClause,
    if_false: CMuDestClause,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_cond = from_MuID(cond);
    let mut _arg_if_true = from_MuID(if_true);
    let mut _arg_if_false = from_MuID(if_false);
    unsafe { (*_arg_b).new_branch2(_arg_id, _arg_cond, _arg_if_true, _arg_if_false) };
}

extern "C" fn _forwarder__MuIRBuilder__new_switch(
    b: *mut CMuIRBuilder,
    id: CMuID,
    opnd_ty: CMuTypeNode,
    opnd: CMuVarNode,
    default_dest: CMuDestClause,
    cases: *mut CMuConstNode,
    dests: *mut CMuDestClause,
    ncasesdests: CMuArraySize,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_opnd_ty = from_MuID(opnd_ty);
    let mut _arg_opnd = from_MuID(opnd);
    let mut _arg_default_dest = from_MuID(default_dest);
    let mut _arg_cases = from_MuID_array(cases, ncasesdests);
    let mut _arg_dests = from_MuID_array(dests, ncasesdests);
    unsafe {
        (*_arg_b).new_switch(
            _arg_id,
            _arg_opnd_ty,
            _arg_opnd,
            _arg_default_dest,
            _arg_cases,
            _arg_dests,
        )
    };
}

extern "C" fn _forwarder__MuIRBuilder__new_call(
    b: *mut CMuIRBuilder,
    id: CMuID,
    result_ids: *mut CMuID,
    n_result_ids: CMuArraySize,
    sig: CMuFuncSigNode,
    callee: CMuVarNode,
    args: *mut CMuVarNode,
    nargs: CMuArraySize,
    exc_clause: CMuExcClause,
    keepalive_clause: CMuKeepaliveClause,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_ids = from_MuID_array(result_ids, n_result_ids);
    let mut _arg_sig = from_MuID(sig);
    let mut _arg_callee = from_MuID(callee);
    let mut _arg_args = from_MuID_array(args, nargs);
    let mut _arg_exc_clause = from_MuID_optional(exc_clause);
    let mut _arg_keepalive_clause = from_MuID_optional(keepalive_clause);
    unsafe {
        (*_arg_b).new_call(
            _arg_id,
            _arg_result_ids,
            _arg_sig,
            _arg_callee,
            _arg_args,
            _arg_exc_clause,
            _arg_keepalive_clause,
        )
    };
}

extern "C" fn _forwarder__MuIRBuilder__new_tailcall(
    b: *mut CMuIRBuilder,
    id: CMuID,
    sig: CMuFuncSigNode,
    callee: CMuVarNode,
    args: *mut CMuVarNode,
    nargs: CMuArraySize,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_sig = from_MuID(sig);
    let mut _arg_callee = from_MuID(callee);
    let mut _arg_args = from_MuID_array(args, nargs);
    unsafe { (*_arg_b).new_tailcall(_arg_id, _arg_sig, _arg_callee, _arg_args) };
}

extern "C" fn _forwarder__MuIRBuilder__new_ret(
    b: *mut CMuIRBuilder,
    id: CMuID,
    rvs: *mut CMuVarNode,
    nrvs: CMuArraySize,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_rvs = from_MuID_array(rvs, nrvs);
    unsafe { (*_arg_b).new_ret(_arg_id, _arg_rvs) };
}

extern "C" fn _forwarder__MuIRBuilder__new_throw(b: *mut CMuIRBuilder, id: CMuID, exc: CMuVarNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_exc = from_MuID(exc);
    unsafe { (*_arg_b).new_throw(_arg_id, _arg_exc) };
}

extern "C" fn _forwarder__MuIRBuilder__new_extractvalue(
    b: *mut CMuIRBuilder,
    id: CMuID,
    result_id: CMuID,
    strty: CMuTypeNode,
    index: c_int,
    opnd: CMuVarNode,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_strty = from_MuID(strty);
    let mut _arg_index = index;
    let mut _arg_opnd = from_MuID(opnd);
    unsafe {
        (*_arg_b).new_extractvalue(_arg_id, _arg_result_id, _arg_strty, _arg_index, _arg_opnd)
    };
}

extern "C" fn _forwarder__MuIRBuilder__new_insertvalue(
    b: *mut CMuIRBuilder,
    id: CMuID,
    result_id: CMuID,
    strty: CMuTypeNode,
    index: c_int,
    opnd: CMuVarNode,
    newval: CMuVarNode,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_strty = from_MuID(strty);
    let mut _arg_index = index;
    let mut _arg_opnd = from_MuID(opnd);
    let mut _arg_newval = from_MuID(newval);
    unsafe {
        (*_arg_b).new_insertvalue(
            _arg_id,
            _arg_result_id,
            _arg_strty,
            _arg_index,
            _arg_opnd,
            _arg_newval,
        )
    };
}

extern "C" fn _forwarder__MuIRBuilder__new_extractelement(
    b: *mut CMuIRBuilder,
    id: CMuID,
    result_id: CMuID,
    seqty: CMuTypeNode,
    indty: CMuTypeNode,
    opnd: CMuVarNode,
    index: CMuVarNode,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_seqty = from_MuID(seqty);
    let mut _arg_indty = from_MuID(indty);
    let mut _arg_opnd = from_MuID(opnd);
    let mut _arg_index = from_MuID(index);
    unsafe {
        (*_arg_b).new_extractelement(
            _arg_id,
            _arg_result_id,
            _arg_seqty,
            _arg_indty,
            _arg_opnd,
            _arg_index,
        )
    };
}

extern "C" fn _forwarder__MuIRBuilder__new_insertelement(
    b: *mut CMuIRBuilder,
    id: CMuID,
    result_id: CMuID,
    seqty: CMuTypeNode,
    indty: CMuTypeNode,
    opnd: CMuVarNode,
    index: CMuVarNode,
    newval: CMuVarNode,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_seqty = from_MuID(seqty);
    let mut _arg_indty = from_MuID(indty);
    let mut _arg_opnd = from_MuID(opnd);
    let mut _arg_index = from_MuID(index);
    let mut _arg_newval = from_MuID(newval);
    unsafe {
        (*_arg_b).new_insertelement(
            _arg_id,
            _arg_result_id,
            _arg_seqty,
            _arg_indty,
            _arg_opnd,
            _arg_index,
            _arg_newval,
        )
    };
}

extern "C" fn _forwarder__MuIRBuilder__new_shufflevector(
    b: *mut CMuIRBuilder,
    id: CMuID,
    result_id: CMuID,
    vecty: CMuTypeNode,
    maskty: CMuTypeNode,
    vec1: CMuVarNode,
    vec2: CMuVarNode,
    mask: CMuVarNode,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_vecty = from_MuID(vecty);
    let mut _arg_maskty = from_MuID(maskty);
    let mut _arg_vec1 = from_MuID(vec1);
    let mut _arg_vec2 = from_MuID(vec2);
    let mut _arg_mask = from_MuID(mask);
    unsafe {
        (*_arg_b).new_shufflevector(
            _arg_id,
            _arg_result_id,
            _arg_vecty,
            _arg_maskty,
            _arg_vec1,
            _arg_vec2,
            _arg_mask,
        )
    };
}

extern "C" fn _forwarder__MuIRBuilder__new_new(
    b: *mut CMuIRBuilder,
    id: CMuID,
    result_id: CMuID,
    allocty: CMuTypeNode,
    exc_clause: CMuExcClause,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_allocty = from_MuID(allocty);
    let mut _arg_exc_clause = from_MuID_optional(exc_clause);
    unsafe { (*_arg_b).new_new(_arg_id, _arg_result_id, _arg_allocty, _arg_exc_clause) };
}

extern "C" fn _forwarder__MuIRBuilder__new_newhybrid(
    b: *mut CMuIRBuilder,
    id: CMuID,
    result_id: CMuID,
    allocty: CMuTypeNode,
    lenty: CMuTypeNode,
    length: CMuVarNode,
    exc_clause: CMuExcClause,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_allocty = from_MuID(allocty);
    let mut _arg_lenty = from_MuID(lenty);
    let mut _arg_length = from_MuID(length);
    let mut _arg_exc_clause = from_MuID_optional(exc_clause);
    unsafe {
        (*_arg_b).new_newhybrid(
            _arg_id,
            _arg_result_id,
            _arg_allocty,
            _arg_lenty,
            _arg_length,
            _arg_exc_clause,
        )
    };
}

extern "C" fn _forwarder__MuIRBuilder__new_alloca(
    b: *mut CMuIRBuilder,
    id: CMuID,
    result_id: CMuID,
    allocty: CMuTypeNode,
    exc_clause: CMuExcClause,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_allocty = from_MuID(allocty);
    let mut _arg_exc_clause = from_MuID_optional(exc_clause);
    unsafe { (*_arg_b).new_alloca(_arg_id, _arg_result_id, _arg_allocty, _arg_exc_clause) };
}

extern "C" fn _forwarder__MuIRBuilder__new_allocahybrid(
    b: *mut CMuIRBuilder,
    id: CMuID,
    result_id: CMuID,
    allocty: CMuTypeNode,
    lenty: CMuTypeNode,
    length: CMuVarNode,
    exc_clause: CMuExcClause,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_allocty = from_MuID(allocty);
    let mut _arg_lenty = from_MuID(lenty);
    let mut _arg_length = from_MuID(length);
    let mut _arg_exc_clause = from_MuID_optional(exc_clause);
    unsafe {
        (*_arg_b).new_allocahybrid(
            _arg_id,
            _arg_result_id,
            _arg_allocty,
            _arg_lenty,
            _arg_length,
            _arg_exc_clause,
        )
    };
}

extern "C" fn _forwarder__MuIRBuilder__new_getiref(
    b: *mut CMuIRBuilder,
    id: CMuID,
    result_id: CMuID,
    refty: CMuTypeNode,
    opnd: CMuVarNode,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_refty = from_MuID(refty);
    let mut _arg_opnd = from_MuID(opnd);
    unsafe { (*_arg_b).new_getiref(_arg_id, _arg_result_id, _arg_refty, _arg_opnd) };
}

extern "C" fn _forwarder__MuIRBuilder__new_getfieldiref(
    b: *mut CMuIRBuilder,
    id: CMuID,
    result_id: CMuID,
    is_ptr: CMuBool,
    refty: CMuTypeNode,
    index: c_int,
    opnd: CMuVarNode,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_is_ptr = from_MuBool(is_ptr);
    let mut _arg_refty = from_MuID(refty);
    let mut _arg_index = index;
    let mut _arg_opnd = from_MuID(opnd);
    unsafe {
        (*_arg_b).new_getfieldiref(
            _arg_id,
            _arg_result_id,
            _arg_is_ptr,
            _arg_refty,
            _arg_index,
            _arg_opnd,
        )
    };
}

extern "C" fn _forwarder__MuIRBuilder__new_getelemiref(
    b: *mut CMuIRBuilder,
    id: CMuID,
    result_id: CMuID,
    is_ptr: CMuBool,
    refty: CMuTypeNode,
    indty: CMuTypeNode,
    opnd: CMuVarNode,
    index: CMuVarNode,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_is_ptr = from_MuBool(is_ptr);
    let mut _arg_refty = from_MuID(refty);
    let mut _arg_indty = from_MuID(indty);
    let mut _arg_opnd = from_MuID(opnd);
    let mut _arg_index = from_MuID(index);
    unsafe {
        (*_arg_b).new_getelemiref(
            _arg_id,
            _arg_result_id,
            _arg_is_ptr,
            _arg_refty,
            _arg_indty,
            _arg_opnd,
            _arg_index,
        )
    };
}

extern "C" fn _forwarder__MuIRBuilder__new_shiftiref(
    b: *mut CMuIRBuilder,
    id: CMuID,
    result_id: CMuID,
    is_ptr: CMuBool,
    refty: CMuTypeNode,
    offty: CMuTypeNode,
    opnd: CMuVarNode,
    offset: CMuVarNode,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_is_ptr = from_MuBool(is_ptr);
    let mut _arg_refty = from_MuID(refty);
    let mut _arg_offty = from_MuID(offty);
    let mut _arg_opnd = from_MuID(opnd);
    let mut _arg_offset = from_MuID(offset);
    unsafe {
        (*_arg_b).new_shiftiref(
            _arg_id,
            _arg_result_id,
            _arg_is_ptr,
            _arg_refty,
            _arg_offty,
            _arg_opnd,
            _arg_offset,
        )
    };
}

extern "C" fn _forwarder__MuIRBuilder__new_getvarpartiref(
    b: *mut CMuIRBuilder,
    id: CMuID,
    result_id: CMuID,
    is_ptr: CMuBool,
    refty: CMuTypeNode,
    opnd: CMuVarNode,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_is_ptr = from_MuBool(is_ptr);
    let mut _arg_refty = from_MuID(refty);
    let mut _arg_opnd = from_MuID(opnd);
    unsafe {
        (*_arg_b).new_getvarpartiref(_arg_id, _arg_result_id, _arg_is_ptr, _arg_refty, _arg_opnd)
    };
}

extern "C" fn _forwarder__MuIRBuilder__new_load(
    b: *mut CMuIRBuilder,
    id: CMuID,
    result_id: CMuID,
    is_ptr: CMuBool,
    ord: CMuMemOrd,
    refty: CMuTypeNode,
    loc: CMuVarNode,
    exc_clause: CMuExcClause,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_is_ptr = from_MuBool(is_ptr);
    let mut _arg_ord = ord;
    let mut _arg_refty = from_MuID(refty);
    let mut _arg_loc = from_MuID(loc);
    let mut _arg_exc_clause = from_MuID_optional(exc_clause);
    unsafe {
        (*_arg_b).new_load(
            _arg_id,
            _arg_result_id,
            _arg_is_ptr,
            _arg_ord,
            _arg_refty,
            _arg_loc,
            _arg_exc_clause,
        )
    };
}

extern "C" fn _forwarder__MuIRBuilder__new_store(
    b: *mut CMuIRBuilder,
    id: CMuID,
    is_ptr: CMuBool,
    ord: CMuMemOrd,
    refty: CMuTypeNode,
    loc: CMuVarNode,
    newval: CMuVarNode,
    exc_clause: CMuExcClause,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_is_ptr = from_MuBool(is_ptr);
    let mut _arg_ord = ord;
    let mut _arg_refty = from_MuID(refty);
    let mut _arg_loc = from_MuID(loc);
    let mut _arg_newval = from_MuID(newval);
    let mut _arg_exc_clause = from_MuID_optional(exc_clause);
    unsafe {
        (*_arg_b).new_store(
            _arg_id,
            _arg_is_ptr,
            _arg_ord,
            _arg_refty,
            _arg_loc,
            _arg_newval,
            _arg_exc_clause,
        )
    };
}

extern "C" fn _forwarder__MuIRBuilder__new_cmpxchg(
    b: *mut CMuIRBuilder,
    id: CMuID,
    value_result_id: CMuID,
    succ_result_id: CMuID,
    is_ptr: CMuBool,
    is_weak: CMuBool,
    ord_succ: CMuMemOrd,
    ord_fail: CMuMemOrd,
    refty: CMuTypeNode,
    loc: CMuVarNode,
    expected: CMuVarNode,
    desired: CMuVarNode,
    exc_clause: CMuExcClause,
) {
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
    unsafe {
        (*_arg_b).new_cmpxchg(
            _arg_id,
            _arg_value_result_id,
            _arg_succ_result_id,
            _arg_is_ptr,
            _arg_is_weak,
            _arg_ord_succ,
            _arg_ord_fail,
            _arg_refty,
            _arg_loc,
            _arg_expected,
            _arg_desired,
            _arg_exc_clause,
        )
    };
}

extern "C" fn _forwarder__MuIRBuilder__new_atomicrmw(
    b: *mut CMuIRBuilder,
    id: CMuID,
    result_id: CMuID,
    is_ptr: CMuBool,
    ord: CMuMemOrd,
    optr: CMuAtomicRMWOptr,
    ref_ty: CMuTypeNode,
    loc: CMuVarNode,
    opnd: CMuVarNode,
    exc_clause: CMuExcClause,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_is_ptr = from_MuBool(is_ptr);
    let mut _arg_ord = ord;
    let mut _arg_optr = optr;
    let mut _arg_ref_ty = from_MuID(ref_ty);
    let mut _arg_loc = from_MuID(loc);
    let mut _arg_opnd = from_MuID(opnd);
    let mut _arg_exc_clause = from_MuID_optional(exc_clause);
    unsafe {
        (*_arg_b).new_atomicrmw(
            _arg_id,
            _arg_result_id,
            _arg_is_ptr,
            _arg_ord,
            _arg_optr,
            _arg_ref_ty,
            _arg_loc,
            _arg_opnd,
            _arg_exc_clause,
        )
    };
}

extern "C" fn _forwarder__MuIRBuilder__new_fence(b: *mut CMuIRBuilder, id: CMuID, ord: CMuMemOrd) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_ord = ord;
    unsafe { (*_arg_b).new_fence(_arg_id, _arg_ord) };
}

extern "C" fn _forwarder__MuIRBuilder__new_trap(
    b: *mut CMuIRBuilder,
    id: CMuID,
    result_ids: *mut CMuID,
    rettys: *mut CMuTypeNode,
    nretvals: CMuArraySize,
    exc_clause: CMuExcClause,
    keepalive_clause: CMuKeepaliveClause,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_ids = from_MuID_array(result_ids, nretvals);
    let mut _arg_rettys = from_MuID_array(rettys, nretvals);
    let mut _arg_exc_clause = from_MuID_optional(exc_clause);
    let mut _arg_keepalive_clause = from_MuID_optional(keepalive_clause);
    unsafe {
        (*_arg_b).new_trap(
            _arg_id,
            _arg_result_ids,
            _arg_rettys,
            _arg_exc_clause,
            _arg_keepalive_clause,
        )
    };
}

extern "C" fn _forwarder__MuIRBuilder__new_watchpoint(
    b: *mut CMuIRBuilder,
    id: CMuID,
    wpid: CMuWPID,
    result_ids: *mut CMuID,
    rettys: *mut CMuTypeNode,
    nretvals: CMuArraySize,
    dis: CMuDestClause,
    ena: CMuDestClause,
    exc: CMuDestClause,
    keepalive_clause: CMuKeepaliveClause,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_wpid = wpid;
    let mut _arg_result_ids = from_MuID_array(result_ids, nretvals);
    let mut _arg_rettys = from_MuID_array(rettys, nretvals);
    let mut _arg_dis = from_MuID(dis);
    let mut _arg_ena = from_MuID(ena);
    let mut _arg_exc = from_MuID_optional(exc);
    let mut _arg_keepalive_clause = from_MuID_optional(keepalive_clause);
    unsafe {
        (*_arg_b).new_watchpoint(
            _arg_id,
            _arg_wpid,
            _arg_result_ids,
            _arg_rettys,
            _arg_dis,
            _arg_ena,
            _arg_exc,
            _arg_keepalive_clause,
        )
    };
}

extern "C" fn _forwarder__MuIRBuilder__new_wpbranch(
    b: *mut CMuIRBuilder,
    id: CMuID,
    wpid: CMuWPID,
    dis: CMuDestClause,
    ena: CMuDestClause,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_wpid = wpid;
    let mut _arg_dis = from_MuID(dis);
    let mut _arg_ena = from_MuID(ena);
    unsafe { (*_arg_b).new_wpbranch(_arg_id, _arg_wpid, _arg_dis, _arg_ena) };
}

extern "C" fn _forwarder__MuIRBuilder__new_ccall(
    b: *mut CMuIRBuilder,
    id: CMuID,
    result_ids: *mut CMuID,
    n_result_ids: CMuArraySize,
    callconv: CMuCallConv,
    callee_ty: CMuTypeNode,
    sig: CMuFuncSigNode,
    callee: CMuVarNode,
    args: *mut CMuVarNode,
    nargs: CMuArraySize,
    exc_clause: CMuExcClause,
    keepalive_clause: CMuKeepaliveClause,
) {
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
    unsafe {
        (*_arg_b).new_ccall(
            _arg_id,
            _arg_result_ids,
            _arg_callconv,
            _arg_callee_ty,
            _arg_sig,
            _arg_callee,
            _arg_args,
            _arg_exc_clause,
            _arg_keepalive_clause,
        )
    };
}

extern "C" fn _forwarder__MuIRBuilder__new_newthread(
    b: *mut CMuIRBuilder,
    id: CMuID,
    result_id: CMuID,
    stack: CMuVarNode,
    threadlocal: CMuVarNode,
    new_stack_clause: CMuNewStackClause,
    exc_clause: CMuExcClause,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_stack = from_MuID(stack);
    let mut _arg_threadlocal = from_MuID_optional(threadlocal);
    let mut _arg_new_stack_clause = from_MuID(new_stack_clause);
    let mut _arg_exc_clause = from_MuID_optional(exc_clause);
    unsafe {
        (*_arg_b).new_newthread(
            _arg_id,
            _arg_result_id,
            _arg_stack,
            _arg_threadlocal,
            _arg_new_stack_clause,
            _arg_exc_clause,
        )
    };
}

extern "C" fn _forwarder__MuIRBuilder__new_swapstack(
    b: *mut CMuIRBuilder,
    id: CMuID,
    result_ids: *mut CMuID,
    n_result_ids: CMuArraySize,
    swappee: CMuVarNode,
    cur_stack_clause: CMuCurStackClause,
    new_stack_clause: CMuNewStackClause,
    exc_clause: CMuExcClause,
    keepalive_clause: CMuKeepaliveClause,
) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_ids = from_MuID_array(result_ids, n_result_ids);
    let mut _arg_swappee = from_MuID(swappee);
    let mut _arg_cur_stack_clause = from_MuID(cur_stack_clause);
    let mut _arg_new_stack_clause = from_MuID(new_stack_clause);
    let mut _arg_exc_clause = from_MuID_optional(exc_clause);
    let mut _arg_keepalive_clause = from_MuID_optional(keepalive_clause);
    unsafe {
        (*_arg_b).new_swapstack(
            _arg_id,
            _arg_result_ids,
            _arg_swappee,
            _arg_cur_stack_clause,
            _arg_new_stack_clause,
            _arg_exc_clause,
            _arg_keepalive_clause,
        )
    };
}

extern "C" fn _forwarder__MuIRBuilder__new_comminst(
    b: *mut CMuIRBuilder,
    id: CMuID,
    result_ids: *mut CMuID,
    n_result_ids: CMuArraySize,
    opcode: CMuCommInst,
    flags: *mut CMuFlag,
    nflags: CMuArraySize,
    tys: *mut CMuTypeNode,
    ntys: CMuArraySize,
    sigs: *mut CMuFuncSigNode,
    nsigs: CMuArraySize,
    args: *mut CMuVarNode,
    nargs: CMuArraySize,
    exc_clause: CMuExcClause,
    keepalive_clause: CMuKeepaliveClause,
) {
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
    unsafe {
        (*_arg_b).new_comminst(
            _arg_id,
            _arg_result_ids,
            _arg_opcode,
            _arg_flags,
            _arg_tys,
            _arg_sigs,
            _arg_args,
            _arg_exc_clause,
            _arg_keepalive_clause,
        )
    };
}
// GEN:END:Forwarders

// GEN:BEGIN:Fillers
pub fn make_new_MuVM(header: *mut c_void) -> *mut CMuVM {
    let bx = Box::new(CMuVM {
        header: header,
        new_context: _forwarder__MuVM__new_context,
        id_of: _forwarder__MuVM__id_of,
        name_of: _forwarder__MuVM__name_of,
        set_trap_handler: _forwarder__MuVM__set_trap_handler,
        compile_to_sharedlib: _forwarder__MuVM__compile_to_sharedlib,
        current_thread_as_mu_thread: _forwarder__MuVM__current_thread_as_mu_thread,
    });

    Box::into_raw(bx)
}

pub fn make_new_MuCtx(header: *mut c_void) -> *mut CMuCtx {
    let bx = Box::new(CMuCtx {
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

    Box::into_raw(bx)
}

pub fn make_new_MuIRBuilder(header: *mut c_void) -> *mut CMuIRBuilder {
    let bx = Box::new(CMuIRBuilder {
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

    Box::into_raw(bx)
}
// GEN:END:Fillers
