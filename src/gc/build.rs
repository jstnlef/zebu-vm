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

extern crate gcc;

//#[cfg(not(feature = "sel4-rumprun-target-side"))]
#[cfg(any(target_os = "macos", target_os = "linux"))]
#[cfg(target_arch = "x86_64")]
fn build_libgc(){
    println!("detected macos or linux");
    gcc::compile_library("libgc_clib_x64.a", &["src/heap/gc/clib_x64.c"]);
}

//#[cfg(not(feature = "sel4-rumprun-target-side"))]
#[cfg(target_os = "linux")]
#[cfg(target_arch = "aarch64")]
fn build_libgc() {
    println!("detected linux aarch64");
    gcc::compile_library("libgc_clib_aarch64.a", &["src/heap/gc/clib_aarch64.S"]);
}

// This is here to enable cross compiling from windows/x86_64 to linux/aarch64
//#[cfg(not(feature = "sel4-rumprun-target-side"))]
#[cfg(target_os = "windows")]
#[cfg(target_arch = "x86_64")]
fn build_libgc() {
    println!("detected windows 64");
    gcc::compile_library("libgc_clib_aarch64.a", &["src/heap/gc/clib_aarch64.S"]);
}


//#[cfg(all(target_os = "netbsd", target_vendor = "rumprun"))]
//#[cfg(feature = "sel4-rumprun-target-side")]
//#[cfg(target_arch = "x86_64")]
fn main() {
    use std::env;
    let mut gc_target = 0;  // 1 for rumprun-sel4 and 0 otherwise
    for (key, value) in env::vars() {
        if key == "ZEBU_TARGET" {
            if value == "x86_64-rumprun-netbsd" {
                gc_target = 1;
                break;
            }
            else {
                gc_target = 0;
                break;
            }
        }
    }
    // for sel4-rumprun, do (if)
    // otherwise, call the related target function
    if gc_target == 1 {
        use std::path::Path;
        println!("detected rumprun-netbsd");
        let mut compiler_name = String::new();
        compiler_name.push_str("x86_64-rumprun-netbsd-gcc");
        gcc::Config::new().flag("-O3").flag("-c")
            .compiler(Path::new(compiler_name.as_str()))
            .file("src/heap/gc/clib_x64_sel4_rumprun.c")
            .compile("libgc_clib_x64.a");
    }
    else { build_libgc(); }
}
