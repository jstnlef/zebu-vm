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

use mu::ast::types::*;
use mu::ast::ir::*;
use mu::ast::inst::*;
use mu::ast::op::*;
use mu::vm::*;
use mu::compiler::*;
use mu::utils::LinkedHashMap;
use mu::utils::mem::memsec;

use std::sync::Arc;
use mu::linkutils::aot;
use mu::linkutils;

use test_compiler::test_call::gen_ccall_exit;

#[test]
fn test_store_seqcst() {
    let lib = linkutils::aot::compile_fnc("store_seqcst", &store_seqcst);

    unsafe {
        let ptr: *mut u64 = match memsec::malloc(8) {
            Some(ptr) => ptr,
            None => panic!("failed to allocate memory for test"),
        };

        let store_seqcst: libloading::Symbol<unsafe extern "C" fn(*mut u64, u64)> =
            lib.get(b"store_seqcst").unwrap();

        store_seqcst(ptr, 42);
        let load_val = *ptr;
        println!("result = {}", load_val);
        assert!(load_val == 42);
    }
}

fn store_seqcst() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int64      = mu_int(64));
    typedef!    ((vm) iref_int64 = mu_iref(int64));

    funcsig!    ((vm) sig = (iref_int64, int64) -> ());
    funcdecl!   ((vm) <sig> store_seqcst);
    funcdef!    ((vm) <sig> store_seqcst VERSION store_seqcst_v1);

    block!      ((vm, store_seqcst_v1) blk_entry);
    ssa!        ((vm, store_seqcst_v1) <iref_int64> loc);
    ssa!        ((vm, store_seqcst_v1) <int64> val);

    inst!       ((vm, store_seqcst_v1) blk_entry_store:
        STORE loc val (is_ptr: false, order: MemoryOrder::SeqCst)
    );

    inst!       ((vm, store_seqcst_v1) blk_entry_ret:
        RET
    );

    define_block!((vm, store_seqcst_v1) blk_entry(loc, val) {
        blk_entry_store,
        blk_entry_ret
    });

    define_func_ver!((vm) store_seqcst_v1 (entry: blk_entry) {
        blk_entry
    });

    vm
}

#[repr(C)]
struct Foo(i8, i8, i8);

#[test]
#[allow(unused_variables)]
fn test_write_int8_val() {
    let lib = linkutils::aot::compile_fnc("write_int8", &write_int8);

    unsafe {
        let ptr: *mut Foo = Box::into_raw(Box::new(Foo(1, 2, 3)));
        let a = (*ptr).0;
        let b = (*ptr).1;
        let c = (*ptr).2;
        println!("foo.0 = {}", (*ptr).0);
        println!("foo.1 = {}", (*ptr).1);
        println!("foo.2 = {}", (*ptr).2);

        let write_int8: libloading::Symbol<unsafe extern "C" fn(*mut Foo, i8)> =
            lib.get(b"write_int8").unwrap();

        write_int8(ptr, 42);

        println!("foo.0 = {}", (*ptr).0);
        println!("foo.1 = {}", (*ptr).1);
        println!("foo.2 = {}", (*ptr).2);

        assert_eq!((*ptr).0, a);
        assert_eq!((*ptr).1, 42);
        assert_eq!((*ptr).2, c);
    }
}

fn write_int8() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int8 = mu_int(8));
    typedef!    ((vm) foo  = mu_struct(int8, int8, int8));
    typedef!    ((vm) ref_foo   = mu_ref(foo));
    typedef!    ((vm) iref_foo  = mu_iref(foo));
    typedef!    ((vm) iref_int8 = mu_iref(int8));

    funcsig!    ((vm) write_int8_sig = (ref_foo, int8) -> ());
    funcdecl!   ((vm) <write_int8_sig> write_int8);
    funcdef!    ((vm) <write_int8_sig> write_int8 VERSION write_int8_v1);

    block!      ((vm, write_int8_v1) blk_entry);
    ssa!        ((vm, write_int8_v1) <ref_foo> refx);
    ssa!        ((vm, write_int8_v1) <int8> val);

    ssa!        ((vm, write_int8_v1) <iref_foo> irefx);
    inst!       ((vm, write_int8_v1) blk_entry_getiref:
        irefx = GETIREF refx
    );

    ssa!        ((vm, write_int8_v1) <iref_int8> iref_field1);
    inst!       ((vm, write_int8_v1) blk_entry_getfieldiref:
        iref_field1 = GETFIELDIREF irefx (is_ptr: false, index: 1)
    );

    inst!       ((vm, write_int8_v1) blk_entry_write:
        STORE iref_field1 val (is_ptr: false, order: MemoryOrder::Relaxed)
    );

    inst!       ((vm, write_int8_v1) blk_entry_ret:
        RET
    );

    define_block!((vm, write_int8_v1) blk_entry(refx, val) {
        blk_entry_getiref, blk_entry_getfieldiref, blk_entry_write, blk_entry_ret
    });

    define_func_ver!((vm) write_int8_v1 (entry: blk_entry) {blk_entry});

    vm
}

#[allow(unused_variables)]
#[test]
fn test_write_int8_const() {
    let lib = linkutils::aot::compile_fnc("write_int8_const", &write_int8_const);

    unsafe {
        let ptr: *mut Foo = Box::into_raw(Box::new(Foo(1, 2, 3)));
        let a = (*ptr).0;
        let b = (*ptr).1;
        let c = (*ptr).2;
        println!("foo.0 = {}", (*ptr).0);
        println!("foo.1 = {}", (*ptr).1);
        println!("foo.2 = {}", (*ptr).2);

        let write_int8: libloading::Symbol<unsafe extern "C" fn(*mut Foo)> =
            lib.get(b"write_int8_const").unwrap();

        write_int8(ptr);

        println!("foo.0 = {}", (*ptr).0);
        println!("foo.1 = {}", (*ptr).1);
        println!("foo.2 = {}", (*ptr).2);

        assert_eq!((*ptr).0, a);
        assert_eq!((*ptr).1, 42);
        assert_eq!((*ptr).2, c);
    }
}

