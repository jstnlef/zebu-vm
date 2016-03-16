use ast::ptr::P;
use ast::op::{BinOp, CmpOp, AtomicRMWOp};
use ast::types::*;

use std::fmt;

pub type WPID  = usize;
pub type MuID  = usize;
pub type MuTag = &'static str;
pub type Address = usize; // TODO: replace this with Address(usize)

#[derive(Debug)]
pub struct MuFunction {
    pub fn_name: MuTag,
    pub sig: P<MuFuncSig>,
    pub entry: MuTag,
    pub blocks: Vec<(MuTag, Block)>
}

#[derive(Debug)]
pub struct Block {
    pub label: MuTag,
    pub content: Option<BlockContent>
}

impl Block {
    pub fn new(label: MuTag) -> Block {
        Block{label: label, content: None}
    }
}

#[derive(Debug)]
pub struct BlockContent {
    pub args: Vec<P<TreeNode>>,
    pub body: Vec<P<TreeNode>>,
    pub keepalives: Option<Vec<P<TreeNode>>>    
}

#[derive(Clone)]
/// always use with P<TreeNode>
pub struct TreeNode {
    pub id: MuID,
    pub tag: MuTag,
    pub v: TreeNode_,
}

impl TreeNode {
    pub fn new_ssa(id: MuID, tag: MuTag, ty: P<MuType>) -> P<TreeNode> {
        P(TreeNode{
                id: id, 
                tag: tag, 
                v: TreeNode_::Value(P(Value{ty: ty, v: Value_::SSAVar}))
        })
    }
    
    pub fn new_constant(id: MuID, tag: MuTag, ty: P<MuType>, v: Constant) -> P<TreeNode> {
        P(TreeNode{
                id: id,
                tag: tag,
                v: TreeNode_::Value(P(Value{ty: ty, v: Value_::Constant(v)}))
            })
    }
    
    pub fn new_value(id: MuID, tag: MuTag, v: P<Value>) -> P<TreeNode> {
        P(TreeNode{
                id: id,
                tag: tag,
                v: TreeNode_::Value(v)
            }
        )
    }
    
    pub fn new_inst(id: MuID, tag: MuTag, v: Instruction) -> P<TreeNode> {
        P(TreeNode{id: id, tag: tag, v: TreeNode_::Instruction(v)})
    }
    
    pub fn as_value(&self) -> Option<&P<Value>> {
        match self.v {
            TreeNode_::Value(ref pv) => Some(&pv),
            _ => None
        }
    }
}

impl fmt::Debug for TreeNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.v {
            TreeNode_::Value(ref pv) => {
                match pv.v {
                    Value_::SSAVar => {
                        write!(f, "{:?} %{}#{}", pv.ty, self.tag, self.id)
                    },
                    Value_::Constant(ref c) => {
                        write!(f, "{:?} {:?}", pv.ty, c) 
                    }
                }
            },
            TreeNode_::Instruction(ref inst) => {
                write!(f, "{:?}", inst)
            }
        }
    }
}

#[derive(Clone)]
pub enum TreeNode_ {
    Value(P<Value>),
    Instruction(Instruction),
}

/// always use with P<Value>
#[derive(Clone)]
pub struct Value {
    pub ty: P<MuType>,
    pub v: Value_
}

#[derive(Clone)]
pub enum Value_ {
    SSAVar,
    Constant(Constant)
}

#[derive(Clone)]
pub enum Constant {
    Int(usize),
    Float(f32),
    Double(f64),
    IRef(Address),
    FuncRef(Address),
    UFuncRef(Address),
    Vector(Vec<Constant>),    
}

impl fmt::Debug for Constant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Constant::Int(v) => write!(f, "{}", v),
            &Constant::Float(v) => write!(f, "{}", v),
            &Constant::Double(v) => write!(f, "{}", v),
            &Constant::IRef(v) => write!(f, "{}", v),
            &Constant::FuncRef(v) => write!(f, "{}", v),
            &Constant::UFuncRef(v) => write!(f, "{}", v),
            &Constant::Vector(ref v) => write!(f, "{:?}", v)
        }
    }
}

