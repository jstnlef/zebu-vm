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

use self::mu::ast::types::*;
use self::mu::ast::ir::*;
use self::mu::ast::inst::*;
use self::mu::ast::op::*;
use self::mu::vm::*;
use self::mu::utils::LinkedHashMap;

use std::sync::Arc;

#[test]
#[allow(unused_variables)]
fn test_sum() {
    let vm = sum();
}

pub fn sum() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int64 = mu_int(64));
    typedef!    ((vm) int1  = mu_int(1));

    constdef!   ((vm) <int64> int64_0 = Constant::Int(0));
    constdef!   ((vm) <int64> int64_1 = Constant::Int(1));

    funcsig!    ((vm) sum_sig = (int64) -> (int64));
    funcdecl!   ((vm) <sum_sig> sum);
    funcdef!    ((vm) <sum_sig> sum VERSION sum_v1);

    // entry
    block!      ((vm, sum_v1) blk_entry);
    ssa!        ((vm, sum_v1) <int64> blk_entry_n);
    consta!     ((vm, sum_v1) int64_0_local = int64_0);
    consta!     ((vm, sum_v1) int64_1_local = int64_1);

    block!      ((vm, sum_v1) blk_head);
    inst!       ((vm, sum_v1) blk_entry_branch:
        BRANCH blk_head (blk_entry_n, int64_0_local, int64_0_local)
    );

    define_block!((vm, sum_v1) blk_entry(blk_entry_n) {
        blk_entry_branch
    });

    // %head(<@int_64> %n, <@int_64> %s, <@int_64> %i):
    ssa!        ((vm, sum_v1) <int64> blk_head_n);
    ssa!        ((vm, sum_v1) <int64> blk_head_s);
    ssa!        ((vm, sum_v1) <int64> blk_head_i);

    // %s2 = ADD %s %i
    ssa!        ((vm, sum_v1) <int64> blk_head_s2);
    inst!       ((vm, sum_v1) blk_head_add:
        blk_head_s2 = BINOP (BinOp::Add) blk_head_s blk_head_i
    );

    // %i2 = ADD %i 1
    ssa!        ((vm, sum_v1) <int64> blk_head_i2);
    inst!       ((vm, sum_v1) blk_head_add2:
        blk_head_i2 = BINOP (BinOp::Add) blk_head_i int64_1_local
    );

    // %cond = UGE %i %n
    ssa!        ((vm, sum_v1) <int1> blk_head_cond);
    inst!       ((vm, sum_v1) blk_head_uge:
        blk_head_cond = CMPOP (CmpOp::UGE) blk_head_i blk_head_n
    );

    // BRANCH2 %cond %ret(%s2) %head(%n %s2 %i2)
    block!      ((vm, sum_v1) blk_ret);
    inst!       ((vm, sum_v1) blk_head_branch2:
        BRANCH2 (blk_head_cond, blk_head_n, blk_head_s2, blk_head_i2)
        IF (OP 0)
        THEN blk_ret  (vec![2]) WITH 0.6f32,
        ELSE blk_head (vec![1, 2, 3])
    );

    define_block!((vm, sum_v1) blk_head(blk_head_n, blk_head_s, blk_head_i) {
        blk_head_add,
        blk_head_add2,
        blk_head_uge,
        blk_head_branch2
    });

    // %ret(<@int_64> %s):
    ssa!        ((vm, sum_v1) <int64> blk_ret_s);

    inst!       ((vm, sum_v1) blk_ret_term:
        RET (blk_ret_s)
    );

    define_block!((vm, sum_v1) blk_ret(blk_ret_s) {
        blk_ret_term
    });

    // wrap into a function
    define_func_ver!((vm) sum_v1(entry: blk_entry) {
        blk_entry, blk_head, blk_ret
    });

    vm
}

#[test]
#[allow(unused_variables)]
fn test_factorial() {
    let vm = factorial();
}

