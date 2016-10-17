#![allow(dead_code)]
#![allow(non_upper_case_globals)]

pub mod inst_sel;

mod codegen;
pub use compiler::backend::x86_64::codegen::CodeGenerator;

mod asm_backend;
pub use compiler::backend::x86_64::asm_backend::ASMCodeGen;
pub use compiler::backend::x86_64::asm_backend::emit_code;
pub use compiler::backend::x86_64::asm_backend::emit_context;
#[cfg(feature = "aot")]
pub use compiler::backend::x86_64::asm_backend::spill_rewrite;

use ast::ptr::P;
use ast::ir::*;
use ast::types::*;
use compiler::backend::RegGroup;

use std::collections::HashMap;

macro_rules! GPR {
    ($id:expr, $name: expr) => {
        {
            P(Value {
                hdr: MuEntityHeader::named($id, $name.to_string()),
                ty: UINT64_TYPE.clone(),
                v: Value_::SSAVar($id)
            })
        }
    };
}

macro_rules! FPR {
    ($id:expr, $name: expr) => {
        {
            P(Value {
                hdr: MuEntityHeader::named($id, $name.to_string()),
                ty: DOUBLE_TYPE.clone(),
                v: Value_::SSAVar($id)
            })
        }
    };
}

// put into several segments to avoid 'recursion limit reached' error
lazy_static! {
    pub static ref RAX : P<Value> = GPR!(0, "rax");
    pub static ref RCX : P<Value> = GPR!(1, "rcx");
    pub static ref RDX : P<Value> = GPR!(2, "rdx");
    pub static ref RBX : P<Value> = GPR!(3, "rbx");
    pub static ref RSP : P<Value> = GPR!(4, "rsp");
    pub static ref RBP : P<Value> = GPR!(5, "rbp");
    pub static ref RSI : P<Value> = GPR!(6, "rsi");
    pub static ref RDI : P<Value> = GPR!(7, "rdi");
    pub static ref R8  : P<Value> = GPR!(8, "r8");
    pub static ref R9  : P<Value> = GPR!(9, "r9");
    pub static ref R10 : P<Value> = GPR!(10,"r10");
    pub static ref R11 : P<Value> = GPR!(11,"r11");
    pub static ref R12 : P<Value> = GPR!(12,"r12");
    pub static ref R13 : P<Value> = GPR!(13,"r13");
    pub static ref R14 : P<Value> = GPR!(14,"r14");
    pub static ref R15 : P<Value> = GPR!(15,"r15");
    
    pub static ref RIP : P<Value> = GPR!(32,"rip");
    
    pub static ref RETURN_GPRs : [P<Value>; 2] = [
        RAX.clone(),
        RDX.clone(),
    ];
    
    pub static ref ARGUMENT_GPRs : [P<Value>; 6] = [
        RDI.clone(),
        RSI.clone(),
        RDX.clone(),
        RCX.clone(),
        R8.clone(),
        R9.clone()
    ];
    
    pub static ref CALLEE_SAVED_GPRs : [P<Value>; 6] = [
        RBX.clone(),
        RBP.clone(),
        R12.clone(),
        R13.clone(),
        R14.clone(),
        R15.clone()
    ];

    pub static ref CALLER_SAVED_GPRs : [P<Value>; 9] = [
        RAX.clone(),
        RCX.clone(),
        RDX.clone(),
        RSI.clone(),
        RDI.clone(),
        R8.clone(),
        R9.clone(),
        R10.clone(),
        R11.clone()
    ];
    
    pub static ref ALL_GPRs : [P<Value>; 15] = [
        RAX.clone(),
        RCX.clone(),
        RDX.clone(),
        RBX.clone(),
        RSP.clone(),
//        RBP.clone(),
        RSI.clone(),
        RDI.clone(),
        R8.clone(),
        R9.clone(),
        R10.clone(),
        R11.clone(),
        R12.clone(),
        R13.clone(),
        R14.clone(),
        R15.clone()
    ];
}

lazy_static!{
    pub static ref XMM0  : P<Value> = FPR!(16,"xmm0");
    pub static ref XMM1  : P<Value> = FPR!(17,"xmm1");
    pub static ref XMM2  : P<Value> = FPR!(18,"xmm2");
    pub static ref XMM3  : P<Value> = FPR!(19,"xmm3");
    pub static ref XMM4  : P<Value> = FPR!(20,"xmm4");
    pub static ref XMM5  : P<Value> = FPR!(21,"xmm5");
    pub static ref XMM6  : P<Value> = FPR!(22,"xmm6");
    pub static ref XMM7  : P<Value> = FPR!(23,"xmm7");
    pub static ref XMM8  : P<Value> = FPR!(24,"xmm8");
    pub static ref XMM9  : P<Value> = FPR!(25,"xmm9");
    pub static ref XMM10 : P<Value> = FPR!(26,"xmm10");
    pub static ref XMM11 : P<Value> = FPR!(27,"xmm11");
    pub static ref XMM12 : P<Value> = FPR!(28,"xmm12");
    pub static ref XMM13 : P<Value> = FPR!(29,"xmm13");
    pub static ref XMM14 : P<Value> = FPR!(30,"xmm14");
    pub static ref XMM15 : P<Value> = FPR!(31,"xmm15"); 
    
    pub static ref RETURN_FPRs : [P<Value>; 2] = [
        XMM0.clone(),
        XMM1.clone()
    ];
    
    pub static ref ARGUMENT_FPRs : [P<Value>; 6] = [
        XMM2.clone(),
        XMM3.clone(),
        XMM4.clone(),
        XMM5.clone(),
        XMM6.clone(),
        XMM7.clone()
    ];
    
    pub static ref CALLEE_SAVED_FPRs : [P<Value>; 0] = [];

    pub static ref CALLER_SAVED_FPRs : [P<Value>; 16] = [
        XMM0.clone(),
        XMM1.clone(),
        XMM2.clone(),
        XMM3.clone(),
        XMM4.clone(),
        XMM5.clone(),
        XMM6.clone(),
        XMM7.clone(),
        XMM8.clone(),
        XMM9.clone(),
        XMM10.clone(),
        XMM11.clone(),
        XMM12.clone(),
        XMM13.clone(),
        XMM14.clone(),
        XMM15.clone(),
    ];
    
    pub static ref ALL_FPRs : [P<Value>; 16] = [
        XMM0.clone(),
        XMM1.clone(),
        XMM2.clone(),
        XMM3.clone(),
        XMM4.clone(),
        XMM5.clone(),
        XMM6.clone(),
        XMM7.clone(),
        XMM8.clone(),
        XMM9.clone(),
        XMM10.clone(),
        XMM11.clone(),
        XMM12.clone(),
        XMM13.clone(),
        XMM14.clone(),
        XMM15.clone(),
    ];
}

