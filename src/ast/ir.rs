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

pub struct TerminationData {
    normal_dest: Destination,
    exn_dest: Destination
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
    Constant(P<Constant>),
    
    // memory operations
    
    CmpXchg{
        is_iref: bool, // T for iref, F for ptr
        is_strong: bool,
        success_order: MemoryOrder,
        fail_order: MemoryOrder,
        mem_loc: P<SSAVar>,
        expected_value: P<Value>,
        desired_value: P<Value>
    },
    
    AtomicRMW{
        is_iref: bool, // T for iref, F for ptr
        order: MemoryOrder,
        op: AtomicRMWOp,
        mem_loc: P<Value>,
        value: P<Value> // operand for op
    },
    
    Fence(MemoryOrder),
    
    // allocation operations
    
    New(P<MuType_>),
    AllocA(P<MuType_>),
    NewHybrid{    // hybrid type, var part length
        ty: P<MuType_>, 
        var_len: P<Value>
    },  
    AllocAHybrid{
        ty: P<MuType_>, 
        var_len: P<Value>
    },
    NewStack{
        func: P<Value>
    },
    NewThread{
        stack: P<Value>,
        args: Vec<P<Value>>
    },
    NewThreadExn{   // NewThreadExn SSAVar (* stack id *) SSAVar (* exception value *) ???
        stack: P<Value>,
        exn: P<Value>
    },
    
    PushFrame{
        stack: P<Value>,
        func: P<Value>
    },
    PopFrame{
        stack: P<Value>
    }
}

pub enum Instruction {
    Assign{
        left: Vec<P<Value>>,
        right: Expression
    },
    Load{
        dest: P<SSAVar>,
        is_iref: bool,
        mem_loc: P<Value>,
        order: MemoryOrder
    },
    Store{
        src: P<SSAVar>,
        is_iref: bool,
        mem_loc: P<Value>,
        order: MemoryOrder        
    }
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
    Watchpoint, // TODO: Watchpoint ((wpid # destination) option) termination_data
    WPBranch{
        wp: WPID, 
        disable_dest: Destination,
        enable_dest: Destination
    },
    Call{
        data: CallData,
        normal_dest: Destination,
        exn_dest: Option<Destination>
    },
    SwapStack{
        stack: P<Value>,
        args: Vec<P<Value>>,
        normal_dest: Destination,
        exn_dest: Destination
    },
    Switch{
        cond: P<Value>,
        default: Destination,
        branches: Vec<(P<Constant>, Destination)>
    },
    ExnInstruction{
        inner: Expression,
        term: TerminationData
    }
}

#[derive(Clone)]
pub struct MuConstant{
    ty: P<MuType_>, 
    val: Constant
}

pub struct MuFunction {
    pub fn_name: MuTag,
    pub sig: P<MuFuncSig>,
    pub entry: MuTag,
    pub blocks: Vec<(MuTag, Block)>
}

pub fn declare_const(const_name: MuTag, ty: P<MuType_>, val: Constant) -> Value {
    Value::Constant(MuConstant{ty: ty, val: val})
}
pub fn declare_type(type_name: MuTag, ty: P<MuType_>) -> P<MuType_> {
    ty
}
pub fn declare_func_sig(sig_name: MuTag, ret_tys: Vec<P<MuType_>>, arg_tys: Vec<P<MuType_>>) -> MuFuncSig {
    MuFuncSig::new(ret_tys, arg_tys)
}
pub fn declare_func (fn_name: MuTag, sig: P<MuFuncSig>, entry: MuTag, blocks: Vec<(MuTag, Block)>) -> MuFunction {
    MuFunction{fn_name: fn_name, sig: sig, entry: entry, blocks: blocks}
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