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

extern crate mu;
extern crate libloading;

use self::mu::ast::types::*;
use self::mu::ast::ir::*;
use self::mu::ast::inst::*;
use self::mu::ast::op::*;
use self::mu::vm::*;
use self::mu::utils::LinkedHashMap;

use mu::testutil;

use self::mu::compiler::*;
use self::mu::testutil::aot;
use std::sync::Arc;

#[test]
fn test_add_u8() {
    build_and_run_test!(add_u8, add_u8_test1);
    build_and_run_test!(add_u8, add_u8_test2);
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
    
    emit_test!      ((vm) (add_u8 add_u8_test1 add_u8_test1_v1 Int,Int,Int,EQ (add_u8_sig, u8(1u8 as u64), u8(1u8 as u64), u8(2u8 as u64))));
    emit_test!      ((vm) (add_u8 add_u8_test2 add_u8_test2_v1 Int,Int,Int,EQ (add_u8_sig, u8(255u8 as u64), u8(1u8 as u64), u8(0u8 as u64))));
    
    vm
}

#[test]
fn test_truncate() {
    build_and_run_test!(truncate, truncate_test1);
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
    
    emit_test!      ((vm) (truncate truncate_test1 truncate_test1_v1 Int,Int,EQ (sig, u64(0xF01u64), u64(1u64))));
    
    vm
}

#[test]
fn test_sext() {
    build_and_run_test!(sext, sext_test1);
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
    
    emit_test!      ((vm) (sext sext_test1 sext_test1_v1 Int,Int,EQ (sext_sig, i8(-1i8 as u64), i64(-1i64 as u64))));
    
    vm
}

#[test]
fn test_add_9f() {
    build_and_run_test!(add_9f, add_9f_test1);
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
    
    emit_test!      ((vm) (add_9f add_9f_test1 add_9f_test1_v1 Int,Int,EQ (add_9f_sig, u64(1u64), u64(0x1000000000u64))));
    
    vm
}
