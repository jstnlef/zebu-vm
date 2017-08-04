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
extern crate log;
extern crate libloading;

use self::mu::ast::types::*;
use self::mu::ast::ir::*;
use self::mu::ast::inst::*;
use self::mu::ast::op::*;
use self::mu::vm::*;
use self::mu::linkutils;
use mu::utils::LinkedHashMap;

#[test]
fn test_double_add() {
    let lib = linkutils::aot::compile_fnc("double_add", &double_add);

    unsafe {
        let double_add: libloading::Symbol<unsafe extern "C" fn(f64, f64) -> f64> =
            lib.get(b"double_add").unwrap();

        let double_add_1_1 = double_add(1f64, 1f64);
        println!("double_add(1, 1) = {}", double_add_1_1);
        assert!(double_add_1_1 == 2f64);
    }
}

fn double_add() -> VM {
    let vm = VM::new();

    typedef!        ((vm) double = mu_double);

    funcsig!        ((vm) double_add_sig = (double, double) -> (double));
    funcdecl!       ((vm) <double_add_sig> double_add);
    funcdef!        ((vm) <double_add_sig> double_add VERSION double_add_v1);

    // %entry(<@double> %a, <@double> %b):
    block!          ((vm, double_add_v1) blk_entry);
    ssa!            ((vm, double_add_v1) <double> a);
    ssa!            ((vm, double_add_v1) <double> b);

    // %r = FADD %a %b
    ssa!            ((vm, double_add_v1) <double> r);
    inst!           ((vm, double_add_v1) blk_entry_fadd:
        r = BINOP (BinOp::FAdd) a b
    );

    // RET %r
    inst!           ((vm, double_add_v1) blk_entry_ret:
        RET (r)
    );

    define_block!   ((vm, double_add_v1) blk_entry(a, b) {
        blk_entry_fadd,
        blk_entry_ret
    });

    define_func_ver!((vm) double_add_v1(entry: blk_entry) {
        blk_entry
    });

    vm
}

#[test]
fn test_float_add() {
    let lib = linkutils::aot::compile_fnc("float_add", &float_add);

    unsafe {
        let float_add: libloading::Symbol<unsafe extern "C" fn(f32, f32) -> f32> =
            lib.get(b"float_add").unwrap();

        let float_add_1_1 = float_add(1f32, 1f32);
        println!("float_add(1, 1) = {}", float_add_1_1);
        assert!(float_add_1_1 == 2f32);
    }
}

fn float_add() -> VM {
    let vm = VM::new();

    typedef!        ((vm) float = mu_float);

    funcsig!        ((vm) float_add_sig = (float, float) -> (float));
    funcdecl!       ((vm) <float_add_sig> float_add);
    funcdef!        ((vm) <float_add_sig> float_add VERSION float_add_v1);

    // %entry(<@float> %a, <@float> %b):
    block!          ((vm, float_add_v1) blk_entry);
    ssa!            ((vm, float_add_v1) <float> a);
    ssa!            ((vm, float_add_v1) <float> b);

    // %r = FADD %a %b
    ssa!            ((vm, float_add_v1) <float> r);
    inst!           ((vm, float_add_v1) blk_entry_fadd:
        r = BINOP (BinOp::FAdd) a b
    );

    // RET %r
    inst!           ((vm, float_add_v1) blk_entry_ret:
        RET (r)
    );

    define_block!   ((vm, float_add_v1) blk_entry(a, b) {
        blk_entry_fadd,
        blk_entry_ret
    });

    define_func_ver!((vm) float_add_v1(entry: blk_entry) {
        blk_entry
    });

    vm
}

