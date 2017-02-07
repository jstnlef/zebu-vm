use testutil::*;
use ast::ir::MuName;
use runtime;
use vm::VM;
use compiler::backend;

use std::path::PathBuf;
use std::process::Command;
use std::process::Output;

fn link_executable_internal (files: Vec<PathBuf>, lib: &Vec<String>, libpath: &Vec<String>, out: PathBuf) -> PathBuf {
    let mut cc = Command::new(get_test_clang_path());

    for file in files {
        trace!("link with {:?}", file.as_path());
        cc.arg(file.as_path());
    }

    // external libs
    for path in libpath.iter() {
        cc.arg(format!("-L{}", path));
    }
    for l in lib.iter() {
        cc.arg(format!("-l{}", l));
    }

    println!("output as {:?}", out.as_path());
    if cfg!(target_os = "linux") {
        cc.arg("-ldl");
        cc.arg("-lrt");
        cc.arg("-lm");
        cc.arg("-lpthread");
    }

    // so we can find symbols in itself
    cc.arg("-rdynamic");
    cc.arg("-o");
    cc.arg(out.as_os_str());

    assert!(exec(cc).status.success());

    out
}

fn link_dylib_internal (files: Vec<PathBuf>, lib: &Vec<String>, libpath: &Vec<String>, out: PathBuf) -> PathBuf {
    let mut object_files : Vec<PathBuf> = vec![];

    for file in files {
        let mut cc = Command::new(get_test_clang_path());

        cc.arg("-c");
        cc.arg("-fPIC");

        let mut out = file.clone();
        out.set_extension("o");

        cc.arg(file.as_os_str());
        cc.arg("-o");
        cc.arg(out.as_os_str());

        object_files.push(out);
        exec(cc);
    }

    let mut cc = Command::new(get_test_clang_path());

    // external libs
    for path in libpath.iter() {
        cc.arg(format!("-L{}", path));
    }
    for l in lib.iter() {
        cc.arg(format!("-l{}", l));
    }

    cc.arg("-shared");
    cc.arg("-fPIC");
    cc.arg("-Wl");
    cc.arg("-undefined");
    cc.arg("dynamic_lookup");
    for obj in object_files {
        cc.arg(obj.as_os_str());
    }

    cc.arg("-o");
    cc.arg(out.as_os_str());

    exec(cc);

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
        let source   = get_path_under_mu(runtime::PRIMORDIAL_ENTRY);
        let dest = {
            let mut ret = PathBuf::from(&vm.vm_options.flag_aot_emit_dir);
            ret.push("main.c");

            ret
        };

        trace!("copying from {:?} to {:?}", source, dest);
        fs::copy(source.as_path(), dest.as_path()).unwrap();

        // include the primordial C main
        ret.push(dest);

        // include mu static lib
        let libmu_path = if cfg!(debug_assertions) {
            "target/debug/libmu.a"
        } else {
            "target/release/libmu.a"
        };
        let libmu = get_path_under_mu(libmu_path);
        ret.push(libmu);

        ret
    };

    let mut out_path = emit_dir.clone();
    out_path.push(out);

    link_executable_internal(files,
                             &vm.vm_options.flag_bootimage_external_lib,
                             &vm.vm_options.flag_bootimage_external_libpath,
                             out_path)
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

    link_dylib_internal(files,
                        &vm.vm_options.flag_bootimage_external_lib,
                        &vm.vm_options.flag_bootimage_external_libpath,
                        out_path)
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

    link_dylib_internal(files,
                        &vm.vm_options.flag_bootimage_external_lib,
                        &vm.vm_options.flag_bootimage_external_libpath,
                        out_path)
}
