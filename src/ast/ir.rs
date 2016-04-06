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

#[derive(Debug, Clone)]
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

/// use +() to display a node
impl fmt::Display for TreeNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.v {
            TreeNode_::Value(ref pv) => {
                match pv.v {
                    Value_::SSAVar(id) => {
                        write!(f, "+({} %{}#{})", pv.ty, pv.tag, id)
                    },
                    Value_::Constant(ref c) => {
                        write!(f, "+({} {})", pv.ty, c) 
                    }
                }
            },
            TreeNode_::Instruction(ref inst) => {
                write!(f, "+({})", inst)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum TreeNode_ {
    Value(P<Value>),
    Instruction(Instruction)
}

/// always use with P<Value>
#[derive(Debug, Clone)]
pub struct Value {
    pub tag: MuTag,
    pub ty: P<MuType>,
    pub v: Value_
}

#[derive(Debug, Clone)]
pub enum Value_ {
    SSAVar(MuID),
    Constant(Constant)
}

#[derive(Debug, Clone)]
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

impl fmt::Display for ValueEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}#{}", self.ty, self.tag, self.id)
    }
}

#[derive(Debug, Clone)]
pub enum Constant {
    Int(usize),
    Float(f32),
    Double(f64),
    IRef(Address),
    FuncRef(Address),
    UFuncRef(Address),
    Vector(Vec<Constant>),    
}

impl fmt::Display for Constant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Constant::Int(v) => write!(f, "{}", v),
            &Constant::Float(v) => write!(f, "{}", v),
            &Constant::Double(v) => write!(f, "{}", v),
            &Constant::IRef(v) => write!(f, "{}", v),
            &Constant::FuncRef(v) => write!(f, "{}", v),
            &Constant::UFuncRef(v) => write!(f, "{}", v),
            &Constant::Vector(ref v) => {
                write!(f, "[").unwrap();
                for i in 0..v.len() {
                    write!(f, "{}", v[i]).unwrap();
                    if i != v.len() - 1 {
                        write!(f, ", ").unwrap();
                    }
                }
                write!(f, "]")
            }
        }
    }
}

#[derive(Debug, Clone)]
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

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ops = &self.ops.borrow();
        if self.value.is_some() {
            write!(f, "{} = {}", node_vector_str(self.value.as_ref().unwrap()), self.v.debug_str(ops))
        } else {
            write!(f, "{}", self.v.debug_str(ops))
        }
    }
}