#[test]
fn test_fp_ogt_branch() {
    let lib = linkutils::aot::compile_fnc("fp_ogt_branch", &fp_ogt_branch);

    unsafe {
        let fp_ogt: libloading::Symbol<unsafe extern "C" fn(f64, f64) -> u32> =
            lib.get(b"fp_ogt_branch").unwrap();

        let res = fp_ogt(-1f64, 0f64);
        println!("fp_ogt(-1, 0) = {}", res);
        assert!(res == 0);

        let res = fp_ogt(0f64, -1f64);
        println!("fp_ogt(0, -1) = {}", res);
        assert!(res == 1);

        let res = fp_ogt(-1f64, -1f64);
        println!("fp_ogt(-1, -1) = {}", res);
        assert!(res == 0);

        let res = fp_ogt(-1f64, -2f64);
        println!("fp_ogt(-1, -2) = {}", res);
        assert!(res == 1);

        let res = fp_ogt(-2f64, -1f64);
        println!("fp_ogt(-2, -1) = {}", res);
        assert!(res == 0);

        let res = fp_ogt(1f64, 2f64);
        println!("fp_ogt(1, 2) = {}", res);
        assert!(res == 0);

        let res = fp_ogt(2f64, 1f64);
        println!("fp_ogt(2, 1) = {}", res);
        assert!(res == 1);
    }
}

fn fp_ogt_branch() -> VM {
    let vm = VM::new();

    typedef!    ((vm) double = mu_double);
    typedef!    ((vm) int32  = mu_int(32));
    typedef!    ((vm) int1   = mu_int(1));

    constdef!   ((vm) <int32> int32_0 = Constant::Int(0));
    constdef!   ((vm) <int32> int32_1 = Constant::Int(1));

    funcsig!    ((vm) sig = (double, double) -> (int32));
    funcdecl!   ((vm) <sig> fp_ogt_branch);
    funcdef!    ((vm) <sig> fp_ogt_branch VERSION fp_ogt_branch_v1);

    // blk entry
    block!      ((vm, fp_ogt_branch_v1) blk_entry);
    ssa!        ((vm, fp_ogt_branch_v1) <double> a);
    ssa!        ((vm, fp_ogt_branch_v1) <double> b);

    ssa!        ((vm, fp_ogt_branch_v1) <int1> cond);
    inst!       ((vm, fp_ogt_branch_v1) blk_entry_cmp:
        cond = CMPOP (CmpOp::FOGT) a b
    );

    block!      ((vm, fp_ogt_branch_v1) blk_ret1);
    consta!     ((vm, fp_ogt_branch_v1) int32_1_local = int32_1);
    block!      ((vm, fp_ogt_branch_v1) blk_ret0);
    consta!     ((vm, fp_ogt_branch_v1) int32_0_local = int32_0);

    inst!       ((vm, fp_ogt_branch_v1) blk_entry_branch:
        BRANCH2 (cond, int32_1_local, int32_0_local)
            IF (OP 0)
            THEN blk_ret1 (vec![1]) WITH 0.6f32,
            ELSE blk_ret0 (vec![2])
    );

    define_block! ((vm, fp_ogt_branch_v1) blk_entry(a, b){
        blk_entry_cmp, blk_entry_branch
    });

    // blk_ret1
    ssa!        ((vm, fp_ogt_branch_v1) <int32> blk_ret1_res);
    inst!       ((vm, fp_ogt_branch_v1) blk_ret1_inst:
        RET (blk_ret1_res)
    );

    define_block! ((vm, fp_ogt_branch_v1) blk_ret1(blk_ret1_res){
        blk_ret1_inst
    });

    // blk_ret0
    ssa!        ((vm, fp_ogt_branch_v1) <int32> blk_ret0_res);
    inst!       ((vm, fp_ogt_branch_v1) blk_ret0_inst:
        RET (blk_ret0_res)
    );

    define_block! ((vm, fp_ogt_branch_v1) blk_ret0(blk_ret0_res){
        blk_ret0_inst
    });

    define_func_ver!((vm) fp_ogt_branch_v1 (entry: blk_entry) {
        blk_entry, blk_ret1, blk_ret0
    });

    vm
}

#[test]
fn test_sitofp() {
    let lib = linkutils::aot::compile_fnc("sitofp", &sitofp);

    unsafe {
        let sitofp: libloading::Symbol<unsafe extern "C" fn(i64) -> f64> =
            lib.get(b"sitofp").unwrap();

        let res = sitofp(-1i64);
        println!("sitofp(-1) = {}", res);
        assert!(res == -1f64);

        let res = sitofp(0i64);
        println!("sitofp(0) = {}", res);
        assert!(res == 0f64);

        let res = sitofp(1i64);
        println!("sitofp(1) = {}", res);
        assert!(res == 1f64);
    }
}