fn write_int8_const() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int8 = mu_int(8));
    typedef!    ((vm) foo  = mu_struct(int8, int8, int8));
    typedef!    ((vm) ref_foo   = mu_ref(foo));
    typedef!    ((vm) iref_foo  = mu_iref(foo));
    typedef!    ((vm) iref_int8 = mu_iref(int8));

    constdef!   ((vm) <int8> int8_42 = Constant::Int(42));

    funcsig!    ((vm) write_int8_const_sig = (ref_foo) -> ());
    funcdecl!   ((vm) <write_int8_const_sig> write_int8_const);
    funcdef!    ((vm) <write_int8_const_sig> write_int8_const VERSION write_int8_const_v1);

    block!      ((vm, write_int8_const_v1) blk_entry);
    ssa!        ((vm, write_int8_const_v1) <ref_foo> refx);

    ssa!        ((vm, write_int8_const_v1) <iref_foo> irefx);
    inst!       ((vm, write_int8_const_v1) blk_entry_getiref:
        irefx = GETIREF refx
    );

    ssa!        ((vm, write_int8_const_v1) <iref_int8> iref_field1);
    inst!       ((vm, write_int8_const_v1) blk_entry_getfieldiref:
        iref_field1 = GETFIELDIREF irefx (is_ptr: false, index: 1)
    );

    consta!     ((vm, write_int8_const_v1) int8_42_local = int8_42);
    inst!       ((vm, write_int8_const_v1) blk_entry_write:
        STORE iref_field1 int8_42_local (is_ptr: false, order: MemoryOrder::Relaxed)
    );

    inst!       ((vm, write_int8_const_v1) blk_entry_ret:
        RET
    );

    define_block!((vm, write_int8_const_v1) blk_entry(refx) {
        blk_entry_getiref, blk_entry_getfieldiref, blk_entry_write, blk_entry_ret
    });

    define_func_ver!((vm) write_int8_const_v1 (entry: blk_entry) {blk_entry});

    vm
}

#[test]
fn test_get_field_iref1() {
    let lib = linkutils::aot::compile_fnc("get_field_iref1", &get_field_iref1);

    unsafe {
        let get_field_iref1: libloading::Symbol<unsafe extern "C" fn(u64) -> u64> =
            lib.get(b"get_field_iref1").unwrap();

        let addr = 0x10000000;
        let res = get_field_iref1(addr);
        println!("get_field_iref1({}) = {}", addr, res);

        assert!(addr + 8 == res);
    }
}

fn get_field_iref1() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int64         = mu_int(64));
    typedef!    ((vm) ref_int64     = mu_ref(int64));
    typedef!    ((vm) iref_int64    = mu_iref(int64));
    typedef!    ((vm) mystruct      = mu_struct(int64, int64, ref_int64));
    typedef!    ((vm) ref_mystruct  = mu_ref(mystruct));
    typedef!    ((vm) iref_mystruct = mu_iref(mystruct));

    funcsig!    ((vm) sig = (ref_mystruct) -> (iref_int64));
    funcdecl!   ((vm) <sig> get_field_iref1);

    funcdef!    ((vm) <sig> get_field_iref1 VERSION get_field_iref1_v1);

    block!      ((vm, get_field_iref1_v1) blk_entry);
    ssa!        ((vm, get_field_iref1_v1) <ref_mystruct> x);

    ssa!        ((vm, get_field_iref1_v1) <iref_mystruct> x_);
    inst!       ((vm, get_field_iref1_v1) blk_entry_get_iref:
        x_ = GETIREF x
    );

    ssa!        ((vm, get_field_iref1_v1) <iref_int64> ret);
    inst!       ((vm, get_field_iref1_v1) blk_entry_get_field_iref1:
        ret = GETFIELDIREF x_ (is_ptr: false, index: 1)
    );

    inst!       ((vm, get_field_iref1_v1) blk_entry_ret:
        RET (ret)
    );


    define_block!   ((vm, get_field_iref1_v1) blk_entry(x) {
        blk_entry_get_iref, blk_entry_get_field_iref1, blk_entry_ret
    });

    define_func_ver!((vm) get_field_iref1_v1 (entry: blk_entry) {blk_entry});

    vm
}

#[test]
fn test_get_iref() {
    let lib = linkutils::aot::compile_fnc("get_iref", &get_iref);

    unsafe {
        let get_iref: libloading::Symbol<unsafe extern "C" fn(u64) -> u64> =
            lib.get(b"get_iref").unwrap();

        let addr = 0x10000000;
        let res = get_iref(addr);
        println!("get_iref({}) = {}", addr, res);

        assert!(addr == res);
    }
}

fn get_iref() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int64      = mu_int(64));
    typedef!    ((vm) ref_int64  = mu_ref(int64));
    typedef!    ((vm) iref_int64 = mu_iref(int64));

    funcsig!    ((vm) sig = (ref_int64) -> (iref_int64));
    funcdecl!   ((vm) <sig> get_iref);

    funcdef!    ((vm) <sig> get_iref VERSION get_iref_v1);

    block!      ((vm, get_iref_v1) blk_entry);
    ssa!        ((vm, get_iref_v1) <ref_int64> x);

    ssa!        ((vm, get_iref_v1) <iref_int64> ret);
    inst!       ((vm, get_iref_v1) blk_entry_get_iref:
        ret = GETIREF x
    );

    inst!       ((vm, get_iref_v1) blk_entry_ret:
        RET (ret)
    );

    define_block!   ((vm, get_iref_v1) blk_entry(x) {
        blk_entry_get_iref, blk_entry_ret
    });

    define_func_ver!((vm) get_iref_v1 (entry: blk_entry) {blk_entry});

    vm
}

#[test]
fn test_struct() {
    VM::start_logging_trace();

    let vm = Arc::new(struct_insts_macro());

    let compiler = Compiler::new(CompilerPolicy::default(), &vm);

    let func_id = vm.id_of("struct_insts");
    {
        let funcs = vm.funcs().read().unwrap();
        let func = funcs.get(&func_id).unwrap().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();
        let mut func_ver = func_vers
            .get(&func.cur_ver.unwrap())
            .unwrap()
            .write()
            .unwrap();

        compiler.compile(&mut func_ver);
    }

    vm.set_primordial_thread(func_id, true, vec![]);
    backend::emit_context(&vm);

    let executable =
        aot::link_primordial(vec!["struct_insts".to_string()], "struct_insts_test", &vm);
    let output = linkutils::exec_path_nocheck(executable);

    assert!(output.status.code().is_some());

    let ret_code = output.status.code().unwrap();
    println!("return code: {}", ret_code);
    assert!(ret_code == 1);
}

