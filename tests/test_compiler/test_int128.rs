extern crate libloading;
extern crate extprim;

use mu::ast::ir::*;
use mu::ast::inst::*;
use mu::ast::types::*;
use mu::ast::op::*;
use mu::vm::*;

use std::sync::RwLock;
use mu::utils::LinkedHashMap;
use mu::testutil;

#[test]
fn test_add_u128() {
    let lib = testutil::compile_fnc("add_u128", &add_u128);

    unsafe {
        use std::u64;

        let add_u128 : libloading::Symbol<unsafe extern fn(u64, u64, u64, u64) -> (u64, u64)> = lib.get(b"add_u128").unwrap();

        let res = add_u128(1, 0, 1, 0);
        println!("add_u128(1, 1) = {:?}", res);
        assert!(res == (2, 0));

        let res = add_u128(u64::MAX, 0, 1, 0);
        println!("add_u128(u64::MAX, 1) = {:?}", res);
        assert!(res == (0, 1));
    }
}

fn add_u128() -> VM {
    let vm = VM::new();

    typedef!    ((vm) u128 = mu_int(128));

    funcsig!    ((vm) sig = (u128, u128) -> (u128));
    funcdecl!   ((vm) <sig> add_u128);
    funcdef!    ((vm) <sig> add_u128 VERSION add_u128_v1);

    block!      ((vm, add_u128_v1) blk_entry);
    ssa!        ((vm, add_u128_v1) <u128> a);
    ssa!        ((vm, add_u128_v1) <u128> b);

    // sum = Add %a %b
    ssa!        ((vm, add_u128_v1) <u128> sum);
    inst!       ((vm, add_u128_v1) blk_entry_add_u128:
        sum = BINOP (BinOp::Add) a b
    );

    inst!       ((vm, add_u128_v1) blk_entry_ret:
        RET (sum)
    );

    define_block!   ((vm, add_u128_v1) blk_entry(a, b) {
        blk_entry_add_u128, blk_entry_ret
    });

    define_func_ver!((vm) add_u128_v1 (entry: blk_entry) {blk_entry});

    vm
}

#[test]
fn test_add_const_u128() {
    let lib = testutil::compile_fnc("add_const_u128", &add_const_u128);

    unsafe {
        use std::u64;

        let add_const_u128 : libloading::Symbol<unsafe extern fn(u64, u64) -> (u64, u64)> = lib.get(b"add_const_u128").unwrap();

        let res = add_const_u128(1, 0);
        println!("add_const_u128(1, 1) = {:?}", res);
        assert!(res == (2, 0));

        let res = add_const_u128(u64::MAX, 0);
        println!("add_const_u128(u64::MAX, 1) = {:?}", res);
        assert!(res == (0, 1));
    }
}

fn add_const_u128() -> VM {
    let vm = VM::new();

    typedef!    ((vm) u128 = mu_int(128));

    constdef!   ((vm) <u128> b = Constant::IntEx(vec![1, 0]));

    funcsig!    ((vm) sig = (u128) -> (u128));
    funcdecl!   ((vm) <sig> add_const_u128);
    funcdef!    ((vm) <sig> add_const_u128 VERSION add_const_u128_v1);

    block!      ((vm, add_const_u128_v1) blk_entry);
    ssa!        ((vm, add_const_u128_v1) <u128> a);

    // sum = Add %a %b
    ssa!        ((vm, add_const_u128_v1) <u128> sum);
    consta!     ((vm, add_const_u128_v1) b_local = b);
    inst!       ((vm, add_const_u128_v1) blk_entry_add_const_u128:
        sum = BINOP (BinOp::Add) a b_local
    );

    inst!       ((vm, add_const_u128_v1) blk_entry_ret:
        RET (sum)
    );

    define_block!   ((vm, add_const_u128_v1) blk_entry(a) {
        blk_entry_add_const_u128, blk_entry_ret
    });

    define_func_ver!((vm) add_const_u128_v1 (entry: blk_entry) {blk_entry});

    vm
}