fn sitofp() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int64 = mu_int(64));
    typedef!    ((vm) double = mu_double);

    funcsig!    ((vm) sig = (int64) -> (double));
    funcdecl!   ((vm) <sig> sitofp);
    funcdef!    ((vm) <sig> sitofp VERSION sitofp_v1);

    // blk entry
    block!      ((vm, sitofp_v1) blk_entry);
    ssa!        ((vm, sitofp_v1) <int64> x);

    ssa!        ((vm, sitofp_v1) <double> res);
    inst!       ((vm, sitofp_v1) blk_entry_conv:
        res = CONVOP (ConvOp::SITOFP) <int64 double> x
    );

    inst!       ((vm, sitofp_v1) blk_entry_ret:
        RET (res)
    );

    define_block!((vm, sitofp_v1) blk_entry(x){
        blk_entry_conv, blk_entry_ret
    });

    define_func_ver!((vm) sitofp_v1 (entry: blk_entry) {blk_entry});

    vm
}

#[test]
fn test_ui64tofp() {
    let lib = linkutils::aot::compile_fnc("ui64tofp", &ui64tofp);

    unsafe {
        let ui64tofp: libloading::Symbol<unsafe extern "C" fn(u64) -> f64> =
            lib.get(b"ui64tofp").unwrap();

        let res = ui64tofp(0u64);
        println!("ui64tofp(0) = {}", res);
        assert!(res == 0f64);

        let res = ui64tofp(1u64);
        println!("ui64tofp(1) = {}", res);
        assert!(res == 1f64);
    }
}

fn ui64tofp() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int64 = mu_int(64));
    typedef!    ((vm) double = mu_double);

    funcsig!    ((vm) sig = (int64) -> (double));
    funcdecl!   ((vm) <sig> ui64tofp);
    funcdef!    ((vm) <sig> ui64tofp VERSION ui64tofp_v1);

    // blk entry
    block!      ((vm, ui64tofp_v1) blk_entry);
    ssa!        ((vm, ui64tofp_v1) <int64> x);

    ssa!        ((vm, ui64tofp_v1) <double> res);
    inst!       ((vm, ui64tofp_v1) blk_entry_conv:
        res = CONVOP (ConvOp::UITOFP) <int64 double> x
    );

    inst!       ((vm, ui64tofp_v1) blk_entry_ret:
        RET (res)
    );

    define_block!((vm, ui64tofp_v1) blk_entry(x){
        blk_entry_conv, blk_entry_ret
    });

    define_func_ver!((vm) ui64tofp_v1 (entry: blk_entry) {blk_entry});

    vm
}

#[test]
fn test_ui64tofp_float() {
    let lib = linkutils::aot::compile_fnc("ui64tofp_float", &ui64tofp_float);

    unsafe {
        let ui64tofp_float: libloading::Symbol<unsafe extern "C" fn(u64) -> f32> =
            lib.get(b"ui64tofp_float").unwrap();

        let res = ui64tofp_float(0u64);
        println!("ui64tofp_float(0) = {}", res);
        assert!(res == 0f32);

        let res = ui64tofp_float(1u64);
        println!("ui64tofp_float(1) = {}", res);
        assert!(res == 1f32);
    }
}

fn ui64tofp_float() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int64 = mu_int(64));
    typedef!    ((vm) float = mu_float);

    funcsig!    ((vm) sig = (int64) -> (float));
    funcdecl!   ((vm) <sig> ui64tofp_float);
    funcdef!    ((vm) <sig> ui64tofp_float VERSION ui64tofp_float_v1);

    // blk entry
    block!      ((vm, ui64tofp_float_v1) blk_entry);
    ssa!        ((vm, ui64tofp_float_v1) <int64> x);

    ssa!        ((vm, ui64tofp_float_v1) <float> res);
    inst!       ((vm, ui64tofp_float_v1) blk_entry_conv:
        res = CONVOP (ConvOp::UITOFP) <int64 float> x
    );

    inst!       ((vm, ui64tofp_float_v1) blk_entry_ret:
        RET (res)
    );

    define_block!((vm, ui64tofp_float_v1) blk_entry(x){
        blk_entry_conv, blk_entry_ret
    });

    define_func_ver!((vm) ui64tofp_float_v1 (entry: blk_entry) {blk_entry});

    vm
}