#[derive(Clone)]
pub enum Instruction {
    NonTerm(NonTermInstruction),
    Term(Terminal)
}

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Instruction::NonTerm(ref inst) => write!(f, "{:?}", inst),
            &Instruction::Term(ref inst) => write!(f, "{:?}", inst)
        }
    }    
}

#[derive(Clone)]
pub enum Terminal {
    Return(Vec<P<TreeNode>>),
    ThreadExit, // TODO:  common inst
    Throw(Vec<P<TreeNode>>),
    TailCall(CallData),
    Branch1(Destination),
    Branch2{
        cond: P<TreeNode>,
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
        stack: P<TreeNode>,
        is_exception: bool,
        args: Vec<P<TreeNode>>,
        resume: ResumptionData
    },
    Switch{
        cond: P<TreeNode>,
        default: Destination,
        branches: Vec<(P<TreeNode>, Destination)>
    },
    ExnInstruction{
        inner: NonTermInstruction,
        resume: ResumptionData
    }
}

impl fmt::Debug for Terminal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Terminal::Return(ref vals) => write!(f, "RET {:?}", vals),
            &Terminal::ThreadExit => write!(f, "THREADEXIT"),
            &Terminal::Throw(ref vals) => write!(f, "THROW {:?}", vals),
            &Terminal::TailCall(ref call) => write!(f, "TAILCALL {:?}", call),
            &Terminal::Branch1(ref dest) => write!(f, "BRANCH {:?}", dest),
            &Terminal::Branch2{ref cond, ref true_dest, ref false_dest} => {
                write!(f, "BRANCH2 {:?} {:?} {:?}", cond, true_dest, false_dest)
            },
            &Terminal::Watchpoint{id, ref disable_dest, ref resume} => {
                match id {
                    Some(id) => {
                        write!(f, "WATCHPOINT {:?} {:?} {:?}", id, disable_dest.as_ref().unwrap(), resume)
                    },
                    None => {
                        write!(f, "TRAP {:?}", resume)
                    }
                }
            },
            &Terminal::WPBranch{wp, ref disable_dest, ref enable_dest} => {
                write!(f, "WPBRANCH {:?} {:?} {:?}", wp, disable_dest, enable_dest)
            },
            &Terminal::Call{ref data, ref resume} => write!(f, "CALL {:?} {:?}", data, resume),
            &Terminal::SwapStack{ref stack, is_exception, ref args, ref resume} => {
                write!(f, "SWAPSTACK {:?} {:?} {:?} {:?}", stack, is_exception, args, resume)
            },
            &Terminal::Switch{ref cond, ref default, ref branches} => {
                write!(f, "SWITCH {:?} {:?} {{{:?}}}", cond, default, branches)
            },
            &Terminal::ExnInstruction{ref inner, ref resume} => {
                write!(f, "{:?} {:?}", inner, resume)
            }
        }
    }
}

#[derive(Clone)]
pub enum NonTermInstruction {
    Assign{
        left: Vec<P<TreeNode>>,
        right: Expression_
    },

    Fence(MemoryOrder),
}

impl fmt::Debug for NonTermInstruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &NonTermInstruction::Assign{ref left, ref right} => {
                write!(f, "{:?} = {:?}", left, right)
            },
            &NonTermInstruction::Fence(order) => {
                write!(f, "FENCE {:?}", order)
            }
        }
    }
}

#[derive(Clone)]
pub enum Expression_ {
    BinOp(BinOp, P<TreeNode>, P<TreeNode>), 
    CmpOp(CmpOp, P<TreeNode>, P<TreeNode>),
    
    // yields a tuple of results from the call
    ExprCall{
        data: CallData,
        is_abort: bool, // T to abort, F to rethrow
    },
    
    // yields the memory value
    Load{
        is_ptr: bool,
        order: MemoryOrder,
        mem_loc: P<TreeNode>
    },
    
