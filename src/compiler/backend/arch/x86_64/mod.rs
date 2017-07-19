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

/// Tree pattern matching instruction selection.
pub mod inst_sel;

mod codegen;
/// CodeGenerator trait serves as an interface to the backend code generator, which
/// may generate assembly code or binary (not implemented yet)
use compiler::backend::x86_64::codegen::CodeGenerator;

/// assembly backend as AOT compiler
mod asm_backend;
use compiler::backend::x86_64::asm_backend::ASMCodeGen;

// re-export a few functions for AOT compilation
#[cfg(feature = "aot")]
pub use compiler::backend::x86_64::asm_backend::emit_code;
#[cfg(feature = "aot")]
pub use compiler::backend::x86_64::asm_backend::emit_context;
#[cfg(feature = "aot")]
pub use compiler::backend::x86_64::asm_backend::emit_context_with_reloc;
#[cfg(feature = "aot")]
pub use compiler::backend::x86_64::asm_backend::spill_rewrite;

use utils::Address;
use utils::ByteSize;
use ast::ptr::P;
use ast::ir::*;
use ast::types::*;
use compiler::backend::RegGroup;

use utils::LinkedHashMap;
use std::collections::HashMap;

// number of normal callee saved registers (excluding RSP and RBP)
pub const CALLEE_SAVED_COUNT: usize = 5;

/// a macro to declare a set of general purpose registers that are aliased to the first one
macro_rules! GPR_ALIAS {
    ($alias: ident: ($id64: expr, $r64: ident) ->
     $r32: ident, $r16: ident, $r8l: ident, $r8h: ident) => {
        lazy_static!{
            pub static ref $r64 : P<Value> = GPR!($id64,    stringify!($r64), UINT64_TYPE);
            pub static ref $r32 : P<Value> = GPR!($id64 +1, stringify!($r32), UINT32_TYPE);
            pub static ref $r16 : P<Value> = GPR!($id64 +2, stringify!($r16), UINT16_TYPE);
            pub static ref $r8l : P<Value> = GPR!($id64 +3, stringify!($r8l), UINT8_TYPE);
            pub static ref $r8h : P<Value> = GPR!($id64 +4, stringify!($r8h), UINT8_TYPE);

            pub static ref $alias : [P<Value>; 5] = [$r64.clone(), $r32.clone(), $r16.clone(),
                                                     $r8l.clone(), $r8h.clone()];
        }
    };

    ($alias: ident: ($id64: expr, $r64: ident) -> $r32: ident, $r16: ident, $r8: ident) => {
        lazy_static!{
            pub static ref $r64 : P<Value> = GPR!($id64,    stringify!($r64), UINT64_TYPE);
            pub static ref $r32 : P<Value> = GPR!($id64 +1, stringify!($r32), UINT32_TYPE);
            pub static ref $r16 : P<Value> = GPR!($id64 +2, stringify!($r16), UINT16_TYPE);
            pub static ref $r8  : P<Value> = GPR!($id64 +3, stringify!($r8) , UINT8_TYPE );

            pub static ref $alias : [P<Value>; 4] = [$r64.clone(), $r32.clone(),
                                                     $r16.clone(), $r8.clone()];
        }
    };

    ($alias: ident: ($id64: expr, $r64: ident)) => {
        lazy_static!{
            pub static ref $r64 : P<Value> = GPR!($id64, stringify!($r64), UINT64_TYPE);

            pub static ref $alias : [P<Value>; 4] = [$r64.clone(), $r64.clone(),
                                                     $r64.clone(), $r64.clone()];
        }
    };
}

/// a macro to declare a general purpose register
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

/// a macro to declare a floating point register
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

// declare all general purpose registers for x86_64
// non 64-bit registers are alias of its 64-bit one

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
    /// a map from 64-bit register IDs to a vector of its aliased register (Values),
    /// including the 64-bit register
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

    /// a map from any register to its 64-bit alias
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

/// returns P<Value> for a register ID of its alias of the given length
/// panics if the ID is not a machine register ID
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
        for r in ALL_FPRS.iter() {
            if r.id() == id {
                return r.clone();
            }
        }

        panic!("didnt find {} as FPR", id)
    }
}

/// are two registers aliased? (both must be machine register IDs, otherwise this function panics)
pub fn is_aliased(id1: MuID, id2: MuID) -> bool {
    if get_color_for_precolored(id1) == get_color_for_precolored(id2) {
        // we need to specially check the case for AH/BH/CH/DH
        // because both AH and AL are aliased to RAX, but AH and AL are not aliased
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
        false
    }
}