// this IR construction function is a replicate of struct_insts() with macros
pub fn struct_insts_macro() -> VM {
    let vm = VM::new();

    typedef! ((vm) int64        = mu_int(64));
    typedef! ((vm) struct_point = mu_struct(int64, int64));
    typedef! ((vm) ref_point    = mu_ref(struct_point));
    typedef! ((vm) iref_point   = mu_iref(struct_point));
    typedef! ((vm) iref_int64   = mu_iref(int64));

    constdef!((vm) <int64> int64_0 = Constant::Int(0));
    constdef!((vm) <int64> int64_1 = Constant::Int(1));

    funcsig! ((vm) noparam_noret_sig = () -> ());
    funcdecl!((vm) <noparam_noret_sig> struct_insts);

    funcdef! ((vm) <noparam_noret_sig> struct_insts VERSION struct_insts_v1);

    // blk entry
    block!  ((vm, struct_insts_v1) blk_entry);

    ssa!    ((vm, struct_insts_v1) <ref_point> blk_entry_a);
    inst!   ((vm, struct_insts_v1) blk_entry_inst0:
                blk_entry_a = NEW <struct_point>
    );

    ssa!    ((vm, struct_insts_v1) <iref_point> blk_entry_iref_a);
    inst!   ((vm, struct_insts_v1) blk_entry_inst1:
                blk_entry_iref_a = GETIREF blk_entry_a
    );

    ssa!    ((vm, struct_insts_v1) <iref_int64> blk_entry_iref_x);
    inst!   ((vm, struct_insts_v1) blk_entry_inst2:
                blk_entry_iref_x = GETFIELDIREF blk_entry_iref_a (is_ptr: false, index: 0)
    );

    consta! ((vm, struct_insts_v1) int64_1_local = int64_1);
    inst!   ((vm, struct_insts_v1) blk_entry_inst3:
                STORE blk_entry_iref_x int64_1_local (is_ptr: false, order: MemoryOrder::Relaxed)
    );

    block!  ((vm, struct_insts_v1) blk_check);
    inst!   ((vm, struct_insts_v1) blk_entry_branch:
                BRANCH blk_check (blk_entry_a)
    );

    define_block! ((vm, struct_insts_v1) blk_entry() {
        blk_entry_inst0, blk_entry_inst1, blk_entry_inst2, blk_entry_inst3, blk_entry_branch
    });

    // blk check
    ssa!    ((vm, struct_insts_v1) <ref_point> blk_check_a);

    ssa!    ((vm, struct_insts_v1) <iref_point> blk_check_iref_a);
    inst!   ((vm, struct_insts_v1) blk_check_inst0:
                blk_check_iref_a = GETIREF blk_check_a
    );

    ssa!    ((vm, struct_insts_v1) <iref_int64> blk_check_iref_x);
    inst!   ((vm, struct_insts_v1) blk_check_inst1:
                blk_check_iref_x = GETFIELDIREF blk_check_iref_a (is_ptr: false, index: 0)
    );

    ssa!    ((vm, struct_insts_v1) <int64> blk_check_x);
    inst!   ((vm, struct_insts_v1) blk_check_inst2:
                blk_check_x = LOAD blk_check_iref_x (is_ptr: false, order: MemoryOrder::Relaxed)
    );

    ssa!    ((vm, struct_insts_v1) <iref_int64> blk_check_iref_y);
    inst!   ((vm, struct_insts_v1) blk_check_inst3:
                blk_check_iref_y = GETFIELDIREF blk_check_iref_a (is_ptr: false, index: 1)
    );

    ssa!    ((vm, struct_insts_v1) <int64> blk_check_y);
    inst!   ((vm, struct_insts_v1) blk_check_inst4:
                blk_check_y = LOAD blk_check_iref_y (is_ptr: false, order: MemoryOrder::Relaxed)
    );

    ssa!    ((vm, struct_insts_v1) <int64> blk_check_res);
    inst!   ((vm, struct_insts_v1) blk_check_inst5:
                blk_check_res = BINOP (BinOp::Add) blk_check_x blk_check_y
    );

    let blk_check_ccall = gen_ccall_exit(blk_check_res.clone(), &mut struct_insts_v1, &vm);

    inst!   ((vm, struct_insts_v1) blk_check_ret:
                RET
    );

    define_block! ((vm, struct_insts_v1) blk_check(blk_check_a) {
        blk_check_inst0, blk_check_inst1, blk_check_inst2, blk_check_inst3,
        blk_check_inst4, blk_check_inst5, blk_check_ccall, blk_check_ret
    });

    define_func_ver! ((vm) struct_insts_v1 (entry: blk_entry) {blk_entry, blk_check});

    vm
}

#[allow(dead_code)]
pub fn struct_insts() -> VM {
    let vm = VM::new();

    typedef!        ((vm) int64 = mu_int(64));
    typedef!        ((vm) point = mu_struct(int64, int64));
    typedef!        ((vm) ref_point  = mu_ref(point));
    typedef!        ((vm) iref_point = mu_iref(point));
    typedef!        ((vm) iref_int64 = mu_iref(int64));

    constdef!       ((vm) <int64> int64_0 = Constant::Int(0));
    constdef!       ((vm) <int64> int64_1 = Constant::Int(1));

    // .funcsig @noparam_noret_sig = () -> ()
    funcsig!        ((vm) noparam_noret_sig = () -> ());
    funcdecl!       ((vm) <noparam_noret_sig> struct_insts);
    funcdef!        ((vm) <noparam_noret_sig> struct_insts VERSION struct_insts_v1);

    // %entry():
    block!          ((vm, struct_insts_v1) blk_entry);

    // %a = NEW <@point>
    ssa!            ((vm, struct_insts_v1) <ref_point> a);
    inst!           ((vm, struct_insts_v1) blk_entry_new:
        a = NEW <point>
    );

    // %iref_a = GETIREF <@int64> %a
    ssa!            ((vm, struct_insts_v1) <iref_point> iref_a);
    inst!           ((vm, struct_insts_v1) blk_entry_getiref:
        iref_a = GETIREF a
    );

    // %iref_x = GETFIELDIREF <@point 0> %iref_a
    ssa!            ((vm, struct_insts_v1) <iref_int64> iref_x);
    inst!           ((vm, struct_insts_v1) blk_entry_getfield:
        iref_x = GETFIELDIREF iref_a (is_ptr: false, index: 0)
    );

    // STORE <@int64> %iref_x @int64_1
    consta!         ((vm, struct_insts_v1) int64_1_local = int64_1);
    inst!           ((vm, struct_insts_v1) blk_entry_store:
        STORE iref_x int64_1_local (is_ptr: false, order: MemoryOrder::Relaxed)
    );

    // BRANCH %check(%a)
    block!          ((vm, struct_insts_v1) blk_check);
    inst!           ((vm, struct_insts_v1) blk_entry_branch:
        BRANCH blk_check (a)
    );

    define_block!   ((vm, struct_insts_v1) blk_entry() {
        blk_entry_new,
        blk_entry_getiref,
        blk_entry_getfield,
        blk_entry_store,
        blk_entry_branch
    });

    // %check(%a):
    ssa!            ((vm, struct_insts_v1) <ref_point> blk_check_a);

    // %blk_check_iref_a = GETIREF <@point> a
    ssa!            ((vm, struct_insts_v1) <iref_point> blk_check_iref_a);
    inst!           ((vm, struct_insts_v1) blk_check_getiref:
        blk_check_iref_a = GETIREF blk_check_a
    );

    // %blk_check_iref_x = GETFIELDIREF <@point 0> %blk_check_iref_a
    ssa!            ((vm, struct_insts_v1) <iref_int64> blk_check_iref_x);
    inst!           ((vm, struct_insts_v1) blk_check_getfield:
        blk_check_iref_x = GETFIELDIREF blk_check_iref_a (is_ptr: false, index: 0)
    );

    // %x = LOAD <@int64> %blk_check_iref_x
    ssa!            ((vm, struct_insts_v1) <int64> blk_check_x);
    inst!           ((vm, struct_insts_v1) blk_check_load:
        blk_check_x = LOAD blk_check_iref_x (is_ptr: false, order: MemoryOrder::Relaxed)
    );

    // %blk_check_iref_y = GETFIELDIREF <@point 1> %blk_check_iref_a
    ssa!            ((vm, struct_insts_v1) <iref_int64> blk_check_iref_y);
    inst!           ((vm, struct_insts_v1) blk_check_getfield2:
        blk_check_iref_y = GETFIELDIREF blk_check_iref_a (is_ptr: false, index: 1)
    );

    // %y = LOAD <@int64> %blk_check_iref_y
    ssa!            ((vm, struct_insts_v1) <int64> blk_check_y);
    inst!           ((vm, struct_insts_v1) blk_check_load2:
        blk_check_y = LOAD blk_check_iref_y (is_ptr: false, order: MemoryOrder::Relaxed)
    );

    // %res = ADD <@int64> %x %y
    ssa!            ((vm, struct_insts_v1) <int64> res);
    inst!           ((vm, struct_insts_v1) blk_check_add:
        res = BINOP (BinOp::Add) blk_check_x blk_check_y
    );

    // CCALL exit(%res)
    let blk_check_ccall = gen_ccall_exit(res.clone(), &mut struct_insts_v1, &vm);

    // RET <@int64> 0
    consta!         ((vm, struct_insts_v1) int64_0_local = int64_0);
    inst!           ((vm, struct_insts_v1) blk_check_ret:
        RET (int64_0_local)
    );

    define_block!   ((vm, struct_insts_v1) blk_check(blk_check_a) {
        blk_check_getiref,
        blk_check_getfield,
        blk_check_load,
        blk_check_getfield2,
        blk_check_load2,
        blk_check_add,
        blk_check_ccall,
        blk_check_ret
    });

    define_func_ver!((vm) struct_insts_v1(entry: blk_entry) {
        blk_entry,
        blk_check
    });

    vm
}

