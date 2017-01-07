use testutil::*;
use ast::ir::MuName;
use runtime;
use vm::VM;
use compiler::backend;

use std::path::PathBuf;
use std::process::Command;
use std::process::Output;

fn link_executable_internal (files: Vec<PathBuf>, out: PathBuf) -> PathBuf {
    let mut gcc = Command::new(get_test_clang_path());

    for file in files {
        println!("link with {:?}", file.as_path());
        gcc.arg(file.as_path());
    }

    println!("output as {:?}", out.as_path());
    if cfg!(target_os = "linux") {
        gcc.arg("-ldl");
        gcc.arg("-lrt");
        gcc.arg("-lm");
        gcc.arg("-lpthread");
    }
    // so we can find symbols in itself
    gcc.arg("-rdynamic");
    gcc.arg("-o");
    gcc.arg(out.as_os_str());

    assert!(exec(gcc).status.success());

    out
}

fn link_dylib_internal (files: Vec<PathBuf>, out: PathBuf) -> PathBuf {
    let mut object_files : Vec<PathBuf> = vec![];

    for file in files {
        let mut gcc = Command::new(get_test_clang_path());

        gcc.arg("-c");
        gcc.arg("-fPIC");

        let mut out = file.clone();
        out.set_extension("o");

        gcc.arg(file.as_os_str());
        gcc.arg("-o");
        gcc.arg(out.as_os_str());

        object_files.push(out);
        exec(gcc);
    }

    let mut gcc = Command::new(get_test_clang_path());
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

fn get_path_for_mu_func (f: MuName, vm: &VM) -> PathBuf {
    let mut ret = PathBuf::from(&vm.vm_options.flag_aot_emit_dir);
    ret.push(f);
    ret.set_extension("s");

    ret
}

fn get_path_for_mu_context (vm: &VM) -> PathBuf {
    let mut ret = PathBuf::from(&vm.vm_options.flag_aot_emit_dir);
    ret.push(backend::AOT_EMIT_CONTEXT_FILE);
    ret
}

pub fn link_primordial (funcs: Vec<MuName>, out: &str, vm: &VM) -> PathBuf {
    let emit_dir = PathBuf::from(&vm.vm_options.flag_aot_emit_dir);

    let files : Vec<PathBuf> = {
        use std::fs;

        let mut ret = vec![];

        // all interested mu funcs
        for func in funcs {
            ret.push(get_path_for_mu_func(func, vm));
        }

        // mu context
        ret.push(get_path_for_mu_context(vm));

        // copy primoridal entry
        let source   = PathBuf::from(runtime::PRIMORDIAL_ENTRY);
        let mut dest = PathBuf::from(&vm.vm_options.flag_aot_emit_dir);
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

pub fn execute(executable: PathBuf) -> Output {
    let run = Command::new(executable.as_os_str());
    exec(run)
}

pub fn execute_nocheck(executable: PathBuf) -> Output {
    let run = Command::new(executable.as_os_str());
    exec_nocheck(run)
}

pub fn link_dylib (funcs: Vec<MuName>, out: &str, vm: &VM) -> PathBuf {
    let files = {
        let mut ret = vec![];

        for func in funcs {
            ret.push(get_path_for_mu_func(func, vm));
        }

        ret.push(get_path_for_mu_context(vm));

        ret
    };

    let mut out_path = PathBuf::from(&vm.vm_options.flag_aot_emit_dir);
    out_path.push(out);

    link_dylib_internal(files, out_path)
}

pub fn link_dylib_with_extra_srcs(funcs: Vec<MuName>, srcs: Vec<String>, out: &str, vm: &VM) -> PathBuf{
    let files = {
        let mut ret = vec![];

        for func in funcs {
            ret.push(get_path_for_mu_func(func, vm));
        }

        for src in srcs {
            ret.push(PathBuf::from(src));
        }

        ret.push(get_path_for_mu_context(vm));

        ret
    };

    let mut out_path = PathBuf::from(&vm.vm_options.flag_aot_emit_dir);
    out_path.push(out);

    link_dylib_internal(files, out_path)
}
