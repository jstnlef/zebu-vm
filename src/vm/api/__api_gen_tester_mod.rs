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

// Invoke "python3 muapi2rustapi.py", and then
// invoke "rustc --test __api_gen_tester_mod.rs -o /tmp/api_gen_tester_junk"
// to test whether the generated code compiles.

mod api_c;
mod api_bridge;
mod __api_impl_stubs;
mod api_impl {
    pub use __api_impl_stubs::*;
}

/// This is for testing. In the productional setting, replace them with the definitions from
/// `src/ast/src/ir.rs` and `src/ast/src/bundle.rs`
mod deps { 

    // should import from ast/src/ir.rs
    pub type WPID  = usize;
    pub type MuID  = usize;
    pub type MuName = String;
    pub type CName  = MuName;

    pub struct APIHandle {
        // stub
    }

}
