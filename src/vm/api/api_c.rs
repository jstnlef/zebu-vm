/*
 * This file contains the C-facing interfaces.
 *
 * It is basically the muapi.h header written in Rust. It does not contain any
 * implementation-specific code. Most codes are simply generated from muapi.h.
 */

// This file is for interfacing with C, so it is not idiomatic Rust code.
#![allow(non_camel_case_types)]
#![allow(dead_code)]

use std::os::raw::*;

// some hand-written function pointer types
pub type CMuCFP = extern fn();
pub type CMuValuesFreer = extern fn(*mut CMuValue, CMuCPtr);
pub type CMuTrapHandler = extern fn(
    // input parameters
    *mut CMuCtx,
    CMuThreadRefValue,
    CMuStackRefValue,
    CMuWPID,
    // output parameters
    *mut CMuTrapHandlerResult,
    *mut CMuStackRefValue,
    *mut *mut CMuValue,
    *mut CMuArraySize,
    *mut CMuValuesFreer,
    *mut CMuCPtr,
    *mut CMuRefValue,
    // input parameter (userdata)
    CMuCPtr);

// GEN:BEGIN:Types
pub type CMuValue = *mut c_void;
pub type CMuSeqValue = CMuValue;
pub type CMuGenRefValue = CMuValue;
pub type CMuIntValue = CMuValue;
pub type CMuFloatValue = CMuValue;
pub type CMuDoubleValue = CMuValue;
pub type CMuUPtrValue = CMuValue;
pub type CMuUFPValue = CMuValue;
pub type CMuStructValue = CMuValue;
pub type CMuArrayValue = CMuSeqValue;
pub type CMuVectorValue = CMuSeqValue;
pub type CMuRefValue = CMuGenRefValue;
pub type CMuIRefValue = CMuGenRefValue;
pub type CMuTagRef64Value = CMuGenRefValue;
pub type CMuFuncRefValue = CMuGenRefValue;
pub type CMuThreadRefValue = CMuGenRefValue;
pub type CMuStackRefValue = CMuGenRefValue;
pub type CMuFCRefValue = CMuGenRefValue;
pub type CMuIBRefValue = CMuGenRefValue;
pub type CMuCString = *mut c_char;
pub type CMuID = u32;
pub type CMuName = CMuCString;
pub type CMuCPtr = *mut c_void;
pub type CMuBool = c_int;
pub type CMuArraySize = usize;
pub type CMuWPID = u32;
pub type CMuFlag = u32;
pub type CMuTrapHandlerResult = CMuFlag;
pub type CMuBinOpStatus = CMuFlag;
pub type CMuBinOptr = CMuFlag;
pub type CMuCmpOptr = CMuFlag;
pub type CMuConvOptr = CMuFlag;
pub type CMuMemOrd = CMuFlag;
pub type CMuAtomicRMWOptr = CMuFlag;
pub type CMuCallConv = CMuFlag;
pub type CMuCommInst = CMuFlag;
pub type CMuTypeNode = CMuID;
pub type CMuFuncSigNode = CMuID;
pub type CMuVarNode = CMuID;
pub type CMuGlobalVarNode = CMuID;
pub type CMuLocalVarNode = CMuID;
pub type CMuConstNode = CMuID;
pub type CMuFuncNode = CMuID;
pub type CMuFuncVerNode = CMuID;
pub type CMuBBNode = CMuID;
pub type CMuInstNode = CMuID;
pub type CMuDestClause = CMuID;
pub type CMuExcClause = CMuID;
pub type CMuKeepaliveClause = CMuID;
pub type CMuCurStackClause = CMuID;
pub type CMuNewStackClause = CMuID;
// GEN:END:Types

// GEN:BEGIN:Structs
#[repr(C)]
pub struct CMuVM {
    pub header: *mut c_void,
    pub new_context: fn(*mut CMuVM)-> *mut CMuCtx,
    pub id_of: fn(*mut CMuVM, CMuName)-> CMuID,
    pub name_of: fn(*mut CMuVM, CMuID)-> CMuName,
    pub set_trap_handler: fn(*mut CMuVM, CMuTrapHandler, CMuCPtr),
}

