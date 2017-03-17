extern crate log;
extern crate libloading;

use mu::ast::types::*;
use mu::ast::ir::*;
use mu::ast::inst::*;
use mu::ast::op::*;
use mu::vm::*;
use mu::compiler::*;
use mu::utils::LinkedHashMap;

use mu::testutil::aot;

use test_compiler::test_call::gen_ccall_exit;

use std::sync::Arc;
use std::sync::RwLock;

#[test]
fn test_exception_throw_catch_simple() {
    VM::start_logging_trace();
    let vm = Arc::new(throw_catch_simple());
    
    let compiler = Compiler::new(CompilerPolicy::default(), &vm);
    
    let func_throw = vm.id_of("throw_exception");
    let func_catch = vm.id_of("catch_exception");    
    {
        let funcs = vm.funcs().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();
        
        {
            let func = funcs.get(&func_throw).unwrap().read().unwrap();
            let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();
    
            compiler.compile(&mut func_ver);
        }
        {
            let func = funcs.get(&func_catch).unwrap().read().unwrap();
            let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();
    
            compiler.compile(&mut func_ver);
        }
    }
    
    vm.make_primordial_thread(func_catch, true, vec![]);
    backend::emit_context(&vm);
    
    let executable = aot::link_primordial(vec![Mu("throw_exception"), Mu("catch_exception")], "throw_catch_simple_test", &vm);
    aot::execute(executable);
}

fn declare_commons(vm: &VM) {
    typedef!    ((vm) int64      = mu_int(64));
    typedef!    ((vm) ref_int64  = mu_ref(int64));
    typedef!    ((vm) iref_int64 = mu_iref(int64));

    constdef!   ((vm) <int64> int64_0 = Constant::Int(0));
    constdef!   ((vm) <int64> int64_1 = Constant::Int(1));
}

fn throw_catch_simple() -> VM {
    let vm = VM::new();
    
    declare_commons(&vm);
    
    create_throw_exception_func(&vm);
    create_catch_exception_func(&vm, true);
    
    vm
}

fn create_catch_exception_func (vm: &VM, use_exception_arg: bool) {
    // .typedef @funcref_throw_exception <@throw_exception_sig>
    let throw_exception_sig = vm.get_func_sig(vm.id_of("throw_exception_sig"));
    let throw_exception_id = vm.id_of("throw_exception");
    let type_funcref_throw_exception = vm.declare_type(vm.next_id(), MuType_::funcref(throw_exception_sig));
    // .const @throw_exception_func
    let const_funcref_throw_exception = vm.declare_const(vm.next_id(), type_funcref_throw_exception, Constant::FuncRef(throw_exception_id)); 
    
    // .funcsig @catch_exception_sig = () -> ()
    let func_sig = vm.declare_func_sig(vm.next_id(), vec![], vec![]);
    vm.set_name(func_sig.as_entity(), Mu("catch_exception_sig"));
    
    // .funcdecl @catch_exception <@catch_exception_sig>
    let func = MuFunction::new(vm.next_id(), func_sig.clone());
    vm.set_name(func.as_entity(), Mu("catch_exception"));
    let func_id = func.id();
    vm.declare_func(func);
    
    // .funcdef @catch_exception VERSION @v1 <@catch_exception_sig);
    let mut func_ver = MuFunctionVersion::new(vm.next_id(), func_id, func_sig.clone());
    
    // %blk_0():
    let mut blk_0 = Block::new(vm.next_id());
    vm.set_name(blk_0.as_entity(), Mu("blk_0"));
    
    let blk_normal_cont_id = vm.next_id();
    let blk_exn_cont_id = vm.next_id();
    
    let blk_0_throw = func_ver.new_constant(const_funcref_throw_exception.clone());
    let blk_0_term = func_ver.new_inst(Instruction {
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: None,
        ops: RwLock::new(vec![blk_0_throw]),
        v: Instruction_::Call {
            data: CallData {
                func: 0,
                args: vec![],
                convention: CallConvention::Mu
            },
            resume: ResumptionData {
                normal_dest: Destination {
                    target: blk_normal_cont_id,
                    args: vec![]
                },
                exn_dest: Destination {
                    target: blk_exn_cont_id,
                    args: vec![]
                }
            }
        }
    });
    
    let blk_0_content = BlockContent {
        args: vec![],
        exn_arg: None,
        body: vec![blk_0_term],
        keepalives: None
    };
    blk_0.content = Some(blk_0_content);
    
    // %blk_normal_cont():
    let mut blk_normal_cont = Block::new(blk_normal_cont_id);
    vm.set_name(blk_normal_cont.as_entity(), Mu("blk_normal_cont"));
    let blk_normal_cont_thread_exit = func_ver.new_inst(Instruction {
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: None,
        ops: RwLock::new(vec![]),
        v: Instruction_::ThreadExit
    });
    blk_normal_cont.content = Some(BlockContent {
        args: vec![],
        exn_arg: None,
        body: vec![blk_normal_cont_thread_exit],
        keepalives: None
    });
    
    // %blk_exn_cont() %EXN:
    let mut blk_exn_cont = Block::new(blk_exn_cont_id);
    vm.set_name(blk_exn_cont.as_entity(), Mu("blk_exn_cont"));
    let type_ref_int64 = vm.get_type(vm.id_of("ref_int64"));    
    let blk_exn_cont_exception_arg = func_ver.new_ssa(vm.next_id(), type_ref_int64.clone());
    vm.set_name(blk_exn_cont_exception_arg.as_entity(), Mu("blk_0_exception_arg"));
    let blk_exn_cont_thread_exit = func_ver.new_inst(Instruction {
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: None,
        ops: RwLock::new(vec![]),
        v: Instruction_::ThreadExit
    });
    blk_exn_cont.content = Some(BlockContent {
        args: vec![],
        exn_arg: if use_exception_arg {
            Some(blk_exn_cont_exception_arg.clone_value())
        } else {
            None
        },
        body: vec![blk_exn_cont_thread_exit],
        keepalives: None
    });    
    
    func_ver.define(FunctionContent::new(
        blk_0.id(),
        {
            let mut ret = LinkedHashMap::new();
            ret.insert(blk_0.id(), blk_0);
            ret.insert(blk_normal_cont.id(), blk_normal_cont);
            ret.insert(blk_exn_cont.id(), blk_exn_cont);
            ret
        }
    ));
    
    vm.define_func_version(func_ver);
}

