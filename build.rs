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
    gcc::compile_library("libruntime_c.a", &["src/runtime/runtime_c_x64_sysv.c"]);
    
    gcc::Config::new().flag("-O3").flag("-c")
                     .file("src/runtime/runtime_asm_x64_sysv.S")
                     .compile("libruntime_asm.a");
}

#[cfg(target_os = "linux")]
#[cfg(target_arch = "aarch64")]
fn main() {
    gcc::compile_library("libruntime_c.a", &["src/runtime/runtime_c_aarch64_sysv.c"]);

    gcc::Config::new().flag("-O3").flag("-c")
        .file("src/runtime/runtime_asm_aarch64_sysv.S")
        .compile("libruntime_asm.a");
}