use mu::ast::types::*;
use mu::ast::ir::*;
use mu::ast::inst::*;
use mu::ast::op::*;
use mu::vm::*;
use mu::compiler::*;

use std::sync::Arc;
use std::sync::RwLock;
use mu::testutil::aot;

use test_compiler::test_call::gen_ccall_exit;

#[test]
fn test_struct() {
    VM::start_logging_trace();

    let vm = Arc::new(struct_insts_macro());

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

    let executable = aot::link_primordial(vec!["struct_insts".to_string()], "struct_insts_test", &vm);
    let output = aot::execute_nocheck(executable);

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
                RET (blk_check_res)
    );

    define_block! ((vm, struct_insts_v1) blk_check(blk_check_a) {
        blk_check_inst0, blk_check_inst1, blk_check_inst2, blk_check_inst3, blk_check_inst4, blk_check_inst5, blk_check_ccall, blk_check_ret
    });

    define_func_ver! ((vm) struct_insts_v1 (entry: blk_entry) {blk_entry, blk_check});

    vm
}

#[allow(dead_code)]
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

    let executable = aot::link_primordial(vec!["hybrid_fix_part_insts".to_string()], "hybrid_fix_part_insts_test", &vm);
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

#[test]
fn test_hybrid_var_part() {
    VM::start_logging_trace();

    let vm = Arc::new(hybrid_var_part_insts());

    let compiler = Compiler::new(CompilerPolicy::default(), vm.clone());

    let func_id = vm.id_of("hybrid_var_part_insts");
    {
        let funcs = vm.funcs().read().unwrap();
        let func = funcs.get(&func_id).unwrap().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();
        let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();

        compiler.compile(&mut func_ver);
    }

    vm.make_primordial_thread(func_id, vec![]);
    backend::emit_context(&vm);

    let executable = aot::link_primordial(vec!["hybrid_var_part_insts".to_string()], "hybrid_var_part_insts_test", &vm);
    let output = aot::execute_nocheck(executable);

    assert!(output.status.code().is_some());

    let ret_code = output.status.code().unwrap();
    println!("return code: {}", ret_code);
    assert!(ret_code == 20);
}