fn create_throw_exception_func (vm: &VM) {
    let type_ref_int64 = vm.get_type(vm.id_of("ref_int64"));
    let type_iref_int64 = vm.get_type(vm.id_of("iref_int64"));
    
    // .funcsig @throw_exception = () -> ()
    let func_sig = vm.declare_func_sig(vm.next_id(), vec![], vec![]);
    vm.set_name(func_sig.as_entity(), Mu("throw_exception_sig"));
    
    // .funcdecl @catch_exception <@throw_exception>
    let func = MuFunction::new(vm.next_id(), func_sig.clone());
    vm.set_name(func.as_entity(), Mu("throw_exception"));
    let func_id = func.id();
    vm.declare_func(func);
    
    // .funcdef @catch_exception VERSION @v1 <@throw_exception>
    let mut func_ver = MuFunctionVersion::new(vm.next_id(), func_id, func_sig.clone());
    
    // %blk_0():
    let mut blk_0 = Block::new(vm.next_id());
    vm.set_name(blk_0.as_entity(), Mu("blk_0"));
    
    // %exception_obj = NEW <@int64>
    let blk_0_exception_obj = func_ver.new_ssa(vm.next_id(), type_ref_int64.clone());
    vm.set_name(blk_0_exception_obj.as_entity(), Mu("blk_0_exception_obj"));
    let blk_0_inst0 = func_ver.new_inst(Instruction {
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_0_exception_obj.clone_value()]),
        ops: RwLock::new(vec![]),
        v: Instruction_::New(type_ref_int64.clone())
    });
    
    // %exception_obj_iref = GETIREF <@int64> %exception_obj
    let blk_0_exception_obj_iref = func_ver.new_ssa(vm.next_id(), type_iref_int64.clone());
    vm.set_name(blk_0_exception_obj_iref.as_entity(), Mu("blk_0_exception_obj_iref"));
    let blk_0_inst1 = func_ver.new_inst(Instruction {
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_0_exception_obj_iref.clone_value()]),
        ops: RwLock::new(vec![blk_0_exception_obj.clone()]),
        v: Instruction_::GetIRef(0)
     });
    
    // STORE <@int64> %exception_obj_iref @int64_1
    let const_int64_1 = vm.get_const(vm.id_of("int64_1"));
    let blk_0_const_int64_1 = func_ver.new_constant(const_int64_1);
    let blk_0_inst2 = func_ver.new_inst(Instruction {
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: None,
        ops: RwLock::new(vec![blk_0_exception_obj_iref.clone(), blk_0_const_int64_1.clone()]),
        v: Instruction_::Store {
            is_ptr: false,
            order: MemoryOrder::Relaxed,
            mem_loc: 0,
            value: 1
        }
    });
    
    let blk_0_term = func_ver.new_inst(Instruction {
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: None,
        ops: RwLock::new(vec![blk_0_exception_obj.clone()]),
        v: Instruction_::Throw(0)
    });
    
    let blk_0_content = BlockContent {
        args: vec![],
        exn_arg: None,
        body: vec![blk_0_inst0, blk_0_inst1, blk_0_inst2, blk_0_term],
        keepalives: None
    };
    blk_0.content = Some(blk_0_content);
    
    func_ver.define(FunctionContent::new(
        blk_0.id(),
        {
            let mut ret = LinkedHashMap::new();
            ret.insert(blk_0.id(), blk_0);
            ret
        }
    ));
    
    vm.define_func_version(func_ver);
}

