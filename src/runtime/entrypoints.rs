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

use runtime::ValueLocation;

use ast::ir;
use ast::ir::*;
use ast::ptr::*;
use ast::types::*;
use compiler::backend::RegGroup;

use std::sync::{Arc, RwLock};
pub type EntryFuncSig = MuFuncSig;

pub struct RuntimeEntrypoint {
    pub sig: P<MuFuncSig>,
    pub aot: ValueLocation,
    pub jit: RwLock<Option<ValueLocation>>
}

impl RuntimeEntrypoint {
    fn new(c_name: &str, arg_tys: Vec<P<MuType>>, ret_tys: Vec<P<MuType>>) -> RuntimeEntrypoint {
        RuntimeEntrypoint {
            sig: P(MuFuncSig {
                hdr: MuEntityHeader::unnamed(ir::new_internal_id()),
                ret_tys: ret_tys,
                arg_tys: arg_tys
            }),
            aot: ValueLocation::Relocatable(RegGroup::GPR, Arc::new(c_name.to_string())),
            jit: RwLock::new(None)
        }
    }
}

// decl: thread.rs
lazy_static! {
    // impl: runtime_ARCH_OS.c
    pub static ref GET_THREAD_LOCAL : RuntimeEntrypoint = RuntimeEntrypoint::new(
        "muentry_get_thread_local",
        vec![],
        vec![ADDRESS_TYPE.clone()]);
    pub static ref SET_RETVAL : RuntimeEntrypoint = RuntimeEntrypoint::new(
        "muentry_set_retval",
        vec![UINT32_TYPE.clone()],
        vec![]);
    pub static ref THREAD_EXIT : RuntimeEntrypoint = RuntimeEntrypoint::new(
        "muentry_thread_exit",
        vec![ADDRESS_TYPE.clone()],
        vec![]);

    // impl: thread.rs
    pub static ref NEW_STACK: RuntimeEntrypoint = RuntimeEntrypoint::new(
        "muentry_new_stack",
        vec![ADDRESS_TYPE.clone(), ADDRESS_TYPE.clone()],
        vec![STACKREF_TYPE.clone()]);
    pub static ref KILL_STACK: RuntimeEntrypoint = RuntimeEntrypoint::new(
        "muentry_kill_stack",
        vec![STACKREF_TYPE.clone()],
        vec![]);
    pub static ref SAFECALL_KILL_STACK: RuntimeEntrypoint = RuntimeEntrypoint::new(
        "muentry_safecall_kill_stack",
        vec![STACKREF_TYPE.clone()],
        vec![]);
    pub static ref NEW_THREAD_NORMAL: RuntimeEntrypoint = RuntimeEntrypoint::new(
        "muentry_new_thread_normal",
        vec![STACKREF_TYPE.clone(), REF_VOID_TYPE.clone()],
        vec![THREADREF_TYPE.clone()]);
    pub static ref NEW_THREAD_EXCEPTIONAL: RuntimeEntrypoint = RuntimeEntrypoint::new(
        "muentry_new_thread_exceptional",
        vec![STACKREF_TYPE.clone(), REF_VOID_TYPE.clone(), REF_VOID_TYPE.clone()],
        vec![THREADREF_TYPE.clone()]);
}