#[test]
fn test_ui32tofp() {
    let lib = linkutils::aot::compile_fnc("ui32tofp", &ui32tofp);

    unsafe {
        let ui32tofp: libloading::Symbol<unsafe extern "C" fn(u32) -> f64> =
            lib.get(b"ui32tofp").unwrap();

        let res = ui32tofp(0u32);
        println!("ui32tofp(0) = {}", res);
        assert!(res == 0f64);

        let res = ui32tofp(1u32);
        println!("ui32tofp(1) = {}", res);
        assert!(res == 1f64);
    }
}

fn ui32tofp() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int32 = mu_int(32));
    typedef!    ((vm) double = mu_double);

    funcsig!    ((vm) sig = (int32) -> (double));
    funcdecl!   ((vm) <sig> ui32tofp);
    funcdef!    ((vm) <sig> ui32tofp VERSION ui32tofp_v1);

    // blk entry
    block!      ((vm, ui32tofp_v1) blk_entry);
    ssa!        ((vm, ui32tofp_v1) <int32> x);

    ssa!        ((vm, ui32tofp_v1) <double> res);
    inst!       ((vm, ui32tofp_v1) blk_entry_conv:
        res = CONVOP (ConvOp::UITOFP) <int32 double> x
    );

    inst!       ((vm, ui32tofp_v1) blk_entry_ret:
        RET (res)
    );

    define_block!((vm, ui32tofp_v1) blk_entry(x){
        blk_entry_conv, blk_entry_ret
    });

    define_func_ver!((vm) ui32tofp_v1 (entry: blk_entry) {blk_entry});

    vm
}

#[test]
fn test_ui16tofp() {
    let lib = linkutils::aot::compile_fnc("ui16tofp", &ui16tofp);

    unsafe {
        let ui16tofp: libloading::Symbol<unsafe extern "C" fn(u16) -> f64> =
            lib.get(b"ui16tofp").unwrap();

        let res = ui16tofp(0u16);
        println!("ui16tofp(0) = {}", res);
        assert!(res == 0f64);

        let res = ui16tofp(1u16);
        println!("ui16tofp(1) = {}", res);
        assert!(res == 1f64);
    }
}

fn ui16tofp() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int16 = mu_int(16));
    typedef!    ((vm) double = mu_double);

    funcsig!    ((vm) sig = (int16) -> (double));
    funcdecl!   ((vm) <sig> ui16tofp);
    funcdef!    ((vm) <sig> ui16tofp VERSION ui16tofp_v1);

    // blk entry
    block!      ((vm, ui16tofp_v1) blk_entry);
    ssa!        ((vm, ui16tofp_v1) <int16> x);

    ssa!        ((vm, ui16tofp_v1) <double> res);
    inst!       ((vm, ui16tofp_v1) blk_entry_conv:
        res = CONVOP (ConvOp::UITOFP) <int16 double> x
    );

    inst!       ((vm, ui16tofp_v1) blk_entry_ret:
        RET (res)
    );

    define_block!((vm, ui16tofp_v1) blk_entry(x){
        blk_entry_conv, blk_entry_ret
    });

    define_func_ver!((vm) ui16tofp_v1 (entry: blk_entry) {blk_entry});

    vm
}

#[test]
fn test_ui8tofp() {
    let lib = linkutils::aot::compile_fnc("ui8tofp", &ui8tofp);

    unsafe {
        let ui8tofp: libloading::Symbol<unsafe extern "C" fn(u8) -> f64> =
            lib.get(b"ui8tofp").unwrap();

        let res = ui8tofp(0u8);
        println!("ui8tofp(0) = {}", res);
        assert!(res == 0f64);

        let res = ui8tofp(1u8);
        println!("ui8tofp(1) = {}", res);
        assert!(res == 1f64);
    }
}

