use std::os::raw::*;
use super::api_c::*;

// GEN:BEGIN:Fillers
// GEN:END:Fillers

// GEN:BEGIN:Forwarders
fn _forwarder__MuVM__new_context(mvm: *mut CMuVM)-> *mut CMuCtx {
    let mut _arg_mvm = from_MuVM_ptr(mvm);
    panic!("not implemented")
}

fn _forwarder__MuVM__id_of(mvm: *mut CMuVM, name: CMuName)-> CMuID {
    let mut _arg_mvm = from_MuVM_ptr(mvm);
    let mut _arg_name = from_MuName(name);
    panic!("not implemented")
}

fn _forwarder__MuVM__name_of(mvm: *mut CMuVM, id: CMuID)-> CMuName {
    let mut _arg_mvm = from_MuVM_ptr(mvm);
    let mut _arg_id = from_MuID(id);
    panic!("not implemented")
}

fn _forwarder__MuVM__set_trap_handler(mvm: *mut CMuVM, trap_handler: CMuTrapHandler, userdata: CMuCPtr) {
    let mut _arg_mvm = from_MuVM_ptr(mvm);
    let mut _arg_trap_handler = from_MuTrapHandler(trap_handler);
    let mut _arg_userdata = from_MuCPtr(userdata);
    panic!("not implemented")
}

fn _forwarder__MuCtx__id_of(ctx: *mut CMuCtx, name: CMuName)-> CMuID {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_name = from_MuName(name);
    panic!("not implemented")
}

fn _forwarder__MuCtx__name_of(ctx: *mut CMuCtx, id: CMuID)-> CMuName {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_id = from_MuID(id);
    panic!("not implemented")
}

fn _forwarder__MuCtx__close_context(ctx: *mut CMuCtx) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    panic!("not implemented")
}

fn _forwarder__MuCtx__load_bundle(ctx: *mut CMuCtx, buf: *mut c_char, sz: CMuArraySize) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_buf = from_char_array(buf, sz);
    panic!("not implemented")
}

fn _forwarder__MuCtx__load_hail(ctx: *mut CMuCtx, buf: *mut c_char, sz: CMuArraySize) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_buf = from_char_array(buf, sz);
    panic!("not implemented")
}

fn _forwarder__MuCtx__handle_from_sint8(ctx: *mut CMuCtx, num: i8, len: c_int)-> CMuIntValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_num = from_int8_t(num);
    let mut _arg_len = from_int(len);
    panic!("not implemented")
}

fn _forwarder__MuCtx__handle_from_uint8(ctx: *mut CMuCtx, num: u8, len: c_int)-> CMuIntValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_num = from_uint8_t(num);
    let mut _arg_len = from_int(len);
    panic!("not implemented")
}

fn _forwarder__MuCtx__handle_from_sint16(ctx: *mut CMuCtx, num: i16, len: c_int)-> CMuIntValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_num = from_int16_t(num);
    let mut _arg_len = from_int(len);
    panic!("not implemented")
}

fn _forwarder__MuCtx__handle_from_uint16(ctx: *mut CMuCtx, num: u16, len: c_int)-> CMuIntValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_num = from_uint16_t(num);
    let mut _arg_len = from_int(len);
    panic!("not implemented")
}

fn _forwarder__MuCtx__handle_from_sint32(ctx: *mut CMuCtx, num: i32, len: c_int)-> CMuIntValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_num = from_int32_t(num);
    let mut _arg_len = from_int(len);
    panic!("not implemented")
}

fn _forwarder__MuCtx__handle_from_uint32(ctx: *mut CMuCtx, num: u32, len: c_int)-> CMuIntValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_num = from_uint32_t(num);
    let mut _arg_len = from_int(len);
    panic!("not implemented")
}

fn _forwarder__MuCtx__handle_from_sint64(ctx: *mut CMuCtx, num: i64, len: c_int)-> CMuIntValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_num = from_int64_t(num);
    let mut _arg_len = from_int(len);
    panic!("not implemented")
}

fn _forwarder__MuCtx__handle_from_uint64(ctx: *mut CMuCtx, num: u64, len: c_int)-> CMuIntValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_num = from_uint64_t(num);
    let mut _arg_len = from_int(len);
    panic!("not implemented")
}

fn _forwarder__MuCtx__handle_from_uint64s(ctx: *mut CMuCtx, nums: *mut u64, nnums: CMuArraySize, len: c_int)-> CMuIntValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_nums = from_uint64_t_array(nums, nnums);
    let mut _arg_len = from_int(len);
    panic!("not implemented")
}

fn _forwarder__MuCtx__handle_from_float(ctx: *mut CMuCtx, num: f32)-> CMuFloatValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_num = from_float(num);
    panic!("not implemented")
}

fn _forwarder__MuCtx__handle_from_double(ctx: *mut CMuCtx, num: f64)-> CMuDoubleValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_num = from_double(num);
    panic!("not implemented")
}

fn _forwarder__MuCtx__handle_from_ptr(ctx: *mut CMuCtx, mu_type: CMuID, ptr: CMuCPtr)-> CMuUPtrValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_mu_type = from_MuID(mu_type);
    let mut _arg_ptr = from_MuCPtr(ptr);
    panic!("not implemented")
}

fn _forwarder__MuCtx__handle_from_fp(ctx: *mut CMuCtx, mu_type: CMuID, fp: CMuCFP)-> CMuUFPValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_mu_type = from_MuID(mu_type);
    let mut _arg_fp = from_MuCFP(fp);
    panic!("not implemented")
}

fn _forwarder__MuCtx__handle_to_sint8(ctx: *mut CMuCtx, opnd: CMuIntValue)-> i8 {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_MuIntValue(opnd);
    panic!("not implemented")
}

fn _forwarder__MuCtx__handle_to_uint8(ctx: *mut CMuCtx, opnd: CMuIntValue)-> u8 {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_MuIntValue(opnd);
    panic!("not implemented")
}

fn _forwarder__MuCtx__handle_to_sint16(ctx: *mut CMuCtx, opnd: CMuIntValue)-> i16 {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_MuIntValue(opnd);
    panic!("not implemented")
}

fn _forwarder__MuCtx__handle_to_uint16(ctx: *mut CMuCtx, opnd: CMuIntValue)-> u16 {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_MuIntValue(opnd);
    panic!("not implemented")
}

