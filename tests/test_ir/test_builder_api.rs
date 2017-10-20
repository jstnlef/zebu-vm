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

#![allow(unused_imports)]
#![allow(dead_code)]
extern crate mu;

use self::mu::ast::types::*;
use self::mu::ast::ir::*;
use self::mu::ast::inst::*;
use self::mu::ast::ptr::*;
use self::mu::ast::op::*;
use self::mu::vm::*;
use self::mu::vm::api::*;
use self::mu::vm::api::api_c::*;

use std::mem;
use std::ptr;
use std::ffi::CString;
use std::os::raw::c_char;

#[test]
#[allow(unused_variables)]
fn test_builder_factorial() {
    builder_factorial()
}

fn builder_factorial() {
    //    let mvm = MuVM::new();
    //    let mvm_ref = unsafe {mvm.as_mut()}.unwrap();
    //    let ctx = (mvm_ref.new_context)(mvm);
    //    let ctx_ref = unsafe {ctx.as_mut()}.unwrap();
}

#[test]
#[allow(unused_variables)]
fn test_startup_shutdown() {
    unsafe {
        VM::start_logging_trace();

        info!("Starting micro VM...");

        let mvm = mu_fastimpl_new();

        let ctx = ((*mvm).new_context)(mvm);

        let b = ((*ctx).new_ir_builder)(ctx);

        let id1 = ((*b).gen_sym)(b, ptr::null_mut());
        let id2 = ((*b).gen_sym)(b, CString::new("@id2").unwrap().as_ptr());
        let id3 = ((*b).gen_sym)(b, ptr::null_mut());

        ((*b).abort)(b);
        ((*ctx).close_context)(ctx);

        info!("Finished.");
    }
}

#[derive(Default)]
struct CStringPool {
    strings: Vec<CString>
}

impl CStringPool {
    fn get(&mut self, s: &str) -> *const c_char {
        self.strings.push(CString::new(s).unwrap());
        self.strings.last().unwrap().as_ptr()
    }
}


#[test]
#[allow(unused_variables)]
fn test_types_sigs_loading() {
    let mut csp: CStringPool = Default::default();

    unsafe {
        VM::start_logging_trace();

        info!("Starting micro VM...");

        let mvm = mu_fastimpl_new();

        let ctx = ((*mvm).new_context)(mvm);

        let b = ((*ctx).new_ir_builder)(ctx);

        let id1 = ((*b).gen_sym)(b, csp.get("@i8"));
        let id2 = ((*b).gen_sym)(b, csp.get("@i32"));
        let id3 = ((*b).gen_sym)(b, csp.get("@pi32"));
        //let id4 = ((*b).gen_sym)(b, csp.get("@str1"));
        //let id5 = ((*b).gen_sym)(b, ptr::null_mut());
        let id6 = ((*b).gen_sym)(b, csp.get("@str2"));
        let id7 = ((*b).gen_sym)(b, csp.get("@pstr2"));

        ((*b).new_type_int)(b, id1, 8);
        ((*b).new_type_int)(b, id2, 32);
        ((*b).new_type_uptr)(b, id3, id2);
        //((*b).new_type_struct)(b, id4, ptr::null_mut(), 0);
        //((*b).new_type_struct)(b, id5, ptr::null_mut(), 0);

        let mut fields = vec![id3, id7];
        ((*b).new_type_struct)(b, id6, fields.as_mut_ptr(), fields.len());
        ((*b).new_type_uptr)(b, id7, id6);

        let id8 = ((*b).gen_sym)(b, csp.get("@sig1"));
        let id9 = ((*b).gen_sym)(b, csp.get("@funcptr1"));

        let mut ptys = vec![id1, id2];
        let mut rtys = vec![id3, id7];
        ((*b).new_funcsig)(
            b,
            id8,
            ptys.as_mut_ptr(),
            ptys.len(),
            rtys.as_mut_ptr(),
            rtys.len()
        );
        ((*b).new_type_ufuncptr)(b, id9, id8);

        let id10 = ((*b).gen_sym)(b, csp.get("@hyb1"));
        let id11 = ((*b).gen_sym)(b, csp.get("@rhyb1"));

        let mut fixeds = vec![id2, id2];
        ((*b).new_type_hybrid)(b, id10, fixeds.as_mut_ptr(), fixeds.len(), id1);
        ((*b).new_type_ref)(b, id11, id10);

        ((*b).load)(b);
        ((*ctx).close_context)(ctx);

        info!("Finished.");
    }
}


