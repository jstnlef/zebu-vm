extern crate log;
extern crate simple_logger;
extern crate libloading as ll;

use compiler::*;
use ast::ir::*;
use vm::*;
use std::sync::Arc;

pub mod aot {
    use ast::ir::MuName;
    use runtime;
    use compiler::backend;
    use std::path::PathBuf;
    use std::process::Command;
    use std::process::Output;

    const CC : &'static str = "clang";

    fn exec (mut cmd: Command) -> Output {
        println!("executing: {:?}", cmd);
        let output = cmd.output().expect("failed to execute");

        println!("---out---");
        println!("{}", String::from_utf8_lossy(&output.stdout));
        println!("---err---");
        println!("{}", String::from_utf8_lossy(&output.stderr));

        output
    }

    fn link_executable_internal (files: Vec<PathBuf>, out: PathBuf) -> PathBuf {
        let mut gcc = Command::new(CC);

        for file in files {
            println!("link with {:?}", file.as_path());
            gcc.arg(file.as_path());
        }

        println!("output as {:?}", out.as_path());
        gcc.arg("-o");
        gcc.arg(out.as_os_str());

        assert!(exec(gcc).status.success());

        out
    }

    fn link_dylib_internal (files: Vec<PathBuf>, out: PathBuf) -> PathBuf {
        let mut object_files : Vec<PathBuf> = vec![];

        for file in files {
            let mut gcc = Command::new(CC);

            gcc.arg("-c");
            gcc.arg("-fpic");

            let mut out = file.clone();
            out.set_extension("o");

            gcc.arg(file.as_os_str());
            gcc.arg("-o");
            gcc.arg(out.as_os_str());

            object_files.push(out);
            exec(gcc);
        }

        let mut gcc = Command::new(CC);
        gcc.arg("-shared");
        gcc.arg("-Wl");
        gcc.arg("-undefined");
        gcc.arg("dynamic_lookup");
        for obj in object_files {
            gcc.arg(obj.as_os_str());
        }
        gcc.arg("-o");
        gcc.arg(out.as_os_str());

        exec(gcc);

        out
    }

    fn get_path_for_mu_func (f: MuName) -> PathBuf {
        let mut ret = PathBuf::from(backend::AOT_EMIT_DIR);
        ret.push(f);
        ret.set_extension("s");

        ret
    }

    fn get_path_for_mu_context () -> PathBuf {
        let mut ret = PathBuf::from(backend::AOT_EMIT_DIR);
        ret.push(backend::AOT_EMIT_CONTEXT_FILE);
        ret
    }

    pub fn link_primordial (funcs: Vec<MuName>, out: &str) -> PathBuf {
        let emit_dir = PathBuf::from(backend::AOT_EMIT_DIR);

        let files : Vec<PathBuf> = {
            use std::fs;

            let mut ret = vec![];

            // all interested mu funcs
            for func in funcs {
                ret.push(get_path_for_mu_func(func));
            }

            // mu context
            ret.push(get_path_for_mu_context());

            // copy primoridal entry
            let source   = PathBuf::from(runtime::PRIMORDIAL_ENTRY);
            let mut dest = PathBuf::from(backend::AOT_EMIT_DIR);
            dest.push("main.c");
            fs::copy(source.as_path(), dest.as_path()).unwrap();
            // include the primordial C main
            ret.push(dest);

            // include mu static lib
            let libmu = PathBuf::from("target/debug/libmu.a");
            ret.push(libmu);

            ret
        };

        let mut out_path = emit_dir.clone();
        out_path.push(out);

        link_executable_internal(files, out_path)
    }

    pub fn execute(executable: PathBuf) {
        let run = Command::new(executable.as_os_str());
        assert!(exec(run).status.success());
    }

    pub fn link_dylib (funcs: Vec<MuName>, out: &str) -> PathBuf {
        let files = {
            let mut ret = vec![];

            for func in funcs {
                ret.push(get_path_for_mu_func(func));
            }

            ret.push(get_path_for_mu_context());

            ret
        };

        let mut out_path = PathBuf::from(backend::AOT_EMIT_DIR);
        out_path.push(out);

        link_dylib_internal(files, out_path)
    }
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
