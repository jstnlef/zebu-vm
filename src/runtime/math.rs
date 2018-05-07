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

//! Some mathematical functions used at runtime

/// remainder for float type
#[no_mangle]
pub extern "C" fn muentry_frem32(a: f32, b: f32) -> f32 {
    use std::ops::Rem;
    a.rem(b)
}

/// remainder for double type
#[no_mangle]
pub extern "C" fn muentry_frem64(a: f64, b: f64) -> f64 {
    use std::ops::Rem;
    a.rem(b)
}

extern crate num_traits;
use extprim::i128::i128;
use extprim::u128::u128;
use runtime::math::num_traits::FromPrimitive;
use runtime::math::num_traits::ToPrimitive;

/// unsigned division for int128
#[no_mangle]
pub extern "C" fn muentry_udiv_u128(a: u128, b: u128) -> u128 {
    a.wrapping_div(b)
}
/// signed division for int128
#[no_mangle]
pub extern "C" fn muentry_sdiv_i128(a: i128, b: i128) -> i128 {
    a.wrapping_div(b)
}
/// unsigned remainder for int128
#[no_mangle]
pub extern "C" fn muentry_urem_u128(a: u128, b: u128) -> u128 {
    a.wrapping_rem(b)
}
/// signed division for int128
#[no_mangle]
pub extern "C" fn muentry_srem_i128(a: i128, b: i128) -> i128 {
    a.wrapping_rem(b)
}

/// double to unsigned int128
#[no_mangle]
pub extern "C" fn muentry_fptoui_double_u128(a: f64) -> u128 {
    u128::from_f64(a).unwrap()
}
/// double to signed int128
#[no_mangle]
pub extern "C" fn muentry_fptosi_double_i128(a: f64) -> i128 {
    i128::from_f64(a).unwrap()
}
/// unsigned int128 to double
#[no_mangle]
pub extern "C" fn muentry_uitofp_u128_double(a: u128) -> f64 {
    a.to_f64().unwrap()
}
/// signed int128 to double
#[no_mangle]
pub extern "C" fn muentry_sitofp_i128_double(a: i128) -> f64 {
    a.to_f64().unwrap()
}

/// float to unsigned int128
#[no_mangle]
pub extern "C" fn muentry_fptoui_float_u128(a: f32) -> u128 {
    u128::from_f32(a).unwrap()
}
/// float to signed int128
#[no_mangle]
pub extern "C" fn muentry_fptosi_float_i128(a: f32) -> i128 {
    i128::from_f32(a).unwrap()
}
/// unsigned int128 to float
#[no_mangle]
pub extern "C" fn muentry_uitofp_u128_float(a: u128) -> f32 {
    a.to_f32().unwrap()
}
/// signed int128 to float
#[no_mangle]
pub extern "C" fn muentry_sitofp_i128_float(a: i128) -> f32 {
    a.to_f32().unwrap()
}
