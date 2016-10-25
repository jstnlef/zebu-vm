extern crate mu;
extern crate log;
extern crate simple_logger;
extern crate libloading;

use aot;
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
use std::collections::HashMap;

#[test]
fn test_ir_liveness_fac() {
    simple_logger::init_with_level(log::LogLevel::Trace).ok();
    
    let vm = Arc::new(factorial());
    
    let compiler = Compiler::new(CompilerPolicy::new(vec![
            Box::new(passes::DefUse::new()),
            Box::new(passes::TreeGen::new()),
            Box::new(passes::ControlFlowAnalysis::new()),
            Box::new(passes::TraceGen::new()),
            Box::new(backend::inst_sel::InstructionSelection::new()),
    ]), vm.clone());
    
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
    simple_logger::init_with_level(log::LogLevel::Trace).ok();
    
    let vm = Arc::new(create_spill1());
    
    let compiler = Compiler::new(CompilerPolicy::default(), vm.clone());
    
    let func_id = vm.id_of("spill1");
    {
        let funcs = vm.funcs().read().unwrap();
        let func = funcs.get(&func_id).unwrap().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();
        let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();
        
        compiler.compile(&mut func_ver);
    }

    backend::emit_context(&vm);

    let dylib = aot::link_dylib(vec![Mu("spill1")], "libspill1.dylib");

    let lib = libloading::Library::new(dylib.as_os_str()).unwrap();
    unsafe {
        let simple_spill : libloading::Symbol<unsafe extern fn() -> u64> = match lib.get(b"spill1") {
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
    
    func_ver.define(FunctionContent {
        entry: blk_entry.id(),
        blocks: {
            let mut blocks = HashMap::new();
            blocks.insert(blk_entry.id(), blk_entry);
            blocks
        }
    });
    
    vm.define_func_version(func_ver);
    
    vm
}

#[test]
fn test_simple_spill() {
    simple_logger::init_with_level(log::LogLevel::Trace).ok();

    let vm = Arc::new(create_simple_spill());

    let compiler = Compiler::new(CompilerPolicy::default(), vm.clone());

    let func_id = vm.id_of("simple_spill");
    {
        let funcs = vm.funcs().read().unwrap();
        let func = funcs.get(&func_id).unwrap().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();
        let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();

        compiler.compile(&mut func_ver);
    }

    backend::emit_context(&vm);

    let dylib = aot::link_dylib(vec![Mu("simple_spill")], "libsimple_spill.dylib");

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

    func_ver.define(FunctionContent {
        entry: blk_entry.id(),
        blocks: {
            let mut blocks = HashMap::new();
            blocks.insert(blk_entry.id(), blk_entry);
            blocks.insert(blk_start.id(), blk_start);
            blocks.insert(blk_ret.id(), blk_ret);
            blocks
        }
    });

    vm.define_func_version(func_ver);

    vm
}