#[repr(C)]
pub struct CMuCtx {
    pub header: *mut c_void,
    pub id_of: fn(*mut CMuCtx, CMuName)-> CMuID,
    pub name_of: fn(*mut CMuCtx, CMuID)-> CMuName,
    pub close_context: fn(*mut CMuCtx),
    pub load_bundle: fn(*mut CMuCtx, *mut c_char, CMuArraySize),
    pub load_hail: fn(*mut CMuCtx, *mut c_char, CMuArraySize),
    pub handle_from_sint8: fn(*mut CMuCtx, i8, c_int)-> CMuIntValue,
    pub handle_from_uint8: fn(*mut CMuCtx, u8, c_int)-> CMuIntValue,
    pub handle_from_sint16: fn(*mut CMuCtx, i16, c_int)-> CMuIntValue,
    pub handle_from_uint16: fn(*mut CMuCtx, u16, c_int)-> CMuIntValue,
    pub handle_from_sint32: fn(*mut CMuCtx, i32, c_int)-> CMuIntValue,
    pub handle_from_uint32: fn(*mut CMuCtx, u32, c_int)-> CMuIntValue,
    pub handle_from_sint64: fn(*mut CMuCtx, i64, c_int)-> CMuIntValue,
    pub handle_from_uint64: fn(*mut CMuCtx, u64, c_int)-> CMuIntValue,
    pub handle_from_uint64s: fn(*mut CMuCtx, *mut u64, CMuArraySize, c_int)-> CMuIntValue,
    pub handle_from_float: fn(*mut CMuCtx, f32)-> CMuFloatValue,
    pub handle_from_double: fn(*mut CMuCtx, f64)-> CMuDoubleValue,
    pub handle_from_ptr: fn(*mut CMuCtx, CMuID, CMuCPtr)-> CMuUPtrValue,
    pub handle_from_fp: fn(*mut CMuCtx, CMuID, CMuCFP)-> CMuUFPValue,
    pub handle_to_sint8: fn(*mut CMuCtx, CMuIntValue)-> i8,
    pub handle_to_uint8: fn(*mut CMuCtx, CMuIntValue)-> u8,
    pub handle_to_sint16: fn(*mut CMuCtx, CMuIntValue)-> i16,
    pub handle_to_uint16: fn(*mut CMuCtx, CMuIntValue)-> u16,
    pub handle_to_sint32: fn(*mut CMuCtx, CMuIntValue)-> i32,
    pub handle_to_uint32: fn(*mut CMuCtx, CMuIntValue)-> u32,
    pub handle_to_sint64: fn(*mut CMuCtx, CMuIntValue)-> i64,
    pub handle_to_uint64: fn(*mut CMuCtx, CMuIntValue)-> u64,
    pub handle_to_float: fn(*mut CMuCtx, CMuFloatValue)-> f32,
    pub handle_to_double: fn(*mut CMuCtx, CMuDoubleValue)-> f64,
    pub handle_to_ptr: fn(*mut CMuCtx, CMuUPtrValue)-> CMuCPtr,
    pub handle_to_fp: fn(*mut CMuCtx, CMuUFPValue)-> CMuCFP,
    pub handle_from_const: fn(*mut CMuCtx, CMuID)-> CMuValue,
    pub handle_from_global: fn(*mut CMuCtx, CMuID)-> CMuIRefValue,
    pub handle_from_func: fn(*mut CMuCtx, CMuID)-> CMuFuncRefValue,
    pub handle_from_expose: fn(*mut CMuCtx, CMuID)-> CMuValue,
    pub delete_value: fn(*mut CMuCtx, CMuValue),
    pub ref_eq: fn(*mut CMuCtx, CMuGenRefValue, CMuGenRefValue)-> CMuBool,
    pub ref_ult: fn(*mut CMuCtx, CMuIRefValue, CMuIRefValue)-> CMuBool,
    pub extract_value: fn(*mut CMuCtx, CMuStructValue, c_int)-> CMuValue,
    pub insert_value: fn(*mut CMuCtx, CMuStructValue, c_int, CMuValue)-> CMuStructValue,
    pub extract_element: fn(*mut CMuCtx, CMuSeqValue, CMuIntValue)-> CMuValue,
    pub insert_element: fn(*mut CMuCtx, CMuSeqValue, CMuIntValue, CMuValue)-> CMuSeqValue,
    pub new_fixed: fn(*mut CMuCtx, CMuID)-> CMuRefValue,
    pub new_hybrid: fn(*mut CMuCtx, CMuID, CMuIntValue)-> CMuRefValue,
    pub refcast: fn(*mut CMuCtx, CMuGenRefValue, CMuID)-> CMuGenRefValue,
    pub get_iref: fn(*mut CMuCtx, CMuRefValue)-> CMuIRefValue,
    pub get_field_iref: fn(*mut CMuCtx, CMuIRefValue, c_int)-> CMuIRefValue,
    pub get_elem_iref: fn(*mut CMuCtx, CMuIRefValue, CMuIntValue)-> CMuIRefValue,
    pub shift_iref: fn(*mut CMuCtx, CMuIRefValue, CMuIntValue)-> CMuIRefValue,
    pub get_var_part_iref: fn(*mut CMuCtx, CMuIRefValue)-> CMuIRefValue,
    pub load: fn(*mut CMuCtx, CMuMemOrd, CMuIRefValue)-> CMuValue,
    pub store: fn(*mut CMuCtx, CMuMemOrd, CMuIRefValue, CMuValue),
    pub cmpxchg: fn(*mut CMuCtx, CMuMemOrd, CMuMemOrd, CMuBool, CMuIRefValue, CMuValue, CMuValue, *mut CMuBool)-> CMuValue,
    pub atomicrmw: fn(*mut CMuCtx, CMuMemOrd, CMuAtomicRMWOptr, CMuIRefValue, CMuValue)-> CMuValue,
    pub fence: fn(*mut CMuCtx, CMuMemOrd),
    pub new_stack: fn(*mut CMuCtx, CMuFuncRefValue)-> CMuStackRefValue,
    pub new_thread_nor: fn(*mut CMuCtx, CMuStackRefValue, CMuRefValue, *mut CMuValue, CMuArraySize)-> CMuThreadRefValue,
    pub new_thread_exc: fn(*mut CMuCtx, CMuStackRefValue, CMuRefValue, CMuRefValue)-> CMuThreadRefValue,
    pub kill_stack: fn(*mut CMuCtx, CMuStackRefValue),
    pub set_threadlocal: fn(*mut CMuCtx, CMuThreadRefValue, CMuRefValue),
    pub get_threadlocal: fn(*mut CMuCtx, CMuThreadRefValue)-> CMuRefValue,
    pub new_cursor: fn(*mut CMuCtx, CMuStackRefValue)-> CMuFCRefValue,
    pub next_frame: fn(*mut CMuCtx, CMuFCRefValue),
    pub copy_cursor: fn(*mut CMuCtx, CMuFCRefValue)-> CMuFCRefValue,
    pub close_cursor: fn(*mut CMuCtx, CMuFCRefValue),
    pub cur_func: fn(*mut CMuCtx, CMuFCRefValue)-> CMuID,
    pub cur_func_ver: fn(*mut CMuCtx, CMuFCRefValue)-> CMuID,
    pub cur_inst: fn(*mut CMuCtx, CMuFCRefValue)-> CMuID,
    pub dump_keepalives: fn(*mut CMuCtx, CMuFCRefValue, *mut CMuValue),
    pub pop_frames_to: fn(*mut CMuCtx, CMuFCRefValue),
    pub push_frame: fn(*mut CMuCtx, CMuStackRefValue, CMuFuncRefValue),
    pub tr64_is_fp: fn(*mut CMuCtx, CMuTagRef64Value)-> CMuBool,
    pub tr64_is_int: fn(*mut CMuCtx, CMuTagRef64Value)-> CMuBool,
    pub tr64_is_ref: fn(*mut CMuCtx, CMuTagRef64Value)-> CMuBool,
    pub tr64_to_fp: fn(*mut CMuCtx, CMuTagRef64Value)-> CMuDoubleValue,
    pub tr64_to_int: fn(*mut CMuCtx, CMuTagRef64Value)-> CMuIntValue,
    pub tr64_to_ref: fn(*mut CMuCtx, CMuTagRef64Value)-> CMuRefValue,
    pub tr64_to_tag: fn(*mut CMuCtx, CMuTagRef64Value)-> CMuIntValue,
    pub tr64_from_fp: fn(*mut CMuCtx, CMuDoubleValue)-> CMuTagRef64Value,
    pub tr64_from_int: fn(*mut CMuCtx, CMuIntValue)-> CMuTagRef64Value,
    pub tr64_from_ref: fn(*mut CMuCtx, CMuRefValue, CMuIntValue)-> CMuTagRef64Value,
    pub enable_watchpoint: fn(*mut CMuCtx, CMuWPID),
    pub disable_watchpoint: fn(*mut CMuCtx, CMuWPID),
    pub pin: fn(*mut CMuCtx, CMuValue)-> CMuUPtrValue,
    pub unpin: fn(*mut CMuCtx, CMuValue),
    pub get_addr: fn(*mut CMuCtx, CMuValue)-> CMuUPtrValue,
    pub expose: fn(*mut CMuCtx, CMuFuncRefValue, CMuCallConv, CMuIntValue)-> CMuValue,
    pub unexpose: fn(*mut CMuCtx, CMuCallConv, CMuValue),
    pub new_ir_builder: fn(*mut CMuCtx)-> *mut CMuIRBuilder,
    pub make_boot_image: fn(*mut CMuCtx, *mut CMuID, CMuArraySize, CMuFuncRefValue, CMuStackRefValue, CMuRefValue, *mut CMuIRefValue, *mut CMuCString, CMuArraySize, *mut CMuIRefValue, *mut CMuCString, CMuArraySize, CMuCString),
}

