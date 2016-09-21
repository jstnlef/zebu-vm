use ast::ptr::*;
use ast::ir::*;
use ast::types::*;
use vm::api;
use vm::VM;
use vm::bundle::*;

use std::mem;
use std::os::raw;
use std::collections::HashMap;
use std::sync::Arc;

macro_rules! unimplemented_api {
    () => {
        // cast fn to usize before transmute, see: https://github.com/rust-lang/rust/issues/19925
        unsafe {mem::transmute(unimplemented_api as usize)}
    }
}

macro_rules! api {
    ($func: expr) => {
        unsafe {mem::transmute($func as usize)}
    }
}

pub fn unimplemented_api() {
    unimplemented!()
}

#[repr(C)]
pub struct MuVM {
    // void* header
    internal: Arc<VM>,
    
    pub new_context: fn (mvm: *mut MuVM) -> *mut MuCtx,
    
    id_of: fn (mvm: *const MuVM, name: MuName) -> MuID,
    name_of : fn (mvm: *const MuVM, id: MuID) -> MuName,
    
    // set_trap_handler: fn(mvm: *mut MuVM, trap_handler: MuTrapHandler, user_data: MuCPtr)
}

impl MuVM {
    pub fn new() -> *mut MuVM {
        let vm = Box::new(MuVM {
            internal: Arc::new(VM::new()),
            new_context: api!(MuVM::new_context),
            id_of: api!(MuVM::id_of),
            name_of: api!(MuVM::name_of)
        });
        
        Box::into_raw(vm)
    }
    
    pub fn new_context(&mut self) -> *mut MuCtx {
        let ctx = Box::new(MuCtx::new(self.internal.clone()));
        
        Box::into_raw(ctx)
    }
    
    pub fn id_of(&self, name: &str) -> MuID {
        self.internal.id_of(name)
    }
    
    pub fn name_of(&self, id: MuID) -> MuName {
        self.internal.name_of(id)
    }
}

#[repr(C)]
pub struct MuCtx {
    // void* header - current not planed to use this
    internal: Box<MuCtxInternal>,
}

struct MuCtxInternal {
    vm: Arc<VM>,
    cur_bundles: HashMap<MuID, MuBundle>
}