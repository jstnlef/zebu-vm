#![allow(dead_code)]
#![allow(unused_variables)]

use ast::ptr::P;
use ast::types::*;

pub type WPID  = usize;
pub type MuID  = usize;
pub type MuTag = &'static str;
pub type Address = usize; // TODO: replace this with Address(usize)

#[derive(Clone)]
pub struct SSAVar {
    pub id: MuID,
    pub tag: MuTag,
    pub ty: P<MuType_>
}

#[derive(Clone)]
pub enum Value {
    SSAVar(SSAVar),
    Constant(MuConstant)
}

#[derive(Copy, Clone)]
pub enum MemoryOrder {
    NotAtomic,
    Relaxed,
    Consume,
    Acquire,
    Release,
    AcqRel,
    SeqCst
}

#[derive(Copy, Clone)]
pub enum CallConvention {
    Mu,
    Foreign(ForeignFFI)
}

#[derive(Copy, Clone)]
pub enum ForeignFFI {
    C
}

pub struct CallData {
    pub func: P<SSAVar>,
    pub args: Vec<P<Value>>,
    pub convention: CallConvention
}

pub struct Block {
    label: MuTag,
    content: Option<BlockContent>
}

impl Block {
    pub fn new(label: MuTag) -> Block {
        Block{label: label, content: None}
    }
    
    pub fn set_content(&mut self, v: BlockContent) {
        self.content = Some(v);
    }
}

pub struct BlockContent {
    pub args: Vec<P<Value>>,
    pub body: Vec<Instruction>,
    pub exit: Terminal,
    pub keepalives: Option<Vec<P<SSAVar>>>    
}

pub struct ResumptionData {
    pub normal_dest: Destination,
    pub exn_dest: Destination
}

pub enum DestArg {
    Normal(P<Value>),
    Freshbound(usize)
}

pub struct Destination {
    pub target: MuTag,
    pub args: Vec<DestArg>
}

#[derive(Clone)]
pub enum Constant {
    Int(usize, usize),
    IRef(P<MuType_>, Address),
    FloatV(f32),
    DoubleV(f64),
    VectorV(Vec<Constant>),
    FuncRefV(Address),
    UFuncRefV(Address)
}

pub enum Expression {
    BinOp(BinOp, P<Value>, P<Value>), 
    CmpOp(CmpOp, P<Value>, P<Value>),
    
    // yields the constant value
    Constant(P<Constant>),
    
    // yields a tuple of results from the call
    ExprCall{
        data: CallData,
        is_abort: bool, // T to abort, F to rethrow
    },
    
    // yields the memory value
    Load{
        is_iref: bool,
        mem_loc: P<Value>,
        order: MemoryOrder
    },
    
    // yields nothing
    Store{
        is_iref: bool,
        mem_loc: P<Value>,
        order: MemoryOrder        
    },
    
    // yields pair (oldvalue, boolean (T = success, F = failure))
    CmpXchg{
        is_iref: bool, // T for iref, F for ptr
        is_strong: bool,
        success_order: MemoryOrder,
        fail_order: MemoryOrder,
        mem_loc: P<SSAVar>,
        expected_value: P<Value>,
        desired_value: P<Value>
    },
    
    // yields old memory value
    AtomicRMW{
        is_iref: bool, // T for iref, F for ptr
        order: MemoryOrder,
        op: AtomicRMWOp,
        mem_loc: P<Value>,
        value: P<Value> // operand for op
    },
    
    // yields a reference of the type
    New(P<MuType_>),
    
    // yields an iref of the type
    AllocA(P<MuType_>),
    
    // yields ref
    NewHybrid{    // hybrid type, var part length
        ty: P<MuType_>, 
        var_len: P<Value>
    },  
    
    // yields iref
    AllocAHybrid{
        ty: P<MuType_>, 
        var_len: P<Value>
    },
    
    // yields stack ref
    NewStack{
        func: P<Value>
    },
    
    // yields thread reference
    NewThread{
        stack: P<Value>,
        args: Vec<P<Value>>
    },
    
    // yields thread reference (thread resumes with exceptional value)
    NewThreadExn{
        stack: P<Value>,
        exn: P<Value>
    },
    
    // yields frame cursor
    NewFrameCursor(P<Value>), // stack
    
    GetIRef(P<Value>),
    
    GetFieldIRef{
        base: P<Value>, // iref or ptr
        index: P<Constant>
    },
    
    GetElementIRef{
        base: P<Value>,
        index: P<Value>
    },
    
    ShiftIRef{
        base: P<Value>,
        offset: P<Value>
    },
    
    GetVarPartIRef(P<Value>),
    
//    PushFrame{
//        stack: P<Value>,
//        func: P<Value>
//    },
//    PopFrame{
//        stack: P<Value>
//    }
}

pub enum Instruction {
    Assign{
        left: Vec<P<Value>>,
        right: Expression
    },

    Fence(MemoryOrder),
}

pub enum Terminal {
    Return(Vec<P<Value>>),
    ThreadExit,
    Throw(Vec<P<Value>>),
    TailCall(CallData),
    Branch1(Destination),
    Branch2{
        cond: P<Value>,
        true_dest: Destination,
        false_dest: Destination
    },
    Watchpoint{ // Watchpoint NONE ResumptionData
                //   serves as an unconditional trap. Trap to client, and resume with ResumptionData
                // Watchpoint (WPID dest) ResumptionData
                //   when disabled, jump to dest
                //   when enabled, trap to client and resume
        id: Option<WPID>,
        disable_dest: Option<Destination>,
        resume: ResumptionData
    }, 
    WPBranch{
        wp: WPID, 
        disable_dest: Destination,
        enable_dest: Destination
    },
    Call{
        data: CallData,
        resume: ResumptionData
    },
    SwapStack{
        stack: P<Value>,
        is_exception: bool,
        args: Vec<P<Value>>,
        resume: ResumptionData
    },
    Switch{
        cond: P<Value>,
        default: Destination,
        branches: Vec<(P<Constant>, Destination)>
    },
    ExnInstruction{
        inner: Expression,
        resume: ResumptionData
    }
}

#[derive(Clone)]
pub struct MuConstant{
    pub ty: P<MuType_>, 
    pub val: Constant
}

pub struct MuFunction {
    pub fn_name: MuTag,
    pub sig: P<MuFuncSig>,
    pub entry: MuTag,
    pub blocks: Vec<(MuTag, Block)>
}

#[derive(Copy, Clone)]
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

#[derive(Copy, Clone)]
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

#[derive(Copy, Clone)]
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