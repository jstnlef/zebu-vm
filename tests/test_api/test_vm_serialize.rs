extern crate rustc_serialize;

use test_ir::test_ir::factorial;
use mu::ast::ir::*;
use mu::vm::*;

use std::sync::Arc;

use self::rustc_serialize::json;

#[test]
fn test_vm_serialize_factorial() {
    ::simple_logger::init_with_level(::log::LogLevel::Trace).ok();
    
    let vm = Arc::new(factorial());
    
    let serialized = json::encode(&vm).unwrap();
    println!("{}", serialized);
}