#[test]
fn test_hybrid_fix_part() {
    VM::start_logging_trace();

    let vm = Arc::new(hybrid_fix_part_insts());

    let compiler = Compiler::new(CompilerPolicy::default(), &vm);

    let func_id = vm.id_of("hybrid_fix_part_insts");
    {
        let funcs = vm.funcs().read().unwrap();
        let func = funcs.get(&func_id).unwrap().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();
        let mut func_ver = func_vers
            .get(&func.cur_ver.unwrap())
            .unwrap()
            .write()
            .unwrap();

        compiler.compile(&mut func_ver);
    }

    vm.set_primordial_thread(func_id, true, vec![]);
    backend::emit_context(&vm);

    let executable = aot::link_primordial(
        vec!["hybrid_fix_part_insts".to_string()],
        "hybrid_fix_part_insts_test",
        &vm,
    );
    let output = linkutils::exec_path_nocheck(executable);

    assert!(output.status.code().is_some());

    let ret_code = output.status.code().unwrap();
    println!("return code: {}", ret_code);
    assert!(ret_code == 1);
}

pub fn hybrid_fix_part_insts() -> VM {
    let vm = VM::new();

    typedef!        ((vm) int64 = mu_int(64));
    typedef!        ((vm) my_hybrid   = mu_hybrid(int64, int64)(int64));
    typedef!        ((vm) ref_hybrid  = mu_ref(my_hybrid));
    typedef!        ((vm) iref_hybrid = mu_iref(my_hybrid));
    typedef!        ((vm) iref_int64  = mu_iref(int64));

    constdef!       ((vm) <int64> int64_0  = Constant::Int(0));
    constdef!       ((vm) <int64> int64_1  = Constant::Int(1));
    constdef!       ((vm) <int64> int64_10 = Constant::Int(10));

    funcsig!        ((vm) noparam_noret_sig = () -> ());
    funcdecl!       ((vm) <noparam_noret_sig> hybrid_fix_part_insts);
    funcdef!        ((vm) <noparam_noret_sig> hybrid_fix_part_insts
                          VERSION hybrid_fix_part_insts_v1);

    // %entry():
    block!          ((vm, hybrid_fix_part_insts_v1) blk_entry);

    // %a = NEWHYBRID <@my_hybrid @int64> @int64_10
    ssa!            ((vm, hybrid_fix_part_insts_v1) <ref_hybrid> a);
    consta!         ((vm, hybrid_fix_part_insts_v1) int64_10_local = int64_10);
    inst!           ((vm, hybrid_fix_part_insts_v1) blk_entry_newhybrid:
        a = NEWHYBRID <my_hybrid> int64_10_local
    );

    // %iref_a = GETIREF <@int64> %a
    ssa!            ((vm, hybrid_fix_part_insts_v1) <iref_hybrid> iref_a);
    inst!           ((vm, hybrid_fix_part_insts_v1) blk_entry_getiref:
        iref_a = GETIREF a
    );

    // %iref_x = GETFIELDIREF <@my_hybrid 0> %iref_a
    ssa!            ((vm, hybrid_fix_part_insts_v1) <iref_int64> iref_x);
    inst!           ((vm, hybrid_fix_part_insts_v1) blk_entry_getfield:
        iref_x = GETFIELDIREF iref_a (is_ptr: false, index: 0)
    );

    // STORE <@int64> %iref_x @int64_1
    consta!         ((vm, hybrid_fix_part_insts_v1) int64_1_local = int64_1);
    inst!           ((vm, hybrid_fix_part_insts_v1) blk_entry_store:
        STORE iref_x int64_1_local (is_ptr: false, order: MemoryOrder::Relaxed)
    );

    // BRANCH %check(%a)
    block!          ((vm, hybrid_fix_part_insts_v1) blk_check);
    inst!           ((vm, hybrid_fix_part_insts_v1) blk_entry_branch:
        BRANCH blk_check (a)
    );

    define_block!   ((vm, hybrid_fix_part_insts_v1) blk_entry() {
        blk_entry_newhybrid,
        blk_entry_getiref,
        blk_entry_getfield,
        blk_entry_store,
        blk_entry_branch
    });

    // %check(%a):
    ssa!            ((vm, hybrid_fix_part_insts_v1) <ref_hybrid> blk_check_a);

    // %blk_check_iref_a = GETIREF <@my_hybrid> a
    ssa!            ((vm, hybrid_fix_part_insts_v1) <iref_hybrid> blk_check_iref_a);
    inst!           ((vm, hybrid_fix_part_insts_v1) blk_check_getiref:
        blk_check_iref_a = GETIREF blk_check_a
    );

    // %blk_check_iref_x = GETFIELDIREF <@my_hybrid 0> %blk_check_iref_a
    ssa!            ((vm, hybrid_fix_part_insts_v1) <iref_int64> blk_check_iref_x);
    inst!           ((vm, hybrid_fix_part_insts_v1) blk_check_getfield:
        blk_check_iref_x = GETFIELDIREF blk_check_iref_a (is_ptr: false, index: 0)
    );

    // %x = LOAD <@int64> %blk_check_iref_x
    ssa!            ((vm, hybrid_fix_part_insts_v1) <int64> blk_check_x);
    inst!           ((vm, hybrid_fix_part_insts_v1) blk_check_load:
        blk_check_x = LOAD blk_check_iref_x (is_ptr: false, order: MemoryOrder::Relaxed)
    );

    // %blk_check_iref_y = GETFIELDIREF <@my_hybrid 1> %blk_check_iref_a
    ssa!            ((vm, hybrid_fix_part_insts_v1) <iref_int64> blk_check_iref_y);
    inst!           ((vm, hybrid_fix_part_insts_v1) blk_check_getfield2:
        blk_check_iref_y = GETFIELDIREF blk_check_iref_a (is_ptr: false, index: 1)
    );

    // %y = LOAD <@int64> %blk_check_iref_y
    ssa!            ((vm, hybrid_fix_part_insts_v1) <int64> blk_check_y);
    inst!           ((vm, hybrid_fix_part_insts_v1) blk_check_load2:
        blk_check_y = LOAD blk_check_iref_y (is_ptr: false, order: MemoryOrder::Relaxed)
    );

    // %res = ADD <@int64> %x %y
    ssa!            ((vm, hybrid_fix_part_insts_v1) <int64> blk_check_res);
    inst!           ((vm, hybrid_fix_part_insts_v1) blk_check_add:
        blk_check_res = BINOP (BinOp::Add) blk_check_x blk_check_y
    );

    // CCALL exit(%res)
    let blk_check_ccall = gen_ccall_exit(blk_check_res.clone(), &mut hybrid_fix_part_insts_v1, &vm);

    // RET
    inst!           ((vm, hybrid_fix_part_insts_v1) blk_check_ret:
        RET
    );

    define_block!   ((vm, hybrid_fix_part_insts_v1) blk_check(blk_check_a) {
        blk_check_getiref,
        blk_check_getfield,
        blk_check_load,
        blk_check_getfield2,
        blk_check_load2,
        blk_check_add,
        blk_check_ccall,
        blk_check_ret
    });

    define_func_ver!((vm) hybrid_fix_part_insts_v1 (entry: blk_entry) {
        blk_entry,
        blk_check
    });

    vm
}