#[repr(C)]
pub struct CMuIRBuilder {
    pub header: *mut c_void,
    pub load: fn(*mut CMuIRBuilder),
    pub abort: fn(*mut CMuIRBuilder),
    pub gen_sym: fn(*mut CMuIRBuilder, CMuCString)-> CMuID,
    pub new_type_int: fn(*mut CMuIRBuilder, CMuID, c_int),
    pub new_type_float: fn(*mut CMuIRBuilder, CMuID),
    pub new_type_double: fn(*mut CMuIRBuilder, CMuID),
    pub new_type_uptr: fn(*mut CMuIRBuilder, CMuID, CMuTypeNode),
    pub new_type_ufuncptr: fn(*mut CMuIRBuilder, CMuID, CMuFuncSigNode),
    pub new_type_struct: fn(*mut CMuIRBuilder, CMuID, *mut CMuTypeNode, CMuArraySize),
    pub new_type_hybrid: fn(*mut CMuIRBuilder, CMuID, *mut CMuTypeNode, CMuArraySize, CMuTypeNode),
    pub new_type_array: fn(*mut CMuIRBuilder, CMuID, CMuTypeNode, u64),
    pub new_type_vector: fn(*mut CMuIRBuilder, CMuID, CMuTypeNode, u64),
    pub new_type_void: fn(*mut CMuIRBuilder, CMuID),
    pub new_type_ref: fn(*mut CMuIRBuilder, CMuID, CMuTypeNode),
    pub new_type_iref: fn(*mut CMuIRBuilder, CMuID, CMuTypeNode),
    pub new_type_weakref: fn(*mut CMuIRBuilder, CMuID, CMuTypeNode),
    pub new_type_funcref: fn(*mut CMuIRBuilder, CMuID, CMuFuncSigNode),
    pub new_type_tagref64: fn(*mut CMuIRBuilder, CMuID),
    pub new_type_threadref: fn(*mut CMuIRBuilder, CMuID),
    pub new_type_stackref: fn(*mut CMuIRBuilder, CMuID),
    pub new_type_framecursorref: fn(*mut CMuIRBuilder, CMuID),
    pub new_type_irbuilderref: fn(*mut CMuIRBuilder, CMuID),
    pub new_funcsig: fn(*mut CMuIRBuilder, CMuID, *mut CMuTypeNode, CMuArraySize, *mut CMuTypeNode, CMuArraySize),
    pub new_const_int: fn(*mut CMuIRBuilder, CMuID, CMuTypeNode, u64),
    pub new_const_int_ex: fn(*mut CMuIRBuilder, CMuID, CMuTypeNode, *mut u64, CMuArraySize),
    pub new_const_float: fn(*mut CMuIRBuilder, CMuID, CMuTypeNode, f32),
    pub new_const_double: fn(*mut CMuIRBuilder, CMuID, CMuTypeNode, f64),
    pub new_const_null: fn(*mut CMuIRBuilder, CMuID, CMuTypeNode),
    pub new_const_seq: fn(*mut CMuIRBuilder, CMuID, CMuTypeNode, *mut CMuGlobalVarNode, CMuArraySize),
    pub new_const_extern: fn(*mut CMuIRBuilder, CMuID, CMuTypeNode, CMuCString),
    pub new_global_cell: fn(*mut CMuIRBuilder, CMuID, CMuTypeNode),
    pub new_func: fn(*mut CMuIRBuilder, CMuID, CMuFuncSigNode),
    pub new_exp_func: fn(*mut CMuIRBuilder, CMuID, CMuFuncNode, CMuCallConv, CMuConstNode),
    pub new_func_ver: fn(*mut CMuIRBuilder, CMuID, CMuFuncNode, *mut CMuBBNode, CMuArraySize),
    pub new_bb: fn(*mut CMuIRBuilder, CMuID, *mut CMuID, *mut CMuTypeNode, CMuArraySize, CMuID, *mut CMuInstNode, CMuArraySize),
    pub new_dest_clause: fn(*mut CMuIRBuilder, CMuID, CMuBBNode, *mut CMuVarNode, CMuArraySize),
    pub new_exc_clause: fn(*mut CMuIRBuilder, CMuID, CMuDestClause, CMuDestClause),
    pub new_keepalive_clause: fn(*mut CMuIRBuilder, CMuID, *mut CMuLocalVarNode, CMuArraySize),
    pub new_csc_ret_with: fn(*mut CMuIRBuilder, CMuID, *mut CMuTypeNode, CMuArraySize),
    pub new_csc_kill_old: fn(*mut CMuIRBuilder, CMuID),
    pub new_nsc_pass_values: fn(*mut CMuIRBuilder, CMuID, *mut CMuTypeNode, *mut CMuVarNode, CMuArraySize),
    pub new_nsc_throw_exc: fn(*mut CMuIRBuilder, CMuID, CMuVarNode),
    pub new_binop: fn(*mut CMuIRBuilder, CMuID, CMuID, CMuBinOptr, CMuTypeNode, CMuVarNode, CMuVarNode, CMuExcClause),
    pub new_binop_with_status: fn(*mut CMuIRBuilder, CMuID, CMuID, *mut CMuID, CMuArraySize, CMuBinOptr, CMuBinOpStatus, CMuTypeNode, CMuVarNode, CMuVarNode, CMuExcClause),
    pub new_cmp: fn(*mut CMuIRBuilder, CMuID, CMuID, CMuCmpOptr, CMuTypeNode, CMuVarNode, CMuVarNode),
    pub new_conv: fn(*mut CMuIRBuilder, CMuID, CMuID, CMuConvOptr, CMuTypeNode, CMuTypeNode, CMuVarNode),
    pub new_select: fn(*mut CMuIRBuilder, CMuID, CMuID, CMuTypeNode, CMuTypeNode, CMuVarNode, CMuVarNode, CMuVarNode),
    pub new_branch: fn(*mut CMuIRBuilder, CMuID, CMuDestClause),
    pub new_branch2: fn(*mut CMuIRBuilder, CMuID, CMuVarNode, CMuDestClause, CMuDestClause),
    pub new_switch: fn(*mut CMuIRBuilder, CMuID, CMuTypeNode, CMuVarNode, CMuDestClause, *mut CMuConstNode, *mut CMuDestClause, CMuArraySize),
    pub new_call: fn(*mut CMuIRBuilder, CMuID, *mut CMuID, CMuArraySize, CMuFuncSigNode, CMuVarNode, *mut CMuVarNode, CMuArraySize, CMuExcClause, CMuKeepaliveClause),
    pub new_tailcall: fn(*mut CMuIRBuilder, CMuID, CMuFuncSigNode, CMuVarNode, *mut CMuVarNode, CMuArraySize),
    pub new_ret: fn(*mut CMuIRBuilder, CMuID, *mut CMuVarNode, CMuArraySize),
    pub new_throw: fn(*mut CMuIRBuilder, CMuID, CMuVarNode),
    pub new_extractvalue: fn(*mut CMuIRBuilder, CMuID, CMuID, CMuTypeNode, c_int, CMuVarNode),
    pub new_insertvalue: fn(*mut CMuIRBuilder, CMuID, CMuID, CMuTypeNode, c_int, CMuVarNode, CMuVarNode),
    pub new_extractelement: fn(*mut CMuIRBuilder, CMuID, CMuID, CMuTypeNode, CMuTypeNode, CMuVarNode, CMuVarNode),
    pub new_insertelement: fn(*mut CMuIRBuilder, CMuID, CMuID, CMuTypeNode, CMuTypeNode, CMuVarNode, CMuVarNode, CMuVarNode),
    pub new_shufflevector: fn(*mut CMuIRBuilder, CMuID, CMuID, CMuTypeNode, CMuTypeNode, CMuVarNode, CMuVarNode, CMuVarNode),
    pub new_new: fn(*mut CMuIRBuilder, CMuID, CMuID, CMuTypeNode, CMuExcClause),
    pub new_newhybrid: fn(*mut CMuIRBuilder, CMuID, CMuID, CMuTypeNode, CMuTypeNode, CMuVarNode, CMuExcClause),
    pub new_alloca: fn(*mut CMuIRBuilder, CMuID, CMuID, CMuTypeNode, CMuExcClause),
    pub new_allocahybrid: fn(*mut CMuIRBuilder, CMuID, CMuID, CMuTypeNode, CMuTypeNode, CMuVarNode, CMuExcClause),
    pub new_getiref: fn(*mut CMuIRBuilder, CMuID, CMuID, CMuTypeNode, CMuVarNode),
    pub new_getfieldiref: fn(*mut CMuIRBuilder, CMuID, CMuID, CMuBool, CMuTypeNode, c_int, CMuVarNode),
    pub new_getelemiref: fn(*mut CMuIRBuilder, CMuID, CMuID, CMuBool, CMuTypeNode, CMuTypeNode, CMuVarNode, CMuVarNode),
    pub new_shiftiref: fn(*mut CMuIRBuilder, CMuID, CMuID, CMuBool, CMuTypeNode, CMuTypeNode, CMuVarNode, CMuVarNode),
    pub new_getvarpartiref: fn(*mut CMuIRBuilder, CMuID, CMuID, CMuBool, CMuTypeNode, CMuVarNode),
    pub new_load: fn(*mut CMuIRBuilder, CMuID, CMuID, CMuBool, CMuMemOrd, CMuTypeNode, CMuVarNode, CMuExcClause),
    pub new_store: fn(*mut CMuIRBuilder, CMuID, CMuBool, CMuMemOrd, CMuTypeNode, CMuVarNode, CMuVarNode, CMuExcClause),
    pub new_cmpxchg: fn(*mut CMuIRBuilder, CMuID, CMuID, CMuID, CMuBool, CMuBool, CMuMemOrd, CMuMemOrd, CMuTypeNode, CMuVarNode, CMuVarNode, CMuVarNode, CMuExcClause),
    pub new_atomicrmw: fn(*mut CMuIRBuilder, CMuID, CMuID, CMuBool, CMuMemOrd, CMuAtomicRMWOptr, CMuTypeNode, CMuVarNode, CMuVarNode, CMuExcClause),
    pub new_fence: fn(*mut CMuIRBuilder, CMuID, CMuMemOrd),
    pub new_trap: fn(*mut CMuIRBuilder, CMuID, *mut CMuID, *mut CMuTypeNode, CMuArraySize, CMuExcClause, CMuKeepaliveClause),
    pub new_watchpoint: fn(*mut CMuIRBuilder, CMuID, CMuWPID, *mut CMuID, *mut CMuTypeNode, CMuArraySize, CMuDestClause, CMuDestClause, CMuDestClause, CMuKeepaliveClause),
    pub new_wpbranch: fn(*mut CMuIRBuilder, CMuID, CMuWPID, CMuDestClause, CMuDestClause),
    pub new_ccall: fn(*mut CMuIRBuilder, CMuID, *mut CMuID, CMuArraySize, CMuCallConv, CMuTypeNode, CMuFuncSigNode, CMuVarNode, *mut CMuVarNode, CMuArraySize, CMuExcClause, CMuKeepaliveClause),
    pub new_newthread: fn(*mut CMuIRBuilder, CMuID, CMuID, CMuVarNode, CMuVarNode, CMuNewStackClause, CMuExcClause),
    pub new_swapstack: fn(*mut CMuIRBuilder, CMuID, *mut CMuID, CMuArraySize, CMuVarNode, CMuCurStackClause, CMuNewStackClause, CMuExcClause, CMuKeepaliveClause),
    pub new_comminst: fn(*mut CMuIRBuilder, CMuID, *mut CMuID, CMuArraySize, CMuCommInst, *mut CMuFlag, CMuArraySize, *mut CMuTypeNode, CMuArraySize, *mut CMuFuncSigNode, CMuArraySize, *mut CMuVarNode, CMuArraySize, CMuExcClause, CMuKeepaliveClause),
}
// GEN:END:Structs

