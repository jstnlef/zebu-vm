// Copyright 2017 The Australian National University
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate libloading;
extern crate zebu_vm as mu;

use self::mu::ast::inst::*;
use self::mu::ast::ir::*;
use self::mu::ast::op::*;
use self::mu::ast::types::*;
use self::mu::utils::LinkedHashMap;
use self::mu::vm::*;

use mu::linkutils;

#[test]
fn test_add_u8() {
    let lib = linkutils::aot::compile_fnc("add_u8", &add_u8);

    unsafe {
        let add_u8: libloading::Symbol<unsafe extern "C" fn(u8, u8) -> u8> = lib.get(b"add_u8").unwrap();

        let add_u8_1_1 = add_u8(1, 1);
        println!("add_u8(1, 1) = {}", add_u8_1_1);
        assert!(add_u8_1_1 == 2);

        let add_u8_255_1 = add_u8(255u8, 1u8);
        println!("add_u8(255, 1) = {}", add_u8_255_1);
        assert!(add_u8_255_1 == 0);
    }
}

fn add_u8() -> VM {
    let vm = VM::new();

    typedef!        ((vm) u8 = mu_int(8));

    funcsig!        ((vm) add_u8_sig = (u8, u8) -> (u8));
    funcdecl!       ((vm) <add_u8_sig> add_u8);
    funcdef!        ((vm) <add_u8_sig> add_u8 VERSION add_u8_v1);

    // %entry(<@u8> %a, <@u8> %b):
    block!          ((vm, add_u8_v1) blk_entry);
    ssa!            ((vm, add_u8_v1) <u8> a);
    ssa!            ((vm, add_u8_v1) <u8> b);

    // %r = ADD %a %b
    ssa!            ((vm, add_u8_v1) <u8> r);
    inst!           ((vm, add_u8_v1) blk_entry_add:
        r = BINOP (BinOp::Add) a b
    );

    // RET %r
    inst!           ((vm, add_u8_v1) blk_entry_ret:
        RET (r)
    );

    define_block!   ((vm, add_u8_v1) blk_entry(a, b) {
        blk_entry_add,
        blk_entry_ret
    });

    define_func_ver!((vm) add_u8_v1 (entry: blk_entry) {
        blk_entry
    });

    vm
}

#[test]
fn test_truncate() {
    let lib = linkutils::aot::compile_fnc("truncate", &truncate);

    unsafe {
        let truncate: libloading::Symbol<unsafe extern "C" fn(u64) -> u8> = lib.get(b"truncate").unwrap();

        let res = truncate(0xF01u64);
        println!("truncate(0xF01) = {}", res);
        assert!(res == 1);
    }
}

fn truncate() -> VM {
    let vm = VM::new();

    typedef! ((vm) u64 = mu_int(64));
    typedef! ((vm) u8  = mu_int(8));

    funcsig! ((vm) sig = (u64) -> (u64));
    funcdecl!((vm) <sig> truncate);
    funcdef! ((vm) <sig> truncate VERSION truncate_v1);

    block!   ((vm, trucnate_v1) blk_entry);
    ssa!     ((vm, truncate_v1) <u64> blk_entry_a);

    ssa!     ((vm, truncate_v1) <u8>  blk_entry_r);
    inst!    ((vm, truncate_v1) blk_entry_truncate:
        blk_entry_r = CONVOP (ConvOp::TRUNC) <u64 u8> blk_entry_a
    );

    ssa!     ((vm, truncate_v1) <u64> blk_entry_r2);
    inst!    ((vm, truncate_v1) blk_entry_zext:
        blk_entry_r2 = CONVOP (ConvOp::ZEXT) <u8 u64> blk_entry_r
    );

    inst!    ((vm, truncate_v1) blk_entry_ret:
        RET (blk_entry_r2)
    );

    define_block! ((vm, truncate_v1) blk_entry(blk_entry_a) {
        blk_entry_truncate,
        blk_entry_zext,
        blk_entry_ret
    });

    define_func_ver! ((vm) truncate_v1 (entry: blk_entry) {blk_entry});

    vm
}

#[test]
fn test_sext() {
    let lib = linkutils::aot::compile_fnc("sext", &sext);

    unsafe {
        let sext: libloading::Symbol<unsafe extern "C" fn(i8) -> i64> = lib.get(b"sext").unwrap();

        let res = sext(-1);
        println!("truncate(-1) = {}", res);
        assert!(res == -1);
    }
}

fn sext() -> VM {
    let vm = VM::new();

    typedef!        ((vm) i8  = mu_int(8));
    typedef!        ((vm) i64 = mu_int(64));

    funcsig!        ((vm) sext_sig = (i8) -> (i64));
    funcdecl!       ((vm) <sext_sig> sext);
    funcdef!        ((vm) <sext_sig> sext VERSION sext_v1);

    // %entry(<@i8> %a):
    block!          ((vm, sext_v1) blk_entry);
    ssa!            ((vm, sext_v1) <i8> a);

    // %r = SEXT @i8->@i64 %a
    ssa!            ((vm, sext_v1) <i64> r);
    inst!           ((vm, sext_v1) blk_entry_sext:
        r = CONVOP (ConvOp::SEXT) <i8 i64> a
    );

    // RET %r
    inst!           ((vm, sext_v1) blk_entry_ret:
        RET (r)
    );

    define_block!   ((vm, sext_v1) blk_entry(a) {
        blk_entry_sext, blk_entry_ret
    });

    define_func_ver!((vm) sext_v1 (entry: blk_entry) {
        blk_entry
    });

    vm
}

#[test]
fn test_add_9f() {
    let lib = linkutils::aot::compile_fnc("add_9f", &add_9f);

    unsafe {
        let add_9f: libloading::Symbol<unsafe extern "C" fn(u64) -> u64> = lib.get(b"add_9f").unwrap();

        let add_9f_1 = add_9f(1);
        println!("add_9f(1) = {}", add_9f_1);
        assert!(add_9f_1 == 0x1000000000);
    }
}

fn add_9f() -> VM {
    let vm = VM::new();

    typedef!        ((vm) u64 = mu_int(64));

    constdef!       ((vm) <u64> int64_9f = Constant::Int(0xfffffffff));

    funcsig!        ((vm) add_9f_sig = (u64) -> (u64));
    funcdecl!       ((vm) <add_9f_sig> add_9f);
    funcdef!        ((vm) <add_9f_sig> add_9f VERSION add_9f_v1);

    // %entry(<@u64> %a):
    block!          ((vm, add_9f_v1) blk_entry);
    ssa!            ((vm, add_9f_v1) <u64> a);

    // %r = ADD %a %b
    ssa!            ((vm, add_9f_v1) <u64> r);
    consta!         ((vm, add_9f_v1) int64_9f_local = int64_9f);
    inst!           ((vm, add_9f_v1) blk_entry_add:
        r = BINOP (BinOp::Add) a int64_9f_local
    );

    // RET %r
    inst!           ((vm, add_9f_v1) blk_entry_ret:
        RET (r)
    );

    define_block!   ((vm, add_9f_v1) blk_entry(a) {
        blk_entry_add,
        blk_entry_ret
    });

    define_func_ver!((vm) add_9f_v1(entry: blk_entry) {
        blk_entry
    });

    vm
}
