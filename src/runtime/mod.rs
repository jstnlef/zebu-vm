pub mod mm;
pub mod thread;
pub mod entrypoints;

pub use runtime::mm::common::Address;
pub use runtime::mm::common::ObjectReference;

use log;
use simple_logger;
use utils;
use ast::ir;
use ast::ptr::*;
use ast::types::MuType_;
use ast::types::MuType;
use ast::ir::*;
use vm::VM;
use compiler::backend::Word;
use compiler::backend::RegGroup;

use std::os::raw::c_char;
use std::os::raw::c_void;
use std::ffi::CString;
use std::ffi::CStr;

use std::sync::Arc;

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

#[no_mangle]
pub extern fn mu_trace_level_log() {
    simple_logger::init_with_level(log::LogLevel::Trace).ok();
}

#[no_mangle]
pub extern fn mu_main(serialized_vm : *const c_char) {      
    debug!("mu_main() started...");
    
    let str_vm = unsafe{CStr::from_ptr(serialized_vm)}.to_str().unwrap();
    
    let vm : Arc<VM> = Arc::new(VM::resume_vm(str_vm));
    
    let primordial = vm.primordial.read().unwrap();
    if primordial.is_none() {
        panic!("no primordial thread/stack/function. Client should provide an entry point");
    } else {
        let primordial = primordial.as_ref().unwrap();
        
        // create mu stack
        let stack = vm.new_stack(primordial.func_id);
        
        let args : Vec<ValueLocation> = primordial.args.iter().map(|arg| ValueLocation::from_constant(arg.clone())).collect();
        
        // FIXME: currently assumes no user defined thread local
        // will need to fix this after we can serialize heap object
        let thread = vm.new_thread_normal(stack, unsafe{Address::zero()}, args);
        
        thread.join().unwrap();
    }
}