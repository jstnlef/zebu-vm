use ast::types::*;

pub enum MemoryOrder {
    NotAtomic,
    Relaxed,
    Consume,
    Acquire,
    Release,
    AcqRel,
    SeqCst
}

pub enum BinOp {
    // Int(n) BinOp Int(n) -> Int(n)
    Add,
    Sub,
    Mul,
    Sdiv,
    Srem,
    Udiv,
    And,
    Or,
    Xor,
        
    // Int(n) BinOp Int(m) -> Int(n)
    Shl,
    Lshr,
    AsHR,

    // FP BinOp FP -> FP
    Fadd,
    FSub,
    FMul,
    FDiv,
    FRem
}

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

pub enum Literal_ {
    LitIntDec(usize, MuType_),
    LitIntHex(usize, MuType_),
    
    LitFP(f64, MuType_),
    LitFPNaN(MuType_),
    LitFPInfPos(MuType_),
    LitFPInfNeg(MuType_),
    
    LitNull(MuType_)
}

pub struct SSAVar {
    id: MuID,
    tag:Option<MuTag>,
    ty: MuType_
}