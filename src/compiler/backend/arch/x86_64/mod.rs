#![allow(dead_code)]
#![allow(non_upper_case_globals)]

pub mod inst_sel;

mod codegen;
pub use compiler::backend::x86_64::codegen::CodeGenerator;

mod asm_backend;
pub use compiler::backend::x86_64::asm_backend::ASMCodeGen;
pub use compiler::backend::x86_64::asm_backend::emit_code;
pub use compiler::backend::x86_64::asm_backend::emit_context;

use ast::ptr::P;
use ast::ir::*;
use ast::types::*;
use compiler::backend::RegGroup;

macro_rules! GPR {
    ($name: expr, $id: expr) => {
        P(Value {
            id: $id,
            name: Some($name),
            ty: GPR_TY.clone(),
            v: Value_::SSAVar($id)
        })
    };
}

macro_rules! FPR {
    ($name: expr, $id: expr) => {
        P(Value {
            id: $id,
            name: Some($name),
            ty: FPR_TY.clone(),
            v: Value_::SSAVar($id)
        })
    };
}

lazy_static! {
    pub static ref GPR_TY : P<MuType> = P(MuType::new(INTERNAL_ID_START + 0, MuType_::int(64)));
    pub static ref FPR_TY : P<MuType> = P(MuType::new(INTERNAL_ID_START + 1, MuType_::double()));
}

// put into several segments to avoid 'recursion limit reached' error
lazy_static! {
    pub static ref RAX : P<Value> = GPR!("rax", 0);
    pub static ref RCX : P<Value> = GPR!("rcx", 1);
    pub static ref RDX : P<Value> = GPR!("rdx", 2);
    pub static ref RBX : P<Value> = GPR!("rbx", 3);
    pub static ref RSP : P<Value> = GPR!("rsp", 4);
    pub static ref RBP : P<Value> = GPR!("rbp", 5);
    pub static ref RSI : P<Value> = GPR!("rsi", 6);
    pub static ref RDI : P<Value> = GPR!("rdi", 7);
    pub static ref R8  : P<Value> = GPR!("r8",  8);
    pub static ref R9  : P<Value> = GPR!("r9",  9);
    pub static ref R10 : P<Value> = GPR!("r10", 10);
    pub static ref R11 : P<Value> = GPR!("r11", 11);
    pub static ref R12 : P<Value> = GPR!("r12", 12);
    pub static ref R13 : P<Value> = GPR!("r13", 13);
    pub static ref R14 : P<Value> = GPR!("r14", 14);
    pub static ref R15 : P<Value> = GPR!("r15", 15);
    
    pub static ref RIP : P<Value> = GPR!("rip", 32);
    
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
    pub static ref XMM0  : P<Value> = FPR!("xmm0", 16);
    pub static ref XMM1  : P<Value> = FPR!("xmm1", 17);
    pub static ref XMM2  : P<Value> = FPR!("xmm2", 18);
    pub static ref XMM3  : P<Value> = FPR!("xmm3", 19);
    pub static ref XMM4  : P<Value> = FPR!("xmm4", 20);
    pub static ref XMM5  : P<Value> = FPR!("xmm5", 21);
    pub static ref XMM6  : P<Value> = FPR!("xmm6", 22);
    pub static ref XMM7  : P<Value> = FPR!("xmm7", 23);
    pub static ref XMM8  : P<Value> = FPR!("xmm8", 24);
    pub static ref XMM9  : P<Value> = FPR!("xmm9", 25);
    pub static ref XMM10 : P<Value> = FPR!("xmm10",26);
    pub static ref XMM11 : P<Value> = FPR!("xmm11",27);
    pub static ref XMM12 : P<Value> = FPR!("xmm12",28);
    pub static ref XMM13 : P<Value> = FPR!("xmm13",29);
    pub static ref XMM14 : P<Value> = FPR!("xmm14",30);
    pub static ref XMM15 : P<Value> = FPR!("xmm15",31); 
    
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
    pub static ref ALL_MACHINE_REGs : Vec<P<Value>> = vec![
        RAX.clone(),
        RCX.clone(),
        RDX.clone(),
        RBX.clone(),
        RSP.clone(),
        RBP.clone(),
        RSI.clone(),
        RDI.clone(),
        R8.clone(),
        R9.clone(),
        R10.clone(),
        R11.clone(),
        R12.clone(),
        R13.clone(),
        R14.clone(),
        R15.clone(),
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
        RIP.clone()
    ];
    
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
    use std::cell::Cell;
    
    for reg in ALL_MACHINE_REGs.iter() {
        let reg_id = reg.extract_ssa_id().unwrap();
        let entry = SSAVarEntry {
            id: reg_id,
            name: reg.name, 
            ty: reg.ty.clone(),
            use_count: Cell::new(0),
            expr: None
        };
        
        func_context.value_tags.insert(reg.name.unwrap(), reg_id);
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

pub fn all_regs() -> &'static Vec<P<Value>> {
    &ALL_MACHINE_REGs
}

pub fn all_usable_regs() -> &'static Vec<P<Value>> {
    &ALL_USABLE_MACHINE_REGs
}

pub fn pick_group_for_reg(reg_id: MuID) -> RegGroup {
    match reg_id {
        0...15  => RegGroup::GPR,
        16...31 => RegGroup::FPR,
        _ => panic!("expected a machine reg ID, got {}", reg_id)
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
        Value_::Constant(Constant::Int(val)) if val <= u32::MAX as usize => {
            true
        },
        _ => false
    }
}
