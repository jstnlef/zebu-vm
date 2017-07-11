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

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[cfg(target_arch = "x86_64")]
fn main() {
    gcc::compile_library("libruntime.a", &["src/runtime/runtime_x64_sysv.c"]);
    
    gcc::Config::new().flag("-O3").flag("-c")
                     .file("src/runtime/swap_stack_x64_sysv.S")
                     .compile("libswap_stack.a"); 
}

#[cfg(target_os = "linux")]
#[cfg(target_arch = "aarch64")]
fn main() {
    gcc::compile_library("libruntime.a", &["src/runtime/runtime_aarch64_sysv.c"]);

    gcc::Config::new().flag("-O3").flag("-c")
        .file("src/runtime/swap_stack_aarch64_sysv.S")
        .compile("libswap_stack.a");
}

// This is here to enable cross compiling from windows/x86_64 to linux/aarch64
#[cfg(target_os = "windows")]
#[cfg(target_arch = "x86_64")]
fn main() {
    gcc::compile_library("libruntime.a", &["src/runtime/runtime_aarch64_sysv.c"]);

    gcc::Config::new().flag("-O3").flag("-c")
        .file("src/runtime/swap_stack_aarch64_sysv.S")
        .compile("libswap_stack.a");
}

#[cfg(feature = "sel4-rumprun")]
#[cfg(target_arch = "x86_64")]
fn main() {
    let mut compiler_name = String::new();
    compiler_name.push_str("x86_64-rumprun-netbsd-gcc");
    gcc::Config::new().flag("-O3").flag("-c")
        .compiler(Path::new(compiler_name.as_str()))
        .file("src/runtime/runtime_x64_sel4_rumprun_sysv.c")
        .compile("libruntime.a");
    gcc::Config::new().flag("-O3").flag("-c")
        .compiler(Path::new(compiler_name.as_str()))
        .file("src/runtime/swap_stack_x64_sysv.S")
        .compile("libswap_stack.a");
    gcc::Config::new().flag("-O3").flag("-c")
        .compiler(Path::new(compiler_name.as_str()))
        .file("c_helpers.c")
        .compile("libc_helpers.a");
}