#[test]
fn test_hybrid_var_part() {
    VM::start_logging_trace();

    let vm = Arc::new(hybrid_var_part_insts());

    let compiler = Compiler::new(CompilerPolicy::default(), &vm);

    let func_id = vm.id_of("hybrid_var_part_insts");
    {
        let funcs = vm.funcs().read().unwrap();
        let func = funcs.get(&func_id).unwrap().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();
        let mut func_ver = func_vers
            .get(&func.cur_ver.unwrap())
            .unwrap()
            .write()
            .unwrap();

        compiler.compile(&mut func_ver);
    }

    vm.set_primordial_thread(func_id, true, vec![]);
    backend::emit_context(&vm);

    let executable = aot::link_primordial(
        vec!["hybrid_var_part_insts".to_string()],
        "hybrid_var_part_insts_test",
        &vm,
    );
    let output = linkutils::exec_path_nocheck(executable);

    assert!(output.status.code().is_some());

    let ret_code = output.status.code().unwrap();
    println!("return code: {}", ret_code);
    assert!(ret_code == 20);
}

pub fn hybrid_var_part_insts() -> VM {
    let vm = VM::new();

    typedef!        ((vm) int64      = mu_int(64));
    typedef!        ((vm) my_hybrid  = mu_hybrid(int64, int64)(int64));
    typedef!        ((vm) ref_hybrid  = mu_ref(my_hybrid));
    typedef!        ((vm) iref_hybrid = mu_iref(my_hybrid));
    typedef!        ((vm) iref_int64  = mu_iref(int64));
    typedef!        ((vm) int1       = mu_int(1));

    constdef!       ((vm) <int64> int64_0 = Constant::Int(0));
    constdef!       ((vm) <int64> int64_1 = Constant::Int(1));
    constdef!       ((vm) <int64> int64_2 = Constant::Int(2));
    constdef!       ((vm) <int64> int64_3 = Constant::Int(3));
    constdef!       ((vm) <int64> int64_4 = Constant::Int(4));
    constdef!       ((vm) <int64> int64_10 = Constant::Int(10));

    funcsig!        ((vm) noparam_noret_sig = () -> ());
    funcdecl!       ((vm) <noparam_noret_sig> hybrid_var_part_insts);
    funcdef!        ((vm) <noparam_noret_sig> hybrid_var_part_insts
                          VERSION hybrid_var_part_insts_v1);

    // %entry():
    block!          ((vm, hybrid_var_part_insts_v1) blk_entry);

    // %a = NEWHYBRID <@my_hybrid @int64> @int64_10
    ssa!            ((vm, hybrid_var_part_insts_v1) <ref_hybrid> a);
    consta!         ((vm, hybrid_var_part_insts_v1) int64_10_local = int64_10);
    inst!           ((vm, hybrid_var_part_insts_v1) blk_entry_newhybrid:
        a = NEWHYBRID <my_hybrid> int64_10_local
    );

    // %iref_a = GETIREF <@int64> %a
    ssa!            ((vm, hybrid_var_part_insts_v1) <iref_hybrid> iref_a);
    inst!           ((vm, hybrid_var_part_insts_v1) blk_entry_getiref:
        iref_a = GETIREF a
    );

    // %iref_var = GETVARPARTIREF <@my_hybrid> %iref_a
    ssa!            ((vm, hybrid_var_part_insts_v1) <iref_int64> iref_var);
    inst!           ((vm, hybrid_var_part_insts_v1) blk_entry_getvarpart:
        iref_var = GETVARPARTIREF iref_a (is_ptr: false)
    );

    // %var0 = SHIFTIREF <@int64> %iref_var %int64_0
    ssa!            ((vm, hybrid_var_part_insts_v1) <iref_int64> var0);
    consta!         ((vm, hybrid_var_part_insts_v1) int64_0_local = int64_0);
    inst!           ((vm, hybrid_var_part_insts_v1) blk_entry_shift0:
        var0 = SHIFTIREF iref_var int64_0_local (is_ptr: false)
    );

    // STORE <@int64> %var0 @int64_10
    inst!           ((vm, hybrid_var_part_insts_v1) blk_entry_store0:
        STORE var0 int64_10_local (is_ptr: false, order: MemoryOrder::Relaxed)
    );

    // %var4 = SHIFTIREF <@int64> %iref_var %int64_4
    ssa!            ((vm, hybrid_var_part_insts_v1) <iref_int64> var4);
    consta!         ((vm, hybrid_var_part_insts_v1) int64_4_local = int64_4);
    inst!           ((vm, hybrid_var_part_insts_v1) blk_entry_shift4:
        var4 = SHIFTIREF iref_var int64_4_local (is_ptr: false)
    );

    // STORE <@int64> %var4 @int64_10
    inst!           ((vm, hybrid_var_part_insts_v1) blk_entry_store4:
        STORE var4 int64_10_local (is_ptr: false, order: MemoryOrder::Relaxed)
    );

    // BRANCH %check(%a)
    block!          ((vm, hybrid_var_part_insts_v1) blk_check);
    inst!           ((vm, hybrid_var_part_insts_v1) blk_entry_branch:
        BRANCH blk_check(a)
    );

    define_block!   ((vm, hybrid_var_part_insts_v1) blk_entry() {
        blk_entry_newhybrid,
        blk_entry_getiref,
        blk_entry_getvarpart,
        blk_entry_shift0,
        blk_entry_store0,
        blk_entry_shift4,
        blk_entry_store4,
        blk_entry_branch
    });

    // %check(%a):
    ssa!            ((vm, hybrid_var_part_insts_v1) <ref_hybrid> blk_check_a);

    // BRANCH %head (<@int64> sum, <@int64> n, <@int64> %i, <@ref_hybrid> %a)
    //                        0             10            0
    block!          ((vm, hybrid_var_part_insts_v1) blk_head);
    inst!           ((vm, hybrid_var_part_insts_v1) blk_check_branch:
        BRANCH blk_head (int64_0_local, int64_10_local, int64_0_local, blk_check_a)
    );

    define_block!   ((vm, hybrid_var_part_insts_v1) blk_check(blk_check_a) {
        blk_check_branch
    });

    // %head(%sum, %n, %i, %a)
    ssa!            ((vm, hybrid_var_part_insts_v1) <int64> blk_head_sum);
    ssa!            ((vm, hybrid_var_part_insts_v1) <int64> blk_head_n);
    ssa!            ((vm, hybrid_var_part_insts_v1) <int64> blk_head_i);
    ssa!            ((vm, hybrid_var_part_insts_v1) <int64> blk_head_a);

    // %cond = SLT <@int64> %i %n
    ssa!            ((vm, hybrid_var_part_insts_v1) <int1> blk_head_cond);
    inst!           ((vm, hybrid_var_part_insts_v1) blk_head_slt:
        blk_head_cond = CMPOP (CmpOp::SLT) blk_head_i blk_head_n
    );

    // BRANCH2 %cond %body(%sum, %n, %i, %a) %exit(%sum)
    block!          ((vm, hybrid_var_part_insts_v1) blk_body);
    block!          ((vm, hybrid_var_part_insts_v1) blk_exit);
    inst!           ((vm, hybrid_var_part_insts_v1) blk_head_branch2:
        BRANCH2 (blk_head_cond, blk_head_sum, blk_head_n, blk_head_i, blk_head_a)
            IF (OP 0)
            THEN blk_body (vec![1, 2, 3, 4]) WITH 0.9f32,
            ELSE blk_exit (vec![1])
    );

    define_block!   ((vm, hybrid_var_part_insts_v1)
        blk_head (blk_head_sum, blk_head_n, blk_head_i, blk_head_a) {
        blk_head_slt,
        blk_head_branch2
    });

    // %body(%sum, %n, %i, %a):
    ssa!            ((vm, hybrid_var_part_insts_v1) <int64> blk_body_sum);
    ssa!            ((vm, hybrid_var_part_insts_v1) <int64> blk_body_n);
    ssa!            ((vm, hybrid_var_part_insts_v1) <int64> blk_body_i);
    ssa!            ((vm, hybrid_var_part_insts_v1) <int64> blk_body_a);

    // %blk_body_iref_a = GETIREF <@my_hybrid> a
    ssa!            ((vm, hybrid_var_part_insts_v1) <iref_hybrid> blk_body_iref_a);
    inst!           ((vm, hybrid_var_part_insts_v1) blk_body_getiref:
        blk_body_iref_a = GETIREF blk_body_a
    );

    // %blk_body_iref_var = GETVARPARTIREF <@my_hybrid> %blk_body_iref_a
    ssa!            ((vm, hybrid_var_part_insts_v1) <iref_int64> blk_body_iref_var);
    inst!           ((vm, hybrid_var_part_insts_v1) blk_body_getvarpart:
        blk_body_iref_var = GETVARPARTIREF blk_body_iref_a (is_ptr: false)
    );

    // %blk_body_iref_var_i = SHIFTIREF <@int64> %blk_body_iref_var %i
    ssa!            ((vm, hybrid_var_part_insts_v1) <iref_int64> blk_body_iref_var_i);
    inst!           ((vm, hybrid_var_part_insts_v1) blk_body_shiftiref:
        blk_body_iref_var_i = SHIFTIREF blk_body_iref_var blk_body_i (is_ptr: false)
    );

    // %blk_body_ele = LOAD <@int64> %blk_body_iref_var_i
    ssa!            ((vm, hybrid_var_part_insts_v1) <int64> blk_body_ele);
    inst!           ((vm, hybrid_var_part_insts_v1) blk_body_load:
        blk_body_ele = LOAD blk_body_iref_var_i (is_ptr: false, order: MemoryOrder::Relaxed)
    );

    // %blk_body_sum2 = ADD <@int64> %blk_body_sum %blk_body_ele
    ssa!            ((vm, hybrid_var_part_insts_v1) <int64> blk_body_sum2);
    inst!           ((vm, hybrid_var_part_insts_v1) blk_body_add:
        blk_body_sum2 = BINOP (BinOp::Add) blk_body_sum blk_body_ele
    );

    // %blk_body_i2 = ADD <@int64> %blk_body_i @int64_1
    ssa!            ((vm, hybrid_var_part_insts_v1) <int64> blk_body_i2);
    consta!         ((vm, hybrid_var_part_insts_v1) int64_1_local = int64_1);
    inst!           ((vm, hybrid_var_part_insts_v1) blk_body_add2:
        blk_body_i2 = BINOP (BinOp::Add) blk_body_i int64_1_local
    );

    // BRANCH1 %head (%sum2, %n, %i2, %a)
    inst!           ((vm, hybrid_var_part_insts_v1) blk_body_branch:
        BRANCH blk_head (blk_body_sum2, blk_body_n, blk_body_i2, blk_body_a)
    );

    define_block!   ((vm, hybrid_var_part_insts_v1)
        blk_body(blk_body_sum, blk_body_n, blk_body_i, blk_body_a) {
        blk_body_getiref,
        blk_body_getvarpart,
        blk_body_shiftiref,
        blk_body_load,
        blk_body_add,
        blk_body_add2,
        blk_body_branch
    });

    // %exit(%sum):
    ssa!            ((vm, hybrid_var_part_insts_v1) <int64> blk_exit_sum);

    let blk_exit_exit = gen_ccall_exit(blk_exit_sum.clone(), &mut hybrid_var_part_insts_v1, &vm);

    // RET
    inst!           ((vm, hybrid_var_part_insts_v1) blk_exit_ret:
        RET
    );

    define_block!   ((vm, hybrid_var_part_insts_v1) blk_exit(blk_exit_sum) {
        blk_exit_exit,
        blk_exit_ret
    });

    define_func_ver!((vm) hybrid_var_part_insts_v1 (entry: blk_entry) {
        blk_entry,
        blk_check,
        blk_head,
        blk_body,
        blk_exit
    });

    vm
}

