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

mod api_c;
mod api_bridge;
pub mod api_impl;

mod deps { 
    use std::cell::*;

    // should import from ast/src/ir.rs
    pub type WPID  = usize;
    pub type MuID  = usize;
    pub type MuName = String;
    pub type CName  = MuName;

    #[derive(Debug)]
    pub enum ValueBox {
        BoxInt(u64, i32),
        BoxF32(f32),
        BoxF64(f64),
        BoxRef(Cell<usize>),    // so that GC can update the pointer
        BoxSeq(Vec<ValueBox>),
        BoxThread,
        BoxStack,
    }

    #[derive(Debug)]
    pub struct APIHandle {
        pub ty: MuID,
        pub vb: ValueBox,
    }

}
