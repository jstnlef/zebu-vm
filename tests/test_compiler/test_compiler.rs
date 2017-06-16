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

extern crate libloading as ll;
extern crate mu;

use test_ir::test_ir::sum;
use test_ir::test_ir::factorial;
use mu::testutil;

#[test]
fn test_factorial() {
    let lib = testutil::compile_fnc("fac", &factorial);
    unsafe {
        let fac: ll::Symbol<unsafe extern fn (u64) -> u64> = lib.get(b"fac").unwrap();
        println!("fac(10) = {}", fac(10));
        assert!(fac(10) == 3628800);
    }
}

#[test]
fn test_sum() {
    let lib = testutil::compile_fnc("sum", &sum);
    unsafe {
        let sumptr: ll::Symbol<unsafe extern fn (u64) -> u64> = lib.get(b"sum").unwrap();
        println!("sum(5) = {}", sumptr(5));
        assert!(sumptr(5) == 15);
        println!("sun(10) = {}", sumptr(10));
        assert!(sumptr(10) == 55);
    }
}