#[test]
fn test_shift_iref_ele_4bytes() {
    let lib = linkutils::aot::compile_fnc("shift_iref_ele_4bytes", &shift_iref_ele_4bytes);

    unsafe {
        let shift_iref_ele_4bytes: libloading::Symbol<
            unsafe extern "C" fn(u64, u64) -> u64,
        > = lib.get(b"shift_iref_ele_4bytes").unwrap();

        let res = shift_iref_ele_4bytes(0, 0);
        println!("shift_iref_ele_4bytes(0, 0) = {}", res);
        assert_eq!(res, 0);

        let res = shift_iref_ele_4bytes(0, 1);
        println!("shift_iref_ele_4bytes(0, 1) = {}", res);
        assert_eq!(res, 4);

        let res = shift_iref_ele_4bytes(0, 2);
        println!("shift_iref_ele_4bytes(0, 2) = {}", res);
        assert_eq!(res, 8);
    }
}

fn shift_iref_ele_4bytes() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int32  = mu_int(32));
    typedef!    ((vm) int64  = mu_int(64));
    typedef!    ((vm) elem   = mu_struct(int32));
    typedef!    ((vm) iref_elem  = mu_iref(elem));

    funcsig!    ((vm) sig = (iref_elem, int64) -> (iref_elem));
    funcdecl!   ((vm) <sig> shift_iref_ele_4bytes);

    funcdef!    ((vm) <sig> shift_iref_ele_4bytes VERSION shift_iref_ele_4bytes_v1);

    // blk entry
    block!      ((vm, shift_iref_ele_4bytes_v1) blk_entry);

    ssa!        ((vm, shift_iref_ele_4bytes_v1) <iref_elem> base);
    ssa!        ((vm, shift_iref_ele_4bytes_v1) <int64> index);
    ssa!        ((vm, shift_iref_ele_4bytes_v1) <iref_elem> res);

    inst!       ((vm, shift_iref_ele_4bytes_v1) blk_entry_shiftiref:
        res = SHIFTIREF base index (is_ptr: false)
    );

    inst!       ((vm, shift_iref_ele_4bytes_v1) blk_entry_ret:
        RET (res)
    );

    define_block!   ((vm, shift_iref_ele_4bytes_v1) blk_entry(base, index) {
        blk_entry_shiftiref, blk_entry_ret
    });

    define_func_ver!((vm) shift_iref_ele_4bytes_v1 (entry: blk_entry) {
        blk_entry
    });

    vm
}

