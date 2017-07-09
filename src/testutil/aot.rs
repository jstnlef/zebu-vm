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

    // external libs
    for path in libpath.iter() {
        cc.arg(format!("-L{}", path));
    }
    for l in lib.iter() {
        cc.arg(format!("-l{}", l));
    }

    info!("output as {:?}", out.as_path());
    if cfg!(target_os = "linux") {
        cc.arg("-ldl");
        cc.arg("-lrt");
        cc.arg("-lm");
        cc.arg("-lpthread");
    }

    for file in files {
        info!("link with {:?}", file.as_path());
        cc.arg(file.as_path());
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

    // options
    cc.arg("-shared");
    cc.arg("-fPIC");
    cc.arg("-Wl");
    cc.arg("-undefined");
    cc.arg("dynamic_lookup");

    // all object files
    for obj in object_files {
        cc.arg(obj.as_os_str());
    }

    // output
    cc.arg("-o");
    cc.arg(out.as_os_str());

    exec(cc);

    out
}

fn get_path_for_mu_func (f: MuName, vm: &VM) -> PathBuf {
    let mut ret = PathBuf::from(&vm.vm_options.flag_aot_emit_dir);
    ret.push(f + ".S");

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
        match fs::copy(source.as_path(), dest.as_path()) {
            Ok(_)  => {},
            Err(e) => panic!("failed to copy: {}", e)
        }

        // include the primordial C main
        ret.push(dest);

        // include mu static lib
        ret.push(get_path_under_mu(if cfg!(debug_assertions) {
            "target/debug/libmu.a"
        } else {
            "target/release/libmu.a"
        }));
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
