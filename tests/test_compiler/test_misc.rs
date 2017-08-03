extern crate mu;
extern crate log;
extern crate libloading;

use self::mu::ast::types::*;
use self::mu::ast::ir::*;
use self::mu::ast::inst::*;
use self::mu::vm::*;
use self::mu::linkutils;
use self::mu::utils::LinkedHashMap;

#[test]
fn test_mov_minus_one_to_int8() {
    let lib = linkutils::aot::compile_fnc("mov_minus_one_to_int8", &mov_minus_one_to_int8);

    unsafe {
        let mov_minus_one_to_int8: libloading::Symbol<unsafe extern "C" fn() -> (i8)> =
            lib.get(b"mov_minus_one_to_int8").unwrap();

        let res = mov_minus_one_to_int8();
        println!("mov_minus_one_to_u8() = {}", res);
        assert!(res == -1);
    }
}

fn mov_minus_one_to_int8() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int8 = mu_int(8));

    constdef!   ((vm) <int8> int8_minus_one = Constant::Int(-1i8 as u64));

    funcsig!    ((vm) mov_minus_one_to_int8_sig = () -> (int8));
    funcdecl!   ((vm) <mov_minus_one_to_int8_sig> mov_minus_one_to_int8);
    funcdef!    ((vm) <mov_minus_one_to_int8_sig> mov_minus_one_to_int8
        VERSION mov_minus_one_to_int8_v1);

    block!      ((vm, mov_minus_one_to_int8_v1) blk_entry);
    consta!     ((vm, mov_minus_one_to_int8_v1) int8_minus_one_local = int8_minus_one);
    ssa!        ((vm, mov_minus_one_to_int8_v1) <int8> ret);
    inst!       ((vm, mov_minus_one_to_int8_v1) blk_entry_mov:
        MOVE int8_minus_one_local -> ret
    );

    inst!       ((vm, mov_minus_one_to_int8_v1) blk_entry_ret:
        RET (ret)
    );

    define_block!((vm, mov_minus_one_to_int8_v1) blk_entry() {
        blk_entry_mov,
        blk_entry_ret
    });

    define_func_ver!((vm) mov_minus_one_to_int8_v1 (entry: blk_entry) {
        blk_entry
    });

    vm
}

#[test]
fn test_branch_minus_one_to_int8() {
    let lib = linkutils::aot::compile_fnc("branch_minus_one_to_int8", &branch_minus_one_to_int8);

    unsafe {
        let branch_minus_one_to_int8: libloading::Symbol<unsafe extern "C" fn() -> (i8)> =
            lib.get(b"branch_minus_one_to_int8").unwrap();

        let res = branch_minus_one_to_int8();
        println!("branch_minus_one_to_u8() = {}", res);
        assert!(res == -1);
    }
}

fn branch_minus_one_to_int8() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int8 = mu_int(8));

    constdef!   ((vm) <int8> int8_minus_one = Constant::Int(-1i8 as u64));

    funcsig!    ((vm) branch_minus_one_to_int8_sig = () -> (int8));
    funcdecl!   ((vm) <branch_minus_one_to_int8_sig> branch_minus_one_to_int8);
    funcdef!    ((vm) <branch_minus_one_to_int8_sig> branch_minus_one_to_int8
        VERSION branch_minus_one_to_int8_v1);

    // blk_entry
    block!      ((vm, branch_minus_one_to_int8_v1) blk_entry);
    block!      ((vm, branch_minus_one_to_int8_v1) blk_ret);
    consta!     ((vm, branch_minus_one_to_int8_v1) int8_minus_one_local = int8_minus_one);
    inst!       ((vm, branch_minus_one_to_int8_v1) blk_entry_branch:
        BRANCH blk_ret (int8_minus_one_local)
    );

    define_block!((vm, branch_minus_one_to_int8_v1) blk_entry() {
        blk_entry_branch
    });

    // blk_ret
    ssa!        ((vm, branch_minus_one_to_int8_v1) <int8> res);
    inst!       ((vm, branch_minus_one_to_int8_v1) blk_ret_ret:
        RET (res)
    );

    define_block!((vm, branch_minus_one_to_int8_v1) blk_ret(res) {
        blk_ret_ret
    });

    define_func_ver!((vm) branch_minus_one_to_int8_v1 (entry: blk_entry) {
        blk_entry,
        blk_ret
    });

    vm
}
