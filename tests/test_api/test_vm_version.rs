use mu::vm::api::mu_get_version;
use std::ffi::CStr;

#[test]
fn test_mu_get_version() {
    println!("{:?}", unsafe { CStr::from_ptr(mu_get_version()) });
}