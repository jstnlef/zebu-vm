extern crate mu;

#[cfg(test)]
mod test_ir {
    use mu::ast::types::*;
    use mu::ast::ir::*;
    use mu::ast::ptr::*;
    
    #[test]
    #[allow(unused_variables)]
    fn factorial() {
        // .typedef @int_64 = int<64>
        // .typedef @int_1 = int<1>
        // .typedef @float = float
        // .typedef @double = double
        // .typedef @void = void
        // .typedef @int_8 = int<8>
        // .typedef @int_32 = int<32>
        let type_def_int64 = declare_type("int_64", P(MuType_::int(64)));
        let type_def_int1  = declare_type("int_1", P(MuType_::int(1)));
        let type_def_float = declare_type("float", P(MuType_::float()));
        let type_def_double = declare_type("double", P(MuType_::double()));
        let type_def_void  = declare_type("void", P(MuType_::void()));
        let type_def_int8  = declare_type("int8", P(MuType_::int(8)));
        let type_def_int32 = declare_type("int32", P(MuType_::int(32)));
        
        // .const @int_64_1 <@int_64> = 1
        let const_def_int64_1 = P(declare_const("int64_1", type_def_int64.clone(), Constant::Int(64, 1)));
        
        // .funcsig @fac_sig = (@int_64) -> (@int_64)
        let fac_sig = P(declare_func_sig("fac_sig", vec![type_def_int64.clone()], vec![type_def_int64.clone()]));
        
        // .funcdef @fac VERSION @fac_v1 <@fac_sig>
        let fac_func_ref = P(MuType_::funcref(fac_sig.clone()));
        
        // %blk_0(<@int_64> %n_3):
        let mut blk_0 = Block::new("blk_0");
        let blk_0_n_3 = P(Value::SSAVar(SSAVar{id: 0, tag: "n_3", ty: type_def_int64.clone()}));
        
        //   %v48 = EQ <@int_64> %n_3 @int_64_1
        let blk_0_v48 = P(Value::SSAVar(SSAVar{id: 1, tag: "v48", ty: type_def_int64.clone()}));
        let blk_0_v48_expr = Expression::CmpOp(
            CmpOp::EQ,
            blk_0_n_3.clone(),
            const_def_int64_1.clone()
        );
        let blk_0_inst0 = Instruction::Assign{left: vec![blk_0_v48.clone()], right: blk_0_v48_expr};
        
        //   BRANCH2 %v48 %blk_2(@int_64_1) %blk_1(%n_3)        
        let blk_0_term = Terminal::Branch2{
            cond: blk_0_v48.clone(), 
            true_dest: Destination {
                target: "blk_2", 
                args: vec![DestArg::Normal(const_def_int64_1.clone())]
            },
            false_dest: Destination {
                target: "blk_1", 
                args: vec![DestArg::Normal(blk_0_n_3.clone())]
            }
        };
        
        let blk_0_content = BlockContent {
            args: vec![blk_0_n_3.clone()], 
            body: vec![blk_0_inst0], 
            exit: blk_0_term, 
            keepalives: None
        }; 
        blk_0.set_content(blk_0_content);

        // %blk_2(<@int_64> %v53):
        let mut blk_2 = Block::new("blk_2");
        let blk_2_v53 = P(Value::SSAVar(SSAVar{id: 2, tag: "v53", ty: type_def_int64.clone()}));
        
        //   RET %v53
        let blk_2_term = Terminal::Return(vec![blk_2_v53.clone()]);
        
        let blk_2_content = BlockContent {
            args: vec![blk_2_v53.clone()], 
            body: vec![], 
            exit: blk_2_term, 
            keepalives: None
        };
        blk_2.set_content(blk_2_content);
        
        // %blk_1(<@int_64> %n_3):
        let mut blk_1 = Block::new("blk_1");
        let blk_1_n_3 = P(Value::SSAVar(SSAVar{id: 3, tag: "n_3", ty: type_def_int64.clone()}));
        
        //   %v50 = SUB <@int_64> %n_3 @int_64_1
        let blk_1_v50 = P(Value::SSAVar(SSAVar{id: 4, tag: "v50", ty: type_def_int64.clone()}));
        let blk_1_v50_expr = Expression::BinOp(
            BinOp::Sub,
            blk_1_n_3.clone(),
            const_def_int64_1.clone()
        );
        let blk_1_inst0 = Instruction::Assign{left: vec![blk_1_v50.clone()], right: blk_1_v50_expr};
        
        //   %v51 = CALL <@fac_sig> @fac (%v50)
        let blk_1_v51 = P(Value::SSAVar(SSAVar{id: 5, tag: "v51", ty: type_def_int64.clone()}));
        let blk_1_term = Terminal::Call {
            data: CallData {
                func: P(SSAVar{id: 6, tag: "fac", ty: fac_func_ref.clone()}),
                args: vec![blk_1_v50.clone()],
                convention: CallConvention::Mu
            },
            normal_dest: Destination {
                target: "blk_1_cont", 
                args: vec![DestArg::Normal(blk_1_n_3.clone()), DestArg::Freshbound(0)]
            },
            exn_dest: None
        };
        
        let blk_1_content = BlockContent {
            args: vec![blk_1_n_3.clone()], 
            body: vec![blk_1_inst0], 
            exit: blk_1_term, 
            keepalives: None
        };
        blk_1.set_content(blk_1_content);
        
        // %blk_1_cont(<@int_64> %n_3, <@int_64> %v51):
        let mut blk_1_cont = Block::new("blk_1_cont");
        let blk_1_cont_n_3 = P(Value::SSAVar(SSAVar{id: 7, tag: "n_3", ty: type_def_int64.clone()}));
        let blk_1_cont_v51 = P(Value::SSAVar(SSAVar{id: 8, tag: "v51", ty: type_def_int64.clone()}));
        
        //   %v52 = MUL <@int_64> %n_3 %v51
        let blk_1_cont_v52 = P(Value::SSAVar(SSAVar{id: 9, tag: "v52", ty: type_def_int64.clone()}));
        let blk_1_cont_v52_expr = Expression::BinOp(
            BinOp::Mul,
            blk_1_cont_n_3.clone(),
            blk_1_cont_v52.clone()
        );
        let blk_1_cont_inst0 = Instruction::Assign{left: vec![blk_1_cont_v52.clone()], right: blk_1_cont_v52_expr};
        
        let blk_1_cont_term = Terminal::Branch1 (
            Destination {
                target: "blk_2", 
                args: vec![DestArg::Normal(blk_1_cont_v52.clone())]
            }
        );
        
        let blk_1_cont_content = BlockContent {
            args: vec![blk_1_cont_n_3.clone(), blk_1_cont_v52.clone()], 
            body: vec![blk_1_cont_inst0],
            exit: blk_1_cont_term, 
            keepalives: None
        };
        blk_1_cont.set_content(blk_1_cont_content);
        
        // wrap into a function
        let func_fac = declare_func("fac", fac_sig.clone(), "blk_0", vec![
                ("blk_0", blk_0),
                ("blk_1", blk_1),
                ("blk_1_cont", blk_1_cont),
                ("blk_2", blk_2)
            ]
        );
    }
}