pub fn hybrid_var_part_insts() -> VM {
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
    // .const @int64_2 <@int64> = 2
    let int64_2 = vm.declare_const(vm.next_id(), int64.clone(), Constant::Int(2));
    vm.set_name(int64_2.as_entity(), Mu("int64_2"));
    // .const @int64_3 <@int64> = 3
    let int64_3 = vm.declare_const(vm.next_id(), int64.clone(), Constant::Int(3));
    vm.set_name(int64_3.as_entity(), Mu("int64_3"));
    // .const @int64_4 <@int64> = 4
    let int64_4 = vm.declare_const(vm.next_id(), int64.clone(), Constant::Int(4));
    vm.set_name(int64_4.as_entity(), Mu("int64_4"));
    // .const @int64_10 <@int64> = 10
    let int64_10 = vm.declare_const(vm.next_id(), int64.clone(), Constant::Int(10));
    vm.set_name(int64_10.as_entity(), Mu("int64_10"));

    // .funcsig @noparam_noret_sig = () -> ()
    let noparam_noret_sig = vm.declare_func_sig(vm.next_id(), vec![], vec![]);
    vm.set_name(noparam_noret_sig.as_entity(), Mu("noparam_noret_sig"));

    // .funcdecl @hybrid_var_part_insts <@noparam_noret_sig>
    let func = MuFunction::new(vm.next_id(), noparam_noret_sig.clone());
    vm.set_name(func.as_entity(), Mu("hybrid_var_part_insts"));
    let func_id = func.id();
    vm.declare_func(func);

    // .funcdef @hybrid_var_part_insts VERSION @hybrid_var_part_insts_v1 <@noparam_noret_si>
    let mut func_ver = MuFunctionVersion::new(vm.next_id(), func_id, noparam_noret_sig.clone());
    vm.set_name(func_ver.as_entity(), Mu("hybrid_var_part_insts_v1"));

    // %entry():
    let mut blk_entry = Block::new(vm.next_id());
    vm.set_name(blk_entry.as_entity(), Mu("entry"));

    // %a = NEWHYBRID <@my_hybrid @int64> @int64_10
    let blk_entry_a = func_ver.new_ssa(vm.next_id(), ref_hybrid.clone());
    vm.set_name(blk_entry_a.as_entity(), Mu("blk_entry_a"));

    let int64_0_local = func_ver.new_constant(int64_0.clone());
    let int64_1_local = func_ver.new_constant(int64_1.clone());
    let int64_4_local = func_ver.new_constant(int64_4.clone());
    let int64_10_local = func_ver.new_constant(int64_10.clone());

    let blk_entry_inst_newhybrid = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_entry_a.clone_value()]),
        ops: RwLock::new(vec![int64_10_local.clone()]),
        v: Instruction_::NewHybrid(my_hybrid.clone(), 0)
    });

    // %iref_a = GETIREF <@int64> %a
    let blk_entry_iref_a = func_ver.new_ssa(vm.next_id(), iref_hybrid.clone());
    vm.set_name(blk_entry_iref_a.as_entity(), Mu("blk_entry_iref_a"));

    let blk_entry_inst_getiref = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_entry_iref_a.clone_value()]),
        ops: RwLock::new(vec![blk_entry_a.clone()]),
        v: Instruction_::GetIRef(0)
    });

    // %iref_var = GETVARPARTIREF <@my_hybrid> %iref_a
    let blk_entry_iref_var = func_ver.new_ssa(vm.next_id(), iref_int64.clone());
    vm.set_name(blk_entry_iref_var.as_entity(), Mu("blk_entry_iref_var"));

    let blk_entry_inst_getvarpart = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_entry_iref_var.clone_value()]),
        ops: RwLock::new(vec![blk_entry_iref_a.clone()]),
        v: Instruction_::GetVarPartIRef{
            is_ptr: false,
            base: 0
        }
    });

    // %var0 = SHIFTIREF <@int64> %iref_var %int64_0
    let blk_entry_var0 = func_ver.new_ssa(vm.next_id(), iref_int64.clone());
    vm.set_name(blk_entry_var0.as_entity(), Mu("blk_entry_var0"));

    let blk_entry_inst_shiftiref_0 = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_entry_var0.clone_value()]),
        ops: RwLock::new(vec![blk_entry_iref_var.clone(), int64_0_local.clone()]),
        v: Instruction_::ShiftIRef {
            is_ptr: false,
            base: 0,
            offset: 1
        }
    });

    // STORE <@int64> %var0 @int64_10
    let blk_entry_inst_store_0 = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: None,
        ops: RwLock::new(vec![blk_entry_var0.clone(), int64_10_local.clone()]),
        v: Instruction_::Store{
            is_ptr: false,
            order: MemoryOrder::Relaxed,
            mem_loc: 0,
            value: 1
        }
    });

    // %var4 = SHIFTIREF <@int64> %iref_var %int64_4
    let blk_entry_var4 = func_ver.new_ssa(vm.next_id(), iref_int64.clone());
    vm.set_name(blk_entry_var4.as_entity(), Mu("blk_entry_var4"));

    let blk_entry_inst_shiftiref_4 = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_entry_var4.clone_value()]),
        ops: RwLock::new(vec![blk_entry_iref_var.clone(), int64_4_local.clone()]),
        v: Instruction_::ShiftIRef {
            is_ptr: false,
            base: 0,
            offset: 1
        }
    });

    // STORE <@int64> %var4 @int64_10
    let blk_entry_inst_store_4 = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: None,
        ops: RwLock::new(vec![blk_entry_var4.clone(), int64_10_local.clone()]),
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
        body: vec![
        blk_entry_inst_newhybrid,
        blk_entry_inst_getiref,
        blk_entry_inst_getvarpart,
        blk_entry_inst_shiftiref_0,
        blk_entry_inst_store_0,
        blk_entry_inst_shiftiref_4,
        blk_entry_inst_store_4,
        blk_entry_branch
        ],
        keepalives: None
    });

    // %check(%a):
    let blk_check_a = func_ver.new_ssa(vm.next_id(), ref_hybrid.clone());
    vm.set_name(blk_check_a.as_entity(), Mu("blk_check_a"));
    let mut blk_check = Block::new(blk_check_id);
    vm.set_name(blk_check.as_entity(), Mu("check"));

    // BRANCH %head (<@int64> sum, <@int64> n, <@int64> %i, <@ref_hybrid> %a)
    //                        0             10            0
    let blk_head_id = vm.next_id();
    let blk_check_branch = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: None,
        ops: RwLock::new(vec![int64_0_local.clone(), int64_10_local.clone(), int64_0_local.clone(), blk_check_a.clone()]),
        v: Instruction_::Branch1(Destination{
            target: blk_head_id,
            args: vec![DestArg::Normal(0), DestArg::Normal(1), DestArg::Normal(2), DestArg::Normal(3)]
        })
    });

    blk_check.content = Some(BlockContent{
        args: vec![blk_check_a.clone_value()],
        exn_arg: None,
        body: vec![blk_check_branch],
        keepalives: None
    });

    // %head(%sum, %n, %i, %a)
    let blk_head_sum = func_ver.new_ssa(vm.next_id(), int64.clone());
    vm.set_name(blk_head_sum.as_entity(), Mu("blk_head_sum"));
    let blk_head_n   = func_ver.new_ssa(vm.next_id(), int64.clone());
    vm.set_name(blk_head_n.as_entity(), Mu("blk_head_n"));
    let blk_head_i   = func_ver.new_ssa(vm.next_id(), int64.clone());
    vm.set_name(blk_head_i.as_entity(), Mu("blk_head_i"));
    let blk_head_a   = func_ver.new_ssa(vm.next_id(), ref_hybrid.clone());
    vm.set_name(blk_head_a.as_entity(), Mu("blk_head_a"));

    let mut blk_head = Block::new(blk_head_id);
    vm.set_name(blk_head.as_entity(), Mu("head"));

    // %cond = SLT <@int64> %i %n
    let blk_head_cond = func_ver.new_ssa(vm.next_id(), int64.clone());
    vm.set_name(blk_head_cond.as_entity(), Mu("blk_head_cond"));

    let blk_head_inst_slt = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_head_cond.clone_value()]),
        ops: RwLock::new(vec![blk_head_i.clone(), blk_head_n.clone()]),
        v: Instruction_::CmpOp(CmpOp::SLT, 0, 1)
    });

    // BRANCH2 %cond %body(%sum, %n, %i, %a) %exit(%sum)
    let blk_body_id = vm.next_id();
    let blk_exit_id = vm.next_id();

    let blk_head_inst_branch2 = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: None,
        ops: RwLock::new(vec![blk_head_cond.clone(), blk_head_sum.clone(), blk_head_n.clone(), blk_head_i.clone(), blk_head_a.clone()]),
        v: Instruction_::Branch2{
            cond: 0,
            true_dest: Destination {
                target: blk_body_id,
                args: vec![DestArg::Normal(1), DestArg::Normal(2), DestArg::Normal(3), DestArg::Normal(4)]
            },
            false_dest: Destination {
                target: blk_exit_id,
                args: vec![DestArg::Normal(1)]
            },
            true_prob: 0.9f32
        }
    });

    blk_head.content = Some(BlockContent {
        args: vec![blk_head_sum.clone_value(), blk_head_n.clone_value(), blk_head_i.clone_value(), blk_head_a.clone_value()],
        exn_arg: None,
        body: vec![blk_head_inst_slt, blk_head_inst_branch2],
        keepalives: None
    });

    // %body(%sum, %n, %i, %a):
    let mut blk_body = Block::new(blk_body_id);
    vm.set_name(blk_body.as_entity(), Mu("blk_body"));
    let blk_body_sum = func_ver.new_ssa(vm.next_id(), int64.clone());
    vm.set_name(blk_body_sum.as_entity(), Mu("blk_body_sum"));
    let blk_body_n   = func_ver.new_ssa(vm.next_id(), int64.clone());
    vm.set_name(blk_body_n.as_entity(), Mu("blk_body_n"));
    let blk_body_i   = func_ver.new_ssa(vm.next_id(), int64.clone());
    vm.set_name(blk_body_i.as_entity(), Mu("blk_body_i"));
    let blk_body_a   = func_ver.new_ssa(vm.next_id(), ref_hybrid.clone());
    vm.set_name(blk_body_a.as_entity(), Mu("blk_body_a"));
    

    // %blk_body_iref_a = GETIREF <@my_hybrid> a
    let blk_body_iref_a = func_ver.new_ssa(vm.next_id(), iref_hybrid.clone());
    vm.set_name(blk_body_iref_a.as_entity(), Mu("blk_body_iref_a"));

    let blk_body_inst_getiref = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_body_iref_a.clone_value()]),
        ops: RwLock::new(vec![blk_body_a.clone()]),
        v: Instruction_::GetIRef(0)
    });

    // %blk_body_iref_var = GETVARPARTIREF <@my_hybrid> %blk_body_iref_a
    let blk_body_iref_var = func_ver.new_ssa(vm.next_id(), iref_int64.clone());
    vm.set_name(blk_body_iref_var.as_entity(), Mu("blk_body_iref_var"));

    let blk_body_inst_getvar = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_body_iref_var.clone_value()]),
        ops: RwLock::new(vec![blk_body_iref_a]),
        v: Instruction_::GetVarPartIRef {
            is_ptr: false,
            base: 0
        }
    });

    // %blk_body_iref_var_i = SHIFTIREF <@int64> %blk_body_iref_var %i
    let blk_body_iref_var_i = func_ver.new_ssa(vm.next_id(), iref_int64.clone());
    vm.set_name(blk_body_iref_var_i.as_entity(), Mu("blk_body_iref_var_i"));

    let blk_body_inst_shiftiref = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_body_iref_var_i.clone_value()]),
        ops: RwLock::new(vec![blk_body_iref_var.clone(), blk_body_i.clone()]),
        v: Instruction_::ShiftIRef {
            is_ptr: false,
            base: 0,
            offset: 1
        }
    });

    // %blk_body_ele = LOAD <@int64> %blk_body_iref_var_i
    let blk_body_ele = func_ver.new_ssa(vm.next_id(), int64.clone());
    vm.set_name(blk_body_ele.as_entity(), Mu("blk_body_ele"));
    let blk_body_inst_load = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_body_ele.clone_value()]),
        ops: RwLock::new(vec![blk_body_iref_var_i.clone()]),
        v: Instruction_::Load {
            is_ptr: false,
            order: MemoryOrder::Relaxed,
            mem_loc: 0
        }
    });

    // %blk_body_sum2 = ADD <@int64> %blk_body_sum %blk_body_ele
    let blk_body_sum2 = func_ver.new_ssa(vm.next_id(), int64.clone());
    vm.set_name(blk_body_sum2.as_entity(), Mu("blk_body_sum2"));
    let blk_body_inst_add_sum = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_body_sum2.clone_value()]),
        ops: RwLock::new(vec![blk_body_sum.clone(), blk_body_ele.clone()]),
        v: Instruction_::BinOp(BinOp::Add, 0, 1)
    });

    // %blk_body_i2 = ADD <@int64> %blk_body_i @int64_1
    let blk_body_i2 = func_ver.new_ssa(vm.next_id(), int64.clone());
    vm.set_name(blk_body_i2.as_entity(), Mu("blk_body_i2"));
    let blk_body_inst_add_i = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_body_i2.clone_value()]),
        ops: RwLock::new(vec![blk_body_i.clone(), int64_1_local.clone()]),
        v: Instruction_::BinOp(BinOp::Add, 0, 1)
    });

    // BRANCH1 %head (%sum2, %n, %i2, %a)
    let blk_body_inst_branch = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: None,
        ops: RwLock::new(vec![blk_body_sum2.clone(), blk_body_n.clone(), blk_body_i2.clone(), blk_body_a.clone()]),
        v: Instruction_::Branch1(Destination{
            target: blk_head.id(),
            args: vec![DestArg::Normal(0), DestArg::Normal(1), DestArg::Normal(2), DestArg::Normal(3)]
        })
    });

    blk_body.content = Some(BlockContent{
        args: vec![blk_body_sum.clone_value(), blk_body_n.clone_value(), blk_body_i.clone_value(), blk_body_a.clone_value()],
        exn_arg: None,
        body: vec![
        blk_body_inst_getiref,
        blk_body_inst_getvar,
        blk_body_inst_shiftiref,
        blk_body_inst_load,
        blk_body_inst_add_sum,
        blk_body_inst_add_i,
        blk_body_inst_branch
        ],
        keepalives: None,
    });

    // %exit(%sum):
    let mut blk_exit = Block::new(blk_exit_id);
    vm.set_name(blk_exit.as_entity(), Mu("blk_exit"));

    let blk_exit_sum = func_ver.new_ssa(vm.next_id(), int64.clone());
    vm.set_name(blk_exit_sum.as_entity(), Mu("blk_exit_sum"));

    let blk_exit_exit = gen_ccall_exit(blk_exit_sum.clone(), &mut func_ver, &vm);

    // RET @int64_0
    let blk_exit_ret = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: None,
        ops: RwLock::new(vec![int64_0_local.clone()]),
        v: Instruction_::Return(vec![0])
    });

    blk_exit.content = Some(BlockContent{
        args: vec![blk_exit_sum.clone_value()],
        exn_arg: None,
        body: vec![blk_exit_exit, blk_exit_ret],
        keepalives: None
    });

    func_ver.define(FunctionContent{
        entry: blk_entry.id(),
        blocks: hashmap!{
            blk_entry.id() => blk_entry,
            blk_check.id() => blk_check,
            blk_head.id()  => blk_head,
            blk_body.id()  => blk_body,
            blk_exit.id()  => blk_exit
        }
    });

    vm.define_func_version(func_ver);

    vm
}