#[test]
fn test_shift_iref_ele_8bytes() {
    let lib = linkutils::aot::compile_fnc("shift_iref_ele_8bytes", &shift_iref_ele_8bytes);

    unsafe {
        let shift_iref_ele_8bytes: libloading::Symbol<
            unsafe extern "C" fn(u64, u64) -> u64,
        > = lib.get(b"shift_iref_ele_8bytes").unwrap();

        let res = shift_iref_ele_8bytes(0, 0);
        println!("shift_iref_ele_8bytes(0, 0) = {}", res);
        assert_eq!(res, 0);

        let res = shift_iref_ele_8bytes(0, 1);
        println!("shift_iref_ele_8bytes(0, 1) = {}", res);
        assert_eq!(res, 8);

        let res = shift_iref_ele_8bytes(0, 2);
        println!("shift_iref_ele_8bytes(0, 2) = {}", res);
        assert_eq!(res, 16);
    }
}

fn shift_iref_ele_8bytes() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int64  = mu_int(64));
    typedef!    ((vm) elem   = mu_struct(int64));
    typedef!    ((vm) iref_elem  = mu_iref(elem));

    funcsig!    ((vm) sig = (iref_elem, int64) -> (iref_elem));
    funcdecl!   ((vm) <sig> shift_iref_ele_8bytes);

    funcdef!    ((vm) <sig> shift_iref_ele_8bytes VERSION shift_iref_ele_8bytes_v1);

    // blk entry
    block!      ((vm, shift_iref_ele_8bytes_v1) blk_entry);

    ssa!        ((vm, shift_iref_ele_8bytes_v1) <iref_elem> base);
    ssa!        ((vm, shift_iref_ele_8bytes_v1) <int64> index);
    ssa!        ((vm, shift_iref_ele_8bytes_v1) <iref_elem> res);

    inst!       ((vm, shift_iref_ele_8bytes_v1) blk_entry_shiftiref:
        res = SHIFTIREF base index (is_ptr: false)
    );

    inst!       ((vm, shift_iref_ele_8bytes_v1) blk_entry_ret:
        RET (res)
    );

    define_block!   ((vm, shift_iref_ele_8bytes_v1) blk_entry(base, index) {
        blk_entry_shiftiref, blk_entry_ret
    });

    define_func_ver!((vm) shift_iref_ele_8bytes_v1 (entry: blk_entry) {
        blk_entry
    });

    vm
}

#[test]
fn test_shift_iref_ele_9bytes() {
    let lib = linkutils::aot::compile_fnc("shift_iref_ele_9bytes", &shift_iref_ele_9bytes);

    unsafe {
        let shift_iref_ele_9bytes: libloading::Symbol<
            unsafe extern "C" fn(u64, u64) -> u64,
        > = lib.get(b"shift_iref_ele_9bytes").unwrap();

        let res = shift_iref_ele_9bytes(0, 0);
        println!("shift_iref_ele_9bytes(0, 0) = {}", res);
        assert_eq!(res, 0);

        let res = shift_iref_ele_9bytes(0, 1);
        println!("shift_iref_ele_9bytes(0, 1) = {}", res);
        assert_eq!(res, 16);

        let res = shift_iref_ele_9bytes(0, 2);
        println!("shift_iref_ele_9bytes(0, 2) = {}", res);
        assert_eq!(res, 32);
    }
}