#[test]
fn test_mul_u128() {
    let lib = testutil::compile_fnc("mul_u128", &mul_u128);

    unsafe {
        use std::u64;

        let mul_u128 : libloading::Symbol<unsafe extern fn(u64, u64, u64, u64) -> (u64, u64)> = lib.get(b"mul_u128").unwrap();

        let res = mul_u128(6, 0, 7, 0);
        println!("mul_u128(6, 7) = {:?}", res);
        assert!(res == (42, 0));

        let res = mul_u128(6, 6, 7, 7);
        println!("mul_u128(??, ??) = {:?}", res);
        assert!(res == (42, 84));
    }
}

fn mul_u128() -> VM {
    let vm = VM::new();

    typedef!    ((vm) u128 = mu_int(128));

    funcsig!    ((vm) sig = (u128, u128) -> (u128));
    funcdecl!   ((vm) <sig> mul_u128);
    funcdef!    ((vm) <sig> mul_u128 VERSION mul_u128_v1);

    block!      ((vm, mul_u128_v1) blk_entry);
    ssa!        ((vm, mul_u128_v1) <u128> a);
    ssa!        ((vm, mul_u128_v1) <u128> b);

    // sum = Add %a %b
    ssa!        ((vm, mul_u128_v1) <u128> sum);
    inst!       ((vm, mul_u128_v1) blk_entry_mul_u128:
        sum = BINOP (BinOp::Mul) a b
    );

    inst!       ((vm, mul_u128_v1) blk_entry_ret:
        RET (sum)
    );

    define_block!   ((vm, mul_u128_v1) blk_entry(a, b) {
        blk_entry_mul_u128, blk_entry_ret
    });

    define_func_ver!((vm) mul_u128_v1 (entry: blk_entry) {blk_entry});

    vm
}

#[test]
fn test_udiv_u128() {
    let lib = testutil::compile_fnc("udiv_u128", &udiv_u128);

    unsafe {
        use self::extprim::u128::u128;

        let udiv_u128 : libloading::Symbol<unsafe extern fn(u64, u64, u64, u64) -> (u64, u64)> = lib.get(b"udiv_u128").unwrap();

        let res = udiv_u128(42, 0, 7, 0);
        println!("udiv_u128(42, 7) = {:?}", res);
        assert!(res == (6, 0));

        let res = udiv_u128(41, 42, 6, 7);
        let a = u128::from_parts(42, 41); // hi, lo
        let b = u128::from_parts(7, 6);
        let expect = a.wrapping_div(b);

        println!("udiv_u128(??, ??) = {:?}", res);
        assert!(expect.low64()  == res.0);
        assert!(expect.high64() == res.1)
    }
}

fn udiv_u128() -> VM {
    let vm = VM::new();

    typedef!    ((vm) u128 = mu_int(128));

    funcsig!    ((vm) sig = (u128, u128) -> (u128));
    funcdecl!   ((vm) <sig> udiv_u128);
    funcdef!    ((vm) <sig> udiv_u128 VERSION udiv_u128_v1);

    block!      ((vm, udiv_u128_v1) blk_entry);
    ssa!        ((vm, udiv_u128_v1) <u128> a);
    ssa!        ((vm, udiv_u128_v1) <u128> b);

    // sum = Add %a %b
    ssa!        ((vm, udiv_u128_v1) <u128> sum);
    inst!       ((vm, udiv_u128_v1) blk_entry_udiv_u128:
        sum = BINOP (BinOp::Udiv) a b
    );

    inst!       ((vm, udiv_u128_v1) blk_entry_ret:
        RET (sum)
    );

    define_block!   ((vm, udiv_u128_v1) blk_entry(a, b) {
        blk_entry_udiv_u128, blk_entry_ret
    });

    define_func_ver!((vm) udiv_u128_v1 (entry: blk_entry) {blk_entry});

    vm
}