pub const GPR_COUNT : usize = 16;
pub const FPR_COUNT : usize = 16;

lazy_static! {
    pub static ref ALL_MACHINE_REGs : HashMap<MuID, P<Value>> = {
        let mut map = HashMap::new();
        map.insert(RAX.id(), RAX.clone());
        map.insert(RCX.id(), RCX.clone());
        map.insert(RDX.id(), RDX.clone());
        map.insert(RBX.id(), RBX.clone());
        map.insert(RSP.id(), RSP.clone());
        map.insert(RBP.id(), RBP.clone());
        map.insert(RSI.id(), RSI.clone());
        map.insert(RDI.id(), RDI.clone());
        map.insert(R8.id(), R8.clone());
        map.insert(R9.id(), R9.clone());
        map.insert(R10.id(), R10.clone());
        map.insert(R11.id(), R11.clone());
        map.insert(R12.id(), R12.clone());
        map.insert(R13.id(), R13.clone());
        map.insert(R14.id(), R14.clone());
        map.insert(R15.id(), R15.clone());
        map.insert(XMM0.id(), XMM0.clone());
        map.insert(XMM1.id(), XMM1.clone());
        map.insert(XMM2.id(), XMM2.clone());
        map.insert(XMM3.id(), XMM3.clone());
        map.insert(XMM4.id(), XMM4.clone());
        map.insert(XMM5.id(), XMM5.clone());
        map.insert(XMM6.id(), XMM6.clone());
        map.insert(XMM7.id(), XMM7.clone());
        map.insert(XMM8.id(), XMM8.clone());
        map.insert(XMM9.id(), XMM9.clone());
        map.insert(XMM10.id(), XMM10.clone());
        map.insert(XMM11.id(), XMM11.clone());
        map.insert(XMM12.id(), XMM12.clone());
        map.insert(XMM13.id(), XMM13.clone());
        map.insert(XMM14.id(), XMM14.clone());
        map.insert(XMM15.id(), XMM15.clone());
        map.insert(RIP.id(), RIP.clone());
        
        map
    };
    
    // put callee saved regs first
    pub static ref ALL_USABLE_MACHINE_REGs : Vec<P<Value>> = vec![
        RBX.clone(),
        R12.clone(),
        R13.clone(),
        R14.clone(),
        R15.clone(),
            
        RAX.clone(),
        RCX.clone(),
        RDX.clone(),
        RSI.clone(),
        RDI.clone(),
        R8.clone(),
        R9.clone(),
        R10.clone(),
        R11.clone(),
        XMM0.clone(),
        XMM1.clone(),
        XMM2.clone(),
        XMM3.clone(),
        XMM4.clone(),
        XMM5.clone(),
        XMM6.clone(),
        XMM7.clone(),
        XMM8.clone(),
        XMM9.clone(),
        XMM10.clone(),
        XMM11.clone(),
        XMM12.clone(),
        XMM13.clone(),
        XMM14.clone(),
        XMM15.clone()
    ];
}

pub fn init_machine_regs_for_func (func_context: &mut FunctionContext) {
    for reg in ALL_MACHINE_REGs.values() {
        let reg_id = reg.extract_ssa_id().unwrap();
        let entry = SSAVarEntry::new(reg.clone());
        
        func_context.values.insert(reg_id, entry);
    }
}

pub fn number_of_regs_in_group(group: RegGroup) -> usize {
    match group {
        RegGroup::GPR => ALL_GPRs.len(),
        RegGroup::FPR => ALL_FPRs.len()
    }
}

pub fn number_of_all_regs() -> usize {
    ALL_MACHINE_REGs.len()
}

pub fn all_regs() -> &'static HashMap<MuID, P<Value>> {
    &ALL_MACHINE_REGs
}

pub fn all_usable_regs() -> &'static Vec<P<Value>> {
    &ALL_USABLE_MACHINE_REGs
}

pub fn pick_group_for_reg(reg_id: MuID) -> RegGroup {
    let reg = all_regs().get(&reg_id).unwrap();
    if reg.is_int_reg() {
        RegGroup::GPR
    } else if reg.is_fp_reg() {
        RegGroup::FPR
    } else {
        panic!("expect a machine reg to be either a GPR or a FPR: {}", reg)
    }
}

pub fn is_callee_saved(reg_id: MuID) -> bool {
    for reg in CALLEE_SAVED_GPRs.iter() {
        if reg_id == reg.extract_ssa_id().unwrap() {
            return true;
        }
    }
    
    false 
}

pub fn is_valid_x86_imm(op: &P<Value>) -> bool {
    use std::u32;
    match op.v {
        Value_::Constant(Constant::Int(val)) if val <= u32::MAX as u64 => {
            true
        },
        _ => false
    }
}
