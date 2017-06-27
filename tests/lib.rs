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

extern crate mu;
extern crate utils;
#[macro_use]
extern crate log;
extern crate maplit;

#[macro_use]
mod ir_macros;

mod test_ir;
mod test_compiler;
mod test_runtime;
mod test_api;

mod common {
    use std::fmt;

    #[allow(dead_code)]
    pub fn assert_vector_ordered <T: fmt::Debug> (left: &Vec<T>, right: &Vec<T>) {
        assert_debug_str(left, right);
    }

    #[allow(dead_code)]
    pub fn assert_vector_no_order <T: Ord + fmt::Debug + Clone> (left: &Vec<T>, right: &Vec<T>) {
        let mut left_clone = left.clone();
        left_clone.sort();
        let mut right_clone = right.clone();
        right_clone.sort();
        
        assert_debug_str(left_clone, right_clone);
    }

    #[allow(dead_code)]
    pub fn assert_debug_str<T: fmt::Debug, U: fmt::Debug> (left: T, right: U) {
        assert_eq!(format!("{:?}", left), format!("{:?}", right))
    }
}
