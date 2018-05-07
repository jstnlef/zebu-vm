use ast::ir::*;
use ast::ptr::*;
use ast::types::*;
use compiler::backend::BackendType;
use compiler::backend::RegGroup;
use compiler::backend::x86_64;
use utils::ByteSize;
use vm::VM;

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

pub mod swapstack {
    pub use super::c::compute_arguments;
    pub use super::c::compute_arguments as compute_return_values;
    pub use super::c::compute_stack_args;
    pub use super::c::compute_stack_args as compute_stack_retvals;
}

pub mod c {
    use super::*;

    /// computes arguments for the function signature,
    /// returns a vector of CallConvResult for each argument type
    pub fn compute_arguments(tys: &Vec<P<MuType>>) -> Vec<CallConvResult> {
        let mut ret = vec![];

        let mut gpr_arg_count = 0;
        let mut fpr_arg_count = 0;

        for ty in tys.iter() {
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

    pub fn compute_stack_args(tys: &Vec<P<MuType>>, vm: &VM) -> (ByteSize, Vec<ByteSize>) {
        let callconv = compute_arguments(tys);

        let mut stack_arg_tys = vec![];
        for i in 0..callconv.len() {
            let ref cc = callconv[i];
            match cc {
                &CallConvResult::STACK => stack_arg_tys.push(tys[i].clone()),
                _ => {}
            }
        }

        compute_stack_locations(&stack_arg_tys, vm)
    }

    /// computes the return values for the function signature,
    /// returns a vector of CallConvResult for each return type
    pub fn compute_return_values(tys: &Vec<P<MuType>>) -> Vec<CallConvResult> {
        let mut ret = vec![];

        let mut gpr_ret_count = 0;
        let mut fpr_ret_count = 0;

        for ty in tys.iter() {
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

    pub fn compute_stack_retvals(tys: &Vec<P<MuType>>, vm: &VM) -> (ByteSize, Vec<ByteSize>) {
        let callconv = compute_return_values(tys);

        let mut stack_ret_val_tys = vec![];
        for i in 0..callconv.len() {
            let ref cc = callconv[i];
            match cc {
                &CallConvResult::STACK => stack_ret_val_tys.push(tys[i].clone()),
                _ => {}
            }
        }

        compute_stack_locations(&stack_ret_val_tys, vm)
    }

    /// computes the area on the stack for a list of types that need to put on stack,
    /// returns a tuple of (size, offset for each values on stack)
    pub fn compute_stack_locations(stack_val_tys: &Vec<P<MuType>>, vm: &VM) -> (ByteSize, Vec<ByteSize>) {
        let (stack_arg_size, _, stack_arg_offsets) = BackendType::sequential_layout(stack_val_tys, vm);

        // "The end of the input argument area shall be aligned on a 16
        // (32, if __m256 is passed on stack) byte boundary." - x86 ABI
        // if we need to special align the args, we do it now
        // (then the args will be put to stack following their regular alignment)
        let mut stack_arg_size_with_padding = stack_arg_size;

        if stack_arg_size % 16 == 0 {
            // do not need to adjust rsp
        } else if stack_arg_size % 8 == 0 {
            // adjust rsp by -8
            stack_arg_size_with_padding += 8;
        } else {
            let rem = stack_arg_size % 16;
            let stack_arg_padding = 16 - rem;
            stack_arg_size_with_padding += stack_arg_padding;
        }

        (stack_arg_size_with_padding, stack_arg_offsets)
    }
}