fn _forwarder__MuCtx__handle_to_sint32(ctx: *mut CMuCtx, opnd: CMuIntValue)-> i32 {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_MuIntValue(opnd);
    panic!("not implemented")
}

fn _forwarder__MuCtx__handle_to_uint32(ctx: *mut CMuCtx, opnd: CMuIntValue)-> u32 {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_MuIntValue(opnd);
    panic!("not implemented")
}

fn _forwarder__MuCtx__handle_to_sint64(ctx: *mut CMuCtx, opnd: CMuIntValue)-> i64 {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_MuIntValue(opnd);
    panic!("not implemented")
}

fn _forwarder__MuCtx__handle_to_uint64(ctx: *mut CMuCtx, opnd: CMuIntValue)-> u64 {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_MuIntValue(opnd);
    panic!("not implemented")
}

fn _forwarder__MuCtx__handle_to_float(ctx: *mut CMuCtx, opnd: CMuFloatValue)-> f32 {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_MuFloatValue(opnd);
    panic!("not implemented")
}

fn _forwarder__MuCtx__handle_to_double(ctx: *mut CMuCtx, opnd: CMuDoubleValue)-> f64 {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_MuDoubleValue(opnd);
    panic!("not implemented")
}

fn _forwarder__MuCtx__handle_to_ptr(ctx: *mut CMuCtx, opnd: CMuUPtrValue)-> CMuCPtr {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_MuUPtrValue(opnd);
    panic!("not implemented")
}

fn _forwarder__MuCtx__handle_to_fp(ctx: *mut CMuCtx, opnd: CMuUFPValue)-> CMuCFP {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_MuUFPValue(opnd);
    panic!("not implemented")
}

fn _forwarder__MuCtx__handle_from_const(ctx: *mut CMuCtx, id: CMuID)-> CMuValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_id = from_MuID(id);
    panic!("not implemented")
}

fn _forwarder__MuCtx__handle_from_global(ctx: *mut CMuCtx, id: CMuID)-> CMuIRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_id = from_MuID(id);
    panic!("not implemented")
}

fn _forwarder__MuCtx__handle_from_func(ctx: *mut CMuCtx, id: CMuID)-> CMuFuncRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_id = from_MuID(id);
    panic!("not implemented")
}

fn _forwarder__MuCtx__handle_from_expose(ctx: *mut CMuCtx, id: CMuID)-> CMuValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_id = from_MuID(id);
    panic!("not implemented")
}

fn _forwarder__MuCtx__delete_value(ctx: *mut CMuCtx, opnd: CMuValue) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_MuValue(opnd);
    panic!("not implemented")
}

fn _forwarder__MuCtx__ref_eq(ctx: *mut CMuCtx, lhs: CMuGenRefValue, rhs: CMuGenRefValue)-> CMuBool {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_lhs = from_MuGenRefValue(lhs);
    let mut _arg_rhs = from_MuGenRefValue(rhs);
    panic!("not implemented")
}

fn _forwarder__MuCtx__ref_ult(ctx: *mut CMuCtx, lhs: CMuIRefValue, rhs: CMuIRefValue)-> CMuBool {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_lhs = from_MuIRefValue(lhs);
    let mut _arg_rhs = from_MuIRefValue(rhs);
    panic!("not implemented")
}

fn _forwarder__MuCtx__extract_value(ctx: *mut CMuCtx, str: CMuStructValue, index: c_int)-> CMuValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_str = from_MuStructValue(str);
    let mut _arg_index = from_int(index);
    panic!("not implemented")
}

fn _forwarder__MuCtx__insert_value(ctx: *mut CMuCtx, str: CMuStructValue, index: c_int, newval: CMuValue)-> CMuStructValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_str = from_MuStructValue(str);
    let mut _arg_index = from_int(index);
    let mut _arg_newval = from_MuValue(newval);
    panic!("not implemented")
}

fn _forwarder__MuCtx__extract_element(ctx: *mut CMuCtx, str: CMuSeqValue, index: CMuIntValue)-> CMuValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_str = from_MuSeqValue(str);
    let mut _arg_index = from_MuIntValue(index);
    panic!("not implemented")
}

fn _forwarder__MuCtx__insert_element(ctx: *mut CMuCtx, str: CMuSeqValue, index: CMuIntValue, newval: CMuValue)-> CMuSeqValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_str = from_MuSeqValue(str);
    let mut _arg_index = from_MuIntValue(index);
    let mut _arg_newval = from_MuValue(newval);
    panic!("not implemented")
}

fn _forwarder__MuCtx__new_fixed(ctx: *mut CMuCtx, mu_type: CMuID)-> CMuRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_mu_type = from_MuID(mu_type);
    panic!("not implemented")
}

fn _forwarder__MuCtx__new_hybrid(ctx: *mut CMuCtx, mu_type: CMuID, length: CMuIntValue)-> CMuRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_mu_type = from_MuID(mu_type);
    let mut _arg_length = from_MuIntValue(length);
    panic!("not implemented")
}

fn _forwarder__MuCtx__refcast(ctx: *mut CMuCtx, opnd: CMuGenRefValue, new_type: CMuID)-> CMuGenRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_MuGenRefValue(opnd);
    let mut _arg_new_type = from_MuID(new_type);
    panic!("not implemented")
}

fn _forwarder__MuCtx__get_iref(ctx: *mut CMuCtx, opnd: CMuRefValue)-> CMuIRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_MuRefValue(opnd);
    panic!("not implemented")
}

fn _forwarder__MuCtx__get_field_iref(ctx: *mut CMuCtx, opnd: CMuIRefValue, field: c_int)-> CMuIRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_MuIRefValue(opnd);
    let mut _arg_field = from_int(field);
    panic!("not implemented")
}

fn _forwarder__MuCtx__get_elem_iref(ctx: *mut CMuCtx, opnd: CMuIRefValue, index: CMuIntValue)-> CMuIRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_MuIRefValue(opnd);
    let mut _arg_index = from_MuIntValue(index);
    panic!("not implemented")
}

fn _forwarder__MuCtx__shift_iref(ctx: *mut CMuCtx, opnd: CMuIRefValue, offset: CMuIntValue)-> CMuIRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_MuIRefValue(opnd);
    let mut _arg_offset = from_MuIntValue(offset);
    panic!("not implemented")
}

