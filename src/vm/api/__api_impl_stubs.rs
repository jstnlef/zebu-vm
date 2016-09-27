/*!
 * Stubs for api_impl.rs
 *
 * The muapi2rustapi.py script will always generate these stubs. Copy them into api_impl.rs as the
 * template to the implementation.
 */

#![allow(non_camel_case_types)]
#![allow(dead_code)]

use std::os::raw::*;

use api_c::*;
use api_bridge::*;
use deps::*;

pub struct MuVM {
    // Stub
}

pub struct MuCtx {
    // Stub
}

pub struct MuIRBuilder {
    // Stub
}

// GEN:BEGIN:StubImpls
impl MuVM {
    pub fn new_context(&mut self) -> *mut CMuCtx {
        panic!("Not implemented")
    }

    pub fn id_of(&mut self, name: MuName) -> MuID {
        panic!("Not implemented")
    }

    pub fn name_of(&mut self, id: MuID) -> CMuCString {
        panic!("Not implemented")
    }

    pub fn set_trap_handler(&mut self, trap_handler: CMuTrapHandler, userdata: CMuCPtr) {
        panic!("Not implemented")
    }

}

impl MuCtx {
    pub fn id_of(&mut self, name: MuName) -> MuID {
        panic!("Not implemented")
    }

    pub fn name_of(&mut self, id: MuID) -> CMuCString {
        panic!("Not implemented")
    }

    pub fn close_context(&mut self) {
        panic!("Not implemented")
    }

    pub fn load_bundle(&mut self, buf: &[c_char]) {
        panic!("Not implemented")
    }

    pub fn load_hail(&mut self, buf: &[c_char]) {
        panic!("Not implemented")
    }

