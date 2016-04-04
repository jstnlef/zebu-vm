use ast::ptr::P;
use ast::op::*;
use ast::types::*;

use std::collections::HashMap;
use std::fmt;
use std::cell::Cell;
use std::cell::RefCell;

pub type WPID  = usize;
pub type MuID  = usize;
pub type MuTag = &'static str;
pub type Address = usize; // TODO: replace this with Address(usize)

pub type OpIndex = usize;

#[derive(Debug)]
pub struct MuFunction {
    pub fn_name: MuTag,
    pub sig: P<MuFuncSig>,
    pub content: Option<FunctionContent>,
    pub context: FunctionContext
}

#[derive(Debug)]
pub struct FunctionContent {
    pub entry: MuTag,
    pub blocks: Vec<(MuTag, Block)>
}

#[derive(Debug)]
pub struct FunctionContext {
    pub values: HashMap<MuID, ValueEntry>
}

impl FunctionContext {
    fn new() -> FunctionContext {
        FunctionContext {
            values: HashMap::new()
        }
    }
    
    pub fn get_value(&self, id: MuID) -> Option<&ValueEntry> {
        self.values.get(&id)
    }
    
    pub fn get_value_mut(&mut self, id: MuID) -> Option<&mut ValueEntry> {
        self.values.get_mut(&id)
    }
}

impl MuFunction {
    pub fn new(fn_name: MuTag, sig: P<MuFuncSig>) -> MuFunction {
        MuFunction{fn_name: fn_name, sig: sig, content: None, context: FunctionContext::new()}
    }
    
    pub fn define(&mut self, content: FunctionContent) {
        self.content = Some(content)
    }
    
    pub fn new_ssa(&mut self, id: MuID, tag: MuTag, ty: P<MuType>) -> P<TreeNode> {
        self.context.values.insert(id, ValueEntry{id: id, tag: tag, ty: ty.clone(), use_count: Cell::new(0), expr: None});
        
        P(TreeNode {
            v: TreeNode_::Value(P(Value{
                tag: tag,
                ty: ty,
                v: Value_::SSAVar(id)
            }))
        })
    }
    
    pub fn new_constant(&mut self, tag: MuTag, ty: P<MuType>, v: Constant) -> P<TreeNode> {
        P(TreeNode{
            v: TreeNode_::Value(P(Value{
                tag: tag,
                ty: ty, 
                v: Value_::Constant(v)
            }))
        })
    }
    
    pub fn new_value(&mut self, v: P<Value>) -> P<TreeNode> {
        P(TreeNode{
            v: TreeNode_::Value(v)
        })
    }    
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
//    pub op: OpCode,
    pub v: TreeNode_
}

impl TreeNode {   
    pub fn new_inst(v: Instruction) -> P<TreeNode> {
        P(TreeNode{v: TreeNode_::Instruction(v)})
    }
    
    pub fn extract_ssa_id(&self) -> Option<MuID> {
        match self.v {
            TreeNode_::Value(ref pv) => {
                match pv.v {
                    Value_::SSAVar(id) => Some(id),
                    _ => None
                }
            },
            _ => None
        }
    }
}

