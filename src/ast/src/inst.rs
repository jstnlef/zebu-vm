// Copyright 2017 The Australian National University
// 
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// 
//     http://www.apache.org/licenses/LICENSE-2.0
// 
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use ir::*;
use ptr::*;
use types::*;
use op::*;
use ir_semantics;

use utils::vec_utils;

use std::fmt;

#[derive(Debug)]
// this implements Clone and Display
pub struct Instruction {
    pub hdr: MuEntityHeader,
    pub value : Option<Vec<P<Value>>>,
    pub ops : Vec<P<TreeNode>>,
    pub v: Instruction_
}

impl_mu_entity!(Instruction);

impl Clone for Instruction {
    fn clone(&self) -> Self {
        Instruction {
            hdr: self.hdr.clone(),
            value: self.value.clone(),
            ops: self.ops.clone(),
            v: self.v.clone()
        }
    }
}

impl Instruction {
    pub fn clone_with_id(&self, new_id: MuID) -> Instruction {
        let mut clone = self.clone();
        clone.hdr = self.hdr.clone_with_id(new_id);

        clone
    }

    pub fn has_exception_clause(&self) -> bool {
        ir_semantics::is_potentially_excepting_instruction(&self.v)
    }

    pub fn get_exception_target(&self) -> Option<MuID> {
        use inst::Instruction_::*;
        match self.v {
            Watchpoint {ref resume, ..}
            | Call {ref resume, ..}
            | CCall {ref resume, ..}
            | SwapStack {ref resume, ..}
            | ExnInstruction {ref resume, ..} => {
                Some(resume.exn_dest.target)
            },

            _ => None
        }
    }

