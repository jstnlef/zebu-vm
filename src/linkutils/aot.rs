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

extern crate time;

use linkutils::*;
use ast::ir::MuName;
use runtime;
use vm::VM;
use compiler::backend;

use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::process::Output;

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

/// links generated code for the given test's functions, static library of Zebu,
/// and a main function to produce an executable of the given name
pub fn link_test_primordial(funcs: Vec<MuName>, out: &str, vm: &VM) -> PathBuf {
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
            ret.push("main_test.c");

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
    } else if cfg!(target_os = "macos") {
        cc.arg("-liconv");
        cc.arg("-framework");
        cc.arg("Security");
        cc.arg("-framework");
        cc.arg("CoreFoundation");
        cc.arg("-lz");
        cc.arg("-lSystem");
        cc.arg("-lresolv");
        cc.arg("-lc");
        cc.arg("-lm");
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

#[cfg(not(feature = "sel4-rumprun"))]
pub fn run_test(vm: &VM, test_name: &str, tester_name: &str) {
    let output_name = test_name.to_string() + "_" + tester_name;
    let executable = link_test_primordial(
        vec![test_name.to_string(), tester_name.to_string()],
        output_name.as_str(),
        vm
    );
    self::super::exec_path(executable);
}

#[cfg(feature = "sel4-rumprun")]
pub fn run_test(vm: &VM, test_name: &str, tester_name: &str) {

    use std::fs::File;

    //  emit/add.s
    let test_asm_file = "emit/".to_string() + test_name + ".S";
    //  emit/add_test1.s
    let tester_asm_file = "emit/".to_string() + tester_name + ".S";
    //  emit/context.s
    let context_asm_file = "emit/".to_string() + "context.S";
    //  emit/mu_sym_table.s
    let mu_sym_table_asm_file = "emit/".to_string() + "mu_sym_table.S";

    // clean the destination first
    let destination_prefix = "../rumprun-sel4/apps/zebu_rt/src/emit/";
    let output = Command::new("rm")
        .arg("-R")
        .arg(destination_prefix)
        .output()
        .expect("failed to RM dest emit");

    assert!(output.status.success());

    //  recreate the emit folder, deleted by the previous command
    let output = Command::new("mkdir")
        .arg(destination_prefix)
        .output()
        .expect("failed to RM dest emit");

    assert!(output.status.success());


    // above file will be pasted in \
    //  rumprun-sel4/apps/zebu_rt/src + the above Strings
    let destination_prefix = "../rumprun-sel4/apps/zebu_rt/src/";

    let dest_test_asm_file = destination_prefix.to_string() + &test_asm_file;
    let dest_tester_asm_file = destination_prefix.to_string() + &tester_asm_file;
    let dest_context_asm_file = destination_prefix.to_string() + &context_asm_file;
    let dest_mu_sym_table_asm_file = destination_prefix.to_string() + &mu_sym_table_asm_file;

    /*
        The following 4 commands, copy 4 asm source files to \
        the proper location of filesystem for sel4-rumprun runtime
        This is currently src/emit
    */

    let output = Command::new("cp")
        .arg(&test_asm_file)
        .arg(&dest_test_asm_file)
        .output()
        .expect("failed to copy test_asm_file");

    assert!(output.status.success());

    let output = Command::new("cp")
        .arg(&tester_asm_file)
        .arg(&dest_tester_asm_file)
        .output()
        .expect("failed to copy tester_asm_file");

    assert!(output.status.success());

    let output = Command::new("cp")
        .arg(&context_asm_file)
        .arg(&dest_context_asm_file)
        .output()
        .expect("failed to copy context_asm_file");

    assert!(output.status.success());

    let output = Command::new("cp")
        .arg(&mu_sym_table_asm_file)
        .arg(&dest_mu_sym_table_asm_file)
        .output()
        .expect("failed to copy dest_mu_sym_table_asm_file");

    assert!(output.status.success());


    /*
        Everything is ready for our sel4-rumprun Zebu runtime
        to start building the final test executable(s)
    */

    use std::os::unix::io::FromRawFd;
    use std::os::unix::io::AsRawFd;


    let output = Command::new("rm")
        .arg("outputs.txt")
        .output()
        .expect("failed to change directory2");

    let mut outputs_file = File::create("outputs.txt").unwrap();

    let rawfd = outputs_file.as_raw_fd();
    //    let rawfd = unsafe { File::from_raw_fd(rawfd) };
    let rawfd = unsafe { Stdio::from_raw_fd(rawfd) };

    let output = Command::new("bash")
        .arg("build_for_sel4_rumprun.sh")
        .stdout(Stdio::inherit())
        .output()
        .expect("failed to Build");

    println!("****************************************");
    println!("Build Output -{:?}-", output);
    println!("****************************************");

    assert!(output.status.success());

    // First create a child process which runs qemu for testing
    // Then, create a watchdog to check if test is finished

    let mut tester_proc = Command::new("qemu-system-x86_64")
        .arg("-nographic")
        .arg("-m")
        .arg("512")
        .arg("-kernel")
        .arg("../rumprun-sel4/images/kernel-x86_64-pc99")
        .arg("-initrd")
        .arg("../rumprun-sel4/images/roottask-image-x86_64-pc99")
        .arg("-cpu")
        .arg("Haswell")
        .stdout(rawfd)
        .spawn()
        .expect("failed to RUN");

    use std::thread;
    use std::io;
    use std::io::prelude::*;

    let mut child_proc_finished = 0;
    let mut test_succeeded = 0;
    let mut test_length = 0;
    let test_length_max = 60; // Maximum allowed length for a test is currently 60 seconds

    // This loop checks the output file to recognize when qemu vm \
    // which is running the test, should be terminated
    while child_proc_finished == 0 {
        thread::sleep_ms(5000);
        test_length += 5;
        {
            let mut results_file = File::open("outputs.txt");
            let mut results_file = match results_file {
                Ok(the_file) => the_file,
                Err(error) => {
                    panic!("Checking outputs file failed with error -{}-", error);
                }
            };
            let mut file_content = String::new();

            results_file.read_to_string(&mut file_content);

            if file_content.contains("bmk_platform_halt@kernel.c:95 All is well in the universe.") {
                child_proc_finished = 1;
                if file_content.contains("@#$%PASSED%$#@") {
                    test_succeeded = 1;
                } else if file_content.contains("@#$%FAILED%$#@") {
                    test_succeeded = 0;
                } else {
                    panic!("Invalid test outcome!");
                }
            } else {
                continue;
            }

            use std::str::FromStr;
            use std::fs::OpenOptions;

            let mut lines = file_content.lines();
            let mut search_finished = 0;
            let mut test_name = String::new();
            while search_finished == 0 {
                let mut current_line = lines.next();
                if current_line == None {
                    panic!("Test name not found in outputs.txt");
                }
                let current_line = current_line.unwrap();
                println!("{}", current_line);
                if current_line.contains("**TEST**") {
                    search_finished = 1;
                    test_name = String::from_str(lines.next().unwrap()).unwrap();
                }
            }

            //            let mut log_file = File::create("results_log.txt");
            let mut log_file = OpenOptions::new()
                .write(true)
                .append(true)
                .open("results_log.txt");
            let mut log_file = match log_file {
                Ok(the_file) => the_file,
                Err(error) => {
                    panic!("Creating-Opening log file failed with error -{}-", error);
                }
            };

            log_file
                .write_fmt(format_args!("******************************\n"))
                .unwrap();
            log_file
                .write_fmt(format_args!("Test time : {}\n", time::now_utc().ctime()))
                .unwrap();
            log_file
                .write_fmt(format_args!("Test name : {}\n", test_name))
                .unwrap();
            if test_succeeded == 1 {
                log_file
                    .write_fmt(format_args!("Test result : PASSED\n"))
                    .unwrap();
            } else {
                log_file
                    .write_fmt(format_args!("Test result : FAILED\n"))
                    .unwrap();
            }
            log_file
                .write_fmt(format_args!("******************************"))
                .unwrap();
        }
        println!("+ 5 secs");
        if test_length == test_length_max {
            let output = Command::new("kill")
                .arg("-15")
                .arg("--")
                .arg(tester_proc.id().to_string())
                .output()
                .expect("failed to kill TO");

            assert!(output.status.success());

            panic!("Test Timed Out!");
        }
    }

    // Terminate the test proc
    let output = Command::new("kill")
        .arg("-15")
        .arg("--")
        .arg(tester_proc.id().to_string())
        .output()
        .expect("failed to kill");

    assert!(output.status.success());

    assert!(test_succeeded == 1);
}

#[cfg(not(feature = "sel4-rumprun"))]
pub fn run_test_2f(vm: &VM, test_name: &str, dep_name: &str, tester_name: &str) {
    let output_name = test_name.to_string() + "_" + tester_name;
    let executable = link_test_primordial(
        vec![
            test_name.to_string(),
            dep_name.to_string(),
            tester_name.to_string(),
        ],
        output_name.as_str(),
        vm
    );
    self::super::exec_path(executable);
}

#[cfg(feature = "sel4-rumprun")]
pub fn run_test_2f(vm: &VM, test_name: &str, dep_name: &str, tester_name: &str) {

    use std::fs::File;

    //  emit/add.s
    let test_asm_file = "emit/".to_string() + test_name + ".S";
    //  emit/add_test1.s
    let tester_asm_file = "emit/".to_string() + tester_name + ".S";
    //  emit/context.s
    let context_asm_file = "emit/".to_string() + "context.S";
    //  emit/mu_sym_table.s
    let mu_sym_table_asm_file = "emit/".to_string() + "mu_sym_table.S";
    //  something like emit/dummy_call.s
    let dep_asm_file = "emit/".to_string() + dep_name + ".S";

    // clean the destination first
    let destination_prefix = "../rumprun-sel4/apps/zebu_rt/src/emit/";
    let output = Command::new("rm")
        .arg("-R")
        .arg(destination_prefix)
        .output()
        .expect("failed to RM dest emit");

    assert!(output.status.success());

    //  recreate the emit folder, deleted by the previous command
    let output = Command::new("mkdir")
        .arg(destination_prefix)
        .output()
        .expect("failed to RM dest emit");

    assert!(output.status.success());


    // above file will be pasted in \
    //  rumprun-sel4/apps/zebu_rt/src + the above Strings
    let destination_prefix = "../rumprun-sel4/apps/zebu_rt/src/";

    let dest_test_asm_file = destination_prefix.to_string() + &test_asm_file;
    let dest_tester_asm_file = destination_prefix.to_string() + &tester_asm_file;
    let dest_context_asm_file = destination_prefix.to_string() + &context_asm_file;
    let dest_mu_sym_table_asm_file = destination_prefix.to_string() + &mu_sym_table_asm_file;
    let dest_dep_asm_file = destination_prefix.to_string() + &dep_asm_file;

    /*
        The following 4 commands, copy 4 asm source files to \
        the proper location of filesystem for sel4-rumprun runtime
        This is currently src/emit
    */

    let output = Command::new("cp")
        .arg(&test_asm_file)
        .arg(&dest_test_asm_file)
        .output()
        .expect("failed to copy test_asm_file");

    assert!(output.status.success());

    let output = Command::new("cp")
        .arg(&tester_asm_file)
        .arg(&dest_tester_asm_file)
        .output()
        .expect("failed to copy tester_asm_file");

    assert!(output.status.success());

    let output = Command::new("cp")
        .arg(&context_asm_file)
        .arg(&dest_context_asm_file)
        .output()
        .expect("failed to copy context_asm_file");

    assert!(output.status.success());

    let output = Command::new("cp")
        .arg(&mu_sym_table_asm_file)
        .arg(&dest_mu_sym_table_asm_file)
        .output()
        .expect("failed to copy dest_mu_sym_table_asm_file");

    assert!(output.status.success());

    let output = Command::new("cp")
        .arg(&dep_asm_file)
        .arg(&dest_dep_asm_file)
        .output()
        .expect("failed to copy dep_asm_file");

    assert!(output.status.success());

    /*
        Everything is ready for our sel4-rumprun Zebu runtime
        to start building the final test executable(s)
    */

    use std::os::unix::io::FromRawFd;
    use std::os::unix::io::AsRawFd;


    let output = Command::new("rm")
        .arg("outputs.txt")
        .output()
        .expect("failed to change directory2");

    let mut outputs_file = File::create("outputs.txt").unwrap();

    let rawfd = outputs_file.as_raw_fd();
    //    let rawfd = unsafe { File::from_raw_fd(rawfd) };
    let rawfd = unsafe { Stdio::from_raw_fd(rawfd) };

    let output = Command::new("bash")
        .arg("build_for_sel4_rumprun.sh")
        .stdout(Stdio::inherit())
        .output()
        .expect("failed to Build");

    println!("****************************************");
    println!("Build Output -{:?}-", output);
    println!("****************************************");

    assert!(output.status.success());

    // First create a child process which runs qemu for testing
    // Then, create a watchdog to check if test is finished

    let mut tester_proc = Command::new("qemu-system-x86_64")
        .arg("-nographic")
        .arg("-m")
        .arg("512")
        .arg("-kernel")
        .arg("../rumprun-sel4/images/kernel-x86_64-pc99")
        .arg("-initrd")
        .arg("../rumprun-sel4/images/roottask-image-x86_64-pc99")
        .arg("-cpu")
        .arg("Haswell")
        .stdout(rawfd)
        .spawn()
        .expect("failed to RUN");

    use std::thread;
    use std::io;
    use std::io::prelude::*;

    let mut child_proc_finished = 0;
    let mut test_succeeded = 0;
    let mut test_length = 0;
    let test_length_max = 60; // Maximum allowed length for a test is currently 60 seconds

    // This loop checks the output file to recognize when qemu vm \
    // which is running the test, should be terminated
    while child_proc_finished == 0 {
        thread::sleep_ms(5000);
        test_length += 5;
        {
            let mut results_file = File::open("outputs.txt");
            let mut results_file = match results_file {
                Ok(the_file) => the_file,
                Err(error) => {
                    panic!("Checking outputs file failed with error -{}-", error);
                }
            };
            let mut file_content = String::new();

            results_file.read_to_string(&mut file_content);

            if file_content.contains("bmk_platform_halt@kernel.c:95 All is well in the universe.") {
                child_proc_finished = 1;
                if file_content.contains("@#$%PASSED%$#@") {
                    test_succeeded = 1;
                } else if file_content.contains("@#$%FAILED%$#@") {
                    test_succeeded = 0;
                } else {
                    panic!("Invalid test outcome!");
                }
            } else {
                continue;
            }

            use std::str::FromStr;
            use std::fs::OpenOptions;

            let mut lines = file_content.lines();
            let mut search_finished = 0;
            let mut test_name = String::new();
            while search_finished == 0 {
                let mut current_line = lines.next();
                if current_line == None {
                    panic!("Test name not found in outputs.txt");
                }
                let current_line = current_line.unwrap();
                println!("{}", current_line);
                if current_line.contains("**TEST**") {
                    search_finished = 1;
                    test_name = String::from_str(lines.next().unwrap()).unwrap();
                }
            }

            //            let mut log_file = File::create("results_log.txt");
            let mut log_file = OpenOptions::new()
                .write(true)
                .append(true)
                .open("results_log.txt");
            let mut log_file = match log_file {
                Ok(the_file) => the_file,
                Err(error) => {
                    panic!("Creating-Opening log file failed with error -{}-", error);
                }
            };

            log_file
                .write_fmt(format_args!("******************************\n"))
                .unwrap();
            log_file
                .write_fmt(format_args!("Test time : {}\n", time::now_utc().ctime()))
                .unwrap();
            log_file
                .write_fmt(format_args!("Test name : {}\n", test_name))
                .unwrap();
            if test_succeeded == 1 {
                log_file
                    .write_fmt(format_args!("Test result : PASSED\n"))
                    .unwrap();
            } else {
                log_file
                    .write_fmt(format_args!("Test result : FAILED\n"))
                    .unwrap();
            }
            log_file
                .write_fmt(format_args!("******************************"))
                .unwrap();
        }
        println!("+ 5 secs");
        if test_length == test_length_max {
            let output = Command::new("kill")
                .arg("-15")
                .arg("--")
                .arg(tester_proc.id().to_string())
                .output()
                .expect("failed to kill TO");

            assert!(output.status.success());

            panic!("Test Timed Out!");
        }
    }

    // Terminate the test proc
    let output = Command::new("kill")
        .arg("-15")
        .arg("--")
        .arg(tester_proc.id().to_string())
        .output()
        .expect("failed to kill");

    assert!(output.status.success());

    assert!(test_succeeded == 1);
}
