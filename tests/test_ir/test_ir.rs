extern crate mu;

use self::mu::ast::types::*;
use self::mu::ast::ir::*;
use self::mu::ast::inst::*;
use self::mu::ast::op::*;
use self::mu::vm::*;

use std::sync::RwLock;
use std::collections::HashMap;

#[test]
#[allow(unused_variables)]
fn test_factorial() {
    let vm = factorial();
}

#[test]
#[allow(unused_variables)]
fn test_sum() {
    let vm = sum();
}

#[test]
#[allow(unused_variables)]
fn test_global_access() {
    let vm = global_access();
}

pub fn sum() -> VM {
    let vm = VM::new();

    // .typedef @int_64 = int<64>
    let type_def_int64 = vm.declare_type(vm.next_id(), MuType_::int(64));
    vm.set_name(type_def_int64.as_entity(), "int_64".to_string());
    let type_def_int1  = vm.declare_type(vm.next_id(), MuType_::int(1));
    vm.set_name(type_def_int1.as_entity(), "int_1".to_string());

    // .const @int_64_0 <@int_64> = 0
    // .const @int_64_1 <@int_64> = 1
    let const_def_int64_0 = vm.declare_const(vm.next_id(), type_def_int64.clone(), Constant::Int(0));
    vm.set_name(const_def_int64_0.as_entity(), "int64_0".to_string());
    let const_def_int64_1 = vm.declare_const(vm.next_id(), type_def_int64.clone(), Constant::Int(1));
    vm.set_name(const_def_int64_1.as_entity(), "int64_1".to_string());

    // .funcsig @sum_sig = (@int_64) -> (@int_64)
    let sum_sig = vm.declare_func_sig(vm.next_id(), vec![type_def_int64.clone()], vec![type_def_int64.clone()]);
    vm.set_name(sum_sig.as_entity(), "sum_sig".to_string());

    // .funcdecl @sum <@sum_sig>
    let func = MuFunction::new(vm.next_id(), sum_sig.clone());
    vm.set_name(func.as_entity(), "sum".to_string());
    let func_id = func.id();
    vm.declare_func(func);

    // .funcdef @sum VERSION @sum_v1 <@sum_sig> 
    let mut func_ver = MuFunctionVersion::new(vm.next_id(), func_id, sum_sig.clone());
    vm.set_name(func_ver.as_entity(), "sum_v1".to_string());

    // %entry(<@int_64> %n):
    let mut blk_entry = Block::new(vm.next_id());
    vm.set_name(blk_entry.as_entity(), "entry".to_string());
    
    let blk_entry_n = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_n.as_entity(), "blk_entry_n".to_string());
    let const_def_int64_0_local = func_ver.new_constant(vm.next_id(), const_def_int64_0.clone()); // FIXME: why we need a local version?
    let const_def_int64_1_local = func_ver.new_constant(vm.next_id(), const_def_int64_1.clone());

    // BRANCH %head
    let mut blk_head = Block::new(vm.next_id());
    vm.set_name(blk_head.as_entity(), "head".to_string());
    let blk_entry_term = func_ver.new_inst(vm.next_id(), Instruction {
        value: None,
        ops: RwLock::new(vec![blk_entry_n.clone(), const_def_int64_0_local.clone(), const_def_int64_0_local.clone()]),
        v: Instruction_::Branch1(Destination{
            target: blk_head.id(),
            args: vec![DestArg::Normal(0), DestArg::Normal(1), DestArg::Normal(2)]
        })
    });

    let blk_entry_content = BlockContent {
        args: vec![blk_entry_n.clone_value()],
        body: vec![blk_entry_term],
        keepalives: None
    };
    blk_entry.content = Some(blk_entry_content);

    // %head(<@int_64> %n, <@int_64> %s, <@int_64> %i):

    let blk_head_n = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_head_n.as_entity(), "blk_head_n".to_string());
    let blk_head_s = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_head_s.as_entity(), "blk_head_s".to_string());
    let blk_head_i = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_head_i.as_entity(), "blk_head_i".to_string());

    // %s2 = ADD %s %i
    let blk_head_s2 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_head_s2.as_entity(), "blk_head_s2".to_string());
    let blk_head_inst0 = func_ver.new_inst(vm.next_id(), Instruction {
        value: Some(vec![blk_head_s2.clone_value()]),
        ops: RwLock::new(vec![blk_head_s.clone(), blk_head_i.clone()]),
        v: Instruction_::BinOp(BinOp::Add, 0, 1)
    });

    // %i2 = ADD %i 1
    let blk_head_i2 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_head_i2.as_entity(), "blk_head_i2".to_string());
    let blk_head_inst1 = func_ver.new_inst(vm.next_id(), Instruction {
        value: Some(vec![blk_head_i2.clone_value()]),
        ops: RwLock::new(vec![blk_head_i.clone(), const_def_int64_1_local.clone()]),
        v: Instruction_::BinOp(BinOp::Add, 0, 1)
    });

    // %cond = UGT %i %n
    let blk_head_cond = func_ver.new_ssa(vm.next_id(), type_def_int1.clone());
    vm.set_name(blk_head_cond.as_entity(), "blk_head_cond".to_string());
    let blk_head_inst2 = func_ver.new_inst(vm.next_id(), Instruction {
        value: Some(vec![blk_head_cond.clone_value()]),
        ops: RwLock::new(vec![blk_head_i.clone(), blk_head_n.clone()]),
        v: Instruction_::CmpOp(CmpOp::UGT, 0, 1)
    });

    // BRANCH2 %cond %ret(%s2) %head(%n %s2 %i2)
    let mut blk_ret = Block::new(vm.next_id());
    vm.set_name(blk_ret.as_entity(), "ret".to_string());    
    let blk_head_term = func_ver.new_inst(vm.next_id(), Instruction{
        value: None,
        ops: RwLock::new(vec![blk_head_cond.clone(), blk_head_n.clone(), blk_head_s2.clone(), blk_head_i2.clone()]),
        v: Instruction_::Branch2 {
            cond: 0,
            true_dest: Destination {
                target: blk_ret.id(),
                args: vec![DestArg::Normal(2)]
            },
            false_dest: Destination {
                target: blk_head.id(),
                args: vec![DestArg::Normal(1), DestArg::Normal(2), DestArg::Normal(3)]
            },
            true_prob: 0.6f32
        }
    });

    let blk_head_content = BlockContent {
        args: vec![blk_head_n.clone_value(), blk_head_s.clone_value(), blk_head_i.clone_value()],
        body: vec![blk_head_inst0, blk_head_inst1, blk_head_inst2, blk_head_term],
        keepalives: None
    };
    blk_head.content = Some(blk_head_content);

    // %ret(<@int_64> %s):
    let blk_ret_s = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_ret_s.as_entity(), "blk_ret_s".to_string());

    // RET %s
    let blk_ret_term = func_ver.new_inst(vm.next_id(), Instruction{
        value: None,
        ops: RwLock::new(vec![blk_ret_s.clone()]),
        v: Instruction_::Return(vec![0])
    });

    let blk_ret_content = BlockContent {
        args: vec![blk_ret_s.clone_value()],
        body: vec![blk_ret_term],
        keepalives: None
    };
    blk_ret.content = Some(blk_ret_content);

    // wrap into a function
    func_ver.define(FunctionContent{
            entry: blk_entry.id(),
            blocks: {
                let mut blocks = HashMap::new();
                blocks.insert(blk_entry.id(), blk_entry);
                blocks.insert(blk_head.id(), blk_head);
                blocks.insert(blk_ret.id(), blk_ret);
                blocks
            }
    });

    vm.define_func_version(func_ver);

    vm
}