#[test]
fn test_exception_throw_catch_dont_use_exception_arg() {
    VM::start_logging_trace();
    let vm = Arc::new(throw_catch_dont_use_exception_arg());

    let compiler = Compiler::new(CompilerPolicy::default(), &vm);

    let func_throw = vm.id_of("throw_exception");
    let func_catch = vm.id_of("catch_exception");
    {
        let funcs = vm.funcs().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();

        {
            let func = funcs.get(&func_throw).unwrap().read().unwrap();
            let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();

            compiler.compile(&mut func_ver);
        }
        {
            let func = funcs.get(&func_catch).unwrap().read().unwrap();
            let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();

            compiler.compile(&mut func_ver);
        }
    }

    vm.make_primordial_thread(func_catch, true, vec![]);
    backend::emit_context(&vm);

    let executable = aot::link_primordial(vec![Mu("throw_exception"), Mu("catch_exception")], "throw_catch_simple_test", &vm);
    aot::execute(executable);
}

fn throw_catch_dont_use_exception_arg() -> VM {
    let vm = VM::new();

    declare_commons(&vm);

    create_throw_exception_func(&vm);
    create_catch_exception_func(&vm, false);

    vm
}

#[test]
#[ignore]
// issue: didn't restore callee-saved register correctly, temporarily ignore this test
// FIXME: fix the bug
fn test_exception_throw_catch_and_add() {
    VM::start_logging_trace();
    let vm = Arc::new(throw_catch_and_add());

    let compiler = Compiler::new(CompilerPolicy::default(), &vm);

    let func_throw = vm.id_of("throw_exception");
    let func_catch = vm.id_of("catch_and_add");
    {
        let funcs = vm.funcs().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();

        {
            let func = funcs.get(&func_throw).unwrap().read().unwrap();
            let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();

            compiler.compile(&mut func_ver);
        }
        {
            let func = funcs.get(&func_catch).unwrap().read().unwrap();
            let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();

            compiler.compile(&mut func_ver);
        }
    }

    vm.make_primordial_thread(func_catch, true, vec![]);
    backend::emit_context(&vm);

    let executable = aot::link_primordial(vec![Mu("throw_exception"), Mu("catch_and_add")], "throw_catch_and_add", &vm);
    let output = aot::execute_nocheck(executable);

    // throw 1, add 0, 1, 2, 3, 4
    assert!(output.status.code().is_some());
    assert_eq!(output.status.code().unwrap(), 11);
}

fn throw_catch_and_add() -> VM {
    let vm = VM::new();

    declare_commons(&vm);

    create_throw_exception_func(&vm);
    create_catch_exception_and_add(&vm);

    vm
}

