use mu::ast::types::*;
use mu::ast::ir::*;
use mu::ast::inst::*;
use mu::ast::op::*;
use mu::vm::*;
use mu::compiler::*;
use mu::runtime::thread::MuThread;
use mu::utils::Address;

use std::sync::Arc;
use std::sync::RwLock;
use std::collections::HashMap;
use mu::testutil::aot;

use test_compiler::test_call::gen_ccall_exit;

#[test]
fn test_struct() {
    VM::start_logging_trace();

    let vm = Arc::new(struct_insts());

    let compiler = Compiler::new(CompilerPolicy::default(), vm.clone());

    let func_id = vm.id_of("struct_insts");
    {
        let funcs = vm.funcs().read().unwrap();
        let func = funcs.get(&func_id).unwrap().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();
        let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();

        compiler.compile(&mut func_ver);
    }

    vm.make_primordial_thread(func_id, vec![]);
    backend::emit_context(&vm);

    let executable = aot::link_primordial(vec!["struct_insts".to_string()], "struct_insts_test");
    let output = aot::execute_nocheck(executable);

    assert!(output.status.code().is_some());

    let ret_code = output.status.code().unwrap();
    println!("return code: {}", ret_code);
    assert!(ret_code == 1);
}

pub fn struct_insts() -> VM {
    let vm = VM::new();

    // .typedef @int64 = int<64>
    let int64 = vm.declare_type(vm.next_id(), MuType_::int(64));
    vm.set_name(int64.as_entity(), Mu("int64"));
    // .typedef @point = struct<@int64 @int64>
    let struct_point = vm.declare_type(vm.next_id(), MuType_::mustruct("Point".to_string(), vec![int64.clone(), int64.clone()]));
    vm.set_name(struct_point.as_entity(), Mu("point"));
    // .typedef @ref_point = ref<@point>
    let ref_point = vm.declare_type(vm.next_id(), MuType_::muref(struct_point.clone()));
    vm.set_name(ref_point.as_entity(), Mu("ref_point"));
    // .typedef @iref_point = iref<@point>
    let iref_point = vm.declare_type(vm.next_id(), MuType_::iref(struct_point.clone()));
    vm.set_name(iref_point.as_entity(), Mu("iref_point"));
    // .typedef @iref_int64 = iref<@int64>
    let iref_int64 = vm.declare_type(vm.next_id(), MuType_::iref(int64.clone()));
    vm.set_name(iref_int64.as_entity(), Mu("iref_int64"));

    // .const @int64_0 <@int64> = 0
    let int64_0 = vm.declare_const(vm.next_id(), int64.clone(), Constant::Int(0));
    vm.set_name(int64_0.as_entity(), Mu("int64_0"));
    // .const @int64_1 <@int64> = 1
    let int64_1 = vm.declare_const(vm.next_id(), int64.clone(), Constant::Int(1));
    vm.set_name(int64_1.as_entity(), Mu("int64_1"));

    // .funcsig @noparam_noret_sig = () -> ()
    let noparam_noret_sig = vm.declare_func_sig(vm.next_id(), vec![], vec![]);
    vm.set_name(noparam_noret_sig.as_entity(), Mu("noparam_noret_sig"));

    // .funcdecl @struct_insts <@noparam_noret_sig>
    let func = MuFunction::new(vm.next_id(), noparam_noret_sig.clone());
    vm.set_name(func.as_entity(), Mu("struct_insts"));
    let func_id = func.id();
    vm.declare_func(func);

    // .funcdef @struct_insts VERSION @struct_insts <@noparam_noret_si>
    let mut func_ver = MuFunctionVersion::new(vm.next_id(), func_id, noparam_noret_sig.clone());
    vm.set_name(func_ver.as_entity(), Mu("struct_insts_v1"));

    // %entry():
    let mut blk_entry = Block::new(vm.next_id());
    vm.set_name(blk_entry.as_entity(), Mu("entry"));

    // %a = NEW <@point>
    let blk_entry_a = func_ver.new_ssa(vm.next_id(), ref_point.clone());
    vm.set_name(blk_entry_a.as_entity(), Mu("blk_entry_a"));

    let blk_entry_inst0 = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_entry_a.clone_value()]),
        ops: RwLock::new(vec![]),
        v: Instruction_::New(struct_point.clone())
    });

    // %iref_a = GETIREF <@int64> %a
    let blk_entry_iref_a = func_ver.new_ssa(vm.next_id(), iref_point.clone());
    vm.set_name(blk_entry_iref_a.as_entity(), Mu("blk_entry_iref_a"));

    let blk_entry_inst1 = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_entry_iref_a.clone_value()]),
        ops: RwLock::new(vec![blk_entry_a.clone()]),
        v: Instruction_::GetIRef(0)
    });

    // %iref_x = GETFIELDIREF <@point 0> %iref_a
    let blk_entry_iref_x = func_ver.new_ssa(vm.next_id(), iref_int64.clone());
    vm.set_name(blk_entry_iref_x.as_entity(), Mu("blk_entry_iref_x"));
    let int64_0_local = func_ver.new_constant(int64_0.clone());

    let blk_entry_inst2 = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_entry_iref_x.clone_value()]),
        ops: RwLock::new(vec![blk_entry_iref_a.clone()]),
        v: Instruction_::GetFieldIRef {
            is_ptr: false,
            base: 0,    // 0th node in ops
            index: 0    // 0th element in the struct
        }
    });

    // STORE <@int64> %iref_x @int64_1
    let int64_1_local = func_ver.new_constant(int64_1.clone());
    let blk_entry_inst3 = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: None,
        ops: RwLock::new(vec![blk_entry_iref_x.clone(), int64_1_local.clone()]),
        v: Instruction_::Store{
            is_ptr: false,
            order: MemoryOrder::Relaxed,
            mem_loc: 0,
            value: 1
        }
    });

    // BRANCH %check(%a)
    let blk_check_id = vm.next_id();

    let blk_entry_branch = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: None,
        ops: RwLock::new(vec![blk_entry_a]),
        v: Instruction_::Branch1(Destination{
            target: blk_check_id,
            args: vec![DestArg::Normal(0)]
        })
    });

    blk_entry.content = Some(BlockContent{
        args: vec![],
        exn_arg: None,
        body: vec![blk_entry_inst0, blk_entry_inst1, blk_entry_inst2, blk_entry_inst3, blk_entry_branch],
        keepalives: None
    });

    // %check(%a):
    let blk_check_a = func_ver.new_ssa(vm.next_id(), ref_point.clone());
    vm.set_name(blk_check_a.as_entity(), Mu("blk_check_a"));

    let mut blk_check = Block::new(blk_check_id);
    vm.set_name(blk_check.as_entity(), Mu("check"));

    // %blk_check_iref_a = GETIREF <@point> a
    let blk_check_iref_a = func_ver.new_ssa(vm.next_id(), iref_point.clone());
    vm.set_name(blk_check_iref_a.as_entity(), Mu("blk_check_iref_a"));

    let blk_check_inst0 = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_check_iref_a.clone_value()]),
        ops: RwLock::new(vec![blk_check_a.clone()]),
        v: Instruction_::GetIRef(0)
    });

    // %blk_check_iref_x = GETFIELDIREF <@point 0> %blk_check_iref_a
    let blk_check_iref_x = func_ver.new_ssa(vm.next_id(), iref_int64.clone());
    vm.set_name(blk_check_iref_x.as_entity(), Mu("blk_check_iref_x"));

    let blk_check_inst1 = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_check_iref_x.clone_value()]),
        ops: RwLock::new(vec![blk_check_iref_a.clone()]),
        v: Instruction_::GetFieldIRef {
            is_ptr: false,
            base: 0,    // 0th node in ops
            index: 0    // 0th element in the struct
        }
    });

    // %x = LOAD <@int64> %blk_check_iref_x
    let blk_check_x = func_ver.new_ssa(vm.next_id(), int64.clone());
    vm.set_name(blk_check_x.as_entity(), Mu("blk_check_x"));
    let blk_check_inst2 = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_check_x.clone_value()]),
        ops: RwLock::new(vec![blk_check_iref_x.clone()]),
        v: Instruction_::Load {
            is_ptr: false,
            order: MemoryOrder::Relaxed,
            mem_loc: 0
        }
    });

    // %blk_check_iref_y = GETFIELDIREF <@point 1> %blk_check_iref_a
    let blk_check_iref_y = func_ver.new_ssa(vm.next_id(), iref_int64.clone());
    vm.set_name(blk_check_iref_y.as_entity(), Mu("blk_check_iref_y"));

    let blk_check_inst3 = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_check_iref_y.clone_value()]),
        ops: RwLock::new(vec![blk_check_iref_a.clone()]),
        v: Instruction_::GetFieldIRef {
            is_ptr: false,
            base: 0,    // 0th node in ops
            index: 1    // 1th element in the struct
        }
    });

    // %y = LOAD <@int64> %blk_check_iref_y
    let blk_check_y = func_ver.new_ssa(vm.next_id(), int64.clone());
    vm.set_name(blk_check_y.as_entity(), Mu("blk_check_y"));
    let blk_check_inst4 = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_check_y.clone_value()]),
        ops: RwLock::new(vec![blk_check_iref_y.clone()]),
        v: Instruction_::Load {
            is_ptr: false,
            order: MemoryOrder::Relaxed,
            mem_loc: 0
        }
    });

    // %res = ADD <@int64> %x %y
    let blk_check_res = func_ver.new_ssa(vm.next_id(), int64.clone());
    vm.set_name(blk_check_res.as_entity(), Mu("blk_check_res"));
    let blk_check_inst5 = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_check_res.clone_value()]),
        ops: RwLock::new(vec![blk_check_x.clone(), blk_check_y.clone()]),
        v: Instruction_::BinOp(BinOp::Add, 0, 1)
    });

    // CCALL exit(%res)
    let blk_check_ccall = gen_ccall_exit(blk_check_res.clone(), &mut func_ver, &vm);

    // RET <@int64> 0
    let blk_check_ret = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: None,
        ops: RwLock::new(vec![int64_0_local]),
        v: Instruction_::Return(vec![0])
    });

    blk_check.content = Some(BlockContent{
        args: vec![blk_check_a.clone_value()],
        exn_arg: None,
        body: vec![
            blk_check_inst0,
            blk_check_inst1,
            blk_check_inst2,
            blk_check_inst3,
            blk_check_inst4,
            blk_check_inst5,
            blk_check_ccall,
            blk_check_ret
        ],
        keepalives: None
    });

    func_ver.define(FunctionContent{
        entry: blk_entry.id(),
        blocks: hashmap!{
            blk_entry.id() => blk_entry,
            blk_check_id   => blk_check
        }
    });

    vm.define_func_version(func_ver);

    vm
}

