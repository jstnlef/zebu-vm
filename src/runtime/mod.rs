pub mod mm;
pub mod thread;
pub mod entrypoints;

pub use runtime::mm::common::Address;
pub use runtime::mm::common::ObjectReference;

use utils;
use ast::ir;
use ast::ptr::*;
use ast::types::MuType_;
use ast::types::MuType;
use ast::ir::*;
use compiler::backend::Word;
use compiler::backend::RegGroup;

use std::os::raw::c_char;
use std::os::raw::c_void;
use std::ffi::CString;

lazy_static! {
    pub static ref ADDRESS_TYPE : P<MuType> = P(
        MuType::new(ir::new_internal_id(), MuType_::int(64))
    );
}

#[link(name="dl")]
extern "C" {
    fn dlopen(filename: *const c_char, flags: isize) -> *const c_void;
    fn dlsym(handle: *const c_void, symbol: *const c_char) -> *const c_void;
}

pub fn resolve_symbol(symbol: String) -> Address {
    use std::ptr;
    
    let rtld_default = unsafe {dlopen(ptr::null(), 0)};
    let ret = unsafe {dlsym(rtld_default, CString::new(symbol.clone()).unwrap().as_ptr())};
    
    if ret == 0 as *const c_void {
        panic!("cannot find symbol {}", symbol);
    }
    
    Address::from_ptr(ret)
}

#[derive(Clone, Debug)]
pub enum ValueLocation {
    Register(RegGroup, MuID),
    Direct(RegGroup, Address),
    Indirect(RegGroup, Address),
    Constant(RegGroup, Word),
    
    Relocatable(RegGroup, MuName)
}

impl ValueLocation {
    pub fn load_value(&self) -> (RegGroup, Word) {
        match self {
            &ValueLocation::Register(_, _)
            | &ValueLocation::Direct(_, _)
            | &ValueLocation::Indirect(_, _) => unimplemented!(),
            
            &ValueLocation::Constant(group, word) => {
                (group, word)
            }
            &ValueLocation::Relocatable(_, _) => panic!("expect a runtime value")
        }
    }
    
    #[allow(unused_variables)]
    pub fn from_constant(c: Constant) -> ValueLocation {
        match c {
            Constant::Int(int_val) => ValueLocation::Constant(RegGroup::GPR, utils::mem::u64_to_raw(int_val)),
            Constant::Float(f32_val) => ValueLocation::Constant(RegGroup::FPR, utils::mem::f32_to_raw(f32_val)),
            Constant::Double(f64_val) => ValueLocation::Constant(RegGroup::FPR, utils::mem::f64_to_raw(f64_val)),
            
            _ => unimplemented!()
        }
    }
}