impl fmt::Debug for TreeNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.v {
            TreeNode_::Value(ref pv) => {
                match pv.v {
                    Value_::SSAVar(id) => {
                        write!(f, "{:?} %{}#{}", pv.ty, pv.tag, id)
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
    Instruction(Instruction)
}

/// always use with P<Value>
#[derive(Clone)]
pub struct Value {
    pub tag: MuTag,
    pub ty: P<MuType>,
    pub v: Value_
}

#[derive(Clone)]
pub enum Value_ {
    SSAVar(MuID),
    Constant(Constant)
}

#[derive(Clone)]
pub struct ValueEntry {
    pub id: MuID,
    pub tag: MuTag,
    pub ty: P<MuType>,
    pub use_count: Cell<usize>,  // how many times this entry is used
    pub expr: Option<Instruction>
}

impl ValueEntry {
    pub fn assign_expr(&mut self, expr: Instruction) {
        self.expr = Some(expr)
    }
}

impl fmt::Debug for ValueEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} {}({})", self.ty, self.tag, self.id)
    }
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
pub struct Instruction {
    pub value : Option<Vec<P<TreeNode>>>,
    pub ops : RefCell<Vec<P<TreeNode>>>,
    pub v: Instruction_
}

impl Instruction {
    fn debug_str(&self, ops: &Vec<P<TreeNode>>) -> String {
        self.v.debug_str(ops)
    }
}

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ops = &self.ops.borrow();
        if self.value.is_some() {
            write!(f, "{:?} = {}", self.value.as_ref().unwrap(), self.v.debug_str(ops))
        } else {
            write!(f, "{}", self.v.debug_str(ops))
        }
    }
}

#[derive(Clone)]
pub enum Instruction_ {
    // non-terminal instruction
    
    // expressions
    
    BinOp(BinOp, OpIndex, OpIndex), 
    CmpOp(CmpOp, OpIndex, OpIndex),
    
    // yields a tuple of results from the call
    ExprCall{
        data: CallData,
        is_abort: bool, // T to abort, F to rethrow
    },
    
    // yields the memory value
    Load{
        is_ptr: bool,
        order: MemoryOrder,
        mem_loc: OpIndex
    },
    
    // yields nothing
    Store{
        is_ptr: bool,
        order: MemoryOrder,        
        mem_loc: OpIndex,
        value: OpIndex
    },
    
    // yields pair (oldvalue, boolean (T = success, F = failure))
    CmpXchg{
        is_ptr: bool,
        is_weak: bool,
        success_order: MemoryOrder,
        fail_order: MemoryOrder,
        mem_loc: OpIndex,
        expected_value: OpIndex,
        desired_value: OpIndex
    },
    
    // yields old memory value
    AtomicRMW{
        is_ptr: bool, // T for iref, F for ptr
        order: MemoryOrder,
        op: AtomicRMWOp,
        mem_loc: OpIndex,
        value: OpIndex // operand for op
    },
    
    // yields a reference of the type
    New(P<MuType>),
    
    // yields an iref of the type
    AllocA(P<MuType>),
    
    // yields ref
    NewHybrid(P<MuType>, OpIndex),
    
    // yields iref
    AllocAHybrid(P<MuType>, OpIndex),
    
    // yields stack ref
    NewStack(OpIndex), // func
                           // TODO: common inst
    
    // yields thread reference
    NewThread(OpIndex, Vec<OpIndex>), // stack, args
    
    // yields thread reference (thread resumes with exceptional value)
    NewThreadExn(OpIndex, OpIndex), // stack, exception
    
    // yields frame cursor
    NewFrameCursor(OpIndex), // stack
    
    // ref<T> -> iref<T>
    GetIRef(OpIndex),
    
    // iref|uptr<struct|hybrid<T>> int<M> -> iref|uptr<U>
    GetFieldIRef{
        is_ptr: bool,
        base: OpIndex, // iref or uptr
        index: OpIndex // constant
    },
    
    // iref|uptr<array<T N>> int<M> -> iref|uptr<T>
    GetElementIRef{
        is_ptr: bool,
        base: OpIndex,
        index: OpIndex // can be constant or ssa var
    },
    
    // iref|uptr<T> int<M> -> iref|uptr<T>
    ShiftIRef{
        is_ptr: bool,
        base: OpIndex,
        offset: OpIndex
    },
    
    // iref|uptr<hybrid<T U>> -> iref|uptr<U>
    GetVarPartIRef{
        is_ptr: bool,
        base: OpIndex
    },
    
//    PushFrame{
//        stack: P<Value>,
//        func: P<Value>
//    },
//    PopFrame{
//        stack: P<Value>
//    }

    Fence(MemoryOrder),
    