fn ui8tofp() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int8 = mu_int(8));
    typedef!    ((vm) double = mu_double);

    funcsig!    ((vm) sig = (int8) -> (double));
    funcdecl!   ((vm) <sig> ui8tofp);
    funcdef!    ((vm) <sig> ui8tofp VERSION ui8tofp_v1);

    // blk entry
    block!      ((vm, ui8tofp_v1) blk_entry);
    ssa!        ((vm, ui8tofp_v1) <int8> x);

    ssa!        ((vm, ui8tofp_v1) <double> res);
    inst!       ((vm, ui8tofp_v1) blk_entry_conv:
        res = CONVOP (ConvOp::UITOFP) <int8 double> x
    );

    inst!       ((vm, ui8tofp_v1) blk_entry_ret:
        RET (res)
    );

    define_block!((vm, ui8tofp_v1) blk_entry(x){
        blk_entry_conv, blk_entry_ret
    });

    define_func_ver!((vm) ui8tofp_v1 (entry: blk_entry) {blk_entry});

    vm
}

#[test]
fn test_fptoui64() {
    let lib = linkutils::aot::compile_fnc("fptoui64", &fptoui64);

    unsafe {
        let fptoui64: libloading::Symbol<unsafe extern "C" fn(f64) -> u64> =
            lib.get(b"fptoui64").unwrap();

        let res = fptoui64(0f64);
        println!("fptoui64(0) = {}", res);
        assert!(res == 0u64);

        let res = fptoui64(1f64);
        println!("fptoui64(1) = {}", res);
        assert!(res == 1u64);
    }
}

fn fptoui64() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int64 = mu_int(64));
    typedef!    ((vm) double = mu_double);

    funcsig!    ((vm) sig = (double) -> (int64));
    funcdecl!   ((vm) <sig> fptoui64);
    funcdef!    ((vm) <sig> fptoui64 VERSION fptoui64_v1);

    // blk entry
    block!      ((vm, fptoui64_v1) blk_entry);
    ssa!        ((vm, fptoui64_v1) <double> x);

    ssa!        ((vm, fptoui64_v1) <int64> res);
    inst!       ((vm, fptoui64_v1) blk_entry_conv:
        res = CONVOP (ConvOp::FPTOUI) <double int64> x
    );

    inst!       ((vm, fptoui64_v1) blk_entry_ret:
        RET (res)
    );

    define_block!((vm, fptoui64_v1) blk_entry(x){
        blk_entry_conv, blk_entry_ret
    });

    define_func_ver!((vm) fptoui64_v1 (entry: blk_entry) {blk_entry});

    vm
}

#[test]
fn test_fptoui32() {
    let lib = linkutils::aot::compile_fnc("fptoui32", &fptoui32);

    unsafe {
        let fptoui32: libloading::Symbol<unsafe extern "C" fn(f64) -> u32> =
            lib.get(b"fptoui32").unwrap();

        let res = fptoui32(0f64);
        println!("fptoui32(0) = {}", res);
        assert!(res == 0u32);

        let res = fptoui32(1f64);
        println!("fptoui32(1) = {}", res);
        assert!(res == 1u32);
    }
}

fn fptoui32() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int32 = mu_int(32));
    typedef!    ((vm) double = mu_double);

    funcsig!    ((vm) sig = (double) -> (int32));
    funcdecl!   ((vm) <sig> fptoui32);
    funcdef!    ((vm) <sig> fptoui32 VERSION fptoui32_v1);

    // blk entry
    block!      ((vm, fptoui32_v1) blk_entry);
    ssa!        ((vm, fptoui32_v1) <double> x);

    ssa!        ((vm, fptoui32_v1) <int32> res);
    inst!       ((vm, fptoui32_v1) blk_entry_conv:
        res = CONVOP (ConvOp::FPTOUI) <double int32> x
    );

    inst!       ((vm, fptoui32_v1) blk_entry_ret:
        RET (res)
    );

    define_block!((vm, fptoui32_v1) blk_entry(x){
        blk_entry_conv, blk_entry_ret
    });

    define_func_ver!((vm) fptoui32_v1 (entry: blk_entry) {blk_entry});

    vm
}

#[test]
fn test_fptoui16() {
    let lib = linkutils::aot::compile_fnc("fptoui16", &fptoui16);

    unsafe {
        let fptoui16: libloading::Symbol<unsafe extern "C" fn(f64) -> u16> =
            lib.get(b"fptoui16").unwrap();

        let res = fptoui16(0f64);
        println!("fptoui16(0) = {}", res);
        assert!(res == 0u16);

        let res = fptoui16(1f64);
        println!("fptoui16(1) = {}", res);
        assert!(res == 1u16);
    }
}

