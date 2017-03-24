extern crate mu;
extern crate libloading;

use mu::testutil;
use mu::testutil::aot;
use mu::utils::LinkedHashMap;
use test_compiler::test_call::gen_ccall_exit;
use test_ir::test_ir::factorial;
use self::mu::compiler::*;
use self::mu::utils::vec_utils;
use self::mu::ast::ir::*;
use self::mu::ast::types::*;
use self::mu::ast::inst::*;
use self::mu::ast::op::*;
use self::mu::vm::VM;

use std::sync::Arc;
use std::sync::RwLock;

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
fn test_ir_liveness_fac() {
    VM::start_logging_trace();
    
    let vm = Arc::new(factorial());
    
    let compiler = Compiler::new(CompilerPolicy::new(vec![
            Box::new(passes::DefUse::new()),
            Box::new(passes::TreeGen::new()),
            Box::new(passes::ControlFlowAnalysis::new()),
            Box::new(passes::TraceGen::new()),
            Box::new(backend::inst_sel::InstructionSelection::new()),
    ]), &vm);
    
    let func_id = vm.id_of("fac");
    let funcs = vm.funcs().read().unwrap();
    let func = funcs.get(&func_id).unwrap().read().unwrap();
    let func_vers = vm.func_vers().read().unwrap();
    let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();
    
    compiler.compile(&mut func_ver);
    
    let cf_lock = vm.compiled_funcs().read().unwrap();
    let cf = cf_lock.get(&func_ver.id()).unwrap().read().unwrap();
    
    // block 0
    
    let block_0_livein = cf.mc().get_ir_block_livein("blk_0").unwrap();
    let blk_0_n_3 = vm.id_of("blk_0_n_3");;
    assert!(vec_utils::is_identical_to_str_ignore_order(block_0_livein, vec![blk_0_n_3]));
    
    let block_0_liveout = cf.mc().get_ir_block_liveout("blk_0").unwrap();
    assert!(vec_utils::is_identical_to_str_ignore_order(block_0_liveout, vec![blk_0_n_3]));
    
    // block 1
    
    let block_1_livein = cf.mc().get_ir_block_livein("blk_1").unwrap();
    let blk_1_n_3 = vm.id_of("blk_1_n_3");
    assert!(vec_utils::is_identical_to_str_ignore_order(block_1_livein, vec![blk_1_n_3]));
    
    let block_1_liveout = cf.mc().get_ir_block_liveout("blk_1").unwrap();
    let blk_1_v52 = vm.id_of("blk_1_v52");
    trace!("lhs: {:?}", block_1_liveout);
    trace!("rhs: {:?}", vec![blk_1_v52]);
    assert!(vec_utils::is_identical_to_str_ignore_order(block_1_liveout, vec![blk_1_v52]));
    
    // block 2
    
    let block_2_livein = cf.mc().get_ir_block_livein("blk_2").unwrap();
    let blk_2_v53 = vm.id_of("blk_2_v53");
    assert!(vec_utils::is_identical_to_str_ignore_order(block_2_livein, vec![blk_2_v53]));
    
    let block_2_liveout = cf.mc().get_ir_block_liveout("blk_2").unwrap();
    let expect : Vec<MuID> = vec![];
    assert!(vec_utils::is_identical_to_str_ignore_order(block_2_liveout, expect));
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

fn create_spill1() -> VM {
    let vm = VM::new();
    
    // .typedef @int_64 = int<64>
    let type_def_int64 = vm.declare_type(vm.next_id(), MuType_::int(64));
    vm.set_name(type_def_int64.as_entity(), "int_64".to_string());
    
    // .funcsig @spill1_sig = (@int_64 x 10) -> (@int_64)
    let spill1_sig = vm.declare_func_sig(vm.next_id(), vec![type_def_int64.clone()], vec![type_def_int64.clone(); 10]);
    vm.set_name(spill1_sig.as_entity(), Mu("spill1_sig"));
    
    // .typedef @funcref_spill1 = funcref<@spill1_sig>
    let type_def_funcref_spill1 = vm.declare_type(vm.next_id(), MuType_::funcref(spill1_sig.clone()));
    vm.set_name(type_def_funcref_spill1.as_entity(), Mu("funcref_spill1"));
    
    // .funcdecl @spill1 <@spill1_sig>
    let func = MuFunction::new(vm.next_id(), spill1_sig.clone());
    vm.set_name(func.as_entity(), Mu("spill1"));
    let func_id = func.id();
    vm.declare_func(func);
    
    // .funcdef @spill1 VERSION @spill1_v1 <@spill1_sig>
    let const_func_spill1 = vm.declare_const(vm.next_id(), type_def_funcref_spill1, Constant::FuncRef(func_id));    
    let mut func_ver = MuFunctionVersion::new(vm.next_id(), func_id, spill1_sig.clone());
    vm.set_name(func_ver.as_entity(), Mu("spill1_v1"));
    
    // %entry(<@int_64> %t1, t2, ... t10):
    let mut blk_entry = Block::new(vm.next_id());
    vm.set_name(blk_entry.as_entity(), Mu("entry"));
    
    // callee
    let blk_entry_spill1_funcref = func_ver.new_constant(const_func_spill1.clone());
    // args
    let blk_entry_t1 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_t1.as_entity(), Mu("blk_entry_t1"));
    let blk_entry_t2 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_t2.as_entity(), Mu("blk_entry_t2"));
    let blk_entry_t3 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_t3.as_entity(), Mu("blk_entry_t3"));
    let blk_entry_t4 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_t4.as_entity(), Mu("blk_entry_t4"));
    let blk_entry_t5 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_t5.as_entity(), Mu("blk_entry_t5"));
    let blk_entry_t6 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_t6.as_entity(), Mu("blk_entry_t6"));
    let blk_entry_t7 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_t7.as_entity(), Mu("blk_entry_t7"));
    let blk_entry_t8 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_t8.as_entity(), Mu("blk_entry_t8"));
    let blk_entry_t9 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_t9.as_entity(), Mu("blk_entry_t9"));
    let blk_entry_t10= func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_t10.as_entity(), Mu("blk_entry_t10"));
    
    // %x = CALL spill1(%t1, %t2, ... t10)
    let blk_entry_x = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    let blk_entry_call = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_entry_x.clone_value()]),
        ops: RwLock::new(vec![
                blk_entry_spill1_funcref,
                blk_entry_t1.clone(),
                blk_entry_t2.clone(),
                blk_entry_t3.clone(),
                blk_entry_t4.clone(),
                blk_entry_t5.clone(),
                blk_entry_t6.clone(),
                blk_entry_t7.clone(),
                blk_entry_t8.clone(),
                blk_entry_t9.clone(),
                blk_entry_t10.clone()
            ]),
        v: Instruction_::ExprCall {
            data: CallData {
                func: 0,
                args: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
                convention: CallConvention::Mu
            },
            is_abort: false
        }
    });
    
    // %res0 = ADD %t1 %t2
    let blk_entry_res0 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_res0.as_entity(), Mu("blk_entry_res0"));
    let blk_entry_add0 = func_ver.new_inst(Instruction {
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_entry_res0.clone_value()]),
        ops: RwLock::new(vec![blk_entry_t1.clone(), blk_entry_t2.clone()]),
        v: Instruction_::BinOp(BinOp::Add, 0, 1)
    });
    
    // %res1 = ADD %res0 %t3
    let blk_entry_res1 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_res1.as_entity(), Mu("blk_entry_res1"));
    let blk_entry_add1 = func_ver.new_inst(Instruction {
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_entry_res1.clone_value()]),
        ops: RwLock::new(vec![blk_entry_res0.clone(), blk_entry_t3.clone()]),
        v: Instruction_::BinOp(BinOp::Add, 0, 1)
    });
    
    // %res2 = ADD %res1 %t4
    let blk_entry_res2 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_res2.as_entity(), Mu("blk_entry_res2"));
    let blk_entry_add2 = func_ver.new_inst(Instruction {
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_entry_res2.clone_value()]),
        ops: RwLock::new(vec![blk_entry_res1.clone(), blk_entry_t4.clone()]),
        v: Instruction_::BinOp(BinOp::Add, 0, 1)
    });
    
    // %res3 = ADD %res2 %t5
    let blk_entry_res3 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_res3.as_entity(), Mu("blk_entry_res3"));
    let blk_entry_add3 = func_ver.new_inst(Instruction {
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_entry_res3.clone_value()]),
        ops: RwLock::new(vec![blk_entry_res2.clone(), blk_entry_t5.clone()]),
        v: Instruction_::BinOp(BinOp::Add, 0, 1)
    });
    
    // RET %res3
    let blk_entry_ret = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: None,
        ops: RwLock::new(vec![blk_entry_res3.clone()]),
        v: Instruction_::Return(vec![0])
    });
    
    blk_entry.content = Some(BlockContent{
        args: vec![
            blk_entry_t1.clone_value(),
            blk_entry_t2.clone_value(),
            blk_entry_t3.clone_value(),
            blk_entry_t4.clone_value(),
            blk_entry_t5.clone_value(),
            blk_entry_t6.clone_value(),
            blk_entry_t7.clone_value(),
            blk_entry_t8.clone_value(),
            blk_entry_t9.clone_value(),
            blk_entry_t10.clone_value()
        ],
        exn_arg: None,
        body: vec![
            blk_entry_call,
            blk_entry_add0,
            blk_entry_add1,
            blk_entry_add2,
            blk_entry_add3,
            blk_entry_ret
        ],
        keepalives: None
    });
    
    func_ver.define(FunctionContent::new(
        blk_entry.id(),
        {
            let mut blocks = LinkedHashMap::new();
            blocks.insert(blk_entry.id(), blk_entry);
            blocks
        }
    ));
    
    vm.define_func_version(func_ver);
    
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

