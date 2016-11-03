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