    // yields nothing
    Store{
        is_ptr: bool,
        order: MemoryOrder,        
        mem_loc: P<TreeNode>,
        value: P<TreeNode>
    },
    
    // yields pair (oldvalue, boolean (T = success, F = failure))
    CmpXchg{
        is_ptr: bool,
        is_weak: bool,
        success_order: MemoryOrder,
        fail_order: MemoryOrder,
        mem_loc: P<TreeNode>,
        expected_value: P<TreeNode>,
        desired_value: P<TreeNode>
    },
    
    // yields old memory value
    AtomicRMW{
        is_ptr: bool, // T for iref, F for ptr
        order: MemoryOrder,
        op: AtomicRMWOp,
        mem_loc: P<TreeNode>,
        value: P<TreeNode> // operand for op
    },
    
    // yields a reference of the type
    New(P<MuType>),
    
    // yields an iref of the type
    AllocA(P<MuType>),
    
    // yields ref
    NewHybrid(P<MuType>, P<TreeNode>),
    
    // yields iref
    AllocAHybrid(P<MuType>, P<TreeNode>),
    
    // yields stack ref
    NewStack(P<TreeNode>), // func
                           // TODO: common inst
    
    // yields thread reference
    NewThread(P<TreeNode>, Vec<P<TreeNode>>), // stack, args
    
    // yields thread reference (thread resumes with exceptional value)
    NewThreadExn(P<TreeNode>, P<TreeNode>), // stack, exception
    
    // yields frame cursor
    NewFrameCursor(P<TreeNode>), // stack
    
    // ref<T> -> iref<T>
    GetIRef(P<TreeNode>),
    
    // iref|uptr<struct|hybrid<T>> int<M> -> iref|uptr<U>
    GetFieldIRef{
        is_ptr: bool,
        base: P<TreeNode>, // iref or uptr
        index: P<TreeNode> // constant
    },
    
    // iref|uptr<array<T N>> int<M> -> iref|uptr<T>
    GetElementIRef{
        is_ptr: bool,
        base: P<TreeNode>,
        index: P<TreeNode> // can be constant or ssa var
    },
    
    // iref|uptr<T> int<M> -> iref|uptr<T>
    ShiftIRef{
        is_ptr: bool,
        base: P<TreeNode>,
        offset: P<TreeNode>
    },
    
    // iref|uptr<hybrid<T U>> -> iref|uptr<U>
    GetVarPartIRef{
        is_ptr: bool,
        base: P<TreeNode>
    },
    
//    PushFrame{
//        stack: P<Value>,
//        func: P<Value>
//    },
//    PopFrame{
//        stack: P<Value>
//    }
}

macro_rules! select {
    ($cond: expr, $res1 : expr, $res2 : expr) => {
        if $cond {
            $res1
        } else {
            $res2
        }
    }
}