#[test]
#[allow(unused_variables)]
fn test_consts_loading() {
    let mut csp: CStringPool = Default::default();

    unsafe {
        VM::start_logging_trace();

        info!("Starting micro VM...");

        let mvm = mu_fastimpl_new();

        let ctx = ((*mvm).new_context)(mvm);

        let b = ((*ctx).new_ir_builder)(ctx);

        let id1 = ((*b).gen_sym)(b, csp.get("@i32"));
        let id2 = ((*b).gen_sym)(b, csp.get("@CONST_I32_42"));

        ((*b).new_type_int)(b, id1, 32);
        ((*b).new_const_int)(b, id2, id1, 42);

        let id_refi32 = ((*b).gen_sym)(b, csp.get("@refi32"));
        let id_nullrefi32 = ((*b).gen_sym)(b, csp.get("@CONST_REFI32_NULL"));
        ((*b).new_type_ref)(b, id_refi32, id1);
        ((*b).new_const_null)(b, id_nullrefi32, id_refi32);

        ((*b).load)(b);
        ((*ctx).close_context)(ctx);

        info!("Finished.");
    }
}

#[test]
#[allow(unused_variables)]
fn test_globals_loading() {
    let mut csp: CStringPool = Default::default();

    unsafe {
        VM::start_logging_trace();

        info!("Starting micro VM...");

        let mvm = mu_fastimpl_new();

        let ctx = ((*mvm).new_context)(mvm);

        let b = ((*ctx).new_ir_builder)(ctx);

        let id_int32 = ((*b).gen_sym)(b, csp.get("@i32"));
        ((*b).new_type_int)(b, id_int32, 32);

        let id_global = ((*b).gen_sym)(b, csp.get("@my_global"));
        ((*b).new_global_cell)(b, id_global, id_int32);

        ((*b).load)(b);
        ((*ctx).close_context)(ctx);

        info!("Finished. ");
    }
}