#[test]
fn test_shl_u128() {
    let lib = testutil::compile_fnc("shl_u128", &shl_u128);

    unsafe {
        let shl_u128 : libloading::Symbol<unsafe extern fn(u64, u64, u64, u64) -> (u64, u64)> = lib.get(b"shl_u128").unwrap();

        let res = shl_u128(1, 0, 64, 0);
        println!("shl_u128(1, 64) = {:?}", res);
        assert!(res == (0, 1));

        let res = shl_u128(1, 1, 64, 0);
        println!("shl_u128(1, 64) = {:?}", res);
        assert!(res == (0, 1));
    }
}

fn shl_u128() -> VM {
    let vm = VM::new();

    typedef!    ((vm) u128 = mu_int(128));

    funcsig!    ((vm) sig = (u128, u128) -> (u128));
    funcdecl!   ((vm) <sig> shl_u128);
    funcdef!    ((vm) <sig> shl_u128 VERSION shl_u128_v1);

    block!      ((vm, shl_u128_v1) blk_entry);
    ssa!        ((vm, shl_u128_v1) <u128> a);
    ssa!        ((vm, shl_u128_v1) <u128> b);

    // sum = Add %a %b
    ssa!        ((vm, shl_u128_v1) <u128> sum);
    inst!       ((vm, shl_u128_v1) blk_entry_shl_u128:
        sum = BINOP (BinOp::Shl) a b
    );

    inst!       ((vm, shl_u128_v1) blk_entry_ret:
        RET (sum)
    );

    define_block!   ((vm, shl_u128_v1) blk_entry(a, b) {
        blk_entry_shl_u128, blk_entry_ret
    });

    define_func_ver!((vm) shl_u128_v1 (entry: blk_entry) {blk_entry});

    vm
}

#[test]
fn test_lshr_u128() {
    let lib = testutil::compile_fnc("lshr_u128", &lshr_u128);

    unsafe {
        let lshr_u128 : libloading::Symbol<unsafe extern fn(u64, u64, u64, u64) -> (u64, u64)> = lib.get(b"lshr_u128").unwrap();

        let res = lshr_u128(1, 1, 64, 0);
        println!("lshr_u128(100000000000...0001, 64) = {:?}", res);
        assert!(res == (1, 0));

        let res = lshr_u128(1, 0xffffffffffffffff, 64, 0);
        println!("lshr_u128(0xffffffffffffffff0000000000000001, 64) = {:?}", res);
        assert!(res == (0xffffffffffffffff, 0));
    }
}

fn lshr_u128() -> VM {
    let vm = VM::new();

    typedef!    ((vm) u128 = mu_int(128));

    funcsig!    ((vm) sig = (u128, u128) -> (u128));
    funcdecl!   ((vm) <sig> lshr_u128);
    funcdef!    ((vm) <sig> lshr_u128 VERSION lshr_u128_v1);

    block!      ((vm, lshr_u128_v1) blk_entry);
    ssa!        ((vm, lshr_u128_v1) <u128> a);
    ssa!        ((vm, lshr_u128_v1) <u128> b);

    // sum = Add %a %b
    ssa!        ((vm, lshr_u128_v1) <u128> sum);
    inst!       ((vm, lshr_u128_v1) blk_entry_lshr_u128:
        sum = BINOP (BinOp::Lshr) a b
    );

    inst!       ((vm, lshr_u128_v1) blk_entry_ret:
        RET (sum)
    );

    define_block!   ((vm, lshr_u128_v1) blk_entry(a, b) {
        blk_entry_lshr_u128, blk_entry_ret
    });

    define_func_ver!((vm) lshr_u128_v1 (entry: blk_entry) {blk_entry});

    vm
}

