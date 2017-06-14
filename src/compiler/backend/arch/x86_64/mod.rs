#![allow(dead_code)]
#![allow(non_upper_case_globals)]

pub mod inst_sel;

mod codegen;
pub use compiler::backend::x86_64::codegen::CodeGenerator;

pub mod asm_backend;
pub use compiler::backend::x86_64::asm_backend::ASMCodeGen;
pub use compiler::backend::x86_64::asm_backend::emit_code;
pub use compiler::backend::x86_64::asm_backend::emit_context;
pub use compiler::backend::x86_64::asm_backend::emit_context_with_reloc;
#[cfg(feature = "aot")]
pub use compiler::backend::x86_64::asm_backend::spill_rewrite;

use ast::ptr::P;
use ast::ir::*;
use ast::types::*;
use compiler::backend::RegGroup;

use utils::LinkedHashMap;
use std::collections::HashMap;

macro_rules! GPR_ALIAS {
    ($alias: ident: ($id64: expr, $r64: ident) -> $r32: ident, $r16: ident, $r8l: ident, $r8h: ident) => {
        lazy_static!{
            pub static ref $r64 : P<Value> = GPR!($id64,    stringify!($r64), UINT64_TYPE);
            pub static ref $r32 : P<Value> = GPR!($id64 +1, stringify!($r32), UINT32_TYPE);
            pub static ref $r16 : P<Value> = GPR!($id64 +2, stringify!($r16), UINT16_TYPE);
            pub static ref $r8l : P<Value> = GPR!($id64 +3, stringify!($r8l), UINT8_TYPE);
            pub static ref $r8h : P<Value> = GPR!($id64 +4, stringify!($r8h), UINT8_TYPE);

            pub static ref $alias : [P<Value>; 5] = [$r64.clone(), $r32.clone(), $r16.clone(), $r8l.clone(), $r8h.clone()];
        }
    };

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

GPR_ALIAS!(RAX_ALIAS: (0, RAX)  -> EAX, AX , AL, AH);
GPR_ALIAS!(RCX_ALIAS: (5, RCX)  -> ECX, CX , CL, CH);
GPR_ALIAS!(RDX_ALIAS: (10,RDX)  -> EDX, DX , DL, DH);
GPR_ALIAS!(RBX_ALIAS: (15,RBX)  -> EBX, BX , BL, BH);
GPR_ALIAS!(RSP_ALIAS: (20,RSP)  -> ESP, SP , SPL);
GPR_ALIAS!(RBP_ALIAS: (24,RBP)  -> EBP, BP , BPL);
GPR_ALIAS!(RSI_ALIAS: (28,RSI)  -> ESI, SI , SIL);
GPR_ALIAS!(RDI_ALIAS: (32,RDI)  -> EDI, DI , DIL);
GPR_ALIAS!(R8_ALIAS : (36,R8 )  -> R8D, R8W, R8B);
GPR_ALIAS!(R9_ALIAS : (40,R9 )  -> R9D, R9W, R9B);
GPR_ALIAS!(R10_ALIAS: (44,R10) -> R10D,R10W,R10B);
GPR_ALIAS!(R11_ALIAS: (48,R11) -> R11D,R11W,R11B);
GPR_ALIAS!(R12_ALIAS: (52,R12) -> R12D,R12W,R12B);
GPR_ALIAS!(R13_ALIAS: (56,R13) -> R13D,R13W,R13B);
GPR_ALIAS!(R14_ALIAS: (60,R14) -> R14D,R14W,R14B);
GPR_ALIAS!(R15_ALIAS: (64,R15) -> R15D,R15W,R15B);
GPR_ALIAS!(RIP_ALIAS: (68,RIP));

lazy_static! {
    pub static ref GPR_ALIAS_TABLE : LinkedHashMap<MuID, Vec<P<Value>>> = {
        let mut ret = LinkedHashMap::new();

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

    // e.g. given eax, return rax
    pub static ref GPR_ALIAS_LOOKUP : HashMap<MuID, P<Value>> = {
        let mut ret = HashMap::new();

        for vec in GPR_ALIAS_TABLE.values() {
            let colorable = vec[0].clone();

            for gpr in vec {
                ret.insert(gpr.id(), colorable.clone());
            }
        }

        ret
    };
}

pub fn get_alias_for_length(id: MuID, length: usize) -> P<Value> {
    if id < FPR_ID_START {
        let vec = match GPR_ALIAS_TABLE.get(&id) {
            Some(vec) => vec,
            None => panic!("didnt find {} as GPR", id)
        };

        match length {
            64 => vec[0].clone(),
            32 => vec[1].clone(),
            16 => vec[2].clone(),
            8 => vec[3].clone(),
            1 => vec[3].clone(),
            _ => panic!("unexpected length {} for {}", length, vec[0])
        }
    } else {
        for r in ALL_FPRs.iter() {
            if r.id() == id {
                return r.clone();
            }
        }

        panic!("didnt find {} as FPR", id)
    }
}

pub fn is_aliased(id1: MuID, id2: MuID) -> bool {
    if get_color_for_precolored(id1) == get_color_for_precolored(id2) {
        macro_rules! is_match {
            ($a1: expr, $a2: expr; $b: expr) => {
                $a1 == $b.id() || $a2 == $b.id()
            }
        };

        if is_match!(id1, id2; AH) {
            return false;
        } else if is_match!(id1, id2; BH) {
            return false;
        } else if is_match!(id1, id2; CH) {
            return false;
        } else if is_match!(id1, id2; DH) {
            return false;
        } else {
            return true;
        }
    } else {
        return false;
    }
}

pub fn get_color_for_precolored(id: MuID) -> MuID {
    debug_assert!(id < MACHINE_ID_END);

    if id < FPR_ID_START {
        match GPR_ALIAS_LOOKUP.get(&id) {
            Some(val) => val.id(),
            None => panic!("cannot find GPR {}", id)
        }
    } else {
        // we do not have alias for FPRs
        id
    }
}

#[inline(always)]
pub fn check_op_len(op: &P<Value>) -> usize {
    match op.ty.get_int_length() {
        Some(64) => 64,
        Some(32) => 32,
        Some(16) => 16,
        Some(8)  => 8,
        Some(1)  => 8,
        _ => panic!("unimplemented int types: {}", op.ty)
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

    static ref ALL_GPRs : [P<Value>; 15] = [
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

pub const FPR_ID_START : usize = 100;

lazy_static!{
    pub static ref XMM0  : P<Value> = FPR!(FPR_ID_START,    "xmm0");
    pub static ref XMM1  : P<Value> = FPR!(FPR_ID_START + 1,"xmm1");
    pub static ref XMM2  : P<Value> = FPR!(FPR_ID_START + 2,"xmm2");
    pub static ref XMM3  : P<Value> = FPR!(FPR_ID_START + 3,"xmm3");
    pub static ref XMM4  : P<Value> = FPR!(FPR_ID_START + 4,"xmm4");
    pub static ref XMM5  : P<Value> = FPR!(FPR_ID_START + 5,"xmm5");
    pub static ref XMM6  : P<Value> = FPR!(FPR_ID_START + 6,"xmm6");
    pub static ref XMM7  : P<Value> = FPR!(FPR_ID_START + 7,"xmm7");
    pub static ref XMM8  : P<Value> = FPR!(FPR_ID_START + 8,"xmm8");
    pub static ref XMM9  : P<Value> = FPR!(FPR_ID_START + 9,"xmm9");
    pub static ref XMM10 : P<Value> = FPR!(FPR_ID_START + 10,"xmm10");
    pub static ref XMM11 : P<Value> = FPR!(FPR_ID_START + 11,"xmm11");
    pub static ref XMM12 : P<Value> = FPR!(FPR_ID_START + 12,"xmm12");
    pub static ref XMM13 : P<Value> = FPR!(FPR_ID_START + 13,"xmm13");
    pub static ref XMM14 : P<Value> = FPR!(FPR_ID_START + 14,"xmm14");
    pub static ref XMM15 : P<Value> = FPR!(FPR_ID_START + 15,"xmm15");

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

    static ref ALL_FPRs : [P<Value>; 16] = [
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
    pub static ref ALL_MACHINE_REGs : LinkedHashMap<MuID, P<Value>> = {
        let mut map = LinkedHashMap::new();

        for vec in GPR_ALIAS_TABLE.values() {
            for reg in vec {
                map.insert(reg.id(), reg.clone());
            }
        }

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
        RegGroup::GPR   => ALL_GPRs.len(),
        RegGroup::GPREX => ALL_GPRs.len(),
        RegGroup::FPR   => ALL_FPRs.len()
    }
}

pub fn number_of_all_regs() -> usize {
    ALL_MACHINE_REGs.len()
}

pub fn all_regs() -> &'static LinkedHashMap<MuID, P<Value>> {
    &ALL_MACHINE_REGs
}

pub fn all_usable_regs() -> &'static Vec<P<Value>> {
    &ALL_USABLE_MACHINE_REGs
}

pub fn pick_group_for_reg(reg_id: MuID) -> RegGroup {
    let reg = all_regs().get(&reg_id).unwrap();
    RegGroup::get_from_value(reg)
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

use ast::inst::*;
pub fn estimate_insts_for_ir(inst: &Instruction) -> usize {
    use ast::inst::Instruction_::*;

    match inst.v {
        // simple
        BinOp(_, _, _)  => 1,
        BinOpWithStatus(_, _, _, _) => 2,
        CmpOp(_, _, _)  => 1,
        ConvOp{..}      => 0,

        // control flow
        Branch1(_)     => 1,
        Branch2{..}    => 1,
        Select{..}     => 2,
        Watchpoint{..} => 1,
        WPBranch{..}   => 2,
        Switch{..}     => 3,

        // call
        ExprCall{..} | ExprCCall{..} | Call{..} | CCall{..} => 5,
        Return(_)   => 1,
        TailCall(_) => 1,

        // memory access
        Load{..} | Store{..} => 1,
        CmpXchg{..}          => 1,
        AtomicRMW{..}        => 1,
        AllocA(_)            => 1,
        AllocAHybrid(_, _)   => 1,
        Fence(_)             => 1,

        // memory addressing
        GetIRef(_) | GetFieldIRef{..} | GetElementIRef{..} | ShiftIRef{..} | GetVarPartIRef{..} => 0,

        // runtime
        New(_) | NewHybrid(_, _) => 10,
        NewStack(_) | NewThread(_, _) | NewThreadExn(_, _) | NewFrameCursor(_) => 10,
        ThreadExit    => 10,
        Throw(_)      => 10,
        SwapStack{..} => 10,
        CommonInst_GetThreadLocal | CommonInst_SetThreadLocal(_) => 10,
        CommonInst_Pin(_) | CommonInst_Unpin(_) => 10,

        // others
        Move(_) => 0,
        PrintHex(_)  => 10,
        SetRetval(_) => 10,
        ExnInstruction{ref inner, ..} => estimate_insts_for_ir(&inner)
    }
}