#[test]
#[allow(unused_variables)]
fn test_function_loading() {
    let mut csp: CStringPool = Default::default();

    unsafe {
        VM::start_logging_trace();

        info!("Starting micro VM...");

        let mvm = mu_fastimpl_new();

        let ctx = ((*mvm).new_context)(mvm);

        let b = ((*ctx).new_ir_builder)(ctx);

        let id_i32 = ((*b).gen_sym)(b, csp.get("@i32"));
        let id_i64 = ((*b).gen_sym)(b, csp.get("@i64"));
        let id_sig = ((*b).gen_sym)(b, csp.get("@sig"));
        let id_func = ((*b).gen_sym)(b, csp.get("@func"));

        ((*b).new_type_int)(b, id_i32, 32);
        ((*b).new_type_int)(b, id_i64, 64);

        let mut ptys = vec![id_i32];
        let mut rtys = vec![id_i32];
        ((*b).new_funcsig)(
            b,
            id_sig,
            ptys.as_mut_ptr(),
            ptys.len(),
            rtys.as_mut_ptr(),
            rtys.len()
        );

        ((*b).new_func)(b, id_func, id_sig);

        let id_const1 = ((*b).gen_sym)(b, csp.get("@const_i32_1"));
        ((*b).new_const_int)(b, id_const1, id_i32, 1);

        let id_const99 = ((*b).gen_sym)(b, csp.get("@const_i32_99"));
        ((*b).new_const_int)(b, id_const99, id_i32, 99);

        let id_funcver = ((*b).gen_sym)(b, csp.get("@func.v1"));

        let id_entry = ((*b).gen_sym)(b, csp.get("@func.v1.entry"));
        let id_bb1 = ((*b).gen_sym)(b, csp.get("@func.v1.bb1"));
        let id_bb2 = ((*b).gen_sym)(b, csp.get("@func.v1.bb2"));
        let id_bb3 = ((*b).gen_sym)(b, csp.get("@func.v1.bb3"));
        let id_bb4 = ((*b).gen_sym)(b, csp.get("@func.v1.bb4"));
        let id_bb5 = ((*b).gen_sym)(b, csp.get("@func.v1.bb5"));
        let id_bbxxx = ((*b).gen_sym)(b, csp.get("@func.v1.bbxxx"));

        let mut bbs = vec![id_entry, id_bb1, id_bb2, id_bb3, id_bb4, id_bb5, id_bbxxx];
        ((*b).new_func_ver)(b, id_funcver, id_func, bbs.as_mut_ptr(), bbs.len());

        {
            let id_x = ((*b).gen_sym)(b, csp.get("@func.v1.entry.x"));
            let mut args = vec![id_x];
            let mut argtys = vec![id_i32];

            let id_add = ((*b).gen_sym)(b, csp.get("@func.v1.entry.add"));
            let id_sub = ((*b).gen_sym)(b, csp.get("@func.v1.entry.sub"));
            let id_branch = ((*b).gen_sym)(b, csp.get("@func.v1.entry.branch"));
            let mut insts = vec![id_add, id_sub, id_branch];

            ((*b).new_bb)(
                b,
                id_entry,
                args.as_mut_ptr(),
                argtys.as_mut_ptr(),
                args.len(),
                0,
                insts.as_mut_ptr(),
                insts.len()
            );

            let id_y = ((*b).gen_sym)(b, csp.get("@func.v1.entry.y"));
            ((*b).new_binop)(b, id_add, id_y, CMU_BINOP_ADD, id_i32, id_x, id_x, 0);

            let id_z = ((*b).gen_sym)(b, csp.get("@func.v1.entry.z"));
            ((*b).new_binop)(b, id_sub, id_z, CMU_BINOP_SUB, id_i32, id_y, id_const99, 0);

            let id_dest = ((*b).gen_sym)(b, csp.get("@func.v1.entry.dest"));
            let mut dest_args = vec![id_z];
            ((*b).new_dest_clause)(b, id_dest, id_bb1, dest_args.as_mut_ptr(), dest_args.len());
            ((*b).new_branch)(b, id_branch, id_dest);
        }

        {
            let id_a = ((*b).gen_sym)(b, csp.get("@func.v1.bb1.a"));
            let id_ret = ((*b).gen_sym)(b, csp.get("@func.v1.bb1.ret"));

            let mut args = vec![id_a];
            let mut argtys = vec![id_i32];
            let mut insts = vec![id_ret];
            ((*b).new_bb)(
                b,
                id_bb1,
                args.as_mut_ptr(),
                argtys.as_mut_ptr(),
                args.len(),
                0,
                insts.as_mut_ptr(),
                insts.len()
            );

            let mut rvs = vec![id_a];
            ((*b).new_ret)(b, id_ret, rvs.as_mut_ptr(), rvs.len())
        }

        {
            let id_x = ((*b).gen_sym)(b, csp.get("@func.v1.bb2.x"));
            let id_add = ((*b).gen_sym)(b, csp.get("@func.v1.bb2.add"));
            let id_eq = ((*b).gen_sym)(b, csp.get("@func.v1.bb2.eq"));
            let id_br2 = ((*b).gen_sym)(b, csp.get("@func.v1.bb2.br2"));
            let mut args = vec![id_x];
            let mut argtys = vec![id_i32];
            let mut insts = vec![id_add, id_eq, id_br2];
            ((*b).new_bb)(
                b,
                id_bb2,
                args.as_mut_ptr(),
                argtys.as_mut_ptr(),
                args.len(),
                0,
                insts.as_mut_ptr(),
                insts.len()
            );

            let id_y = ((*b).gen_sym)(b, csp.get("@func.v1.bb2.y"));
            ((*b).new_binop)(b, id_add, id_y, CMU_BINOP_ADD, id_i32, id_x, id_const1, 0);

            let id_e = ((*b).gen_sym)(b, csp.get("@func.v1.bb2.e"));
            ((*b).new_cmp)(b, id_eq, id_e, CMU_CMP_EQ, id_i32, id_x, id_const99);

            let id_dest_t = ((*b).gen_sym)(b, csp.get("@func.v1.bb2.dest_t"));
            let id_dest_f = ((*b).gen_sym)(b, csp.get("@func.v1.bb2.dest_f"));

            {
                let mut dest_args = vec![id_x];
                ((*b).new_dest_clause)(
                    b,
                    id_dest_t,
                    id_bb3,
                    dest_args.as_mut_ptr(),
                    dest_args.len()
                );
            }
            {
                let mut dest_args = vec![id_y, id_y, id_x];
                ((*b).new_dest_clause)(
                    b,
                    id_dest_f,
                    id_bb4,
                    dest_args.as_mut_ptr(),
                    dest_args.len()
                );
            }

            ((*b).new_branch2)(b, id_br2, id_e, id_dest_t, id_dest_f);
        }

        {
            let id_x = ((*b).gen_sym)(b, csp.get("@func.v1.bb3.x"));
            let id_ret = ((*b).gen_sym)(b, csp.get("@func.v1.bb3.ret"));
            let mut args = vec![id_x];
            let mut argtys = vec![id_i32];
            let mut insts = vec![id_ret];
            ((*b).new_bb)(
                b,
                id_bb3,
                args.as_mut_ptr(),
                argtys.as_mut_ptr(),
                args.len(),
                0,
                insts.as_mut_ptr(),
                insts.len()
            );

            let mut rvs = vec![id_const99];
            ((*b).new_ret)(b, id_ret, rvs.as_mut_ptr(), rvs.len())
        }

        {
            let id_x = ((*b).gen_sym)(b, csp.get("@func.v1.bb4.x"));
            let id_y = ((*b).gen_sym)(b, csp.get("@func.v1.bb4.y"));
            let id_z = ((*b).gen_sym)(b, csp.get("@func.v1.bb4.z"));
            let id_ret = ((*b).gen_sym)(b, csp.get("@func.v1.bb4.ret"));
            let mut args = vec![id_x, id_y, id_z];
            let mut argtys = vec![id_i32, id_i32, id_i32];
            let mut insts = vec![id_ret];
            ((*b).new_bb)(
                b,
                id_bb4,
                args.as_mut_ptr(),
                argtys.as_mut_ptr(),
                args.len(),
                0,
                insts.as_mut_ptr(),
                insts.len()
            );

            let mut rvs = vec![id_const99];
            ((*b).new_ret)(b, id_ret, rvs.as_mut_ptr(), rvs.len())
        }

        {
            let id_exc = ((*b).gen_sym)(b, csp.get("@func.v1.bbxxx.exc"));
            let id_ret = ((*b).gen_sym)(b, csp.get("@func.v1.bbxxx.ret"));
            let mut args = vec![];
            let mut argtys = vec![];
            let mut insts = vec![id_ret];
            ((*b).new_bb)(
                b,
                id_bbxxx,
                args.as_mut_ptr(),
                argtys.as_mut_ptr(),
                args.len(),
                id_exc,
                insts.as_mut_ptr(),
                insts.len()
            );

            let mut rvs = vec![id_const99];
            ((*b).new_ret)(b, id_ret, rvs.as_mut_ptr(), rvs.len())
        }

        {
            let id_x = ((*b).gen_sym)(b, csp.get("@func.v1.bb5.x"));
            let id_add = ((*b).gen_sym)(b, csp.get("@func.v1.bb5.add"));
            let id_switch = ((*b).gen_sym)(b, csp.get("@func.v1.bb5.switch"));
            let mut args = vec![id_x];
            let mut argtys = vec![id_i32];
            let mut insts = vec![id_add, id_switch];
            ((*b).new_bb)(
                b,
                id_bb5,
                args.as_mut_ptr(),
                argtys.as_mut_ptr(),
                args.len(),
                0,
                insts.as_mut_ptr(),
                insts.len()
            );

            let id_y = ((*b).gen_sym)(b, csp.get("@func.v1.bb5.y"));
            ((*b).new_binop)(b, id_add, id_y, CMU_BINOP_ADD, id_i32, id_x, id_const1, 0);

            let id_dest_def = ((*b).gen_sym)(b, csp.get("@func.v1.bb5.dest_def"));
            let id_dest_99 = ((*b).gen_sym)(b, csp.get("@func.v1.bb5.dest_99"));

            {
                let mut dest_args = vec![id_x];
                ((*b).new_dest_clause)(
                    b,
                    id_dest_def,
                    id_bb3,
                    dest_args.as_mut_ptr(),
                    dest_args.len()
                );
            }
            {
                let mut dest_args = vec![id_y, id_y, id_x];
                ((*b).new_dest_clause)(
                    b,
                    id_dest_99,
                    id_bb4,
                    dest_args.as_mut_ptr(),
                    dest_args.len()
                );
            }

            let mut cases = vec![id_const99];
            let mut dests = vec![id_dest_99];

            ((*b).new_switch)(
                b,
                id_switch,
                id_i32,
                id_y,
                id_dest_def,
                cases.as_mut_ptr(),
                dests.as_mut_ptr(),
                cases.len()
            )
        }

        ((*b).load)(b);
        ((*ctx).close_context)(ctx);

        info!("Finished.");
    }
}

