extern crate libloading;

use mu::ast::ir::*;
use mu::ast::inst::*;
use mu::ast::types;
use mu::ast::types::*;
use mu::ast::op::*;
use mu::vm::*;

use std::sync::Arc;
use std::sync::RwLock;
use mu::utils::LinkedHashMap;
use mu::testutil;

#[test]
fn test_add_u128() {
    let lib = testutil::compile_fnc("add_u128", &add_u128);

    unsafe {
        let add_u128 : libloading::Symbol<unsafe extern fn(u64, u64, u64, u64) -> (u64, u64)> = lib.get(b"add_u128").unwrap();

        let res = add_u128(1, 0, 1, 0);
        println!("add_u128(1, 1) = {:?}", res);
        assert!(res == (2, 0));
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