fn shift_iref_ele_9bytes() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int8   = mu_int(8));
    typedef!    ((vm) int64  = mu_int(64));
    typedef!    ((vm) elem   = mu_struct(int64, int8));
    typedef!    ((vm) iref_elem  = mu_iref(elem));

    funcsig!    ((vm) sig = (iref_elem, int64) -> (iref_elem));
    funcdecl!   ((vm) <sig> shift_iref_ele_9bytes);

    funcdef!    ((vm) <sig> shift_iref_ele_9bytes VERSION shift_iref_ele_9bytes_v1);

    // blk entry
    block!      ((vm, shift_iref_ele_9bytes_v1) blk_entry);

    ssa!        ((vm, shift_iref_ele_9bytes_v1) <iref_elem> base);
    ssa!        ((vm, shift_iref_ele_9bytes_v1) <int64> index);
    ssa!        ((vm, shift_iref_ele_9bytes_v1) <iref_elem> res);

    inst!       ((vm, shift_iref_ele_9bytes_v1) blk_entry_shiftiref:
        res = SHIFTIREF base index (is_ptr: false)
    );

    inst!       ((vm, shift_iref_ele_9bytes_v1) blk_entry_ret:
        RET (res)
    );

    define_block!   ((vm, shift_iref_ele_9bytes_v1) blk_entry(base, index) {
        blk_entry_shiftiref, blk_entry_ret
    });

    define_func_ver!((vm) shift_iref_ele_9bytes_v1 (entry: blk_entry) {
        blk_entry
    });

    vm
}

#[test]
fn test_shift_iref_ele_16bytes() {
    let lib = linkutils::aot::compile_fnc("shift_iref_ele_16bytes", &shift_iref_ele_16bytes);

    unsafe {
        let shift_iref_ele_16bytes: libloading::Symbol<
            unsafe extern "C" fn(u64, u64) -> u64,
        > = lib.get(b"shift_iref_ele_16bytes").unwrap();

        let res = shift_iref_ele_16bytes(0, 0);
        println!("shift_iref_ele_16bytes(0, 0) = {}", res);
        assert_eq!(res, 0);

        let res = shift_iref_ele_16bytes(0, 1);
        println!("shift_iref_ele_16bytes(0, 1) = {}", res);
        assert_eq!(res, 16);

        let res = shift_iref_ele_16bytes(0, 2);
        println!("shift_iref_ele_16bytes(0, 2) = {}", res);
        assert_eq!(res, 32);
    }
}

fn shift_iref_ele_16bytes() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int64  = mu_int(64));
    typedef!    ((vm) elem   = mu_struct(int64, int64));
    typedef!    ((vm) iref_elem  = mu_iref(elem));

    funcsig!    ((vm) sig = (iref_elem, int64) -> (iref_elem));
    funcdecl!   ((vm) <sig> shift_iref_ele_16bytes);

    funcdef!    ((vm) <sig> shift_iref_ele_16bytes VERSION shift_iref_ele_16bytes_v1);

    // blk entry
    block!      ((vm, shift_iref_ele_16bytes_v1) blk_entry);

    ssa!        ((vm, shift_iref_ele_16bytes_v1) <iref_elem> base);
    ssa!        ((vm, shift_iref_ele_16bytes_v1) <int64> index);
    ssa!        ((vm, shift_iref_ele_16bytes_v1) <iref_elem> res);

    inst!       ((vm, shift_iref_ele_16bytes_v1) blk_entry_shiftiref:
        res = SHIFTIREF base index (is_ptr: false)
    );

    inst!       ((vm, shift_iref_ele_16bytes_v1) blk_entry_ret:
        RET (res)
    );

    define_block!   ((vm, shift_iref_ele_16bytes_v1) blk_entry(base, index) {
        blk_entry_shiftiref, blk_entry_ret
    });

    define_func_ver!((vm) shift_iref_ele_16bytes_v1 (entry: blk_entry) {
        blk_entry
    });

    vm
}

#[test]
fn test_get_elem_iref_array_ele_9bytes() {
    let lib = linkutils::aot::compile_fnc(
        "get_elem_iref_array_ele_9bytes",
        &get_elem_iref_array_ele_9bytes,
    );

    unsafe {
        let get_elem_iref_array_ele_9bytes: libloading::Symbol<
            unsafe extern "C" fn(u64, u64) -> u64,
        > = lib.get(b"get_elem_iref_array_ele_9bytes").unwrap();

        let res = get_elem_iref_array_ele_9bytes(0, 0);
        println!("get_elem_iref_array_ele_9bytes(0, 0) = {}", res);
        assert_eq!(res, 0);

        let res = get_elem_iref_array_ele_9bytes(0, 1);
        println!("get_elem_iref_array_ele_9bytes(0, 1) = {}", res);
        assert_eq!(res, 16);

        let res = get_elem_iref_array_ele_9bytes(0, 2);
        println!("get_elem_iref_array_ele_9bytes(0, 2) = {}", res);
        assert_eq!(res, 32);
    }
}

fn get_elem_iref_array_ele_9bytes() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int8  = mu_int(8));
    typedef!    ((vm) int64 = mu_int(64));
    typedef!    ((vm) elem  = mu_struct(int64, int8));
    typedef!    ((vm) array_9bytes = mu_array(elem, 5));
    typedef!    ((vm) iref_elem  = mu_iref(elem));
    typedef!    ((vm) iref_array = mu_iref(array_9bytes));

    funcsig!    ((vm) sig = (iref_array, int64) -> (iref_elem));
    funcdecl!   ((vm) <sig> get_elem_iref_array_ele_9bytes);

    funcdef!    ((vm) <sig> get_elem_iref_array_ele_9bytes
                      VERSION get_elem_iref_array_ele_9bytes_v1);

    // blk entry
    block!      ((vm, get_elem_iref_array_ele_9bytes_v1) blk_entry);

    ssa!        ((vm, get_elem_iref_array_ele_9bytes_v1) <iref_array> base);
    ssa!        ((vm, get_elem_iref_array_ele_9bytes_v1) <int64> index);
    ssa!        ((vm, get_elem_iref_array_ele_9bytes_v1) <iref_elem> res);

    inst!       ((vm, get_elem_iref_array_ele_9bytes_v1) blk_entry_get_elem_iref:
        res = GETELEMIREF base index (is_ptr: false)
    );

    inst!       ((vm, get_elem_iref_array_ele_9bytes_v1) blk_entry_ret:
        RET (res)
    );

    define_block!((vm, get_elem_iref_array_ele_9bytes_v1) blk_entry(base, index) {
        blk_entry_get_elem_iref, blk_entry_ret
    });

    define_func_ver!((vm) get_elem_iref_array_ele_9bytes_v1 (entry: blk_entry) {
        blk_entry
    });

    vm
}