#[allow(unused_variables)]
pub fn factorial() -> VM {
    let vm = VM::new();

    // .typedef @int_64 = int<64>
    // .typedef @int_1 = int<1>
    // .typedef @float = float
    // .typedef @double = double
    // .typedef @void = void
    // .typedef @int_8 = int<8>
    // .typedef @int_32 = int<32>
    let type_def_int64 = vm.declare_type(vm.next_id(), MuType_::int(64));
    vm.set_name(type_def_int64.as_entity(), "int_64".to_string());
    let type_def_int1  = vm.declare_type(vm.next_id(), MuType_::int(1));
    vm.set_name(type_def_int1.as_entity(), "int_1".to_string());
    let type_def_float = vm.declare_type(vm.next_id(), MuType_::float());
    vm.set_name(type_def_float.as_entity(), "float".to_string());
    let type_def_double = vm.declare_type(vm.next_id(), MuType_::double());
    vm.set_name(type_def_double.as_entity(), "double".to_string());
    let type_def_void  = vm.declare_type(vm.next_id(), MuType_::void());
    vm.set_name(type_def_void.as_entity(), "void".to_string());
    let type_def_int8  = vm.declare_type(vm.next_id(), MuType_::int(8));
    vm.set_name(type_def_int8.as_entity(), "int8".to_string());
    let type_def_int32 = vm.declare_type(vm.next_id(), MuType_::int(32));
    vm.set_name(type_def_int32.as_entity(), "int32".to_string());

    // .const @int_64_1 <@int_64> = 1
    let const_def_int64_1 = vm.declare_const(vm.next_id(), type_def_int64.clone(), Constant::Int(1));
    vm.set_name(const_def_int64_1.as_entity(), "int64_1".to_string());

    // .funcsig @fac_sig = (@int_64) -> (@int_64)
    let fac_sig = vm.declare_func_sig(vm.next_id(), vec![type_def_int64.clone()], vec![type_def_int64.clone()]);
    vm.set_name(fac_sig.as_entity(), "fac_sig".to_string());
    let type_def_funcref_fac = vm.declare_type(vm.next_id(), MuType_::funcref(fac_sig.clone()));
    vm.set_name(type_def_funcref_fac.as_entity(), "fac_sig".to_string());

    // .funcdecl @fac <@fac_sig>
    let func = MuFunction::new(vm.next_id(), fac_sig.clone());
    vm.set_name(func.as_entity(), "fac".to_string());
    let func_id = func.id();
    vm.declare_func(func);

    // .funcdef @fac VERSION @fac_v1 <@fac_sig>
    let const_func_fac = vm.declare_const(vm.next_id(), type_def_funcref_fac, Constant::FuncRef(func_id));
    let mut func_ver = MuFunctionVersion::new(vm.next_id(), func_id, fac_sig.clone());

    // %blk_0(<@int_64> %n_3):
    let mut blk_0 = Block::new(vm.next_id());
    vm.set_name(blk_0.as_entity(), "blk_0".to_string());
    let blk_0_n_3 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_0_n_3.as_entity(), "blk_0_n_3".to_string());
    let const_def_int64_1_local = func_ver.new_constant(vm.next_id(), const_def_int64_1.clone());

    //   %v48 = EQ <@int_64> %n_3 @int_64_1
    let blk_0_v48 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_0_v48.as_entity(), "blk_0_v48".to_string());
    let blk_0_inst0 = func_ver.new_inst(vm.next_id(), Instruction {
            value: Some(vec![blk_0_v48.clone_value()]),
            ops: RwLock::new(vec![blk_0_n_3.clone(), const_def_int64_1_local.clone()]),
            v: Instruction_::CmpOp(CmpOp::EQ, 0, 1)
    });

    //   BRANCH2 %v48 %blk_2(@int_64_1) %blk_1(%n_3)
    let mut blk_1 = Block::new(vm.next_id());
    vm.set_name(blk_1.as_entity(), "blk_1".to_string());    
    let mut blk_2 = Block::new(vm.next_id());
    vm.set_name(blk_2.as_entity(), "blk_2".to_string());
    let blk_0_term = func_ver.new_inst(vm.next_id(), Instruction{
        value: None,
        ops: RwLock::new(vec![blk_0_v48.clone(), const_def_int64_1_local.clone(), blk_0_n_3.clone()]),
        v: Instruction_::Branch2 {
            cond: 0,
            true_dest: Destination {
                target: blk_2.id(),
                args: vec![DestArg::Normal(1)]
            },
            false_dest: Destination {
                target: blk_1.id(),
                args: vec![DestArg::Normal(2)]
            },
            true_prob: 0.3f32
        }
    });

    let blk_0_content = BlockContent {
        args: vec![blk_0_n_3.clone_value()],
        body: vec![blk_0_inst0, blk_0_term],
        keepalives: None
    };
    blk_0.content = Some(blk_0_content);

    // %blk_2(<@int_64> %v53):
    let blk_2_v53 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_2_v53.as_entity(), "blk_2_v53".to_string());

    //   RET %v53
    let blk_2_term = func_ver.new_inst(vm.next_id(), Instruction{
        value: None,
        ops: RwLock::new(vec![blk_2_v53.clone()]),
        v: Instruction_::Return(vec![0])
    });

    let blk_2_content = BlockContent {
        args: vec![blk_2_v53.clone_value()],
        body: vec![blk_2_term],
        keepalives: None
    };
    blk_2.content = Some(blk_2_content);

    // %blk_1(<@int_64> %n_3):
    let blk_1_n_3 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_1_n_3.as_entity(), "blk_1_n_3".to_string());

    //   %v50 = SUB <@int_64> %n_3 @int_64_1
    let blk_1_v50 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_1_v50.as_entity(), "blk_1_v50".to_string());
    let blk_1_inst0 = func_ver.new_inst(vm.next_id(), Instruction{
        value: Some(vec![blk_1_v50.clone_value()]),
        ops: RwLock::new(vec![blk_1_n_3.clone(), const_def_int64_1_local.clone()]),
        v: Instruction_::BinOp(BinOp::Sub, 0, 1)
    });

    //   %v51 = CALL <@fac_sig> @fac (%v50)
    let blk_1_v51 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_1_v51.as_entity(), "blk_1_v51".to_string());
    let blk_1_fac = func_ver.new_constant(vm.next_id(), const_func_fac.clone());
    let blk_1_inst1 = func_ver.new_inst(vm.next_id(), Instruction{
        value: Some(vec![blk_1_v51.clone_value()]),
        ops: RwLock::new(vec![blk_1_fac, blk_1_v50.clone()]),
        v: Instruction_::ExprCall {
            data: CallData {
                func: 0,
                args: vec![1],
                convention: CallConvention::Mu
            },
            is_abort: true
        }
    });

    //   %v52 = MUL <@int_64> %n_3 %v51
    let blk_1_v52 = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_1_v52.as_entity(), "blk_1_v52".to_string());
    let blk_1_inst2 = func_ver.new_inst(vm.next_id(), Instruction{
        value: Some(vec![blk_1_v52.clone_value()]),
        ops: RwLock::new(vec![blk_1_n_3.clone(), blk_1_v51.clone()]),
        v: Instruction_::BinOp(BinOp::Mul, 0, 1)
    });

    // BRANCH blk_2 (%blk_1_v52)
    let blk_1_term = func_ver.new_inst(vm.next_id(), Instruction{
        value: None,
        ops: RwLock::new(vec![blk_1_v52.clone()]),
        v: Instruction_::Branch1(Destination {
                target: blk_2.id(),
                args: vec![DestArg::Normal(0)]
           })
    });

    let blk_1_content = BlockContent {
        args: vec![blk_1_n_3.clone_value()],
        body: vec![blk_1_inst0, blk_1_inst1, blk_1_inst2, blk_1_term],
        keepalives: None
    };
    blk_1.content = Some(blk_1_content);

    // wrap into a function
    func_ver.define(FunctionContent{
            entry: blk_0.id(),
            blocks: {
                let mut blocks = HashMap::new();
                blocks.insert(blk_0.id(), blk_0);
                blocks.insert(blk_1.id(), blk_1);
                blocks.insert(blk_2.id(), blk_2);
                blocks
            }
    });

    vm.define_func_version(func_ver);

    vm
}

