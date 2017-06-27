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

use mu::testutil;
use mu::testutil::aot;
use mu::utils::LinkedHashMap;
use test_compiler::test_call::gen_ccall_exit;
use self::mu::compiler::*;
use self::mu::ast::ir::*;
use self::mu::ast::types::*;
use self::mu::ast::inst::*;
use self::mu::ast::op::*;
use self::mu::vm::VM;

use std::sync::Arc;

// NOTE: aarch64 has 28 usable GPRs (wheras x86-64 has 14) so there are slightly different tests for spilling on aarch64

fn get_number_of_moves(fv_id: MuID, vm: &VM) -> usize {
    let cfs = vm.compiled_funcs().read().unwrap();
    let cf  = cfs.get(&fv_id).unwrap().read().unwrap();

    let mut n_mov_insts = 0;

    let mc = cf.mc();
    for i in 0..mc.number_of_insts() {
        if mc.is_move(i) {
            n_mov_insts += 1;
        }
    }

    n_mov_insts
}

#[test]
#[allow(unused_variables)]
fn test_spill1() {
    VM::start_logging_trace();
    
    let vm = Arc::new(create_spill1());
    
    let compiler = Compiler::new(CompilerPolicy::default(), &vm);
    
    let func_id = vm.id_of("spill1");
    {
        let funcs = vm.funcs().read().unwrap();
        let func = funcs.get(&func_id).unwrap().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();
        let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();
        
        compiler.compile(&mut func_ver);
    }

    backend::emit_context(&vm);

    let dylib = aot::link_dylib(vec![Mu("spill1")], &testutil::get_dylib_name("spill1"), &vm);

    let lib = libloading::Library::new(dylib.as_os_str()).unwrap();
    unsafe {
        let spill1 : libloading::Symbol<unsafe extern fn() -> u64> = match lib.get(b"spill1") {
            Ok(symbol) => symbol,
            Err(e) => panic!("cannot find symbol spill1 in dylib: {:?}", e)
        };

        // we cannot call this (it doesnt return)
    }
}

#[cfg(target_arch = "x86_64")]
fn create_spill1() -> VM {
    let vm = VM::new();

    typedef!        ((vm) int64 = mu_int(64));

    funcsig!        ((vm) spill1_sig = (int64, int64, int64, int64, int64,
                                        int64, int64, int64, int64, int64) -> (int64));
    funcdecl!       ((vm) <spill1_sig> spill1);
    funcdef!        ((vm) <spill1_sig> spill1 VERSION spill1_v1);

    typedef!        ((vm) funcref_spill1 = mu_funcref(spill1_sig));
    constdef!       ((vm) <funcref_spill1> const_funcref_spill1 = Constant::FuncRef(vm.id_of("spill1")));
    
    // %entry(<@int_64> %t1, t2, ... t10):
    block!          ((vm, spill1_v1) blk_entry);
    ssa!            ((vm, spill1_v1) <int64> t1);
    ssa!            ((vm, spill1_v1) <int64> t2);
    ssa!            ((vm, spill1_v1) <int64> t3);
    ssa!            ((vm, spill1_v1) <int64> t4);
    ssa!            ((vm, spill1_v1) <int64> t5);
    ssa!            ((vm, spill1_v1) <int64> t6);
    ssa!            ((vm, spill1_v1) <int64> t7);
    ssa!            ((vm, spill1_v1) <int64> t8);
    ssa!            ((vm, spill1_v1) <int64> t9);
    ssa!            ((vm, spill1_v1) <int64> t10);
    
    // %x = CALL spill1(%t1, %t2, ... t10)
    ssa!            ((vm, spill1_v1) <int64> x);
    consta!         ((vm, spill1_v1) const_funcref_spill1_local = const_funcref_spill1);
    inst!           ((vm, spill1_v1) blk_entry_call:
        x = EXPRCALL (CallConvention::Mu, is_abort: false)
                     const_funcref_spill1_local(t1, t2, t3, t4, t5, t6, t7, t8, t9, t10)
    );
    
    // %res0 = ADD %t1 %t2
    ssa!            ((vm, spill1_v1) <int64> res0);
    inst!           ((vm, spill1_v1) blk_entry_add0:
        res0 = BINOP (BinOp::Add) t1 t2
    );
    
    // %res1 = ADD %res0 %t3
    ssa!            ((vm, spill1_v1) <int64> res1);
    inst!           ((vm, spill1_v1) blk_entry_add1:
        res1 = BINOP (BinOp::Add) res0 t3
    );
    
    // %res2 = ADD %res1 %t4
    ssa!            ((vm, spill1_v1) <int64> res2);
    inst!           ((vm, spill1_v1) blk_entry_add2:
        res2 = BINOP (BinOp::Add) res1 t4
    );
    
    // %res3 = ADD %res2 %t5
    ssa!            ((vm, spill1_v1) <int64> res3);
    inst!           ((vm, spill1_v1) blk_entry_add3:
        res3 = BINOP (BinOp::Add) res2 t5
    );
    
    // RET %res3
    inst!           ((vm, spill1_v1) blk_entry_ret:
        RET (res3)
    );

    define_block!   ((vm, spill1_v1) blk_entry
        (t1, t2, t3, t4, t5, t6, t7, t8, t9, t10) {
            blk_entry_call,
            blk_entry_add0,
            blk_entry_add1,
            blk_entry_add2,
            blk_entry_add3,
            blk_entry_ret
        }
    );

    define_func_ver!((vm) spill1_v1(entry: blk_entry) {
        blk_entry
    });
    
    vm
}

#[cfg(target_arch = "aarch64")]
fn create_spill1() -> VM {
    let vm = VM::new();

    typedef!        ((vm) int64 = mu_int(64));

    funcsig!        ((vm) spill1_sig = (int64, int64, int64, int64, int64,
                                        int64, int64, int64, int64, int64,
                                        int64, int64, int64, int64, int64,
                                        int64, int64, int64, int64, int64,
                                        int64, int64, int64, int64) -> (int64));
    funcdecl!       ((vm) <spill1_sig> spill1);
    funcdef!        ((vm) <spill1_sig> spill1 VERSION spill1_v1);

    typedef!        ((vm) funcref_spill1 = mu_funcref(spill1_sig));
    constdef!       ((vm) <funcref_spill1> const_funcref_spill1 = Constant::FuncRef(vm.id_of("spill1")));

    // %entry(<@int_64> %t1, t2, ... t10):
    block!          ((vm, spill1_v1) blk_entry);
    ssa!            ((vm, spill1_v1) <int64> t1);
    ssa!            ((vm, spill1_v1) <int64> t2);
    ssa!            ((vm, spill1_v1) <int64> t3);
    ssa!            ((vm, spill1_v1) <int64> t4);
    ssa!            ((vm, spill1_v1) <int64> t5);
    ssa!            ((vm, spill1_v1) <int64> t6);
    ssa!            ((vm, spill1_v1) <int64> t7);
    ssa!            ((vm, spill1_v1) <int64> t8);
    ssa!            ((vm, spill1_v1) <int64> t9);
    ssa!            ((vm, spill1_v1) <int64> t10);
    ssa!            ((vm, spill1_v1) <int64> t11);
    ssa!            ((vm, spill1_v1) <int64> t12);
    ssa!            ((vm, spill1_v1) <int64> t13);
    ssa!            ((vm, spill1_v1) <int64> t14);
    ssa!            ((vm, spill1_v1) <int64> t15);
    ssa!            ((vm, spill1_v1) <int64> t16);
    ssa!            ((vm, spill1_v1) <int64> t17);
    ssa!            ((vm, spill1_v1) <int64> t18);
    ssa!            ((vm, spill1_v1) <int64> t19);
    ssa!            ((vm, spill1_v1) <int64> t20);
    ssa!            ((vm, spill1_v1) <int64> t21);
    ssa!            ((vm, spill1_v1) <int64> t22);
    ssa!            ((vm, spill1_v1) <int64> t23);
    ssa!            ((vm, spill1_v1) <int64> t24);

    // %x = CALL spill1(%t1, %t2, ... t10)
    ssa!            ((vm, spill1_v1) <int64> x);
    consta!         ((vm, spill1_v1) const_funcref_spill1_local = const_funcref_spill1);
    inst!           ((vm, spill1_v1) blk_entry_call:
        x = EXPRCALL (CallConvention::Mu, is_abort: false)
                     const_funcref_spill1_local(t1, t2, t3, t4, t5, t6, t7, t8, t9, t10,
                      t11, t12, t13, t14, t15, t16, t17, t18, t19, t20, t21, t22, t23, t24)
    );
    // %res0 = ADD %t1 %t2
    ssa!            ((vm, spill1_v1) <int64> res0);
    inst!           ((vm, spill1_v1) blk_entry_add0:
        res0 = BINOP (BinOp::Add) t1 t2
    );

    // %res1 = ADD %res0 %t3
    ssa!            ((vm, spill1_v1) <int64> res1);
    inst!           ((vm, spill1_v1) blk_entry_add1:
        res1 = BINOP (BinOp::Add) res0 t3
    );

    // %res2 = ADD %res1 %t4
    ssa!            ((vm, spill1_v1) <int64> res2);
    inst!           ((vm, spill1_v1) blk_entry_add2:
        res2 = BINOP (BinOp::Add) res1 t4
    );

    // %res3 = ADD %res2 %t5
    ssa!            ((vm, spill1_v1) <int64> res3);
    inst!           ((vm, spill1_v1) blk_entry_add3:
        res3 = BINOP (BinOp::Add) res2 t5
    );

    // RET %res3
    inst!           ((vm, spill1_v1) blk_entry_ret:
        RET (res3)
    );

    define_block!   ((vm, spill1_v1) blk_entry
        (t1, t2, t3, t4, t5, t6, t7, t8, t9, t10, t11, t12, t13, t14, t15, t16, t17, t18, t19, t20, t21, t22, t23, t24) {
            blk_entry_call,
            blk_entry_add0,
            blk_entry_add1,
            blk_entry_add2,
            blk_entry_add3,
            blk_entry_ret
        }
    );

    define_func_ver!((vm) spill1_v1(entry: blk_entry) {
        blk_entry
    });

    vm
}

