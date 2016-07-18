extern crate mu;

use self::mu::ast::types::*;
use self::mu::ast::ir::*;
use self::mu::ast::inst::*;
use self::mu::ast::ptr::*;
use self::mu::ast::op::*;
use self::mu::vm::context::*;

use std::cell::RefCell;
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

pub fn sum() -> VMContext {
    let vm = VMContext::new();

    // .typedef @int_64 = int<64>
    let type_def_int64 = vm.declare_type("int_64", P(MuType::int(64)));
    let type_def_int1  = vm.declare_type("int_1", P(MuType::int(1)));

    // .const @int_64_0 <@int_64> = 0
    // .const @int_64_1 <@int_64> = 1
    let const_def_int64_0 = vm.declare_const("int64_0", type_def_int64.clone(), Constant::Int(0));
    let const_def_int64_1 = vm.declare_const("int64_1", type_def_int64.clone(), Constant::Int(1));

    // .funcsig @sum_sig = (@int_64) -> (@int_64)
    let sum_sig = vm.declare_func_sig("sum_sig", vec![type_def_int64.clone()], vec![type_def_int64.clone()]);

    // .funcdecl @sum <@sum_sig>
    let func = MuFunction::new("sum", sum_sig.clone());
    vm.declare_func(func);

    // .funcdef @sum VERSION @sum_v1 <@sum_sig> 
    let mut func_ver = MuFunctionVersion::new("sum", "sum_v1", sum_sig.clone());

    // %entry(<@int_64> %n):
    let mut blk_entry = Block::new("entry");
    let blk_entry_n = func_ver.new_ssa("blk_entry_n", type_def_int64.clone());
    let const_def_int64_0_local = func_ver.new_constant(const_def_int64_0.clone()); // FIXME: why we need a local version?
    let const_def_int64_1_local = func_ver.new_constant(const_def_int64_1.clone());

    // BRANCH %head
    let blk_entry_term = func_ver.new_inst(Instruction {
        value: None,
        ops: RefCell::new(vec![blk_entry_n.clone(), const_def_int64_0_local.clone(), const_def_int64_0_local.clone()]),
        v: Instruction_::Branch1(Destination{
            target: "head",
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
    let mut blk_head = Block::new("head");
    let blk_head_n = func_ver.new_ssa("blk_head_n", type_def_int64.clone());
    let blk_head_s = func_ver.new_ssa("blk_head_s", type_def_int64.clone());
    let blk_head_i = func_ver.new_ssa("blk_head_i", type_def_int64.clone());

    // %s2 = ADD %s %i
    let blk_head_s2 = func_ver.new_ssa("blk_head_s2", type_def_int64.clone());
    let blk_head_inst0 = func_ver.new_inst(Instruction {
        value: Some(vec![blk_head_s2.clone_value()]),
        ops: RefCell::new(vec![blk_head_s.clone(), blk_head_i.clone()]),
        v: Instruction_::BinOp(BinOp::Add, 0, 1)
    });

    // %i2 = ADD %i 1
    let blk_head_i2 = func_ver.new_ssa("blk_head_i2", type_def_int64.clone());
    let blk_head_inst1 = func_ver.new_inst(Instruction {
        value: Some(vec![blk_head_i2.clone_value()]),
        ops: RefCell::new(vec![blk_head_i.clone(), const_def_int64_1_local.clone()]),
        v: Instruction_::BinOp(BinOp::Add, 0, 1)
    });

    // %cond = UGT %i %n
    let blk_head_cond = func_ver.new_ssa("blk_head_cond", type_def_int1.clone());
    let blk_head_inst2 = func_ver.new_inst(Instruction {
        value: Some(vec![blk_head_cond.clone_value()]),
        ops: RefCell::new(vec![blk_head_i.clone(), blk_head_n.clone()]),
        v: Instruction_::CmpOp(CmpOp::UGT, 0, 1)
    });

    // BRANCH2 %cond %ret(%s2) %head(%n %s2 %i2)
    let blk_head_term = func_ver.new_inst(Instruction{
        value: None,
        ops: RefCell::new(vec![blk_head_cond.clone(), blk_head_n.clone(), blk_head_s2.clone(), blk_head_i2.clone()]),
        v: Instruction_::Branch2 {
            cond: 0,
            true_dest: Destination {
                target: "ret",
                args: vec![DestArg::Normal(2)]
            },
            false_dest: Destination {
                target: "head",
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
    let mut blk_ret = Block::new("ret");
    let blk_ret_s = func_ver.new_ssa("blk_ret_s", type_def_int64.clone());

    // RET %s
    let blk_ret_term = func_ver.new_inst(Instruction{
        value: None,
        ops: RefCell::new(vec![blk_ret_s.clone()]),
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
            entry: "entry",
            blocks: {
                let mut blocks = HashMap::new();
                blocks.insert("entry", blk_entry);
                blocks.insert("head", blk_head);
                blocks.insert("ret", blk_ret);
                blocks
            }
    });

    vm.define_func_version(func_ver);

    vm
}

#[allow(unused_variables)]
pub fn factorial() -> VMContext {
    let vm = VMContext::new();

    // .typedef @int_64 = int<64>
    // .typedef @int_1 = int<1>
    // .typedef @float = float
    // .typedef @double = double
    // .typedef @void = void
    // .typedef @int_8 = int<8>
    // .typedef @int_32 = int<32>
    let type_def_int64 = vm.declare_type("int_64", P(MuType::int(64)));
    let type_def_int1  = vm.declare_type("int_1", P(MuType::int(1)));
    let type_def_float = vm.declare_type("float", P(MuType::float()));
    let type_def_double = vm.declare_type("double", P(MuType::double()));
    let type_def_void  = vm.declare_type("void", P(MuType::void()));
    let type_def_int8  = vm.declare_type("int8", P(MuType::int(8)));
    let type_def_int32 = vm.declare_type("int32", P(MuType::int(32)));

    // .const @int_64_1 <@int_64> = 1
    let const_def_int64_1 = vm.declare_const("int64_1", type_def_int64.clone(), Constant::Int(1));

    // .funcsig @fac_sig = (@int_64) -> (@int_64)
    let fac_sig = vm.declare_func_sig("fac_sig", vec![type_def_int64.clone()], vec![type_def_int64.clone()]);
    let type_def_funcref_fac = vm.declare_type("fac_sig", P(MuType::funcref(fac_sig.clone())));

    // .funcdecl @fac <@fac_sig>
    let func = MuFunction::new("fac", fac_sig.clone());
    vm.declare_func(func);

    // .funcdef @fac VERSION @fac_v1 <@fac_sig>
    let const_func_fac = vm.declare_const("fac", type_def_funcref_fac, Constant::FuncRef("fac"));
    let mut func_ver = MuFunctionVersion::new("fac", "fac_v1", fac_sig.clone());

    // %blk_0(<@int_64> %n_3):
    let mut blk_0 = Block::new("blk_0");
    let blk_0_n_3 = func_ver.new_ssa("blk_0_n_3", type_def_int64.clone());
    let const_def_int64_1_local = func_ver.new_constant(const_def_int64_1.clone());

    //   %v48 = EQ <@int_64> %n_3 @int_64_1
    let blk_0_v48 = func_ver.new_ssa("blk_0_v48", type_def_int64.clone());
    let blk_0_inst0 = func_ver.new_inst(Instruction {
            value: Some(vec![blk_0_v48.clone_value()]),
            ops: RefCell::new(vec![blk_0_n_3.clone(), const_def_int64_1_local.clone()]),
            v: Instruction_::CmpOp(CmpOp::EQ, 0, 1)
    });

    //   BRANCH2 %v48 %blk_2(@int_64_1) %blk_1(%n_3)
    let blk_0_term = func_ver.new_inst(Instruction{
        value: None,
        ops: RefCell::new(vec![blk_0_v48.clone(), const_def_int64_1_local.clone(), blk_0_n_3.clone()]),
        v: Instruction_::Branch2 {
            cond: 0,
            true_dest: Destination {
                target: "blk_2",
                args: vec![DestArg::Normal(1)]
            },
            false_dest: Destination {
                target: "blk_1",
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
    let mut blk_2 = Block::new("blk_2");
    let blk_2_v53 = func_ver.new_ssa("blk_2_v53", type_def_int64.clone());

    //   RET %v53
    let blk_2_term = func_ver.new_inst(Instruction{
        value: None,
        ops: RefCell::new(vec![blk_2_v53.clone()]),
        v: Instruction_::Return(vec![0])
    });

    let blk_2_content = BlockContent {
        args: vec![blk_2_v53.clone_value()],
        body: vec![blk_2_term],
        keepalives: None
    };
    blk_2.content = Some(blk_2_content);

    // %blk_1(<@int_64> %n_3):
    let mut blk_1 = Block::new("blk_1");
    let blk_1_n_3 = func_ver.new_ssa("blk_1_n_3", type_def_int64.clone());

    //   %v50 = SUB <@int_64> %n_3 @int_64_1
    let blk_1_v50 = func_ver.new_ssa("blk_1_v50", type_def_int64.clone());
    let blk_1_inst0 = func_ver.new_inst(Instruction{
        value: Some(vec![blk_1_v50.clone_value()]),
        ops: RefCell::new(vec![blk_1_n_3.clone(), const_def_int64_1_local.clone()]),
        v: Instruction_::BinOp(BinOp::Sub, 0, 1)
    });

    //   %v51 = CALL <@fac_sig> @fac (%v50)
    let blk_1_v51 = func_ver.new_ssa("blk_1_v51", type_def_int64.clone());
    let blk_1_fac = func_ver.new_constant(const_func_fac.clone());
    let blk_1_inst1 = func_ver.new_inst(Instruction{
        value: Some(vec![blk_1_v51.clone_value()]),
        ops: RefCell::new(vec![blk_1_fac, blk_1_v50.clone()]),
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
    let blk_1_v52 = func_ver.new_ssa("blk_1_v52", type_def_int64.clone());
    let blk_1_inst2 = func_ver.new_inst(Instruction{
        value: Some(vec![blk_1_v52.clone_value()]),
        ops: RefCell::new(vec![blk_1_n_3.clone(), blk_1_v51.clone()]),
        v: Instruction_::BinOp(BinOp::Mul, 0, 1)
    });

    // BRANCH blk_2 (%blk_1_v52)
    let blk_1_term = func_ver.new_inst(Instruction{
        value: None,
        ops: RefCell::new(vec![blk_1_v52.clone()]),
        v: Instruction_::Branch1(Destination {
                target: "blk_2",
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
            entry: "blk_0",
            blocks: {
                let mut blocks = HashMap::new();
                blocks.insert("blk_0", blk_0);
                blocks.insert("blk_1", blk_1);
                blocks.insert("blk_2", blk_2);
                blocks
            }
    });

    vm.define_func_version(func_ver);

    vm
}

#[allow(unused_variables)]
pub fn global_access() -> VMContext {
    let vm = VMContext::new();
    
    // .typedef @int64 = int<64>
    // .typedef @iref_int64 = iref<int<64>>
    let type_def_int64 = vm.declare_type("int64", P(MuType::int(64)));
    let type_def_iref_int64 = vm.declare_type("iref_int64", P(MuType::iref(type_def_int64.clone())));
    
    // .const @int_64_0 <@int_64> = 0
    // .const @int_64_1 <@int_64> = 1
    let const_def_int64_0 = vm.declare_const("int64_0", type_def_int64.clone(), Constant::Int(0));
    let const_def_int64_1 = vm.declare_const("int64_1", type_def_int64.clone(), Constant::Int(1));
    
    // .global @a <@int_64>
    let global_a = vm.declare_global("a", type_def_int64.clone());
    
    // .funcsig @global_access_sig = () -> ()
    let func_sig = vm.declare_func_sig("global_access_sig", vec![], vec![]);

    // .funcdecl @global_access <@global_access_sig>
    let func = MuFunction::new("global_access", func_sig.clone());
    vm.declare_func(func);
    
    // .funcdef @global_access VERSION @v1 <@global_access_sig>
    let mut func_ver = MuFunctionVersion::new("global_access", "v1", func_sig.clone());
    
    // %blk_0():
    let mut blk_0 = Block::new("blk_0");
    
    // %x = LOAD <@int_64> @a
    let blk_0_x = func_ver.new_ssa("blk_0_x", type_def_iref_int64.clone()).clone_value();
    let blk_0_a = func_ver.new_global(global_a.clone());
    let blk_0_inst0 = func_ver.new_inst(Instruction{
        value: Some(vec![blk_0_x]),
        ops: RefCell::new(vec![blk_0_a.clone()]),
        v: Instruction_::Load{
            is_ptr: false,
            order: MemoryOrder::SeqCst,
            mem_loc: 0
        }
    });
    
    // STORE <@int_64> @a @int_64_1
    let blk_0_const_int64_1 = func_ver.new_constant(const_def_int64_1.clone());
    let blk_0_inst1 = func_ver.new_inst(Instruction{
        value: None,
        ops: RefCell::new(vec![blk_0_a.clone(), blk_0_const_int64_1.clone()]),
        v: Instruction_::Store{
            is_ptr: false,
            order: MemoryOrder::SeqCst,
            mem_loc: 0,
            value: 1
        }
    });
    
    let blk_0_term = func_ver.new_inst(Instruction{
        value: None,
        ops: RefCell::new(vec![]),
        v: Instruction_::Return(vec![])
    });
    
    let blk_0_content = BlockContent {
        args: vec![],
        body: vec![blk_0_inst0, blk_0_inst1, blk_0_term],
        keepalives: None
    };
    blk_0.content = Some(blk_0_content);
    
    func_ver.define(FunctionContent{
        entry: "blk_0",
        blocks: {
            let mut ret = HashMap::new();
            ret.insert("blk_0", blk_0);
            ret
        }
    });
    
    vm.define_func_version(func_ver);
    
    vm
}