#[test]
fn test_hybrid_fix_part() {
    VM::start_logging_trace();

    let vm = Arc::new(hybrid_fix_part_insts());

    let compiler = Compiler::new(CompilerPolicy::default(), vm.clone());

    let func_id = vm.id_of("hybrid_fix_part_insts");
    {
        let funcs = vm.funcs().read().unwrap();
        let func = funcs.get(&func_id).unwrap().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();
        let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();

        compiler.compile(&mut func_ver);
    }

    vm.make_primordial_thread(func_id, vec![]);
    backend::emit_context(&vm);

    let executable = aot::link_primordial(vec!["hybrid_fix_part_insts".to_string()], "hybrid_fix_part_insts_test");
    let output = aot::execute_nocheck(executable);

    assert!(output.status.code().is_some());

    let ret_code = output.status.code().unwrap();
    println!("return code: {}", ret_code);
    assert!(ret_code == 1);
}

pub fn hybrid_fix_part_insts() -> VM {
    let vm = VM::new();

    // .typedef @int64 = int<64>
    let int64 = vm.declare_type(vm.next_id(), MuType_::int(64));
    vm.set_name(int64.as_entity(), Mu("int64"));
    // .typedef @my_hybrid = hybrid<@int64 @int64 | @int64>
    let my_hybrid = vm.declare_type(vm.next_id(), MuType_::hybrid("MyHybrid".to_string(), vec![int64.clone(), int64.clone()], int64.clone()));
    vm.set_name(my_hybrid.as_entity(), Mu("my_hybrid"));
    // .typedef @ref_hybrid = ref<@my_hybrid>
    let ref_hybrid = vm.declare_type(vm.next_id(), MuType_::muref(my_hybrid.clone()));
    vm.set_name(ref_hybrid.as_entity(), Mu("ref_hybrid"));
    // .typedef @iref_hybrid = iref<@my_hybrid>
    let iref_hybrid = vm.declare_type(vm.next_id(), MuType_::iref(my_hybrid.clone()));
    vm.set_name(iref_hybrid.as_entity(), Mu("iref_hybrid"));
    // .typedef @iref_int64 = iref<@int64>
    let iref_int64 = vm.declare_type(vm.next_id(), MuType_::iref(int64.clone()));
    vm.set_name(iref_int64.as_entity(), Mu("iref_int64"));

    // .const @int64_0 <@int64> = 0
    let int64_0 = vm.declare_const(vm.next_id(), int64.clone(), Constant::Int(0));
    vm.set_name(int64_0.as_entity(), Mu("int64_0"));
    // .const @int64_1 <@int64> = 1
    let int64_1 = vm.declare_const(vm.next_id(), int64.clone(), Constant::Int(1));
    vm.set_name(int64_1.as_entity(), Mu("int64_1"));
    // .const @int64_10 <@int64> = 10
    let int64_10 = vm.declare_const(vm.next_id(), int64.clone(), Constant::Int(10));
    vm.set_name(int64_10.as_entity(), Mu("int64_10"));

    // .funcsig @noparam_noret_sig = () -> ()
    let noparam_noret_sig = vm.declare_func_sig(vm.next_id(), vec![], vec![]);
    vm.set_name(noparam_noret_sig.as_entity(), Mu("noparam_noret_sig"));

    // .funcdecl @hybrid_fix_part_insts <@noparam_noret_sig>
    let func = MuFunction::new(vm.next_id(), noparam_noret_sig.clone());
    vm.set_name(func.as_entity(), Mu("hybrid_fix_part_insts"));
    let func_id = func.id();
    vm.declare_func(func);

    // .funcdef @hybrid_fix_part_insts VERSION @hybrid_fix_part_insts_v1 <@noparam_noret_si>
    let mut func_ver = MuFunctionVersion::new(vm.next_id(), func_id, noparam_noret_sig.clone());
    vm.set_name(func_ver.as_entity(), Mu("hybrid_fix_part_insts_v1"));

    // %entry():
    let mut blk_entry = Block::new(vm.next_id());
    vm.set_name(blk_entry.as_entity(), Mu("entry"));

    // %a = NEWHYBRID <@my_hybrid @int64> @int64_10
    let blk_entry_a = func_ver.new_ssa(vm.next_id(), ref_hybrid.clone());
    vm.set_name(blk_entry_a.as_entity(), Mu("blk_entry_a"));

    let int64_10_local = func_ver.new_constant(int64_10.clone());

    let blk_entry_inst0 = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_entry_a.clone_value()]),
        ops: RwLock::new(vec![int64_10_local]),
        v: Instruction_::NewHybrid(my_hybrid.clone(), 0)
    });

    // %iref_a = GETIREF <@int64> %a
    let blk_entry_iref_a = func_ver.new_ssa(vm.next_id(), iref_hybrid.clone());
    vm.set_name(blk_entry_iref_a.as_entity(), Mu("blk_entry_iref_a"));

    let blk_entry_inst1 = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_entry_iref_a.clone_value()]),
        ops: RwLock::new(vec![blk_entry_a.clone()]),
        v: Instruction_::GetIRef(0)
    });

    // %iref_x = GETFIELDIREF <@my_hybrid 0> %iref_a
    let blk_entry_iref_x = func_ver.new_ssa(vm.next_id(), iref_int64.clone());
    vm.set_name(blk_entry_iref_x.as_entity(), Mu("blk_entry_iref_x"));
    let int64_0_local = func_ver.new_constant(int64_0.clone());

    let blk_entry_inst2 = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_entry_iref_x.clone_value()]),
        ops: RwLock::new(vec![blk_entry_iref_a.clone()]),
        v: Instruction_::GetFieldIRef {
            is_ptr: false,
            base: 0,    // 0th node in ops
            index: 0    // 0th element in the struct
        }
    });

    // STORE <@int64> %iref_x @int64_1
    let int64_1_local = func_ver.new_constant(int64_1.clone());
    let blk_entry_inst3 = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: None,
        ops: RwLock::new(vec![blk_entry_iref_x.clone(), int64_1_local.clone()]),
        v: Instruction_::Store{
            is_ptr: false,
            order: MemoryOrder::Relaxed,
            mem_loc: 0,
            value: 1
        }
    });

    // BRANCH %check(%a)
    let blk_check_id = vm.next_id();

    let blk_entry_branch = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: None,
        ops: RwLock::new(vec![blk_entry_a]),
        v: Instruction_::Branch1(Destination{
            target: blk_check_id,
            args: vec![DestArg::Normal(0)]
        })
    });

    blk_entry.content = Some(BlockContent{
        args: vec![],
        exn_arg: None,
        body: vec![blk_entry_inst0, blk_entry_inst1, blk_entry_inst2, blk_entry_inst3, blk_entry_branch],
        keepalives: None
    });

    // %check(%a):
    let blk_check_a = func_ver.new_ssa(vm.next_id(), ref_hybrid.clone());
    vm.set_name(blk_check_a.as_entity(), Mu("blk_check_a"));

    let mut blk_check = Block::new(blk_check_id);
    vm.set_name(blk_check.as_entity(), Mu("check"));

    // %blk_check_iref_a = GETIREF <@my_hybrid> a
    let blk_check_iref_a = func_ver.new_ssa(vm.next_id(), iref_hybrid.clone());
    vm.set_name(blk_check_iref_a.as_entity(), Mu("blk_check_iref_a"));

    let blk_check_inst0 = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_check_iref_a.clone_value()]),
        ops: RwLock::new(vec![blk_check_a.clone()]),
        v: Instruction_::GetIRef(0)
    });

    // %blk_check_iref_x = GETFIELDIREF <@my_hybrid 0> %blk_check_iref_a
    let blk_check_iref_x = func_ver.new_ssa(vm.next_id(), iref_int64.clone());
    vm.set_name(blk_check_iref_x.as_entity(), Mu("blk_check_iref_x"));

    let blk_check_inst1 = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_check_iref_x.clone_value()]),
        ops: RwLock::new(vec![blk_check_iref_a.clone()]),
        v: Instruction_::GetFieldIRef {
            is_ptr: false,
            base: 0,    // 0th node in ops
            index: 0    // 0th element in the struct
        }
    });

    // %x = LOAD <@int64> %blk_check_iref_x
    let blk_check_x = func_ver.new_ssa(vm.next_id(), int64.clone());
    vm.set_name(blk_check_x.as_entity(), Mu("blk_check_x"));
    let blk_check_inst2 = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_check_x.clone_value()]),
        ops: RwLock::new(vec![blk_check_iref_x.clone()]),
        v: Instruction_::Load {
            is_ptr: false,
            order: MemoryOrder::Relaxed,
            mem_loc: 0
        }
    });

    // %blk_check_iref_y = GETFIELDIREF <@my_hybrid 1> %blk_check_iref_a
    let blk_check_iref_y = func_ver.new_ssa(vm.next_id(), iref_int64.clone());
    vm.set_name(blk_check_iref_y.as_entity(), Mu("blk_check_iref_y"));

    let blk_check_inst3 = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_check_iref_y.clone_value()]),
        ops: RwLock::new(vec![blk_check_iref_a.clone()]),
        v: Instruction_::GetFieldIRef {
            is_ptr: false,
            base: 0,    // 0th node in ops
            index: 1    // 1th element in the struct
        }
    });

    // %y = LOAD <@int64> %blk_check_iref_y
    let blk_check_y = func_ver.new_ssa(vm.next_id(), int64.clone());
    vm.set_name(blk_check_y.as_entity(), Mu("blk_check_y"));
    let blk_check_inst4 = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_check_y.clone_value()]),
        ops: RwLock::new(vec![blk_check_iref_y.clone()]),
        v: Instruction_::Load {
            is_ptr: false,
            order: MemoryOrder::Relaxed,
            mem_loc: 0
        }
    });

    // %res = ADD <@int64> %x %y
    let blk_check_res = func_ver.new_ssa(vm.next_id(), int64.clone());
    vm.set_name(blk_check_res.as_entity(), Mu("blk_check_res"));
    let blk_check_inst5 = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_check_res.clone_value()]),
        ops: RwLock::new(vec![blk_check_x.clone(), blk_check_y.clone()]),
        v: Instruction_::BinOp(BinOp::Add, 0, 1)
    });

    // CCALL exit(%res)
    let blk_check_ccall = gen_ccall_exit(blk_check_res.clone(), &mut func_ver, &vm);

    // RET <@int64> 0
    let blk_check_ret = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: None,
        ops: RwLock::new(vec![int64_0_local]),
        v: Instruction_::Return(vec![0])
    });

    blk_check.content = Some(BlockContent{
        args: vec![blk_check_a.clone_value()],
        exn_arg: None,
        body: vec![
        blk_check_inst0,
        blk_check_inst1,
        blk_check_inst2,
        blk_check_inst3,
        blk_check_inst4,
        blk_check_inst5,
        blk_check_ccall,
        blk_check_ret
        ],
        keepalives: None
    });

    func_ver.define(FunctionContent{
        entry: blk_entry.id(),
        blocks: hashmap!{
            blk_entry.id() => blk_entry,
            blk_check_id   => blk_check
        }
    });

    vm.define_func_version(func_ver);

    vm
}