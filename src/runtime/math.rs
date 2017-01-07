#[no_mangle]
#[allow(unreachable_code)]
pub extern fn muentry_frem(a: f64, b: f64) -> f64 {
    use std::ops::Rem;

    a.rem(b)
}