#[test]
fn test_simple_spill() {
    VM::start_logging_trace();

    let vm = Arc::new(create_simple_spill());

    let compiler = Compiler::new(CompilerPolicy::default(), &vm);

    let func_id = vm.id_of("simple_spill");
    {
        let funcs = vm.funcs().read().unwrap();
        let func = funcs.get(&func_id).unwrap().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();
        let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();

        compiler.compile(&mut func_ver);
    }

    backend::emit_context(&vm);

    let dylib = aot::link_dylib(vec![Mu("simple_spill")], &testutil::get_dylib_name("simple_spill"), &vm);

    let lib = libloading::Library::new(dylib.as_os_str()).unwrap();
    unsafe {
        let simple_spill : libloading::Symbol<unsafe extern fn() -> u64> = match lib.get(b"simple_spill") {
            Ok(symbol) => symbol,
            Err(e) => panic!("cannot find symbol simple_spill in dylib: {:?}", e)
        };

        let res = simple_spill();
        println!("simple_spill() = {}", res);
        assert!(res == 2);
    }
}

#[cfg(target_arch = "x86_64")]
fn create_simple_spill() -> VM {
    let vm = VM::new();

    typedef!        ((vm) int64 = mu_int(64));
    constdef!       ((vm) <int64> int64_1 = Constant::Int(1));

    funcsig!        ((vm) simple_spill_sig = () -> (int64));
    funcdecl!       ((vm) <simple_spill_sig> simple_spill);
    funcdef!        ((vm) <simple_spill_sig> simple_spill VERSION simple_spill_v1);

    // %entry():
    block!          ((vm, simple_spill_v1) blk_entry);

    // BRANCH %start(1, 1, 1, 1, ..., 1) // 14 constant ONE
    consta!         ((vm, simple_spill_v1) int64_1_local = int64_1);
    block!          ((vm, simple_spill_v1) blk_start);
    inst!           ((vm, simple_spill_v1) blk_entry_branch:
        BRANCH blk_start (int64_1_local, int64_1_local, int64_1_local,
                          int64_1_local, int64_1_local, int64_1_local,
                          int64_1_local, int64_1_local, int64_1_local,
                          int64_1_local, int64_1_local, int64_1_local,
                          int64_1_local, int64_1_local
        )
    );

    define_block!   ((vm, simple_spill_v1) blk_entry() {
        blk_entry_branch
    });

    // %start(%t1, %t2, ..., %t14):
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t1);
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t2);
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t3);
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t4);
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t5);
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t6);
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t7);
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t8);
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t9);
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t10);
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t11);
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t12);
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t13);
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t14);

    // %res = ADD %t1 %t2
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_res);
    inst!           ((vm, simple_spill_v1) blk_start_add:
        blk_start_res = BINOP (BinOp::Add) blk_start_t1 blk_start_t2
    );

    // BRANCH %ret (%res, %t1, %t2, ..., %t14)
    block!          ((vm, simple_spill_v1) blk_ret);
    inst!           ((vm, simple_spill_v1) blk_start_branch:
        BRANCH blk_ret (
            blk_start_res,
            blk_start_t1,
            blk_start_t2,
            blk_start_t3,
            blk_start_t4,
            blk_start_t5,
            blk_start_t6,
            blk_start_t7,
            blk_start_t8,
            blk_start_t9,
            blk_start_t10,
            blk_start_t11,
            blk_start_t12,
            blk_start_t13,
            blk_start_t14
        )
    );

    define_block!   ((vm, simple_spill_v1) blk_start(
        blk_start_t1,
        blk_start_t2,
        blk_start_t3,
        blk_start_t4,
        blk_start_t5,
        blk_start_t6,
        blk_start_t7,
        blk_start_t8,
        blk_start_t9,
        blk_start_t10,
        blk_start_t11,
        blk_start_t12,
        blk_start_t13,
        blk_start_t14) {
            blk_start_add,
            blk_start_branch
        }
     );

    // %ret(%res, %t1, %t2, ... %t14):
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_res);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t1);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t2);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t3);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t4);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t5);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t6);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t7);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t8);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t9);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t10);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t11);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t12);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t13);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t14);

    // RET %res
    inst!           ((vm, simple_spill_v1) blk_ret_ret:
        RET (blk_ret_res)
    );

    define_block!   ((vm, simple_spill_v1) blk_ret
        (blk_ret_res,
         blk_ret_t1,
         blk_ret_t2,
         blk_ret_t3,
         blk_ret_t4,
         blk_ret_t5,
         blk_ret_t6,
         blk_ret_t7,
         blk_ret_t8,
         blk_ret_t9,
         blk_ret_t10,
         blk_ret_t11,
         blk_ret_t12,
         blk_ret_t13,
         blk_ret_t14
        ) {
            blk_ret_ret
        }
     );

    define_func_ver!((vm) simple_spill_v1 (entry: blk_entry) {
        blk_entry,
        blk_start,
        blk_ret
    });

    vm
}


#[cfg(target_arch = "aarch64")]
fn create_simple_spill() -> VM {
    let vm = VM::new();

    typedef!        ((vm) int64 = mu_int(64));
    constdef!       ((vm) <int64> int64_1 = Constant::Int(1));

    funcsig!        ((vm) simple_spill_sig = () -> (int64));
    funcdecl!       ((vm) <simple_spill_sig> simple_spill);
    funcdef!        ((vm) <simple_spill_sig> simple_spill VERSION simple_spill_v1);

    // %entry():
    block!          ((vm, simple_spill_v1) blk_entry);

    // BRANCH %start(1, 1, 1, 1, ..., 1) // 28 constant ONE
    consta!         ((vm, simple_spill_v1) int64_1_local = int64_1);
    block!          ((vm, simple_spill_v1) blk_start);
    inst!           ((vm, simple_spill_v1) blk_entry_branch:
        BRANCH blk_start (int64_1_local, int64_1_local, int64_1_local,
                          int64_1_local, int64_1_local, int64_1_local,
                          int64_1_local, int64_1_local, int64_1_local,
                          int64_1_local, int64_1_local, int64_1_local,
                          int64_1_local, int64_1_local, int64_1_local,
                          int64_1_local, int64_1_local, int64_1_local,
                          int64_1_local, int64_1_local, int64_1_local,
                          int64_1_local, int64_1_local, int64_1_local,
                          int64_1_local, int64_1_local, int64_1_local,
                          int64_1_local
        )
    );

    define_block!   ((vm, simple_spill_v1) blk_entry() {
        blk_entry_branch
    });

    // %start(%t1, %t2, ..., %t14):
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t1);
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t2);
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t3);
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t4);
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t5);
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t6);
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t7);
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t8);
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t9);
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t10);
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t11);
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t12);
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t13);
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t14);
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t15);
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t16);
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t17);
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t18);
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t19);
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t20);
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t21);
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t22);
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t23);
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t24);
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t25);
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t26);
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t27);
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_t28);


    // %res = ADD %t1 %t2
    ssa!            ((vm, simple_spill_v1) <int64> blk_start_res);
    inst!           ((vm, simple_spill_v1) blk_start_add:
        blk_start_res = BINOP (BinOp::Add) blk_start_t1 blk_start_t2
    );

    // BRANCH %ret (%res, %t1, %t2, ..., %t14)
    block!          ((vm, simple_spill_v1) blk_ret);
    inst!           ((vm, simple_spill_v1) blk_start_branch:
        BRANCH blk_ret (
            blk_start_res,
            blk_start_t1,
            blk_start_t2,
            blk_start_t3,
            blk_start_t4,
            blk_start_t5,
            blk_start_t6,
            blk_start_t7,
            blk_start_t8,
            blk_start_t9,
            blk_start_t10,
            blk_start_t11,
            blk_start_t12,
            blk_start_t13,
            blk_start_t14,
            blk_start_t15,
            blk_start_t16,
            blk_start_t17,
            blk_start_t18,
            blk_start_t19,
            blk_start_t20,
            blk_start_t21,
            blk_start_t22,
            blk_start_t23,
            blk_start_t24,
            blk_start_t25,
            blk_start_t26,
            blk_start_t27,
            blk_start_t28
        )
    );

    define_block!   ((vm, simple_spill_v1) blk_start(
        blk_start_t1,
        blk_start_t2,
        blk_start_t3,
        blk_start_t4,
        blk_start_t5,
        blk_start_t6,
        blk_start_t7,
        blk_start_t8,
        blk_start_t9,
        blk_start_t10,
        blk_start_t11,
        blk_start_t12,
        blk_start_t13,
        blk_start_t14,
        blk_start_t15,
        blk_start_t16,
        blk_start_t17,
        blk_start_t18,
        blk_start_t19,
        blk_start_t20,
        blk_start_t21,
        blk_start_t22,
        blk_start_t23,
        blk_start_t24,
        blk_start_t25,
        blk_start_t26,
        blk_start_t27,
        blk_start_t28) {
            blk_start_add,
            blk_start_branch
        }
     );

    // %ret(%res, %t1, %t2, ... %t14):
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_res);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t1);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t2);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t3);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t4);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t5);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t6);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t7);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t8);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t9);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t10);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t11);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t12);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t13);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t14);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t15);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t16);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t17);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t18);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t19);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t20);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t21);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t22);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t23);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t24);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t25);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t26);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t27);
    ssa!            ((vm, simple_spill_v1) <int64> blk_ret_t28);

    // RET %res
    inst!           ((vm, simple_spill_v1) blk_ret_ret:
        RET (blk_ret_res)
    );

    define_block!   ((vm, simple_spill_v1) blk_ret
        (blk_ret_res,
         blk_ret_t1,
         blk_ret_t2,
         blk_ret_t3,
         blk_ret_t4,
         blk_ret_t5,
         blk_ret_t6,
         blk_ret_t7,
         blk_ret_t8,
         blk_ret_t9,
         blk_ret_t10,
         blk_ret_t11,
         blk_ret_t12,
         blk_ret_t13,
         blk_ret_t14,
         blk_ret_t15,
         blk_ret_t16,
         blk_ret_t17,
         blk_ret_t18,
         blk_ret_t19,
         blk_ret_t20,
         blk_ret_t21,
         blk_ret_t22,
         blk_ret_t23,
         blk_ret_t24,
         blk_ret_t25,
         blk_ret_t26,
         blk_ret_t27,
         blk_ret_t28
        ) {
            blk_ret_ret
        }
     );

    define_func_ver!((vm) simple_spill_v1 (entry: blk_entry) {
        blk_entry,
        blk_start,
        blk_ret
    });

    vm
}