fn _forwarder__MuCtx__get_var_part_iref(ctx: *mut CMuCtx, opnd: CMuIRefValue)-> CMuIRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_opnd = from_MuIRefValue(opnd);
    panic!("not implemented")
}

fn _forwarder__MuCtx__load(ctx: *mut CMuCtx, ord: CMuMemOrd, loc: CMuIRefValue)-> CMuValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_ord = from_MuMemOrd(ord);
    let mut _arg_loc = from_MuIRefValue(loc);
    panic!("not implemented")
}

fn _forwarder__MuCtx__store(ctx: *mut CMuCtx, ord: CMuMemOrd, loc: CMuIRefValue, newval: CMuValue) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_ord = from_MuMemOrd(ord);
    let mut _arg_loc = from_MuIRefValue(loc);
    let mut _arg_newval = from_MuValue(newval);
    panic!("not implemented")
}

fn _forwarder__MuCtx__cmpxchg(ctx: *mut CMuCtx, ord_succ: CMuMemOrd, ord_fail: CMuMemOrd, weak: CMuBool, loc: CMuIRefValue, expected: CMuValue, desired: CMuValue, is_succ: *mut CMuBool)-> CMuValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_ord_succ = from_MuMemOrd(ord_succ);
    let mut _arg_ord_fail = from_MuMemOrd(ord_fail);
    let mut _arg_weak = from_MuBool(weak);
    let mut _arg_loc = from_MuIRefValue(loc);
    let mut _arg_expected = from_MuValue(expected);
    let mut _arg_desired = from_MuValue(desired);
    let mut _arg_is_succ = from_MuBool_ptr(is_succ);
    panic!("not implemented")
}

fn _forwarder__MuCtx__atomicrmw(ctx: *mut CMuCtx, ord: CMuMemOrd, op: CMuAtomicRMWOptr, loc: CMuIRefValue, opnd: CMuValue)-> CMuValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_ord = from_MuMemOrd(ord);
    let mut _arg_op = from_MuAtomicRMWOptr(op);
    let mut _arg_loc = from_MuIRefValue(loc);
    let mut _arg_opnd = from_MuValue(opnd);
    panic!("not implemented")
}

fn _forwarder__MuCtx__fence(ctx: *mut CMuCtx, ord: CMuMemOrd) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_ord = from_MuMemOrd(ord);
    panic!("not implemented")
}

fn _forwarder__MuCtx__new_stack(ctx: *mut CMuCtx, func: CMuFuncRefValue)-> CMuStackRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_func = from_MuFuncRefValue(func);
    panic!("not implemented")
}

fn _forwarder__MuCtx__new_thread_nor(ctx: *mut CMuCtx, stack: CMuStackRefValue, threadlocal: CMuRefValue, vals: *mut CMuValue, nvals: CMuArraySize)-> CMuThreadRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_stack = from_MuStackRefValue(stack);
    let mut _arg_threadlocal = from_ptr_optional(threadlocal);
    let mut _arg_vals = from_ptr_array(vals, nvals);
    panic!("not implemented")
}

fn _forwarder__MuCtx__new_thread_exc(ctx: *mut CMuCtx, stack: CMuStackRefValue, threadlocal: CMuRefValue, exc: CMuRefValue)-> CMuThreadRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_stack = from_MuStackRefValue(stack);
    let mut _arg_threadlocal = from_ptr_optional(threadlocal);
    let mut _arg_exc = from_MuRefValue(exc);
    panic!("not implemented")
}

fn _forwarder__MuCtx__kill_stack(ctx: *mut CMuCtx, stack: CMuStackRefValue) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_stack = from_MuStackRefValue(stack);
    panic!("not implemented")
}

fn _forwarder__MuCtx__set_threadlocal(ctx: *mut CMuCtx, thread: CMuThreadRefValue, threadlocal: CMuRefValue) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_thread = from_MuThreadRefValue(thread);
    let mut _arg_threadlocal = from_MuRefValue(threadlocal);
    panic!("not implemented")
}

fn _forwarder__MuCtx__get_threadlocal(ctx: *mut CMuCtx, thread: CMuThreadRefValue)-> CMuRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_thread = from_MuThreadRefValue(thread);
    panic!("not implemented")
}

fn _forwarder__MuCtx__new_cursor(ctx: *mut CMuCtx, stack: CMuStackRefValue)-> CMuFCRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_stack = from_MuStackRefValue(stack);
    panic!("not implemented")
}

fn _forwarder__MuCtx__next_frame(ctx: *mut CMuCtx, cursor: CMuFCRefValue) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_cursor = from_MuFCRefValue(cursor);
    panic!("not implemented")
}

fn _forwarder__MuCtx__copy_cursor(ctx: *mut CMuCtx, cursor: CMuFCRefValue)-> CMuFCRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_cursor = from_MuFCRefValue(cursor);
    panic!("not implemented")
}

fn _forwarder__MuCtx__close_cursor(ctx: *mut CMuCtx, cursor: CMuFCRefValue) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_cursor = from_MuFCRefValue(cursor);
    panic!("not implemented")
}

fn _forwarder__MuCtx__cur_func(ctx: *mut CMuCtx, cursor: CMuFCRefValue)-> CMuID {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_cursor = from_MuFCRefValue(cursor);
    panic!("not implemented")
}

fn _forwarder__MuCtx__cur_func_ver(ctx: *mut CMuCtx, cursor: CMuFCRefValue)-> CMuID {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_cursor = from_MuFCRefValue(cursor);
    panic!("not implemented")
}

fn _forwarder__MuCtx__cur_inst(ctx: *mut CMuCtx, cursor: CMuFCRefValue)-> CMuID {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_cursor = from_MuFCRefValue(cursor);
    panic!("not implemented")
}

fn _forwarder__MuCtx__dump_keepalives(ctx: *mut CMuCtx, cursor: CMuFCRefValue, results: *mut CMuValue) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_cursor = from_MuFCRefValue(cursor);
    let mut _arg_results = from_MuValue_ptr(results);
    panic!("not implemented")
}

fn _forwarder__MuCtx__pop_frames_to(ctx: *mut CMuCtx, cursor: CMuFCRefValue) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_cursor = from_MuFCRefValue(cursor);
    panic!("not implemented")
}