fn fptoui16() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int16 = mu_int(16));
    typedef!    ((vm) double = mu_double);

    funcsig!    ((vm) sig = (double) -> (int16));
    funcdecl!   ((vm) <sig> fptoui16);
    funcdef!    ((vm) <sig> fptoui16 VERSION fptoui16_v1);

    // blk entry
    block!      ((vm, fptoui16_v1) blk_entry);
    ssa!        ((vm, fptoui16_v1) <double> x);

    ssa!        ((vm, fptoui16_v1) <int16> res);
    inst!       ((vm, fptoui16_v1) blk_entry_conv:
        res = CONVOP (ConvOp::FPTOUI) <double int16> x
    );

    inst!       ((vm, fptoui16_v1) blk_entry_ret:
        RET (res)
    );

    define_block!((vm, fptoui16_v1) blk_entry(x){
        blk_entry_conv, blk_entry_ret
    });

    define_func_ver!((vm) fptoui16_v1 (entry: blk_entry) {blk_entry});

    vm
}

#[test]
fn test_fptoui8() {
    let lib = linkutils::aot::compile_fnc("fptoui8", &fptoui8);

    unsafe {
        let fptoui8: libloading::Symbol<unsafe extern "C" fn(f64) -> u8> =
            lib.get(b"fptoui8").unwrap();

        let res = fptoui8(0f64);
        println!("fptoui8(0) = {}", res);
        assert!(res == 0u8);

        let res = fptoui8(1f64);
        println!("fptoui8(1) = {}", res);
        assert!(res == 1u8);
    }
}

fn fptoui8() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int8 = mu_int(8));
    typedef!    ((vm) double = mu_double);

    funcsig!    ((vm) sig = (double) -> (int8));
    funcdecl!   ((vm) <sig> fptoui8);
    funcdef!    ((vm) <sig> fptoui8 VERSION fptoui8_v1);

    // blk entry
    block!      ((vm, fptoui8_v1) blk_entry);
    ssa!        ((vm, fptoui8_v1) <double> x);

    ssa!        ((vm, fptoui8_v1) <int8> res);
    inst!       ((vm, fptoui8_v1) blk_entry_conv:
        res = CONVOP (ConvOp::FPTOUI) <double int8> x
    );

    inst!       ((vm, fptoui8_v1) blk_entry_ret:
        RET (res)
    );

    define_block!((vm, fptoui8_v1) blk_entry(x){
        blk_entry_conv, blk_entry_ret
    });

    define_func_ver!((vm) fptoui8_v1 (entry: blk_entry) {blk_entry});

    vm
}

#[test]
fn test_fp_arraysum() {
    use std::os::raw::c_double;

    let lib = linkutils::aot::compile_fnc("fp_arraysum", &fp_arraysum);

    unsafe {
        let fp_arraysum: libloading::Symbol<unsafe extern "C" fn(*const c_double, u64) -> f64> =
            lib.get(b"fp_arraysum").unwrap();

        let array: [f64; 10] = [
            0f64,
            0.1f64,
            0.2f64,
            0.3f64,
            0.4f64,
            0.5f64,
            0.6f64,
            0.7f64,
            0.8f64,
            0.9f64,
        ];
        let c_array = array.as_ptr() as *const c_double;

        let res = fp_arraysum(c_array, 10);
        println!("fp_arraysum(array, 10) = {}", res);
        assert!(res == 4.5f64);
    }
}