#[test] // was x86-64 only
fn test_coalesce_branch_moves() {
    VM::start_logging_trace();

    let vm = Arc::new(coalesce_branch_moves());

    let compiler = Compiler::new(CompilerPolicy::default(), &vm);

    let func_id = vm.id_of("coalesce_branch_moves");
    {
        let funcs = vm.funcs().read().unwrap();
        let func = funcs.get(&func_id).unwrap().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();
        let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();

        compiler.compile(&mut func_ver);

        // check
        let fv_id = func_ver.id();

        assert!(get_number_of_moves(fv_id, &vm) == 1, "The function should not yield any mov instructions other than mov %rsp->%rbp (some possible coalescing failed)");
    }
}

fn coalesce_branch_moves() -> VM {
    let vm = VM::new();

    typedef! ((vm) int64 = mu_int(64));

    funcsig! ((vm) sig = (int64, int64, int64, int64) -> ());
    funcdecl!((vm) <sig> coalesce_branch_moves);
    funcdef! ((vm) <sig> coalesce_branch_moves VERSION coalesce_branch_moves_v1);

    // blk entry
    block!   ((vm, coalesce_branch_moves_v1) blk_entry);
    ssa!     ((vm, coalesce_branch_moves_v1) <int64> arg0);
    ssa!     ((vm, coalesce_branch_moves_v1) <int64> arg1);
    ssa!     ((vm, coalesce_branch_moves_v1) <int64> arg2);
    ssa!     ((vm, coalesce_branch_moves_v1) <int64> arg3);

    block!   ((vm, coalesce_branch_moves_v1) blk1);
    inst!    ((vm, coalesce_branch_moves_v1) blk_entry_branch:
        BRANCH blk1 (arg0, arg1, arg2, arg3)
    );

    define_block!((vm, coalesce_branch_moves_v1) blk_entry (arg0, arg1, arg2, arg3) {blk_entry_branch});

    ssa!     ((vm, coalesce_branch_moves_v1) <int64> blk1_arg0);
    ssa!     ((vm, coalesce_branch_moves_v1) <int64> blk1_arg1);
    ssa!     ((vm, coalesce_branch_moves_v1) <int64> blk1_arg2);
    ssa!     ((vm, coalesce_branch_moves_v1) <int64> blk1_arg3);

    inst!    ((vm, coalesce_branch_moves_v1) blk1_ret:
        RET
    );

    define_block!((vm, coalesce_branch_moves_v1) blk1 (blk1_arg0, blk1_arg1, blk1_arg2, blk1_arg3) {
        blk1_ret
    });

    define_func_ver!((vm) coalesce_branch_moves_v1 (entry: blk_entry){
        blk_entry, blk1
    });

    vm
}

#[test] // was x86-64 only
fn test_coalesce_args() {
    VM::start_logging_trace();

    let vm = Arc::new(coalesce_args());

    let compiler = Compiler::new(CompilerPolicy::default(), &vm);

    let func_id = vm.id_of("coalesce_args");
    {
        let funcs = vm.funcs().read().unwrap();
        let func = funcs.get(&func_id).unwrap().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();
        let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();

        compiler.compile(&mut func_ver);

        // check
        let fv_id = func_ver.id();

        assert!(get_number_of_moves(fv_id, &vm) == 1, "The function should not yield any mov instructions other than mov %rsp->%rbp (or MOV SP -> FP on aarch64) (some possible coalescing failed)");
    }
}

fn coalesce_args() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int64 = mu_int(64));

    funcsig!    ((vm) sig = (int64, int64, int64, int64) -> ());
    funcdecl!   ((vm) <sig> coalesce_args);
    funcdef!    ((vm) <sig> coalesce_args VERSION coalesce_args_v1);

    typedef!    ((vm) funcref_to_sig = mu_funcref(sig));
    constdef!   ((vm) <funcref_to_sig> funcref = Constant::FuncRef(coalesce_args));

    // blk entry
    block!      ((vm, coalesce_args_v1) blk_entry);
    ssa!        ((vm, coalesce_args_v1) <int64> arg0);
    ssa!        ((vm, coalesce_args_v1) <int64> arg1);
    ssa!        ((vm, coalesce_args_v1) <int64> arg2);
    ssa!        ((vm, coalesce_args_v1) <int64> arg3);

    consta!     ((vm, coalesce_args_v1) funcref_local = funcref);
    inst!       ((vm, coalesce_args_v1) blk_entry_call:
        EXPRCALL (CallConvention::Mu, is_abort: false) funcref_local (arg0, arg1, arg2, arg3)
    );

    inst!       ((vm, coalesce_args_v1) blk_entry_ret:
        RET
    );

    define_block!   ((vm, coalesce_args_v1) blk_entry(arg0, arg1, arg2, arg3) {blk_entry_call, blk_entry_ret});

    define_func_ver!((vm) coalesce_args_v1 (entry: blk_entry) {blk_entry});

    vm
}

#[test] // was x86_64 only
fn test_coalesce_branch2_moves() {
    VM::start_logging_trace();

    let vm = Arc::new(coalesce_branch2_moves());

    let compiler = Compiler::new(CompilerPolicy::default(), &vm);

    let func_id = vm.id_of("coalesce_branch2_moves");
    {
        let funcs = vm.funcs().read().unwrap();
        let func = funcs.get(&func_id).unwrap().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();
        let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();

        compiler.compile(&mut func_ver);

        // check
        let fv_id = func_ver.id();

        assert!(get_number_of_moves(fv_id, &vm) <= 3, "too many moves (some possible coalescing failed)");
    }

    backend::emit_context(&vm);

    let dylib = aot::link_dylib(vec![Mu("coalesce_branch2_moves")], &testutil::get_dylib_name("coalesce_branch2_moves"), &vm);

    let lib = libloading::Library::new(dylib.as_os_str()).unwrap();
    unsafe {
        let coalesce_branch2_moves : libloading::Symbol<unsafe extern fn(u64, u64, u64, u64, u64, u64) -> u64> = match lib.get(b"coalesce_branch2_moves") {
            Ok(symbol) => symbol,
            Err(e) => panic!("cannot find symbol coalesce_branch2_moves in dylib: {:?}", e)
        };

        let res = coalesce_branch2_moves(1, 1, 10, 10, 0, 0);
        println!("if 0 == 0 then return 1 + 1 else return 10 + 10");
        println!("coalesce_branch2_moves(1, 1, 10, 10, 0, 0) = {}", res);
        assert!(res == 2);

        let res = coalesce_branch2_moves(1, 1, 10, 10, 1, 0);
        println!("if 1 == 0 then return 1 + 1 else return 10 + 10");
        println!("coalesce_branch2_moves(1, 1, 10, 10, 1, 0) = {}", res);
        assert!(res == 20);
    }
}