fn _forwarder__MuCtx__push_frame(ctx: *mut CMuCtx, stack: CMuStackRefValue, func: CMuFuncRefValue) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_stack = from_MuStackRefValue(stack);
    let mut _arg_func = from_MuFuncRefValue(func);
    panic!("not implemented")
}

fn _forwarder__MuCtx__tr64_is_fp(ctx: *mut CMuCtx, value: CMuTagRef64Value)-> CMuBool {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_value = from_MuTagRef64Value(value);
    panic!("not implemented")
}

fn _forwarder__MuCtx__tr64_is_int(ctx: *mut CMuCtx, value: CMuTagRef64Value)-> CMuBool {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_value = from_MuTagRef64Value(value);
    panic!("not implemented")
}

fn _forwarder__MuCtx__tr64_is_ref(ctx: *mut CMuCtx, value: CMuTagRef64Value)-> CMuBool {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_value = from_MuTagRef64Value(value);
    panic!("not implemented")
}

fn _forwarder__MuCtx__tr64_to_fp(ctx: *mut CMuCtx, value: CMuTagRef64Value)-> CMuDoubleValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_value = from_MuTagRef64Value(value);
    panic!("not implemented")
}

fn _forwarder__MuCtx__tr64_to_int(ctx: *mut CMuCtx, value: CMuTagRef64Value)-> CMuIntValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_value = from_MuTagRef64Value(value);
    panic!("not implemented")
}

fn _forwarder__MuCtx__tr64_to_ref(ctx: *mut CMuCtx, value: CMuTagRef64Value)-> CMuRefValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_value = from_MuTagRef64Value(value);
    panic!("not implemented")
}

fn _forwarder__MuCtx__tr64_to_tag(ctx: *mut CMuCtx, value: CMuTagRef64Value)-> CMuIntValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_value = from_MuTagRef64Value(value);
    panic!("not implemented")
}

fn _forwarder__MuCtx__tr64_from_fp(ctx: *mut CMuCtx, value: CMuDoubleValue)-> CMuTagRef64Value {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_value = from_MuDoubleValue(value);
    panic!("not implemented")
}

fn _forwarder__MuCtx__tr64_from_int(ctx: *mut CMuCtx, value: CMuIntValue)-> CMuTagRef64Value {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_value = from_MuIntValue(value);
    panic!("not implemented")
}

fn _forwarder__MuCtx__tr64_from_ref(ctx: *mut CMuCtx, reff: CMuRefValue, tag: CMuIntValue)-> CMuTagRef64Value {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_reff = from_MuRefValue(reff);
    let mut _arg_tag = from_MuIntValue(tag);
    panic!("not implemented")
}

fn _forwarder__MuCtx__enable_watchpoint(ctx: *mut CMuCtx, wpid: CMuWPID) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_wpid = from_MuWPID(wpid);
    panic!("not implemented")
}

fn _forwarder__MuCtx__disable_watchpoint(ctx: *mut CMuCtx, wpid: CMuWPID) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_wpid = from_MuWPID(wpid);
    panic!("not implemented")
}

fn _forwarder__MuCtx__pin(ctx: *mut CMuCtx, loc: CMuValue)-> CMuUPtrValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_loc = from_MuValue(loc);
    panic!("not implemented")
}

fn _forwarder__MuCtx__unpin(ctx: *mut CMuCtx, loc: CMuValue) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_loc = from_MuValue(loc);
    panic!("not implemented")
}

fn _forwarder__MuCtx__get_addr(ctx: *mut CMuCtx, loc: CMuValue)-> CMuUPtrValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_loc = from_MuValue(loc);
    panic!("not implemented")
}

fn _forwarder__MuCtx__expose(ctx: *mut CMuCtx, func: CMuFuncRefValue, call_conv: CMuCallConv, cookie: CMuIntValue)-> CMuValue {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_func = from_MuFuncRefValue(func);
    let mut _arg_call_conv = from_MuCallConv(call_conv);
    let mut _arg_cookie = from_MuIntValue(cookie);
    panic!("not implemented")
}

fn _forwarder__MuCtx__unexpose(ctx: *mut CMuCtx, call_conv: CMuCallConv, value: CMuValue) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_call_conv = from_MuCallConv(call_conv);
    let mut _arg_value = from_MuValue(value);
    panic!("not implemented")
}

fn _forwarder__MuCtx__new_ir_builder(ctx: *mut CMuCtx)-> *mut CMuIRBuilder {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    panic!("not implemented")
}

