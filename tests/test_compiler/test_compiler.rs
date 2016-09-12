extern crate mu;
extern crate log;
extern crate simple_logger;
extern crate libloading;

use test_ir::test_ir::sum;
use test_ir::test_ir::factorial;
use self::mu::compiler::*;
use self::mu::ast::ir::*;

use std::sync::Arc;
use aot;

#[test]
fn test_regalloc_fac() {
    simple_logger::init_with_level(log::LogLevel::Trace).ok();
    
    let vm = Arc::new(factorial());
    
    let compiler = Compiler::new(CompilerPolicy::default(), vm.clone());
    
    let func_id = vm.id_of("fac");
    
    {
        let funcs = vm.funcs().read().unwrap();
        let func = funcs.get(&func_id).unwrap().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();
        let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();
        
        compiler.compile(&mut func_ver);
    }
    
    backend::emit_context(&vm);
    
    let dylib = aot::link_dylib(vec![Mu("fac")], "libfac.dylib");
    
    let lib = libloading::Library::new(dylib.as_os_str()).unwrap();
    unsafe {
        let fac : libloading::Symbol<unsafe extern fn(u64) -> u64> = lib.get(b"fac").unwrap();
        
        let fac5 = fac(5);
        println!("fac(5) = {}", fac5);
        assert!(fac5 == 120);
    }
}

#[test]
fn test_regalloc_sum() {
    simple_logger::init_with_level(log::LogLevel::Trace).ok();
    
    let vm = Arc::new(sum());
    
    let compiler = Compiler::new(CompilerPolicy::default(), vm.clone());
    
    let func_id = vm.id_of("sum");
    
    {
        let funcs = vm.funcs().read().unwrap();
        let func = funcs.get(&func_id).unwrap().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();
        let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();
        
        compiler.compile(&mut func_ver);
    }
    
    backend::emit_context(&vm);
    
    let dylib = aot::link_dylib(vec![Mu("sum")], "libsum.dylib");
    
    let lib = libloading::Library::new(dylib.as_os_str()).unwrap();
    unsafe {
        let sum : libloading::Symbol<unsafe extern fn(u64) -> u64> = lib.get(b"sum").unwrap();
        
        let sum5 = sum(5);
        println!("sum(1..4) = {}", sum5);
        assert!(sum5 == 10);        
        
        let sum10 = sum(10);
        println!("sum(0..9) = {}", sum10);
        assert!(sum10 == 45);
    }    
}