fn create_catch_exception_and_add(vm: &VM) {
    let throw_exception_sig = vm.get_func_sig(vm.id_of("throw_exception_sig"));
    let throw_exception_id = vm.id_of("throw_exception");

    let int64 = vm.get_type(vm.id_of("int64"));
    constdef!   ((vm) <int64> int64_0 = Constant::Int(0));
    constdef!   ((vm) <int64> int64_1 = Constant::Int(1));
    constdef!   ((vm) <int64> int64_2 = Constant::Int(2));
    constdef!   ((vm) <int64> int64_3 = Constant::Int(3));
    constdef!   ((vm) <int64> int64_4 = Constant::Int(4));
    constdef!   ((vm) <int64> int64_5 = Constant::Int(5));

    typedef!    ((vm) type_funcref_throw_exception  = mu_funcref(throw_exception_sig));
    constdef!   ((vm) <type_funcref_throw_exception> const_funcref_throw_exception = Constant::FuncRef(throw_exception_id));

    funcsig!    ((vm) catch_exception_sig = () -> ());
    funcdecl!   ((vm) <catch_exception_sig> catch_and_add);
    funcdef!    ((vm) <catch_exception_sig> catch_and_add VERSION catch_and_add_v1);

    // blk_entry
    consta!     ((vm, catch_and_add_v1) int0_local = int64_0);
    consta!     ((vm, catch_and_add_v1) int1_local = int64_1);
    consta!     ((vm, catch_and_add_v1) int2_local = int64_2);
    consta!     ((vm, catch_and_add_v1) int3_local = int64_3);
    consta!     ((vm, catch_and_add_v1) int4_local = int64_4);

    block!      ((vm, catch_and_add_v1) blk_entry);
    block!      ((vm, catch_and_add_v1) blk_main);
    inst!       ((vm, catch_and_add_v1) blk_entry_branch:
        BRANCH blk_main (int0_local, int1_local, int2_local, int3_local, int4_local)
    );

    define_block!   ((vm, catch_and_add_v1) blk_entry () {
        blk_entry_branch
    });

    ssa!        ((vm, catch_and_add_v1) <int64> v0);
    ssa!        ((vm, catch_and_add_v1) <int64> v1);
    ssa!        ((vm, catch_and_add_v1) <int64> v2);
    ssa!        ((vm, catch_and_add_v1) <int64> v3);
    ssa!        ((vm, catch_and_add_v1) <int64> v4);

    // blk_main
    consta!     ((vm, catch_and_add_v1) funcref_throw_local = const_funcref_throw_exception);
    block!      ((vm, catch_and_add_v1) blk_normal);
    block!      ((vm, catch_and_add_v1) blk_exception);
    inst!       ((vm, catch_and_add_v1) blk_main_call:
        CALL (funcref_throw_local, v0, v1, v2, v3, v4) FUNC(0) (vec![]) CallConvention::Mu,
            normal: blk_normal (vec![]),
            exc   : blk_exception (vec![
                DestArg::Normal(1),
                DestArg::Normal(2),
                DestArg::Normal(3),
                DestArg::Normal(4),
                DestArg::Normal(5),
            ])
    );
    define_block!   ((vm, catch_and_add_v1) blk_main(v0, v1, v2, v3, v4) {
        blk_main_call
    });

    // blk_normal
    inst!       ((vm, catch_and_add_v1) blk_normal_threadexit:
        THREADEXIT
    );
    define_block!   ((vm, catch_and_add_v1) blk_normal() {
        blk_normal_threadexit
    });

    // blk_exception
    ssa!        ((vm, catch_and_add_v1) <int64> ev0);
    ssa!        ((vm, catch_and_add_v1) <int64> ev1);
    ssa!        ((vm, catch_and_add_v1) <int64> ev2);
    ssa!        ((vm, catch_and_add_v1) <int64> ev3);
    ssa!        ((vm, catch_and_add_v1) <int64> ev4);
    ssa!        ((vm, catch_and_add_v1) <int64> exc_arg);

    inst!       ((vm, catch_and_add_v1) blk_exception_px0:
        PRINTHEX ev0
    );
    inst!       ((vm, catch_and_add_v1) blk_exception_px1:
        PRINTHEX ev1
    );
    inst!       ((vm, catch_and_add_v1) blk_exception_px2:
        PRINTHEX ev2
    );
    inst!       ((vm, catch_and_add_v1) blk_exception_px3:
        PRINTHEX ev3
    );
    inst!       ((vm, catch_and_add_v1) blk_exception_px4:
        PRINTHEX ev4
    );
    inst!       ((vm, catch_and_add_v1) blk_exception_px5:
        PRINTHEX exc_arg
    );

    ssa!        ((vm, catch_and_add_v1) <int64> res0);
    inst!       ((vm, catch_and_add_v1) blk_exception_add0:
        res0 = BINOP (BinOp::Add) exc_arg ev0
    );

    ssa!        ((vm, catch_and_add_v1) <int64> res1);
    inst!       ((vm, catch_and_add_v1) blk_exception_add1:
        res1 = BINOP (BinOp::Add) res0 ev1
    );

    ssa!        ((vm, catch_and_add_v1) <int64> res2);
    inst!       ((vm, catch_and_add_v1) blk_exception_add2:
        res2 = BINOP (BinOp::Add) res1 ev2
    );

    ssa!        ((vm, catch_and_add_v1) <int64> res3);
    inst!       ((vm, catch_and_add_v1) blk_exception_add3:
        res3 = BINOP (BinOp::Add) res2 ev3
    );

    ssa!        ((vm, catch_and_add_v1) <int64> res4);
    inst!       ((vm, catch_and_add_v1) blk_exception_add4:
        res4 = BINOP (BinOp::Add) res3 ev4
    );

    let blk_exception_exit = gen_ccall_exit(res4.clone(), &mut catch_and_add_v1, &vm);

    inst!       ((vm, catch_and_add_v1) blk_exception_ret:
        RET (res4)
    );

    define_block!   ((vm, catch_and_add_v1) blk_exception(ev0, ev1, ev2, ev3, ev4) [exc_arg] {
        blk_exception_px0,
        blk_exception_px1,
        blk_exception_px2,
        blk_exception_px3,
        blk_exception_px4,
        blk_exception_px5,

        blk_exception_add0,
        blk_exception_add1,
        blk_exception_add2,
        blk_exception_add3,
        blk_exception_add4,

        blk_exception_exit,
        blk_exception_ret
    });

    define_func_ver!((vm) catch_and_add_v1 (entry: blk_entry) {
        blk_entry, blk_main, blk_normal, blk_exception
    });
}