fn _forwarder__MuCtx__make_boot_image(ctx: *mut CMuCtx, whitelist: *mut CMuID, whitelist_sz: CMuArraySize, primordial_func: CMuFuncRefValue, primordial_stack: CMuStackRefValue, primordial_threadlocal: CMuRefValue, sym_fields: *mut CMuIRefValue, sym_strings: *mut CMuCString, nsyms: CMuArraySize, reloc_fields: *mut CMuIRefValue, reloc_strings: *mut CMuCString, nrelocs: CMuArraySize, output_file: CMuCString) {
    let mut _arg_ctx = from_MuCtx_ptr(ctx);
    let mut _arg_whitelist = from_MuID_array(whitelist, whitelist_sz);
    let mut _arg_primordial_func = from_ptr_optional(primordial_func);
    let mut _arg_primordial_stack = from_ptr_optional(primordial_stack);
    let mut _arg_primordial_threadlocal = from_ptr_optional(primordial_threadlocal);
    let mut _arg_sym_fields = from_ptr_array(sym_fields, nsyms);
    let mut _arg_sym_strings = from_MuCString_array(sym_strings, nsyms);
    let mut _arg_reloc_fields = from_ptr_array(reloc_fields, nrelocs);
    let mut _arg_reloc_strings = from_MuCString_array(reloc_strings, nrelocs);
    let mut _arg_output_file = from_MuCString(output_file);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__load(b: *mut CMuIRBuilder) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__abort(b: *mut CMuIRBuilder) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__gen_sym(b: *mut CMuIRBuilder, name: CMuCString)-> CMuID {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_name = from_MuCString_optional(name);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_type_int(b: *mut CMuIRBuilder, id: CMuID, len: c_int) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_len = from_int(len);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_type_float(b: *mut CMuIRBuilder, id: CMuID) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_type_double(b: *mut CMuIRBuilder, id: CMuID) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_type_uptr(b: *mut CMuIRBuilder, id: CMuID, ty: CMuTypeNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_ty = from_MuTypeNode(ty);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_type_ufuncptr(b: *mut CMuIRBuilder, id: CMuID, sig: CMuFuncSigNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_sig = from_MuFuncSigNode(sig);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_type_struct(b: *mut CMuIRBuilder, id: CMuID, fieldtys: *mut CMuTypeNode, nfieldtys: CMuArraySize) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_fieldtys = from_MuID_array(fieldtys, nfieldtys);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_type_hybrid(b: *mut CMuIRBuilder, id: CMuID, fixedtys: *mut CMuTypeNode, nfixedtys: CMuArraySize, varty: CMuTypeNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_fixedtys = from_MuID_array(fixedtys, nfixedtys);
    let mut _arg_varty = from_MuTypeNode(varty);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_type_array(b: *mut CMuIRBuilder, id: CMuID, elemty: CMuTypeNode, len: u64) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_elemty = from_MuTypeNode(elemty);
    let mut _arg_len = from_uint64_t(len);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_type_vector(b: *mut CMuIRBuilder, id: CMuID, elemty: CMuTypeNode, len: u64) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_elemty = from_MuTypeNode(elemty);
    let mut _arg_len = from_uint64_t(len);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_type_void(b: *mut CMuIRBuilder, id: CMuID) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_type_ref(b: *mut CMuIRBuilder, id: CMuID, ty: CMuTypeNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_ty = from_MuTypeNode(ty);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_type_iref(b: *mut CMuIRBuilder, id: CMuID, ty: CMuTypeNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_ty = from_MuTypeNode(ty);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_type_weakref(b: *mut CMuIRBuilder, id: CMuID, ty: CMuTypeNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_ty = from_MuTypeNode(ty);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_type_funcref(b: *mut CMuIRBuilder, id: CMuID, sig: CMuFuncSigNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_sig = from_MuFuncSigNode(sig);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_type_tagref64(b: *mut CMuIRBuilder, id: CMuID) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_type_threadref(b: *mut CMuIRBuilder, id: CMuID) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_type_stackref(b: *mut CMuIRBuilder, id: CMuID) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_type_framecursorref(b: *mut CMuIRBuilder, id: CMuID) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_type_irbuilderref(b: *mut CMuIRBuilder, id: CMuID) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_funcsig(b: *mut CMuIRBuilder, id: CMuID, paramtys: *mut CMuTypeNode, nparamtys: CMuArraySize, rettys: *mut CMuTypeNode, nrettys: CMuArraySize) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_paramtys = from_MuID_array(paramtys, nparamtys);
    let mut _arg_rettys = from_MuID_array(rettys, nrettys);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_const_int(b: *mut CMuIRBuilder, id: CMuID, ty: CMuTypeNode, value: u64) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_ty = from_MuTypeNode(ty);
    let mut _arg_value = from_uint64_t(value);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_const_int_ex(b: *mut CMuIRBuilder, id: CMuID, ty: CMuTypeNode, values: *mut u64, nvalues: CMuArraySize) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_ty = from_MuTypeNode(ty);
    let mut _arg_values = from_uint64_t_array(values, nvalues);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_const_float(b: *mut CMuIRBuilder, id: CMuID, ty: CMuTypeNode, value: f32) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_ty = from_MuTypeNode(ty);
    let mut _arg_value = from_float(value);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_const_double(b: *mut CMuIRBuilder, id: CMuID, ty: CMuTypeNode, value: f64) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_ty = from_MuTypeNode(ty);
    let mut _arg_value = from_double(value);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_const_null(b: *mut CMuIRBuilder, id: CMuID, ty: CMuTypeNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_ty = from_MuTypeNode(ty);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_const_seq(b: *mut CMuIRBuilder, id: CMuID, ty: CMuTypeNode, elems: *mut CMuGlobalVarNode, nelems: CMuArraySize) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_ty = from_MuTypeNode(ty);
    let mut _arg_elems = from_MuID_array(elems, nelems);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_const_extern(b: *mut CMuIRBuilder, id: CMuID, ty: CMuTypeNode, symbol: CMuCString) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_ty = from_MuTypeNode(ty);
    let mut _arg_symbol = from_MuCString(symbol);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_global_cell(b: *mut CMuIRBuilder, id: CMuID, ty: CMuTypeNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_ty = from_MuTypeNode(ty);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_func(b: *mut CMuIRBuilder, id: CMuID, sig: CMuFuncSigNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_sig = from_MuFuncSigNode(sig);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_exp_func(b: *mut CMuIRBuilder, id: CMuID, func: CMuFuncNode, callconv: CMuCallConv, cookie: CMuConstNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_func = from_MuFuncNode(func);
    let mut _arg_callconv = from_MuCallConv(callconv);
    let mut _arg_cookie = from_MuConstNode(cookie);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_func_ver(b: *mut CMuIRBuilder, id: CMuID, func: CMuFuncNode, bbs: *mut CMuBBNode, nbbs: CMuArraySize) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_func = from_MuFuncNode(func);
    let mut _arg_bbs = from_MuID_array(bbs, nbbs);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_bb(b: *mut CMuIRBuilder, id: CMuID, nor_param_ids: *mut CMuID, nor_param_types: *mut CMuTypeNode, n_nor_params: CMuArraySize, exc_param_id: CMuID, insts: *mut CMuInstNode, ninsts: CMuArraySize) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_nor_param_ids = from_MuID_array(nor_param_ids, n_nor_params);
    let mut _arg_nor_param_types = from_MuID_array(nor_param_types, n_nor_params);
    let mut _arg_exc_param_id = from_MuID_optional(exc_param_id);
    let mut _arg_insts = from_MuID_array(insts, ninsts);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_dest_clause(b: *mut CMuIRBuilder, id: CMuID, dest: CMuBBNode, vars: *mut CMuVarNode, nvars: CMuArraySize) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_dest = from_MuBBNode(dest);
    let mut _arg_vars = from_MuID_array(vars, nvars);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_exc_clause(b: *mut CMuIRBuilder, id: CMuID, nor: CMuDestClause, exc: CMuDestClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_nor = from_MuDestClause(nor);
    let mut _arg_exc = from_MuDestClause(exc);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_keepalive_clause(b: *mut CMuIRBuilder, id: CMuID, vars: *mut CMuLocalVarNode, nvars: CMuArraySize) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_vars = from_MuID_array(vars, nvars);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_csc_ret_with(b: *mut CMuIRBuilder, id: CMuID, rettys: *mut CMuTypeNode, nrettys: CMuArraySize) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_rettys = from_MuID_array(rettys, nrettys);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_csc_kill_old(b: *mut CMuIRBuilder, id: CMuID) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_nsc_pass_values(b: *mut CMuIRBuilder, id: CMuID, tys: *mut CMuTypeNode, vars: *mut CMuVarNode, ntysvars: CMuArraySize) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_tys = from_MuID_array(tys, ntysvars);
    let mut _arg_vars = from_MuID_array(vars, ntysvars);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_nsc_throw_exc(b: *mut CMuIRBuilder, id: CMuID, exc: CMuVarNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_exc = from_MuVarNode(exc);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_binop(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, optr: CMuBinOptr, ty: CMuTypeNode, opnd1: CMuVarNode, opnd2: CMuVarNode, exc_clause: CMuExcClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_optr = from_MuBinOptr(optr);
    let mut _arg_ty = from_MuTypeNode(ty);
    let mut _arg_opnd1 = from_MuVarNode(opnd1);
    let mut _arg_opnd2 = from_MuVarNode(opnd2);
    let mut _arg_exc_clause = from_node_optional(exc_clause);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_binop_with_status(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, status_result_ids: *mut CMuID, n_status_result_ids: CMuArraySize, optr: CMuBinOptr, status_flags: CMuBinOpStatus, ty: CMuTypeNode, opnd1: CMuVarNode, opnd2: CMuVarNode, exc_clause: CMuExcClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_status_result_ids = from_MuID_array(status_result_ids, n_status_result_ids);
    let mut _arg_optr = from_MuBinOptr(optr);
    let mut _arg_status_flags = from_MuBinOpStatus(status_flags);
    let mut _arg_ty = from_MuTypeNode(ty);
    let mut _arg_opnd1 = from_MuVarNode(opnd1);
    let mut _arg_opnd2 = from_MuVarNode(opnd2);
    let mut _arg_exc_clause = from_node_optional(exc_clause);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_cmp(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, optr: CMuCmpOptr, ty: CMuTypeNode, opnd1: CMuVarNode, opnd2: CMuVarNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_optr = from_MuCmpOptr(optr);
    let mut _arg_ty = from_MuTypeNode(ty);
    let mut _arg_opnd1 = from_MuVarNode(opnd1);
    let mut _arg_opnd2 = from_MuVarNode(opnd2);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_conv(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, optr: CMuConvOptr, from_ty: CMuTypeNode, to_ty: CMuTypeNode, opnd: CMuVarNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_optr = from_MuConvOptr(optr);
    let mut _arg_from_ty = from_MuTypeNode(from_ty);
    let mut _arg_to_ty = from_MuTypeNode(to_ty);
    let mut _arg_opnd = from_MuVarNode(opnd);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_select(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, cond_ty: CMuTypeNode, opnd_ty: CMuTypeNode, cond: CMuVarNode, if_true: CMuVarNode, if_false: CMuVarNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_cond_ty = from_MuTypeNode(cond_ty);
    let mut _arg_opnd_ty = from_MuTypeNode(opnd_ty);
    let mut _arg_cond = from_MuVarNode(cond);
    let mut _arg_if_true = from_MuVarNode(if_true);
    let mut _arg_if_false = from_MuVarNode(if_false);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_branch(b: *mut CMuIRBuilder, id: CMuID, dest: CMuDestClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_dest = from_MuDestClause(dest);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_branch2(b: *mut CMuIRBuilder, id: CMuID, cond: CMuVarNode, if_true: CMuDestClause, if_false: CMuDestClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_cond = from_MuVarNode(cond);
    let mut _arg_if_true = from_MuDestClause(if_true);
    let mut _arg_if_false = from_MuDestClause(if_false);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_switch(b: *mut CMuIRBuilder, id: CMuID, opnd_ty: CMuTypeNode, opnd: CMuVarNode, default_dest: CMuDestClause, cases: *mut CMuConstNode, dests: *mut CMuDestClause, ncasesdests: CMuArraySize) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_opnd_ty = from_MuTypeNode(opnd_ty);
    let mut _arg_opnd = from_MuVarNode(opnd);
    let mut _arg_default_dest = from_MuDestClause(default_dest);
    let mut _arg_cases = from_MuID_array(cases, ncasesdests);
    let mut _arg_dests = from_MuID_array(dests, ncasesdests);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_call(b: *mut CMuIRBuilder, id: CMuID, result_ids: *mut CMuID, n_result_ids: CMuArraySize, sig: CMuFuncSigNode, callee: CMuVarNode, args: *mut CMuVarNode, nargs: CMuArraySize, exc_clause: CMuExcClause, keepalive_clause: CMuKeepaliveClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_ids = from_MuID_array(result_ids, n_result_ids);
    let mut _arg_sig = from_MuFuncSigNode(sig);
    let mut _arg_callee = from_MuVarNode(callee);
    let mut _arg_args = from_MuID_array(args, nargs);
    let mut _arg_exc_clause = from_node_optional(exc_clause);
    let mut _arg_keepalive_clause = from_node_optional(keepalive_clause);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_tailcall(b: *mut CMuIRBuilder, id: CMuID, sig: CMuFuncSigNode, callee: CMuVarNode, args: *mut CMuVarNode, nargs: CMuArraySize) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_sig = from_MuFuncSigNode(sig);
    let mut _arg_callee = from_MuVarNode(callee);
    let mut _arg_args = from_MuID_array(args, nargs);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_ret(b: *mut CMuIRBuilder, id: CMuID, rvs: *mut CMuVarNode, nrvs: CMuArraySize) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_rvs = from_MuID_array(rvs, nrvs);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_throw(b: *mut CMuIRBuilder, id: CMuID, exc: CMuVarNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_exc = from_MuVarNode(exc);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_extractvalue(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, strty: CMuTypeNode, index: c_int, opnd: CMuVarNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_strty = from_MuTypeNode(strty);
    let mut _arg_index = from_int(index);
    let mut _arg_opnd = from_MuVarNode(opnd);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_insertvalue(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, strty: CMuTypeNode, index: c_int, opnd: CMuVarNode, newval: CMuVarNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_strty = from_MuTypeNode(strty);
    let mut _arg_index = from_int(index);
    let mut _arg_opnd = from_MuVarNode(opnd);
    let mut _arg_newval = from_MuVarNode(newval);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_extractelement(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, seqty: CMuTypeNode, indty: CMuTypeNode, opnd: CMuVarNode, index: CMuVarNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_seqty = from_MuTypeNode(seqty);
    let mut _arg_indty = from_MuTypeNode(indty);
    let mut _arg_opnd = from_MuVarNode(opnd);
    let mut _arg_index = from_MuVarNode(index);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_insertelement(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, seqty: CMuTypeNode, indty: CMuTypeNode, opnd: CMuVarNode, index: CMuVarNode, newval: CMuVarNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_seqty = from_MuTypeNode(seqty);
    let mut _arg_indty = from_MuTypeNode(indty);
    let mut _arg_opnd = from_MuVarNode(opnd);
    let mut _arg_index = from_MuVarNode(index);
    let mut _arg_newval = from_MuVarNode(newval);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_shufflevector(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, vecty: CMuTypeNode, maskty: CMuTypeNode, vec1: CMuVarNode, vec2: CMuVarNode, mask: CMuVarNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_vecty = from_MuTypeNode(vecty);
    let mut _arg_maskty = from_MuTypeNode(maskty);
    let mut _arg_vec1 = from_MuVarNode(vec1);
    let mut _arg_vec2 = from_MuVarNode(vec2);
    let mut _arg_mask = from_MuVarNode(mask);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_new(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, allocty: CMuTypeNode, exc_clause: CMuExcClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_allocty = from_MuTypeNode(allocty);
    let mut _arg_exc_clause = from_node_optional(exc_clause);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_newhybrid(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, allocty: CMuTypeNode, lenty: CMuTypeNode, length: CMuVarNode, exc_clause: CMuExcClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_allocty = from_MuTypeNode(allocty);
    let mut _arg_lenty = from_MuTypeNode(lenty);
    let mut _arg_length = from_MuVarNode(length);
    let mut _arg_exc_clause = from_node_optional(exc_clause);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_alloca(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, allocty: CMuTypeNode, exc_clause: CMuExcClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_allocty = from_MuTypeNode(allocty);
    let mut _arg_exc_clause = from_node_optional(exc_clause);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_allocahybrid(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, allocty: CMuTypeNode, lenty: CMuTypeNode, length: CMuVarNode, exc_clause: CMuExcClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_allocty = from_MuTypeNode(allocty);
    let mut _arg_lenty = from_MuTypeNode(lenty);
    let mut _arg_length = from_MuVarNode(length);
    let mut _arg_exc_clause = from_node_optional(exc_clause);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_getiref(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, refty: CMuTypeNode, opnd: CMuVarNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_refty = from_MuTypeNode(refty);
    let mut _arg_opnd = from_MuVarNode(opnd);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_getfieldiref(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, is_ptr: CMuBool, refty: CMuTypeNode, index: c_int, opnd: CMuVarNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_is_ptr = from_MuBool(is_ptr);
    let mut _arg_refty = from_MuTypeNode(refty);
    let mut _arg_index = from_int(index);
    let mut _arg_opnd = from_MuVarNode(opnd);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_getelemiref(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, is_ptr: CMuBool, refty: CMuTypeNode, indty: CMuTypeNode, opnd: CMuVarNode, index: CMuVarNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_is_ptr = from_MuBool(is_ptr);
    let mut _arg_refty = from_MuTypeNode(refty);
    let mut _arg_indty = from_MuTypeNode(indty);
    let mut _arg_opnd = from_MuVarNode(opnd);
    let mut _arg_index = from_MuVarNode(index);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_shiftiref(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, is_ptr: CMuBool, refty: CMuTypeNode, offty: CMuTypeNode, opnd: CMuVarNode, offset: CMuVarNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_is_ptr = from_MuBool(is_ptr);
    let mut _arg_refty = from_MuTypeNode(refty);
    let mut _arg_offty = from_MuTypeNode(offty);
    let mut _arg_opnd = from_MuVarNode(opnd);
    let mut _arg_offset = from_MuVarNode(offset);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_getvarpartiref(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, is_ptr: CMuBool, refty: CMuTypeNode, opnd: CMuVarNode) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_is_ptr = from_MuBool(is_ptr);
    let mut _arg_refty = from_MuTypeNode(refty);
    let mut _arg_opnd = from_MuVarNode(opnd);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_load(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, is_ptr: CMuBool, ord: CMuMemOrd, refty: CMuTypeNode, loc: CMuVarNode, exc_clause: CMuExcClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_is_ptr = from_MuBool(is_ptr);
    let mut _arg_ord = from_MuMemOrd(ord);
    let mut _arg_refty = from_MuTypeNode(refty);
    let mut _arg_loc = from_MuVarNode(loc);
    let mut _arg_exc_clause = from_node_optional(exc_clause);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_store(b: *mut CMuIRBuilder, id: CMuID, is_ptr: CMuBool, ord: CMuMemOrd, refty: CMuTypeNode, loc: CMuVarNode, newval: CMuVarNode, exc_clause: CMuExcClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_is_ptr = from_MuBool(is_ptr);
    let mut _arg_ord = from_MuMemOrd(ord);
    let mut _arg_refty = from_MuTypeNode(refty);
    let mut _arg_loc = from_MuVarNode(loc);
    let mut _arg_newval = from_MuVarNode(newval);
    let mut _arg_exc_clause = from_node_optional(exc_clause);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_cmpxchg(b: *mut CMuIRBuilder, id: CMuID, value_result_id: CMuID, succ_result_id: CMuID, is_ptr: CMuBool, is_weak: CMuBool, ord_succ: CMuMemOrd, ord_fail: CMuMemOrd, refty: CMuTypeNode, loc: CMuVarNode, expected: CMuVarNode, desired: CMuVarNode, exc_clause: CMuExcClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_value_result_id = from_MuID(value_result_id);
    let mut _arg_succ_result_id = from_MuID(succ_result_id);
    let mut _arg_is_ptr = from_MuBool(is_ptr);
    let mut _arg_is_weak = from_MuBool(is_weak);
    let mut _arg_ord_succ = from_MuMemOrd(ord_succ);
    let mut _arg_ord_fail = from_MuMemOrd(ord_fail);
    let mut _arg_refty = from_MuTypeNode(refty);
    let mut _arg_loc = from_MuVarNode(loc);
    let mut _arg_expected = from_MuVarNode(expected);
    let mut _arg_desired = from_MuVarNode(desired);
    let mut _arg_exc_clause = from_node_optional(exc_clause);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_atomicrmw(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, is_ptr: CMuBool, ord: CMuMemOrd, optr: CMuAtomicRMWOptr, refTy: CMuTypeNode, loc: CMuVarNode, opnd: CMuVarNode, exc_clause: CMuExcClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_is_ptr = from_MuBool(is_ptr);
    let mut _arg_ord = from_MuMemOrd(ord);
    let mut _arg_optr = from_MuAtomicRMWOptr(optr);
    let mut _arg_refTy = from_MuTypeNode(refTy);
    let mut _arg_loc = from_MuVarNode(loc);
    let mut _arg_opnd = from_MuVarNode(opnd);
    let mut _arg_exc_clause = from_node_optional(exc_clause);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_fence(b: *mut CMuIRBuilder, id: CMuID, ord: CMuMemOrd) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_ord = from_MuMemOrd(ord);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_trap(b: *mut CMuIRBuilder, id: CMuID, result_ids: *mut CMuID, rettys: *mut CMuTypeNode, nretvals: CMuArraySize, exc_clause: CMuExcClause, keepalive_clause: CMuKeepaliveClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_ids = from_MuID_array(result_ids, nretvals);
    let mut _arg_rettys = from_MuID_array(rettys, nretvals);
    let mut _arg_exc_clause = from_node_optional(exc_clause);
    let mut _arg_keepalive_clause = from_node_optional(keepalive_clause);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_watchpoint(b: *mut CMuIRBuilder, id: CMuID, wpid: CMuWPID, result_ids: *mut CMuID, rettys: *mut CMuTypeNode, nretvals: CMuArraySize, dis: CMuDestClause, ena: CMuDestClause, exc: CMuDestClause, keepalive_clause: CMuKeepaliveClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_wpid = from_MuWPID(wpid);
    let mut _arg_result_ids = from_MuID_array(result_ids, nretvals);
    let mut _arg_rettys = from_MuID_array(rettys, nretvals);
    let mut _arg_dis = from_MuDestClause(dis);
    let mut _arg_ena = from_MuDestClause(ena);
    let mut _arg_exc = from_node_optional(exc);
    let mut _arg_keepalive_clause = from_node_optional(keepalive_clause);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_wpbranch(b: *mut CMuIRBuilder, id: CMuID, wpid: CMuWPID, dis: CMuDestClause, ena: CMuDestClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_wpid = from_MuWPID(wpid);
    let mut _arg_dis = from_MuDestClause(dis);
    let mut _arg_ena = from_MuDestClause(ena);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_ccall(b: *mut CMuIRBuilder, id: CMuID, result_ids: *mut CMuID, n_result_ids: CMuArraySize, callconv: CMuCallConv, callee_ty: CMuTypeNode, sig: CMuFuncSigNode, callee: CMuVarNode, args: *mut CMuVarNode, nargs: CMuArraySize, exc_clause: CMuExcClause, keepalive_clause: CMuKeepaliveClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_ids = from_MuID_array(result_ids, n_result_ids);
    let mut _arg_callconv = from_MuCallConv(callconv);
    let mut _arg_callee_ty = from_MuTypeNode(callee_ty);
    let mut _arg_sig = from_MuFuncSigNode(sig);
    let mut _arg_callee = from_MuVarNode(callee);
    let mut _arg_args = from_MuID_array(args, nargs);
    let mut _arg_exc_clause = from_node_optional(exc_clause);
    let mut _arg_keepalive_clause = from_node_optional(keepalive_clause);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_newthread(b: *mut CMuIRBuilder, id: CMuID, result_id: CMuID, stack: CMuVarNode, threadlocal: CMuVarNode, new_stack_clause: CMuNewStackClause, exc_clause: CMuExcClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_id = from_MuID(result_id);
    let mut _arg_stack = from_MuVarNode(stack);
    let mut _arg_threadlocal = from_node_optional(threadlocal);
    let mut _arg_new_stack_clause = from_MuNewStackClause(new_stack_clause);
    let mut _arg_exc_clause = from_node_optional(exc_clause);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_swapstack(b: *mut CMuIRBuilder, id: CMuID, result_ids: *mut CMuID, n_result_ids: CMuArraySize, swappee: CMuVarNode, cur_stack_clause: CMuCurStackClause, new_stack_clause: CMuNewStackClause, exc_clause: CMuExcClause, keepalive_clause: CMuKeepaliveClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_ids = from_MuID_array(result_ids, n_result_ids);
    let mut _arg_swappee = from_MuVarNode(swappee);
    let mut _arg_cur_stack_clause = from_MuCurStackClause(cur_stack_clause);
    let mut _arg_new_stack_clause = from_MuNewStackClause(new_stack_clause);
    let mut _arg_exc_clause = from_node_optional(exc_clause);
    let mut _arg_keepalive_clause = from_node_optional(keepalive_clause);
    panic!("not implemented")
}

fn _forwarder__MuIRBuilder__new_comminst(b: *mut CMuIRBuilder, id: CMuID, result_ids: *mut CMuID, n_result_ids: CMuArraySize, opcode: CMuCommInst, flags: *mut CMuFlag, nflags: CMuArraySize, tys: *mut CMuTypeNode, ntys: CMuArraySize, sigs: *mut CMuFuncSigNode, nsigs: CMuArraySize, args: *mut CMuVarNode, nargs: CMuArraySize, exc_clause: CMuExcClause, keepalive_clause: CMuKeepaliveClause) {
    let mut _arg_b = from_MuIRBuilder_ptr(b);
    let mut _arg_id = from_MuID(id);
    let mut _arg_result_ids = from_MuID_array(result_ids, n_result_ids);
    let mut _arg_opcode = from_MuCommInst(opcode);
    let mut _arg_flags = from_MuFlag_array(flags, nflags);
    let mut _arg_tys = from_MuID_array(tys, ntys);
    let mut _arg_sigs = from_MuID_array(sigs, nsigs);
    let mut _arg_args = from_MuID_array(args, nargs);
    let mut _arg_exc_clause = from_node_optional(exc_clause);
    let mut _arg_keepalive_clause = from_node_optional(keepalive_clause);
    panic!("not implemented")
}
// GEN:END:Forwarders