    fn debug_str(&self, ops: &Vec<P<TreeNode>>) -> String {
        self.v.debug_str(ops)
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ref ops = self.ops;
        if self.value.is_some() {
            write!(f, "{} = {}", vec_utils::as_str(self.value.as_ref().unwrap()), self.v.debug_str(ops))
        } else {
            write!(f, "{}", self.v.debug_str(ops))
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum Instruction_ {
    // non-terminal instruction

    // expressions

    BinOp(BinOp, OpIndex, OpIndex),
    BinOpWithStatus(BinOp, BinOpStatus, OpIndex, OpIndex),

    CmpOp(CmpOp, OpIndex, OpIndex),
    ConvOp{
        operation: ConvOp,
        from_ty: P<MuType>,
        to_ty: P<MuType>,
        operand: OpIndex
    },

    // yields a tuple of results from the call
    ExprCall{
        data: CallData,
        is_abort: bool, // T to abort, F to rethrow - FIXME: current, always rethrow for now
    },

    ExprCCall{
        data: CallData,
        is_abort: bool
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
        index: usize // constant
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
    Select{
        cond: OpIndex,
        true_val: OpIndex,
        false_val: OpIndex
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
    },

    // common inst
    CommonInst_GetThreadLocal,
    CommonInst_SetThreadLocal(OpIndex),

    // pin/unpin
    CommonInst_Pin(OpIndex),
    CommonInst_Unpin(OpIndex),

    // Tagref64 stuff
    CommonInst_Tr64IsFp(OpIndex),
    CommonInst_Tr64IsInt(OpIndex),
    CommonInst_Tr64IsRef(OpIndex),
    CommonInst_Tr64FromFp(OpIndex),
    CommonInst_Tr64FromInt(OpIndex),
    CommonInst_Tr64FromRef(OpIndex, OpIndex),
    CommonInst_Tr64ToFp(OpIndex),
    CommonInst_Tr64ToInt(OpIndex),
    CommonInst_Tr64ToRef(OpIndex),
    CommonInst_Tr64ToTag(OpIndex),

    // internal use: mov from ops[0] to value
    Move(OpIndex),
    // internal use: print op as hex value
    PrintHex(OpIndex),
    // internal use: set return value for main
    SetRetval(OpIndex),
}

impl Instruction_ {
    fn debug_str(&self, ops: &Vec<P<TreeNode>>) -> String {
        match self {
            &Instruction_::BinOp(op, op1, op2) => format!("{:?} {} {}", op, ops[op1], ops[op2]),
            &Instruction_::BinOpWithStatus(op, status, op1, op2) => {
                format!("{:?} {:?} {} {}", op, status, ops[op1], ops[op2])
            }
            &Instruction_::CmpOp(op, op1, op2) => format!("{:?} {} {}", op, ops[op1], ops[op2]),
            &Instruction_::ConvOp{operation, ref from_ty, ref to_ty, operand} => {
                format!("{:?} {} {} {}", operation, from_ty, to_ty, ops[operand])
            }
            &Instruction_::ExprCall{ref data, is_abort} => {
                let abort = select_value!(is_abort, "ABORT_ON_EXN", "RETHROW");
                format!("CALL {} {}", data.debug_str(ops), abort)
            },
            &Instruction_::ExprCCall{ref data, is_abort} => {
                let abort = select_value!(is_abort, "ABORT_ON_EXN", "RETHROW");
                format!("CCALL {} {}", data.debug_str(ops), abort)
            }
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
                format!("GETFIELDIREF {} {} {}", ptr, ops[base], index)
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
            &Instruction_::Select{cond, true_val, false_val} => {
                format!("SELECT if {} then {} else {}", ops[cond], ops[true_val], ops[false_val])
            }
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
            &Instruction_::CCall{ref data, ref resume} => format!("CCALL {} {}", data.debug_str(ops), resume.debug_str(ops)),
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
            },

            // common inst
            &Instruction_::CommonInst_GetThreadLocal => format!("COMMONINST GetThreadLocal"),
            &Instruction_::CommonInst_SetThreadLocal(op) => format!("COMMONINST SetThreadLocal {}", ops[op]),

            &Instruction_::CommonInst_Pin(op)   => format!("COMMONINST Pin {}",   ops[op]),
            &Instruction_::CommonInst_Unpin(op) => format!("COMMONINST Unpin {}", ops[op]),

            // Tagerf64
            &Instruction_::CommonInst_Tr64IsFp(op) => format!("COMMONINST Tr64IsFp {}", ops[op]),
            &Instruction_::CommonInst_Tr64IsInt(op) => format!("COMMONINST Tr64IsInt {}", ops[op]),
            &Instruction_::CommonInst_Tr64IsRef(op) => format!("COMMONINST Tr64IsRef {}", ops[op]),
            &Instruction_::CommonInst_Tr64FromFp(op) => format!("COMMONINST Tr64FromFp {}", ops[op]),
            &Instruction_::CommonInst_Tr64FromInt(op) => format!("COMMONINST Tr64FromInt {}", ops[op]),
            &Instruction_::CommonInst_Tr64FromRef(op1, op2) => format!("COMMONINST Tr64FromRet {} {}", ops[op1], ops[op2]),
            &Instruction_::CommonInst_Tr64ToFp(op) => format!("COMMONINST Tr64ToFp {}", ops[op]),
            &Instruction_::CommonInst_Tr64ToInt(op) => format!("COMMONINST Tr64ToInt {}", ops[op]),
            &Instruction_::CommonInst_Tr64ToRef(op) => format!("COMMONINST Tr64ToRef {}", ops[op]),
            &Instruction_::CommonInst_Tr64ToTag(op) => format!("COMMONINST Tr64ToTag {}", ops[op]),

            // move
            &Instruction_::Move(from) => format!("MOVE {}", ops[from]),
            // print hex
            &Instruction_::PrintHex(i) => format!("PRINTHEX {}", ops[i]),
            // set retval
            &Instruction_::SetRetval(val) => format!("SETRETVAL {}", ops[val]),
        }
    }
}

#[derive(Copy, Clone)]
pub struct BinOpStatus {
    pub flag_n: bool,
    pub flag_z: bool,
    pub flag_c: bool,
    pub flag_v: bool
}

impl BinOpStatus {
    pub fn none() -> BinOpStatus {
        BinOpStatus {flag_n: false, flag_z: false, flag_c: false, flag_v: false}
    }

    pub fn n() -> BinOpStatus {
        BinOpStatus {flag_n: true, flag_z: false, flag_c: false, flag_v: false}
    }

    pub fn z() -> BinOpStatus {
        BinOpStatus {flag_n: false, flag_z: true, flag_c: false, flag_v: false}
    }

    pub fn c() -> BinOpStatus {
        BinOpStatus {flag_n: false, flag_z: false, flag_c: true, flag_v: false}
    }

    pub fn v() -> BinOpStatus {
        BinOpStatus {flag_n: false, flag_z: false, flag_c: false, flag_v: true}
    }
}

impl fmt::Debug for BinOpStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.flag_n {
            write!(f, "#N").unwrap();
        }
        if self.flag_z {
            write!(f, "#Z").unwrap();
        }
        if self.flag_c {
            write!(f, "#C").unwrap();
        }
        if self.flag_v {
            write!(f, "#V").unwrap();
        }
        Ok(())
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
        let func_name = ops[self.func].name();
        format!("{:?} {} [{}]", self.convention, func_name, op_vector_str(&self.args, ops))
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
    pub target: MuID,
    pub args: Vec<DestArg>
}

impl Destination {
    fn debug_str(&self, ops: &Vec<P<TreeNode>>) -> String {
        let mut ret = format!("{}", self.target);
        ret.push('(');
        for i in 0..self.args.len() {
            let ref arg = self.args[i];
            ret.push_str(arg.debug_str(ops).as_str());
            if i != self.args.len() - 1 {
                ret.push_str(", ");
            }
        }
        ret.push(')');

        ret
    }

    pub fn get_arguments_as_node(&self, ops: &Vec<P<TreeNode>>) -> Vec<P<TreeNode>> {
       vec_utils::map(&self.args,
           |x| {
               match x {
                   &DestArg::Normal(i) => ops[i].clone(),
                   &DestArg::Freshbound(_) => unimplemented!()
               }
       })
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
