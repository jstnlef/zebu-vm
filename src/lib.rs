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

//! Zebu micro virtual machine

extern crate libc;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate rodal;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate stderrlog;
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate field_offset;
extern crate extprim;
extern crate num;

#[macro_use]
pub extern crate mu_ast as ast;
#[macro_use]
pub extern crate mu_utils as utils;
pub extern crate mu_gc as gc;
pub mod vm;
pub mod compiler;
pub mod runtime;
pub mod linkutils;
