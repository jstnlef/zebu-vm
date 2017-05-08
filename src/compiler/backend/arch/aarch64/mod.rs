#![allow(non_upper_case_globals)]

pub mod inst_sel;

mod codegen;
pub use compiler::backend::aarch64::codegen::CodeGenerator;

mod asm_backend;
pub use compiler::backend::aarch64::asm_backend::ASMCodeGen;
pub use compiler::backend::aarch64::asm_backend::emit_code;
pub use compiler::backend::aarch64::asm_backend::emit_context;
pub use compiler::backend::aarch64::asm_backend::emit_context_with_reloc;
#[cfg(feature = "aot")]
pub use compiler::backend::aarch64::asm_backend::spill_rewrite;

use ast::ptr::P;
use ast::ir::*;
use ast::types::*;
use ast::op;
use compiler::backend::RegGroup;
use vm::VM;

use utils::LinkedHashMap;
use std::collections::HashMap;

macro_rules! REGISTER {
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

macro_rules! GPR_ALIAS {
    ($alias: ident: ($id64: expr, $r64: ident) -> $r32: ident) => {
        lazy_static!{
            pub static ref $r64 : P<Value> = REGISTER!($id64,    stringify!($r64), UINT64_TYPE);
            pub static ref $r32 : P<Value> = REGISTER!($id64 +1, stringify!($r32), UINT32_TYPE);
            pub static ref $alias : [P<Value>; 2] = [$r64.clone(), $r32.clone()];
        }
    };
}

// Used to create a generic alias name
macro_rules! ALIAS {
    ($src: ident -> $dest: ident) => {
        //pub use $src as $dest;
        lazy_static!{
            pub static ref $dest : P<Value> = $src.clone();
        }
    };
}


macro_rules! FPR_ALIAS {
    ($alias: ident: ($id64: expr, $r64: ident) -> $r32: ident) => {
        lazy_static!{
            pub static ref $r64 : P<Value> = REGISTER!($id64,    stringify!($r64), DOUBLE_TYPE);
            pub static ref $r32 : P<Value> = REGISTER!($id64 +1, stringify!($r32), FLOAT_TYPE);
            pub static ref $alias : [P<Value>; 2] = [$r64.clone(), $r32.clone()];
        }
    };
}

GPR_ALIAS!(X0_ALIAS: (0, X0)  -> W0);
GPR_ALIAS!(X1_ALIAS: (2, X1)  -> W1);
GPR_ALIAS!(X2_ALIAS: (4, X2)  -> W2);
GPR_ALIAS!(X3_ALIAS: (6, X3)  -> W3);
GPR_ALIAS!(X4_ALIAS: (8, X4)  -> W4);
GPR_ALIAS!(X5_ALIAS: (10, X5)  -> W5);
GPR_ALIAS!(X6_ALIAS: (12, X6)  -> W6);
GPR_ALIAS!(X7_ALIAS: (14, X7)  -> W7);
GPR_ALIAS!(X8_ALIAS: (16, X8)  -> W8);
GPR_ALIAS!(X9_ALIAS: (18, X9)  -> W9);
GPR_ALIAS!(X10_ALIAS: (20, X10)  -> W10);
GPR_ALIAS!(X11_ALIAS: (22, X11)  -> W11);
GPR_ALIAS!(X12_ALIAS: (24, X12)  -> W12);
GPR_ALIAS!(X13_ALIAS: (26, X13)  -> W13);
GPR_ALIAS!(X14_ALIAS: (28, X14)  -> W14);
GPR_ALIAS!(X15_ALIAS: (30, X15)  -> W15);
GPR_ALIAS!(X16_ALIAS: (32, X16)  -> W16);
GPR_ALIAS!(X17_ALIAS: (34, X17)  -> W17);
GPR_ALIAS!(X18_ALIAS: (36, X18)  -> W18);
GPR_ALIAS!(X19_ALIAS: (38, X19)  -> W19);
GPR_ALIAS!(X20_ALIAS: (40, X20)  -> W20);
GPR_ALIAS!(X21_ALIAS: (42, X21)  -> W21);
GPR_ALIAS!(X22_ALIAS: (44, X22)  -> W22);
GPR_ALIAS!(X23_ALIAS: (46, X23)  -> W23);
GPR_ALIAS!(X24_ALIAS: (48, X24)  -> W24);
GPR_ALIAS!(X25_ALIAS: (50, X25)  -> W25);
GPR_ALIAS!(X26_ALIAS: (52, X26)  -> W26);
GPR_ALIAS!(X27_ALIAS: (54, X27)  -> W27);
GPR_ALIAS!(X28_ALIAS: (56, X28)  -> W28);
GPR_ALIAS!(X29_ALIAS: (58, X29)  -> W29);
GPR_ALIAS!(X30_ALIAS: (60, X30)  -> W30);
GPR_ALIAS!(SP_ALIAS: (62, SP)  -> WSP); // Special register (only some instructions can reference it)
GPR_ALIAS!(XZR_ALIAS: (64, XZR)  -> WZR); // Pseudo register, not to be used by register allocator

// Aliases
ALIAS!(X8 -> XR); // Indirect result location register (points to a location in memory to write return values to)
ALIAS!(X16 -> IP0); // Intra proecdure call register 0 (may be modified by the linker when executing BL/BLR instructions)
ALIAS!(X17 -> IP1);// Intra proecdure call register 1 (may be modified by the linker when executing BL/BLR instructions)
    ALIAS!(X18 -> PR); // Platform Register (NEVER TOUCH THIS REGISTER (Unless you can proove Linux dosn't use it))
ALIAS!(X29 -> FP); // Frame Pointer (can be used as a normal register when not calling or returning)
ALIAS!(X30 -> LR); // Link Register (not supposed to be used for any other purpose)


lazy_static! {
    pub static ref GPR_ALIAS_TABLE : LinkedHashMap<MuID, Vec<P<Value>>> = {
        let mut ret = LinkedHashMap::new();

        ret.insert(X0.id(), X0_ALIAS.to_vec());
        ret.insert(X1.id(), X1_ALIAS.to_vec());
        ret.insert(X2.id(), X2_ALIAS.to_vec());
        ret.insert(X3.id(), X3_ALIAS.to_vec());
        ret.insert(X4.id(), X4_ALIAS.to_vec());
        ret.insert(X5.id(), X5_ALIAS.to_vec());
        ret.insert(X6.id(), X6_ALIAS.to_vec());
        ret.insert(X7.id(), X7_ALIAS.to_vec());
        ret.insert(X8.id(), X8_ALIAS.to_vec());
        ret.insert(X9.id(), X9_ALIAS.to_vec());
        ret.insert(X10.id(), X10_ALIAS.to_vec());
        ret.insert(X11.id(), X11_ALIAS.to_vec());
        ret.insert(X12.id(), X12_ALIAS.to_vec());
        ret.insert(X13.id(), X13_ALIAS.to_vec());
        ret.insert(X14.id(), X14_ALIAS.to_vec());
        ret.insert(X15.id(), X15_ALIAS.to_vec());
        ret.insert(X16.id(), X16_ALIAS.to_vec());
        ret.insert(X17.id(), X17_ALIAS.to_vec());
        ret.insert(X18.id(), X18_ALIAS.to_vec());
        ret.insert(X19.id(), X19_ALIAS.to_vec());
        ret.insert(X20.id(), X20_ALIAS.to_vec());
        ret.insert(X21.id(), X21_ALIAS.to_vec());
        ret.insert(X22.id(), X22_ALIAS.to_vec());
        ret.insert(X23.id(), X23_ALIAS.to_vec());
        ret.insert(X24.id(), X24_ALIAS.to_vec());
        ret.insert(X25.id(), X25_ALIAS.to_vec());
        ret.insert(X26.id(), X26_ALIAS.to_vec());
        ret.insert(X27.id(), X27_ALIAS.to_vec());
        ret.insert(X28.id(), X28_ALIAS.to_vec());
        ret.insert(X29.id(), X29_ALIAS.to_vec());
        ret.insert(X30.id(), X30_ALIAS.to_vec());
        ret.insert(SP.id(), SP_ALIAS.to_vec());
        ret.insert(XZR.id(), XZR_ALIAS.to_vec());
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

// Is val a hard coded machine register (not a pseudo register)
pub fn is_machine_reg(val: &P<Value>) -> bool {
    match val.v {
        Value_::SSAVar(ref id) => {
            if *id < FPR_ID_START {
                match GPR_ALIAS_LOOKUP.get(&id) {
                    Some(_) => true,
                    None => false
                }
            } else {
                match FPR_ALIAS_LOOKUP.get(&id) {
                    Some(_) => true,
                    None => false
                }
            }
        }
        _ => false
    }

}


// Returns a P<Value> to the register id
pub fn get_register_from_id(id: MuID) -> P<Value> {
    if id < FPR_ID_START {
        match GPR_ALIAS_LOOKUP.get(&id) {
            Some(val) => val.clone(),
            None => panic!("cannot find GPR {}", id)
        }
    } else {
        match FPR_ALIAS_LOOKUP.get(&id) {
            Some(val) => val.clone(),
            None => panic!("cannot find FPR {}", id)
        }
    }
}

pub fn get_alias_for_length(id: MuID, length: usize) -> P<Value> {
    if id < FPR_ID_START {
        let vec = match GPR_ALIAS_TABLE.get(&id) {
            Some(vec) => vec,
            None => panic!("didnt find {} as GPR", id)
        };

        match length {
            64 => vec[0].clone(),
            _ if length <= 32 => vec[1].clone(),
            _ => panic!("unexpected length {} for {}", length, vec[0])
        }
    } else {
        let vec = match FPR_ALIAS_TABLE.get(&id) {
            Some(vec) => vec,
            None => panic!("didnt find {} as FPR", id)
        };

        match length {
            64 => vec[0].clone(),
            32 => vec[1].clone(),
            _ => panic!("unexpected length {} for {}", length, vec[0])
        }
    }
}

pub fn is_aliased(id1: MuID, id2: MuID) -> bool {
    return get_color_for_precolored(id1) == get_color_for_precolored(id2);
}

pub fn get_color_for_precolored(id: MuID) -> MuID {
    debug_assert!(id < MACHINE_ID_END);

    if id < FPR_ID_START {
        match GPR_ALIAS_LOOKUP.get(&id) {
            Some(val) => val.id(),
            None => panic!("cannot find GPR {}", id)
        }
    } else {
        match FPR_ALIAS_LOOKUP.get(&id) {
            Some(val) => val.id(),
            None => panic!("cannot find FPR {}", id)
        }
    }
}

#[inline(always)]
pub fn check_op_len(ty: &P<MuType>) -> usize {
    match ty.get_int_length() {
        Some(64) => 64,
        Some(32) => 32,
        Some(n) if n < 32 => 32,
        Some(n) => panic!("unimplemented int size: {}", n),
        None => {
            match ty.v {
                MuType_::Float => 32,
                MuType_::Double => 64,
                _ => panic!("unimplemented primitive type: {}", ty)
            }
        }
    }
}

#[inline(always)]
pub fn get_bit_size(ty : &P<MuType>, vm: &VM) -> usize
{
    match ty.get_int_length() {
        Some(val) => val,
        None => {
            match ty.v {
                MuType_::Float => 32,
                MuType_::Double => 64,
                MuType_::Vector(ref t, n) => get_bit_size(t, vm)*n,
                MuType_::Array(ref t, n) => get_bit_size(t, vm)*n,
                MuType_::Void => 0,
                _ => vm.get_type_size(ty.id())*8,
            }
        }
    }
}

#[inline(always)]
pub fn primitive_byte_size(ty : &P<MuType>) -> usize
{
    match ty.get_int_length() {
        Some(val) => round_up(val, 8)/8,
        None => {
            match ty.v {
                MuType_::Float => 4,
                MuType_::Double => 8,
                MuType_::Void => 0,
                _ => panic!("Not a primitive type")
            }
        }
    }
}

lazy_static! {
    // Note: these are the same as the ARGUMENT_GPRs
    pub static ref RETURN_GPRs : [P<Value>; 8] = [
        X0.clone(),
        X1.clone(),
        X2.clone(),
        X3.clone(),
        X4.clone(),
        X5.clone(),
        X6.clone(),
        X7.clone()
    ];

    pub static ref ARGUMENT_GPRs : [P<Value>; 8] = [
        X0.clone(),
        X1.clone(),
        X2.clone(),
        X3.clone(),
        X4.clone(),
        X5.clone(),
        X6.clone(),
        X7.clone()
    ];

    pub static ref CALLEE_SAVED_GPRs : [P<Value>; 10] = [
        X19.clone(),
        X20.clone(),
        X21.clone(),
        X22.clone(),
        X23.clone(),
        X24.clone(),
        X25.clone(),
        X26.clone(),
        X27.clone(),
        X28.clone(),

        // Note: These two are technically CALEE saved but need to be dealt with specially
        //X29.clone(), // Frame Pointer
        //X30.clone() // Link Register
    ];

    pub static ref CALLER_SAVED_GPRs : [P<Value>; 18] = [
        X0.clone(),
        X1.clone(),
        X2.clone(),
        X3.clone(),
        X4.clone(),
        X5.clone(),
        X6.clone(),
        X7.clone(),
        X8.clone(),
        X9.clone(),
        X10.clone(),
        X11.clone(),
        X12.clone(),
        X13.clone(),
        X14.clone(),
        X15.clone(),
        X16.clone(),
        X17.clone(),
        //X18.clone(), // Platform Register
    ];

    static ref ALL_GPRs : [P<Value>; 30] = [
        X0.clone(),
        X1.clone(),
        X2.clone(),
        X3.clone(),
        X4.clone(),
        X5.clone(),
        X6.clone(),
        X7.clone(),
        X8.clone(),
        X9.clone(),
        X10.clone(),
        X11.clone(),
        X12.clone(),
        X13.clone(),
        X14.clone(),
        X15.clone(),
        X16.clone(),
        X17.clone(),
        //X18.clone(), // Platform Register
        X19.clone(),
        X20.clone(),
        X21.clone(),
        X22.clone(),
        X23.clone(),
        X24.clone(),
        X25.clone(),
        X26.clone(),
        X27.clone(),
        X28.clone(),
        X29.clone(), // Frame Pointer
        X30.clone() // Link Register
    ];
}

pub const FPR_ID_START : usize = 100;

FPR_ALIAS!(D0_ALIAS: (FPR_ID_START + 0, D0)  -> S0);
FPR_ALIAS!(D1_ALIAS: (FPR_ID_START + 2, D1)  -> S1);
FPR_ALIAS!(D2_ALIAS: (FPR_ID_START + 4, D2)  -> S2);
FPR_ALIAS!(D3_ALIAS: (FPR_ID_START + 6, D3)  -> S3);
FPR_ALIAS!(D4_ALIAS: (FPR_ID_START + 8, D4)  -> S4);
FPR_ALIAS!(D5_ALIAS: (FPR_ID_START + 10, D5)  -> S5);
FPR_ALIAS!(D6_ALIAS: (FPR_ID_START + 12, D6)  -> S6);
FPR_ALIAS!(D7_ALIAS: (FPR_ID_START + 14, D7)  -> S7);
FPR_ALIAS!(D8_ALIAS: (FPR_ID_START + 16, D8)  -> S8);
FPR_ALIAS!(D9_ALIAS: (FPR_ID_START + 18, D9)  -> S9);
FPR_ALIAS!(D10_ALIAS: (FPR_ID_START + 20, D10)  -> S10);
FPR_ALIAS!(D11_ALIAS: (FPR_ID_START + 22, D11)  -> S11);
FPR_ALIAS!(D12_ALIAS: (FPR_ID_START + 24, D12)  -> S12);
FPR_ALIAS!(D13_ALIAS: (FPR_ID_START + 26, D13)  -> S13);
FPR_ALIAS!(D14_ALIAS: (FPR_ID_START + 28, D14)  -> S14);
FPR_ALIAS!(D15_ALIAS: (FPR_ID_START + 30, D15)  -> S15);
FPR_ALIAS!(D16_ALIAS: (FPR_ID_START + 32, D16)  -> S16);
FPR_ALIAS!(D17_ALIAS: (FPR_ID_START + 34, D17)  -> S17);
FPR_ALIAS!(D18_ALIAS: (FPR_ID_START + 36, D18)  -> S18);
FPR_ALIAS!(D19_ALIAS: (FPR_ID_START + 38, D19)  -> S19);
FPR_ALIAS!(D20_ALIAS: (FPR_ID_START + 40, D20)  -> S20);
FPR_ALIAS!(D21_ALIAS: (FPR_ID_START + 42, D21)  -> S21);
FPR_ALIAS!(D22_ALIAS: (FPR_ID_START + 44, D22)  -> S22);
FPR_ALIAS!(D23_ALIAS: (FPR_ID_START + 46, D23)  -> S23);
FPR_ALIAS!(D24_ALIAS: (FPR_ID_START + 48, D24)  -> S24);
FPR_ALIAS!(D25_ALIAS: (FPR_ID_START + 50, D25)  -> S25);
FPR_ALIAS!(D26_ALIAS: (FPR_ID_START + 52, D26)  -> S26);
FPR_ALIAS!(D27_ALIAS: (FPR_ID_START + 54, D27)  -> S27);
FPR_ALIAS!(D28_ALIAS: (FPR_ID_START + 56, D28)  -> S28);
FPR_ALIAS!(D29_ALIAS: (FPR_ID_START + 58, D29)  -> S29);
FPR_ALIAS!(D30_ALIAS: (FPR_ID_START + 60, D30)  -> S30);
FPR_ALIAS!(D31_ALIAS: (FPR_ID_START + 62, D31)  -> S31);

lazy_static! {
    pub static ref FPR_ALIAS_TABLE : LinkedHashMap<MuID, Vec<P<Value>>> = {
        let mut ret = LinkedHashMap::new();

        ret.insert(D0.id(), D0_ALIAS.to_vec());
        ret.insert(D1.id(), D1_ALIAS.to_vec());
        ret.insert(D2.id(), D2_ALIAS.to_vec());
        ret.insert(D3.id(), D3_ALIAS.to_vec());
        ret.insert(D4.id(), D4_ALIAS.to_vec());
        ret.insert(D5.id(), D5_ALIAS.to_vec());
        ret.insert(D6.id(), D6_ALIAS.to_vec());
        ret.insert(D7.id(), D7_ALIAS.to_vec());
        ret.insert(D8.id(), D8_ALIAS.to_vec());
        ret.insert(D9.id(), D9_ALIAS.to_vec());
        ret.insert(D10.id(), D10_ALIAS.to_vec());
        ret.insert(D11.id(), D11_ALIAS.to_vec());
        ret.insert(D12.id(), D12_ALIAS.to_vec());
        ret.insert(D13.id(), D13_ALIAS.to_vec());
        ret.insert(D14.id(), D14_ALIAS.to_vec());
        ret.insert(D15.id(), D15_ALIAS.to_vec());
        ret.insert(D16.id(), D16_ALIAS.to_vec());
        ret.insert(D17.id(), D17_ALIAS.to_vec());
        ret.insert(D18.id(), D18_ALIAS.to_vec());
        ret.insert(D19.id(), D19_ALIAS.to_vec());
        ret.insert(D20.id(), D20_ALIAS.to_vec());
        ret.insert(D21.id(), D21_ALIAS.to_vec());
        ret.insert(D22.id(), D22_ALIAS.to_vec());
        ret.insert(D23.id(), D23_ALIAS.to_vec());
        ret.insert(D24.id(), D24_ALIAS.to_vec());
        ret.insert(D25.id(), D25_ALIAS.to_vec());
        ret.insert(D26.id(), D26_ALIAS.to_vec());
        ret.insert(D27.id(), D27_ALIAS.to_vec());
        ret.insert(D28.id(), D28_ALIAS.to_vec());
        ret.insert(D29.id(), D29_ALIAS.to_vec());
        ret.insert(D30.id(), D30_ALIAS.to_vec());
        ret.insert(D31.id(), D31_ALIAS.to_vec());

        ret
    };


    pub static ref FPR_ALIAS_LOOKUP : HashMap<MuID, P<Value>> = {
        let mut ret = HashMap::new();

        for vec in FPR_ALIAS_TABLE.values() {
            let colorable = vec[0].clone();

            for fpr in vec {
                ret.insert(fpr.id(), colorable.clone());
            }
        }

        ret
    };
}

lazy_static!{
    // Same as ARGUMENT_FPRs
    pub static ref RETURN_FPRs : [P<Value>; 8] = [
        D0.clone(),
        D1.clone(),
        D2.clone(),
        D3.clone(),
        D4.clone(),
        D5.clone(),
        D6.clone(),
        D7.clone()
    ];

    pub static ref ARGUMENT_FPRs : [P<Value>; 8] = [
        D0.clone(),
        D1.clone(),
        D2.clone(),
        D3.clone(),
        D4.clone(),
        D5.clone(),
        D6.clone(),
        D7.clone(),
    ];

    pub static ref CALLEE_SAVED_FPRs : [P<Value>; 8] = [
        D8.clone(),
        D9.clone(),
        D10.clone(),
        D11.clone(),
        D12.clone(),
        D13.clone(),
        D14.clone(),
        D15.clone()
    ];

    pub static ref CALLER_SAVED_FPRs : [P<Value>; 24] = [
        D0.clone(),
        D1.clone(),
        D2.clone(),
        D3.clone(),
        D4.clone(),
        D5.clone(),
        D6.clone(),
        D7.clone(),

        D16.clone(),
        D17.clone(),
        D18.clone(),
        D19.clone(),
        D20.clone(),
        D21.clone(),
        D22.clone(),
        D23.clone(),
        D24.clone(),
        D25.clone(),
        D26.clone(),
        D27.clone(),
        D28.clone(),
        D29.clone(),
        D30.clone(),
        D31.clone()
    ];

    static ref ALL_FPRs : [P<Value>; 32] = [
        D0.clone(),
        D1.clone(),
        D2.clone(),
        D3.clone(),
        D4.clone(),
        D5.clone(),
        D6.clone(),
        D7.clone(),

        D8.clone(),
        D9.clone(),
        D10.clone(),
        D11.clone(),
        D12.clone(),
        D13.clone(),
        D14.clone(),
        D15.clone(),

        D16.clone(),
        D17.clone(),
        D18.clone(),
        D19.clone(),
        D20.clone(),
        D21.clone(),
        D22.clone(),
        D23.clone(),
        D24.clone(),
        D25.clone(),
        D26.clone(),
        D27.clone(),
        D28.clone(),
        D29.clone(),
        D30.clone(),
        D31.clone()
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

        for vec in FPR_ALIAS_TABLE.values() {
            for reg in vec {
                map.insert(reg.id(), reg.clone());
            }
        }

        map
    };

    // put caller saved regs first (they imposes no overhead if there is no call instruction)
    pub static ref ALL_USABLE_MACHINE_REGs : Vec<P<Value>> = vec![
        X19.clone(),
        X20.clone(),
        X21.clone(),
        X22.clone(),
        X23.clone(),
        X24.clone(),
        X25.clone(),
        X26.clone(),
        X27.clone(),
        X28.clone(),
        //X29.clone(), // Frame Pointer
        //X30.clone(), // Link Register

        X0.clone(),
        X1.clone(),
        X2.clone(),
        X3.clone(),
        X4.clone(),
        X5.clone(),
        X6.clone(),
        X7.clone(),
        X8.clone(),
        X9.clone(),
        X10.clone(),
        X11.clone(),
        X12.clone(),
        X13.clone(),
        X14.clone(),
        X15.clone(),
        X16.clone(),
        X17.clone(),
        // X18.clone(), // Platform Register

        D8.clone(),
        D9.clone(),
        D10.clone(),
        D11.clone(),
        D12.clone(),
        D13.clone(),
        D14.clone(),
        D15.clone(),

        D0.clone(),
        D1.clone(),
        D2.clone(),
        D3.clone(),
        D4.clone(),
        D5.clone(),
        D6.clone(),
        D7.clone(),

        D16.clone(),
        D17.clone(),
        D18.clone(),
        D19.clone(),
        D20.clone(),
        D21.clone(),
        D22.clone(),
        D23.clone(),
        D24.clone(),
        D25.clone(),
        D26.clone(),
        D27.clone(),
        D28.clone(),
        D29.clone(),
        D30.clone(),
        D31.clone()
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

pub fn all_regs() -> &'static LinkedHashMap<MuID, P<Value>> {
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

    for reg in CALLEE_SAVED_FPRs.iter() {
        if reg_id == reg.extract_ssa_id().unwrap() {
            return true;
        }
    }
    false
}

// TODO: Check that these numbers are reasonable (THEY ARE ONLY AN ESTIMATE)
use ast::inst::*;
pub fn estimate_insts_for_ir(inst: &Instruction) -> usize {
    use ast::inst::Instruction_::*;

    match inst.v {
        // simple
        BinOp(_, _, _)  => 1,
        BinOpWithStatus(_, _, _, _) => 2,
        CmpOp(_, _, _)  => 1,
        ConvOp{..}      => 1,

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
        PrintHex(_) => 10,
        ExnInstruction{ref inner, ..} => estimate_insts_for_ir(&inner)
    }
}


// Splits an integer immediate into four 16-bit segments (returns the least significant first)
pub fn split_aarch64_iimm(val: u64) -> (u16, u16, u16, u16) {
    (val as u16, (val >> 16) as u16, (val >> 32) as u16, (val >> 48) as u16)
}

// Trys to reduce the given floating point to an immediate u64 that can be used with MOVI
pub fn f64_to_aarch64_u64(val: f64) -> Option<u64> {
    use std::mem;
    // WARNING: this assumes a little endian representation
    let bytes: [u8; 8] = unsafe { mem::transmute(val) };

    // Check that each byte is all 1 or all 0
    for i in 0..7 {
        if bytes[i] != 0b11111111 || bytes[i] != 0 {
            return None;
        }
    }

    Some(unsafe {mem::transmute::<f64, u64>(val)})
}

// Check that the given floating point fits in 8 bits
pub fn is_valid_f32_imm(val: f32) -> bool {
    use std::mem;

    // returns true if val has the format:
    //       aBbbbbbc defgh000 00000000 00000000 (where B = !b)
    //index: FEDCBA98 76543210 FEDCBA98 76543210
    //                       1                 0

    let uval = unsafe { mem::transmute::<f32, u32>(val) };

    let b = get_bit(uval as u64, 0x19);

    get_bit(uval as u64, 0x1E) == !b &&
        ((uval & (0b11111 << 0x19)) == if b {0b11111 << 0x19} else {0}) &&
        ((uval & !(0b1111111111111 << 0x13)) == 0)
}

// Reduces the given floating point constant to 8-bits (if it won't loose precision, otherwise returns 0)
pub fn is_valid_f64_imm(val: f64) -> bool {
    use std::mem;

    // returns true if val has the format:
    //       aBbbbbbb bbcdefgh 00000000 00000000 00000000 00000000 00000000 00000000 (where B = !b)
    //index: FEDCBA98 76543210 FEDCBA98 76543210 FEDCBA98 76543210 FEDCBA98 76543210
    //                       3                 2                 1                 0

    let uval = unsafe { mem::transmute::<f64, u64>(val) };

    let b = (uval & (1 << 0x36)) != 0;

    ((uval & (1 << 0x3E)) != 0) == !b &&
        ((uval & (0b11111111 << 0x36)) == if b {0b11111111 << 0x36} else {0}) &&
        ((uval & !(0b1111111111111111 << 0x30)) == 0)

}

// Returns the 'ith bit of x
#[inline(always)]
pub fn get_bit(x: u64, i: usize) -> bool {
    (x & ((1 as u64) << i) ) != 0
}

// Returns true if val = A << S, from some A < 4096, and S = 0 or S = 12
pub fn is_valid_arithmetic_imm(val : u64) -> bool {

    val < 4096 || ((val & 0b111111111111) == 0 && val < (4096 << 12))
}

// aarch64 instructions only operate on 32 and 64-bit registers
// so a valid n bit logical immediate (where n < 32) can't be dirrectly used
// this function will replicate the bit pattern so that it can be used
// (the resulting value will be valid iff 'val' is valid, and the lower 'n' bits will equal val)
pub fn replicate_logical_imm(val : u64, n : usize) -> u64 {
    if n < 32 {
        let mut val = val;
        for i in 1..32/n {
            val |= val << i*n;
        }
        val
    } else {
        val
    }
}


// 'val' is a valid logical immediate if the binary value of ROR(val, r) matches the regular expresion
//      (0{k-x}1{x}){m/k}
//      for some r, k that divides N, 2 <= k <= n, and x with 0 < x < k
//      (note: 0 =< r < k);
pub fn is_valid_logical_imm(val : u64, n : usize) -> bool {
    // val should be an 'n' bit number
    debug_assert!(0 < n && n <= 64 && (n == 64 || (val < (1 << n))));
    debug_assert!(n.is_power_of_two());

    // all 0's and all 1's are invalid
    if val == 0 || val == bits_ones(n) {
        return false;
    }

    // find the rightmost '1' with '0' to the right
    let mut r = 0;
    while r < n {
        let current_bit = get_bit(val, r);
        let next_bit = get_bit(val, (r + n - 1) % n);
        if current_bit && !next_bit {
            break;
        }

        r += 1;
    }

    // rotate 'val' so that the MSB is a 0, and the LSB is a 1
    // (since there is a '0' to the right of val[start_index])
    let mut val = val.rotate_right(r as u32);

    // lower n bits ored with the upper n bits
    if n < 64 {
        val = (val & bits_ones(n)) | ((val & (bits_ones(n) << (64 - n))) >> (64 - n))
    }


    let mut x = 0; // number of '1's in a row
    while x < n {
        // found a '0' at position x, there must be x 1's to the right
        if !get_bit(val, x) {
            break;
        }
        x += 1;
    }

    let mut k = x + 1; // where the next '1' is
    while k < n {
        // found a '1'
        if get_bit(val, k) {
            break;
        }
        k += 1;
    }
    // Note: the above may not have found a 1, in which case k == n

    // note: k >= 2, since if k = 1, val = 1....1 (which we've already checked for)
    // check that k divides N
    if n % k != 0 {
        return false;
    }

    // Now we need to check that the pattern (0{k-x}1{x}) is repetead N/K times in val

    let k_mask = bits_ones(k);
    let val_0 = val & k_mask; // the first 'k' bits of val

    // for each N/k expected repitions of val_0 (except the first one_
    for i in 1..(n/k)-1 {
        if val_0 != ((val >> k*i) & k_mask) {
            return false; // val_0 dosen't repeat
        }
    }

    return true;
}

// Returns the value of 'val' truncated to 'size', interpreted as an unsigned integer
pub fn get_unsigned_value(val: u64, size: usize) -> u64 {
    (val & bits_ones(size)) as u64 // clears all but the lowest 'size' bits of val
}

// Returns the value of 'val' truncated to 'size', interpreted as a signed integer
pub fn get_signed_value(val: u64, size: usize) -> i64 {
    if size == 64 {
        val as i64
    } else {
        let negative = (val & (1 << (size - 1))) != 0;

        if negative {
            (val | (bits_ones(64-size) << size)) as i64 // set the highest '64 - size' bits of val
        } else {
            (val & bits_ones(size)) as i64 // clears all but the lowest 'size' bits of val
        }
    }
}

fn invert_condition_code(cond: &str) -> &'static str {
    match cond {
        "EQ" => "NE",
        "NE" => "EQ",

        "CC" => "CS",
        "CS" => "CV",

        "HS" => "LO",
        "LO" => "HS",

        "MI" => "PL",
        "PL" => "MI",

        "VS" => "VN",
        "VN" => "VS",

        "HI" => "LS",
        "LS" => "HI",

        "GE" => "LT",
        "LT" => "GE",

        "GT" => "LE",
        "LE" => "GT",

        "AL" | "NV" => panic!("AL and NV don't have inverses"),
        _ => panic!("Unrecognised condition code")
    }
}

// Returns the aarch64 condition codes corresponding to the given comparison op
// (the comparisoon is true when the logical or of these conditions is true)
fn get_condition_codes(op: op::CmpOp) -> Vec<&'static str> {
    match op {
        op::CmpOp::EQ  | op::CmpOp::FOEQ => vec!["EQ"],
        op::CmpOp::NE  | op::CmpOp::FUNE => vec!["NE"],
        op::CmpOp::SGT | op::CmpOp::FOGT => vec!["GT"],
        op::CmpOp::SGE | op::CmpOp::FOGE => vec!["GE"],
        op::CmpOp::SLT | op::CmpOp::FULT => vec!["LT"],
        op::CmpOp::SLE | op::CmpOp::FULE => vec!["LE"],
        op::CmpOp::UGT | op::CmpOp::FUGT => vec!["HI"],
        op::CmpOp::UGE | op::CmpOp::FUGE => vec!["HS"],
        op::CmpOp::ULE | op::CmpOp::FOLE => vec!["LS"],
        op::CmpOp::ULT | op::CmpOp::FOLT => vec!["LO"],
                         op::CmpOp::FUNO => vec!["VS"],
                         op::CmpOp::FORD => vec!["VC"],
                         op::CmpOp::FUEQ => vec!["EQ", "VS"],
                         op::CmpOp::FONE => vec!["MI", "GT"],

        // These need to be handeled specially
                         op::CmpOp::FFALSE => vec![],
                         op::CmpOp::FTRUE  => vec![],
    }
}

// if t is a homogenouse floating point aggregate
// (i.e. an array or struct where each element is the same floating-point type, and there are at most 4 elements)
// returns the number of elements, otherwise returns 0

fn hfa_length(t : P<MuType>) -> usize
{
    match t.v {
        MuType_::Struct(ref name) => {
            let read_lock = STRUCT_TAG_MAP.read().unwrap();
            let struc = read_lock.get(name).unwrap();
            let tys = struc.get_tys();
            if tys.len() < 1 || tys.len() > 4 {
                return 0;
            }

            let ref base = tys[0];
            match base.v {
                MuType_::Float | MuType_::Double => {
                    for i in 1..tys.len() - 1 {
                        if tys[i].v != base.v {
                            return 0;
                        }
                    }
                    return tys.len(); // All elements are the same type
                }
                _ => return 0,
            }


        }, // TODO: how do I extra the list of member-types from this??
        MuType_::Array(ref base, n) if n <= 4 => {
                match base.v {
                    MuType_::Float | MuType_::Double => n,
                    _ => 0
                }
            }
        _ => 0

    }
}

#[inline(always)]
// Returns the number that has 'n' 1's in a row (i.e. 2^n-1)
pub fn bits_ones(n: usize) -> u64 {
    if n == 64 { (-(1 as i64)) as u64 }
    else { (1 << n) - 1 }
}
// val is an unsigned multiple of n and val/n fits in 12 bits
#[inline(always)]
pub fn is_valid_immediate_offset(val: i64, n : usize) -> bool {
    use std;
    let n =  std::cmp::max(n, 8);
    val >= 0 && (val as u64) % (n as u64) == 0 && ((val as u64)/(n as u64) < (1 << 12))
}

#[inline(always)]
// Can be used to load or store a pair
pub fn is_valid_immediate_pair_offset(val: i64, n : usize) -> bool {
    use std;
    let n =  std::cmp::max(n, 8);
    (val as u64) % (n as u64) == 0  && ((val as u64)/(n as u64) < (1 << 7))
}

#[inline(always)]
pub fn is_valid_immediate_scale(val: u64, n : usize) -> bool { val == (n as u64) || val == 1 }

#[inline(always)]
pub fn is_valid_immediate_extension(val: u64) -> bool { val <= 4 }

// Rounds n to the next multiple of d
#[inline(always)]
pub fn round_up(n: usize, d: usize) -> usize { ((n + d - 1)/d)*d }

#[inline(always)]
// Log2, assumes value is a power of two
// TODO: Implement this more efficiently?
pub fn log2(val: u64) -> u64 {
    debug_assert!(val.is_power_of_two());
    /*debug_assert!(val != 0);
    let mut ret = 0;
    for i in 0..63 {
        if val & (1 << i) != 0 {
            ret = i;
        }
    }*/
    // WARNING: This will only work for val < 2^31
    let ret = (val as f64).log2() as u64;
    debug_assert!(val == 1 << ret);
    ret
}

// Gets a primitive integer type with the given alignment
pub fn get_alignment_type(align: usize) -> P<MuType> {
    match align {
        1 => UINT8_TYPE.clone(),
        2 => UINT16_TYPE.clone(),
        4 => UINT32_TYPE.clone(),
        8 => UINT64_TYPE.clone(),
        16 => UINT128_TYPE.clone(),
        _ => panic!("aarch64 dosn't have types with alignment {}", align)
    }
}