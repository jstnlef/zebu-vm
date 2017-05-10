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
