extern crate log;
extern crate simple_logger;
extern crate libloading as ll;

use compiler::*;
use ast::ir::*;
use vm::*;
use std::sync::Arc;

use std::process::Command;
use std::process::Output;

pub mod aot;
pub mod c_api;

pub fn get_test_clang_path() -> String {
    use std::env;

    match env::var("CC") {
        Ok(val) => val,
        Err(_) => "clang".to_string()
    }
}

pub fn exec (mut cmd: Command) -> Output {
    println!("executing: {:?}", cmd);
    let output = match cmd.output() {
        Ok(res) => res,
        Err(e) => panic!("failed to execute: {}", e)
    };

    println!("---out---");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    println!("---err---");
    println!("{}", String::from_utf8_lossy(&output.stderr));

    assert!(output.status.success());

    output
}

pub fn compile_fnc<'a>(fnc_name: &'static str, build_fnc: &'a Fn() -> VM) -> ll::Library {
    simple_logger::init_with_level(log::LogLevel::Trace).ok();
    let vm = Arc::new(build_fnc());
    let compiler = Compiler::new(CompilerPolicy::default(), vm.clone());
    let func_id = vm.id_of(fnc_name);
    {
        let funcs = vm.funcs().read().unwrap();
        let func = funcs.get(&func_id).unwrap().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();
        let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();
        compiler.compile(&mut func_ver);
    }
    backend::emit_context(&vm);
    let libname = &format!("lib{}.dylib", fnc_name);
    let dylib = aot::link_dylib(vec![Mu(fnc_name)], libname);
    ll::Library::new(dylib.as_os_str()).unwrap()
}