    pub fn handle_from_sint8(&mut self, num: i8, len: c_int) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn handle_from_uint8(&mut self, num: u8, len: c_int) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn handle_from_sint16(&mut self, num: i16, len: c_int) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn handle_from_uint16(&mut self, num: u16, len: c_int) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn handle_from_sint32(&mut self, num: i32, len: c_int) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn handle_from_uint32(&mut self, num: u32, len: c_int) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn handle_from_sint64(&mut self, num: i64, len: c_int) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn handle_from_uint64(&mut self, num: u64, len: c_int) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn handle_from_uint64s(&mut self, nums: &[u64], len: c_int) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn handle_from_float(&mut self, num: f32) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn handle_from_double(&mut self, num: f64) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn handle_from_ptr(&mut self, mu_type: MuID, ptr: CMuCPtr) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn handle_from_fp(&mut self, mu_type: MuID, fp: CMuCFP) -> *mut APIMuValue {
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

    pub fn handle_from_const(&mut self, id: MuID) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn handle_from_global(&mut self, id: MuID) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn handle_from_func(&mut self, id: MuID) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn handle_from_expose(&mut self, id: MuID) -> *mut APIMuValue {
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

    pub fn extract_value(&mut self, str: &APIMuValue, index: c_int) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn insert_value(&mut self, str: &APIMuValue, index: c_int, newval: &APIMuValue) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn extract_element(&mut self, str: &APIMuValue, index: &APIMuValue) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn insert_element(&mut self, str: &APIMuValue, index: &APIMuValue, newval: &APIMuValue) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn new_fixed(&mut self, mu_type: MuID) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn new_hybrid(&mut self, mu_type: MuID, length: &APIMuValue) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn refcast(&mut self, opnd: &APIMuValue, new_type: MuID) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn get_iref(&mut self, opnd: &APIMuValue) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn get_field_iref(&mut self, opnd: &APIMuValue, field: c_int) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn get_elem_iref(&mut self, opnd: &APIMuValue, index: &APIMuValue) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn shift_iref(&mut self, opnd: &APIMuValue, offset: &APIMuValue) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn get_var_part_iref(&mut self, opnd: &APIMuValue) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn load(&mut self, ord: CMuMemOrd, loc: &APIMuValue) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn store(&mut self, ord: CMuMemOrd, loc: &APIMuValue, newval: &APIMuValue) {
        panic!("Not implemented")
    }

    pub fn cmpxchg(&mut self, ord_succ: CMuMemOrd, ord_fail: CMuMemOrd, weak: bool, loc: &APIMuValue, expected: &APIMuValue, desired: &APIMuValue, is_succ: *mut CMuBool) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn atomicrmw(&mut self, ord: CMuMemOrd, op: CMuAtomicRMWOptr, loc: &APIMuValue, opnd: &APIMuValue) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn fence(&mut self, ord: CMuMemOrd) {
        panic!("Not implemented")
    }

    pub fn new_stack(&mut self, func: &APIMuValue) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn new_thread_nor(&mut self, stack: &APIMuValue, threadlocal: Option<&APIMuValue>, vals: Vec<&APIMuValue>) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn new_thread_exc(&mut self, stack: &APIMuValue, threadlocal: Option<&APIMuValue>, exc: &APIMuValue) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn kill_stack(&mut self, stack: &APIMuValue) {
        panic!("Not implemented")
    }

    pub fn set_threadlocal(&mut self, thread: &APIMuValue, threadlocal: &APIMuValue) {
        panic!("Not implemented")
    }

    pub fn get_threadlocal(&mut self, thread: &APIMuValue) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn new_cursor(&mut self, stack: &APIMuValue) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn next_frame(&mut self, cursor: &APIMuValue) {
        panic!("Not implemented")
    }

    pub fn copy_cursor(&mut self, cursor: &APIMuValue) -> *mut APIMuValue {
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

    pub fn tr64_to_fp(&mut self, value: &APIMuValue) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn tr64_to_int(&mut self, value: &APIMuValue) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn tr64_to_ref(&mut self, value: &APIMuValue) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn tr64_to_tag(&mut self, value: &APIMuValue) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn tr64_from_fp(&mut self, value: &APIMuValue) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn tr64_from_int(&mut self, value: &APIMuValue) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn tr64_from_ref(&mut self, reff: &APIMuValue, tag: &APIMuValue) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn enable_watchpoint(&mut self, wpid: CMuWPID) {
        panic!("Not implemented")
    }

    pub fn disable_watchpoint(&mut self, wpid: CMuWPID) {
        panic!("Not implemented")
    }

    pub fn pin(&mut self, loc: &APIMuValue) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn unpin(&mut self, loc: &APIMuValue) {
        panic!("Not implemented")
    }

    pub fn get_addr(&mut self, loc: &APIMuValue) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn expose(&mut self, func: &APIMuValue, call_conv: CMuCallConv, cookie: &APIMuValue) -> *mut APIMuValue {
        panic!("Not implemented")
    }

    pub fn unexpose(&mut self, call_conv: CMuCallConv, value: &APIMuValue) {
        panic!("Not implemented")
    }

    pub fn new_ir_builder(&mut self) -> *mut CMuIRBuilder {
        panic!("Not implemented")
    }

    pub fn make_boot_image(&mut self, whitelist: Vec<MuID>, primordial_func: Option<&APIMuValue>, primordial_stack: Option<&APIMuValue>, primordial_threadlocal: Option<&APIMuValue>, sym_fields: Vec<&APIMuValue>, sym_strings: Vec<String>, reloc_fields: Vec<&APIMuValue>, reloc_strings: Vec<String>, output_file: String) {
        panic!("Not implemented")
    }

}

impl MuIRBuilder {
    pub fn load(&mut self) {
        panic!("Not implemented")
    }

    pub fn abort(&mut self) {
        panic!("Not implemented")
    }

    pub fn gen_sym(&mut self, name: Option<String>) -> MuID {
        panic!("Not implemented")
    }

    pub fn new_type_int(&mut self, id: MuID, len: c_int) {
        panic!("Not implemented")
    }

    pub fn new_type_float(&mut self, id: MuID) {
        panic!("Not implemented")
    }

    pub fn new_type_double(&mut self, id: MuID) {
        panic!("Not implemented")
    }

    pub fn new_type_uptr(&mut self, id: MuID, ty: MuID) {
        panic!("Not implemented")
    }

    pub fn new_type_ufuncptr(&mut self, id: MuID, sig: MuID) {
        panic!("Not implemented")
    }

    pub fn new_type_struct(&mut self, id: MuID, fieldtys: Vec<MuID>) {
        panic!("Not implemented")
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
        panic!("Not implemented")
    }

    pub fn new_const_int(&mut self, id: MuID, ty: MuID, value: u64) {
        panic!("Not implemented")
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
        panic!("Not implemented")
    }

    pub fn new_func(&mut self, id: MuID, sig: MuID) {
        panic!("Not implemented")
    }

    pub fn new_exp_func(&mut self, id: MuID, func: MuID, callconv: CMuCallConv, cookie: MuID) {
        panic!("Not implemented")
    }

    pub fn new_func_ver(&mut self, id: MuID, func: MuID, bbs: Vec<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_bb(&mut self, id: MuID, nor_param_ids: Vec<MuID>, nor_param_types: Vec<MuID>, exc_param_id: Option<MuID>, insts: Vec<MuID>) {
        panic!("Not implemented")
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
        panic!("Not implemented")
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

    pub fn new_atomicrmw(&mut self, id: MuID, result_id: MuID, is_ptr: bool, ord: CMuMemOrd, optr: CMuAtomicRMWOptr, refTy: MuID, loc: MuID, opnd: MuID, exc_clause: Option<MuID>) {
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
// GEN:END:StubImpls