/// gets the color for a machine register (returns 64-bit alias for it)
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

/// returns register length (in bits) for an integer operand
#[inline(always)]
pub fn check_op_len(op: &P<Value>) -> usize {
    match op.ty.get_int_length() {
        Some(64) => 64,
        Some(32) => 32,
        Some(16) => 16,
        Some(8) => 8,
        Some(1) => 8,
        _ => panic!("unsupported register length for x64: {}", op.ty)
    }
}

lazy_static! {
    /// GPRs for returning values
    //  order matters
    pub static ref RETURN_GPRS : [P<Value>; 2] = [
        RAX.clone(),
        RDX.clone(),
    ];

    /// GPRs for passing arguments
    //  order matters
    pub static ref ARGUMENT_GPRS : [P<Value>; 6] = [
        RDI.clone(),
        RSI.clone(),
        RDX.clone(),
        RCX.clone(),
        R8.clone(),
        R9.clone()
    ];

    /// callee saved GPRs
    pub static ref CALLEE_SAVED_GPRS : [P<Value>; 6] = [
        RBX.clone(),
        RBP.clone(),
        R12.clone(),
        R13.clone(),
        R14.clone(),
        R15.clone()
    ];

    /// caller saved GPRs
    pub static ref CALLER_SAVED_GPRS : [P<Value>; 9] = [
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

    /// all the genral purpose registers
    //  FIXME: why RBP is commented out?
    static ref ALL_GPRS : [P<Value>; 15] = [
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

pub const FPR_ID_START: usize = 100;

lazy_static!{
    // floating point registers, we use SSE registers
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

    /// FPRs to return values
    //  order matters
    pub static ref RETURN_FPRS : [P<Value>; 2] = [
        XMM0.clone(),
        XMM1.clone()
    ];

    /// FPRs to pass arguments
    //  order matters
    pub static ref ARGUMENT_FPRS : [P<Value>; 8] = [
        XMM0.clone(),
        XMM1.clone(),
        XMM2.clone(),
        XMM3.clone(),
        XMM4.clone(),
        XMM5.clone(),
        XMM6.clone(),
        XMM7.clone()
    ];

    /// callee saved FPRs (none for x86_64)
    pub static ref CALLEE_SAVED_FPRS : [P<Value>; 0] = [];

    /// caller saved FPRs
    pub static ref CALLER_SAVED_FPRS : [P<Value>; 16] = [
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

    /// all the floating point registers
    static ref ALL_FPRS : [P<Value>; 16] = [
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
    /// a map for all the machine registers, from ID to P<Value>
    pub static ref ALL_MACHINE_REGS : LinkedHashMap<MuID, P<Value>> = {
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

    /// all the usable general purpose registers for reg allocator to assign
    //  order matters here (since register allocator will prioritize assigning temporaries
    //  to a register that appears early)
    //  we put caller saved regs first (they imposes no overhead if there is no call instruction)
    pub static ref ALL_USABLE_GPRS : Vec<P<Value>> = vec![
        // caller saved registers
        RAX.clone(),
        RCX.clone(),
        RDX.clone(),
        RSI.clone(),
        RDI.clone(),
        R8.clone(),
        R9.clone(),
        R10.clone(),
        R11.clone(),
        // callee saved registers
        RBX.clone(),
        R12.clone(),
        R13.clone(),
        R14.clone(),
        R15.clone(),
    ];

    /// all the usable floating point registers for reg allocator to assign
    //  order matters here (since register allocator will prioritize assigning temporaries
    //  to a register that appears early)
    //  we put caller saved regs first (they imposes no overhead if there is no call instruction)
    pub static ref ALL_USABLE_FPRS : Vec<P<Value>> = vec![
        // floating point registers
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

    /// all the usable registers for register allocators to assign
    //  order matters here (since register allocator will prioritize assigning temporaries
    //  to a register that appears early)
    //  we put caller saved regs first (they imposes no overhead if there is no call instruction)
    pub static ref ALL_USABLE_MACHINE_REGS : Vec<P<Value>> = {
        let mut ret = vec![];
        ret.extend_from_slice(&ALL_USABLE_GPRS);
        ret.extend_from_slice(&ALL_USABLE_FPRS);
        ret
    };
}

/// creates context for each machine register in FunctionContext
pub fn init_machine_regs_for_func(func_context: &mut FunctionContext) {
    for reg in ALL_MACHINE_REGS.values() {
        let reg_id = reg.extract_ssa_id().unwrap();
        let entry = SSAVarEntry::new(reg.clone());

        func_context.values.insert(reg_id, entry);
    }
}

/// gets the number of registers in a certain register group
pub fn number_of_usable_regs_in_group(group: RegGroup) -> usize {
    match group {
        RegGroup::GPR => ALL_USABLE_GPRS.len(),
        RegGroup::GPREX => ALL_USABLE_GPRS.len(),
        RegGroup::FPR => ALL_USABLE_FPRS.len()
    }
}

/// returns the number of all registers on this platform
pub fn number_of_all_regs() -> usize {
    ALL_MACHINE_REGS.len()
}

/// returns a reference to a map for all the registers
pub fn all_regs() -> &'static LinkedHashMap<MuID, P<Value>> {
    &ALL_MACHINE_REGS
}

/// returns a reference to a vector of all usable registers
pub fn all_usable_regs() -> &'static Vec<P<Value>> {
    &ALL_USABLE_MACHINE_REGS
}

/// returns RegGroup for a given machine register (by ID)
/// panics if the ID is not a machine register
pub fn pick_group_for_reg(reg_id: MuID) -> RegGroup {
    let reg = all_regs().get(&reg_id).unwrap();
    RegGroup::get_from_value(reg)
}

/// gets the previouse frame pointer with respect to the current
#[inline(always)]
pub fn get_previous_frame_pointer(frame_pointer: Address) -> Address {
    unsafe { frame_pointer.load::<Address>() }
}

/// gets the return address for the current frame pointer
#[inline(always)]
pub fn get_return_address(frame_pointer: Address) -> Address {
    unsafe { (frame_pointer + 8 as ByteSize).load::<Address>() }
}

/// gets the stack pointer before the current frame was created
#[inline(always)]
pub fn get_previous_stack_pointer(frame_pointer: Address, stack_arg_size: usize) -> Address {
    frame_pointer + 16 as ByteSize + stack_arg_size
}

/// sets the stack point
#[inline(always)]
pub fn set_previous_frame_pointer(frame_pointer: Address, value: Address) {
    unsafe { frame_pointer.store::<Address>(value) }
}

/// gets the return address for the current frame pointer
#[inline(always)]
pub fn set_return_address(frame_pointer: Address, value: Address) {
    unsafe { (frame_pointer + 8 as ByteSize).store::<Address>(value) }
}

/// returns offset of callee saved register
/// Reg should be a 64-bit callee saved GPR or FPR
pub fn get_callee_saved_offset(reg: MuID) -> isize {
    debug_assert!(is_callee_saved(reg) && reg != RBP.id());

    let id = if reg == RBX.id() {
        0
    } else {
        (reg - R12.id()) / 4 + 1
    };
    (id as isize + 1) * (-8)
}

/// is a machine register (by ID) callee saved?
/// returns false if the ID is not a machine register
pub fn is_callee_saved(reg_id: MuID) -> bool {
    for reg in CALLEE_SAVED_GPRS.iter() {
        if reg_id == reg.extract_ssa_id().unwrap() {
            return true;
        }
    }

    false
}

/// is a constant a valid x86_64 immediate number (32 bits integer)?
pub fn is_valid_x86_imm(op: &P<Value>) -> bool {
    use std::i32;

    if op.ty.get_int_length().is_some() && op.ty.get_int_length().unwrap() <= 32 {
        match op.v {
            Value_::Constant(Constant::Int(val))
                if val as i32 >= i32::MIN && val as i32 <= i32::MAX => true,
            _ => false
        }
    } else {
        false
    }
}

use ast::inst::*;

/// estimate the number of machine instruction for each IR instruction
pub fn estimate_insts_for_ir(inst: &Instruction) -> usize {
    use ast::inst::Instruction_::*;

    match inst.v {
        // simple
        BinOp(_, _, _) => 1,
        BinOpWithStatus(_, _, _, _) => 2,
        CmpOp(_, _, _) => 1,
        ConvOp { .. } => 0,

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

        // runtime call
        New(_) | NewHybrid(_, _) => 10,
        NewStack(_) | NewThread(_, _) | NewThreadExn(_, _) | NewFrameCursor(_) => 10,
        ThreadExit => 10,
        Throw(_) => 10,
        SwapStack { .. } => 10,
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
