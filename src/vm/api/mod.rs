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

pub mod api_c;      // This is pub because `api_c` can be used directly. It is just an interface.
mod api_bridge;     // This is mostly auto-generatd code, and should not be used externally.
mod api_impl;       // Mostly private. 

pub use self::api_impl::mu_fastimpl_new;
pub use self::api_impl::mu_fastimpl_new_with_opts;

mod deps {
    pub use ast::ir::WPID;
    pub use ast::ir::MuID;
    pub use ast::ir::MuName;
    pub use ast::ir::CName;
    pub use vm::handle::APIHandle;
    extern crate ast;
}