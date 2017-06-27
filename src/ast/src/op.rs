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

#[derive(Copy, Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub enum BinOp {
    // BinOp Int(n) Int(n) -> Int(n)
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

    // BinOp Int(n) Int(m) -> Int(n)
    Shl,
    Lshr,
    Ashr,

    // BinOp FP FP -> FP
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
    /// returns the CmpOp X for CmpOp Y, such that (a Y b) is equivalent to (b X a)
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

    /// returns the CmpOp X for CmpOp Y, such that (a Y b) is equivalent to NOT(a X b)
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

    pub fn is_int_cmp(self) -> bool {
        use op::CmpOp::*;
        match self {
            EQ
            | NE
            | SGE
            | SGT
            | SLE
            | SLT
            | UGE
            | UGT
            | ULE
            | ULT => true,
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