fn coalesce_branch2_moves() -> VM {
    let vm = VM::new();

    typedef! ((vm) int64 = mu_int(64));
    typedef! ((vm) int1  = mu_int(1));

    funcsig! ((vm) sig = (int64, int64, int64, int64, int64, int64) -> ());
    funcdecl!((vm) <sig> coalesce_branch2_moves);
    funcdef! ((vm) <sig> coalesce_branch2_moves VERSION coalesce_branch2_moves_v1);

    // blk entry
    block!   ((vm, coalesce_branch2_moves_v1) blk_entry);
    ssa!     ((vm, coalesce_branch2_moves_v1) <int64> arg0);
    ssa!     ((vm, coalesce_branch2_moves_v1) <int64> arg1);
    ssa!     ((vm, coalesce_branch2_moves_v1) <int64> arg2);
    ssa!     ((vm, coalesce_branch2_moves_v1) <int64> arg3);
    ssa!     ((vm, coalesce_branch2_moves_v1) <int64> arg4);
    ssa!     ((vm, coalesce_branch2_moves_v1) <int64> arg5);

    ssa!     ((vm, coalesce_branch2_moves_v1) <int1> cond);
    inst!    ((vm, coalesce_branch2_moves_v1) blk_entry_cmp:
        cond = CMPOP (CmpOp::EQ) arg4 arg5
    );

    block!   ((vm, coalesce_branch2_moves_v1) blk_add01);
    block!   ((vm, coalesce_branch2_moves_v1) blk_add23);
    block!   ((vm, coalesce_branch2_moves_v1) blk_ret);
    inst!    ((vm, coalesce_branch2_moves_v1) blk_entry_branch2:
        BRANCH2 (cond, arg0, arg1, arg2, arg3)
            IF (OP 0)
            THEN blk_add01 (vec![1, 2]) WITH 0.6f32,
            ELSE blk_add23 (vec![3, 4])
    );

    define_block!((vm, coalesce_branch2_moves_v1) blk_entry (arg0, arg1, arg2, arg3, arg4, arg5) {
        blk_entry_cmp, blk_entry_branch2
    });

    // blk_add01
    ssa!     ((vm, coalesce_branch2_moves_v1) <int64> blk_add01_arg0);
    ssa!     ((vm, coalesce_branch2_moves_v1) <int64> blk_add01_arg1);

    ssa!     ((vm, coalesce_branch2_moves_v1) <int64> res01);
    inst!    ((vm, coalesce_branch2_moves_v1) blk_add01_add:
        res01 = BINOP (BinOp::Add) blk_add01_arg0 blk_add01_arg1
    );

    inst!    ((vm, coalesce_branch2_moves_v1) blk_add01_branch:
        BRANCH blk_ret (res01)
    );

    define_block!((vm, coalesce_branch2_moves_v1) blk_add01 (blk_add01_arg0, blk_add01_arg1) {
        blk_add01_add, blk_add01_branch
    });

    // blk_add23
    ssa!     ((vm, coalesce_branch2_moves_v1) <int64> blk_add23_arg2);
    ssa!     ((vm, coalesce_branch2_moves_v1) <int64> blk_add23_arg3);

    ssa!     ((vm, coalesce_branch2_moves_v1) <int64> res23);
    inst!    ((vm, coalesce_branch2_moves_v1) blk_add23_add:
        res23 = BINOP (BinOp::Add) blk_add23_arg2 blk_add23_arg3
    );

    inst!    ((vm, coalesce_branch2_moves_v1) blk_add23_branch:
        BRANCH blk_ret (res23)
    );

    define_block!((vm, coalesce_branch2_moves_v1) blk_add23 (blk_add23_arg2, blk_add23_arg3) {
        blk_add23_add, blk_add23_branch
    });

    // blk_ret
    ssa!     ((vm, coalesce_branch2_moves_v1) <int64> res);
    inst!    ((vm, coalesce_branch2_moves_v1) blk_ret_ret:
        RET (res)
    );

    define_block!((vm, coalesce_branch2_moves_v1) blk_ret (res) {
        blk_ret_ret
    });

    define_func_ver!((vm) coalesce_branch2_moves_v1 (entry: blk_entry){
        blk_entry, blk_add01, blk_add23, blk_ret
    });

    vm
}

#[test]
fn test_preserve_caller_saved_simple() {
    VM::start_logging_trace();
    let vm = Arc::new(preserve_caller_saved_simple());

    let compiler = Compiler::new(CompilerPolicy::default(), &vm);

    let func_foo = vm.id_of("foo");
    let func_preserve_caller_saved_simple = vm.id_of("preserve_caller_saved_simple");
    {
        let funcs = vm.funcs().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();

        {
            let func = funcs.get(&func_foo).unwrap().read().unwrap();
            let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();

            compiler.compile(&mut func_ver);
        }
        {
            let func = funcs.get(&func_preserve_caller_saved_simple).unwrap().read().unwrap();
            let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();

            compiler.compile(&mut func_ver);
        }
    }

    vm.make_primordial_thread(func_preserve_caller_saved_simple, true, vec![]);
    backend::emit_context(&vm);

    let executable = aot::link_primordial(vec![Mu("foo"), Mu("preserve_caller_saved_simple")], "test_preserve_caller_saved_simple", &vm);
    let output = aot::execute_nocheck(executable);

    // add from 0 to 9
    assert!(output.status.code().is_some());
    assert_eq!(output.status.code().unwrap(), 45);
}

fn preserve_caller_saved_simple() -> VM {
    let vm = VM::new_with_opts("init_mu --disable-inline");

    create_empty_func_foo(&vm);

    typedef!    ((vm) int64 = mu_int(64));

    constdef!   ((vm) <int64> int64_0 = Constant::Int(0));
    constdef!   ((vm) <int64> int64_1 = Constant::Int(1));
    constdef!   ((vm) <int64> int64_2 = Constant::Int(2));
    constdef!   ((vm) <int64> int64_3 = Constant::Int(3));
    constdef!   ((vm) <int64> int64_4 = Constant::Int(4));
    constdef!   ((vm) <int64> int64_5 = Constant::Int(5));
    constdef!   ((vm) <int64> int64_6 = Constant::Int(6));
    constdef!   ((vm) <int64> int64_7 = Constant::Int(7));
    constdef!   ((vm) <int64> int64_8 = Constant::Int(8));
    constdef!   ((vm) <int64> int64_9 = Constant::Int(9));

    funcsig!    ((vm) sig = () -> ());
    funcdecl!   ((vm) <sig> preserve_caller_saved_simple);
    funcdef!    ((vm) <sig> preserve_caller_saved_simple VERSION preserve_caller_saved_simple_v1);

    // blk entry
    block!      ((vm, preserve_caller_saved_simple_v1) blk_entry);
    block!      ((vm, preserve_caller_saved_simple_v1) blk_main);

    consta!   ((vm, preserve_caller_saved_simple_v1)  int64_0_local = int64_0);
    consta!   ((vm, preserve_caller_saved_simple_v1)  int64_1_local = int64_1);
    consta!   ((vm, preserve_caller_saved_simple_v1)  int64_2_local = int64_2);
    consta!   ((vm, preserve_caller_saved_simple_v1)  int64_3_local = int64_3);
    consta!   ((vm, preserve_caller_saved_simple_v1)  int64_4_local = int64_4);
    consta!   ((vm, preserve_caller_saved_simple_v1)  int64_5_local = int64_5);
    consta!   ((vm, preserve_caller_saved_simple_v1)  int64_6_local = int64_6);
    consta!   ((vm, preserve_caller_saved_simple_v1)  int64_7_local = int64_7);
    consta!   ((vm, preserve_caller_saved_simple_v1)  int64_8_local = int64_8);
    consta!   ((vm, preserve_caller_saved_simple_v1)  int64_9_local = int64_9);
    inst!       ((vm, preserve_caller_saved_simple_v1) blk_entry_branch:
        BRANCH blk_main (
             int64_0_local,
             int64_1_local,
             int64_2_local,
             int64_3_local,
             int64_4_local,
             int64_5_local,
             int64_6_local,
             int64_7_local,
             int64_8_local,
             int64_9_local
        )
    );

    define_block!   ((vm, preserve_caller_saved_simple_v1) blk_entry() {
        blk_entry_branch
    });

    // blk main
    ssa!    ((vm, preserve_caller_saved_simple_v1) <int64> v0);
    ssa!    ((vm, preserve_caller_saved_simple_v1) <int64> v1);
    ssa!    ((vm, preserve_caller_saved_simple_v1) <int64> v2);
    ssa!    ((vm, preserve_caller_saved_simple_v1) <int64> v3);
    ssa!    ((vm, preserve_caller_saved_simple_v1) <int64> v4);
    ssa!    ((vm, preserve_caller_saved_simple_v1) <int64> v5);
    ssa!    ((vm, preserve_caller_saved_simple_v1) <int64> v6);
    ssa!    ((vm, preserve_caller_saved_simple_v1) <int64> v7);
    ssa!    ((vm, preserve_caller_saved_simple_v1) <int64> v8);
    ssa!    ((vm, preserve_caller_saved_simple_v1) <int64> v9);

    let foo_sig = vm.get_func_sig(vm.id_of("foo_sig"));
    let foo_id  = vm.id_of("foo");
    typedef!    ((vm) type_funcref_foo = mu_funcref(foo_sig));
    constdef!   ((vm) <type_funcref_foo> const_funcref_foo = Constant::FuncRef(foo_id));

    consta!     ((vm, preserve_caller_saved_simple_v1) const_funcref_foo_local = const_funcref_foo);
    inst!       ((vm, preserve_caller_saved_simple_v1) blk_main_call:
        EXPRCALL (CallConvention::Mu, is_abort: false) const_funcref_foo_local ()
    );

    ssa!        ((vm, preserve_caller_saved_simple_v1) <int64> res1);
    inst!       ((vm, preserve_caller_saved_simple_v1) blk_main_add1:
        res1 = BINOP (BinOp::Add) v0 v1
    );

    ssa!        ((vm, preserve_caller_saved_simple_v1) <int64> res2);
    inst!       ((vm, preserve_caller_saved_simple_v1) blk_main_add2:
        res2 = BINOP (BinOp::Add) res1 v2
    );

    ssa!        ((vm, preserve_caller_saved_simple_v1) <int64> res3);
    inst!       ((vm, preserve_caller_saved_simple_v1) blk_main_add3:
        res3 = BINOP (BinOp::Add) res2 v3
    );

    ssa!        ((vm, preserve_caller_saved_simple_v1) <int64> res4);
    inst!       ((vm, preserve_caller_saved_simple_v1) blk_main_add4:
        res4 = BINOP (BinOp::Add) res3 v4
    );

    ssa!        ((vm, preserve_caller_saved_simple_v1) <int64> res5);
    inst!       ((vm, preserve_caller_saved_simple_v1) blk_main_add5:
        res5 = BINOP (BinOp::Add) res4 v5
    );

    ssa!        ((vm, preserve_caller_saved_simple_v1) <int64> res6);
    inst!       ((vm, preserve_caller_saved_simple_v1) blk_main_add6:
        res6 = BINOP (BinOp::Add) res5 v6
    );

    ssa!        ((vm, preserve_caller_saved_simple_v1) <int64> res7);
    inst!       ((vm, preserve_caller_saved_simple_v1) blk_main_add7:
        res7 = BINOP (BinOp::Add) res6 v7
    );

    ssa!        ((vm, preserve_caller_saved_simple_v1) <int64> res8);
    inst!       ((vm, preserve_caller_saved_simple_v1) blk_main_add8:
        res8 = BINOP (BinOp::Add) res7 v8
    );

    ssa!        ((vm, preserve_caller_saved_simple_v1) <int64> res9);
    inst!       ((vm, preserve_caller_saved_simple_v1) blk_main_add9:
        res9 = BINOP (BinOp::Add) res8 v9
    );

    let blk_main_exit = gen_ccall_exit(res9.clone(), &mut preserve_caller_saved_simple_v1, &vm);

    inst!       ((vm, preserve_caller_saved_simple_v1) blk_main_ret:
        RET
    );

    define_block!   ((vm, preserve_caller_saved_simple_v1) blk_main(v0, v1, v2, v3, v4, v5, v6, v7, v8, v9) {
        blk_main_call,

        blk_main_add1,
        blk_main_add2,
        blk_main_add3,
        blk_main_add4,
        blk_main_add5,
        blk_main_add6,
        blk_main_add7,
        blk_main_add8,
        blk_main_add9,

        blk_main_exit,
        blk_main_ret
    });

    define_func_ver!((vm) preserve_caller_saved_simple_v1 (entry: blk_entry) {
        blk_entry,
        blk_main
    });

    vm
}