// GEN:BEGIN:Enums
pub const CMU_THREAD_EXIT: CMuTrapHandlerResult = 0x00;
pub const CMU_REBIND_PASS_VALUES: CMuTrapHandlerResult = 0x01;
pub const CMU_REBIND_THROW_EXC: CMuTrapHandlerResult = 0x02;
pub const CMU_BOS_N: CMuBinOpStatus = 0x01;
pub const CMU_BOS_Z: CMuBinOpStatus = 0x02;
pub const CMU_BOS_C: CMuBinOpStatus = 0x04;
pub const CMU_BOS_V: CMuBinOpStatus = 0x08;
pub const CMU_BINOP_ADD: CMuBinOptr = 0x01;
pub const CMU_BINOP_SUB: CMuBinOptr = 0x02;
pub const CMU_BINOP_MUL: CMuBinOptr = 0x03;
pub const CMU_BINOP_SDIV: CMuBinOptr = 0x04;
pub const CMU_BINOP_SREM: CMuBinOptr = 0x05;
pub const CMU_BINOP_UDIV: CMuBinOptr = 0x06;
pub const CMU_BINOP_UREM: CMuBinOptr = 0x07;
pub const CMU_BINOP_SHL: CMuBinOptr = 0x08;
pub const CMU_BINOP_LSHR: CMuBinOptr = 0x09;
pub const CMU_BINOP_ASHR: CMuBinOptr = 0x0A;
pub const CMU_BINOP_AND: CMuBinOptr = 0x0B;
pub const CMU_BINOP_OR: CMuBinOptr = 0x0C;
pub const CMU_BINOP_XOR: CMuBinOptr = 0x0D;
pub const CMU_BINOP_FADD: CMuBinOptr = 0xB0;
pub const CMU_BINOP_FSUB: CMuBinOptr = 0xB1;
pub const CMU_BINOP_FMUL: CMuBinOptr = 0xB2;
pub const CMU_BINOP_FDIV: CMuBinOptr = 0xB3;
pub const CMU_BINOP_FREM: CMuBinOptr = 0xB4;
pub const CMU_CMP_EQ: CMuCmpOptr = 0x20;
pub const CMU_CMP_NE: CMuCmpOptr = 0x21;
pub const CMU_CMP_SGE: CMuCmpOptr = 0x22;
pub const CMU_CMP_SGT: CMuCmpOptr = 0x23;
pub const CMU_CMP_SLE: CMuCmpOptr = 0x24;
pub const CMU_CMP_SLT: CMuCmpOptr = 0x25;
pub const CMU_CMP_UGE: CMuCmpOptr = 0x26;
pub const CMU_CMP_UGT: CMuCmpOptr = 0x27;
pub const CMU_CMP_ULE: CMuCmpOptr = 0x28;
pub const CMU_CMP_ULT: CMuCmpOptr = 0x29;
pub const CMU_CMP_FFALSE: CMuCmpOptr = 0xC0;
pub const CMU_CMP_FTRUE: CMuCmpOptr = 0xC1;
pub const CMU_CMP_FUNO: CMuCmpOptr = 0xC2;
pub const CMU_CMP_FUEQ: CMuCmpOptr = 0xC3;
pub const CMU_CMP_FUNE: CMuCmpOptr = 0xC4;
pub const CMU_CMP_FUGT: CMuCmpOptr = 0xC5;
pub const CMU_CMP_FUGE: CMuCmpOptr = 0xC6;
pub const CMU_CMP_FULT: CMuCmpOptr = 0xC7;
pub const CMU_CMP_FULE: CMuCmpOptr = 0xC8;
pub const CMU_CMP_FORD: CMuCmpOptr = 0xC9;
pub const CMU_CMP_FOEQ: CMuCmpOptr = 0xCA;
pub const CMU_CMP_FONE: CMuCmpOptr = 0xCB;
pub const CMU_CMP_FOGT: CMuCmpOptr = 0xCC;
pub const CMU_CMP_FOGE: CMuCmpOptr = 0xCD;
pub const CMU_CMP_FOLT: CMuCmpOptr = 0xCE;
pub const CMU_CMP_FOLE: CMuCmpOptr = 0xCF;
pub const CMU_CONV_TRUNC: CMuConvOptr = 0x30;
pub const CMU_CONV_ZEXT: CMuConvOptr = 0x31;
pub const CMU_CONV_SEXT: CMuConvOptr = 0x32;
pub const CMU_CONV_FPTRUNC: CMuConvOptr = 0x33;
pub const CMU_CONV_FPEXT: CMuConvOptr = 0x34;
pub const CMU_CONV_FPTOUI: CMuConvOptr = 0x35;
pub const CMU_CONV_FPTOSI: CMuConvOptr = 0x36;
pub const CMU_CONV_UITOFP: CMuConvOptr = 0x37;
pub const CMU_CONV_SITOFP: CMuConvOptr = 0x38;
pub const CMU_CONV_BITCAST: CMuConvOptr = 0x39;
pub const CMU_CONV_REFCAST: CMuConvOptr = 0x3A;
pub const CMU_CONV_PTRCAST: CMuConvOptr = 0x3B;
pub const CMU_ORD_NOT_ATOMIC: CMuMemOrd = 0x00;
pub const CMU_ORD_RELAXED: CMuMemOrd = 0x01;
pub const CMU_ORD_CONSUME: CMuMemOrd = 0x02;
pub const CMU_ORD_ACQUIRE: CMuMemOrd = 0x03;
pub const CMU_ORD_RELEASE: CMuMemOrd = 0x04;
pub const CMU_ORD_ACQ_REL: CMuMemOrd = 0x05;
pub const CMU_ORD_SEQ_CST: CMuMemOrd = 0x06;
pub const CMU_ARMW_XCHG: CMuAtomicRMWOptr = 0x00;
pub const CMU_ARMW_ADD: CMuAtomicRMWOptr = 0x01;
pub const CMU_ARMW_SUB: CMuAtomicRMWOptr = 0x02;
pub const CMU_ARMW_AND: CMuAtomicRMWOptr = 0x03;
pub const CMU_ARMW_NAND: CMuAtomicRMWOptr = 0x04;
pub const CMU_ARMW_OR: CMuAtomicRMWOptr = 0x05;
pub const CMU_ARMW_XOR: CMuAtomicRMWOptr = 0x06;
pub const CMU_ARMW_MAX: CMuAtomicRMWOptr = 0x07;
pub const CMU_ARMW_MIN: CMuAtomicRMWOptr = 0x08;
pub const CMU_ARMW_UMAX: CMuAtomicRMWOptr = 0x09;
pub const CMU_ARMW_UMIN: CMuAtomicRMWOptr = 0x0A;
pub const CMU_CC_DEFAULT: CMuCallConv = 0x00;
pub const CMU_CI_UVM_NEW_STACK: CMuCommInst = 0x201;
pub const CMU_CI_UVM_KILL_STACK: CMuCommInst = 0x202;
pub const CMU_CI_UVM_THREAD_EXIT: CMuCommInst = 0x203;
pub const CMU_CI_UVM_CURRENT_STACK: CMuCommInst = 0x204;
pub const CMU_CI_UVM_SET_THREADLOCAL: CMuCommInst = 0x205;
pub const CMU_CI_UVM_GET_THREADLOCAL: CMuCommInst = 0x206;
pub const CMU_CI_UVM_TR64_IS_FP: CMuCommInst = 0x211;
pub const CMU_CI_UVM_TR64_IS_INT: CMuCommInst = 0x212;
pub const CMU_CI_UVM_TR64_IS_REF: CMuCommInst = 0x213;
pub const CMU_CI_UVM_TR64_FROM_FP: CMuCommInst = 0x214;
pub const CMU_CI_UVM_TR64_FROM_INT: CMuCommInst = 0x215;
pub const CMU_CI_UVM_TR64_FROM_REF: CMuCommInst = 0x216;
pub const CMU_CI_UVM_TR64_TO_FP: CMuCommInst = 0x217;
pub const CMU_CI_UVM_TR64_TO_INT: CMuCommInst = 0x218;
pub const CMU_CI_UVM_TR64_TO_REF: CMuCommInst = 0x219;
pub const CMU_CI_UVM_TR64_TO_TAG: CMuCommInst = 0x21a;
pub const CMU_CI_UVM_FUTEX_WAIT: CMuCommInst = 0x220;
pub const CMU_CI_UVM_FUTEX_WAIT_TIMEOUT: CMuCommInst = 0x221;
pub const CMU_CI_UVM_FUTEX_WAKE: CMuCommInst = 0x222;
pub const CMU_CI_UVM_FUTEX_CMP_REQUEUE: CMuCommInst = 0x223;
pub const CMU_CI_UVM_KILL_DEPENDENCY: CMuCommInst = 0x230;
pub const CMU_CI_UVM_NATIVE_PIN: CMuCommInst = 0x240;
pub const CMU_CI_UVM_NATIVE_UNPIN: CMuCommInst = 0x241;
pub const CMU_CI_UVM_NATIVE_GET_ADDR: CMuCommInst = 0x242;
pub const CMU_CI_UVM_NATIVE_EXPOSE: CMuCommInst = 0x243;
pub const CMU_CI_UVM_NATIVE_UNEXPOSE: CMuCommInst = 0x244;
pub const CMU_CI_UVM_NATIVE_GET_COOKIE: CMuCommInst = 0x245;
pub const CMU_CI_UVM_META_ID_OF: CMuCommInst = 0x250;
pub const CMU_CI_UVM_META_NAME_OF: CMuCommInst = 0x251;
pub const CMU_CI_UVM_META_LOAD_BUNDLE: CMuCommInst = 0x252;
pub const CMU_CI_UVM_META_LOAD_HAIL: CMuCommInst = 0x253;
pub const CMU_CI_UVM_META_NEW_CURSOR: CMuCommInst = 0x254;
pub const CMU_CI_UVM_META_NEXT_FRAME: CMuCommInst = 0x255;
pub const CMU_CI_UVM_META_COPY_CURSOR: CMuCommInst = 0x256;
pub const CMU_CI_UVM_META_CLOSE_CURSOR: CMuCommInst = 0x257;
pub const CMU_CI_UVM_META_CUR_FUNC: CMuCommInst = 0x258;
pub const CMU_CI_UVM_META_CUR_FUNC_VER: CMuCommInst = 0x259;
pub const CMU_CI_UVM_META_CUR_INST: CMuCommInst = 0x25a;
pub const CMU_CI_UVM_META_DUMP_KEEPALIVES: CMuCommInst = 0x25b;
pub const CMU_CI_UVM_META_POP_FRAMES_TO: CMuCommInst = 0x25c;
pub const CMU_CI_UVM_META_PUSH_FRAME: CMuCommInst = 0x25d;
pub const CMU_CI_UVM_META_ENABLE_WATCHPOINT: CMuCommInst = 0x25e;
pub const CMU_CI_UVM_META_DISABLE_WATCHPOINT: CMuCommInst = 0x25f;
pub const CMU_CI_UVM_META_SET_TRAP_HANDLER: CMuCommInst = 0x260;
pub const CMU_CI_UVM_IRBUILDER_NEW_IR_BUILDER: CMuCommInst = 0x270;
pub const CMU_CI_UVM_IRBUILDER_LOAD: CMuCommInst = 0x300;
pub const CMU_CI_UVM_IRBUILDER_ABORT: CMuCommInst = 0x301;
pub const CMU_CI_UVM_IRBUILDER_GEN_SYM: CMuCommInst = 0x302;
pub const CMU_CI_UVM_IRBUILDER_NEW_TYPE_INT: CMuCommInst = 0x303;
pub const CMU_CI_UVM_IRBUILDER_NEW_TYPE_FLOAT: CMuCommInst = 0x304;
pub const CMU_CI_UVM_IRBUILDER_NEW_TYPE_DOUBLE: CMuCommInst = 0x305;
pub const CMU_CI_UVM_IRBUILDER_NEW_TYPE_UPTR: CMuCommInst = 0x306;
pub const CMU_CI_UVM_IRBUILDER_NEW_TYPE_UFUNCPTR: CMuCommInst = 0x307;
pub const CMU_CI_UVM_IRBUILDER_NEW_TYPE_STRUCT: CMuCommInst = 0x308;
pub const CMU_CI_UVM_IRBUILDER_NEW_TYPE_HYBRID: CMuCommInst = 0x309;
pub const CMU_CI_UVM_IRBUILDER_NEW_TYPE_ARRAY: CMuCommInst = 0x30a;
pub const CMU_CI_UVM_IRBUILDER_NEW_TYPE_VECTOR: CMuCommInst = 0x30b;
pub const CMU_CI_UVM_IRBUILDER_NEW_TYPE_VOID: CMuCommInst = 0x30c;
pub const CMU_CI_UVM_IRBUILDER_NEW_TYPE_REF: CMuCommInst = 0x30d;
pub const CMU_CI_UVM_IRBUILDER_NEW_TYPE_IREF: CMuCommInst = 0x30e;
pub const CMU_CI_UVM_IRBUILDER_NEW_TYPE_WEAKREF: CMuCommInst = 0x30f;
pub const CMU_CI_UVM_IRBUILDER_NEW_TYPE_FUNCREF: CMuCommInst = 0x310;
pub const CMU_CI_UVM_IRBUILDER_NEW_TYPE_TAGREF64: CMuCommInst = 0x311;
pub const CMU_CI_UVM_IRBUILDER_NEW_TYPE_THREADREF: CMuCommInst = 0x312;
pub const CMU_CI_UVM_IRBUILDER_NEW_TYPE_STACKREF: CMuCommInst = 0x313;
pub const CMU_CI_UVM_IRBUILDER_NEW_TYPE_FRAMECURSORREF: CMuCommInst = 0x314;
pub const CMU_CI_UVM_IRBUILDER_NEW_TYPE_IRBUILDERREF: CMuCommInst = 0x315;
pub const CMU_CI_UVM_IRBUILDER_NEW_FUNCSIG: CMuCommInst = 0x316;
pub const CMU_CI_UVM_IRBUILDER_NEW_CONST_INT: CMuCommInst = 0x317;
pub const CMU_CI_UVM_IRBUILDER_NEW_CONST_INT_EX: CMuCommInst = 0x318;
pub const CMU_CI_UVM_IRBUILDER_NEW_CONST_FLOAT: CMuCommInst = 0x319;
pub const CMU_CI_UVM_IRBUILDER_NEW_CONST_DOUBLE: CMuCommInst = 0x31a;
pub const CMU_CI_UVM_IRBUILDER_NEW_CONST_NULL: CMuCommInst = 0x31b;
pub const CMU_CI_UVM_IRBUILDER_NEW_CONST_SEQ: CMuCommInst = 0x31c;
pub const CMU_CI_UVM_IRBUILDER_NEW_CONST_EXTERN: CMuCommInst = 0x31d;
pub const CMU_CI_UVM_IRBUILDER_NEW_GLOBAL_CELL: CMuCommInst = 0x31e;
pub const CMU_CI_UVM_IRBUILDER_NEW_FUNC: CMuCommInst = 0x31f;
pub const CMU_CI_UVM_IRBUILDER_NEW_EXP_FUNC: CMuCommInst = 0x320;
pub const CMU_CI_UVM_IRBUILDER_NEW_FUNC_VER: CMuCommInst = 0x321;
pub const CMU_CI_UVM_IRBUILDER_NEW_BB: CMuCommInst = 0x322;
pub const CMU_CI_UVM_IRBUILDER_NEW_DEST_CLAUSE: CMuCommInst = 0x323;
pub const CMU_CI_UVM_IRBUILDER_NEW_EXC_CLAUSE: CMuCommInst = 0x324;
pub const CMU_CI_UVM_IRBUILDER_NEW_KEEPALIVE_CLAUSE: CMuCommInst = 0x325;
pub const CMU_CI_UVM_IRBUILDER_NEW_CSC_RET_WITH: CMuCommInst = 0x326;
pub const CMU_CI_UVM_IRBUILDER_NEW_CSC_KILL_OLD: CMuCommInst = 0x327;
pub const CMU_CI_UVM_IRBUILDER_NEW_NSC_PASS_VALUES: CMuCommInst = 0x328;
pub const CMU_CI_UVM_IRBUILDER_NEW_NSC_THROW_EXC: CMuCommInst = 0x329;
pub const CMU_CI_UVM_IRBUILDER_NEW_BINOP: CMuCommInst = 0x32a;
pub const CMU_CI_UVM_IRBUILDER_NEW_BINOP_WITH_STATUS: CMuCommInst = 0x32b;
pub const CMU_CI_UVM_IRBUILDER_NEW_CMP: CMuCommInst = 0x32c;
pub const CMU_CI_UVM_IRBUILDER_NEW_CONV: CMuCommInst = 0x32d;
pub const CMU_CI_UVM_IRBUILDER_NEW_SELECT: CMuCommInst = 0x32e;
pub const CMU_CI_UVM_IRBUILDER_NEW_BRANCH: CMuCommInst = 0x32f;
pub const CMU_CI_UVM_IRBUILDER_NEW_BRANCH2: CMuCommInst = 0x330;
pub const CMU_CI_UVM_IRBUILDER_NEW_SWITCH: CMuCommInst = 0x331;
pub const CMU_CI_UVM_IRBUILDER_NEW_CALL: CMuCommInst = 0x332;
pub const CMU_CI_UVM_IRBUILDER_NEW_TAILCALL: CMuCommInst = 0x333;
pub const CMU_CI_UVM_IRBUILDER_NEW_RET: CMuCommInst = 0x334;
pub const CMU_CI_UVM_IRBUILDER_NEW_THROW: CMuCommInst = 0x335;
pub const CMU_CI_UVM_IRBUILDER_NEW_EXTRACTVALUE: CMuCommInst = 0x336;
pub const CMU_CI_UVM_IRBUILDER_NEW_INSERTVALUE: CMuCommInst = 0x337;
pub const CMU_CI_UVM_IRBUILDER_NEW_EXTRACTELEMENT: CMuCommInst = 0x338;
pub const CMU_CI_UVM_IRBUILDER_NEW_INSERTELEMENT: CMuCommInst = 0x339;
pub const CMU_CI_UVM_IRBUILDER_NEW_SHUFFLEVECTOR: CMuCommInst = 0x33a;
pub const CMU_CI_UVM_IRBUILDER_NEW_NEW: CMuCommInst = 0x33b;
pub const CMU_CI_UVM_IRBUILDER_NEW_NEWHYBRID: CMuCommInst = 0x33c;
pub const CMU_CI_UVM_IRBUILDER_NEW_ALLOCA: CMuCommInst = 0x33d;
pub const CMU_CI_UVM_IRBUILDER_NEW_ALLOCAHYBRID: CMuCommInst = 0x33e;
pub const CMU_CI_UVM_IRBUILDER_NEW_GETIREF: CMuCommInst = 0x33f;
pub const CMU_CI_UVM_IRBUILDER_NEW_GETFIELDIREF: CMuCommInst = 0x340;
pub const CMU_CI_UVM_IRBUILDER_NEW_GETELEMIREF: CMuCommInst = 0x341;
pub const CMU_CI_UVM_IRBUILDER_NEW_SHIFTIREF: CMuCommInst = 0x342;
pub const CMU_CI_UVM_IRBUILDER_NEW_GETVARPARTIREF: CMuCommInst = 0x343;
pub const CMU_CI_UVM_IRBUILDER_NEW_LOAD: CMuCommInst = 0x344;
pub const CMU_CI_UVM_IRBUILDER_NEW_STORE: CMuCommInst = 0x345;
pub const CMU_CI_UVM_IRBUILDER_NEW_CMPXCHG: CMuCommInst = 0x346;
pub const CMU_CI_UVM_IRBUILDER_NEW_ATOMICRMW: CMuCommInst = 0x347;
pub const CMU_CI_UVM_IRBUILDER_NEW_FENCE: CMuCommInst = 0x348;
pub const CMU_CI_UVM_IRBUILDER_NEW_TRAP: CMuCommInst = 0x349;
pub const CMU_CI_UVM_IRBUILDER_NEW_WATCHPOINT: CMuCommInst = 0x34a;
pub const CMU_CI_UVM_IRBUILDER_NEW_WPBRANCH: CMuCommInst = 0x34b;
pub const CMU_CI_UVM_IRBUILDER_NEW_CCALL: CMuCommInst = 0x34c;
pub const CMU_CI_UVM_IRBUILDER_NEW_NEWTHREAD: CMuCommInst = 0x34d;
pub const CMU_CI_UVM_IRBUILDER_NEW_SWAPSTACK: CMuCommInst = 0x34e;
pub const CMU_CI_UVM_IRBUILDER_NEW_COMMINST: CMuCommInst = 0x34f;
// GEN:END:Enums
