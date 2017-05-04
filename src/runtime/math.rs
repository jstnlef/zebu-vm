#[no_mangle]
#[allow(unreachable_code)]
pub extern fn muentry_frem_double(a: f64, b: f64) -> f64 {
    use std::ops::Rem;

    a.rem(b)
}

#[no_mangle]
#[allow(unreachable_code)]
pub extern fn muentry_frem_float(a: f32, b: f32) -> f32 {
    use std::ops::Rem;

    a.rem(b)
}

use extprim::u128::u128;
use extprim::i128::i128;

#[no_mangle]
#[allow(unreachable_code)]
pub extern fn muentry_udiv_u128(a: u128, b: u128) -> u128 {
    a.wrapping_div(b)
}

#[no_mangle]
#[allow(unreachable_code)]
pub extern fn muentry_sdiv_i128(a: i128, b: i128) -> i128 {
    a.wrapping_div(b)
}

#[no_mangle]
#[allow(unreachable_code)]
pub extern fn muentry_urem_u128(a: u128, b: u128) -> u128 {
    a.wrapping_rem(b)
}

#[no_mangle]
#[allow(unreachable_code)]
pub extern fn muentry_srem_i128(a: i128, b: i128) -> i128 {
    a.wrapping_rem(b)
}