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

use ptr::P;
use types::*;
use inst::*;
use types::MuType_::*;

#[derive(Copy, Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
#[allow(non_camel_case_types)]
pub enum OpCode {
    // SSA
    RegI64,
    RegFP,

    // Constant
    IntImmI64,
    FPImm,

    // non-terminal
    Assign,
    Fence,

    //terminal
    Return,
    ThreadExit,
    Throw,
    TailCall,
    Branch1,
    Branch2,
    Select,
    Watchpoint,
    WPBranch,
    Call,
    CCall,
    SwapStack,
    Switch,
    ExnInstruction,

    // expression
    Binary(BinOp),
    BinaryWithStatus(BinOp),
    Comparison(CmpOp),
    Conversion(ConvOp),
    AtomicRMW(AtomicRMWOp),

    ExprCall,
    ExprCCall,
    Load,
    Store,
    CmpXchg,
    New,
    AllocA,
    NewHybrid,
    AllocAHybrid,
    NewStack,
    NewThread,
    NewThreadExn,
    NewFrameCursor,
    GetIRef,
    GetFieldIRef,
    GetElementIRef,
    ShiftIRef,
    GetVarPartIRef,

    CommonInst_GetThreadLocal,
    CommonInst_SetThreadLocal,
    CommonInst_Pin,
    CommonInst_Unpin,

    Move,
    PrintHex,
    SetRetval
}

pub fn pick_op_code_for_ssa(ty: &P<MuType>) -> OpCode {
    let a : &MuType = ty;
    match a.v {
        // currently use i64 for all ints
        Int(_) => OpCode::RegI64,
        // currently do not differentiate float and double
        Float
        | Double => OpCode::RegFP,
        // ref and pointer types use RegI64
        Ref(_)
        | IRef(_)
        | WeakRef(_)
        | UPtr(_)
        | ThreadRef
        | StackRef
        | Tagref64
        | FuncRef(_)
        | UFuncPtr(_) => OpCode::RegI64,
        // we are not supposed to have these as SSA
        Struct(_)
        | Array(_, _)
        | Hybrid(_)
        | Void => panic!("Not expecting {} as SSA", ty),
        // unimplemented
        Vector(_, _) => unimplemented!()
    }
}