fn fp_arraysum() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int64  = mu_int(64));
    typedef!    ((vm) int1   = mu_int(1));
    typedef!    ((vm) double = mu_double);
    typedef!    ((vm) hybrid = mu_hybrid()(double));
    typedef!    ((vm) uptr_hybrid = mu_uptr(hybrid));
    typedef!    ((vm) uptr_double = mu_uptr(double));

    constdef!   ((vm) <int64> int64_0   = Constant::Int(0));
    constdef!   ((vm) <int64> int64_1   = Constant::Int(1));
    constdef!   ((vm) <double> double_0 = Constant::Double(0f64));

    funcsig!    ((vm) sig = (uptr_hybrid, int64) -> (double));
    funcdecl!   ((vm) <sig> fp_arraysum);
    funcdef!    ((vm) <sig> fp_arraysum VERSION fp_arraysum_v1);

    // blk entry
    block!      ((vm, fp_arraysum_v1) blk_entry);
    ssa!        ((vm, fp_arraysum_v1) <uptr_hybrid> blk_entry_arr);
    ssa!        ((vm, fp_arraysum_v1) <int64> blk_entry_sz);

    block!      ((vm, fp_arraysum_v1) blk1);
    consta!     ((vm, fp_arraysum_v1) int64_0_local  = int64_0);
    consta!     ((vm, fp_arraysum_v1) int64_1_local  = int64_1);
    consta!     ((vm, fp_arraysum_v1) double_0_local = double_0);
    inst!       ((vm, fp_arraysum_v1) blk_entry_branch:
        BRANCH blk1 (blk_entry_arr, double_0_local, int64_0_local, blk_entry_sz)
    );

    define_block!   ((vm, fp_arraysum_v1) blk_entry(blk_entry_arr, blk_entry_sz) {
        blk_entry_branch
    });

    // blk1
    ssa!        ((vm, fp_arraysum_v1) <uptr_hybrid> blk1_arr);
    ssa!        ((vm, fp_arraysum_v1) <double> blk1_sum);
    ssa!        ((vm, fp_arraysum_v1) <int64> blk1_v1);
    ssa!        ((vm, fp_arraysum_v1) <int64> blk1_v2);

    ssa!        ((vm, fp_arraysum_v1) <int1> blk1_rtn);
    inst!       ((vm, fp_arraysum_v1) blk1_sge:
        blk1_rtn = CMPOP (CmpOp::SGE) blk1_v1 blk1_v2
    );

    block!      ((vm, fp_arraysum_v1) blk2);
    block!      ((vm, fp_arraysum_v1) blk3);
    inst!       ((vm, fp_arraysum_v1) blk1_branch2:
        BRANCH2 (blk1_rtn, blk1_sum, blk1_v2, blk1_v1, blk1_arr)
        IF (OP 0)
        THEN blk3 (vec![1]) WITH 0.2f32,
        ELSE blk2 (vec![2, 3, 4, 1])
    );

    define_block!   ((vm, fp_arraysum_v1) blk1(blk1_arr, blk1_sum, blk1_v1, blk1_v2) {
        blk1_sge, blk1_branch2
    });

    // blk2
    ssa!        ((vm, fp_arraysum_v1) <int64> blk2_v4);
    ssa!        ((vm, fp_arraysum_v1) <int64> blk2_next);
    ssa!        ((vm, fp_arraysum_v1) <uptr_hybrid> blk2_arr);
    ssa!        ((vm, fp_arraysum_v1) <double> blk2_sum);

    ssa!        ((vm, fp_arraysum_v1) <int64> blk2_v5);
    inst!       ((vm, fp_arraysum_v1) blk2_add:
        blk2_v5 = BINOP (BinOp::Add) blk2_next int64_1_local
    );

    ssa!        ((vm, fp_arraysum_v1) <uptr_double> blk2_rtn2);
    inst!       ((vm, fp_arraysum_v1) blk2_getvarpart:
        blk2_rtn2 = GETVARPARTIREF blk2_arr (is_ptr: true)
    );

    ssa!        ((vm, fp_arraysum_v1) <uptr_double> blk2_rtn3);
    inst!       ((vm, fp_arraysum_v1) blk2_shiftiref:
        blk2_rtn3 = SHIFTIREF blk2_rtn2 blk2_next (is_ptr: true)
    );

    ssa!        ((vm, fp_arraysum_v1) <double> blk2_v7);
    inst!       ((vm, fp_arraysum_v1) blk2_load:
        blk2_v7 = LOAD blk2_rtn3 (is_ptr: true, order: MemoryOrder::NotAtomic)
    );

    ssa!        ((vm, fp_arraysum_v1) <double> blk2_sum2);
    inst!       ((vm, fp_arraysum_v1) blk2_fadd:
        blk2_sum2 = BINOP (BinOp::FAdd) blk2_sum blk2_v7
    );

    inst!       ((vm, fp_arraysum_v1) blk2_branch:
        BRANCH blk1 (blk2_arr, blk2_sum2, blk2_v5, blk2_v4)
    );

    define_block!   ((vm, fp_arraysum_v1) blk2(blk2_v4, blk2_next, blk2_arr, blk2_sum) {
        blk2_add, blk2_getvarpart, blk2_shiftiref, blk2_load, blk2_fadd, blk2_branch
    });

    // blk3
    ssa!        ((vm, fp_arraysum_v1) <double> blk3_v8);
    inst!       ((vm, fp_arraysum_v1) blk3_ret:
        RET (blk3_v8)
    );

    define_block!   ((vm, fp_arraysum_v1) blk3(blk3_v8) {blk3_ret});

    define_func_ver!    ((vm) fp_arraysum_v1 (entry: blk_entry) {blk_entry, blk1, blk2, blk3});

    vm
}

