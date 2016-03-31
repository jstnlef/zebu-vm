#[derive(Copy, Clone, Debug)]
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
    Watchpoint,
    WPBranch,
    Call,
    SwapStack,
    Switch,
    ExnInstruction,
    
    // expression
    BinOp,
    CmpOp,
    ExprCall,
    Load,
    Store,
    CmpXchg,
    AtomicRMWOp,
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
    GetVarPartIRef
}

#[derive(Copy, Clone, Debug)]
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

#[derive(Copy, Clone, Debug)]
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

#[derive(Copy, Clone, Debug)]
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