    // terminal instruction
    Return(Vec<OpIndex>),
    ThreadExit, // TODO:  common inst
    Throw(Vec<OpIndex>),
    TailCall(CallData),
    Branch1(Destination),
    Branch2{
        cond: OpIndex,
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
        stack: OpIndex,
        is_exception: bool,
        args: Vec<OpIndex>,
        resume: ResumptionData
    },
    Switch{
        cond: OpIndex,
        default: Destination,
        branches: Vec<(OpIndex, Destination)>
    },
    ExnInstruction{
        inner: P<Instruction>,
        resume: ResumptionData
    }
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

impl Instruction_ {
    fn debug_str(&self, ops: &Vec<P<TreeNode>>) -> String {
        match self {
            &Instruction_::BinOp(op, op1, op2) => fmt::format(format_args!("{:?} {:?} {:?}", op, ops[op1], ops[op2])),
            &Instruction_::CmpOp(op, op1, op2) => fmt::format(format_args!("{:?} {:?} {:?}", op, ops[op1], ops[op2])),
            &Instruction_::ExprCall{ref data, is_abort} => {
                let abort = select!(is_abort, "ABORT_ON_EXN", "RETHROW");
                fmt::format(format_args!("CALL {} {}", data.debug_str(ops), abort))
            },
            &Instruction_::Load{is_ptr, mem_loc, order} => {
                let ptr = select!(is_ptr, "PTR", "");
                fmt::format(format_args!("LOAD {} {:?} {:?}", ptr, order, ops[mem_loc])) 
            },
            &Instruction_::Store{value, is_ptr, mem_loc, order} => {
                let ptr = select!(is_ptr, "PTR", "");
                fmt::format(format_args!("STORE {} {:?} {:?} {:?}", ptr, order, ops[mem_loc], ops[value]))
            },
            &Instruction_::CmpXchg{is_ptr, is_weak, success_order, fail_order, 
                mem_loc, expected_value, desired_value} => {
                let ptr = select!(is_ptr, "PTR", "");
                let weak = select!(is_weak, "WEAK", "");
                fmt::format(format_args!("CMPXCHG {} {} {:?} {:?} {:?} {:?} {:?}", 
                    ptr, weak, success_order, fail_order, ops[mem_loc], ops[expected_value], ops[desired_value]))  
            },
            &Instruction_::AtomicRMW{is_ptr, order, op, mem_loc, value} => {
                let ptr = select!(is_ptr, "PTR", "");
                fmt::format(format_args!("ATOMICRMW {} {:?} {:?} {:?} {:?}", ptr, order, op, ops[mem_loc], ops[value]))
            },
            &Instruction_::New(ref ty) => fmt::format(format_args!("NEW {:?}", ty)),
            &Instruction_::AllocA(ref ty) => fmt::format(format_args!("ALLOCA {:?}", ty)),
            &Instruction_::NewHybrid(ref ty, len) => fmt::format(format_args!("NEWHYBRID {:?} {:?}", ty, ops[len])),
            &Instruction_::AllocAHybrid(ref ty, len) => fmt::format(format_args!("ALLOCAHYBRID {:?} {:?}", ty, ops[len])),
            &Instruction_::NewStack(func) => fmt::format(format_args!("NEWSTACK {:?}", ops[func])),
            &Instruction_::NewThread(stack, ref args) => fmt::format(format_args!("NEWTHREAD {:?} PASS_VALUES {}", ops[stack], op_vector_str(args, ops))),
            &Instruction_::NewThreadExn(stack, exn) => fmt::format(format_args!("NEWTHREAD {:?} THROW_EXC {:?}", ops[stack], ops[exn])),
            &Instruction_::NewFrameCursor(stack) => fmt::format(format_args!("NEWFRAMECURSOR {:?}", ops[stack])),
            &Instruction_::GetIRef(reference) => fmt::format(format_args!("GETIREF {:?}", ops[reference])),
            &Instruction_::GetFieldIRef{is_ptr, base, index} => {
                let ptr = select!(is_ptr, "PTR", "");
                fmt::format(format_args!("GETFIELDIREF {} {:?} {:?}", ptr, ops[base], ops[index]))
            },
            &Instruction_::GetElementIRef{is_ptr, base, index} => {
                let ptr = select!(is_ptr, "PTR", "");
                fmt::format(format_args!("GETELEMENTIREF {} {:?} {:?}", ptr, ops[base], ops[index]))
            },
            &Instruction_::ShiftIRef{is_ptr, base, offset} => {
                let ptr = select!(is_ptr, "PTR", "");
                fmt::format(format_args!("SHIFTIREF {} {:?} {:?}", ptr, ops[base], ops[offset]))
            },
            &Instruction_::GetVarPartIRef{is_ptr, base} => {
                let ptr = select!(is_ptr, "PTR", "");
                fmt::format(format_args!("GETVARPARTIREF {} {:?}", ptr, ops[base]))
            },
            
            &Instruction_::Fence(order) => {
                fmt::format(format_args!("FENCE {:?}", order))
            },
            
            &Instruction_::Return(ref vals) => fmt::format(format_args!("RET {}", op_vector_str(vals, ops))),
            &Instruction_::ThreadExit => "THREADEXIT".to_string(),
            &Instruction_::Throw(ref vals) => fmt::format(format_args!("THROW {}", op_vector_str(vals, ops))),
            &Instruction_::TailCall(ref call) => fmt::format(format_args!("TAILCALL {}", call.debug_str(ops))),
            &Instruction_::Branch1(ref dest) => fmt::format(format_args!("BRANCH {:?}", dest.debug_str(ops))),
            &Instruction_::Branch2{cond, ref true_dest, ref false_dest} => {
                fmt::format(format_args!("BRANCH2 {:?} {} {}", ops[cond], true_dest.debug_str(ops), false_dest.debug_str(ops)))
            },
            &Instruction_::Watchpoint{id, ref disable_dest, ref resume} => {
                match id {
                    Some(id) => {
                        fmt::format(format_args!("WATCHPOINT {:?} {} {}", id, disable_dest.as_ref().unwrap().debug_str(ops), resume.debug_str(ops)))
                    },
                    None => {
                        fmt::format(format_args!("TRAP {}", resume.debug_str(ops)))
                    }
                }
            },
            &Instruction_::WPBranch{wp, ref disable_dest, ref enable_dest} => {
                fmt::format(format_args!("WPBRANCH {:?} {} {}", wp, disable_dest.debug_str(ops), enable_dest.debug_str(ops)))
            },
            &Instruction_::Call{ref data, ref resume} => fmt::format(format_args!("CALL {} {}", data.debug_str(ops), resume.debug_str(ops))),
            &Instruction_::SwapStack{stack, is_exception, ref args, ref resume} => {
                fmt::format(format_args!("SWAPSTACK {:?} {:?} {} {}", ops[stack], is_exception, op_vector_str(args, ops), resume.debug_str(ops)))
            },
            &Instruction_::Switch{cond, ref default, ref branches} => {
                let mut ret = fmt::format(format_args!("SWITCH {:?} {:?} {{", cond, default.debug_str(ops)));
                for i in 0..branches.len() {
                    let (op, ref dest) = branches[i];
                    ret.push_str(fmt::format(format_args!("{:?} {}", ops[op], dest.debug_str(ops))).as_str());
                    if i != branches.len() - 1 {
                        ret.push_str(", ");
                    }
                }
                ret.push_str("}}");
                
                ret
            },
            &Instruction_::ExnInstruction{ref inner, ref resume} => {
                fmt::format(format_args!("{:?} {:?}", inner.debug_str(ops), resume.debug_str(ops)))
            }
        }
    }    
}

impl fmt::Debug for Instruction_ {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Instruction_::BinOp(op, ref op1, ref op2) => write!(f, "{:?} {:?} {:?}", op, op1, op2),
            &Instruction_::CmpOp(op, ref op1, ref op2) => write!(f, "{:?} {:?} {:?}", op, op1, op2),
            &Instruction_::ExprCall{ref data, is_abort} => {
                let abort = select!(is_abort, "ABORT_ON_EXN", "RETHROW");
                write!(f, "CALL {:?} {}", data, abort)
            },
            &Instruction_::Load{is_ptr, ref mem_loc, order} => {
                let ptr = select!(is_ptr, "PTR", "");
                write!(f, "LOAD {} {:?} {:?}", ptr, order, mem_loc) 
            },
            &Instruction_::Store{ref value, is_ptr, ref mem_loc, order} => {
                let ptr = select!(is_ptr, "PTR", "");
                write!(f, "STORE {} {:?} {:?} {:?}", ptr, order, mem_loc, value)
            },
            &Instruction_::CmpXchg{is_ptr, is_weak, success_order, fail_order, 
                ref mem_loc, ref expected_value, ref desired_value} => {
                let ptr = select!(is_ptr, "PTR", "");
                let weak = select!(is_weak, "WEAK", "");
                write!(f, "CMPXCHG {} {} {:?} {:?} {:?} {:?} {:?}", 
                    ptr, weak, success_order, fail_order, mem_loc, expected_value, desired_value)  
            },
            &Instruction_::AtomicRMW{is_ptr, order, op, ref mem_loc, ref value} => {
                let ptr = select!(is_ptr, "PTR", "");
                write!(f, "ATOMICRMW {} {:?} {:?} {:?} {:?}", ptr, order, op, mem_loc, value)
            },
            &Instruction_::New(ref ty) => write!(f, "NEW {:?}", ty),
            &Instruction_::AllocA(ref ty) => write!(f, "ALLOCA {:?}", ty),
            &Instruction_::NewHybrid(ref ty, ref len) => write!(f, "NEWHYBRID {:?} {:?}", ty, len),
            &Instruction_::AllocAHybrid(ref ty, ref len) => write!(f, "ALLOCAHYBRID {:?} {:?}", ty, len),
            &Instruction_::NewStack(ref func) => write!(f, "NEWSTACK {:?}", func),
            &Instruction_::NewThread(ref stack, ref args) => write!(f, "NEWTHREAD {:?} PASS_VALUES {:?}", stack, args),
            &Instruction_::NewThreadExn(ref stack, ref exn) => write!(f, "NEWTHREAD {:?} THROW_EXC {:?}", stack, exn),
            &Instruction_::NewFrameCursor(ref stack) => write!(f, "NEWFRAMECURSOR {:?}", stack),
            &Instruction_::GetIRef(ref reference) => write!(f, "GETIREF {:?}", reference),
            &Instruction_::GetFieldIRef{is_ptr, ref base, ref index} => {
                let ptr = select!(is_ptr, "PTR", "");
                write!(f, "GETFIELDIREF {} {:?} {:?}", ptr, base, index)
            },
            &Instruction_::GetElementIRef{is_ptr, ref base, ref index} => {
                let ptr = select!(is_ptr, "PTR", "");
                write!(f, "GETELEMENTIREF {} {:?} {:?}", ptr, base, index)
            },
            &Instruction_::ShiftIRef{is_ptr, ref base, ref offset} => {
                let ptr = select!(is_ptr, "PTR", "");
                write!(f, "SHIFTIREF {} {:?} {:?}", ptr, base, offset)
            },
            &Instruction_::GetVarPartIRef{is_ptr, ref base} => {
                let ptr = select!(is_ptr, "PTR", "");
                write!(f, "GETVARPARTIREF {} {:?}", ptr, base)
            },
            