#[test]
fn test_double_to_float() {
    let lib = linkutils::aot::compile_fnc("double_to_float", &double_to_float);

    unsafe {
        use std::f64;

        let double_to_float: libloading::Symbol<unsafe extern "C" fn(f64) -> f32> =
            lib.get(b"double_to_float").unwrap();

        let res = double_to_float(0f64);
        println!("double_fo_float(0) = {}", res);
        assert!(res == 0f32);

        let res = double_to_float(1f64);
        println!("double_fo_float(1) = {}", res);
        assert!(res == 1f32);

        let res = double_to_float(f64::MAX);
        println!("double_to_float(f64::MAX) = {}", res);
        assert!(res.is_infinite());
    }
}

fn double_to_float() -> VM {
    let vm = VM::new();

    typedef!    ((vm) double = mu_double);
    typedef!    ((vm) float = mu_float);

    funcsig!    ((vm) sig = (double) -> (float));
    funcdecl!   ((vm) <sig> double_to_float);
    funcdef!    ((vm) <sig> double_to_float VERSION double_to_float_v1);

    // blk entry
    block!      ((vm, double_to_float_v1) blk_entry);
    ssa!        ((vm, double_to_float_v1) <double> d);
    ssa!        ((vm, double_to_float_v1) <float> f);

    inst!       ((vm, double_to_float_v1) blk_entry_fptrunc:
        f = CONVOP (ConvOp::FPTRUNC) <double float> d
    );

    inst!       ((vm, double_to_float_v1) blk_entry_ret:
        RET (f)
    );

    define_block!((vm, double_to_float_v1) blk_entry(d) {
        blk_entry_fptrunc, blk_entry_ret
    });

    define_func_ver!((vm) double_to_float_v1 (entry: blk_entry) {
        blk_entry
    });

    vm
}

#[test]
fn test_float_to_double() {
    let lib = linkutils::aot::compile_fnc("float_to_double", &float_to_double);

    unsafe {
        let float_to_double: libloading::Symbol<unsafe extern "C" fn(f32) -> f64> =
            lib.get(b"float_to_double").unwrap();

        let res = float_to_double(0f32);
        println!("float_to_double(0) = {}", 0);
        assert!(res == 0f64);

        let res = float_to_double(1f32);
        println!("float_to_double(1) = {}", 0);
        assert!(res == 1f64);
    }
}

fn float_to_double() -> VM {
    let vm = VM::new();

    typedef!    ((vm) double = mu_double);
    typedef!    ((vm) float = mu_float);

    funcsig!    ((vm) sig = (float) -> (double));
    funcdecl!   ((vm) <sig> float_to_double);
    funcdef!    ((vm) <sig> float_to_double VERSION float_to_double_v1);

    // blk entry
    block!      ((vm, float_to_double_v1) blk_entry);
    ssa!        ((vm, float_to_double_v1) <double> d);
    ssa!        ((vm, float_to_double_v1) <float> f);

    inst!       ((vm, float_to_double_v1) blk_entry_fpext:
        d = CONVOP (ConvOp::FPEXT) <float double> f
    );

    inst!       ((vm, float_to_double_v1) blk_entry_ret:
        RET (d)
    );

    define_block!((vm, float_to_double_v1) blk_entry(f) {
        blk_entry_fpext, blk_entry_ret
    });

    define_func_ver!((vm) float_to_double_v1 (entry: blk_entry) {
        blk_entry
    });

    vm
}
