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

#[allow(unused_variables)]
pub fn factorial() -> VMContext {
    let mut vm = VMContext::new();
    
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
    
    // .funcdef @fac VERSION @fac_v1 <@fac_sig>
    let mut func = MuFunction::new("fac", fac_sig.clone());
    
    // %blk_0(<@int_64> %n_3):
    let mut blk_0 = Block::new("blk_0");
    let blk_0_n_3 = func.new_ssa(0, "blk_0_n_3", type_def_int64.clone());
    let const_def_int64_1_local = func.new_value(const_def_int64_1.clone());
    
    //   %v48 = EQ <@int_64> %n_3 @int_64_1
    let blk_0_v48 = func.new_ssa(1, "blk_0_v48", type_def_int64.clone());
    let blk_0_inst0 = TreeNode::new_inst(Instruction {
            value: Some(vec![blk_0_v48.clone()]),
            ops: RefCell::new(vec![blk_0_n_3.clone(), const_def_int64_1_local.clone()]),
            v: Instruction_::CmpOp(CmpOp::EQ, 0, 1)
    });    
    
    //   BRANCH2 %v48 %blk_2(@int_64_1) %blk_1(%n_3)
    let blk_0_term = TreeNode::new_inst(Instruction{
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
            true_prob: 0.5f32
        }
    });
    
    let blk_0_content = BlockContent {
        args: vec![blk_0_n_3.clone()], 
        body: vec![blk_0_inst0, blk_0_term], 
        keepalives: None
    }; 
    blk_0.content = Some(blk_0_content);

    // %blk_2(<@int_64> %v53):
    let mut blk_2 = Block::new("blk_2");
    let blk_2_v53 = func.new_ssa(2, "blk_2_v53", type_def_int64.clone());
    
    //   RET %v53
    let blk_2_term = TreeNode::new_inst(Instruction{
        value: None,
        ops: RefCell::new(vec![blk_2_v53.clone()]),
        v: Instruction_::Return(vec![0])
    });
    
    let blk_2_content = BlockContent {
        args: vec![blk_2_v53.clone()], 
        body: vec![blk_2_term], 
        keepalives: None
    };
    blk_2.content = Some(blk_2_content);
    
    // %blk_1(<@int_64> %n_3):
    let mut blk_1 = Block::new("blk_1");
    let blk_1_n_3 = func.new_ssa(3, "blk_1_n_3", type_def_int64.clone());
    
    //   %v50 = SUB <@int_64> %n_3 @int_64_1
    let blk_1_v50 = func.new_ssa(4, "blk_1_v50", type_def_int64.clone());
    let blk_1_inst0 = TreeNode::new_inst(Instruction{
        value: Some(vec![blk_1_v50.clone()]),
        ops: RefCell::new(vec![blk_1_n_3.clone(), const_def_int64_1_local.clone()]),
        v: Instruction_::BinOp(BinOp::Sub, 0, 1)
    });
    
    //   %v51 = CALL <@fac_sig> @fac (%v50)
    let blk_1_v51 = func.new_ssa(5, "blk_1_v51", type_def_int64.clone());
    let blk_1_inst1 = TreeNode::new_inst(Instruction{
        value: Some(vec![blk_1_v51.clone()]),
        ops: RefCell::new(vec![func.new_ssa(6, "blk_1_fac", P(MuType::funcref(fac_sig.clone()))), blk_1_v50.clone()]),
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
    let blk_1_v52 = func.new_ssa(7, "blk_1_v52", type_def_int64.clone());
    let blk_1_inst2 = TreeNode::new_inst(Instruction{
        value: Some(vec![blk_1_v52.clone()]),
        ops: RefCell::new(vec![blk_1_n_3.clone(), blk_1_v51.clone()]),
        v: Instruction_::BinOp(BinOp::Mul, 0, 1)
    });
    
    let blk_1_term = TreeNode::new_inst(Instruction{
        value: None,
        ops: RefCell::new(vec![blk_1_v52.clone()]),
        v: Instruction_::Branch1(Destination {
                target: "blk_2",
                args: vec![DestArg::Normal(0)]
           })
    });
    
    let blk_1_content = BlockContent {
        args: vec![blk_1_n_3.clone()], 
        body: vec![blk_1_inst0, blk_1_inst1, blk_1_inst2, blk_1_term],
        keepalives: None
    };
    blk_1.content = Some(blk_1_content);
    
    // wrap into a function
    func.define(FunctionContent{
            entry: "blk_0", 
            blocks: {
                let mut blocks = HashMap::new();
                blocks.insert("blk_0", blk_0);
                blocks.insert("blk_1", blk_1);
                blocks.insert("blk_2", blk_2);
                blocks
            }
    });
    
    vm.declare_func(func);
    
    vm
}