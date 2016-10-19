use ir::*;
use ptr::*;
use types::*;
use op::*;

use utils::vec_utils;

use std::fmt;
use std::sync::RwLock;

#[derive(Debug)]
pub struct Instruction {
    pub hdr: MuEntityHeader,
    pub value : Option<Vec<P<Value>>>,
    pub ops : RwLock<Vec<P<TreeNode>>>,
    pub v: Instruction_
}

impl_mu_entity!(Instruction);

use rustc_serialize::{Encodable, Encoder, Decodable, Decoder};
impl Encodable for Instruction {
    fn encode<S: Encoder> (&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_struct("Instruction", 4, |s| {
            try!(s.emit_struct_field("hdr", 0, |s| self.hdr.encode(s)));
            try!(s.emit_struct_field("value", 1, |s| self.value.encode(s)));
            
            let ops = &self.ops.read().unwrap();
            try!(s.emit_struct_field("ops", 2, |s| ops.encode(s)));
            
            try!(s.emit_struct_field("v", 3, |s| self.v.encode(s)));
            
            Ok(()) 
        })        
    }
}

impl Decodable for Instruction {
    fn decode<D: Decoder>(d: &mut D) -> Result<Instruction, D::Error> {
        d.read_struct("Instruction", 4, |d| {
            let hdr = try!(d.read_struct_field("hdr", 0, |d| Decodable::decode(d)));
            let value = try!(d.read_struct_field("value", 1, |d| Decodable::decode(d)));
            
            let ops = try!(d.read_struct_field("ops", 2, |d| Decodable::decode(d)));
            
            let v = try!(d.read_struct_field("v", 3, |d| Decodable::decode(d)));
            
            Ok(Instruction{
                hdr: hdr,
                value: value,
                ops: RwLock::new(ops),
                v: v
            })
        })
    }
}

impl Clone for Instruction {
    fn clone(&self) -> Self {
        Instruction {
            hdr: self.hdr.clone(),
            value: self.value.clone(),
            ops: RwLock::new(self.ops.read().unwrap().clone()),
            v: self.v.clone()
        }
    }
}

impl Instruction {
    fn debug_str(&self, ops: &Vec<P<TreeNode>>) -> String {
        self.v.debug_str(ops)
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ops = &self.ops.read().unwrap();
        if self.value.is_some() {
            write!(f, "{} = {}", vec_utils::as_str(self.value.as_ref().unwrap()), self.v.debug_str(ops))
        } else {
            write!(f, "{}", self.v.debug_str(ops))
        }
    }
}

#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
pub enum Instruction_ {
    // non-terminal instruction

    // expressions

    BinOp(BinOp, OpIndex, OpIndex),
    CmpOp(CmpOp, OpIndex, OpIndex),