#[test]
fn test_ashr_u128() {
    let lib = testutil::compile_fnc("ashr_u128", &ashr_u128);

    unsafe {
        let ashr_u128 : libloading::Symbol<unsafe extern fn(u64, u64, u64, u64) -> (u64, u64)> = lib.get(b"ashr_u128").unwrap();

        let res = ashr_u128(1, 0xffffffffffffffff, 64, 0);
        println!("ashr_u128(0xffffffffffffffff0000000000000001, 64) = {:?}", res);
        assert!(res == (0xffffffffffffffff, 0xffffffffffffffff));
    }
}

fn ashr_u128() -> VM {
    let vm = VM::new();

    typedef!    ((vm) u128 = mu_int(128));

    funcsig!    ((vm) sig = (u128, u128) -> (u128));
    funcdecl!   ((vm) <sig> ashr_u128);
    funcdef!    ((vm) <sig> ashr_u128 VERSION ashr_u128_v1);

    block!      ((vm, ashr_u128_v1) blk_entry);
    ssa!        ((vm, ashr_u128_v1) <u128> a);
    ssa!        ((vm, ashr_u128_v1) <u128> b);

    // sum = Add %a %b
    ssa!        ((vm, ashr_u128_v1) <u128> sum);
    inst!       ((vm, ashr_u128_v1) blk_entry_ashr_u128:
        sum = BINOP (BinOp::Ashr) a b
    );

    inst!       ((vm, ashr_u128_v1) blk_entry_ret:
        RET (sum)
    );

    define_block!   ((vm, ashr_u128_v1) blk_entry(a, b) {
        blk_entry_ashr_u128, blk_entry_ret
    });

    define_func_ver!((vm) ashr_u128_v1 (entry: blk_entry) {blk_entry});

    vm
}

#[test]
fn test_store_load_u128() {
    let lib = testutil::compile_fnc("store_load_u128", &store_load_u128);

    unsafe {
        use mu::utils::mem::memsec::malloc;
        let ptr = match malloc::<u64>(16) {
            Some(ptr) => ptr,
            None => panic!("failed to alloc memory for testing")
        };

        let store_load_u128 : libloading::Symbol<unsafe extern fn(u64, u64, *mut u64) -> (u64, u64)> = lib.get(b"store_load_u128").unwrap();

        let res = store_load_u128(1, 2, ptr);
        println!("store_load(1, 2, ptr) = {:?}", res);
        assert!(res == (1, 2));
    }
}

fn store_load_u128() -> VM {
    let vm = VM::new();

    typedef!    ((vm) u128 = mu_int(128));
    typedef!    ((vm) uptr_u128 = mu_uptr(u128));

    funcsig!    ((vm) sig = (u128, uptr_u128) -> (u128));
    funcdecl!   ((vm) <sig> store_load_u128);
    funcdef!    ((vm) <sig> store_load_u128 VERSION store_load_u128_v1);

    block!      ((vm, store_load_u128_v1) blk_entry);
    ssa!        ((vm, store_load_u128_v1) <u128> x);
    ssa!        ((vm, store_load_u128_v1) <uptr_u128> ptr);

    // store
    inst!       ((vm, store_load_u128_v1) blk_entry_store:
        STORE ptr x (is_ptr: true, order: MemoryOrder::Relaxed)
    );

    // load
    ssa!        ((vm, store_load_u128_v1) <u128> val);
    inst!       ((vm, store_load_u128_v1) blk_entry_load:
        val = LOAD ptr (is_ptr: true, order: MemoryOrder::Relaxed)
    );

    // ret
    inst!       ((vm, store_load_u128_v1) blk_entry_ret:
        RET (val)
    );

    define_block!((vm, store_load_u128_v1) blk_entry(x, ptr) {
        blk_entry_store,
        blk_entry_load,
        blk_entry_ret
    });

    define_func_ver!((vm) store_load_u128_v1(entry: blk_entry) {
        blk_entry
    });

    vm
}