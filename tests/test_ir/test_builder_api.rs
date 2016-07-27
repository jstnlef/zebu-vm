#![allow(unused_imports)]
#![allow(dead_code)]
extern crate mu;

use self::mu::ast::types::*;
use self::mu::ast::ir::*;
use self::mu::ast::inst::*;
use self::mu::ast::ptr::*;
use self::mu::ast::op::*;
use self::mu::vm::*;
use self::mu::vm::api::*;

use std::mem;

#[test]
#[allow(unused_variables)]
fn test_builder_factorial() {
    builder_factorial()
}

fn builder_factorial() {
    let mvm = MuVM::new();
    let mvm_ref = unsafe {mvm.as_mut()}.unwrap();
    let ctx = (mvm_ref.new_context)(mvm);
    let ctx_ref = unsafe {ctx.as_mut()}.unwrap();
}