// impl/decl: gc/lib.rs
lazy_static! {
    pub static ref ALLOC_TINY: RuntimeEntrypoint = RuntimeEntrypoint::new(
        "muentry_alloc_tiny",
        vec![ADDRESS_TYPE.clone(), UINT64_TYPE.clone(), UINT64_TYPE.clone()],
        vec![ADDRESS_TYPE.clone()]
    );
    pub static ref ALLOC_TINY_SLOW: RuntimeEntrypoint = RuntimeEntrypoint::new(
        "muentry_alloc_tiny_slow",
        vec![ADDRESS_TYPE.clone(), UINT64_TYPE.clone(), UINT64_TYPE.clone()],
        vec![ADDRESS_TYPE.clone()]
    );
    pub static ref ALLOC_NORMAL: RuntimeEntrypoint = RuntimeEntrypoint::new(
        "muentry_alloc_normal",
        vec![ADDRESS_TYPE.clone(), UINT64_TYPE.clone(), UINT64_TYPE.clone()],
        vec![ADDRESS_TYPE.clone()]
    );
    pub static ref ALLOC_NORMAL_SLOW: RuntimeEntrypoint = RuntimeEntrypoint::new(
        "muentry_alloc_normal_slow",
        vec![ADDRESS_TYPE.clone(), UINT64_TYPE.clone(), UINT64_TYPE.clone()],
        vec![ADDRESS_TYPE.clone()]
    );
    pub static ref ALLOC_LARGE: RuntimeEntrypoint = RuntimeEntrypoint::new(
        "muentry_alloc_large",
        vec![ADDRESS_TYPE.clone(), UINT64_TYPE.clone(), UINT64_TYPE.clone()],
        vec![ADDRESS_TYPE.clone()]
    );
    pub static ref ALLOC_VAR_SIZE: RuntimeEntrypoint = RuntimeEntrypoint::new(
        "muentry_alloc_var_size",
        vec![
            UINT64_TYPE.clone(),
            UINT64_TYPE.clone(),
            UINT64_TYPE.clone(),
            UINT64_TYPE.clone(),
            UINT64_TYPE.clone(),
            UINT64_TYPE.clone(),
        ],
        vec![ADDRESS_TYPE.clone()]
    );
    pub static ref INIT_TINY: RuntimeEntrypoint = RuntimeEntrypoint::new(
        "muentry_init_tiny_object",
        vec![ADDRESS_TYPE.clone(), ADDRESS_TYPE.clone(), UINT8_TYPE.clone()],
        vec![]
    );
    pub static ref INIT_SMALL: RuntimeEntrypoint = RuntimeEntrypoint::new(
        "muentry_init_small_object",
        vec![ADDRESS_TYPE.clone(), ADDRESS_TYPE.clone(), UINT16_TYPE.clone()],
        vec![]
    );
    pub static ref INIT_MEDIUM: RuntimeEntrypoint = RuntimeEntrypoint::new(
        "muentry_init_medium_object",
        vec![ADDRESS_TYPE.clone(), ADDRESS_TYPE.clone(), UINT32_TYPE.clone()],
        vec![]
    );
    pub static ref INIT_LARGE: RuntimeEntrypoint = RuntimeEntrypoint::new(
        "muentry_init_large_object",
        vec![
            ADDRESS_TYPE.clone(),
            ADDRESS_TYPE.clone(),
            UINT64_TYPE.clone(),
            UINT64_TYPE.clone(),
        ],
        vec![]
    );
    pub static ref PIN_OBJECT: RuntimeEntrypoint = RuntimeEntrypoint::new(
        "muentry_pin_object",
        vec![ADDRESS_TYPE.clone()],
        vec![ADDRESS_TYPE.clone()]
    );
    pub static ref UNPIN_OBJECT: RuntimeEntrypoint =
        RuntimeEntrypoint::new("muentry_unpin_object", vec![ADDRESS_TYPE.clone()], vec![]);
}

// decl: exception.rs
lazy_static! {
    // impl: exception.rs
    pub static ref THROW_EXCEPTION : RuntimeEntrypoint = RuntimeEntrypoint::new(
        "muentry_throw_exception",
        vec![ADDRESS_TYPE.clone()],
        vec![]);
    // impl: runtime_ARCH_OS.S
    pub static ref THROW_EXCEPTION_INTERNAL: RuntimeEntrypoint = RuntimeEntrypoint::new(
        "throw_exception_internal",
        vec![ADDRESS_TYPE.clone(), ADDRESS_TYPE.clone()],
        vec![]);
}