fn create_simple_spill() -> VM {
    let vm = VM::new();

    // .typedef @int_64 = int<64>
    let type_def_int64 = vm.declare_type(vm.next_id(), MuType_::int(64));
    vm.set_name(type_def_int64.as_entity(), Mu("int_64"));

    // .const @int_64_1 <@int_64> = 1
    let const_def_int64_1 = vm.declare_const(vm.next_id(), type_def_int64.clone(), Constant::Int(1));
    vm.set_name(const_def_int64_1.as_entity(), "int64_1".to_string());

    // .funcsig @simple_spill_sig = () -> (@int_64)
    let simple_spill_sig = vm.declare_func_sig(vm.next_id(), vec![type_def_int64.clone()], vec![]);
    vm.set_name(simple_spill_sig.as_entity(), Mu("simple_spill_sig"));

    // .funcdecl @simple_spill <@simple_spill_sig>
    let func = MuFunction::new(vm.next_id(), simple_spill_sig.clone());
    vm.set_name(func.as_entity(), Mu("simple_spill"));
    let func_id = func.id();
    vm.declare_func(func);

    // .funcdef @simple_spill VERSION @simple_spill_v1 <@simple_spill_sig>
    let mut func_ver = MuFunctionVersion::new(vm.next_id(), func_id, simple_spill_sig.clone());
    vm.set_name(func_ver.as_entity(), Mu("simple_spill_v1"));

    // %entry():
    let mut blk_entry = Block::new(vm.next_id());
    vm.set_name(blk_entry.as_entity(), Mu("entry"));

    // BRANCH %start(1, 1, 1, 1, ..., 1) // 14 constant ONE
    let const_int64_1 = func_ver.new_constant(const_def_int64_1.clone());
    let blk_start_id = vm.next_id();
    let blk_entry_branch = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: None,
        ops: RwLock::new(vec![const_int64_1.clone(); 14]),
        v: Instruction_::Branch1(Destination{
            target: blk_start_id,
            args: vec![
                DestArg::Normal(0),
                DestArg::Normal(1),
                DestArg::Normal(2),
                DestArg::Normal(3),
                DestArg::Normal(4),
                DestArg::Normal(5),
                DestArg::Normal(6),
                DestArg::Normal(7),
                DestArg::Normal(8),
                DestArg::Normal(9),
                DestArg::Normal(10),
                DestArg::Normal(11),
                DestArg::Normal(12),
                DestArg::Normal(13),
            ]
        })
    });

    blk_entry.content = Some(BlockContent {
        args: vec![],
        exn_arg: None,
        body: vec![blk_entry_branch],
        keepalives: None
    });

    // %start(%t1, %t2, ..., %t14):
    let mut blk_start = Block::new(blk_start_id);
    vm.set_name(blk_start.as_entity(), Mu("start"));

    // args
    let blk_start_t1 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_start_t1.as_entity(), Mu("blk_start_t1"));
    let blk_start_t2 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_start_t2.as_entity(), Mu("blk_start_t2"));
    let blk_start_t3 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_start_t3.as_entity(), Mu("blk_start_t3"));
    let blk_start_t4 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_start_t4.as_entity(), Mu("blk_start_t4"));
    let blk_start_t5 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_start_t5.as_entity(), Mu("blk_start_t5"));
    let blk_start_t6 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_start_t6.as_entity(), Mu("blk_start_t6"));
    let blk_start_t7 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_start_t7.as_entity(), Mu("blk_start_t7"));
    let blk_start_t8 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_start_t8.as_entity(), Mu("blk_start_t8"));
    let blk_start_t9 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_start_t9.as_entity(), Mu("blk_start_t9"));
    let blk_start_t10= func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_start_t10.as_entity(), Mu("blk_start_t10"));
    let blk_start_t11= func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_start_t11.as_entity(), Mu("blk_start_t11"));
    let blk_start_t12= func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_start_t12.as_entity(), Mu("blk_start_t12"));
    let blk_start_t13= func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_start_t13.as_entity(), Mu("blk_start_t13"));
    let blk_start_t14= func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_start_t14.as_entity(), Mu("blk_start_t14"));

    // %res = ADD %t1 %t2
    let blk_start_res = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_start_res.as_entity(), Mu("blk_start_res"));
    let blk_start_add = func_ver.new_inst(Instruction {
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_start_res.clone_value()]),
        ops: RwLock::new(vec![blk_start_t1.clone(), blk_start_t2.clone()]),
        v: Instruction_::BinOp(BinOp::Add, 0, 1)
    });

    // BRANCH %ret (%res, %t1, %t2, ..., %t14)
    let blk_ret_id = vm.next_id();
    let blk_start_branch = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: None,
        ops: RwLock::new(vec![
            blk_start_res.clone(),
            blk_start_t1.clone(),
            blk_start_t2.clone(),
            blk_start_t3.clone(),
            blk_start_t4.clone(),
            blk_start_t5.clone(),
            blk_start_t6.clone(),
            blk_start_t7.clone(),
            blk_start_t8.clone(),
            blk_start_t9.clone(),
            blk_start_t10.clone(),
            blk_start_t11.clone(),
            blk_start_t12.clone(),
            blk_start_t13.clone(),
            blk_start_t14.clone(),
        ]),
        v: Instruction_::Branch1(Destination{
            target: blk_ret_id,
            args: vec![
                DestArg::Normal(0),
                DestArg::Normal(1),
                DestArg::Normal(2),
                DestArg::Normal(3),
                DestArg::Normal(4),
                DestArg::Normal(5),
                DestArg::Normal(6),
                DestArg::Normal(7),
                DestArg::Normal(8),
                DestArg::Normal(9),
                DestArg::Normal(10),
                DestArg::Normal(11),
                DestArg::Normal(12),
                DestArg::Normal(13),
                DestArg::Normal(14),
            ]
        })
    });

    blk_start.content = Some(BlockContent {
        args: vec![
            blk_start_t1.clone_value(),
            blk_start_t2.clone_value(),
            blk_start_t3.clone_value(),
            blk_start_t4.clone_value(),
            blk_start_t5.clone_value(),
            blk_start_t6.clone_value(),
            blk_start_t7.clone_value(),
            blk_start_t8.clone_value(),
            blk_start_t9.clone_value(),
            blk_start_t10.clone_value(),
            blk_start_t11.clone_value(),
            blk_start_t12.clone_value(),
            blk_start_t13.clone_value(),
            blk_start_t14.clone_value(),
        ],
        exn_arg: None,
        body: vec![blk_start_add, blk_start_branch],
        keepalives: None
    });

    // %ret(%res, %t1, %t2, ... %t14):
    let mut blk_ret = Block::new(blk_ret_id);
    vm.set_name(blk_ret.as_entity(), Mu("ret"));
    
    // args
    let blk_ret_res = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_ret_res.as_entity(), Mu("blk_ret_res"));
    let blk_ret_t1 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_ret_t1.as_entity(), Mu("blk_ret_t1"));
    let blk_ret_t2 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_ret_t2.as_entity(), Mu("blk_ret_t2"));
    let blk_ret_t3 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_ret_t3.as_entity(), Mu("blk_ret_t3"));
    let blk_ret_t4 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_ret_t4.as_entity(), Mu("blk_ret_t4"));
    let blk_ret_t5 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_ret_t5.as_entity(), Mu("blk_ret_t5"));
    let blk_ret_t6 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_ret_t6.as_entity(), Mu("blk_ret_t6"));
    let blk_ret_t7 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_ret_t7.as_entity(), Mu("blk_ret_t7"));
    let blk_ret_t8 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_ret_t8.as_entity(), Mu("blk_ret_t8"));
    let blk_ret_t9 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_ret_t9.as_entity(), Mu("blk_ret_t9"));
    let blk_ret_t10= func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_ret_t10.as_entity(), Mu("blk_ret_t10"));
    let blk_ret_t11= func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_ret_t11.as_entity(), Mu("blk_ret_t11"));
    let blk_ret_t12= func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_ret_t12.as_entity(), Mu("blk_ret_t12"));
    let blk_ret_t13= func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_ret_t13.as_entity(), Mu("blk_ret_t13"));
    let blk_ret_t14= func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_ret_t14.as_entity(), Mu("blk_ret_t14"));

    // RET %res
    let blk_ret_ret = func_ver.new_inst(Instruction {
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: None,
        ops: RwLock::new(vec![blk_ret_res.clone()]),
        v: Instruction_::Return(vec![0])
    });

    blk_ret.content = Some(BlockContent {
        args: vec![
            blk_ret_res.clone_value(),
            blk_ret_t1.clone_value(),
            blk_ret_t2.clone_value(),
            blk_ret_t3.clone_value(),
            blk_ret_t4.clone_value(),
            blk_ret_t5.clone_value(),
            blk_ret_t6.clone_value(),
            blk_ret_t7.clone_value(),
            blk_ret_t8.clone_value(),
            blk_ret_t9.clone_value(),
            blk_ret_t10.clone_value(),
            blk_ret_t11.clone_value(),
            blk_ret_t12.clone_value(),
            blk_ret_t13.clone_value(),
            blk_ret_t14.clone_value(),
        ],
        exn_arg: None,
        body: vec![blk_ret_ret],
        keepalives: None
    });

    func_ver.define(FunctionContent::new(
        blk_entry.id(),
        {
            let mut blocks = LinkedHashMap::new();
            blocks.insert(blk_entry.id(), blk_entry);
            blocks.insert(blk_start.id(), blk_start);
            blocks.insert(blk_ret.id(), blk_ret);
            blocks
        }
    ));

    vm.define_func_version(func_ver);

    vm
}

#[test]
#[cfg(target_arch = "x86_64")]
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

#[test]
#[cfg(target_arch = "x86_64")]
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

        assert!(get_number_of_moves(fv_id, &vm) == 1, "The function should not yield any mov instructions other than mov %rsp->%rbp (some possible coalescing failed)");
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

#[test]
#[cfg(target_arch = "x86_64")]
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

    funcsig! ((vm) sig = (int64, int64, int64, int64) -> ());
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
        assert_eq!(res, 136);
    }
}

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