fn create_empty_func_foo(vm: &VM) {
    funcsig!    ((vm) foo_sig = () -> ());
    funcdecl!   ((vm) <foo_sig> foo);
    funcdef!    ((vm) <foo_sig> foo VERSION foo_v1);

    block!      ((vm, foo_v1) blk_entry);
    inst!       ((vm, foo_v1) blk_entry_ret:
        RET
    );

    define_block!   ((vm, foo_v1) blk_entry() {
        blk_entry_ret
    });

    define_func_ver!((vm) foo_v1 (entry: blk_entry) {blk_entry});
}

#[test]
fn test_preserve_caller_saved_call_args() {
    VM::start_logging_trace();
    let vm = Arc::new(preserve_caller_saved_call_args());

    let compiler = Compiler::new(CompilerPolicy::default(), &vm);

    let func_foo = vm.id_of("foo6");
    let func_preserve_caller_saved_simple = vm.id_of("preserve_caller_saved_call_args");
    {
        let funcs = vm.funcs().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();

        {
            let func = funcs.get(&func_foo).unwrap().read().unwrap();
            let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();

            compiler.compile(&mut func_ver);
        }
        {
            let func = funcs.get(&func_preserve_caller_saved_simple).unwrap().read().unwrap();
            let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();

            compiler.compile(&mut func_ver);
        }
    }

    vm.make_primordial_thread(func_preserve_caller_saved_simple, true, vec![]);
    backend::emit_context(&vm);

    let executable = aot::link_primordial(vec![Mu("foo6"), Mu("preserve_caller_saved_call_args")], "test_preserve_caller_saved_call_args", &vm);
    let output = aot::execute_nocheck(executable);

    // add from 0 to 9
    assert!(output.status.code().is_some());
    assert_eq!(output.status.code().unwrap(), 45);
}

fn preserve_caller_saved_call_args() -> VM {
    let vm = VM::new_with_opts("init_mu --disable-inline");

    typedef!    ((vm) int64 = mu_int(64));

    create_empty_func_foo6(&vm);

    constdef!   ((vm) <int64> int64_0 = Constant::Int(0));
    constdef!   ((vm) <int64> int64_1 = Constant::Int(1));
    constdef!   ((vm) <int64> int64_2 = Constant::Int(2));
    constdef!   ((vm) <int64> int64_3 = Constant::Int(3));
    constdef!   ((vm) <int64> int64_4 = Constant::Int(4));
    constdef!   ((vm) <int64> int64_5 = Constant::Int(5));
    constdef!   ((vm) <int64> int64_6 = Constant::Int(6));
    constdef!   ((vm) <int64> int64_7 = Constant::Int(7));
    constdef!   ((vm) <int64> int64_8 = Constant::Int(8));
    constdef!   ((vm) <int64> int64_9 = Constant::Int(9));

    funcsig!    ((vm) sig = () -> ());
    funcdecl!   ((vm) <sig> preserve_caller_saved_call_args);
    funcdef!    ((vm) <sig> preserve_caller_saved_call_args VERSION preserve_caller_saved_call_args_v1);

    // blk entry
    block!      ((vm, preserve_caller_saved_call_args_v1) blk_entry);
    block!      ((vm, preserve_caller_saved_call_args_v1) blk_main);

    consta!   ((vm, preserve_caller_saved_call_args_v1)  int64_0_local = int64_0);
    consta!   ((vm, preserve_caller_saved_call_args_v1)  int64_1_local = int64_1);
    consta!   ((vm, preserve_caller_saved_call_args_v1)  int64_2_local = int64_2);
    consta!   ((vm, preserve_caller_saved_call_args_v1)  int64_3_local = int64_3);
    consta!   ((vm, preserve_caller_saved_call_args_v1)  int64_4_local = int64_4);
    consta!   ((vm, preserve_caller_saved_call_args_v1)  int64_5_local = int64_5);
    consta!   ((vm, preserve_caller_saved_call_args_v1)  int64_6_local = int64_6);
    consta!   ((vm, preserve_caller_saved_call_args_v1)  int64_7_local = int64_7);
    consta!   ((vm, preserve_caller_saved_call_args_v1)  int64_8_local = int64_8);
    consta!   ((vm, preserve_caller_saved_call_args_v1)  int64_9_local = int64_9);
    inst!       ((vm, preserve_caller_saved_call_args_v1) blk_entry_branch:
        BRANCH blk_main (
             int64_0_local,
             int64_1_local,
             int64_2_local,
             int64_3_local,
             int64_4_local,
             int64_5_local,
             int64_6_local,
             int64_7_local,
             int64_8_local,
             int64_9_local
        )
    );

    define_block!   ((vm, preserve_caller_saved_call_args_v1) blk_entry() {
        blk_entry_branch
    });

    // blk main
    ssa!    ((vm, preserve_caller_saved_call_args_v1) <int64> v0);
    ssa!    ((vm, preserve_caller_saved_call_args_v1) <int64> v1);
    ssa!    ((vm, preserve_caller_saved_call_args_v1) <int64> v2);
    ssa!    ((vm, preserve_caller_saved_call_args_v1) <int64> v3);
    ssa!    ((vm, preserve_caller_saved_call_args_v1) <int64> v4);
    ssa!    ((vm, preserve_caller_saved_call_args_v1) <int64> v5);
    ssa!    ((vm, preserve_caller_saved_call_args_v1) <int64> v6);
    ssa!    ((vm, preserve_caller_saved_call_args_v1) <int64> v7);
    ssa!    ((vm, preserve_caller_saved_call_args_v1) <int64> v8);
    ssa!    ((vm, preserve_caller_saved_call_args_v1) <int64> v9);

    let foo_sig = vm.get_func_sig(vm.id_of("foo6_sig"));
    let foo_id  = vm.id_of("foo6");
    typedef!    ((vm) type_funcref_foo = mu_funcref(foo_sig));
    constdef!   ((vm) <type_funcref_foo> const_funcref_foo = Constant::FuncRef(foo_id));

    consta!     ((vm, preserve_caller_saved_call_args_v1) const_funcref_foo_local = const_funcref_foo);
    inst!       ((vm, preserve_caller_saved_call_args_v1) blk_main_call:
        EXPRCALL (CallConvention::Mu, is_abort: false) const_funcref_foo_local (v0, v1, v2, v3, v4, v5)
    );

    ssa!        ((vm, preserve_caller_saved_call_args_v1) <int64> res1);
    inst!       ((vm, preserve_caller_saved_call_args_v1) blk_main_add1:
        res1 = BINOP (BinOp::Add) v0 v1
    );

    ssa!        ((vm, preserve_caller_saved_call_args_v1) <int64> res2);
    inst!       ((vm, preserve_caller_saved_call_args_v1) blk_main_add2:
        res2 = BINOP (BinOp::Add) res1 v2
    );

    ssa!        ((vm, preserve_caller_saved_call_args_v1) <int64> res3);
    inst!       ((vm, preserve_caller_saved_call_args_v1) blk_main_add3:
        res3 = BINOP (BinOp::Add) res2 v3
    );

    ssa!        ((vm, preserve_caller_saved_call_args_v1) <int64> res4);
    inst!       ((vm, preserve_caller_saved_call_args_v1) blk_main_add4:
        res4 = BINOP (BinOp::Add) res3 v4
    );

    ssa!        ((vm, preserve_caller_saved_call_args_v1) <int64> res5);
    inst!       ((vm, preserve_caller_saved_call_args_v1) blk_main_add5:
        res5 = BINOP (BinOp::Add) res4 v5
    );

    ssa!        ((vm, preserve_caller_saved_call_args_v1) <int64> res6);
    inst!       ((vm, preserve_caller_saved_call_args_v1) blk_main_add6:
        res6 = BINOP (BinOp::Add) res5 v6
    );

    ssa!        ((vm, preserve_caller_saved_call_args_v1) <int64> res7);
    inst!       ((vm, preserve_caller_saved_call_args_v1) blk_main_add7:
        res7 = BINOP (BinOp::Add) res6 v7
    );

    ssa!        ((vm, preserve_caller_saved_call_args_v1) <int64> res8);
    inst!       ((vm, preserve_caller_saved_call_args_v1) blk_main_add8:
        res8 = BINOP (BinOp::Add) res7 v8
    );

    ssa!        ((vm, preserve_caller_saved_call_args_v1) <int64> res9);
    inst!       ((vm, preserve_caller_saved_call_args_v1) blk_main_add9:
        res9 = BINOP (BinOp::Add) res8 v9
    );

    let blk_main_exit = gen_ccall_exit(res9.clone(), &mut preserve_caller_saved_call_args_v1, &vm);

    inst!       ((vm, preserve_caller_saved_call_args_v1) blk_main_ret:
        RET
    );

    define_block!   ((vm, preserve_caller_saved_call_args_v1) blk_main(v0, v1, v2, v3, v4, v5, v6, v7, v8, v9) {
        blk_main_call,

        blk_main_add1,
        blk_main_add2,
        blk_main_add3,
        blk_main_add4,
        blk_main_add5,
        blk_main_add6,
        blk_main_add7,
        blk_main_add8,
        blk_main_add9,

        blk_main_exit,
        blk_main_ret
    });

    define_func_ver!((vm) preserve_caller_saved_call_args_v1 (entry: blk_entry) {
        blk_entry,
        blk_main
    });

    vm
}

