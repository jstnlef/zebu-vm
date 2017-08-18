// Copyright 2017 The Australian National University
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(dead_code)]

// TODO: CHECK THAT THE TYPE OF EVERY MEMORY LOCATION HAS THE CORRECT SIZE
// (the size should be size of the area in memory that it is referring to, and will indicate
// how much data any load/store instructions that uses it will operate on
// (so it should be [1], 8, 16, 32, 64, or 128 bits in size (when using emit_mem,
// it can have other sizes before this))

#![allow(non_upper_case_globals)]
// TODO: Move architecture independent codes in here, inst_sel and asm_backend to somewhere else...
pub mod inst_sel;

use utils::bit_utils::bits_ones;

mod codegen;
pub use compiler::backend::aarch64::codegen::CodeGenerator;

mod asm_backend;
pub use compiler::backend::aarch64::asm_backend::ASMCodeGen;
pub use compiler::backend::aarch64::asm_backend::emit_code;
pub use compiler::backend::aarch64::asm_backend::emit_context;
pub use compiler::backend::aarch64::asm_backend::emit_context_with_reloc;
use utils::Address;

#[cfg(feature = "aot")]
pub use compiler::backend::aarch64::asm_backend::spill_rewrite;

use ast::ptr::P;
use ast::ir::*;
use ast::types::*;
use ast::op;
use compiler::backend::RegGroup;
use vm::VM;

use utils::ByteSize;
use utils::math::align_up;
use utils::LinkedHashMap;
use std::collections::HashMap;

// Number of nromal callee saved registers (excluding FP and LR, and SP)
pub const CALLEE_SAVED_COUNT: usize = 18;

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
GPR_ALIAS!(SP_ALIAS: (62, SP)  -> WSP); // Special register(only some instructions can reference it)
GPR_ALIAS!(XZR_ALIAS: (64, XZR)  -> WZR); // Pseudo register, not to be used by register allocator

// Aliases
// Indirect result location register (points to a location in memory to write return values to)
ALIAS!(X8 -> XR);
// Intra proecdure call register 0
// (may be modified by the linker when executing BL/BLR instructions)
ALIAS!(X16 -> IP0);
// Intra proecdure call register 1
// (may be modified by the linker when executing BL/BLR instructions)
ALIAS!(X17 -> IP1);
// Platform Register (NEVER TOUCH THIS REGISTER (Unless you can prove Linux doesn't use it))
ALIAS!(X18 -> PR);
// Frame Pointer (can be used as a normal register when not calling or returning)
ALIAS!(X29 -> FP);
// Link Register (not supposed to be used for any other purpose)
ALIAS!(X30 -> LR);

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

        if length > 32 {
            vec[0].clone()
        } else {
            vec[1].clone()
        }
    } else {
        let vec = match FPR_ALIAS_TABLE.get(&id) {
            Some(vec) => vec,
            None => panic!("didnt find {} as FPR", id)
        };

        if length > 32 {
            vec[0].clone()
        } else {
            vec[1].clone()
        }
    }
}

