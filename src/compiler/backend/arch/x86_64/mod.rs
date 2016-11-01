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
use utils::ByteSize;

use std::collections::HashMap;

macro_rules! GPR8 {
    ($id:expr, $name: expr) => {
        {
            P(Value {
                hdr: MuEntityHeader::named($id, $name.to_string()),
                ty: UINT8_TYPE.clone(),
                v: Value_::SSAVar($id)
            })
        }
    };
}

macro_rules! GPR16 {
    ($id:expr, $name: expr) => {
        {
            P(Value {
                hdr: MuEntityHeader::named($id, $name.to_string()),
                ty: UINT16_TYPE.clone(),
                v: Value_::SSAVar($id)
            })
        }
    };
}

macro_rules! GPR32 {
    ($id:expr, $name: expr) => {
        {
            P(Value {
                hdr: MuEntityHeader::named($id, $name.to_string()),
                ty: UINT32_TYPE.clone(),
                v: Value_::SSAVar($id)
            })
        }
    };
}

macro_rules! GPR64 {
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

macro_rules! alias {
    ($r8: expr, $r16: expr, $r32: expr, $r64: expr) => {
        hashmap!{
            8  => $r8.clone(),
            16 => $r16.clone(),
            32 => $r32.clone(),
            64 => $r64.clone()
        }
    };
}

pub type RegisterAliasMap = HashMap<ByteSize, P<Value>>;

macro_rules! init_alias_regs {
    ($start_id: expr, $r8: ident, $r16: ident, $r32: ident, $r64: ident, $alias: ident) => {
        lazy_static!{
            pub static ref $r8 : P<Value> = GPR8! ($start_id,     stringify!($r8).to_lowercase());
            pub static ref $r16: P<Value> = GPR16!($start_id + 1, stringify!($r16).to_lowercase());
            pub static ref $r32: P<Value> = GPR32!($start_id + 2, stringify!($r32).to_lowercase());
            pub static ref $r64: P<Value> = GPR64!($start_id + 3, stringify!($r64).to_lowercase());

            pub static ref $alias : RegisterAliasMap = alias!($r8, $r16, $r32, $r64);
        }
    }
}

init_alias_regs!(0,  AL  , AX  , EAX,  RAX, RAX_ALIAS);
init_alias_regs!(4,  CL  , CX  , ECX,  RCX, RCX_ALIAS);
init_alias_regs!(8,  DL  , DX  , EDX,  RDX, RDX_ALIAS);
init_alias_regs!(12, BL  , BX  , EBX,  RBX, RBX_ALIAS);
init_alias_regs!(16, SPL , SP  , ESP,  RSP, RSP_ALIAS);
init_alias_regs!(20, BPL , BP  , EBP,  RBP, RBP_ALIAS);
init_alias_regs!(24, SIL , SI  , ESI,  RSI, RSI_ALIAS);
init_alias_regs!(28, DIL , DI  , EDI,  RDI, RDI_ALIAS);
init_alias_regs!(32, R8L , R8W , R8D,  R8,  R8_ALIAS );
init_alias_regs!(36, R9L , R9W , R9D,  R9,  R9_ALIAS );
init_alias_regs!(40, R10L, R10W, R10D, R10, R10_ALIAS);
init_alias_regs!(44, R11L, R11W, R11D, R11, R11_ALIAS);
init_alias_regs!(48, R12L, R12W, R12D, R12, R12_ALIAS);
init_alias_regs!(52, R13L, R13W, R13D, R13, R13_ALIAS);
init_alias_regs!(56, R14L, R14W, R14D, R14, R14_ALIAS);
init_alias_regs!(60, R15L, R15W, R15D, R15, R15_ALIAS);

lazy_static! {
    pub static ref RIP : P<Value> = GPR64!(65,"rip");

    pub static ref ALL_GPR_ALIAS_MAPs : Vec<RegisterAliasMap> = vec![
        RAX_ALIAS.clone(),
        RCX_ALIAS.clone(),
        RDX_ALIAS.clone(),
        RBX_ALIAS.clone(),
        RSP_ALIAS.clone(),
        RBP_ALIAS.clone(),
        RSI_ALIAS.clone(),
        RDI_ALIAS.clone(),
        R8_ALIAS.clone(),
        R9_ALIAS.clone(),
        R10_ALIAS.clone(),
        R11_ALIAS.clone(),
        R12_ALIAS.clone(),
        R13_ALIAS.clone(),
        R14_ALIAS.clone(),
        R15_ALIAS.clone(),
    ];
}

macro_rules! pick_regs_of_length {
    ($len: expr) => {
        {
            let mut ret = vec![];
            for map in ALL_GPR_ALIAS_MAPs.iter() {
                match map.get(&$len) {
                    Some(reg) => ret.push(reg.clone()),
                    None => {}
                }
            }
            ret
        }
    }
}

macro_rules! pick_regs_of_alias {
    ($alias: ident) => {
        $alias.values().map(|x| x.clone()).collect()
    }
}

lazy_static! {
    pub static ref ALL_GPR8s  : Vec<P<Value>> = pick_regs_of_length!(8);
    pub static ref ALL_GPR16s : Vec<P<Value>> = pick_regs_of_length!(16);
    pub static ref ALL_GPR32s : Vec<P<Value>> = pick_regs_of_length!(32);
    pub static ref ALL_GPR64s : Vec<P<Value>> = pick_regs_of_length!(64);
}

// only use 64bit registers here
lazy_static!{
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
}

lazy_static!{
    pub static ref XMM0  : P<Value> = FPR!(66,"xmm0");
    pub static ref XMM1  : P<Value> = FPR!(67,"xmm1");
    pub static ref XMM2  : P<Value> = FPR!(68,"xmm2");
    pub static ref XMM3  : P<Value> = FPR!(69,"xmm3");
    pub static ref XMM4  : P<Value> = FPR!(70,"xmm4");
    pub static ref XMM5  : P<Value> = FPR!(71,"xmm5");
    pub static ref XMM6  : P<Value> = FPR!(72,"xmm6");
    pub static ref XMM7  : P<Value> = FPR!(73,"xmm7");
    pub static ref XMM8  : P<Value> = FPR!(74,"xmm8");
    pub static ref XMM9  : P<Value> = FPR!(75,"xmm9");
    pub static ref XMM10 : P<Value> = FPR!(76,"xmm10");
    pub static ref XMM11 : P<Value> = FPR!(77,"xmm11");
    pub static ref XMM12 : P<Value> = FPR!(78,"xmm12");
    pub static ref XMM13 : P<Value> = FPR!(79,"xmm13");
    pub static ref XMM14 : P<Value> = FPR!(80,"xmm14");
    pub static ref XMM15 : P<Value> = FPR!(81,"xmm15");
    
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

lazy_static! {
    pub static ref ALL_MACHINE_REGs : HashMap<MuID, P<Value>> = {
        let mut ret = HashMap::new();

        // gprs
        for map in ALL_GPR_ALIAS_MAPs.iter() {
            for val in map.values() {
                ret.insert(val.id(), val.clone());
            }
        }

        ret.insert(RIP.id(), RIP.clone());

        // fprs
        for val in ALL_FPRs.iter() {
            ret.insert(val.id(), val.clone());
        }

        ret
    };
    
    // put caller saved regs first (they imposes no overhead if there is no call instruction)
    pub static ref ALL_USABLE_MACHINE_REGs : Vec<P<Value>> = {
        let mut ret = vec![];

//        ret.append(&mut pick_regs_of_alias!(RAX_ALIAS));
//        ret.append(&mut pick_regs_of_alias!(RCX_ALIAS));
//        ret.append(&mut pick_regs_of_alias!(RDX_ALIAS));
//        ret.append(&mut pick_regs_of_alias!(RBX_ALIAS));
//
//        ret.append(&mut pick_regs_of_alias!(RSI_ALIAS));
//        ret.append(&mut pick_regs_of_alias!(RDI_ALIAS));
//
//        ret.append(&mut pick_regs_of_alias!(R8_ALIAS));
//        ret.append(&mut pick_regs_of_alias!(R9_ALIAS));
//        ret.append(&mut pick_regs_of_alias!(R10_ALIAS));
//        ret.append(&mut pick_regs_of_alias!(R11_ALIAS));
//        ret.append(&mut pick_regs_of_alias!(R12_ALIAS));
//        ret.append(&mut pick_regs_of_alias!(R13_ALIAS));
//        ret.append(&mut pick_regs_of_alias!(R14_ALIAS));
//        ret.append(&mut pick_regs_of_alias!(R15_ALIAS));

        ret.push(RAX.clone());
        ret.push(RCX.clone());
        ret.push(RDX.clone());
        ret.push(RBX.clone());
        ret.push(RSI.clone());
        ret.push(RDI.clone());
        ret.push(R8.clone());
        ret.push(R9.clone());
        ret.push(R10.clone());
        ret.push(R11.clone());
        ret.push(R12.clone());
        ret.push(R13.clone());
        ret.push(R14.clone());
        ret.push(R15.clone());

        ret.push(XMM0.clone());
        ret.push(XMM1.clone());
        ret.push(XMM2.clone());
        ret.push(XMM3.clone());
        ret.push(XMM4.clone());
        ret.push(XMM5.clone());
        ret.push(XMM6.clone());
        ret.push(XMM7.clone());
        ret.push(XMM8.clone());
        ret.push(XMM9.clone());
        ret.push(XMM10.clone());
        ret.push(XMM11.clone());
        ret.push(XMM12.clone());
        ret.push(XMM13.clone());
        ret.push(XMM14.clone());
        ret.push(XMM15.clone());

        ret
    };
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
        RegGroup::GPR8  => 0,
        RegGroup::GPR16 => 0,
        RegGroup::GPR32 => 0,
        RegGroup::GPR64 => ALL_GPR64s.len(),
        RegGroup::FPR32 => 0,
        RegGroup::FPR64 => ALL_FPRs.len(),
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
    RegGroup::get(&reg.ty)
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
