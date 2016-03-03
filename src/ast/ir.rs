use ast::types::*;
use std::sync::Arc;

pub type WPID  = usize;
pub type MuID  = usize;
pub type MuTag = &'static str;
pub type Address = usize; // TODO: replace this with Address(usize)

#[derive(Clone)]
pub struct SSAVar {
    id: MuID,
    tag:Option<MuTag>,
    ty: Arc<MuType_>
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

#[derive(Clone)]
pub enum Literal {
    LitInt(usize, Arc<MuType_>),
    
    LitFP(f64, Arc<MuType_>),
    LitFPNaN(Arc<MuType_>),
    LitFPInfPos(Arc<MuType_>),
    LitFPInfNeg(Arc<MuType_>),
    
    LitNull(Arc<MuType_>)
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

#[derive(Clone)]
pub struct CallData {
    func: SSAVar,
    args: Vec<SSAVar>,
    convention: CallConvention
}

#[derive(Clone)]
pub struct Block<'func> {
    args: Vec<SSAVar>,
    body: Vec<Instruction>,
    exit: Terminal<'func>,
    keepalives: Vec<SSAVar>
}

#[derive(Clone)]
pub struct TerminationData<'func> {
    normal_dest: Destination<'func>,
    exn_dest: Destination<'func>
}

#[derive(Clone)]
pub enum DestArg {
    Normal(SSAVar),
    Freshbound(usize)
}

#[derive(Clone)]
pub struct Destination<'func> {
    block: &'func Block<'func>,
    args: Vec<DestArg>
}

#[derive(Clone)]
pub enum Value {
    Int(usize, usize),
    IRef(Arc<MuType_>, Address),
    FloatV(f32),
    DoubleV(f64),
    VectorV(Vec<Value>),
    FuncRefV(Address),
    UFuncRefV(Address)
}

#[derive(Clone)]
pub enum Expression {
    BinOp(BinOp, SSAVar, SSAVar), 
    Value(Value),
    
    // memory operations
    
    CmpXchg{
        is_iref: bool, // T for iref, F for ptr
        is_strong: bool,
        success_order: MemoryOrder,
        fail_order: MemoryOrder,
        mem_loc: SSAVar,
        expected_value: SSAVar,
        desired_value: SSAVar
    },
    
    AtomicRMW{
        is_iref: bool, // T for iref, F for ptr
        order: MemoryOrder,
        op: AtomicRMWOp,
        mem_loc: SSAVar,
        value: SSAVar // operand for op
    },
    
    Fence(MemoryOrder),
    
    // allocation operations
    
    New(Arc<MuType_>),
    AllocA(Arc<MuType_>),
    NewHybrid{    // hybrid type, var part length
        ty: Arc<MuType_>, 
        var_len: SSAVar
    },  
    AllocAHybrid{
        ty: Arc<MuType_>, 
        var_len: SSAVar
    },
    NewStack{
        func: SSAVar
    },
    NewThread{
        stack: SSAVar,
        args: Vec<SSAVar>
    },
    NewThreadExn{   // NewThreadExn SSAVar (* stack id *) SSAVar (* exception value *) ???
        stack: SSAVar,
        exn: SSAVar
    },
    
    PushFrame{
        stack: SSAVar,
        func: SSAVar
    },
    PopFrame{
        stack: SSAVar
    }
}

#[derive(Clone)]
pub enum Instruction {
    Assign{
        left: Vec<SSAVar>,
        right: Expression
    },
    Load{
        dest: SSAVar,
        is_iref: bool,
        mem_loc: SSAVar,
        order: MemoryOrder
    },
    Store{
        src: SSAVar,
        is_iref: bool,
        mem_loc: SSAVar,
        order: MemoryOrder        
    }
}

#[derive(Clone)]
pub enum Terminal<'func> {
    Return(Vec<SSAVar>),
    ThreadExit,
    Throw(Vec<SSAVar>),
    TailCall(CallData),
    Branch1(Destination<'func>),
    Branch2{
        cond: SSAVar,
        true_dest: Destination<'func>,
        false_dest: Destination<'func>
    },
    Watchpoint, // TODO: Watchpoint ((wpid # destination) option) termination_data
    WPBranch{
        wp: WPID, 
        disable_dest: Destination<'func>, 
        enable_dest: Destination<'func>
    },
    Call{
        data: CallData,
        normal_dest: Destination<'func>,
        exn_dest: Destination<'func>
    },
    SwapStack{
        stack: SSAVar,
        args: Vec<SSAVar>,
        normal_dest: Destination<'func>,
        exn_dest: Destination<'func>
    },
    Switch{
        cond: SSAVar,
        default: Destination<'func>,
        branches: Vec<(Value, Destination<'func>)>
    },
    ExnInstruction{
        inner: Expression,
        term: TerminationData<'func>
    }
}

#[derive(Clone)]
pub enum Declaration<'global> {
    ConstDecl{
        const_name: MuTag, 
        ty: Arc<MuType_>, 
        val: Value
    },
    TypeDef{
        type_name: MuTag, 
        ty: Arc<MuType_>
    },
    FunctionSignature{
        sig_name: MuTag, 
        ret_tys: Vec<Arc<MuType_>>, 
        arg_tys: Vec<Arc<MuType_>>
    },
    FuncDef{
        fn_name: MuTag,
        sig_name: MuTag,
        label: MuTag, // ?
        blocks: Vec<(MuTag, Block<'global>)>
    }
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