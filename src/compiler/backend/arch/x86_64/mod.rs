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

macro_rules! GPR_ALIAS {
    ($alias: ident: ($id64: expr, $r64: ident) -> $r32: ident, $r16: ident, $r8: ident) => {
        lazy_static!{
            pub static ref $r64 : P<Value> = GPR!($id64,    stringify!($r64), UINT64_TYPE);
            pub static ref $r32 : P<Value> = GPR!($id64 +1, stringify!($r32), UINT32_TYPE);
            pub static ref $r16 : P<Value> = GPR!($id64 +2, stringify!($r16), UINT16_TYPE);
            pub static ref $r8  : P<Value> = GPR!($id64 +3, stringify!($r8) , UINT8_TYPE );

            pub static ref $alias : [P<Value>; 4] = [$r64.clone(), $r32.clone(), $r16.clone(), $r8.clone()];
        }
    };

    ($alias: ident: ($id64: expr, $r64: ident)) => {
        lazy_static!{
            pub static ref $r64 : P<Value> = GPR!($id64,    stringify!($r64), UINT64_TYPE);

            pub static ref $alias : [P<Value>; 4] = [$r64.clone(), $r64.clone(), $r64.clone(), $r64.clone()];
        }
    };
}

macro_rules! GPR {
    ($id:expr, $name: expr, $ty: ident) => {
        {
            P(Value {
                hdr: MuEntityHeader::named($id, $name.to_string()),
                ty: $ty.clone(),
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

GPR_ALIAS!(RAX_ALIAS: (0, RAX)  -> EAX, AX , AL);
GPR_ALIAS!(RCX_ALIAS: (4, RCX)  -> ECX, CX , CL);
GPR_ALIAS!(RDX_ALIAS: (8, RDX)  -> EDX, DX , DL);
GPR_ALIAS!(RBX_ALIAS: (12,RBX)  -> EBX, BX , BL);
GPR_ALIAS!(RSP_ALIAS: (16,RSP)  -> ESP, SP , SPL);
GPR_ALIAS!(RBP_ALIAS: (20,RBP)  -> EBP, BP , BPL);
GPR_ALIAS!(RSI_ALIAS: (24,RSI)  -> ESI, SI , SIL);
GPR_ALIAS!(RDI_ALIAS: (28,RDI)  -> EDI, DI , DIL);
GPR_ALIAS!(R8_ALIAS : (32,R8 )  -> R8D, R8W, R8L);
GPR_ALIAS!(R9_ALIAS : (36,R9 )  -> R9D, R9W, R9L);
GPR_ALIAS!(R10_ALIAS: (40,R10) -> R10D,R10W,R10L);
GPR_ALIAS!(R11_ALIAS: (44,R11) -> R11D,R11W,R11L);
GPR_ALIAS!(R12_ALIAS: (48,R12) -> R12D,R12W,R12L);
GPR_ALIAS!(R13_ALIAS: (52,R13) -> R13D,R13W,R13L);
GPR_ALIAS!(R14_ALIAS: (56,R14) -> R14D,R14W,R14L);
GPR_ALIAS!(R15_ALIAS: (60,R15) -> R15D,R15W,R15L);
GPR_ALIAS!(RIP_ALIAS: (64,RIP));

lazy_static! {
    pub static ref GPR_ALIAS_LOOKUP_TABLE : HashMap<MuID, Vec<P<Value>>> = {
        let mut ret = HashMap::new();

        ret.insert(RAX.id(), RAX_ALIAS.to_vec());
        ret.insert(RCX.id(), RCX_ALIAS.to_vec());
        ret.insert(RDX.id(), RDX_ALIAS.to_vec());
        ret.insert(RBX.id(), RBX_ALIAS.to_vec());
        ret.insert(RSP.id(), RSP_ALIAS.to_vec());
        ret.insert(RBP.id(), RBP_ALIAS.to_vec());
        ret.insert(RSI.id(), RSI_ALIAS.to_vec());
        ret.insert(RDI.id(), RDI_ALIAS.to_vec());
        ret.insert(R8.id() , R8_ALIAS.to_vec() );
        ret.insert(R9.id() , R9_ALIAS.to_vec() );
        ret.insert(R10.id(), R10_ALIAS.to_vec());
        ret.insert(R11.id(), R11_ALIAS.to_vec());
        ret.insert(R12.id(), R12_ALIAS.to_vec());
        ret.insert(R13.id(), R13_ALIAS.to_vec());
        ret.insert(R14.id(), R14_ALIAS.to_vec());
        ret.insert(R15.id(), R15_ALIAS.to_vec());
        ret.insert(RIP.id(), RIP_ALIAS.to_vec());

        ret
    };
}

pub fn get_gpr_alias(id: MuID, length: usize) -> P<Value> {
    let vec = match GPR_ALIAS_LOOKUP_TABLE.get(&id) {
        Some(vec) => vec,
        None => panic!("didnt find {} as GPR", id)
    };

    match length {
        64 => vec[0].clone(),
        32 => vec[1].clone(),
        16 => vec[2].clone(),
        8  => vec[3].clone(),
        _  => panic!("unexpected length: {}", length)
    }
}

lazy_static! {
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
    pub static ref XMM0  : P<Value> = FPR!(70,"xmm0");
    pub static ref XMM1  : P<Value> = FPR!(71,"xmm1");
    pub static ref XMM2  : P<Value> = FPR!(72,"xmm2");
    pub static ref XMM3  : P<Value> = FPR!(73,"xmm3");
    pub static ref XMM4  : P<Value> = FPR!(74,"xmm4");
    pub static ref XMM5  : P<Value> = FPR!(75,"xmm5");
    pub static ref XMM6  : P<Value> = FPR!(76,"xmm6");
    pub static ref XMM7  : P<Value> = FPR!(77,"xmm7");
    pub static ref XMM8  : P<Value> = FPR!(78,"xmm8");
    pub static ref XMM9  : P<Value> = FPR!(79,"xmm9");
    pub static ref XMM10 : P<Value> = FPR!(80,"xmm10");
    pub static ref XMM11 : P<Value> = FPR!(81,"xmm11");
    pub static ref XMM12 : P<Value> = FPR!(82,"xmm12");
    pub static ref XMM13 : P<Value> = FPR!(83,"xmm13");
    pub static ref XMM14 : P<Value> = FPR!(84,"xmm14");
    pub static ref XMM15 : P<Value> = FPR!(85,"xmm15");

    pub static ref RETURN_FPRs : [P<Value>; 2] = [
        XMM0.clone(),
        XMM1.clone()
    ];

    pub static ref ARGUMENT_FPRs : [P<Value>; 8] = [
        XMM0.clone(),
        XMM1.clone(),
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

    // put caller saved regs first (they imposes no overhead if there is no call instruction)
    pub static ref ALL_USABLE_MACHINE_REGs : Vec<P<Value>> = vec![
        RAX.clone(),
        RCX.clone(),
        RDX.clone(),
        RSI.clone(),
        RDI.clone(),
        R8.clone(),
        R9.clone(),
        R10.clone(),
        R11.clone(),

        RBX.clone(),
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
    use std::i32;
    match op.v {
        Value_::Constant(Constant::Int(val)) if val <= i32::MAX as u64 => {
            true
        },
        _ => false
    }
}