            &Instruction_::Fence(order) => {
                write!(f, "FENCE {:?}", order)
            }            
            
            &Instruction_::Return(ref vals) => write!(f, "RET {:?}", vals),
            &Instruction_::ThreadExit => write!(f, "THREADEXIT"),
            &Instruction_::Throw(ref vals) => write!(f, "THROW {:?}", vals),
            &Instruction_::TailCall(ref call) => write!(f, "TAILCALL {:?}", call),
            &Instruction_::Branch1(ref dest) => write!(f, "BRANCH {:?}", dest),
            &Instruction_::Branch2{ref cond, ref true_dest, ref false_dest} => {
                write!(f, "BRANCH2 {:?} {:?} {:?}", cond, true_dest, false_dest)
            },
            &Instruction_::Watchpoint{id, ref disable_dest, ref resume} => {
                match id {
                    Some(id) => {
                        write!(f, "WATCHPOINT {:?} {:?} {:?}", id, disable_dest.as_ref().unwrap(), resume)
                    },
                    None => {
                        write!(f, "TRAP {:?}", resume)
                    }
                }
            },
            &Instruction_::WPBranch{wp, ref disable_dest, ref enable_dest} => {
                write!(f, "WPBRANCH {:?} {:?} {:?}", wp, disable_dest, enable_dest)
            },
            &Instruction_::Call{ref data, ref resume} => write!(f, "CALL {:?} {:?}", data, resume),
            &Instruction_::SwapStack{ref stack, is_exception, ref args, ref resume} => {
                write!(f, "SWAPSTACK {:?} {:?} {:?} {:?}", stack, is_exception, args, resume)
            },
            &Instruction_::Switch{ref cond, ref default, ref branches} => {
                write!(f, "SWITCH {:?} {:?} {{{:?}}}", cond, default, branches)
            },
            &Instruction_::ExnInstruction{ref inner, ref resume} => {
                write!(f, "{:?} {:?}", inner, resume)
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
    pub func: OpIndex,
    pub args: Vec<OpIndex>,
    pub convention: CallConvention
}

impl CallData {
    fn debug_str(&self, ops: &Vec<P<TreeNode>>) -> String {
        fmt::format(format_args!("{:?} {:?} ({})", self.convention, ops[self.func], op_vector_str(&self.args, ops)))
    }
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

impl ResumptionData {
    fn debug_str(&self, ops: &Vec<P<TreeNode>>) -> String {
        fmt::format(format_args!("normal: {:?}, exception: {:?}", self.normal_dest.debug_str(ops), self.exn_dest.debug_str(ops)))
    }
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

impl Destination {
    fn debug_str(&self, ops: &Vec<P<TreeNode>>) -> String {
        let mut ret = fmt::format(format_args!("{}", self.target));
        ret.push('[');
        for i in 0..self.args.len() {
            let ref arg = self.args[i];
            ret.push_str(arg.debug_str(ops).as_str());
            if i != self.args.len() - 1 {
                ret.push_str(", ");
            }
        }
        ret.push(']');
        
        ret
    }
}

impl fmt::Debug for Destination {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{:?}", self.target, self.args)
    }
}

#[derive(Clone)]
pub enum DestArg {
    Normal(OpIndex),
    Freshbound(usize)
}

impl DestArg {
    fn debug_str(&self, ops: &Vec<P<TreeNode>>) -> String {
        match self {
            &DestArg::Normal(index) => fmt::format(format_args!("{:?}", ops[index])),
            &DestArg::Freshbound(n) => fmt::format(format_args!("${:?}", n)) 
        }
    }
}

impl fmt::Debug for DestArg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &DestArg::Normal(ref pv) => write!(f, "{:?}", pv),
            &DestArg::Freshbound(n) => write!(f, "${}", n)
        }
    }    
}

fn op_vector_str(vec: &Vec<OpIndex>, ops: &Vec<P<TreeNode>>) -> String {
    let mut ret = String::new();
    for i in 0..vec.len() {
        let index = vec[i];
        ret.push_str(fmt::format(format_args!("{:?}", ops[index])).as_str());
        if i != vec.len() - 1 {
            ret.push_str(", ");
        }
    }
    ret
}