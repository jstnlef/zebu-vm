use ast::ptr::P;
use ast::types::*;
use ast::inst::*;

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
    Binary(BinOp),
    Comparison(CmpOp),
    AtomicRMW(AtomicRMWOp),

    ExprCall,
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
    GetVarPartIRef
}

pub fn pick_op_code_for_ssa(ty: &P<MuType>) -> OpCode {
    use ast::types::MuType_::*;
    let a : &MuType_ = ty;
    match a {
        // currently use i64 for all ints
        &Int(_) => OpCode::RegI64,
        // currently do not differentiate float and double
        &Float
        | &Double => OpCode::RegFP,
        // ref and pointer types use RegI64
        &Ref(_)
        | &IRef(_)
        | &WeakRef(_)
        | &UPtr(_)
        | &ThreadRef
        | &StackRef
        | &Tagref64
        | &FuncRef(_)
        | &UFuncPtr(_) => OpCode::RegI64,
        // we are not supposed to have these as SSA
        &Struct(_)
        | &Array(_, _)
        | &Hybrid(_, _)
        | &Void => panic!("Not expecting {} as SSA", ty),
        // unimplemented
        &Vector(_, _) => unimplemented!()
    }
}

pub fn pick_op_code_for_const(ty: &P<MuType>) -> OpCode {
    use ast::types::MuType_::*;
    let a : &MuType_ = ty;
    match a {
        // currently use i64 for all ints
        &Int(_) => OpCode::IntImmI64,
        // currently do not differentiate float and double
        &Float
        | &Double => OpCode::FPImm,
        // ref and pointer types use RegI64
        &Ref(_)
        | &IRef(_)
        | &WeakRef(_)
        | &UPtr(_)
        | &ThreadRef
        | &StackRef
        | &Tagref64
        | &FuncRef(_)
        | &UFuncPtr(_) => OpCode::IntImmI64,
        // we are not supposed to have these as SSA
        &Struct(_)
        | &Array(_, _)
        | &Hybrid(_, _)
        | &Void => unimplemented!(),
        // unimplemented
        &Vector(_, _) => unimplemented!()
    }
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
    Ashr,

    // FP BinOp FP -> FP
    FAdd,
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

pub fn pick_op_code_for_inst(inst: &Instruction) -> OpCode {
    match inst.v {
        Instruction_::BinOp(op, _, _) => OpCode::Binary(op),
        Instruction_::CmpOp(op, _, _) => OpCode::Comparison(op),
        Instruction_::AtomicRMW{op, ..} => OpCode::AtomicRMW(op),
        Instruction_::ExprCall{..} => OpCode::ExprCall,
        Instruction_::Load{..} => OpCode::Load,
        Instruction_::Store{..} => OpCode::Store,
        Instruction_::CmpXchg{..} => OpCode::CmpXchg,
        Instruction_::New(_) => OpCode::New,
        Instruction_::AllocA(_) => OpCode::AllocA,
        Instruction_::NewHybrid(_, _) => OpCode::NewHybrid,
        Instruction_::AllocAHybrid(_, _) => OpCode::AllocAHybrid,
        Instruction_::NewStack(_) => OpCode::NewStack,
        Instruction_::NewThread(_, _) => OpCode::NewThread,
        Instruction_::NewThreadExn(_, _) => OpCode::NewThreadExn,
        Instruction_::NewFrameCursor(_) => OpCode::NewFrameCursor,
        Instruction_::GetIRef(_) => OpCode::GetIRef,
        Instruction_::GetFieldIRef{..} => OpCode::GetFieldIRef,
        Instruction_::GetElementIRef{..} => OpCode::GetElementIRef,
        Instruction_::ShiftIRef{..} => OpCode::ShiftIRef,
        Instruction_::GetVarPartIRef{..} => OpCode::GetVarPartIRef,
        Instruction_::Fence(_) => OpCode::Fence,
        Instruction_::Return(_) => OpCode::Return,
        Instruction_::ThreadExit => OpCode::ThreadExit,
        Instruction_::Throw(_) => OpCode::Throw,
        Instruction_::TailCall(_) => OpCode::TailCall,
        Instruction_::Branch1(_) => OpCode::Branch1,
        Instruction_::Branch2{..} => OpCode::Branch2,
        Instruction_::Watchpoint{..} => OpCode::Watchpoint,
        Instruction_::WPBranch{..} => OpCode::WPBranch,
        Instruction_::Call{..} => OpCode::Call,
        Instruction_::SwapStack{..} => OpCode::SwapStack,
        Instruction_::Switch{..} => OpCode::Switch,
        Instruction_::ExnInstruction{..} => OpCode::ExnInstruction
    }
}
