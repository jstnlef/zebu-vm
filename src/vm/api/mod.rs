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

//! This module implements the Mu C API (defined in Mu spec:
//! https://gitlab.anu.edu.au/mu/mu-spec/blob/master/muapi.h) for Zebu.
//!
//! The API that Zebu exposes to native is "mu-fastimpl.h". A client
//! should include the header file, link against the static/dynmic library
//! of Zebu and access Mu functionality through the API functions.
//!
//! This module bridges the exposed C API and Zebu's internal representation.
//! Due to the fact that the API is long, formatted, and subject to change, this module
//! contains a lot of generated code from the C API header file (by Python scripts
//! within the same directory).

/// generated from muapi.h, and it declares structs/types in API
pub mod api_c; // This is pub because `api_c` can be used directly. It is just an interface.

/// generated code to bridge C API to internal representation in a Rust-friendly way
mod api_bridge; // This is mostly auto-generatd code, and should not be used externally.

/// implements __api_impl_stubs.rs that is generated from the API
/// this actually implements the API calls
mod api_impl; // Mostly private.

/// creates a Zebu instance with default options
/// There is no standard API to create a Mu VM (it is implementation dependent)
pub use self::api_impl::mu_fastimpl_new;

/// creates a Zebu instance with specified options
/// See vm_options.rs for supported options.
pub use self::api_impl::mu_fastimpl_new_with_opts;

/// returns a version string for current Zebu build
pub use self::api_impl::mu_get_version;

mod deps {
    pub use ast::ir::CName;
    pub use ast::ir::MuID;
    pub use ast::ir::MuName;
    pub use ast::ir::WPID;
    pub use vm::handle::APIHandle;
    extern crate mu_ast as ast;
}

pub use self::api_impl::VALIDATE_IR;
