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

#[cfg(not(feature = "sel4-rumprun-target-side"))]
include!(concat!(env!("OUT_DIR"), "/built.rs"));

use std::ffi::CString;
use std::string::String;

#[cfg(not(feature = "sel4-rumprun-target-side"))]
lazy_static! {
    pub static ref ZEBU_VERSION_STR: String = {
        let git = match GIT_VERSION {
            Some(str) => str,
            None => "no git version found"
        };

        let built_time = built::util::strptime(&BUILT_TIME_UTC);

        format!("Zebu {} ({}, {})", PKG_VERSION, git, built_time.ctime())
    };
    pub static ref ZEBU_VERSION_C_STR: CString = CString::new(ZEBU_VERSION_STR.clone()).unwrap();
}

#[cfg(feature = "sel4-rumprun-target-side")]
lazy_static! {
    pub static ref ZEBU_VERSION_STR: String = { "Not Available in sel4-rumprun".to_string() };
    pub static ref ZEBU_VERSION_C_STR: CString = CString::new(ZEBU_VERSION_STR.clone()).unwrap();
}