#[derive(Debug, Clone)]
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
            &Instruction_::BinOp(op, op1, op2) => format!("{:?} {} {}", op, ops[op1], ops[op2]),
            &Instruction_::CmpOp(op, op1, op2) => format!("{:?} {} {}", op, ops[op1], ops[op2]),
            &Instruction_::ExprCall{ref data, is_abort} => {
                let abort = select!(is_abort, "ABORT_ON_EXN", "RETHROW");
                format!("CALL {} {}", data.debug_str(ops), abort)
            },
            &Instruction_::Load{is_ptr, mem_loc, order} => {
                let ptr = select!(is_ptr, "PTR", "");
                format!("LOAD {} {:?} {}", ptr, order, ops[mem_loc]) 
            },
            &Instruction_::Store{value, is_ptr, mem_loc, order} => {
                let ptr = select!(is_ptr, "PTR", "");
                format!("STORE {} {:?} {} {}", ptr, order, ops[mem_loc], ops[value])
            },
            &Instruction_::CmpXchg{is_ptr, is_weak, success_order, fail_order, 
                mem_loc, expected_value, desired_value} => {
                let ptr = select!(is_ptr, "PTR", "");
                let weak = select!(is_weak, "WEAK", "");
                format!("CMPXCHG {} {} {:?} {:?} {} {} {}", 
                    ptr, weak, success_order, fail_order, ops[mem_loc], ops[expected_value], ops[desired_value])  
            },
            &Instruction_::AtomicRMW{is_ptr, order, op, mem_loc, value} => {
                let ptr = select!(is_ptr, "PTR", "");
                format!("ATOMICRMW {} {:?} {:?} {} {}", ptr, order, op, ops[mem_loc], ops[value])
            },
            &Instruction_::New(ref ty) => format!("NEW {}", ty),
            &Instruction_::AllocA(ref ty) => format!("ALLOCA {}", ty),
            &Instruction_::NewHybrid(ref ty, len) => format!("NEWHYBRID {} {}", ty, ops[len]),
            &Instruction_::AllocAHybrid(ref ty, len) => format!("ALLOCAHYBRID {} {}", ty, ops[len]),
            &Instruction_::NewStack(func) => format!("NEWSTACK {}", ops[func]),
            &Instruction_::NewThread(stack, ref args) => format!("NEWTHREAD {} PASS_VALUES {}", ops[stack], op_vector_str(args, ops)),
            &Instruction_::NewThreadExn(stack, exn) => format!("NEWTHREAD {} THROW_EXC {}", ops[stack], ops[exn]),
            &Instruction_::NewFrameCursor(stack) => format!("NEWFRAMECURSOR {}", ops[stack]),
            &Instruction_::GetIRef(reference) => format!("GETIREF {}", ops[reference]),
            &Instruction_::GetFieldIRef{is_ptr, base, index} => {
                let ptr = select!(is_ptr, "PTR", "");
                format!("GETFIELDIREF {} {} {}", ptr, ops[base], ops[index])
            },
            &Instruction_::GetElementIRef{is_ptr, base, index} => {
                let ptr = select!(is_ptr, "PTR", "");
                format!("GETELEMENTIREF {} {} {}", ptr, ops[base], ops[index])
            },
            &Instruction_::ShiftIRef{is_ptr, base, offset} => {
                let ptr = select!(is_ptr, "PTR", "");
                format!("SHIFTIREF {} {} {}", ptr, ops[base], ops[offset])
            },
            &Instruction_::GetVarPartIRef{is_ptr, base} => {
                let ptr = select!(is_ptr, "PTR", "");
                format!("GETVARPARTIREF {} {}", ptr, ops[base])
            },
            
            &Instruction_::Fence(order) => {
                format!("FENCE {:?}", order)
            },
            
            &Instruction_::Return(ref vals) => format!("RET {}", op_vector_str(vals, ops)),
            &Instruction_::ThreadExit => "THREADEXIT".to_string(),
            &Instruction_::Throw(ref vals) => format!("THROW {}", op_vector_str(vals, ops)),
            &Instruction_::TailCall(ref call) => format!("TAILCALL {}", call.debug_str(ops)),
            &Instruction_::Branch1(ref dest) => format!("BRANCH {}", dest.debug_str(ops)),
            &Instruction_::Branch2{cond, ref true_dest, ref false_dest} => {
                format!("BRANCH2 {} {} {}", ops[cond], true_dest.debug_str(ops), false_dest.debug_str(ops))
            },
            &Instruction_::Watchpoint{id, ref disable_dest, ref resume} => {
                match id {
                    Some(id) => {
                        format!("WATCHPOINT {} {} {}", id, disable_dest.as_ref().unwrap().debug_str(ops), resume.debug_str(ops))
                    },
                    None => {
                        format!("TRAP {}", resume.debug_str(ops))
                    }
                }
            },
            &Instruction_::WPBranch{wp, ref disable_dest, ref enable_dest} => {
                format!("WPBRANCH {} {} {}", wp, disable_dest.debug_str(ops), enable_dest.debug_str(ops))
            },
            &Instruction_::Call{ref data, ref resume} => format!("CALL {} {}", data.debug_str(ops), resume.debug_str(ops)),
            &Instruction_::SwapStack{stack, is_exception, ref args, ref resume} => {
                format!("SWAPSTACK {} {} {} {}", ops[stack], is_exception, op_vector_str(args, ops), resume.debug_str(ops))
            },
            &Instruction_::Switch{cond, ref default, ref branches} => {
                let mut ret = format!("SWITCH {} {} {{", ops[cond], default.debug_str(ops));
                for i in 0..branches.len() {
                    let (op, ref dest) = branches[i];
                    ret.push_str(format!("{} {}", ops[op], dest.debug_str(ops)).as_str());
                    if i != branches.len() - 1 {
                        ret.push_str(", ");
                    }
                }
                ret.push_str("}}");
                
                ret
            },
            &Instruction_::ExnInstruction{ref inner, ref resume} => {
                format!("{} {}", inner.debug_str(ops), resume.debug_str(ops))
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

#[derive(Clone, Debug)]
pub struct CallData {
    pub func: OpIndex,
    pub args: Vec<OpIndex>,
    pub convention: CallConvention
}

impl CallData {
    fn debug_str(&self, ops: &Vec<P<TreeNode>>) -> String {
        format!("{:?} {} [{}]", self.convention, ops[self.func], op_vector_str(&self.args, ops))
    }
}

#[derive(Clone, Debug)]
pub struct ResumptionData {
    pub normal_dest: Destination,
    pub exn_dest: Destination
}

impl ResumptionData {
    fn debug_str(&self, ops: &Vec<P<TreeNode>>) -> String {
        format!("normal: {}, exception: {}", self.normal_dest.debug_str(ops), self.exn_dest.debug_str(ops))
    }
}

#[derive(Clone, Debug)]
pub struct Destination {
    pub target: MuTag,
    pub args: Vec<DestArg>
}

impl Destination {
    fn debug_str(&self, ops: &Vec<P<TreeNode>>) -> String {
        let mut ret = format!("{}", self.target);
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

#[derive(Clone, Debug)]
pub enum DestArg {
    Normal(OpIndex),
    Freshbound(usize)
}

impl DestArg {
    fn debug_str(&self, ops: &Vec<P<TreeNode>>) -> String {
        match self {
            &DestArg::Normal(index) => format!("{}", ops[index]),
            &DestArg::Freshbound(n) => format!("${}", n) 
        }
    }
}

fn op_vector_str(vec: &Vec<OpIndex>, ops: &Vec<P<TreeNode>>) -> String {
    let mut ret = String::new();
    for i in 0..vec.len() {
        let index = vec[i];
        ret.push_str(format!("{}", ops[index]).as_str());
        if i != vec.len() - 1 {
            ret.push_str(", ");
        }
    }
    ret
}

fn node_vector_str(vec: &Vec<P<TreeNode>>) -> String {
    let mut ret = String::new();
    for i in 0..vec.len() {
        ret.push_str(format!("{}", vec[i]).as_str());
        if i != vec.len() - 1 {
            ret.push_str(", ");
        }
    }
    ret
}