extern crate mu;
extern crate log;
extern crate simple_logger;
extern crate libloading;

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
    let blk_entry_spill1_funcref = func_ver.new_constant(vm.next_id(), const_func_spill1.clone());
    // args
    let blk_entry_t1 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_t1.as_entity(), Mu("blk_entry_t1"));
    let blk_entry_t2 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_t1.as_entity(), Mu("blk_entry_t2"));
    let blk_entry_t3 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_t1.as_entity(), Mu("blk_entry_t3"));
    let blk_entry_t4 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_t1.as_entity(), Mu("blk_entry_t4"));
    let blk_entry_t5 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_t1.as_entity(), Mu("blk_entry_t5"));
    let blk_entry_t6 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_t1.as_entity(), Mu("blk_entry_t6"));
    let blk_entry_t7 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_t1.as_entity(), Mu("blk_entry_t7"));
    let blk_entry_t8 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_t1.as_entity(), Mu("blk_entry_t8"));
    let blk_entry_t9 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_t1.as_entity(), Mu("blk_entry_t9"));
    let blk_entry_t10= func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_t1.as_entity(), Mu("blk_entry_t10"));
    
    // CALL spill1(%t1, %t2, ... t10)
    let blk_entry_call = func_ver.new_inst(vm.next_id(), Instruction{
        value: None,
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
    let blk_entry_add0 = func_ver.new_inst(vm.next_id(), Instruction {
        value: Some(vec![blk_entry_res0.clone_value()]),
        ops: RwLock::new(vec![blk_entry_t1.clone(), blk_entry_t2.clone()]),
        v: Instruction_::BinOp(BinOp::Add, 0, 1)
    });
    
    // %res1 = ADD %res0 %t3
    let blk_entry_res1 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_res1.as_entity(), Mu("blk_entry_res1"));
    let blk_entry_add1 = func_ver.new_inst(vm.next_id(), Instruction {
        value: Some(vec![blk_entry_res1.clone_value()]),
        ops: RwLock::new(vec![blk_entry_res0.clone(), blk_entry_t3.clone()]),
        v: Instruction_::BinOp(BinOp::Add, 0, 1)
    });
    
    // %res2 = ADD %res1 %t4
    let blk_entry_res2 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_res2.as_entity(), Mu("blk_entry_res2"));
    let blk_entry_add2 = func_ver.new_inst(vm.next_id(), Instruction {
        value: Some(vec![blk_entry_res2.clone_value()]),
        ops: RwLock::new(vec![blk_entry_res1.clone(), blk_entry_t4.clone()]),
        v: Instruction_::BinOp(BinOp::Add, 0, 1)
    });
    
    // %res3 = ADD %res2 %t5
    let blk_entry_res3 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_res3.as_entity(), Mu("blk_entry_res3"));
    let blk_entry_add3 = func_ver.new_inst(vm.next_id(), Instruction {
        value: Some(vec![blk_entry_res3.clone_value()]),
        ops: RwLock::new(vec![blk_entry_res2.clone(), blk_entry_t5.clone()]),
        v: Instruction_::BinOp(BinOp::Add, 0, 1)
    });
    
    // RET %res3
    let blk_entry_ret = func_ver.new_inst(vm.next_id(), Instruction{
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