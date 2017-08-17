use ast::ir::*;
use ast::ptr::*;
use ast::types::*;
use compiler::backend::RegGroup;
use compiler::backend::x86_64;

#[derive(Clone, Debug)]
pub enum CallConvResult {
    GPR(P<Value>),
    GPREX(P<Value>, P<Value>),
    FPR(P<Value>),
    STACK
}

pub mod mu {
    pub use super::c::*;
}

pub mod c {
    use super::*;

    pub fn compute_arguments(sig: &MuFuncSig) -> Vec<CallConvResult> {
        let mut ret = vec![];

        let mut gpr_arg_count = 0;
        let mut fpr_arg_count = 0;

        for ty in sig.arg_tys.iter() {
            let arg_reg_group = RegGroup::get_from_ty(ty);

            if arg_reg_group == RegGroup::GPR {
                if gpr_arg_count < x86_64::ARGUMENT_GPRS.len() {
                    let arg_gpr = {
                        let ref reg64 = x86_64::ARGUMENT_GPRS[gpr_arg_count];
                        let expected_len = ty.get_int_length().unwrap();
                        x86_64::get_alias_for_length(reg64.id(), expected_len)
                    };

                    ret.push(CallConvResult::GPR(arg_gpr));
                    gpr_arg_count += 1;
                } else {
                    // use stack to pass argument
                    ret.push(CallConvResult::STACK);
                }
            } else if arg_reg_group == RegGroup::GPREX {
                // need two regsiters for this, otherwise, we need to pass on stack
                if gpr_arg_count + 1 < x86_64::ARGUMENT_GPRS.len() {
                    let arg_gpr1 = x86_64::ARGUMENT_GPRS[gpr_arg_count].clone();
                    let arg_gpr2 = x86_64::ARGUMENT_GPRS[gpr_arg_count + 1].clone();

                    ret.push(CallConvResult::GPREX(arg_gpr1, arg_gpr2));
                    gpr_arg_count += 2;
                } else {
                    ret.push(CallConvResult::STACK);
                }
            } else if arg_reg_group == RegGroup::FPR {
                if fpr_arg_count < x86_64::ARGUMENT_FPRS.len() {
                    let arg_fpr = x86_64::ARGUMENT_FPRS[fpr_arg_count].clone();

                    ret.push(CallConvResult::FPR(arg_fpr));
                    fpr_arg_count += 1;
                } else {
                    ret.push(CallConvResult::STACK);
                }
            } else {
                // fp const, struct, etc
                unimplemented!();
            }
        }

        ret
    }

    pub fn compute_return_values(sig: &MuFuncSig) -> Vec<CallConvResult> {
        let mut ret = vec![];

        let mut gpr_ret_count = 0;
        let mut fpr_ret_count = 0;

        for ty in sig.ret_tys.iter() {
            if RegGroup::get_from_ty(ty) == RegGroup::GPR {
                if gpr_ret_count < x86_64::RETURN_GPRS.len() {
                    let ret_gpr = {
                        let ref reg64 = x86_64::RETURN_GPRS[gpr_ret_count];
                        let expected_len = ty.get_int_length().unwrap();
                        x86_64::get_alias_for_length(reg64.id(), expected_len)
                    };

                    ret.push(CallConvResult::GPR(ret_gpr));
                    gpr_ret_count += 1;
                } else {
                    // get return value by stack
                    ret.push(CallConvResult::STACK);
                }
            } else if RegGroup::get_from_ty(ty) == RegGroup::GPREX {
                if gpr_ret_count + 1 < x86_64::RETURN_GPRS.len() {
                    let ret_gpr1 = x86_64::RETURN_GPRS[gpr_ret_count].clone();
                    let ret_gpr2 = x86_64::RETURN_GPRS[gpr_ret_count + 1].clone();

                    ret.push(CallConvResult::GPREX(ret_gpr1, ret_gpr2));
                } else {
                    ret.push(CallConvResult::STACK);
                }
            } else if RegGroup::get_from_ty(ty) == RegGroup::FPR {
                // floating point register
                if fpr_ret_count < x86_64::RETURN_FPRS.len() {
                    let ref ret_fpr = x86_64::RETURN_FPRS[fpr_ret_count];

                    ret.push(CallConvResult::FPR(ret_fpr.clone()));
                    fpr_ret_count += 1;
                } else {
                    ret.push(CallConvResult::STACK);
                }
            } else {
                // other type of return alue
                unimplemented!()
            }
        }

        ret
    }
}