#[allow(unused_variables)]
pub fn global_access() -> VM {
    let vm = VM::new();
    
    // .typedef @int64 = int<64>
    // .typedef @iref_int64 = iref<int<64>>
    let type_def_int64 = vm.declare_type(vm.next_id(), MuType_::int(64));
    vm.set_name(type_def_int64.as_entity(), "int64".to_string());
    let type_def_iref_int64 = vm.declare_type(vm.next_id(), MuType_::iref(type_def_int64.clone()));
    vm.set_name(type_def_iref_int64.as_entity(), "iref_int64".to_string());
    
    // .const @int_64_0 <@int_64> = 0
    // .const @int_64_1 <@int_64> = 1
    let const_def_int64_0 = vm.declare_const(vm.next_id(), type_def_int64.clone(), Constant::Int(0));
    vm.set_name(const_def_int64_0.as_entity(), "int64_0".to_string());
    let const_def_int64_1 = vm.declare_const(vm.next_id(), type_def_int64.clone(), Constant::Int(1));
    vm.set_name(const_def_int64_1.as_entity(), "int64_1".to_string());
    
    // .global @a <@int_64>
    let global_a = vm.declare_global(vm.next_id(), type_def_int64.clone());
    vm.set_name(global_a.as_entity(), "a".to_string());
    
    // .funcsig @global_access_sig = () -> ()
    let func_sig = vm.declare_func_sig(vm.next_id(), vec![type_def_int64.clone()], vec![]);
    vm.set_name(func_sig.as_entity(), "global_access_sig".to_string());

    // .funcdecl @global_access <@global_access_sig>
    let func = MuFunction::new(vm.next_id(), func_sig.clone());
    vm.set_name(func.as_entity(), "global_access".to_string());
    let func_id = func.id();
    vm.declare_func(func);
    
    // .funcdef @global_access VERSION @v1 <@global_access_sig>
    let mut func_ver = MuFunctionVersion::new(vm.next_id(), func_id, func_sig.clone());
    
    // %blk_0():
    let mut blk_0 = Block::new(vm.next_id());
    vm.set_name(blk_0.as_entity(), "blk_0".to_string());
    
    // STORE <@int_64> @a @int_64_1
    let blk_0_a = func_ver.new_global(vm.next_id(), global_a.clone());
    let blk_0_const_int64_1 = func_ver.new_constant(vm.next_id(), const_def_int64_1.clone());
    let blk_0_inst0 = func_ver.new_inst(vm.next_id(), Instruction{
        value: None,
        ops: RwLock::new(vec![blk_0_a.clone(), blk_0_const_int64_1.clone()]),
        v: Instruction_::Store{
            is_ptr: false,
            order: MemoryOrder::Relaxed,
            mem_loc: 0,
            value: 1
        }
    });
        
    // %x = LOAD <@int_64> @a
    let blk_0_x = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_0_x.as_entity(), "blk_0_x".to_string());
    let blk_0_inst1 = func_ver.new_inst(vm.next_id(), Instruction{
        value: Some(vec![blk_0_x.clone_value()]),
        ops: RwLock::new(vec![blk_0_a.clone()]),
        v: Instruction_::Load{
            is_ptr: false,
            order: MemoryOrder::Relaxed,
            mem_loc: 0
        }
    });
    
    let blk_0_term = func_ver.new_inst(vm.next_id(), Instruction{
        value: None,
        ops: RwLock::new(vec![blk_0_x.clone()]),
        v: Instruction_::Return(vec![0])
    });
    
    let blk_0_content = BlockContent {
        args: vec![],
        body: vec![blk_0_inst0, blk_0_inst1, blk_0_term],
        keepalives: None
    };
    blk_0.content = Some(blk_0_content);
    
    func_ver.define(FunctionContent{
        entry: blk_0.id(),
        blocks: {
            let mut ret = HashMap::new();
            ret.insert(blk_0.id(), blk_0);
            ret
        }
    });
    
    vm.define_func_version(func_ver);
    
    vm
}
