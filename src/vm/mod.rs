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

/// vm module: storing metadata, implementing APIs
mod vm;
/// re-export VM. VM stores metadata for a running Zebu instance,
/// which includes types, globals, functions/IRs, compiled functions
/// and other runtime table (exception table etc)
pub use vm::vm::VM;

/// vm_options defines commandline flags to create a new Zebu instance
mod vm_options;

/// api module implements the C functions in muapi.h exposed as Mu API
pub mod api;

/// handle type for client. This handle type is opaque to the client
pub mod handle;