#[test]
#[allow(unused_variables)]
fn test_insts_call() {
    let mut csp: CStringPool = Default::default();

    unsafe {
        VM::start_logging_trace();

        info!("Starting micro VM...");

        let mvm = mu_fastimpl_new();

        let ctx = ((*mvm).new_context)(mvm);

        let b = ((*ctx).new_ir_builder)(ctx);

        let id_i32 = ((*b).gen_sym)(b, csp.get("@i32"));
        let id_i64 = ((*b).gen_sym)(b, csp.get("@i64"));
        let id_sig = ((*b).gen_sym)(b, csp.get("@sig"));
        let id_func = ((*b).gen_sym)(b, csp.get("@func"));

        ((*b).new_type_int)(b, id_i32, 32);
        ((*b).new_type_int)(b, id_i64, 64);

        let mut ptys = vec![id_i32];
        let mut rtys = vec![id_i32];
        ((*b).new_funcsig)(
            b,
            id_sig,
            ptys.as_mut_ptr(),
            ptys.len(),
            rtys.as_mut_ptr(),
            rtys.len()
        );

        ((*b).new_func)(b, id_func, id_sig);

        let id_const1 = ((*b).gen_sym)(b, csp.get("@const_i32_1"));
        ((*b).new_const_int)(b, id_const1, id_i32, 1);

        let id_const99 = ((*b).gen_sym)(b, csp.get("@const_i32_99"));
        ((*b).new_const_int)(b, id_const99, id_i32, 99);

        let id_funcver = ((*b).gen_sym)(b, csp.get("@func.v1"));

        let id_entry = ((*b).gen_sym)(b, csp.get("@func.v1.entry"));
        let id_bb1 = ((*b).gen_sym)(b, csp.get("@func.v1.bb1"));
        let id_bb2 = ((*b).gen_sym)(b, csp.get("@func.v1.bb2"));

        let mut bbs = vec![id_entry, id_bb1, id_bb2];
        ((*b).new_func_ver)(b, id_funcver, id_func, bbs.as_mut_ptr(), bbs.len());

        {
            let id_x = ((*b).gen_sym)(b, csp.get("@func.v1.entry.x"));
            let mut args = vec![id_x];
            let mut argtys = vec![id_i32];

            let id_call1 = ((*b).gen_sym)(b, csp.get("@func.v1.entry.call1"));
            let id_call2 = ((*b).gen_sym)(b, csp.get("@func.v1.entry.call2"));
            let mut insts = vec![id_call1, id_call2];

            ((*b).new_bb)(
                b,
                id_entry,
                args.as_mut_ptr(),
                argtys.as_mut_ptr(),
                args.len(),
                0,
                insts.as_mut_ptr(),
                insts.len()
            );

            let id_y = ((*b).gen_sym)(b, csp.get("@func.v1.entry.y"));

            {
                let mut args = vec![id_x];
                let mut rvs = vec![id_y];
                ((*b).new_call)(
                    b,
                    id_call1,
                    rvs.as_mut_ptr(),
                    rvs.len(),
                    id_sig,
                    id_func,
                    args.as_mut_ptr(),
                    args.len(),
                    0,
                    0
                );
            }

            let id_z = ((*b).gen_sym)(b, csp.get("@func.v1.entry.z"));

            {
                let mut args = vec![id_y];
                let mut rvs = vec![id_z];

                let id_dest1 = ((*b).gen_sym)(b, csp.get("@func.v1.entry.call2.dest1"));
                let id_dest2 = ((*b).gen_sym)(b, csp.get("@func.v1.entry.call2.dest2"));
                let id_exc = ((*b).gen_sym)(b, csp.get("@func.v1.entry.call2.exc"));
                ((*b).new_exc_clause)(b, id_exc, id_dest1, id_dest2);
                {
                    let mut dest_args = vec![id_y, id_z, id_x];
                    ((*b).new_dest_clause)(
                        b,
                        id_dest1,
                        id_bb1,
                        dest_args.as_mut_ptr(),
                        dest_args.len()
                    );
                }
                {
                    let mut dest_args = vec![];
                    ((*b).new_dest_clause)(
                        b,
                        id_dest2,
                        id_bb2,
                        dest_args.as_mut_ptr(),
                        dest_args.len()
                    );
                }

                ((*b).new_call)(
                    b,
                    id_call2,
                    rvs.as_mut_ptr(),
                    rvs.len(),
                    id_sig,
                    id_func,
                    args.as_mut_ptr(),
                    args.len(),
                    id_exc,
                    0
                );
            }
        }

        {
            let id_y = ((*b).gen_sym)(b, csp.get("@func.v1.bb1.y"));
            let id_z = ((*b).gen_sym)(b, csp.get("@func.v1.bb1.z"));
            let id_x = ((*b).gen_sym)(b, csp.get("@func.v1.bb1.x"));
            let mut args = vec![id_y, id_z, id_x];
            let mut argtys = vec![id_i32, id_i32, id_i32];

            let id_ret = ((*b).gen_sym)(b, csp.get("@func.v1.bb1.ret"));
            let mut insts = vec![id_ret];

            ((*b).new_bb)(
                b,
                id_bb1,
                args.as_mut_ptr(),
                argtys.as_mut_ptr(),
                args.len(),
                0,
                insts.as_mut_ptr(),
                insts.len()
            );

            let mut retvars = vec![id_z];
            ((*b).new_ret)(b, id_ret, retvars.as_mut_ptr(), retvars.len());
        }

        {
            let mut args = vec![];
            let mut argtys = vec![];

            let id_ret = ((*b).gen_sym)(b, csp.get("@func.v1.bb2.ret"));
            let mut insts = vec![id_ret];

            ((*b).new_bb)(
                b,
                id_bb2,
                args.as_mut_ptr(),
                argtys.as_mut_ptr(),
                args.len(),
                0,
                insts.as_mut_ptr(),
                insts.len()
            );

            let mut retvars = vec![id_const1];
            ((*b).new_ret)(b, id_ret, retvars.as_mut_ptr(), retvars.len());
        }


        ((*b).load)(b);
        ((*ctx).close_context)(ctx);

        info!("Finished.");
    }
}

