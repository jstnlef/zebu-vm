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

            _ => self, // all other comparisons are reflexive
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
    pub fn is_signed(self) -> bool {
        use op::CmpOp::*;
        match self {
            SGE | SLT | SGT | SLE => true,
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