pub fn pick_op_code_for_value(ty: &P<MuType>) -> OpCode {
    let a : &MuType = ty;
    match a.v {
        // currently use i64 for all ints
        Int(_) => OpCode::IntImmI64,
        // currently do not differentiate float and double
        Float
        | Double => OpCode::FPImm,
        // ref and pointer types use RegI64
        Ref(_)
        | IRef(_)
        | WeakRef(_)
        | UPtr(_)
        | ThreadRef
        | StackRef
        | Tagref64
        | FuncRef(_)
        | UFuncPtr(_) => OpCode::IntImmI64,
        // we are not supposed to have these as SSA
        Struct(_)
        | Array(_, _)
        | Hybrid(_)
        | Void => unimplemented!(),
        // unimplemented
        Vector(_, _) => unimplemented!()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub enum BinOp {
    // Int(n) BinOp Int(n) -> Int(n)
    Add,
    Sub,
    Mul,
    Sdiv,
    Srem,
    Udiv,
    Urem,
    And,
    Or,
    Xor,

    // Int(n) BinOp Int(m) -> Int(n)
    Shl,
    Lshr,
    Ashr,

    // FP BinOp FP -> FP
    FAdd,
    FSub,
    FMul,
    FDiv,
    FRem
}

#[derive(Copy, Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub enum CmpOp {
    // for Int comparison
    EQ,
    NE,
    SGE,
    SGT,
    SLE,
    SLT,
    UGE,
    UGT,
    ULE,
    ULT,

    // for FP comparison
    FFALSE,
    FTRUE,
    FOEQ,
    FOGT,
    FOGE,
    FOLT,
    FOLE,
    FONE,
    FORD,
    FUEQ,
    FUGT,
    FUGE,
    FULT,
    FULE,
    FUNE,
    FUNO
}

impl CmpOp {
    // Returns the CmpOp c, such that (a self b) is equivelent to (b c a)
    pub fn swap_operands(self) -> CmpOp {
        use op::CmpOp::*;
        match self {
            SGE => SLE,
            SLE => SGE,
            SGT => SLT,
            SLT => SGT,

            UGE => ULE,
            ULE => UGE,
            UGT => ULT,
            ULT => UGT,

            FOGE => FOLE,
            FOLE => FOGE,
            FOGT => FOLT,
            FOLT => FOGT,

            FUGE => FULE,
            FULE => FUGE,
            FUGT => FULT,
            FULT => FUGT,

            _ => self, // all other comparisons are symmetric
        }
    }
    pub fn invert(self) -> CmpOp {
        use op::CmpOp::*;
        match self {
            EQ => NE,
            NE => EQ,

            FOEQ => FUNE,
            FUNE => FOEQ,

            FUGE => FOLT,
            FOLT => FUGE,

            FUNO => FORD,
            FORD => FUNO,

            UGT => ULE,
            ULE => UGT,

            FUGT => FOLE,
            FOLE => FUGT,

            SGE => SLT,
            SLT => SGE,

            FOGE => FULT,
            FULT => FOGE,

            SGT => SLE,
            SLE => SGT,

            FOGT => FULE,
            FULE => FOGT,

            UGE => ULT,
            ULT => UGE,

            FUEQ => FONE,
            FONE => FUEQ,

            FFALSE => FTRUE,
            FTRUE => FFALSE,
        }
    }

    // gets the unsigned version of the comparison
    pub fn get_unsigned(self) -> CmpOp {
        use op::CmpOp::*;
        match self {
            SGE => UGE,
            SLT => ULT,
            SGT => UGT,
            SLE => ULE,
            _   => self,
        }
    }
    pub fn is_signed(self) -> bool {
        use op::CmpOp::*;
        match self {
            SGE | SLT | SGT | SLE => true,
            _ => false
        }
    }

    pub fn is_symmetric(self) -> bool {
        use op::CmpOp::*;
        match self {
            EQ | NE | FORD| FUNO| FUNE | FUEQ | FONE | FOEQ => true,
            _ => false
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub enum ConvOp {
    TRUNC,
    ZEXT,
    SEXT,
    FPTRUNC,
    FPEXT,
    FPTOUI,
    FPTOSI,
    UITOFP,
    SITOFP,
    BITCAST,
    REFCAST,
    PTRCAST
}

#[derive(Copy, Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub enum AtomicRMWOp {
    XCHG,
    ADD,
    SUB,
    AND,
    NAND,
    OR,
    XOR,
    MAX,
    MIN,
    UMAX,
    UMIN
}

pub fn is_int_cmp(op: CmpOp) -> bool {
    match op {
        CmpOp::EQ
        | CmpOp::NE
        | CmpOp::SGE
        | CmpOp::SGT
        | CmpOp::SLE
        | CmpOp::SLT
        | CmpOp::UGE
        | CmpOp::UGT
        | CmpOp::ULE
        | CmpOp::ULT => true,
        _ => false
    }
}

pub fn pick_op_code_for_inst(inst: &Instruction) -> OpCode {
    match inst.v {
        Instruction_::BinOp(op, _, _)               => OpCode::Binary(op),
        Instruction_::BinOpWithStatus(op, _, _, _)  => OpCode::BinaryWithStatus(op),
        Instruction_::CmpOp(op, _, _)               => OpCode::Comparison(op),
        Instruction_::ConvOp{operation, ..}         => OpCode::Conversion(operation),
        Instruction_::AtomicRMW{op, ..}             => OpCode::AtomicRMW(op),
        Instruction_::ExprCall{..}                  => OpCode::ExprCall,
        Instruction_::ExprCCall{..}                 => OpCode::ExprCCall,
        Instruction_::Load{..}                      => OpCode::Load,
        Instruction_::Store{..}                     => OpCode::Store,
        Instruction_::CmpXchg{..}                   => OpCode::CmpXchg,
        Instruction_::New(_)                        => OpCode::New,
        Instruction_::AllocA(_)                     => OpCode::AllocA,
        Instruction_::NewHybrid(_, _)               => OpCode::NewHybrid,
        Instruction_::AllocAHybrid(_, _)            => OpCode::AllocAHybrid,
        Instruction_::NewStack(_)                   => OpCode::NewStack,
        Instruction_::NewThread(_, _)               => OpCode::NewThread,
        Instruction_::NewThreadExn(_, _)            => OpCode::NewThreadExn,
        Instruction_::NewFrameCursor(_)             => OpCode::NewFrameCursor,
        Instruction_::GetIRef(_)                    => OpCode::GetIRef,
        Instruction_::GetFieldIRef{..}              => OpCode::GetFieldIRef,
        Instruction_::GetElementIRef{..}            => OpCode::GetElementIRef,
        Instruction_::ShiftIRef{..}                 => OpCode::ShiftIRef,
        Instruction_::GetVarPartIRef{..}            => OpCode::GetVarPartIRef,
        Instruction_::Fence(_)                      => OpCode::Fence,
        Instruction_::Return(_)                     => OpCode::Return,
        Instruction_::ThreadExit                    => OpCode::ThreadExit,
        Instruction_::Throw(_)                      => OpCode::Throw,
        Instruction_::TailCall(_)                   => OpCode::TailCall,
        Instruction_::Branch1(_)                    => OpCode::Branch1,
        Instruction_::Branch2{..}                   => OpCode::Branch2,
        Instruction_::Select{..}                    => OpCode::Select,
        Instruction_::Watchpoint{..}                => OpCode::Watchpoint,
        Instruction_::WPBranch{..}                  => OpCode::WPBranch,
        Instruction_::Call{..}                      => OpCode::Call,
        Instruction_::CCall{..}                     => OpCode::CCall,
        Instruction_::SwapStack{..}                 => OpCode::SwapStack,
        Instruction_::Switch{..}                    => OpCode::Switch,
        Instruction_::ExnInstruction{..}            => OpCode::ExnInstruction,
        Instruction_::CommonInst_GetThreadLocal     => OpCode::CommonInst_GetThreadLocal,
        Instruction_::CommonInst_SetThreadLocal(_)  => OpCode::CommonInst_SetThreadLocal,
        Instruction_::CommonInst_Pin(_)             => OpCode::CommonInst_Pin,
        Instruction_::CommonInst_Unpin(_)           => OpCode::CommonInst_Unpin,
        Instruction_::Move(_)                    => OpCode::Move,
        Instruction_::PrintHex(_)                => OpCode::PrintHex,
        Instruction_::SetRetval(_)               => OpCode::SetRetval
    }
}