    // yields a tuple of results from the call
    ExprCall{
        data: CallData,
        is_abort: bool, // T to abort, F to rethrow - FIXME: current, always rethrow for now
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
    Throw(OpIndex),
    TailCall(CallData),
    Branch1(Destination),
    Branch2{
        cond: OpIndex,
        true_dest: Destination,
        false_dest: Destination,
        true_prob: f32
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
    CCall{
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
        inner: Box<Instruction>,
        resume: ResumptionData
    }
}

impl Instruction_ {
    fn debug_str(&self, ops: &Vec<P<TreeNode>>) -> String {
        match self {
            &Instruction_::BinOp(op, op1, op2) => format!("{:?} {} {}", op, ops[op1], ops[op2]),
            &Instruction_::CmpOp(op, op1, op2) => format!("{:?} {} {}", op, ops[op1], ops[op2]),
            &Instruction_::ExprCall{ref data, is_abort} => {
                let abort = select_value!(is_abort, "ABORT_ON_EXN", "RETHROW");
                format!("CALL {} {}", data.debug_str(ops), abort)
            },
            &Instruction_::Load{is_ptr, mem_loc, order} => {
                let ptr = select_value!(is_ptr, "PTR", "");
                format!("LOAD {} {:?} {}", ptr, order, ops[mem_loc])
            },
            &Instruction_::Store{value, is_ptr, mem_loc, order} => {
                let ptr = select_value!(is_ptr, "PTR", "");
                format!("STORE {} {:?} {} {}", ptr, order, ops[mem_loc], ops[value])
            },
            &Instruction_::CmpXchg{is_ptr, is_weak, success_order, fail_order,
                mem_loc, expected_value, desired_value} => {
                let ptr = select_value!(is_ptr, "PTR", "");
                let weak = select_value!(is_weak, "WEAK", "");
                format!("CMPXCHG {} {} {:?} {:?} {} {} {}",
                    ptr, weak, success_order, fail_order, ops[mem_loc], ops[expected_value], ops[desired_value])
            },
            &Instruction_::AtomicRMW{is_ptr, order, op, mem_loc, value} => {
                let ptr = select_value!(is_ptr, "PTR", "");
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
                let ptr = select_value!(is_ptr, "PTR", "");
                format!("GETFIELDIREF {} {} {}", ptr, ops[base], ops[index])
            },
            &Instruction_::GetElementIRef{is_ptr, base, index} => {
                let ptr = select_value!(is_ptr, "PTR", "");
                format!("GETELEMENTIREF {} {} {}", ptr, ops[base], ops[index])
            },
            &Instruction_::ShiftIRef{is_ptr, base, offset} => {
                let ptr = select_value!(is_ptr, "PTR", "");
                format!("SHIFTIREF {} {} {}", ptr, ops[base], ops[offset])
            },
            &Instruction_::GetVarPartIRef{is_ptr, base} => {
                let ptr = select_value!(is_ptr, "PTR", "");
                format!("GETVARPARTIREF {} {}", ptr, ops[base])
            },

            &Instruction_::Fence(order) => {
                format!("FENCE {:?}", order)
            },

            &Instruction_::Return(ref vals) => format!("RET {}", op_vector_str(vals, ops)),
            &Instruction_::ThreadExit => "THREADEXIT".to_string(),
            &Instruction_::Throw(exn_obj) => format!("THROW {}", ops[exn_obj]),
            &Instruction_::TailCall(ref call) => format!("TAILCALL {}", call.debug_str(ops)),
            &Instruction_::Branch1(ref dest) => format!("BRANCH {}", dest.debug_str(ops)),
            &Instruction_::Branch2{cond, ref true_dest, ref false_dest, ..} => {
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
            &Instruction_::CCall{ref data, ref resume} => format!("CALL {} {}", data.debug_str(ops), resume.debug_str(ops)),
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

#[derive(Copy, Clone, Debug, RustcEncodable, RustcDecodable)]
pub enum MemoryOrder {
    NotAtomic,
    Relaxed,
    Consume,
    Acquire,
    Release,
    AcqRel,
    SeqCst
}

#[derive(Copy, Clone, Debug, RustcEncodable, RustcDecodable)]
pub enum CallConvention {
    Mu,
    Foreign(ForeignFFI)
}

#[derive(Copy, Clone, Debug, RustcEncodable, RustcDecodable)]
pub enum ForeignFFI {
    C
}

#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
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

#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct ResumptionData {
    pub normal_dest: Destination,
    pub exn_dest: Destination
}

impl ResumptionData {
    fn debug_str(&self, ops: &Vec<P<TreeNode>>) -> String {
        format!("normal: {}, exception: {}", self.normal_dest.debug_str(ops), self.exn_dest.debug_str(ops))
    }
}

#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct Destination {
    pub target: MuID,
    pub args: Vec<DestArg>
}

impl Destination {
    fn debug_str(&self, ops: &Vec<P<TreeNode>>) -> String {
        let mut ret = format!("{} with ", self.target);
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
    
    pub fn get_arguments(&self, ops: &Vec<P<TreeNode>>) -> Vec<P<Value>> {
        vec_utils::map(&self.args, 
            |x| {
                match x {
                    &DestArg::Normal(i) => ops[i].clone_value(),
                    &DestArg::Freshbound(_) => unimplemented!()
                }
        })
    }
}

#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
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