#[test]
#[allow(unused_variables)]
fn test_insts_new() {
    let mut csp: CStringPool = Default::default();

    unsafe {
        VM::start_logging_trace();

        info!("Starting micro VM...");

        let mvm = mu_fastimpl_new();

        let ctx = ((*mvm).new_context)(mvm);

        let b = ((*ctx).new_ir_builder)(ctx);

        let id_float = ((*b).gen_sym)(b, csp.get("@float"));
        let id_i8 = ((*b).gen_sym)(b, csp.get("@i8"));
        let id_i32 = ((*b).gen_sym)(b, csp.get("@i32"));
        let id_i64 = ((*b).gen_sym)(b, csp.get("@i64"));
        let id_s = ((*b).gen_sym)(b, csp.get("@s"));
        let id_a = ((*b).gen_sym)(b, csp.get("@a"));
        let id_h = ((*b).gen_sym)(b, csp.get("@h"));

        let id_sig = ((*b).gen_sym)(b, csp.get("@sig"));
        let id_func = ((*b).gen_sym)(b, csp.get("@func"));

        ((*b).new_type_float)(b, id_float);
        ((*b).new_type_int)(b, id_i8, 8);
        ((*b).new_type_int)(b, id_i32, 32);
        ((*b).new_type_int)(b, id_i64, 64);

        {
            let mut fields = vec![id_i32, id_a, id_i64];
            ((*b).new_type_struct)(b, id_s, fields.as_mut_ptr(), fields.len());
            ((*b).new_type_hybrid)(b, id_h, fields.as_mut_ptr(), fields.len(), id_i8);
        }
        ((*b).new_type_array)(b, id_a, id_float, 4);

        let mut ptys = vec![];
        let mut rtys = vec![];
        ((*b).new_funcsig)(
            b,
            id_sig,
            ptys.as_mut_ptr(),
            ptys.len(),
            rtys.as_mut_ptr(),
            rtys.len()
        );

        let id_consti64_3 = ((*b).gen_sym)(b, csp.get("@CONSTI64_3"));
        ((*b).new_const_int)(b, id_consti64_3, id_i64, 3);
        let id_consti64_4 = ((*b).gen_sym)(b, csp.get("@CONSTI64_4"));
        ((*b).new_const_int)(b, id_consti64_4, id_i64, 4);

        ((*b).new_func)(b, id_func, id_sig);

        let id_funcver = ((*b).gen_sym)(b, csp.get("@func.v1"));

        let id_entry = ((*b).gen_sym)(b, csp.get("@func.v1.entry"));

        let mut bbs = vec![id_entry];
        ((*b).new_func_ver)(b, id_funcver, id_func, bbs.as_mut_ptr(), bbs.len());

        {
            let mut args = vec![];
            let mut argtys = vec![];

            let id_new = ((*b).gen_sym)(b, csp.get("@func.v1.entry.new"));
            let id_newhybrid = ((*b).gen_sym)(b, csp.get("@func.v1.entry.newhybrid"));
            let id_getiref1 = ((*b).gen_sym)(b, csp.get("@func.v1.entry.getiref1"));
            let id_getiref2 = ((*b).gen_sym)(b, csp.get("@func.v1.entry.getiref2"));
            let id_getfieldiref1 = ((*b).gen_sym)(b, csp.get("@func.v1.entry.getfieldiref1"));
            let id_getfieldiref2 = ((*b).gen_sym)(b, csp.get("@func.v1.entry.getfieldiref2"));
            let id_getelemiref = ((*b).gen_sym)(b, csp.get("@func.v1.entry.getelemiref"));
            let id_getvarpartiref = ((*b).gen_sym)(b, csp.get("@func.v1.entry.getvarpartiref"));
            let id_shiftiref = ((*b).gen_sym)(b, csp.get("@func.v1.entry.shiftiref"));
            let id_ret = ((*b).gen_sym)(b, csp.get("@func.v1.entry.ret"));
            let mut insts = vec![
                id_new,
                id_newhybrid,
                id_getiref1,
                id_getiref2,
                id_getfieldiref1,
                id_getfieldiref2,
                id_getelemiref,
                id_getvarpartiref,
                id_shiftiref,
                id_ret,
            ];

            ((*b).new_bb)(
                b,
                id_entry,
                args.as_mut_ptr(),
                argtys.as_mut_ptr(),
                args.len(),
                0,
                insts.as_mut_ptr(),
                insts.len()
            );

            let id_v_r1 = ((*b).gen_sym)(b, csp.get("@func.v1.entry.r1"));
            let id_v_r2 = ((*b).gen_sym)(b, csp.get("@func.v1.entry.r2"));
            let id_v_i1 = ((*b).gen_sym)(b, csp.get("@func.v1.entry.i1"));
            let id_v_i2 = ((*b).gen_sym)(b, csp.get("@func.v1.entry.i2"));
            let id_v_f1 = ((*b).gen_sym)(b, csp.get("@func.v1.entry.f1"));
            let id_v_f2 = ((*b).gen_sym)(b, csp.get("@func.v1.entry.f2"));
            let id_v_e = ((*b).gen_sym)(b, csp.get("@func.v1.entry.e"));
            let id_v_v = ((*b).gen_sym)(b, csp.get("@func.v1.entry.v"));
            let id_v_s = ((*b).gen_sym)(b, csp.get("@func.v1.entry.s"));

            ((*b).new_new)(b, id_new, id_v_r1, id_s, 0);
            ((*b).new_newhybrid)(b, id_newhybrid, id_v_r2, id_h, id_i64, id_consti64_4, 0);

            ((*b).new_getiref)(b, id_getiref1, id_v_i1, id_s, id_v_r1);
            ((*b).new_getiref)(b, id_getiref2, id_v_i2, id_h, id_v_r2);

            ((*b).new_getfieldiref)(b, id_getfieldiref1, id_v_f1, 0, id_s, 1, id_v_i1);
            ((*b).new_getfieldiref)(b, id_getfieldiref2, id_v_f2, 0, id_h, 2, id_v_i2);

            ((*b).new_getelemiref)(
                b,
                id_getelemiref,
                id_v_e,
                0,
                id_a,
                id_i64,
                id_v_f1,
                id_consti64_3
            );

            ((*b).new_getvarpartiref)(b, id_getvarpartiref, id_v_v, 0, id_h, id_v_i2);
            ((*b).new_shiftiref)(
                b,
                id_shiftiref,
                id_v_s,
                0,
                id_i8,
                id_i64,
                id_v_v,
                id_consti64_3
            );

            {
                let mut args = vec![];
                ((*b).new_ret)(b, id_ret, args.as_mut_ptr(), args.len());
            }
        }


        ((*b).load)(b);
        ((*ctx).close_context)(ctx);

        info!("Finished.");
    }
}