fn create_empty_func_foo6(vm: &VM) {
    let int64 = vm.get_type(vm.id_of("int64"));

    funcsig!    ((vm) foo6_sig = (int64, int64, int64, int64, int64, int64) -> ());
    funcdecl!   ((vm) <foo6_sig> foo6);
    funcdef!    ((vm) <foo6_sig> foo6 VERSION foo6_v1);

    block!      ((vm, foo6_v1) blk_entry);
    inst!       ((vm, foo6_v1) blk_entry_ret:
        RET
    );

    define_block!   ((vm, foo6_v1) blk_entry() {
        blk_entry_ret
    });

    define_func_ver!((vm) foo6_v1 (entry: blk_entry) {blk_entry});
}


#[test]
#[allow(unused_variables)]
#[allow(overflowing_literals)]
fn test_spill_int8() {
    VM::start_logging_trace();

    let vm = Arc::new(spill_int8());

    let compiler = Compiler::new(CompilerPolicy::default(), &vm);

    let func_id = vm.id_of("spill_int8");
    {
        let funcs = vm.funcs().read().unwrap();
        let func = funcs.get(&func_id).unwrap().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();
        let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();

        compiler.compile(&mut func_ver);
    }

    backend::emit_context(&vm);

    let dylib = aot::link_dylib(vec![Mu("spill_int8")], &testutil::get_dylib_name("spill_int8"), &vm);

    let lib = libloading::Library::new(dylib.as_os_str()).unwrap();
    unsafe {
        let spill_int8 : libloading::Symbol<unsafe extern fn() -> u8> = match lib.get(b"spill_int8") {
            Ok(symbol) => symbol,
            Err(e) => panic!("cannot find symbol spill_int8 in dylib: {:?}", e)
        };

        let res = spill_int8();
        println!("spill_int8() = {}", res);
        if cfg!(target_arch = "x86_64") {
            assert_eq!(res, 136); // add from 0 to 16
        } else {
            //Note: 465 does not fit in a u8
            assert_eq!(res, 465u8); // add from 0 to 30
        }
    }
}

#[cfg(target_arch = "x86_64")]
fn spill_int8() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int8 = mu_int(8));

    constdef!   ((vm) <int8> int8_0  = Constant::Int(0));
    constdef!   ((vm) <int8> int8_1  = Constant::Int(1));
    constdef!   ((vm) <int8> int8_2  = Constant::Int(2));
    constdef!   ((vm) <int8> int8_3  = Constant::Int(3));
    constdef!   ((vm) <int8> int8_4  = Constant::Int(4));
    constdef!   ((vm) <int8> int8_5  = Constant::Int(5));
    constdef!   ((vm) <int8> int8_6  = Constant::Int(6));
    constdef!   ((vm) <int8> int8_7  = Constant::Int(7));
    constdef!   ((vm) <int8> int8_8  = Constant::Int(8));
    constdef!   ((vm) <int8> int8_9  = Constant::Int(9));
    constdef!   ((vm) <int8> int8_10 = Constant::Int(10));
    constdef!   ((vm) <int8> int8_11 = Constant::Int(11));
    constdef!   ((vm) <int8> int8_12 = Constant::Int(12));
    constdef!   ((vm) <int8> int8_13 = Constant::Int(13));
    constdef!   ((vm) <int8> int8_14 = Constant::Int(14));
    constdef!   ((vm) <int8> int8_15 = Constant::Int(15));
    constdef!   ((vm) <int8> int8_16 = Constant::Int(16));

    funcsig!    ((vm) sig = () -> (int8));
    funcdecl!   ((vm) <sig> spill_int8);
    funcdef!    ((vm) <sig> spill_int8 VERSION spill_int8_v1);

    block!      ((vm, spill_int8_v1) blk_entry);

    consta!   ((vm, spill_int8_v1) int8_0_local = int8_0);
    consta!   ((vm, spill_int8_v1) int8_1_local = int8_1);
    consta!   ((vm, spill_int8_v1) int8_2_local = int8_2);
    consta!   ((vm, spill_int8_v1) int8_3_local = int8_3);
    consta!   ((vm, spill_int8_v1) int8_4_local = int8_4);
    consta!   ((vm, spill_int8_v1) int8_5_local = int8_5);
    consta!   ((vm, spill_int8_v1) int8_6_local = int8_6);
    consta!   ((vm, spill_int8_v1) int8_7_local = int8_7);
    consta!   ((vm, spill_int8_v1) int8_8_local = int8_8);
    consta!   ((vm, spill_int8_v1) int8_9_local = int8_9);
    consta!   ((vm, spill_int8_v1) int8_10_local= int8_10);
    consta!   ((vm, spill_int8_v1) int8_11_local= int8_11);
    consta!   ((vm, spill_int8_v1) int8_12_local= int8_12);
    consta!   ((vm, spill_int8_v1) int8_13_local= int8_13);
    consta!   ((vm, spill_int8_v1) int8_14_local= int8_14);
    consta!   ((vm, spill_int8_v1) int8_15_local= int8_15);
    consta!   ((vm, spill_int8_v1) int8_16_local= int8_16);

    block!      ((vm, spill_int8_v1) blk_ret);
    inst!       ((vm, spill_int8_v1) blk_entry_branch:
        BRANCH blk_ret (
            int8_0_local,
            int8_1_local,
            int8_2_local,
            int8_3_local,
            int8_4_local,
            int8_5_local,
            int8_6_local,
            int8_7_local,
            int8_8_local,
            int8_9_local,
            int8_10_local,
            int8_11_local,
            int8_12_local,
            int8_13_local,
            int8_14_local,
            int8_15_local,
            int8_16_local
        )
    );

    define_block!((vm, spill_int8_v1) blk_entry() {
        blk_entry_branch
    });

    ssa!    ((vm, spill_int8_v1) <int8> v0);
    ssa!    ((vm, spill_int8_v1) <int8> v1);
    ssa!    ((vm, spill_int8_v1) <int8> v2);
    ssa!    ((vm, spill_int8_v1) <int8> v3);
    ssa!    ((vm, spill_int8_v1) <int8> v4);
    ssa!    ((vm, spill_int8_v1) <int8> v5);
    ssa!    ((vm, spill_int8_v1) <int8> v6);
    ssa!    ((vm, spill_int8_v1) <int8> v7);
    ssa!    ((vm, spill_int8_v1) <int8> v8);
    ssa!    ((vm, spill_int8_v1) <int8> v9);
    ssa!    ((vm, spill_int8_v1) <int8> v10);
    ssa!    ((vm, spill_int8_v1) <int8> v11);
    ssa!    ((vm, spill_int8_v1) <int8> v12);
    ssa!    ((vm, spill_int8_v1) <int8> v13);
    ssa!    ((vm, spill_int8_v1) <int8> v14);
    ssa!    ((vm, spill_int8_v1) <int8> v15);
    ssa!    ((vm, spill_int8_v1) <int8> v16);

    ssa!    ((vm, spill_int8_v1) <int8> res1);
    inst!   ((vm, spill_int8_v1) blk_ret_add1:
        res1 = BINOP (BinOp::Add) v0 v1
    );

    ssa!    ((vm, spill_int8_v1) <int8> res2);
    inst!   ((vm, spill_int8_v1) blk_ret_add2:
        res2 = BINOP (BinOp::Add) res1 v2
    );

    ssa!    ((vm, spill_int8_v1) <int8> res3);
    inst!   ((vm, spill_int8_v1) blk_ret_add3:
        res3 = BINOP (BinOp::Add) res2 v3
    );

    ssa!    ((vm, spill_int8_v1) <int8> res4);
    inst!   ((vm, spill_int8_v1) blk_ret_add4:
        res4 = BINOP (BinOp::Add) res3 v4
    );

    ssa!    ((vm, spill_int8_v1) <int8> res5);
    inst!   ((vm, spill_int8_v1) blk_ret_add5:
        res5 = BINOP (BinOp::Add) res4 v5
    );

    ssa!    ((vm, spill_int8_v1) <int8> res6);
    inst!   ((vm, spill_int8_v1) blk_ret_add6:
        res6 = BINOP (BinOp::Add) res5 v6
    );

    ssa!    ((vm, spill_int8_v1) <int8> res7);
    inst!   ((vm, spill_int8_v1) blk_ret_add7:
        res7 = BINOP (BinOp::Add) res6 v7
    );

    ssa!    ((vm, spill_int8_v1) <int8> res8);
    inst!   ((vm, spill_int8_v1) blk_ret_add8:
        res8 = BINOP (BinOp::Add) res7 v8
    );

    ssa!    ((vm, spill_int8_v1) <int8> res9);
    inst!   ((vm, spill_int8_v1) blk_ret_add9:
        res9 = BINOP (BinOp::Add) res8 v9
    );

    ssa!    ((vm, spill_int8_v1) <int8> res10);
    inst!   ((vm, spill_int8_v1) blk_ret_add10:
        res10 = BINOP (BinOp::Add) res9 v10
    );

    ssa!    ((vm, spill_int8_v1) <int8> res11);
    inst!   ((vm, spill_int8_v1) blk_ret_add11:
        res11 = BINOP (BinOp::Add) res10 v11
    );

    ssa!    ((vm, spill_int8_v1) <int8> res12);
    inst!   ((vm, spill_int8_v1) blk_ret_add12:
        res12 = BINOP (BinOp::Add) res11 v12
    );

    ssa!    ((vm, spill_int8_v1) <int8> res13);
    inst!   ((vm, spill_int8_v1) blk_ret_add13:
        res13 = BINOP (BinOp::Add) res12 v13
    );

    ssa!    ((vm, spill_int8_v1) <int8> res14);
    inst!   ((vm, spill_int8_v1) blk_ret_add14:
        res14 = BINOP (BinOp::Add) res13 v14
    );

    ssa!    ((vm, spill_int8_v1) <int8> res15);
    inst!   ((vm, spill_int8_v1) blk_ret_add15:
        res15 = BINOP (BinOp::Add) res14 v15
    );

    ssa!    ((vm, spill_int8_v1) <int8> res16);
    inst!   ((vm, spill_int8_v1) blk_ret_add16:
        res16 = BINOP (BinOp::Add) res15 v16
    );

    inst!   ((vm, spill_int8_v1) blk_ret_ret:
        RET (res16)
    );

    define_block!   ((vm, spill_int8_v1) blk_ret(v0, v1, v2, v3, v4, v5, v6, v7, v8, v9, v10, v11, v12, v13, v14, v15, v16) {
        blk_ret_add1,
        blk_ret_add2,
        blk_ret_add3,
        blk_ret_add4,
        blk_ret_add5,
        blk_ret_add6,
        blk_ret_add7,
        blk_ret_add8,
        blk_ret_add9,
        blk_ret_add10,
        blk_ret_add11,
        blk_ret_add12,
        blk_ret_add13,
        blk_ret_add14,
        blk_ret_add15,
        blk_ret_add16,
        blk_ret_ret
    });

    define_func_ver!((vm) spill_int8_v1 (entry: blk_entry) {
        blk_entry,
        blk_ret
    });

    vm
}

