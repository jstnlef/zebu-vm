// Copyright 2017 The Australian National University
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use linkutils::*;
use ast::ir::MuName;
use runtime;
use vm::VM;
use compiler::backend;

use std::path::PathBuf;
use std::process::Command;

/// links generated code for the given functions, static library of Zebu,
/// and a main function to produce an executable of the given name
pub fn link_primordial(funcs: Vec<MuName>, out: &str, vm: &VM) -> PathBuf {
    let emit_dir = PathBuf::from(&vm.vm_options.flag_aot_emit_dir);

    // prepare a list of files that need to be compiled and linked together
    let files: Vec<PathBuf> = {
        use std::fs;

        let mut ret = vec![];

        // all interested mu funcs
        for func in funcs {
            ret.push(get_path_for_mu_func(func, vm));
        }

        // mu context
        ret.push(get_path_for_mu_context(vm));

        // copy primoridal entry
        let source = get_path_under_zebu(runtime::PRIMORDIAL_ENTRY);
        let dest = {
            let mut ret = PathBuf::from(&vm.vm_options.flag_aot_emit_dir);
            ret.push("main.c");

            ret
        };
        trace!("copying from {:?} to {:?}", source, dest);
        match fs::copy(source.as_path(), dest.as_path()) {
            Ok(_) => {}
            Err(e) => panic!("failed to copy: {}", e)
        }

        // include the primordial C main
        ret.push(dest);

        // include mu static lib
        ret.push(get_path_under_zebu(if cfg!(debug_assertions) {
            "target/debug/libmu.a"
        } else {
            "target/release/libmu.a"
        }));

        ret
    };

    let mut out_path = emit_dir.clone();
    out_path.push(out);

    link_executable_internal(
        files,
        &vm.vm_options.flag_bootimage_external_lib,
        &vm.vm_options.flag_bootimage_external_libpath,
        out_path
    )
}

/// invokes the C compiler to link code into an executable
fn link_executable_internal(
    files: Vec<PathBuf>,
    lib: &Vec<String>,
    libpath: &Vec<String>,
    out: PathBuf
) -> PathBuf {
    info!("output as {:?}", out.as_path());

    let mut cc = Command::new(get_c_compiler());

    // external libs
    for path in libpath.iter() {
        cc.arg(format!("-L{}", path));
    }
    for l in lib.iter() {
        cc.arg(format!("-l{}", l));
    }

    // dylibs used for linux
    if cfg!(target_os = "linux") {
        cc.arg("-ldl");
        cc.arg("-lrt");
        cc.arg("-lm");
        cc.arg("-lpthread");
        cc.arg("-lz");
    }

    // all the source code
    for file in files {
        info!("link with {:?}", file.as_path());
        cc.arg(file.as_path());
    }

    // flag to allow find symbols in the executable
    cc.arg("-rdynamic");

    // specified output
    cc.arg("-o");
    cc.arg(out.as_os_str());

    // execute and check results
    assert!(exec_cmd(cc).status.success(), "failed to link code");
    out
}

/// links generated code for the given functions to produce a dynamic
/// library of the given name
pub fn link_dylib(funcs: Vec<MuName>, out: &str, vm: &VM) -> PathBuf {
    link_dylib_with_extra_srcs(funcs, vec![], out, vm)
}

/// links generated code for the given functions with a few external sources
/// to produce a dynamic library of the given name
pub fn link_dylib_with_extra_srcs(
    funcs: Vec<MuName>,
    srcs: Vec<String>,
    out: &str,
    vm: &VM
) -> PathBuf {
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

    link_dylib_internal(
        files,
        &vm.vm_options.flag_bootimage_external_lib,
        &vm.vm_options.flag_bootimage_external_libpath,
        out_path
    )
}

/// invokes the C compiler to link code into a dynamic library
fn link_dylib_internal(
    files: Vec<PathBuf>,
    lib: &Vec<String>,
    libpath: &Vec<String>,
    out: PathBuf
) -> PathBuf {
    let mut object_files: Vec<PathBuf> = vec![];

    // compile each single source file
    for file in files {
        let mut cc = Command::new(get_c_compiler());

        // output object file
        cc.arg("-c");
        // position independent code
        cc.arg("-fPIC");

        let mut out = file.clone();
        out.set_extension("o");

        cc.arg(file.as_os_str());
        cc.arg("-o");
        cc.arg(out.as_os_str());

        object_files.push(out);
        exec_cmd(cc);
    }

    // link object files into a dynamic library
    let mut cc = Command::new(get_c_compiler());

    // external libs
    for path in libpath.iter() {
        cc.arg(format!("-L{}", path));
    }
    for l in lib.iter() {
        cc.arg(format!("-l{}", l));
    }

    // options:
    // shared library
    cc.arg("-shared");
    // position independent code
    cc.arg("-fPIC");
    // allow undefined symbol
    cc.arg("-Wl");
    cc.arg("-undefined");
    // allow dynamic lookup symbols
    cc.arg("dynamic_lookup");

    // all object files
    for obj in object_files {
        cc.arg(obj.as_os_str());
    }

    // output
    cc.arg("-o");
    cc.arg(out.as_os_str());

    exec_cmd(cc);

    out
}

/// builds a bundle (that contains a function), compiles it,
/// links and loads it as a dynamic library
/// This function is used to test compiler.
//  TODO: should think about using make_boot_image() instead of this adhoc code (Issue #52)
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
            None => {
                panic!(
                    "function {} does not have a defined current version",
                    fnc_name
                )
            }
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

/// builds a bundle (that contains several functions), compiles them,
/// links and loads it as a dynamic library
/// This function is used to test compiler.
//  TODO: should think about using make_boot_image() instead of this adhoc code (Issue #52)
pub fn compile_fncs<'a>(
    entry: &'static str,
    fnc_names: Vec<&'static str>,
    build_fnc: &'a Fn() -> VM
) -> ll::Library {
    VM::start_logging_trace();

    let vm = Arc::new(build_fnc());
    let compiler = Compiler::new(CompilerPolicy::default(), &vm);

    for func in fnc_names.iter() {
        let func_id = vm.id_of(func);
        let funcs = vm.funcs().read().unwrap();
        let func = funcs.get(&func_id).unwrap().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();
        let mut func_ver = func_vers
            .get(&func.cur_ver.unwrap())
            .unwrap()
            .write()
            .unwrap();
        compiler.compile(&mut func_ver);
    }

    backend::emit_context(&vm);

    let libname = &get_dylib_name(entry);
    let dylib = aot::link_dylib(fnc_names.iter().map(|x| Mu(x)).collect(), libname, &vm);
    ll::Library::new(dylib.as_os_str()).unwrap()
}

/// gets the path for the generated code of a Mu function
fn get_path_for_mu_func(f: MuName, vm: &VM) -> PathBuf {
    let mut ret = PathBuf::from(&vm.vm_options.flag_aot_emit_dir);
    ret.push(f + ".S");

    ret
}

/// gets the path for generated Mu context (persisted VM/heap)
fn get_path_for_mu_context(vm: &VM) -> PathBuf {
    let mut ret = PathBuf::from(&vm.vm_options.flag_aot_emit_dir);
    ret.push(backend::AOT_EMIT_CONTEXT_FILE);
    ret
}
