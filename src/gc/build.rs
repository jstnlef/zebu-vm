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
    gcc::compile_library("libgc_clib_x64.a", &["src/heap/gc/clib_x64.c"]);
}

#[cfg(target_os = "linux")]
#[cfg(target_arch = "aarch64")]
fn main() {
    gcc::compile_library("libgc_clib_aarch64.a", &["src/heap/gc/clib_aarch64.S"]);
}

// This is here to enable cross compiling from windows/x86_64 to linux/aarch64
#[cfg(target_os = "windows")]
#[cfg(target_arch = "x86_64")]
fn main() {
    gcc::compile_library("libgc_clib_aarch64.a", &["src/heap/gc/clib_aarch64.S"]);
}
