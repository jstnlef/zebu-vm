#![allow(unused_imports)]
#![allow(dead_code)]
extern crate mu;

extern crate log;
extern crate simple_logger;

use self::mu::ast::types::*;
use self::mu::ast::ir::*;
use self::mu::ast::inst::*;
use self::mu::ast::ptr::*;
use self::mu::ast::op::*;
use self::mu::vm::*;
use self::mu::vm::api::*;

use std::mem;
use std::ptr;
use std::ffi::CString;
use std::os::raw::c_char;

#[test]
#[allow(unused_variables)]
fn test_builder_factorial() {
    builder_factorial()
}

fn builder_factorial() {
//    let mvm = MuVM::new();
//    let mvm_ref = unsafe {mvm.as_mut()}.unwrap();
//    let ctx = (mvm_ref.new_context)(mvm);
//    let ctx_ref = unsafe {ctx.as_mut()}.unwrap();
}

#[test]
#[allow(unused_variables)]
fn test_startup_shutdown() {
    unsafe {
        simple_logger::init_with_level(log::LogLevel::Trace).ok();
        
        info!("Starting micro VM...");

        let mvm = mu_fastimpl_new();

        let ctx = ((*mvm).new_context)(mvm);

        let b = ((*ctx).new_ir_builder)(ctx);

        let id1 = ((*b).gen_sym)(b, ptr::null_mut());
        let id2 = ((*b).gen_sym)(b, CString::new("@id2").unwrap().as_ptr());
        let id3 = ((*b).gen_sym)(b, ptr::null_mut());

        ((*b).abort)(b);
        ((*ctx).close_context)(ctx);

        info!("Finished.");
    }
}

#[derive(Default)]
struct CStringPool {
    strings: Vec<CString>,
}

impl CStringPool {
    fn get(&mut self, s: &str) -> *const c_char {
        self.strings.push(CString::new(s).unwrap());
        self.strings.last().unwrap().as_ptr()
    }
}


#[test]
#[allow(unused_variables)]
fn test_types_sigs_loading() {
    let mut csp: CStringPool = Default::default();

    unsafe {
        simple_logger::init_with_level(log::LogLevel::Trace).ok();
        
        info!("Starting micro VM...");

        let mvm = mu_fastimpl_new();

        let ctx = ((*mvm).new_context)(mvm);

        let b = ((*ctx).new_ir_builder)(ctx);

        let id1 = ((*b).gen_sym)(b, csp.get("@i8"));
        let id2 = ((*b).gen_sym)(b, csp.get("@i32"));
        let id3 = ((*b).gen_sym)(b, csp.get("@pi32"));

        ((*b).new_type_uptr)(b, id3, id2);
        ((*b).new_type_int)(b, id2, 32);
        ((*b).new_type_int)(b, id1, 8);

        ((*b).load)(b);
        ((*ctx).close_context)(ctx);

        info!("Finished.");
    }
}

