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

/// returns the exponent if the given number is power of two, otherwise return None
pub fn is_power_of_two(x: usize) -> Option<u8> {
    use std::u8;

    let mut power_of_two = 1;
    let mut i: u8 = 0;
    while power_of_two < x && i < u8::MAX {
        power_of_two *= 2;
        i += 1;
    }

    if power_of_two == x {
        Some(i)
    } else {
        None
    }
}

/// aligns up a number
/// (returns the nearest multiply of the align value that is larger than the given value)
#[inline(always)]
pub fn align_up(x: usize, align: usize) -> usize {
    //use ((x + align - 1)/align)*align if align is not a power of two
    debug_assert!(align.is_power_of_two());
    (x + align - 1) & !(align  - 1)
}
