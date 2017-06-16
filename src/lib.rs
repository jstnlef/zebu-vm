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

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate rustc_serialize;
extern crate stderrlog;
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate field_offset;
extern crate extprim;

#[macro_use]
pub extern crate ast;
#[macro_use]
pub extern crate utils;
pub mod vm;
pub mod compiler;
pub mod runtime;
pub mod testutil;
