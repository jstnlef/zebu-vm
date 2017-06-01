extern crate libloading as ll;

use compiler::*;
use ast::ir::*;
use vm::*;
use std::sync::Arc;

use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::os::unix::process::ExitStatusExt;

pub mod aot;

pub fn get_test_clang_path() -> String {
    use std::env;

    match env::var("CC") {
        Ok(val) => val,
        Err(_) => "clang".to_string()
    }
}

pub fn exec (cmd: Command) -> Output {
    let output = exec_nocheck(cmd);

    assert!(output.status.success());

    output
}

pub fn exec_nocheck (mut cmd: Command) -> Output {
    info!("executing: {:?}", cmd);
    let output = match cmd.output() {
        Ok(res) => res,
        Err(e) => panic!("failed to execute: {}", e)
    };

    info!("---out---");
    info!("{}", String::from_utf8_lossy(&output.stdout));
    info!("---err---");
    info!("{}", String::from_utf8_lossy(&output.stderr));

    if output.status.signal().is_some() {
        info!("terminated by a signal: {}", output.status.signal().unwrap());
    }

    output
}

pub fn get_path_under_mu(str: &'static str) -> PathBuf {
    use std::env;

    match env::var("MU_ZEBU") {
        Ok(v) => {
            let mut ret = PathBuf::from(v);
            ret.push(str);

            ret
        }
        Err(_) => PathBuf::from(str)
    }
}

#[cfg(target_os = "macos")]
pub fn get_dylib_name(name: &'static str) -> String {
    format!("lib{}.dylib", name)
}

#[cfg(target_os = "linux")]
pub fn get_dylib_name(name: &'static str) -> String {
    format!("lib{}.so", name)
}

pub fn compile_fnc<'a>(fnc_name: &'static str, build_fnc: &'a Fn() -> VM) -> ll::Library {
    VM::start_logging_trace();

    let vm = Arc::new(build_fnc());
    let compiler = Compiler::new(CompilerPolicy::default(), &vm);
    let func_id = vm.id_of(fnc_name);
    {
        let funcs = vm.funcs().read().unwrap();
        let func = match funcs.get(&func_id) {
            Some(func) => func.read().unwrap(),
            None => panic!("cannot find function {}", fnc_name)
        };

        let cur_ver = match func.cur_ver {
            Some(v) => v,
            None => panic!("function {} does not have a defined current version", fnc_name)
        };

        let func_vers = vm.func_vers().read().unwrap();
        let mut func_ver = match func_vers.get(&cur_ver) {
            Some(fv) => fv.write().unwrap(),
            None => panic!("cannot find function version {}", cur_ver)
        };
        compiler.compile(&mut func_ver);
    }
    backend::emit_context(&vm);
    let libname = &get_dylib_name(fnc_name);
    let dylib = aot::link_dylib(vec![Mu(fnc_name)], libname, &vm);
    ll::Library::new(dylib.as_os_str()).unwrap()
}

pub fn compile_fncs<'a>(entry: &'static str, fnc_names: Vec<&'static str>, build_fnc: &'a Fn() -> VM) -> ll::Library {
    VM::start_logging_trace();

    let vm = Arc::new(build_fnc());
    let compiler = Compiler::new(CompilerPolicy::default(), &vm);

    for func in fnc_names.iter() {
        let func_id = vm.id_of(func);
        let funcs = vm.funcs().read().unwrap();
        let func = funcs.get(&func_id).unwrap().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();
        let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();
        compiler.compile(&mut func_ver);
    }

    backend::emit_context(&vm);

    let libname = &get_dylib_name(entry);
    let dylib = aot::link_dylib(fnc_names.iter().map(|x| Mu(x)).collect(), libname, &vm);
    ll::Library::new(dylib.as_os_str()).unwrap()
}
