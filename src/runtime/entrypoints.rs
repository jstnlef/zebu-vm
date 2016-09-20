use runtime;
use runtime::ValueLocation;

use ast::ir;
use ast::ir::*;
use ast::ptr::*;
use ast::types::MuFuncSig;
use compiler::backend::RegGroup;

use std::sync::RwLock;

pub type EntryFuncSig = MuFuncSig;

pub struct RuntimeEntrypoint {
    pub sig: P<MuFuncSig>,
    pub aot: ValueLocation,
    pub jit: RwLock<Option<ValueLocation>>
}

lazy_static! {
    pub static ref GET_THREAD_LOCAL : RuntimeEntrypoint = RuntimeEntrypoint {
        sig: P(MuFuncSig {
            hdr: MuEntityHeader::unnamed(ir::new_internal_id()),
            ret_tys: vec![runtime::ADDRESS_TYPE.clone()],
            arg_tys: vec![]
        }),
        aot: ValueLocation::Relocatable(RegGroup::GPR, String::from("get_thread_local")),
        jit: RwLock::new(None),
    };
    
    pub static ref SWAP_BACK_TO_NATIVE_STACK : RuntimeEntrypoint = RuntimeEntrypoint {
        sig: P(MuFuncSig{
            hdr: MuEntityHeader::unnamed(ir::new_internal_id()),
            ret_tys: vec![],
            arg_tys: vec![runtime::ADDRESS_TYPE.clone()]
        }),
        aot: ValueLocation::Relocatable(RegGroup::GPR, String::from("swap_back_to_native_stack")),
        jit: RwLock::new(None),
    };
    
    pub static ref ALLOC_SLOW : RuntimeEntrypoint = RuntimeEntrypoint {
        sig: P(MuFuncSig {
            hdr: MuEntityHeader::unnamed(ir::new_internal_id()),
            ret_tys: vec![runtime::ADDRESS_TYPE.clone()],
            arg_tys: vec![runtime::UINT64_TYPE.clone(), runtime::UINT64_TYPE.clone()]
        }),
        aot: ValueLocation::Relocatable(RegGroup::GPR, String::from("alloc_slow")),
        jit: RwLock::new(None),
    };
}