#[allow(unused_variables)]
pub fn factorial() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int64 = mu_int(64));
    typedef!    ((vm) int1  = mu_int(1));

    constdef!   ((vm) <int64> int64_1 = Constant::Int(1));

    funcsig!    ((vm) fac_sig = (int64) -> (int64));
    funcdecl!   ((vm) <fac_sig> fac);
    funcdef!    ((vm) <fac_sig> fac VERSION fac_v1);

    typedef!    ((vm) funcref_fac = mu_funcref(fac_sig));
    constdef!   ((vm) <funcref_fac> const_funcref_fac = Constant::FuncRef(fac.clone()));

    // %blk_0(<@int_64> %n_3):
    block!      ((vm, fac_v1) blk_0);
    ssa!        ((vm, fac_v1) <int64> blk_0_n_3);
    consta!     ((vm, fac_v1) int64_1_local = int64_1);

    //   %v48 = EQ <@int_64> %n_3 @int_64_1
    ssa!        ((vm, fac_v1) <int1> blk_0_v48);
    inst!       ((vm, fac_v1) blk_0_eq:
        blk_0_v48 = CMPOP (CmpOp::EQ) blk_0_n_3 int64_1_local
    );

    //   BRANCH2 %v48 %blk_2(@int_64_1) %blk_1(%n_3)
    block!      ((vm, fac_v1) blk_1);
    block!      ((vm, fac_v1) blk_2);
    inst!       ((vm, fac_v1) blk_0_branch2:
        BRANCH2 (blk_0_v48, int64_1_local, blk_0_n_3)
        IF (OP 0)
        THEN blk_2 (vec![1]) WITH 0.3f32,
        ELSE blk_1 (vec![2])
    );

    define_block!((vm, fac_v1) blk_0(blk_0_n_3) {
        blk_0_eq,
        blk_0_branch2
    });

    // %blk_2(<@int_64> %v53):
    ssa!        ((vm, fac_v1) <int64> blk_2_v53);
    inst!       ((vm, fac_v1) blk_2_ret:
        RET (blk_2_v53)
    );

    define_block!((vm, fac_v1) blk_2(blk_2_v53) {
        blk_2_ret
    });

    // %blk_1(<@int_64> %n_3):
    ssa!        ((vm, fac_v1) <int64> blk_1_n_3);

    //   %v50 = SUB <@int_64> %n_3 @int_64_1
    ssa!        ((vm, fac_v1) <int64> blk_1_v50);
    inst!       ((vm, fac_v1) blk_1_sub:
        blk_1_v50 = BINOP (BinOp::Sub) blk_1_n_3 int64_1_local
    );

    //   %v51 = CALL <@fac_sig> @fac (%v50)
    ssa!        ((vm, fac_v1) <int64> blk_1_v51);
    consta!     ((vm, fac_v1) const_funcref_fac_local = const_funcref_fac);
    inst!       ((vm, fac_v1) blk_1_call:
        blk_1_v51 =
            EXPRCALL (CallConvention::Mu, is_abort: false) const_funcref_fac_local (blk_1_v50)
    );

    //   %v52 = MUL <@int_64> %n_3 %v51
    ssa!        ((vm, fac_v1) <int64> blk_1_v52);
    inst!       ((vm, fac_v1) blk_1_mul:
        blk_1_v52 = BINOP (BinOp::Mul) blk_1_n_3 blk_1_v51
    );

    // BRANCH blk_2 (%blk_1_v52)
    inst!       ((vm, fac_v1) blk_1_branch:
        BRANCH blk_2 (blk_1_v52)
    );

    define_block!((vm, fac_v1) blk_1(blk_1_n_3) {
        blk_1_sub,
        blk_1_call,
        blk_1_mul,
        blk_1_branch
    });

    define_func_ver!((vm) fac_v1 (entry: blk_0) {
        blk_0, blk_1, blk_2
    });

    vm
}

#[test]
#[allow(unused_variables)]
fn test_global_access() {
    use utils::Address;
    use mu::runtime::thread::MuThread;

    let vm = Arc::new(VM::new());

    unsafe {
        MuThread::current_thread_as_mu_thread(Address::zero(), vm.clone());
    }

    global_access(&vm);
}

#[allow(unused_variables)]
pub fn global_access(vm: &VM) {
    typedef!    ((vm) int64      = mu_int(64));
    typedef!    ((vm) iref_int64 = mu_iref(int64));

    constdef!   ((vm) <int64> int64_0 = Constant::Int(0));
    constdef!   ((vm) <int64> int64_1 = Constant::Int(1));

    globaldef!  ((vm) <int64> a);

    funcsig!    ((vm) global_access_sig = () -> ());
    funcdecl!   ((vm) <global_access_sig> global_access);
    funcdef!    ((vm) <global_access_sig> global_access VERSION global_access_v1);

    // %blk_0():
    block!      ((vm, global_access_v1) blk_0);

    // STORE <@int_64> @a @int_64_1
    global!     ((vm, global_access_v1) blk_0_a = a);
    consta!     ((vm, global_access_v1) int64_1_local = int64_1);
    inst!       ((vm, global_access_v1) blk_0_store:
        STORE blk_0_a int64_1_local (is_ptr: false, order: MemoryOrder::Relaxed)
    );

    // %x = LOAD <@int_64> @a
    ssa!        ((vm, global_access_v1) <int64> blk_0_x);
    inst!       ((vm, global_access_v1) blk_0_load:
        blk_0_x = LOAD blk_0_a (is_ptr: false, order: MemoryOrder::Relaxed)
    );

    inst!       ((vm, global_access_v1) blk_0_ret:
        RET
    );

    define_block!((vm, global_access_v1) blk_0() {
        blk_0_store, blk_0_load, blk_0_ret
    });

    define_func_ver!((vm) global_access_v1(entry: blk_0) {
        blk_0
    });
}