#[cfg(target_arch = "aarch64")]
fn spill_int8() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int8 = mu_int(8));

    constdef!   ((vm) <int8> int8_0  = Constant::Int(0));
    constdef!   ((vm) <int8> int8_1  = Constant::Int(1));
    constdef!   ((vm) <int8> int8_2  = Constant::Int(2));
    constdef!   ((vm) <int8> int8_3  = Constant::Int(3));
    constdef!   ((vm) <int8> int8_4  = Constant::Int(4));
    constdef!   ((vm) <int8> int8_5  = Constant::Int(5));
    constdef!   ((vm) <int8> int8_6  = Constant::Int(6));
    constdef!   ((vm) <int8> int8_7  = Constant::Int(7));
    constdef!   ((vm) <int8> int8_8  = Constant::Int(8));
    constdef!   ((vm) <int8> int8_9  = Constant::Int(9));
    constdef!   ((vm) <int8> int8_10 = Constant::Int(10));
    constdef!   ((vm) <int8> int8_11 = Constant::Int(11));
    constdef!   ((vm) <int8> int8_12 = Constant::Int(12));
    constdef!   ((vm) <int8> int8_13 = Constant::Int(13));
    constdef!   ((vm) <int8> int8_14 = Constant::Int(14));
    constdef!   ((vm) <int8> int8_15 = Constant::Int(15));
    constdef!   ((vm) <int8> int8_16 = Constant::Int(16));
    constdef!   ((vm) <int8> int8_17 = Constant::Int(17));
    constdef!   ((vm) <int8> int8_18 = Constant::Int(18));
    constdef!   ((vm) <int8> int8_19 = Constant::Int(19));
    constdef!   ((vm) <int8> int8_20 = Constant::Int(20));
    constdef!   ((vm) <int8> int8_21 = Constant::Int(21));
    constdef!   ((vm) <int8> int8_22 = Constant::Int(22));
    constdef!   ((vm) <int8> int8_23 = Constant::Int(23));
    constdef!   ((vm) <int8> int8_24 = Constant::Int(24));
    constdef!   ((vm) <int8> int8_25 = Constant::Int(25));
    constdef!   ((vm) <int8> int8_26 = Constant::Int(26));
    constdef!   ((vm) <int8> int8_27 = Constant::Int(27));
    constdef!   ((vm) <int8> int8_28 = Constant::Int(28));
    constdef!   ((vm) <int8> int8_29 = Constant::Int(29));
    constdef!   ((vm) <int8> int8_30 = Constant::Int(30));


    funcsig!    ((vm) sig = () -> (int8));
    funcdecl!   ((vm) <sig> spill_int8);
    funcdef!    ((vm) <sig> spill_int8 VERSION spill_int8_v1);

    block!      ((vm, spill_int8_v1) blk_entry);

    consta!   ((vm, spill_int8_v1) int8_0_local = int8_0);
    consta!   ((vm, spill_int8_v1) int8_1_local = int8_1);
    consta!   ((vm, spill_int8_v1) int8_2_local = int8_2);
    consta!   ((vm, spill_int8_v1) int8_3_local = int8_3);
    consta!   ((vm, spill_int8_v1) int8_4_local = int8_4);
    consta!   ((vm, spill_int8_v1) int8_5_local = int8_5);
    consta!   ((vm, spill_int8_v1) int8_6_local = int8_6);
    consta!   ((vm, spill_int8_v1) int8_7_local = int8_7);
    consta!   ((vm, spill_int8_v1) int8_8_local = int8_8);
    consta!   ((vm, spill_int8_v1) int8_9_local = int8_9);
    consta!   ((vm, spill_int8_v1) int8_10_local= int8_10);
    consta!   ((vm, spill_int8_v1) int8_11_local= int8_11);
    consta!   ((vm, spill_int8_v1) int8_12_local= int8_12);
    consta!   ((vm, spill_int8_v1) int8_13_local= int8_13);
    consta!   ((vm, spill_int8_v1) int8_14_local= int8_14);
    consta!   ((vm, spill_int8_v1) int8_15_local= int8_15);
    consta!   ((vm, spill_int8_v1) int8_16_local= int8_16);
    consta!   ((vm, spill_int8_v1) int8_17_local= int8_17);
    consta!   ((vm, spill_int8_v1) int8_18_local= int8_18);
    consta!   ((vm, spill_int8_v1) int8_19_local= int8_19);
    consta!   ((vm, spill_int8_v1) int8_20_local= int8_20);
    consta!   ((vm, spill_int8_v1) int8_21_local= int8_21);
    consta!   ((vm, spill_int8_v1) int8_22_local= int8_22);
    consta!   ((vm, spill_int8_v1) int8_23_local= int8_23);
    consta!   ((vm, spill_int8_v1) int8_24_local= int8_24);
    consta!   ((vm, spill_int8_v1) int8_25_local= int8_25);
    consta!   ((vm, spill_int8_v1) int8_26_local= int8_26);
    consta!   ((vm, spill_int8_v1) int8_27_local= int8_27);
    consta!   ((vm, spill_int8_v1) int8_28_local= int8_28);
    consta!   ((vm, spill_int8_v1) int8_29_local= int8_29);
    consta!   ((vm, spill_int8_v1) int8_30_local= int8_30);

    block!      ((vm, spill_int8_v1) blk_ret);
    inst!       ((vm, spill_int8_v1) blk_entry_branch:
        BRANCH blk_ret (
            int8_0_local,
            int8_1_local,
            int8_2_local,
            int8_3_local,
            int8_4_local,
            int8_5_local,
            int8_6_local,
            int8_7_local,
            int8_8_local,
            int8_9_local,
            int8_10_local,
            int8_11_local,
            int8_12_local,
            int8_13_local,
            int8_14_local,
            int8_15_local,
            int8_16_local,
            int8_17_local,
            int8_18_local,
            int8_19_local,
            int8_20_local,
            int8_21_local,
            int8_22_local,
            int8_23_local,
            int8_24_local,
            int8_25_local,
            int8_26_local,
            int8_27_local,
            int8_28_local,
            int8_29_local,
            int8_30_local
        )
    );

    define_block!((vm, spill_int8_v1) blk_entry() {
        blk_entry_branch
    });

    ssa!    ((vm, spill_int8_v1) <int8> v0);
    ssa!    ((vm, spill_int8_v1) <int8> v1);
    ssa!    ((vm, spill_int8_v1) <int8> v2);
    ssa!    ((vm, spill_int8_v1) <int8> v3);
    ssa!    ((vm, spill_int8_v1) <int8> v4);
    ssa!    ((vm, spill_int8_v1) <int8> v5);
    ssa!    ((vm, spill_int8_v1) <int8> v6);
    ssa!    ((vm, spill_int8_v1) <int8> v7);
    ssa!    ((vm, spill_int8_v1) <int8> v8);
    ssa!    ((vm, spill_int8_v1) <int8> v9);
    ssa!    ((vm, spill_int8_v1) <int8> v10);
    ssa!    ((vm, spill_int8_v1) <int8> v11);
    ssa!    ((vm, spill_int8_v1) <int8> v12);
    ssa!    ((vm, spill_int8_v1) <int8> v13);
    ssa!    ((vm, spill_int8_v1) <int8> v14);
    ssa!    ((vm, spill_int8_v1) <int8> v15);
    ssa!    ((vm, spill_int8_v1) <int8> v16);
    ssa!    ((vm, spill_int8_v1) <int8> v17);
    ssa!    ((vm, spill_int8_v1) <int8> v18);
    ssa!    ((vm, spill_int8_v1) <int8> v19);
    ssa!    ((vm, spill_int8_v1) <int8> v20);
    ssa!    ((vm, spill_int8_v1) <int8> v21);
    ssa!    ((vm, spill_int8_v1) <int8> v22);
    ssa!    ((vm, spill_int8_v1) <int8> v23);
    ssa!    ((vm, spill_int8_v1) <int8> v24);
    ssa!    ((vm, spill_int8_v1) <int8> v25);
    ssa!    ((vm, spill_int8_v1) <int8> v26);
    ssa!    ((vm, spill_int8_v1) <int8> v27);
    ssa!    ((vm, spill_int8_v1) <int8> v28);
    ssa!    ((vm, spill_int8_v1) <int8> v29);
    ssa!    ((vm, spill_int8_v1) <int8> v30);

    ssa!    ((vm, spill_int8_v1) <int8> res1);
    inst!   ((vm, spill_int8_v1) blk_ret_add1:
        res1 = BINOP (BinOp::Add) v0 v1
    );

    ssa!    ((vm, spill_int8_v1) <int8> res2);
    inst!   ((vm, spill_int8_v1) blk_ret_add2:
        res2 = BINOP (BinOp::Add) res1 v2
    );

    ssa!    ((vm, spill_int8_v1) <int8> res3);
    inst!   ((vm, spill_int8_v1) blk_ret_add3:
        res3 = BINOP (BinOp::Add) res2 v3
    );

    ssa!    ((vm, spill_int8_v1) <int8> res4);
    inst!   ((vm, spill_int8_v1) blk_ret_add4:
        res4 = BINOP (BinOp::Add) res3 v4
    );

    ssa!    ((vm, spill_int8_v1) <int8> res5);
    inst!   ((vm, spill_int8_v1) blk_ret_add5:
        res5 = BINOP (BinOp::Add) res4 v5
    );

    ssa!    ((vm, spill_int8_v1) <int8> res6);
    inst!   ((vm, spill_int8_v1) blk_ret_add6:
        res6 = BINOP (BinOp::Add) res5 v6
    );

    ssa!    ((vm, spill_int8_v1) <int8> res7);
    inst!   ((vm, spill_int8_v1) blk_ret_add7:
        res7 = BINOP (BinOp::Add) res6 v7
    );

    ssa!    ((vm, spill_int8_v1) <int8> res8);
    inst!   ((vm, spill_int8_v1) blk_ret_add8:
        res8 = BINOP (BinOp::Add) res7 v8
    );

    ssa!    ((vm, spill_int8_v1) <int8> res9);
    inst!   ((vm, spill_int8_v1) blk_ret_add9:
        res9 = BINOP (BinOp::Add) res8 v9
    );

    ssa!    ((vm, spill_int8_v1) <int8> res10);
    inst!   ((vm, spill_int8_v1) blk_ret_add10:
        res10 = BINOP (BinOp::Add) res9 v10
    );

    ssa!    ((vm, spill_int8_v1) <int8> res11);
    inst!   ((vm, spill_int8_v1) blk_ret_add11:
        res11 = BINOP (BinOp::Add) res10 v11
    );

    ssa!    ((vm, spill_int8_v1) <int8> res12);
    inst!   ((vm, spill_int8_v1) blk_ret_add12:
        res12 = BINOP (BinOp::Add) res11 v12
    );

    ssa!    ((vm, spill_int8_v1) <int8> res13);
    inst!   ((vm, spill_int8_v1) blk_ret_add13:
        res13 = BINOP (BinOp::Add) res12 v13
    );

    ssa!    ((vm, spill_int8_v1) <int8> res14);
    inst!   ((vm, spill_int8_v1) blk_ret_add14:
        res14 = BINOP (BinOp::Add) res13 v14
    );

    ssa!    ((vm, spill_int8_v1) <int8> res15);
    inst!   ((vm, spill_int8_v1) blk_ret_add15:
        res15 = BINOP (BinOp::Add) res14 v15
    );

    ssa!    ((vm, spill_int8_v1) <int8> res16);
    inst!   ((vm, spill_int8_v1) blk_ret_add16:
        res16 = BINOP (BinOp::Add) res15 v16
    );

    ssa!    ((vm, spill_int8_v1) <int8> res17);
    inst!   ((vm, spill_int8_v1) blk_ret_add17:
        res17 = BINOP (BinOp::Add) res16 v17
    );

    ssa!    ((vm, spill_int8_v1) <int8> res18);
    inst!   ((vm, spill_int8_v1) blk_ret_add18:
        res18 = BINOP (BinOp::Add) res17 v18
    );

    ssa!    ((vm, spill_int8_v1) <int8> res19);
    inst!   ((vm, spill_int8_v1) blk_ret_add19:
        res19 = BINOP (BinOp::Add) res18 v19
    );

    ssa!    ((vm, spill_int8_v1) <int8> res20);
    inst!   ((vm, spill_int8_v1) blk_ret_add20:
        res20 = BINOP (BinOp::Add) res19 v20
    );

    ssa!    ((vm, spill_int8_v1) <int8> res21);
    inst!   ((vm, spill_int8_v1) blk_ret_add21:
        res21 = BINOP (BinOp::Add) res20 v21
    );

    ssa!    ((vm, spill_int8_v1) <int8> res22);
    inst!   ((vm, spill_int8_v1) blk_ret_add22:
        res22 = BINOP (BinOp::Add) res21 v22
    );

    ssa!    ((vm, spill_int8_v1) <int8> res23);
    inst!   ((vm, spill_int8_v1) blk_ret_add23:
        res23 = BINOP (BinOp::Add) res22 v23
    );

    ssa!    ((vm, spill_int8_v1) <int8> res24);
    inst!   ((vm, spill_int8_v1) blk_ret_add24:
        res24 = BINOP (BinOp::Add) res23 v24
    );

    ssa!    ((vm, spill_int8_v1) <int8> res25);
    inst!   ((vm, spill_int8_v1) blk_ret_add25:
        res25 = BINOP (BinOp::Add) res24 v25
    );

    ssa!    ((vm, spill_int8_v1) <int8> res26);
    inst!   ((vm, spill_int8_v1) blk_ret_add26:
        res26 = BINOP (BinOp::Add) res25 v26
    );

    ssa!    ((vm, spill_int8_v1) <int8> res27);
    inst!   ((vm, spill_int8_v1) blk_ret_add27:
        res27 = BINOP (BinOp::Add) res26 v27
    );

    ssa!    ((vm, spill_int8_v1) <int8> res28);
    inst!   ((vm, spill_int8_v1) blk_ret_add28:
        res28 = BINOP (BinOp::Add) res27 v28
    );

    ssa!    ((vm, spill_int8_v1) <int8> res29);
    inst!   ((vm, spill_int8_v1) blk_ret_add29:
        res29 = BINOP (BinOp::Add) res28 v29
    );

    ssa!    ((vm, spill_int8_v1) <int8> res30);
    inst!   ((vm, spill_int8_v1) blk_ret_add30:
        res30 = BINOP (BinOp::Add) res29 v30
    );


    inst!   ((vm, spill_int8_v1) blk_ret_ret:
        RET (res30)
    );

    define_block!   ((vm, spill_int8_v1) blk_ret(v0, v1, v2, v3, v4, v5, v6, v7, v8, v9, v10, v11, v12, v13, v14, v15, v16, v17, v18, v19, v20, v21, v22, v23, v24, v25, v26, v27, v28, v29, v30) {
        blk_ret_add1,
        blk_ret_add2,
        blk_ret_add3,
        blk_ret_add4,
        blk_ret_add5,
        blk_ret_add6,
        blk_ret_add7,
        blk_ret_add8,
        blk_ret_add9,
        blk_ret_add10,
        blk_ret_add11,
        blk_ret_add12,
        blk_ret_add13,
        blk_ret_add14,
        blk_ret_add15,
        blk_ret_add16,
        blk_ret_add17,
        blk_ret_add18,
        blk_ret_add19,
        blk_ret_add20,
        blk_ret_add21,
        blk_ret_add22,
        blk_ret_add23,
        blk_ret_add24,
        blk_ret_add25,
        blk_ret_add26,
        blk_ret_add27,
        blk_ret_add28,
        blk_ret_add29,
        blk_ret_add30,
        blk_ret_ret
    });

    define_func_ver!((vm) spill_int8_v1 (entry: blk_entry) {
        blk_entry,
        blk_ret
    });

    vm
}
