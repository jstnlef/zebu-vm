extern crate libloading;

use mu::testutil::c_api::compile_run_c_test;

mod test_binops;
mod test_cmpops;

#[test]
fn test_constant_function() {
    let dylib_path = compile_run_c_test("suite/test_constfunc.c");
    let lib = libloading::Library::new(dylib_path.as_os_str()).unwrap();

    unsafe {
        let func : libloading::Symbol<unsafe extern fn() -> i32> = lib.get(b"test_fnc").unwrap();

        assert!(func() == 0);
    }
}