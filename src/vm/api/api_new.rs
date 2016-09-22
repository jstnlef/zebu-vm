use vm::VM;
use ast::bundle::*;

use std::mem;
use std::collections::HashMap;
use std::sync::Arc;

use std::ffi::CStr;
use std::os::raw::c_int;
use std::os::raw::c_char;
use ast::ir::MuID;
pub type APIMuName = *const c_char;

macro_rules! unimplemented_api {
    () => {
        // cast fn to usize before transmute, see: https://github.com/rust-lang/rust/issues/19925
        unsafe {mem::transmute(unimplemented_api as usize)}
    }
}

macro_rules! api {
    ($func: expr) => {
        mem::transmute($func as usize)
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
    
    id_of: fn (mvm: *const MuVM, name: APIMuName) -> MuID,
    name_of : fn (mvm: *const MuVM, id: MuID) -> APIMuName,
    
    // unimplemented api
    set_trap_handler: fn () -> ()
}

impl MuVM {
    pub fn new() -> *mut MuVM {
        let vm = Box::new(MuVM {
            internal: Arc::new(VM::new()),
            new_context: unsafe{api!(MuVM::new_context)},
            id_of: unsafe{api!(MuVM::id_of)},
            name_of: unsafe{api!(MuVM::name_of)},
            set_trap_handler: unimplemented_api!()
        });
        
        Box::into_raw(vm)
    }
    
    pub fn new_context(*mut self) -> *mut MuCtx {
        let a : &mut self = self.as_mut().unwarp()
        unimplemented!()
    }
    
    pub fn id_of(&self, name: APIMuName) -> MuID {
        let name = unsafe {CStr::from_ptr(name)}.to_str().unwrap();
        self.internal.id_of(name)
    }
    
    pub fn name_of(&self, id: MuID) -> APIMuName {
        self.internal.name_of(id).as_ptr() as *const c_char
    }
}

#[repr(C)]
pub struct MuCtx {
    // void* header
    internal: Arc<VM>,
    
    // GENERATE_BEGIN: MuCtx
    
    // GENERATE_END: MuCtx
}