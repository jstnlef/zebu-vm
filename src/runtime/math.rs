#[no_mangle]
#[allow(unreachable_code)]
pub extern fn muentry_frem32(a: f32, b: f32) -> f32 {
    use std::ops::Rem;

    a.rem(b)
}

#[no_mangle]
#[allow(unreachable_code)]
pub extern fn muentry_frem64(a: f64, b: f64) -> f64 {
    use std::ops::Rem;

    a.rem(b)
}

extern crate num_traits;
use extprim::u128::u128;
use extprim::i128::i128;
use runtime::math::num_traits::ToPrimitive;
use runtime::math::num_traits::FromPrimitive;

#[no_mangle]
pub extern fn muentry_udiv_u128(a: u128, b: u128) -> u128 {
    a.wrapping_div(b)
}

#[no_mangle]
pub extern fn muentry_sdiv_i128(a: i128, b: i128) -> i128 {
    a.wrapping_div(b)
}

#[no_mangle]
pub extern fn muentry_urem_u128(a: u128, b: u128) -> u128 {
    a.wrapping_rem(b)
}

#[no_mangle]
pub extern fn muentry_srem_i128(a: i128, b: i128) -> i128 {
    a.wrapping_rem(b)
}

#[no_mangle]
pub extern fn muentry_fptoui_double_u128(a: f64) -> u128 { u128::from_f64(a).unwrap() }
#[no_mangle]
pub extern fn muentry_fptosi_double_i128(a: f64) -> i128 { i128::from_f64(a).unwrap() }
#[no_mangle]
pub extern fn muentry_uitofp_u128_double(a: u128) -> f64 { a.to_f64().unwrap() }
#[no_mangle]
pub extern fn muentry_sitofp_i128_double(a: i128) -> f64 { a.to_f64().unwrap() }

#[no_mangle]
pub extern fn muentry_fptoui_float_u128(a: f32) -> u128 { u128::from_f32(a).unwrap() }
#[no_mangle]
pub extern fn muentry_fptosi_float_i128(a: f32) -> i128 { i128::from_f32(a).unwrap() }
#[no_mangle]
pub extern fn muentry_uitofp_u128_float(a: u128) -> f32 { a.to_f32().unwrap() }
#[no_mangle]
pub extern fn muentry_sitofp_i128_float(a: i128) -> f32 { a.to_f32().unwrap() }