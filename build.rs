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

#[cfg(not(feature = "sel4-rumprun-target-side"))]
extern crate built;

extern crate gcc;

#[cfg(not(feature = "sel4-rumprun-target-side"))]
#[cfg(any(target_os = "macos", target_os = "linux"))]
#[cfg(target_arch = "x86_64")]
fn main() {
    gcc::compile_library("libruntime_c.a", &["src/runtime/runtime_c_x64_sysv.c"]);

    gcc::Build::new()
        .flag("-O3")
        .flag("-c")
        .file("src/runtime/runtime_asm_x64_sysv.S")
        .compile("libruntime_asm.a");

    built();
}

#[cfg(not(feature = "sel4-rumprun-target-side"))]
#[cfg(target_os = "linux")]
#[cfg(target_arch = "aarch64")]
fn main() {
    gcc::compile_library("libruntime_c.a", &["src/runtime/runtime_c_aarch64_sysv.c"]);

    gcc::Build::new()
        .flag("-O3")
        .flag("-c")
        .file("src/runtime/runtime_asm_aarch64_sysv.S")
        .compile("libruntime_asm.a");

    built();
}

#[cfg(not(feature = "sel4-rumprun-target-side"))]
fn built() {
    built::write_built_file().expect("Failed to acquire build-time information");
}


#[cfg(feature = "sel4-rumprun-target-side")]
#[cfg(target_arch = "x86_64")]
fn main() {
    use std::path::Path;
    let mut compiler_name = String::new();
    compiler_name.push_str("x86_64-rumprun-netbsd-gcc");
    gcc::Build::new()
        .flag("-O3")
        .flag("-c")
        .compiler(Path::new(compiler_name.as_str()))
        .file("src/runtime/runtime_x64_sel4_rumprun_sysv.c")
        .compile("libruntime_c.a");
    gcc::Build::new()
        .flag("-O3")
        .flag("-c")
        .compiler(Path::new(compiler_name.as_str()))
        .file("src/runtime/runtime_asm_x64_sel4_rumprun_sysv.S")
        .compile("libruntime_asm.a");
    gcc::Build::new()
        .flag("-O3")
        .flag("-c")
        .compiler(Path::new(compiler_name.as_str()))
        .file("zebu_c_helpers.c")
        .compile("libzebu_c_helpers.a");
}