pub fn is_aliased(id1: MuID, id2: MuID) -> bool {
    return id1 == id2 || (id1 < MACHINE_ID_END && id2 < MACHINE_ID_END && get_color_for_precolored(id1) == get_color_for_precolored(id2));
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
        Some(n) if n <= 32 => 32,
        Some(n) if n <= 64 => 64,
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
pub fn get_bit_size(ty: &P<MuType>, vm: &VM) -> usize {
    match ty.get_int_length() {
        Some(val) => val,
        None => {
            match ty.v {
                MuType_::Float => 32,
                MuType_::Double => 64,
                MuType_::Vector(ref t, n) => get_bit_size(t, vm) * n,
                MuType_::Array(ref t, n) => get_bit_size(t, vm) * n,
                MuType_::Void => 0,
                _ => vm.get_backend_type_size(ty.id()) * 8
            }
        }
    }
}

#[inline(always)]
pub fn get_type_alignment(ty: &P<MuType>, vm: &VM) -> usize {
    vm.get_backend_type_info(ty.id()).alignment
}

#[inline(always)]
pub fn primitive_byte_size(ty: &P<MuType>) -> usize {
    match ty.get_int_length() {
        Some(val) => (align_up(val, 8) / 8).next_power_of_two(),
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
    // Note: these are the same as the ARGUMENT_GPRS
    pub static ref RETURN_GPRS : [P<Value>; 8] = [
        X0.clone(),
        X1.clone(),
        X2.clone(),
        X3.clone(),
        X4.clone(),
        X5.clone(),
        X6.clone(),
        X7.clone()
    ];

    pub static ref ARGUMENT_GPRS : [P<Value>; 8] = [
        X0.clone(),
        X1.clone(),
        X2.clone(),
        X3.clone(),
        X4.clone(),
        X5.clone(),
        X6.clone(),
        X7.clone()
    ];

    pub static ref CALLEE_SAVED_GPRS : [P<Value>; 10] = [
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

        // Note: These two are technically CALLEE saved but need to be dealt with specially
        //X29.clone(), // Frame Pointer
        //X30.clone() // Link Register
    ];

    pub static ref CALLER_SAVED_GPRS : [P<Value>; 18] = [
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

    static ref ALL_GPRS : [P<Value>; 30] = [
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

pub const FPR_ID_START: usize = 100;

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
    // Same as ARGUMENT_FPRS
    pub static ref RETURN_FPRS : [P<Value>; 8] = [
        D0.clone(),
        D1.clone(),
        D2.clone(),
        D3.clone(),
        D4.clone(),
        D5.clone(),
        D6.clone(),
        D7.clone()
    ];

    pub static ref ARGUMENT_FPRS : [P<Value>; 8] = [
        D0.clone(),
        D1.clone(),
        D2.clone(),
        D3.clone(),
        D4.clone(),
        D5.clone(),
        D6.clone(),
        D7.clone(),
    ];

    pub static ref CALLEE_SAVED_FPRS : [P<Value>; 8] = [
        D8.clone(),
        D9.clone(),
        D10.clone(),
        D11.clone(),
        D12.clone(),
        D13.clone(),
        D14.clone(),
        D15.clone()
    ];

    pub static ref CALLER_SAVED_FPRS : [P<Value>; 24] = [
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

    static ref ALL_FPRS : [P<Value>; 32] = [
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
    pub static ref ALL_MACHINE_REGS : LinkedHashMap<MuID, P<Value>> = {
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

    pub static ref CALLEE_SAVED_REGS : [P<Value>; 18] = [
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

        // Note: These two are technically CALLEE saved but need to be dealt with specially
        //X29.clone(), // Frame Pointer
        //X30.clone() // Link Register

        D8.clone(),
        D9.clone(),
        D10.clone(),
        D11.clone(),
        D12.clone(),
        D13.clone(),
        D14.clone(),
        D15.clone()
    ];

    pub static ref CALLER_SAVED_REGS : [P<Value>; 42] = [
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

    pub static ref ALL_USABLE_GPRS : Vec<P<Value>> = vec![
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
    ];

    pub static ref ALL_USABLE_FPRS : Vec<P<Value>> = vec![
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
        D31.clone(),

        D8.clone(),
        D9.clone(),
        D10.clone(),
        D11.clone(),
        D12.clone(),
        D13.clone(),
        D14.clone(),
        D15.clone(),
    ];

    // put caller saved regs first (they imposes no overhead if there is no call instruction)
    pub static ref ALL_USABLE_MACHINE_REGS : Vec<P<Value>> = {
        let mut ret = vec![];
        ret.extend_from_slice(&ALL_USABLE_GPRS);
        ret.extend_from_slice(&ALL_USABLE_FPRS);
        ret
    };
}

pub fn init_machine_regs_for_func(func_context: &mut FunctionContext) {
    for reg in ALL_MACHINE_REGS.values() {
        let reg_id = reg.extract_ssa_id().unwrap();
        let entry = SSAVarEntry::new(reg.clone());

        func_context.values.insert(reg_id, entry);
    }
}

pub fn number_of_usable_regs_in_group(group: RegGroup) -> usize {
    match group {
        RegGroup::GPR => ALL_USABLE_GPRS.len(),
        RegGroup::FPR => ALL_USABLE_FPRS.len(),
        RegGroup::GPREX => unimplemented!()
    }
}

pub fn number_of_all_regs() -> usize {
    ALL_MACHINE_REGS.len()
}

pub fn all_regs() -> &'static LinkedHashMap<MuID, P<Value>> {
    &ALL_MACHINE_REGS
}

pub fn all_usable_regs() -> &'static Vec<P<Value>> {
    &ALL_USABLE_MACHINE_REGS
}

pub fn pick_group_for_reg(reg_id: MuID) -> RegGroup {
    let reg = all_regs().get(&reg_id).unwrap();
    if is_int_reg(&reg) {
        RegGroup::GPR
    } else if is_fp_reg(&reg) {
        RegGroup::FPR
    } else {
        panic!("expect a machine reg to be either a GPR or a FPR: {}", reg)
    }
}

// Gets the previouse frame pointer with respect to the current
#[inline(always)]
pub fn get_previous_frame_pointer(frame_pointer: Address) -> Address {
    unsafe { frame_pointer.load::<Address>() }
}

// Gets the return address for the current frame pointer
#[inline(always)]
pub fn get_return_address(frame_pointer: Address) -> Address {
    unsafe { (frame_pointer + 8 as ByteSize).load::<Address>() }
}

// Gets the stack pointer before the current frame was created
#[inline(always)]
pub fn get_previous_stack_pointer(frame_pointer: Address, stack_arg_size: usize) -> Address {
    frame_pointer + 16 as ByteSize + stack_arg_size
}

#[inline(always)]
pub fn set_previous_frame_pointer(frame_pointer: Address, value: Address) {
    unsafe { frame_pointer.store::<Address>(value) }
}

// Gets the return address for the current frame pointer
#[inline(always)]
pub fn set_return_address(frame_pointer: Address, value: Address) {
    unsafe { (frame_pointer + 8 as ByteSize).store::<Address>(value) }
}

// Reg should be a 64-bit callee saved GPR or FPR
pub fn get_callee_saved_offset(reg: MuID) -> isize {
    debug_assert!(is_callee_saved(reg));
    let id = if reg < FPR_ID_START {
        (reg - CALLEE_SAVED_GPRS[0].id()) / 2
    } else {
        (reg - CALLEE_SAVED_FPRS[0].id()) / 2 + CALLEE_SAVED_GPRS.len()
    };
    (id as isize + 1) * (-8)
}

// Returns the callee saved register with the id...
/*pub fn get_callee_saved_register(offset: isize) -> P<Value> {
    debug_assert!(offset <= -8 && (-offset) % 8 == 0);
    let id = ((offset/-8) - 1) as usize;
    if id < CALLEE_SAVED_GPRs.len() {
        CALLEE_SAVED_GPRs[id].clone()
    } else if id - CALLEE_SAVED_GPRs.len() < CALLEE_SAVED_FPRs.len() {
        CALLEE_SAVED_FPRs[id - CALLEE_SAVED_GPRs.len()].clone()
    } else {
        panic!("There is no callee saved register with id {}", offset)
    }
}*/

pub fn is_callee_saved(reg_id: MuID) -> bool {
    for reg in CALLEE_SAVED_GPRS.iter() {
        if reg_id == reg.extract_ssa_id().unwrap() {
            return true;
        }
    }

    for reg in CALLEE_SAVED_FPRS.iter() {
        if reg_id == reg.extract_ssa_id().unwrap() {
            return true;
        }
    }
    false
}

// The stack size needed for a call to the given function signature
pub fn call_stack_size(sig: P<MuFuncSig>, vm: &VM) -> usize {
    compute_argument_locations(&sig.ret_tys, &SP, 0, &vm).2
}
// TODO: Check that these numbers are reasonable (THEY ARE ONLY AN ESTIMATE)
use ast::inst::*;
pub fn estimate_insts_for_ir(inst: &Instruction) -> usize {
    use ast::inst::Instruction_::*;

    match inst.v {
        // simple
        BinOp(_, _, _) => 1,
        BinOpWithStatus(_, _, _, _) => 2,
        CmpOp(_, _, _) => 1,
        ConvOp { .. } => 1,

        // control flow
        Branch1(_) => 1,
        Branch2 { .. } => 1,
        Select { .. } => 2,
        Watchpoint { .. } => 1,
        WPBranch { .. } => 2,
        Switch { .. } => 3,

        // call
        ExprCall { .. } | ExprCCall { .. } | Call { .. } | CCall { .. } => 5,
        Return(_) => 1,
        TailCall(_) => 1,

        // memory access
        Load { .. } | Store { .. } => 1,
        CmpXchg { .. } => 1,
        AtomicRMW { .. } => 1,
        AllocA(_) => 1,
        AllocAHybrid(_, _) => 1,
        Fence(_) => 1,

        // memory addressing
        GetIRef(_) |
        GetFieldIRef { .. } |
        GetElementIRef { .. } |
        ShiftIRef { .. } |
        GetVarPartIRef { .. } => 0,

        // runtime
        New(_) | NewHybrid(_, _) => 10,
        NewStack(_) | NewThread(_, _) | NewThreadExn(_, _) | NewFrameCursor(_) => 10,
        ThreadExit => 10, CurrentStack => 10, KillStack(_) => 10,
        Throw(_) => 10,
        SwapStackExpr { .. } | SwapStackExc { .. } | SwapStackKill { .. } => 10,
        CommonInst_GetThreadLocal | CommonInst_SetThreadLocal(_) => 10,
        CommonInst_Pin(_) | CommonInst_Unpin(_) => 10,

        // others
        Move(_) => 0,
        PrintHex(_) => 10,
        SetRetval(_) => 10,
        ExnInstruction { ref inner, .. } => estimate_insts_for_ir(&inner),
        _ => unimplemented!()
    }
}


// Splits an integer immediate into four 16-bit segments (returns the least significant first)
pub fn split_aarch64_imm_u64(val: u64) -> (u16, u16, u16, u16) {
    (
        val as u16,
        (val >> 16) as u16,
        (val >> 32) as u16,
        (val >> 48) as u16
    )
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

    Some(unsafe { mem::transmute::<f64, u64>(val) })
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
        ((uval & (0b11111 << 0x19)) == if b { 0b11111 << 0x19 } else { 0 }) &&
        ((uval & !(0b1111111111111 << 0x13)) == 0)
}

// Reduces the given floating point constant to 8-bits
// (if it won't loose precision, otherwise returns 0)
pub fn is_valid_f64_imm(val: f64) -> bool {
    use std::mem;

    // returns true if val has the format:
    //       aBbbbbbb bbcdefgh 00000000 00000000 00000000 00000000 00000000 00000000 (where B = !b)
    //index: FEDCBA98 76543210 FEDCBA98 76543210 FEDCBA98 76543210 FEDCBA98 76543210
    //                       3                 2                 1                 0

    let uval = unsafe { mem::transmute::<f64, u64>(val) };

    let b = (uval & (1 << 0x36)) != 0;

    ((uval & (1 << 0x3E)) != 0) == !b &&
        ((uval & (0b11111111 << 0x36)) == if b { 0b11111111 << 0x36 } else { 0 }) &&
        ((uval & !(0b1111111111111111 << 0x30)) == 0)

}

// Returns the 'ith bit of x
#[inline(always)]
pub fn get_bit(x: u64, i: usize) -> bool {
    (x & ((1 as u64) << i)) != 0
}

// Returns true if val = A << S, from some 0 <= A < 4096, and S = 0 or S = 12
pub fn is_valid_arithmetic_imm(val: u64) -> bool {
    val < 4096 || ((val & 0b111111111111) == 0 && val < (4096 << 12))
}

// aarch64 instructions only operate on 32 and 64-bit registers
// so a valid n bit logical immediate (where n < 32) can't be dirrectly used
// this function will replicate the bit pattern so that it can be used
// (the resulting value will be valid iff 'val' is valid, and the lower 'n' bits will equal val)
pub fn replicate_logical_imm(val: u64, n: usize) -> u64 {
    let op_size = if n <= 32 { 32 } else { 64 };
    let mut val = val;
    for i in 1..op_size / n {
        val |= val << i * n;
    }
    val
}


// 'val' is a valid logical immediate if the binary value of ROR(val, r)
// matches the regular expression
//      (0{k-x}1{x}){m/k}
//      for some r, k that divides N, 2 <= k <= n, and x with 0 < x < k
//      (note: 0 =< r < k);
pub fn is_valid_logical_imm(val: u64, n: usize) -> bool {
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
    for i in 1..(n / k) {
        if val_0 != ((val >> (k * i)) & k_mask) {
            return false; // val_0 dosen't repeat
        }
    }

    return true;
}

// Returns the value of 'val' truncated to 'size', and then zero extended
pub fn get_unsigned_value(val: u64, size: usize) -> u64 {
    (val & bits_ones(size)) as u64 // clears all but the lowest 'size' bits of val
}

// Returns the value of 'val' truncated to 'size', and then sign extended
pub fn get_signed_value(val: u64, size: usize) -> i64 {
    if size == 64 {
        val as i64
    } else {
        let negative = (val & (1 << (size - 1))) != 0;

        if negative {
            (val | (bits_ones(64 - size) << size)) as i64 // set the highest '64 - size' bits of val
        } else {
            (val & bits_ones(size)) as i64 // clears all but the lowest 'size' bits of val
        }
    }
}

// Returns the value of 'val' truncated to 'size', treated as a negative number
// (i.e. the highest 64-size bits are set to 1)
pub fn get_negative_value(val: u64, size: usize) -> i64 {
    if size == 64 {
        val as i64
    } else {
        (val | (bits_ones(64 - size) << size)) as i64 // set the highest '64 - size' bits of val
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
        op::CmpOp::EQ | op::CmpOp::FOEQ => vec!["EQ"],
        op::CmpOp::NE | op::CmpOp::FUNE => vec!["NE"],
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
        op::CmpOp::FTRUE => vec![]
    }
}

// if t is a homogenouse floating point aggregate (i.e. an array or struct
// where each element is the same floating-point type, and there are at most 4 elements)
// returns the number of elements, otherwise returns 0

fn hfa_length(t: &P<MuType>) -> usize {
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
                _ => return 0
            }


        } // TODO: how do I extra the list of member-types from this??
        MuType_::Array(ref base, n) if n <= 4 => {
            match base.v {
                MuType_::Float | MuType_::Double => n,
                _ => 0
            }
        }
        _ => 0

    }
}

// val is an unsigned multiple of n and val/n fits in 12 bits
#[inline(always)]
pub fn is_valid_immediate_offset(val: i64, n: usize) -> bool {
    use std;
    let n_align = std::cmp::max(n, 8);
    if n <= 8 {
        (val >= -(1 << 8) && val < (1 << 8)) || // Valid 9 bit signed unscaled offset
            // Valid unsigned 12-bit scalled offset
            val >= 0 && (val as u64) % (n_align as u64) == 0 &&
                ((val as u64) / (n_align as u64) < (1 << 12))
    } else {
        // Will be using a load/store-pair
        // Is val a signed 7 bit multiple of n_align
        (val as u64) % (n_align as u64) == 0 && ((val as u64) / (n_align as u64) < (1 << 7))
    }
}

#[inline(always)]
pub fn is_valid_immediate_scale(val: u64, n: usize) -> bool {
    // if n > 8, then a load pair will be used, and they don't support scales
    n <= 8 && (val == (n as u64) || val == 1)
}

#[inline(always)]
pub fn is_valid_immediate_extension(val: u64) -> bool {
    val <= 4
}

#[inline(always)]
// Log2, assumes value is a power of two
// TODO: Implement this more efficiently?
pub fn log2(val: u64) -> u64 {
    debug_assert!(val.is_power_of_two());
    debug_assert!(val != 0);
    let mut ret = 0;
    for i in 0..63 {
        if val & (1 << i) != 0 {
            ret = i;
        }
    }
    // WARNING: This will only work for val < 2^31
    //let ret = (val as f64).log2() as u64;
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

#[inline(always)]
pub fn is_zero_register(val: &P<Value>) -> bool {
    is_zero_register_id(val.extract_ssa_id().unwrap())
}

#[inline(always)]
pub fn is_zero_register_id(id: MuID) -> bool {
    id == XZR.extract_ssa_id().unwrap() || id == WZR.extract_ssa_id().unwrap()
}

pub fn match_node_f32imm(op: &TreeNode) -> bool {
    match op.v {
        TreeNode_::Value(ref pv) => {
            match pv.v {
                Value_::Constant(Constant::Float(_)) => true,
                _ => false
            }
        }
        _ => false
    }
}

pub fn match_node_f64imm(op: &TreeNode) -> bool {
    match op.v {
        TreeNode_::Value(ref pv) => {
            match pv.v {
                Value_::Constant(Constant::Double(_)) => true,
                _ => false
            }
        }
        _ => false
    }
}

pub fn match_value_f64imm(op: &P<Value>) -> bool {
    match op.v {
        Value_::Constant(Constant::Double(_)) => true,
        _ => false
    }
}

pub fn match_value_f32imm(op: &P<Value>) -> bool {
    match op.v {
        Value_::Constant(Constant::Float(_)) => true,
        _ => false
    }
}

pub fn match_value_imm(op: &P<Value>) -> bool {
    match op.v {
        Value_::Constant(_) => true,
        _ => false
    }
}

pub fn match_value_int_imm(op: &P<Value>) -> bool {
    match op.v {
        Value_::Constant(Constant::Int(_)) => true,
        _ => false
    }
}
pub fn match_value_ref_imm(op: &P<Value>) -> bool {
    match op.v {
        Value_::Constant(Constant::NullRef) => true,
        _ => false
    }
}
pub fn match_node_value(op: &TreeNode) -> bool {
    match op.v {
        TreeNode_::Value(_) => true,
        _ => false
    }
}

pub fn get_node_value(op: &TreeNode) -> P<Value> {
    match op.v {
        TreeNode_::Value(ref pv) => pv.clone(),
        _ => panic!("Expected node with value")
    }
}

pub fn match_node_int_imm(op: &TreeNode) -> bool {
    match op.v {
        TreeNode_::Value(ref pv) => match_value_int_imm(pv),
        _ => false
    }
}

// The only valid ref immediate is a null ref
pub fn match_node_ref_imm(op: &TreeNode) -> bool {
    match op.v {
        TreeNode_::Value(ref pv) => match_value_ref_imm(pv),
        _ => false
    }
}

pub fn match_node_imm(op: &TreeNode) -> bool {
    match op.v {
        TreeNode_::Value(ref pv) => match_value_imm(pv),
        _ => false
    }
}

pub fn node_imm_to_u64(op: &TreeNode) -> u64 {
    match op.v {
        TreeNode_::Value(ref pv) => value_imm_to_u64(pv),
        _ => panic!("expected imm")
    }
}
pub fn node_imm_to_i64(op: &TreeNode, signed: bool) -> u64 {
    match op.v {
        TreeNode_::Value(ref pv) => value_imm_to_i64(pv, signed),
        _ => panic!("expected imm")
    }
}
pub fn node_imm_to_s64(op: &TreeNode) -> i64 {
    match op.v {
        TreeNode_::Value(ref pv) => value_imm_to_s64(pv),
        _ => panic!("expected imm")
    }
}

pub fn node_imm_to_f64(op: &TreeNode) -> f64 {
    match op.v {
        TreeNode_::Value(ref pv) => value_imm_to_f64(pv),
        _ => panic!("expected imm")
    }
}

pub fn node_imm_to_f32(op: &TreeNode) -> f32 {
    match op.v {
        TreeNode_::Value(ref pv) => value_imm_to_f32(pv),
        _ => panic!("expected imm")
    }
}

pub fn node_imm_to_value(op: &TreeNode) -> P<Value> {
    match op.v {
        TreeNode_::Value(ref pv) => pv.clone(),
        _ => panic!("expected imm")
    }
}

pub fn value_imm_to_f32(op: &P<Value>) -> f32 {
    match op.v {
        Value_::Constant(Constant::Float(val)) => val as f32,
        _ => panic!("expected imm float")
    }
}

pub fn value_imm_to_f64(op: &P<Value>) -> f64 {
    match op.v {
        Value_::Constant(Constant::Double(val)) => val as f64,
        _ => panic!("expected imm double")
    }
}

pub fn value_imm_to_u64(op: &P<Value>) -> u64 {
    match op.v {
        Value_::Constant(Constant::Int(val)) => {
            get_unsigned_value(val as u64, op.ty.get_int_length().unwrap())
        }
        Value_::Constant(Constant::NullRef) => 0,
        _ => panic!("expected imm int")
    }
}

pub fn value_imm_to_i64(op: &P<Value>, signed: bool) -> u64 {
    match op.v {
        Value_::Constant(Constant::Int(val)) => {
            if signed {
                get_signed_value(val as u64, op.ty.get_int_length().unwrap()) as u64
            } else {
                get_unsigned_value(val as u64, op.ty.get_int_length().unwrap())
            }
        }
        Value_::Constant(Constant::NullRef) => 0,
        _ => panic!("expected imm int")
    }
}

pub fn value_imm_to_s64(op: &P<Value>) -> i64 {
    match op.v {
        Value_::Constant(Constant::Int(val)) => {
            get_signed_value(val as u64, op.ty.get_int_length().unwrap())
        }
        Value_::Constant(Constant::NullRef) => 0,
        _ => panic!("expected imm int")
    }
}

pub fn make_value_int_const(val: u64, vm: &VM) -> P<Value> {
    P(Value {
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        ty: UINT64_TYPE.clone(),
        v: Value_::Constant(Constant::Int(val))
    })
}

// Replaces the zero register with a temporary whose value is zero (or returns the orignal register)
/* TODO use this function for the following arguments:

We can probabbly allow the zero register to be the second argument to an _ext function
(as the assembler will simply use the shifted-register encoding, which allows it)
add[,s1] // tirival
add_ext[d, s1]  // trivial
add_imm[d, s1] // trivial

adds[,s1 // not trivial (sets flags)
adds_ext[,s1]   // not trivial (sets flags)
adds_imm[, s2] // not trivial (sets flags)

sub_ext[d, s1]  // trivial
sub_imm[d, s1] // trivial

subs_ext[,s1]   // not trivial (sets flags)
subs_imm[, s2] // not trivial (sets flags)

and_imm[d] // trivial
eor_imm[d] // trivial
orr_imm[d] // trivial

cmn_ext[s1] // not trivial (sets flags)
cmn_imm[s1] // not trivial (sets flags)

cmp_ext[s1] // not trivial (sets flags)
cmp_imm[s1] // not trivial (sets flags)

(they are all (or did I miss some??) places that the SP can be used,
which takes up the encoding of the ZR
I believe the Zero register can be used in all other places that an integer register is expected
(BUT AM NOT CERTAIN)
*/

/*
Just insert this immediatly before each emit_XX where XX is one the above instructions,
and arg is the name of the argument that can't be the zero register (do so for each such argument)
let arg = replace_zero_register(backend, &arg, f_context, vm);
*/

pub fn replace_zero_register(
    backend: &mut CodeGenerator,
    val: &P<Value>,
    f_context: &mut FunctionContext,
    vm: &VM
) -> P<Value> {
    if is_zero_register(&val) {
        let temp = make_temporary(f_context, val.ty.clone(), vm);
        backend.emit_mov_imm(&temp, 0);
        temp
    } else {
        val.clone()
    }
}

pub fn make_temporary(f_context: &mut FunctionContext, ty: P<MuType>, vm: &VM) -> P<Value> {
    f_context.make_temporary(vm.next_id(), ty).clone_value()
}

fn emit_mov_f64(
    backend: &mut CodeGenerator,
    dest: &P<Value>,
    f_context: &mut FunctionContext,
    vm: &VM,
    val: f64
) {
    use std::mem;
    if val == 0.0 {
        backend.emit_fmov(&dest, &XZR);
    } else if is_valid_f64_imm(val) {
        backend.emit_fmov_imm(&dest, val as f32);
    } else {
        match f64_to_aarch64_u64(val) {
            Some(v) => {
                // Can use a MOVI to load the immediate
                backend.emit_movi(&dest, v);
            }
            None => {
                // Have to load a temporary GPR with the value first
                let tmp_int = make_temporary(f_context, UINT64_TYPE.clone(), vm);
                emit_mov_u64(
                    backend,
                    &tmp_int,
                    unsafe { mem::transmute::<f64, u64>(val) }
                );

                // then move it to an FPR
                backend.emit_fmov(&dest, &tmp_int);
            }
        }
    }
}

fn emit_mov_f32(
    backend: &mut CodeGenerator,
    dest: &P<Value>,
    f_context: &mut FunctionContext,
    vm: &VM,
    val: f32
) {
    use std::mem;
    if val == 0.0 {
        backend.emit_fmov(&dest, &WZR);
    } else if is_valid_f32_imm(val) {
        backend.emit_fmov_imm(&dest, val);
    } else {
        // Have to load a temporary GPR with the value first
        let tmp_int = make_temporary(f_context, UINT32_TYPE.clone(), vm);

        emit_mov_u64(backend, &tmp_int, unsafe {
            mem::transmute::<f32, u32>(val)
        } as u64);
        // then move it to an FPR
        backend.emit_fmov(&dest, &tmp_int);
    }
}

pub fn emit_mov_u64(backend: &mut CodeGenerator, dest: &P<Value>, val: u64) {
    let n = dest.ty.get_int_length().unwrap();
    let unsigned_value = get_unsigned_value(val, n);
    let negative_value = get_negative_value(val, n) as u64;
    // Can use one instruction
    if n <= 16 {
        backend.emit_movz(&dest, val as u16, 0);
    } else if unsigned_value == 0 {
        backend.emit_movz(&dest, 0, 0); // All zeros
    } else if negative_value == bits_ones(64) {
        backend.emit_movn(&dest, 0, 0); // All ones
    } else if val > 0xFF && is_valid_logical_imm(val, n) {
        // Value is more than 16 bits
        backend.emit_mov_imm(&dest, replicate_logical_imm(val, n));

    // Have to use more than one instruciton
    } else {
        // Otherwise emmit a sequences of MOVZ, MOVN and MOVK, where:
        //  MOVZ(dest, v, n) will set dest = (v << n)
        //  MOVN(dest, v, n) will set dest = !(v << n)
        //  MOVK(dest, v, n) will set dest = dest[63:16+n]:n:dest[(n-1):0];

        // How many halfowrds are all zeros
        let n_zeros = ((unsigned_value & bits_ones(16) == 0) as u64) +
            ((unsigned_value & (bits_ones(16) << 16) == 0) as u64) +
            ((unsigned_value & (bits_ones(16) << 32) == 0) as u64) +
            ((unsigned_value & (bits_ones(16) << 48) == 0) as u64);

        // How many halfowrds are all ones
        let n_ones = ((negative_value & bits_ones(16) == bits_ones(16)) as u64) +
            ((negative_value & (bits_ones(16) << 16) == (bits_ones(16) << 16)) as u64) +
            ((negative_value & (bits_ones(16) << 32) == (bits_ones(16) << 32)) as u64) +
            ((negative_value & (bits_ones(16) << 48) == (bits_ones(16) << 48)) as u64);


        let mut movzn = false; // whether a movz/movn has been emmited yet
        if n_ones > n_zeros {
            // It will take less instructions to use MOVN
            let (pv0, pv1, pv2, pv3) = split_aarch64_imm_u64(negative_value);

            if pv0 != bits_ones(16) as u16 {
                backend.emit_movn(&dest, !pv0, 0);
                movzn = true;
            }
            if pv1 != bits_ones(16) as u16 {
                if !movzn {
                    backend.emit_movn(&dest, !pv1, 16);
                    movzn = true;
                } else {
                    backend.emit_movk(&dest, pv1, 16);
                }
            }
            if pv2 != bits_ones(16) as u16 {
                if !movzn {
                    backend.emit_movn(&dest, !pv2, 32);
                    movzn = true;
                } else {
                    backend.emit_movk(&dest, pv2, 32);
                }
            }
            if pv3 != bits_ones(16) as u16 {
                if !movzn {
                    backend.emit_movn(&dest, pv3, 48);
                } else {
                    backend.emit_movk(&dest, pv3, 48);
                }
            }
        } else {
            // It will take less instructions to use MOVZ
            let (pv0, pv1, pv2, pv3) = split_aarch64_imm_u64(unsigned_value);

            if pv0 != 0 {
                backend.emit_movz(&dest, pv0, 0);
                movzn = true;
            }
            if pv1 != 0 {
                if !movzn {
                    backend.emit_movz(&dest, pv1, 16);
                    movzn = true;
                } else {
                    backend.emit_movk(&dest, pv1, 16);
                }
            }
            if pv2 != 0 {
                if !movzn {
                    backend.emit_movz(&dest, pv2, 32);
                    movzn = true;
                } else {
                    backend.emit_movk(&dest, pv2, 32);
                }
            }
            if pv3 != 0 {
                if !movzn {
                    backend.emit_movz(&dest, pv3, 48);
                } else {
                    backend.emit_movk(&dest, pv3, 48);
                }
            }
        }
    }
}

// TODO: Will this be correct if src is treated as signed (i think so...)
pub fn emit_mul_u64(backend: &mut CodeGenerator, dest: &P<Value>, src: &P<Value>, val: u64) {
    if val == 0 {
        // dest = 0
        backend.emit_mov_imm(&dest, 0);
    } else if val == 1 {
        // dest = src
        if dest.id() != src.id() {
            backend.emit_mov(&dest, &src);
        }
    } else if val.is_power_of_two() {
        // dest = src << log2(val)
        backend.emit_lsl_imm(&dest, &src, log2(val as u64) as u8);
    } else {
        // dest = src * val
        emit_mov_u64(backend, &dest, val as u64);
        backend.emit_mul(&dest, &src, &dest);
    }
}

// Decrement the register by an immediate value
fn emit_sub_u64(backend: &mut CodeGenerator, dest: &P<Value>, src: &P<Value>, val: u64) {
    if (val as i64) < 0 {
        emit_add_u64(backend, &dest, &src, (-(val as i64) as u64));
    } else if val == 0 {
        if dest.id() != src.id() {
            backend.emit_mov(&dest, &src);
        }
    } else if is_valid_arithmetic_imm(val) {
        let imm_shift = val > 4096;
        let imm_val = if imm_shift { val >> 12 } else { val };
        backend.emit_sub_imm(&dest, &src, imm_val as u16, imm_shift);
    } else {
        emit_mov_u64(backend, &dest, val);
        backend.emit_sub(&dest, &src, &dest);
    }
}

// Increment the register by an immediate value
fn emit_add_u64(backend: &mut CodeGenerator, dest: &P<Value>, src: &P<Value>, val: u64) {
    if (val as i64) < 0 {
        emit_sub_u64(backend, &dest, &src, (-(val as i64) as u64));
    } else if val == 0 {
        if dest.id() != src.id() {
            backend.emit_mov(&dest, &src);
        }
    } else if is_valid_arithmetic_imm(val) {
        let imm_shift = val > 4096;
        let imm_val = if imm_shift { val >> 12 } else { val };
        backend.emit_add_imm(&dest, &src, imm_val as u16, imm_shift);
    } else {
        emit_mov_u64(backend, &dest, val);
        backend.emit_add(&dest, &src, &dest);
    }
}

// dest = src1*val + src2
fn emit_madd_u64(
    backend: &mut CodeGenerator,
    dest: &P<Value>,
    src1: &P<Value>,
    val: u64,
    src2: &P<Value>
) {
    if val == 0 {
        // dest = src2
        backend.emit_mov(&dest, &src2);
    } else if val == 1 {
        // dest = src1 + src2
        backend.emit_add(&dest, &src1, &src2);
    } else if val == !0 {
        // dest = src2 - src1
        backend.emit_sub(&dest, &src2, &src1);
    } else if val.is_power_of_two() {
        let shift = log2(val as u64) as u8;
        // dest = src1 << log2(val) + src2
        if shift <= 4 {
            backend.emit_add_ext(&dest, &src2, &src1, false, shift);
        } else {
            backend.emit_lsl_imm(&dest, &src1, shift);
            backend.emit_add(&dest, &dest, &src2);
        }
    } else {
        // dest = src1 * val + src2
        emit_mov_u64(backend, &dest, val as u64);
        backend.emit_madd(&dest, &src1, &dest, &src2);
    }
}

// dest = src*val1 + val2
fn emit_madd_u64_u64(
    backend: &mut CodeGenerator,
    dest: &P<Value>,
    src: &P<Value>,
    f_context: &mut FunctionContext,
    vm: &VM,
    val1: u64,
    val2: u64
) {
    if val2 == 0 {
        // dest = src*val
        emit_mul_u64(backend, &dest, &src, val1);
    } else if val1 == 0 {
        // dest = val2
        emit_mov_u64(backend, &dest, val2);
    } else if val1 == 1 {
        // dest = src1 + val2
        emit_add_u64(backend, &dest, &src, val2);
    } else if val1 == !0 {
        // dest = val2 - src1
        emit_mov_u64(backend, &dest, val2);
        backend.emit_sub(&dest, &dest, &src);
    } else if val1.is_power_of_two() {
        let shift = log2(val1 as u64) as u8;
        // dest = src << log2(val1) + val2
        backend.emit_lsl_imm(&dest, &src, shift);
        emit_add_u64(backend, &dest, &dest, val2);
    } else {
        // dest = src * val1 + val2
        let tmp = make_temporary(f_context, src.ty.clone(), vm);
        emit_mov_u64(backend, &dest, val1 as u64);
        emit_mov_u64(backend, &tmp, val2 as u64);
        backend.emit_madd(&dest, &src, &dest, &tmp);
    }
}

// Compare register with value
fn emit_cmp_u64(
    backend: &mut CodeGenerator,
    src1: &P<Value>,
    f_context: &mut FunctionContext,
    vm: &VM,
    val: u64
) {
    if (val as i64) < 0 {
        emit_cmn_u64(backend, &src1, f_context, vm, (-(val as i64) as u64));
    } else if val == 0 {
        // Operation has no effect
    } else if is_valid_arithmetic_imm(val) {
        let imm_shift = val > 4096;
        let imm_val = if imm_shift { val >> 12 } else { val };
        backend.emit_cmp_imm(&src1, imm_val as u16, imm_shift);
    } else {
        let tmp = make_temporary(f_context, UINT64_TYPE.clone(), vm);
        emit_mov_u64(backend, &tmp, val);
        backend.emit_cmp(&src1, &tmp);
    }
}

// Compare register with value
fn emit_cmn_u64(
    backend: &mut CodeGenerator,
    src1: &P<Value>,
    f_context: &mut FunctionContext,
    vm: &VM,
    val: u64
) {
    if (val as i64) < 0 {
        emit_cmp_u64(backend, &src1, f_context, vm, (-(val as i64) as u64));
    } else if val == 0 {
        // Operation has no effect
    } else if is_valid_arithmetic_imm(val) {
        let imm_shift = val > 4096;
        let imm_val = if imm_shift { val >> 12 } else { val };
        backend.emit_cmn_imm(&src1, imm_val as u16, imm_shift);
    } else {
        let tmp = make_temporary(f_context, UINT64_TYPE.clone(), vm);
        emit_mov_u64(backend, &tmp, val);
        backend.emit_cmn(&src1, &tmp);
    }
}

// sign extends reg, to fit in a 32/64 bit register
fn emit_sext(backend: &mut CodeGenerator, reg: &P<Value>) {
    let nreg = check_op_len(&reg.ty); // The size of the aarch64 register
    let nmu = reg.ty.get_int_length().unwrap(); // the size of the Mu type

    // No need to sign extend the zero register
    if nreg > nmu && !is_zero_register(&reg) {
        backend.emit_sbfx(&reg, &reg, 0, nmu as u8);
    }
}

// zero extends reg, to fit in a 32/64 bit register
fn emit_zext(backend: &mut CodeGenerator, reg: &P<Value>) {
    let nreg = check_op_len(&reg.ty); // The size of the aarch64 register
    let nmu = reg.ty.get_int_length().unwrap(); // the size of the Mu type

    // No need to zero extend the zero register
    if nreg > nmu && !is_zero_register(&reg) {
        backend.emit_ubfx(&reg, &reg, 0, nmu as u8);
    }
}

// one extends reg, to fit in a 32/64 bit register
fn emit_oext(backend: &mut CodeGenerator, reg: &P<Value>) {
    let nreg = check_op_len(&reg.ty); // The size of the aarch64 register
    let nmu = reg.ty.get_int_length().unwrap(); // the size of the Mu type

    if nreg > nmu {
        if is_zero_register(&reg) {
            unimplemented!(); // Can't one extend the zero register
        }
        backend.emit_orr_imm(&reg, &reg, bits_ones(nreg - nmu) << nmu)
    }
}

// Masks 'src' so that it can be used to shift 'dest'
// Returns a register that should be used for the shift operand (may be dest or src)
fn emit_shift_mask<'b>(
    backend: &mut CodeGenerator,
    dest: &'b P<Value>,
    src: &'b P<Value>
) -> &'b P<Value> {
    let ndest = dest.ty.get_int_length().unwrap() as u64;

    if ndest != 32 && ndest != 64 {
        // Not a native integer size, need to mask
        backend.emit_and_imm(&dest, &src, ndest.next_power_of_two() - 1);
        &dest
    } else {
        &src
    }
}
// TODO: Deal with memory case
fn emit_reg_value(
    backend: &mut CodeGenerator,
    pv: &P<Value>,
    f_context: &mut FunctionContext,
    vm: &VM
) -> P<Value> {
    match pv.v {
        Value_::SSAVar(_) => pv.clone(),
        Value_::Constant(ref c) => {
            match c {
                &Constant::Int(val) => {
                    /*if val == 0 {
                        // TODO emit the zero register (NOTE: it can't be used by all instructions)
                        // Use the zero register (saves having to use a temporary)
                        get_alias_for_length(XZR.id(), get_bit_size(&pv.ty, vm))
                    } else {*/
                    let tmp = make_temporary(f_context, pv.ty.clone(), vm);
                    debug!("tmp's ty: {}", tmp.ty);
                    emit_mov_u64(backend, &tmp, val);
                    tmp
                    //}
                }
                &Constant::IntEx(ref val) => {
                    assert!(val.len() == 2);

                    let tmp = make_temporary(f_context, pv.ty.clone(), vm);
                    let (tmp_l, tmp_h) = split_int128(&tmp, f_context, vm);

                    emit_mov_u64(backend, &tmp_l, val[0]);
                    emit_mov_u64(backend, &tmp_h, val[1]);

                    tmp
                }
                &Constant::FuncRef(func_id) => {
                    let tmp = make_temporary(f_context, pv.ty.clone(), vm);

                    let mem =
                        make_value_symbolic(vm.get_name_for_func(func_id), true, &ADDRESS_TYPE, vm);
                    emit_calculate_address(backend, &tmp, &mem, vm);
                    tmp
                }
                &Constant::NullRef => {
                    let tmp = make_temporary(f_context, pv.ty.clone(), vm);
                    backend.emit_mov_imm(&tmp, 0);
                    tmp
                    //get_alias_for_length(XZR.id(), get_bit_size(&pv.ty, vm))
                }
                &Constant::Double(val) => {
                    let tmp = make_temporary(f_context, pv.ty.clone(), vm);
                    emit_mov_f64(backend, &tmp, f_context, vm, val);
                    tmp
                }
                &Constant::Float(val) => {
                    let tmp = make_temporary(f_context, pv.ty.clone(), vm);
                    emit_mov_f32(backend, &tmp, f_context, vm, val);
                    tmp
                }
                _ => panic!("expected fpreg or ireg")
            }
        }
        _ => panic!("expected fpreg or ireg")
    }
}

// TODO: Deal with memory case
pub fn emit_ireg_value(
    backend: &mut CodeGenerator,
    pv: &P<Value>,
    f_context: &mut FunctionContext,
    vm: &VM
) -> P<Value> {
    match pv.v {
        Value_::SSAVar(_) => pv.clone(),
        Value_::Constant(ref c) => {
            match c {
                &Constant::Int(val) => {
                    // TODO Deal with zero case
                    /*if val == 0 {
                        // TODO: Are there any (integer) instructions that can't use the Zero reg?
                        // Use the zero register (saves having to use a temporary)
                        get_alias_for_length(XZR.id(), get_bit_size(&pv.ty, vm))
                    } else {*/
                    let tmp = make_temporary(f_context, pv.ty.clone(), vm);
                    debug!("tmp's ty: {}", tmp.ty);
                    emit_mov_u64(backend, &tmp, val);
                    tmp
                    //}
                }
                &Constant::IntEx(ref val) => {
                    assert!(val.len() == 2);

                    let tmp = make_temporary(f_context, pv.ty.clone(), vm);
                    let (tmp_l, tmp_h) = split_int128(&tmp, f_context, vm);

                    emit_mov_u64(backend, &tmp_l, val[0]);
                    emit_mov_u64(backend, &tmp_h, val[1]);

                    tmp
                }
                &Constant::FuncRef(func_id) => {
                    let tmp = make_temporary(f_context, pv.ty.clone(), vm);

                    let mem =
                        make_value_symbolic(vm.get_name_for_func(func_id), true, &ADDRESS_TYPE, vm);
                    emit_calculate_address(backend, &tmp, &mem, vm);
                    tmp
                }
                &Constant::NullRef => {
                    let tmp = make_temporary(f_context, pv.ty.clone(), vm);
                    backend.emit_mov_imm(&tmp, 0);
                    tmp
                    //get_alias_for_length(XZR.id(), get_bit_size(&pv.ty, vm))
                }
                _ => panic!("expected ireg")
            }
        }
        _ => panic!("expected ireg")
    }
}

// TODO: Deal with memory case
fn emit_fpreg_value(
    backend: &mut CodeGenerator,
    pv: &P<Value>,
    f_context: &mut FunctionContext,
    vm: &VM
) -> P<Value> {
    match pv.v {
        Value_::SSAVar(_) => pv.clone(),
        Value_::Constant(Constant::Double(val)) => {
            let tmp = make_temporary(f_context, DOUBLE_TYPE.clone(), vm);
            emit_mov_f64(backend, &tmp, f_context, vm, val);
            tmp
        }
        Value_::Constant(Constant::Float(val)) => {
            let tmp = make_temporary(f_context, FLOAT_TYPE.clone(), vm);
            emit_mov_f32(backend, &tmp, f_context, vm, val);
            tmp
        }
        _ => panic!("expected fpreg")
    }
}

fn split_int128(
    int128: &P<Value>,
    f_context: &mut FunctionContext,
    vm: &VM
) -> (P<Value>, P<Value>) {
    if f_context.get_value(int128.id()).unwrap().has_split() {
        let vec = f_context
            .get_value(int128.id())
            .unwrap()
            .get_split()
            .as_ref()
            .unwrap();
        (vec[0].clone(), vec[1].clone())
    } else {
        let arg_l = make_temporary(f_context, UINT64_TYPE.clone(), vm);
        let arg_h = make_temporary(f_context, UINT64_TYPE.clone(), vm);

        f_context
            .get_value_mut(int128.id())
            .unwrap()
            .set_split(vec![arg_l.clone(), arg_h.clone()]);

        (arg_l, arg_h)
    }
}

pub fn emit_ireg_ex_value(
    backend: &mut CodeGenerator,
    pv: &P<Value>,
    f_context: &mut FunctionContext,
    vm: &VM
) -> (P<Value>, P<Value>) {
    match pv.v {
        Value_::SSAVar(_) => split_int128(pv, f_context, vm),
        Value_::Constant(Constant::IntEx(ref val)) => {
            assert!(val.len() == 2);

            let tmp_l = make_temporary(f_context, UINT64_TYPE.clone(), vm);
            let tmp_h = make_temporary(f_context, UINT64_TYPE.clone(), vm);

            emit_mov_u64(backend, &tmp_l, val[0]);
            emit_mov_u64(backend, &tmp_h, val[1]);

            (tmp_l, tmp_h)
        }
        _ => panic!("expected ireg_ex")
    }
}

pub fn emit_mem(
    backend: &mut CodeGenerator,
    pv: &P<Value>,
    alignment: usize,
    f_context: &mut FunctionContext,
    vm: &VM
) -> P<Value> {
    match pv.v {
        Value_::Memory(ref mem) => {
            match mem {
                &MemoryLocation::VirtualAddress{ref base, ref offset, scale, signed} => {
                    let mut shift = 0 as u8;
                    let offset =
                        if offset.is_some() {
                            let offset = offset.as_ref().unwrap();
                            if match_value_int_imm(offset) {
                                let mut offset_val = value_imm_to_i64(offset, signed) as i64;
                                offset_val *= scale as i64;

                                if is_valid_immediate_offset(offset_val, alignment) {
                                    Some(make_value_int_const(offset_val as u64, vm))
                                } else if alignment <= 8 {
                                    let offset = make_temporary(f_context, UINT64_TYPE.clone(), vm);
                                    emit_mov_u64(backend, &offset, offset_val as u64);
                                    Some(offset)
                                } else {
                                    // We will be using a store/load pair
                                    // which dosn't support register offsets
                                    return emit_mem_base(backend, &pv, f_context, vm);
                                }
                            } else {
                                let offset = emit_ireg_value(backend, offset, f_context, vm);

                                // TODO: If scale == (2^n)*m (for some m), set shift = n,
                                // and multiply index by m
                                if !is_valid_immediate_scale(scale, alignment) {
                                    let temp = make_temporary(f_context, offset.ty.clone(), vm);

                                    emit_mul_u64(backend, &temp, &offset, scale);
                                    Some(temp)
                                } else {
                                    shift = log2(scale) as u8;
                                    Some(offset)
                                }
                            }
                        }
                            else {
                                None
                            };

                    P(Value {
                        hdr: MuEntityHeader::unnamed(vm.next_id()),
                        ty: pv.ty.clone(),
                        v: Value_::Memory(MemoryLocation::Address {
                            base: base.clone(),
                            offset: offset,
                            shift: shift,
                            signed: signed
                        })
                    })
                }
                &MemoryLocation::Symbolic{is_global, ..} => {
                    if is_global {
                        let temp = make_temporary(f_context, pv.ty.clone(), vm);
                        emit_addr_sym(backend, &temp, &pv, vm);

                        P(Value {
                            hdr: MuEntityHeader::unnamed(vm.next_id()),
                            ty: pv.ty.clone(),
                            v: Value_::Memory(MemoryLocation::Address {
                                base: temp,
                                offset: None,
                                shift: 0,
                                signed: false,
                            })
                        })
                    } else {
                        pv.clone()
                    }
                }
                _ => pv.clone()
            }
        }
        _ => // Use the value as the base registers
            {
                let tmp_mem = make_value_base_offset(&pv, 0, &pv.ty, vm);
                emit_mem(backend, &tmp_mem, alignment, f_context, vm)
            }
    }
}

// Same as emit_mem except returns a memory location with only a base
// NOTE: This code duplicates allot of code in emit_mem and emit_calculate_address
fn emit_mem_base(
    backend: &mut CodeGenerator,
    pv: &P<Value>,
    f_context: &mut FunctionContext,
    vm: &VM
) -> P<Value> {
    match pv.v {
        Value_::Memory(ref mem) => {
            let base = match mem {
                &MemoryLocation::VirtualAddress{ref base, ref offset, scale, signed} => {
                    if offset.is_some() {
                        let offset = offset.as_ref().unwrap();
                        if match_value_int_imm(offset) {
                            let offset_val = value_imm_to_i64(offset, signed) as i64;
                            if offset_val == 0 {
                                base.clone() // trivial
                            } else {
                                let temp = make_temporary(f_context, pv.ty.clone(), vm);
                                emit_add_u64(backend, &temp, &base,
                                             (offset_val * scale as i64) as u64);
                                temp
                            }
                        } else {
                            let offset = emit_ireg_value(backend, offset, f_context, vm);

                            // TODO: If scale == r*m (for some 0 <= m <= 4), multiply offset by r
                            // then use and add_ext(,...,m)
                            if scale.is_power_of_two() &&
                                is_valid_immediate_extension(log2(scale)) {
                                let temp = make_temporary(f_context, pv.ty.clone(), vm);
                                // temp = base + offset << log2(scale)
                                backend.emit_add_ext(&temp, &base, &offset, signed,
                                                     log2(scale) as u8);
                                temp
                            } else {
                                let temp_offset = make_temporary(f_context, offset.ty.clone(), vm);

                                // temp_offset = offset * scale
                                emit_mul_u64(backend, &temp_offset, &offset, scale);

                                // Don't need to create a new register, just overwrite temp_offset
                                let temp = cast_value(&temp_offset, &pv.ty);
                                // Need to use add_ext, in case offset is 32-bits
                                backend.emit_add_ext(&temp, &base, &temp_offset, signed, 0);
                                temp
                            }
                        }
                    }
                        else {
                            base.clone() // trivial
                        }
                }
                &MemoryLocation::Address{ref base, ref offset, shift, signed} => {
                    if offset.is_some() {
                        let ref offset = offset.as_ref().unwrap();

                        if match_value_int_imm(&offset) {
                            let offset = value_imm_to_u64(&offset);
                            if offset == 0 {
                                // Offset is 0, it can be ignored
                                base.clone()
                            } else {
                                let temp = make_temporary(f_context, pv.ty.clone(), vm);
                                emit_add_u64(backend, &temp, &base, offset as u64);
                                temp
                            }
                        } else if RegGroup::get_from_value(&offset) == RegGroup::GPR &&
                            offset.is_reg() {
                            let temp = make_temporary(f_context, pv.ty.clone(), vm);
                            backend.emit_add_ext(&temp, &base, &offset, signed, shift);
                            temp
                        } else {
                            panic!("Offset should be an integer register or a constant")
                        }
                    } else {
                        // Simple base address
                        base.clone()
                    }
                }
                &MemoryLocation::Symbolic{..} => {
                    let temp = make_temporary(f_context, pv.ty.clone(), vm);
                    emit_addr_sym(backend, &temp, &pv, vm);
                    temp
                },
            };

            P(Value {
                hdr: MuEntityHeader::unnamed(vm.next_id()),
                ty: pv.ty.clone(),
                v: Value_::Memory(MemoryLocation::Address {
                    base: base.clone(),
                    offset: None,
                    shift: 0,
                    signed: false
                })
            })
        }
        _ => // Use the value as the base register
            {
                let tmp_mem = make_value_base_offset(&pv, 0, &pv.ty, vm);
                emit_mem_base(backend, &tmp_mem, f_context, vm)
            }
    }
}

// Sets 'dest' to the absolute address of the given global symbolic memory location
//WARNING: this assumes that the resulting assembly file is compiled with -fPIC
pub fn emit_addr_sym(backend: &mut CodeGenerator, dest: &P<Value>, src: &P<Value>, vm: &VM) {
    match src.v {
        Value_::Memory(ref mem) => {
            match mem {
                &MemoryLocation::Symbolic {
                    ref label,
                    is_global,
                    is_native
                } => {
                    if is_global {
                        // Set dest to be the page address of the entry for src in the GOT
                        backend.emit_adrp(&dest, &src);

                        // Note: The offset should always be a valid immediate offset
                        // as it is 12-bits
                        // (The same size as an immediate offset)
                        let offset = P(Value {
                            hdr: MuEntityHeader::unnamed(vm.next_id()),
                            ty: UINT64_TYPE.clone(),
                            v: Value_::Constant(Constant::ExternSym(if is_native {
                                format!("/*C*/:got_lo12:{}", label)
                            } else {
                                format!(":got_lo12:{}", mangle_name(label.clone()))
                            }))
                        });

                        // [dest + low 12 bits of the GOT entry for src]
                        let address_loc = P(Value {
                            hdr: MuEntityHeader::unnamed(vm.next_id()),
                            ty: ADDRESS_TYPE.clone(),
                            // should be ptr<src.ty>
                            v: Value_::Memory(MemoryLocation::Address {
                                base: dest.clone(),
                                offset: Some(offset),
                                shift: 0,
                                signed: false
                            })
                        });

                        // Load dest with the value in the GOT entry for src
                        backend.emit_ldr(&dest, &address_loc, false);
                    } else {
                        // Load 'dest' with the value of PC + the PC-offset of src
                        backend.emit_adr(&dest, &src);
                    }
                }
                _ => panic!("Expected symbolic memory location")
            }
        }
        _ => panic!("Expected memory value")
    }
}

fn emit_calculate_address(backend: &mut CodeGenerator, dest: &P<Value>, src: &P<Value>, vm: &VM) {
    match src.v {
        Value_::Memory(MemoryLocation::VirtualAddress {
            ref base,
            ref offset,
            scale,
            signed
        }) => {
            if offset.is_some() {
                let offset = offset.as_ref().unwrap();
                if match_value_int_imm(offset) {
                    emit_add_u64(
                        backend,
                        &dest,
                        &base,
                        ((value_imm_to_i64(offset, signed) as i64) * (scale as i64)) as u64
                    );
                } else {
                    // dest = offset * scale + base
                    emit_madd_u64(backend, &dest, &offset, scale as u64, &base);
                }
            } else {
                backend.emit_mov(&dest, &base)
            }
        }
        // offset(base,index,scale)
        Value_::Memory(MemoryLocation::Address {
            ref base,
            ref offset,
            shift,
            signed
        }) => {
            if offset.is_some() {
                let ref offset = offset.as_ref().unwrap();

                if match_value_int_imm(&offset) {
                    let offset = value_imm_to_u64(&offset);
                    if offset == 0 {
                        // Offset is 0, address calculation is trivial
                        backend.emit_mov(&dest, &base);
                    } else {
                        emit_add_u64(backend, &dest, &base, offset as u64);
                    }
                } else if is_int_reg(&offset) {
                    backend.emit_add_ext(&dest, &base, &offset, signed, shift);
                } else {
                    panic!("Offset should be an integer register or a constant")
                }
            } else {
                // Simple base address
                backend.emit_mov(&dest, &base);
            }
        }

        Value_::Memory(MemoryLocation::Symbolic { .. }) => {
            emit_addr_sym(backend, &dest, &src, vm);
        }
        _ => panic!("expect mem location as value")
    }
}

fn make_value_base_offset(base: &P<Value>, offset: i64, ty: &P<MuType>, vm: &VM) -> P<Value> {
    let mem = make_memory_location_base_offset(base, offset, vm);
    make_value_from_memory(mem, ty, vm)
}

fn make_value_from_memory(mem: MemoryLocation, ty: &P<MuType>, vm: &VM) -> P<Value> {
    P(Value {
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        ty: ty.clone(),
        v: Value_::Memory(mem)
    })
}

fn make_memory_location_base_offset(base: &P<Value>, offset: i64, vm: &VM) -> MemoryLocation {
    if offset == 0 {
        MemoryLocation::VirtualAddress {
            base: base.clone(),
            offset: None,
            scale: 1,
            signed: true
        }
    } else {
        MemoryLocation::VirtualAddress {
            base: base.clone(),
            offset: Some(make_value_int_const(offset as u64, vm)),
            scale: 1,
            signed: true
        }
    }
}

fn make_memory_location_base_offset_scale(
    base: &P<Value>,
    offset: &P<Value>,
    scale: u64,
    signed: bool
) -> MemoryLocation {
    MemoryLocation::VirtualAddress {
        base: base.clone(),
        offset: Some(offset.clone()),
        scale: scale,
        signed: signed
    }
}

// Returns a memory location that points to 'Base + offset*scale + more_offset'
fn memory_location_shift(
    backend: &mut CodeGenerator,
    mem: MemoryLocation,
    more_offset: i64,
    f_context: &mut FunctionContext,
    vm: &VM
) -> MemoryLocation {
    if more_offset == 0 {
        return mem; // No need to do anything
    }
    match mem {
        MemoryLocation::VirtualAddress {
            ref base,
            ref offset,
            scale,
            signed
        } => {
            let mut new_scale = 1;
            let new_offset = if offset.is_some() {
                let offset = offset.as_ref().unwrap();
                if match_value_int_imm(&offset) {
                    let offset = offset.extract_int_const().unwrap() * scale + (more_offset as u64);
                    make_value_int_const(offset as u64, vm)
                } else {
                    let offset = emit_ireg_value(backend, &offset, f_context, vm);
                    let temp = make_temporary(f_context, offset.ty.clone(), vm);

                    if more_offset % (scale as i64) == 0 {
                        // temp = offset + more_offset/scale
                        emit_add_u64(
                            backend,
                            &temp,
                            &offset,
                            (more_offset / (scale as i64)) as u64
                        );
                        new_scale = scale;
                    } else {
                        // temp = offset*scale + more_offset
                        emit_mul_u64(backend, &temp, &offset, scale);
                        emit_add_u64(backend, &temp, &temp, more_offset as u64);
                    }

                    temp
                }
            } else {
                make_value_int_const(more_offset as u64, vm)
            };

            // if offset was an immediate or more_offset % scale != 0:
            //      new_offset = offset*scale+more_offset
            //      new_scale = 1
            // otherwise:
            //      new_offset = offset + more_offset/scale
            //      new_scale = scale
            // Either way: (new_offset*new_scale) = offset*scale+more_offset
            MemoryLocation::VirtualAddress {
                base: base.clone(),
                offset: Some(new_offset),
                scale: new_scale,
                signed: signed
            }
        }
        _ => panic!("expected a VirtualAddress memory location")
    }
}

// Returns a memory location that points to 'Base + offset*scale + more_offset*new_scale'
fn memory_location_shift_scale(
    backend: &mut CodeGenerator,
    mem: MemoryLocation,
    more_offset: &P<Value>,
    new_scale: u64,
    f_context: &mut FunctionContext,
    vm: &VM
) -> MemoryLocation {
    if match_value_int_imm(&more_offset) {
        let more_offset = value_imm_to_s64(&more_offset);
        memory_location_shift(
            backend,
            mem,
            more_offset * (new_scale as i64),
            f_context,
            vm
        )
    } else {
        let mut new_scale = new_scale;
        match mem {
            MemoryLocation::VirtualAddress {
                ref base,
                ref offset,
                scale,
                signed
            } => {
                let offset = if offset.is_some() {
                    let offset = offset.as_ref().unwrap();
                    if match_value_int_imm(&offset) {
                        let temp = make_temporary(f_context, offset.ty.clone(), vm);
                        let offset_scaled =
                            (offset.extract_int_const().unwrap() as i64) * (scale as i64);
                        if offset_scaled % (new_scale as i64) == 0 {
                            emit_add_u64(
                                backend,
                                &temp,
                                &more_offset,
                                (offset_scaled / (new_scale as i64)) as u64
                            );
                        // new_scale*temp = (more_offset + (offset*scale)/new_scale)
                        //                = more_offset*new_scale + offset*scale
                        } else {
                            // temp = more_offset*new_scale + offset*scale
                            emit_mul_u64(backend, &temp, &more_offset, new_scale);
                            emit_add_u64(backend, &temp, &temp, offset_scaled as u64);
                            new_scale = 1;
                        }
                        temp
                    } else {
                        let offset = emit_ireg_value(backend, &offset, f_context, vm);
                        let temp = make_temporary(f_context, offset.ty.clone(), vm);

                        if new_scale == scale {
                            // just add the offsets
                            backend.emit_add_ext(&temp, &more_offset, &temp, signed, 0);
                        } else {
                            // temp = offset * scale
                            emit_mul_u64(backend, &temp, &offset, scale);

                            if new_scale.is_power_of_two() &&
                                is_valid_immediate_extension(log2(new_scale))
                            {
                                // temp = (offset * scale) + more_offset << log2(new_scale)
                                backend.emit_add_ext(
                                    &temp,
                                    &temp,
                                    &more_offset,
                                    signed,
                                    log2(new_scale) as u8
                                );
                            } else {
                                // temp_more = more_offset * new_scale
                                let temp_more = make_temporary(f_context, offset.ty.clone(), vm);
                                emit_mul_u64(backend, &temp_more, &more_offset, new_scale);

                                // temp = (offset * scale) + (more_offset * new_scale);
                                backend.emit_add_ext(&temp, &temp_more, &temp, signed, 0);
                            }

                            new_scale = 1;
                        }
                        temp
                    }
                } else {
                    more_offset.clone()
                };
                MemoryLocation::VirtualAddress {
                    base: base.clone(),
                    offset: Some(offset),
                    scale: new_scale,
                    signed: signed
                }
            }
            _ => panic!("expected a VirtualAddress memory location")
        }
    }
}

pub fn cast_value(val: &P<Value>, t: &P<MuType>) -> P<Value> {
    let to_size = check_op_len(&val.ty);
    let from_size = check_op_len(&t);
    if to_size == from_size {
        val.clone() // No need to cast
    } else {
        if is_machine_reg(val) {
            if from_size < to_size {
                // 64 bits to 32 bits
                get_register_from_id(val.id() + 1)
            } else {
                // 32 bits to 64 bits
                get_register_from_id(val.id() - 1)
            }
        } else {
            unsafe { val.as_type(t.clone()) }
        }
    }
}

fn make_value_symbolic(label: MuName, global: bool, ty: &P<MuType>, vm: &VM) -> P<Value> {
    P(Value {
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        ty: ty.clone(),
        v: Value_::Memory(MemoryLocation::Symbolic {
            label: label,
            is_global: global,
            is_native: false
        })
    })
}

fn emit_move_value_to_value(
    backend: &mut CodeGenerator,
    dest: &P<Value>,
    src: &P<Value>,
    f_context: &mut FunctionContext,
    vm: &VM
) {
    let ref src_ty = src.ty;
    if src_ty.is_scalar() && !src_ty.is_fp() {
        // 128-bit move
        if is_int_ex_reg(&dest) {
            let (dest_l, dest_h) = split_int128(dest, f_context, vm);
            if src.is_int_ex_const() {
                let val = src.extract_int_ex_const();
                assert!(val.len() == 2);

                emit_mov_u64(backend, &dest_l, val[0]);
                emit_mov_u64(backend, &dest_h, val[1]);
            } else if is_int_ex_reg(&src) {
                let (src_l, src_h) = split_int128(src, f_context, vm);

                backend.emit_mov(&dest_l, &src_l);
                backend.emit_mov(&dest_h, &src_h);
            } else if src.is_mem() {
                emit_load(backend, &dest, &src, f_context, vm);
            } else {
                panic!("unexpected gpr mov between {} -> {}", src, dest);
            }
        } else if is_int_reg(&dest) {
            // gpr mov
            if src.is_int_const() {
                let imm = value_imm_to_u64(src);
                emit_mov_u64(backend, dest, imm);
            } else if is_int_reg(&src) {
                backend.emit_mov(dest, src);
            } else if src.is_mem() {
                emit_load(backend, &dest, &src, f_context, vm);
            } else {
                panic!("unexpected gpr mov between {} -> {}", src, dest);
            }
        } else if dest.is_mem() {
            let temp = emit_ireg_value(backend, src, f_context, vm);
            emit_store(backend, dest, &temp, f_context, vm);
        } else {
            panic!("unexpected gpr mov between {} -> {}", src, dest);
        }
    } else if src_ty.is_scalar() && src_ty.is_fp() {
        // fpr mov
        if is_fp_reg(&dest) {
            if match_value_f32imm(src) {
                let src = value_imm_to_f32(src);
                emit_mov_f32(backend, dest, f_context, vm, src);
            } else if match_value_f64imm(src) {
                let src = value_imm_to_f64(src);
                emit_mov_f64(backend, dest, f_context, vm, src);
            } else if is_fp_reg(&src) {
                backend.emit_fmov(dest, src);
            } else if src.is_mem() {
                emit_load(backend, &dest, &src, f_context, vm);
            } else {
                panic!("unexpected gpr mov between {} -> {}", src, dest);
            }
        } else if dest.is_mem() {
            let temp = emit_fpreg_value(backend, src, f_context, vm);
            emit_store(backend, dest, &temp, f_context, vm);
        } else {
            panic!("unexpected fpr mov between {} -> {}", src, dest);
        }
    } else {
        panic!("unexpected mov of type {}", src_ty)
    }
}

fn emit_load(
    backend: &mut CodeGenerator,
    dest: &P<Value>,
    src: &P<Value>,
    f_context: &mut FunctionContext,
    vm: &VM
) {
    let src = emit_mem(
        backend,
        &src,
        get_type_alignment(&dest.ty, vm),
        f_context,
        vm
    );
    if is_int_reg(dest) || is_fp_reg(dest) {
        backend.emit_ldr(&dest, &src, false);
    } else if is_int_ex_reg(dest) {
        let (dest_l, dest_h) = emit_ireg_ex_value(backend, dest, f_context, vm);
        backend.emit_ldp(&dest_l, &dest_h, &src);
    } else {
        unimplemented!();
    }

}

fn emit_store(
    backend: &mut CodeGenerator,
    dest: &P<Value>,
    src: &P<Value>,
    f_context: &mut FunctionContext,
    vm: &VM
) {
    let dest = emit_mem(
        backend,
        &dest,
        get_type_alignment(&src.ty, vm),
        f_context,
        vm
    );
    if is_int_reg(src) || is_fp_reg(src) {
        backend.emit_str(&dest, &src);
    } else if is_int_ex_reg(src) {
        let (src_l, src_h) = emit_ireg_ex_value(backend, src, f_context, vm);
        backend.emit_stp(&dest, &src_l, &src_h);
    } else {
        unimplemented!();
    }
}

fn emit_load_base_offset(
    backend: &mut CodeGenerator,
    dest: &P<Value>,
    base: &P<Value>,
    offset: i64,
    f_context: &mut FunctionContext,
    vm: &VM
) -> P<Value> {
    let mem = make_value_base_offset(base, offset, &dest.ty, vm);
    emit_load(backend, dest, &mem, f_context, vm);
    mem
}

fn emit_store_base_offset(
    backend: &mut CodeGenerator,
    base: &P<Value>,
    offset: i64,
    src: &P<Value>,
    f_context: &mut FunctionContext,
    vm: &VM
) {
    let mem = make_value_base_offset(base, offset, &src.ty, vm);
    emit_store(backend, &mem, src, f_context, vm);
}


fn is_int_reg(val: &P<Value>) -> bool {
    RegGroup::get_from_value(&val) == RegGroup::GPR && (val.is_reg() || val.is_const())
}
fn is_int_ex_reg(val: &P<Value>) -> bool {
    RegGroup::get_from_value(&val) == RegGroup::GPREX && (val.is_reg() || val.is_const())
}
fn is_fp_reg(val: &P<Value>) -> bool {
    RegGroup::get_from_value(&val) == RegGroup::FPR && (val.is_reg() || val.is_const())
}

// TODO: Thoroughly test this
// (compare with code generated by GCC with variouse different types???)
// The algorithm presented here is derived from the ARM AAPCS64 reference
// Returns a vector indicating whether each should be passed as an IRef (and not directly),
// a vector referencing to the location of each argument (in memory or a register) and
// the amount of stack space used
// NOTE: It currently does not support vectors/SIMD types (or aggregates of such types)
fn compute_argument_locations(
    arg_types: &Vec<P<MuType>>,
    stack: &P<Value>,
    offset: i64,
    vm: &VM
) -> (Vec<bool>, Vec<P<Value>>, usize) {
    if arg_types.len() == 0 {
        // nothing to do
        return (vec![], vec![], 0);
    }

    let mut ngrn = 0 as usize; // The Next General-purpose Register Number
    let mut nsrn = 0 as usize; // The Next SIMD and Floating-point Register Number
    let mut nsaa = 0 as usize; // The next stacked argument address (an offset from the SP)
    use ast::types::MuType_::*;

    // reference[i] = true indicates the argument is passed an IRef to a location on the stack
    let mut reference: Vec<bool> = vec![];
    for t in arg_types {
        reference.push(
            hfa_length(t) == 0 && // HFA's aren't converted to IRef's
                match t.v {
                    // size can't be statically determined
                    Hybrid(_) => panic!("Hybrid argument not supported"),
                    //  type is too large
                    Struct(_) | Array(_, _) if vm.get_backend_type_size(t.id()) > 16 => true,
                    Vector(_, _)  => unimplemented!(),
                    _ => false
                }
        );
    }
    // TODO: How does passing arguments by reference effect the stack size??
    let mut locations: Vec<P<Value>> = vec![];
    for i in 0..arg_types.len() {
        let i = i as usize;
        let t = if reference[i] {
            P(MuType::new(
                new_internal_id(),
                MuType_::IRef(arg_types[i].clone())
            ))
        } else {
            arg_types[i].clone()
        };
        let size = align_up(vm.get_backend_type_size(t.id()), 8);
        let align = get_type_alignment(&t, vm);
        match t.v {
            Hybrid(_) => panic!("hybrid argument not supported"),

            Vector(_, _) => unimplemented!(),
            Float | Double => {
                if nsrn < 8 {
                    locations.push(get_alias_for_length(
                        ARGUMENT_FPRS[nsrn].id(),
                        get_bit_size(&t, vm)
                    ));
                    nsrn += 1;
                } else {
                    nsrn = 8;
                    locations.push(make_value_base_offset(
                        &stack,
                        offset + (nsaa as i64),
                        &t,
                        vm
                    ));
                    nsaa += size;
                }
            }
            Struct(_) | Array(_, _) => {
                let hfa_n = hfa_length(&t);
                if hfa_n > 0 {
                    if nsrn + hfa_n <= 8 {
                        // Note: the argument will occupy succesiv registers
                        // (one for each element)
                        locations.push(get_alias_for_length(
                            ARGUMENT_FPRS[nsrn].id(),
                            get_bit_size(&t, vm) / hfa_n
                        ));
                        nsrn += hfa_n;
                    } else {
                        nsrn = 8;
                        locations.push(make_value_base_offset(
                            &stack,
                            offset + (nsaa as i64),
                            &t,
                            vm
                        ));
                        nsaa += size;
                    }
                } else {
                    if align == 16 {
                        ngrn = align_up(ngrn, 2); // align NGRN to the next even number
                    }

                    if size <= 8 * (8 - ngrn) {
                        // The struct should be packed, starting here
                        // (note: this may result in multiple struct fields in the same regsiter
                        // or even floating points in a GPR)
                        locations.push(ARGUMENT_GPRS[ngrn].clone());
                        // How many GPRS are taken up by t
                        ngrn += if size % 8 != 0 {
                            size / 8 + 1
                        } else {
                            size / 8
                        };
                    } else {
                        ngrn = 8;
                        nsaa = align_up(nsaa, align_up(align, 8));
                        locations.push(make_value_base_offset(
                            &stack,
                            offset + (nsaa as i64) as i64,
                            &t,
                            vm
                        ));
                        nsaa += size;
                    }
                }
            }

            Void => panic!("void argument not supported"),

            // Integral or pointer type
            _ => {
                if size <= 8 {
                    if ngrn < 8 {
                        locations.push(get_alias_for_length(
                            ARGUMENT_GPRS[ngrn].id(),
                            get_bit_size(&t, vm)
                        ));
                        ngrn += 1;
                    } else {
                        nsaa = align_up(nsaa, align_up(align, 8));
                        locations.push(make_value_base_offset(
                            &stack,
                            offset + (nsaa as i64) as i64,
                            &t,
                            vm
                        ));
                        nsaa += size;
                    }

                } else if size == 16 {
                    ngrn = align_up(ngrn, 2); // align NGRN to the next even number

                    if ngrn < 7 {
                        locations.push(ARGUMENT_GPRS[ngrn].clone());
                        ngrn += 2;
                    } else {
                        ngrn = 8;
                        nsaa = align_up(nsaa, 16);
                        locations.push(make_value_base_offset(
                            &stack,
                            offset + (nsaa as i64) as i64,
                            &t,
                            vm
                        ));
                        nsaa += 16;
                    }
                } else {
                    unimplemented!(); // Integer type is too large
                }
            }
        }
    }

    (reference, locations, align_up(nsaa, 16) as usize)
}