// impl/decl: math.rs
lazy_static! {
    pub static ref FREM32: RuntimeEntrypoint = RuntimeEntrypoint::new(
        "muentry_frem32",
        vec![FLOAT_TYPE.clone(), FLOAT_TYPE.clone()],
        vec![FLOAT_TYPE.clone()]
    );
    pub static ref FREM64: RuntimeEntrypoint = RuntimeEntrypoint::new(
        "muentry_frem64",
        vec![DOUBLE_TYPE.clone(), DOUBLE_TYPE.clone()],
        vec![DOUBLE_TYPE.clone()]
    );
    pub static ref UDIV_U128: RuntimeEntrypoint = RuntimeEntrypoint::new(
        "muentry_udiv_u128",
        vec![UINT128_TYPE.clone(), UINT128_TYPE.clone()],
        vec![UINT128_TYPE.clone()]
    );
    pub static ref SDIV_I128: RuntimeEntrypoint = RuntimeEntrypoint::new(
        "muentry_sdiv_i128",
        vec![UINT128_TYPE.clone(), UINT128_TYPE.clone()],
        vec![UINT128_TYPE.clone()]
    );
    pub static ref UREM_U128: RuntimeEntrypoint = RuntimeEntrypoint::new(
        "muentry_urem_u128",
        vec![UINT128_TYPE.clone(), UINT128_TYPE.clone()],
        vec![UINT128_TYPE.clone()]
    );
    pub static ref SREM_I128: RuntimeEntrypoint = RuntimeEntrypoint::new(
        "muentry_srem_i128",
        vec![UINT128_TYPE.clone(), UINT128_TYPE.clone()],
        vec![UINT128_TYPE.clone()]
    );
    pub static ref FPTOUI_DOUBLE_U128: RuntimeEntrypoint = RuntimeEntrypoint::new(
        "muentry_fptoui_double_u128",
        vec![DOUBLE_TYPE.clone()],
        vec![UINT128_TYPE.clone()]
    );
    pub static ref FPTOSI_DOUBLE_I128: RuntimeEntrypoint = RuntimeEntrypoint::new(
        "muentry_fptosi_double_i128",
        vec![DOUBLE_TYPE.clone()],
        vec![UINT128_TYPE.clone()]
    );
    pub static ref UITOFP_U128_DOUBLE: RuntimeEntrypoint = RuntimeEntrypoint::new(
        "muentry_uitofp_u128_double",
        vec![UINT128_TYPE.clone()],
        vec![DOUBLE_TYPE.clone()]
    );
    pub static ref SITOFP_I128_DOUBLE: RuntimeEntrypoint = RuntimeEntrypoint::new(
        "muentry_sitofp_i128_double",
        vec![UINT128_TYPE.clone()],
        vec![DOUBLE_TYPE.clone()]
    );
    pub static ref FPTOUI_FLOAT_U128: RuntimeEntrypoint = RuntimeEntrypoint::new(
        "muentry_fptoui_float_u128",
        vec![FLOAT_TYPE.clone()],
        vec![UINT128_TYPE.clone()]
    );
    pub static ref FPTOSI_FLOAT_I128: RuntimeEntrypoint = RuntimeEntrypoint::new(
        "muentry_fptosi_float_i128",
        vec![FLOAT_TYPE.clone()],
        vec![UINT128_TYPE.clone()]
    );
    pub static ref UITOFP_U128_FLOAT: RuntimeEntrypoint = RuntimeEntrypoint::new(
        "muentry_uitofp_u128_float",
        vec![UINT128_TYPE.clone()],
        vec![FLOAT_TYPE.clone()]
    );
    pub static ref SITOFP_I128_FLOAT: RuntimeEntrypoint = RuntimeEntrypoint::new(
        "muentry_sitofp_i128_float",
        vec![UINT128_TYPE.clone()],
        vec![FLOAT_TYPE.clone()]
    );
}

// impl/decl: mod.rs
lazy_static! {
    pub static ref PRINT_HEX: RuntimeEntrypoint =
        RuntimeEntrypoint::new("muentry_print_hex", vec![UINT64_TYPE.clone()], vec![]);
    pub static ref MEM_ZERO: RuntimeEntrypoint = RuntimeEntrypoint::new(
        "muentry_mem_zero",
        vec![IREF_VOID_TYPE.clone(), UINT64_TYPE.clone()],
        vec![]
    );
}