impl fmt::Debug for Expression_ {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Expression_::BinOp(op, ref op1, ref op2) => write!(f, "{:?} {:?} {:?}", op, op1, op2),
            &Expression_::CmpOp(op, ref op1, ref op2) => write!(f, "{:?} {:?} {:?}", op, op1, op2),
            &Expression_::ExprCall{ref data, is_abort} => {
                let abort = select!(is_abort, "ABORT_ON_EXN", "RETHROW");
                write!(f, "CALL {:?} {}", data, abort)
            },
            &Expression_::Load{is_ptr, ref mem_loc, order} => {
                let ptr = select!(is_ptr, "PTR", "");
                write!(f, "LOAD {} {:?} {:?}", ptr, order, mem_loc) 
            },
            &Expression_::Store{ref value, is_ptr, ref mem_loc, order} => {
                let ptr = select!(is_ptr, "PTR", "");
                write!(f, "STORE {} {:?} {:?} {:?}", ptr, order, mem_loc, value)
            },
            &Expression_::CmpXchg{is_ptr, is_weak, success_order, fail_order, 
                ref mem_loc, ref expected_value, ref desired_value} => {
                let ptr = select!(is_ptr, "PTR", "");
                let weak = select!(is_weak, "WEAK", "");
                write!(f, "CMPXCHG {} {} {:?} {:?} {:?} {:?} {:?}", 
                    ptr, weak, success_order, fail_order, mem_loc, expected_value, desired_value)  
            },
            &Expression_::AtomicRMW{is_ptr, order, op, ref mem_loc, ref value} => {
                let ptr = select!(is_ptr, "PTR", "");
                write!(f, "ATOMICRMW {} {:?} {:?} {:?} {:?}", ptr, order, op, mem_loc, value)
            },
            &Expression_::New(ref ty) => write!(f, "NEW {:?}", ty),
            &Expression_::AllocA(ref ty) => write!(f, "ALLOCA {:?}", ty),
            &Expression_::NewHybrid(ref ty, ref len) => write!(f, "NEWHYBRID {:?} {:?}", ty, len),
            &Expression_::AllocAHybrid(ref ty, ref len) => write!(f, "ALLOCAHYBRID {:?} {:?}", ty, len),
            &Expression_::NewStack(ref func) => write!(f, "NEWSTACK {:?}", func),
            &Expression_::NewThread(ref stack, ref args) => write!(f, "NEWTHREAD {:?} PASS_VALUES {:?}", stack, args),
            &Expression_::NewThreadExn(ref stack, ref exn) => write!(f, "NEWTHREAD {:?} THROW_EXC {:?}", stack, exn),
            &Expression_::NewFrameCursor(ref stack) => write!(f, "NEWFRAMECURSOR {:?}", stack),
            &Expression_::GetIRef(ref reference) => write!(f, "GETIREF {:?}", reference),
            &Expression_::GetFieldIRef{is_ptr, ref base, ref index} => {
                let ptr = select!(is_ptr, "PTR", "");
                write!(f, "GETFIELDIREF {} {:?} {:?}", ptr, base, index)
            },
            &Expression_::GetElementIRef{is_ptr, ref base, ref index} => {
                let ptr = select!(is_ptr, "PTR", "");
                write!(f, "GETELEMENTIREF {} {:?} {:?}", ptr, base, index)
            },
            &Expression_::ShiftIRef{is_ptr, ref base, ref offset} => {
                let ptr = select!(is_ptr, "PTR", "");
                write!(f, "SHIFTIREF {} {:?} {:?}", ptr, base, offset)
            },
            &Expression_::GetVarPartIRef{is_ptr, ref base} => {
                let ptr = select!(is_ptr, "PTR", "");
                write!(f, "GETVARPARTIREF {} {:?}", ptr, base)
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MemoryOrder {
    NotAtomic,
    Relaxed,
    Consume,
    Acquire,
    Release,
    AcqRel,
    SeqCst
}

#[derive(Copy, Clone, Debug)]
pub enum CallConvention {
    Mu,
    Foreign(ForeignFFI)
}

#[derive(Copy, Clone, Debug)]
pub enum ForeignFFI {
    C
}

#[derive(Clone)]
pub struct CallData {
    pub func: P<TreeNode>,
    pub args: Vec<P<TreeNode>>,
    pub convention: CallConvention
}

impl fmt::Debug for CallData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} {:?} ({:?})", self.convention, self.func, self.args)
    }    
}

#[derive(Clone)]
pub struct ResumptionData {
    pub normal_dest: Destination,
    pub exn_dest: Destination
}

impl fmt::Debug for ResumptionData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "normal: {:?}, exception: {:?}", self.normal_dest, self.exn_dest)
    }
}

#[derive(Clone)]
pub struct Destination {
    pub target: MuTag,
    pub args: Vec<DestArg>
}

impl fmt::Debug for Destination {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{:?}", self.target, self.args)
    }
}

#[derive(Clone)]
pub enum DestArg {
    Normal(P<TreeNode>),
    Freshbound(usize)
}

impl fmt::Debug for DestArg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &DestArg::Normal(ref pv) => write!(f, "{:?}", pv),
            &DestArg::Freshbound(n) => write!(f, "${}", n)
        }
    }    
}