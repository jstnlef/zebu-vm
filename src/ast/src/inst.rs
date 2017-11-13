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

use utils::vec_utils;

use std::fmt;

/// Instruction represents a Mu instruction
#[derive(Debug, Clone)] // this implements Display
pub struct Instruction {
    pub hdr: MuEntityHeader,
    /// the values this instruction holds
    pub value: Option<Vec<P<Value>>>,
    /// ops field list all the children nodes,
    /// and in Instruction_, the children nodes are referred by indices
    /// This design makes it easy for the compiler to iterate through all the children
    pub ops: Vec<P<TreeNode>>,
    /// used for pattern matching
    pub v: Instruction_
}

// Instruction implements MuEntity
impl_mu_entity!(Instruction);

impl Instruction {
    pub fn clone_with_id(&self, new_id: MuID) -> Instruction {
        let mut clone = self.clone();
        clone.hdr = self.hdr.clone_with_id(new_id);

        clone
    }

    /// is this instruction the terminal inst of its block?
    /// Terminal instructions end Mu blocks, and Mu block ends with a terminal instruction.
    pub fn is_terminal_inst(&self) -> bool {
        use inst::Instruction_::*;

        match self.v {
            Return(_) |
            ThreadExit |
            Throw(_) |
            TailCall(_) |
            Branch1(_) |
            Branch2 { .. } |
            Watchpoint { .. } |
            WPBranch { .. } |
            Call { .. } |
            CCall { .. } |
            SwapStackExc { .. } |
            SwapStackKill { .. } |
            Switch { .. } |
            ExnInstruction { .. } => true,
            BinOp(_, _, _) |
            BinOpWithStatus(_, _, _, _) |
            CmpOp(_, _, _) |
            ConvOp { .. } |
            ExprCall { .. } |
            ExprCCall { .. } |
            Load { .. } |
            Store { .. } |
            CmpXchg { .. } |
            AtomicRMW { .. } |
            New(_) |
            AllocA(_) |
            NewHybrid(_, _) |
            AllocAHybrid(_, _) |
            NewStack(_) |
            NewThread { .. } |
            NewFrameCursor(_) |
            GetIRef(_) |
            GetFieldIRef { .. } |
            GetElementIRef { .. } |
            ShiftIRef { .. } |
            GetVarPartIRef { .. } |
            Select { .. } |
            Fence(_) |
            CommonInst_GetThreadLocal |
            CommonInst_SetThreadLocal(_) |
            CommonInst_Pin(_) |
            CommonInst_Unpin(_) |
            CommonInst_GetAddr(_) |
            CommonInst_Tr64IsFp(_) |
            CommonInst_Tr64IsInt(_) |
            CommonInst_Tr64IsRef(_) |
            CommonInst_Tr64FromFp(_) |
            CommonInst_Tr64FromInt(_) |
            CommonInst_Tr64FromRef(_, _) |
            CommonInst_Tr64ToFp(_) |
            CommonInst_Tr64ToInt(_) |
            CommonInst_Tr64ToRef(_) |
            CommonInst_Tr64ToTag(_) |
            Move(_) |
            PrintHex(_) |
            SetRetval(_) |
            GetVMThreadLocal |
            KillStack(_) |
            CurrentStack |
            SwapStackExpr { .. } => false
        }
    }

    /// is this instruction a non-terminal instruction of its block?
    pub fn is_non_terminal_inst(&self) -> bool {
        !self.is_terminal_inst()
    }

    /// does this instruction has side effect?
    /// An instruction has side effect if it affects something other than its result operands.
    /// e.g. affecting memory, stack, thread, etc.
    // FIXME: need to check correctness
    pub fn has_side_effect(&self) -> bool {
        use inst::Instruction_::*;

        match self.v {
            ExprCall { .. } |
            ExprCCall { .. } |
            Load { .. } |
            Store { .. } |
            CmpXchg { .. } |
            AtomicRMW { .. } |
            New(_) |
            AllocA(_) |
            NewHybrid(_, _) |
            AllocAHybrid(_, _) |
            NewStack(_) |
            NewThread { .. } |
            NewFrameCursor(_) |
            Fence(_) |
            Return(_) |
            ThreadExit |
            Throw(_) |
            TailCall(_) |
            Branch1(_) |
            Branch2 { .. } |
            Watchpoint { .. } |
            WPBranch { .. } |
            Call { .. } |
            CCall { .. } |
            SwapStackExpr { .. } |
            SwapStackExc { .. } |
            SwapStackKill { .. } |
            Switch { .. } |
            ExnInstruction { .. } |
            CommonInst_GetThreadLocal |
            CommonInst_SetThreadLocal(_) |
            CommonInst_Pin(_) |
            CommonInst_Unpin(_) |
            CommonInst_GetAddr(_) |
            PrintHex(_) |
            SetRetval(_) |
            KillStack(_) => true,
            BinOp(_, _, _) |
            BinOpWithStatus(_, _, _, _) |
            CmpOp(_, _, _) |
            ConvOp { .. } |
            GetIRef(_) |
            GetFieldIRef { .. } |
            GetElementIRef { .. } |
            ShiftIRef { .. } |
            GetVarPartIRef { .. } |
            Select { .. } |
            CommonInst_Tr64IsFp(_) |
            CommonInst_Tr64IsInt(_) |
            CommonInst_Tr64IsRef(_) |
            CommonInst_Tr64FromFp(_) |
            CommonInst_Tr64FromInt(_) |
            CommonInst_Tr64FromRef(_, _) |
            CommonInst_Tr64ToFp(_) |
            CommonInst_Tr64ToInt(_) |
            CommonInst_Tr64ToRef(_) |
            CommonInst_Tr64ToTag(_) |
            Move(_) |
            CurrentStack |
            GetVMThreadLocal => false
        }
    }

    /// can this instruction throw exception?
    /// an instruction with an exceptional branch can throw exception
    pub fn is_potentially_excepting_instruction(&self) -> bool {
        use inst::Instruction_::*;

        match self.v {
            Watchpoint { .. } |
            Call { .. } |
            CCall { .. } |
            SwapStackExc { .. } |
            ExnInstruction { .. } => true,
            BinOp(_, _, _) |
            BinOpWithStatus(_, _, _, _) |
            CmpOp(_, _, _) |
            ConvOp { .. } |
            ExprCall { .. } |
            ExprCCall { .. } |
            Load { .. } |
            Store { .. } |
            CmpXchg { .. } |
            AtomicRMW { .. } |
            New(_) |
            AllocA(_) |
            NewHybrid(_, _) |
            AllocAHybrid(_, _) |
            NewStack(_) |
            NewThread { .. } |
            NewFrameCursor(_) |
            GetIRef(_) |
            GetFieldIRef { .. } |
            GetElementIRef { .. } |
            ShiftIRef { .. } |
            GetVarPartIRef { .. } |
            Fence(_) |
            Return(_) |
            ThreadExit |
            Throw(_) |
            TailCall(_) |
            Branch1(_) |
            Branch2 { .. } |
            Select { .. } |
            WPBranch { .. } |
            Switch { .. } |
            CommonInst_GetThreadLocal |
            CommonInst_SetThreadLocal(_) |
            CommonInst_Pin(_) |
            CommonInst_Unpin(_) |
            CommonInst_GetAddr(_) |
            CommonInst_Tr64IsFp(_) |
            CommonInst_Tr64IsInt(_) |
            CommonInst_Tr64IsRef(_) |
            CommonInst_Tr64FromFp(_) |
            CommonInst_Tr64FromInt(_) |
            CommonInst_Tr64FromRef(_, _) |
            CommonInst_Tr64ToFp(_) |
            CommonInst_Tr64ToInt(_) |
            CommonInst_Tr64ToRef(_) |
            CommonInst_Tr64ToTag(_) |
            Move(_) |
            PrintHex(_) |
            SetRetval(_) |
            GetVMThreadLocal |
            KillStack(_) |
            CurrentStack |
            SwapStackExpr { .. } |
            SwapStackKill { .. } => false
        }
    }

    /// does this instruction have an exceptional clause/branch?
    pub fn has_exception_clause(&self) -> bool {
        self.is_potentially_excepting_instruction()
    }

    /// returns exception target(block ID),
    /// returns None if this instruction does not have exceptional branch
    pub fn get_exception_target(&self) -> Option<MuID> {
        use inst::Instruction_::*;
        match self.v {
            Watchpoint { ref resume, .. } |
            Call { ref resume, .. } |
            CCall { ref resume, .. } |
            SwapStackExc { ref resume, .. } |
            ExnInstruction { ref resume, .. } => Some(resume.exn_dest.target.id()),
            BinOp(_, _, _) |
            BinOpWithStatus(_, _, _, _) |
            CmpOp(_, _, _) |
            ConvOp { .. } |
            ExprCall { .. } |
            ExprCCall { .. } |
            Load { .. } |
            Store { .. } |
            CmpXchg { .. } |
            AtomicRMW { .. } |
            New(_) |
            AllocA(_) |
            NewHybrid(_, _) |
            AllocAHybrid(_, _) |
            NewStack(_) |
            NewThread { .. } |
            NewFrameCursor(_) |
            GetIRef(_) |
            GetFieldIRef { .. } |
            GetElementIRef { .. } |
            ShiftIRef { .. } |
            GetVarPartIRef { .. } |
            Fence(_) |
            Return(_) |
            ThreadExit |
            Throw(_) |
            TailCall(_) |
            Branch1(_) |
            Branch2 { .. } |
            Select { .. } |
            WPBranch { .. } |
            Switch { .. } |
            CommonInst_GetThreadLocal |
            CommonInst_SetThreadLocal(_) |
            CommonInst_Pin(_) |
            CommonInst_Unpin(_) |
            CommonInst_GetAddr(_) |
            CommonInst_Tr64IsFp(_) |
            CommonInst_Tr64IsInt(_) |
            CommonInst_Tr64IsRef(_) |
            CommonInst_Tr64FromFp(_) |
            CommonInst_Tr64FromInt(_) |
            CommonInst_Tr64FromRef(_, _) |
            CommonInst_Tr64ToFp(_) |
            CommonInst_Tr64ToInt(_) |
            CommonInst_Tr64ToRef(_) |
            CommonInst_Tr64ToTag(_) |
            Move(_) |
            PrintHex(_) |
            SetRetval(_) |
            GetVMThreadLocal |
            KillStack(_) |
            CurrentStack |
            SwapStackExpr { .. } |
            SwapStackKill { .. } => None
        }
    }

    /// can this instruction throw exception?
    /// (whether or not it containjs a ctach for it)
    pub fn is_potentially_throwing(&self) -> bool {
        use inst::Instruction_::*;

        match self.v {
            // Note: commented out ones are ones where we haven't implemented exceptions yet
            Watchpoint { .. } |
            Call { .. } |
            CCall { .. } |
            SwapStackExc { .. } |
            SwapStackExpr { .. } |
            ExnInstruction { .. } |
            ExprCall { .. } |
            ExprCCall { .. } |
            //Load { .. } |
            //Store { .. } |
            //CmpXchg { .. } |
            //AtomicRMW { .. } |
            //New(_) |
            //NewHybrid(_, _) |
            Throw(_)  => true,

            // BinOp(op, _, _) |
            // BinOpWithStatus(op, _, _, _) if op.may_throw() => true,
            BinOp(_, _, _) |
            BinOpWithStatus(_, _, _, _) |
            CmpOp(_, _, _) |
            ConvOp { .. } |
            AllocA(_) |
            AllocAHybrid(_, _) |
            NewStack(_) |
            NewThread { .. } |
            NewFrameCursor(_) |
            GetIRef(_) |
            GetFieldIRef { .. } |
            GetElementIRef { .. } |
            ShiftIRef { .. } |
            GetVarPartIRef { .. } |
            Fence(_) |
            Return(_) |
            ThreadExit |
            TailCall(_) |
            Branch1(_) |
            Branch2 { .. } |
            Select { .. } |
            WPBranch { .. } |
            Switch { .. } |
            CommonInst_GetThreadLocal |
            CommonInst_SetThreadLocal(_) |
            CommonInst_Pin(_) |
            CommonInst_Unpin(_) |
            CommonInst_GetAddr(_) |
            CommonInst_Tr64IsFp(_) |
            CommonInst_Tr64IsInt(_) |
            CommonInst_Tr64IsRef(_) |
            CommonInst_Tr64FromFp(_) |
            CommonInst_Tr64FromInt(_) |
            CommonInst_Tr64FromRef(_, _) |
            CommonInst_Tr64ToFp(_) |
            CommonInst_Tr64ToInt(_) |
            CommonInst_Tr64ToRef(_) |
            CommonInst_Tr64ToTag(_) |
            Move(_) |
            PrintHex(_) |
            SetRetval(_) |
            GetVMThreadLocal |
            KillStack(_) |
            CurrentStack |
            SwapStackKill { .. } => false,

            _ => false
        }
    }

    fn debug_str(&self, ops: &Vec<P<TreeNode>>) -> String {
        match &self.v {
            &Instruction_::BinOp(op, op1, op2) => {
                format!("{}<{}> {} {}", op, ops[op1].ty(), ops[op1], ops[op2])
            }
            &Instruction_::BinOpWithStatus(op, status, op1, op2) => {
                format!(
                    "{}{}<{}> {} {}",
                    op,
                    status,
                    ops[op1].ty(),
                    ops[op1],
                    ops[op2]
                )
            }
            &Instruction_::CmpOp(op, op1, op2) => {
                format!("{}<{}> {} {}", op, ops[op1].ty(), ops[op1], ops[op2])
            }
            &Instruction_::ConvOp {
                operation,
                ref from_ty,
                ref to_ty,
                operand
            } => format!("{} <{} {}> {}", operation, from_ty, to_ty, ops[operand]),
            &Instruction_::ExprCall { ref data, is_abort } => {
                if is_abort {
                    panic!("ABORT is not supported");
                }
                format!("CALL{}", data.debug_str(ops))
            }
            &Instruction_::ExprCCall { ref data, is_abort } => {
                if is_abort {
                    panic!("ABORT is not supported");
                }
                format!("CCALL{}", data.debug_str(ops))
            }
            &Instruction_::Load {
                is_ptr,
                mem_loc,
                order
            } => {
                let ptr = select_value!(is_ptr, " PTR", "");
                format!(
                    "LOAD{} {}<{}> {}",
                    ptr,
                    order,
                    ops[mem_loc].ty().get_referent_ty().unwrap(),
                    ops[mem_loc]
                )
            }
            &Instruction_::Store {
                value,
                is_ptr,
                mem_loc,
                order
            } => {
                let ptr = select_value!(is_ptr, " PTR", "");
                format!(
                    "STORE{} {}<{}> {} {}",
                    ptr,
                    order,
                    ops[mem_loc].ty().get_referent_ty().unwrap(),
                    ops[mem_loc],
                    ops[value]
                )
            }
            &Instruction_::CmpXchg {
                is_ptr,
                is_weak,
                success_order,
                fail_order,
                mem_loc,
                expected_value,
                desired_value
            } => {
                let ptr = select_value!(is_ptr, " PTR", "");
                let weak = select_value!(is_weak, " WEAK", "");
                format!(
                    "CMPXCHG{}{} {} {}<{}> {} {} {}",
                    ptr,
                    weak,
                    success_order,
                    fail_order,
                    ops[mem_loc].ty().get_referent_ty().unwrap(),
                    ops[mem_loc],
                    ops[expected_value],
                    ops[desired_value]
                )
            }
            &Instruction_::AtomicRMW {
                is_ptr,
                order,
                op,
                mem_loc,
                value
            } => {
                let ptr = select_value!(is_ptr, " PTR", "");
                format!(
                    "ATOMICRMW{} {} {}<{}> {} {}",
                    ptr,
                    order,
                    op,
                    ops[mem_loc].ty().get_referent_ty().unwrap(),
                    ops[mem_loc],
                    ops[value]
                )
            }
            &Instruction_::New(ref ty) => format!("NEW<{}>", ty),
            &Instruction_::AllocA(ref ty) => format!("ALLOCA<{}>", ty),
            &Instruction_::NewHybrid(ref ty, len) => {
                format!("NEWHYBRID<{} {}> {}", ty, ops[len].ty(), ops[len])
            }
            &Instruction_::AllocAHybrid(ref ty, len) => {
                format!("ALLOCAHYBRID<{} {}> {}", ty, ops[len].ty(), ops[len])
            }
            &Instruction_::NewStack(func) => {
                format!(
                    "COMMINST @uvm.new_stack<[{}]>({})",
                    ops[func].ty().get_sig().unwrap(),
                    ops[func]
                )
            }
            &Instruction_::NewThread {
                stack,
                thread_local,
                is_exception,
                ref args
            } => {
                let new_stack_clause = format_new_stack_clause(is_exception, args, ops);
                let thread_local = thread_local
                    .map(|t| format!(" THREADLOCAL({})", ops[t]))
                    .unwrap_or("".to_string());
                format!(
                    "NEWSTACK {}{} {}",
                    ops[stack],
                    thread_local,
                    new_stack_clause,
                )
            }
            &Instruction_::NewFrameCursor(stack) => {
                format!("COMMINST @uvm.meta.new_cursor({})", ops[stack])
            }
            &Instruction_::GetIRef(reference) => {
                format!(
                    "GETIREF<{}> {}",
                    ops[reference].ty().get_referent_ty().unwrap(),
                    ops[reference]
                )
            }
            &Instruction_::GetFieldIRef {
                is_ptr,
                base,
                index
            } => {
                let ptr = select_value!(is_ptr, " PTR", "");
                format!(
                    "GETFIELDIREF{}<{} {}> {}",
                    ptr,
                    ops[base].ty().get_referent_ty().unwrap(),
                    index,
                    ops[base]
                )
            }
            &Instruction_::GetElementIRef {
                is_ptr,
                base,
                index
            } => {
                let ptr = select_value!(is_ptr, " PTR", "");
                format!(
                    "GETELEMIREF{}<{} {}>{} {}",
                    ptr,
                    ops[base].ty().get_referent_ty().unwrap(),
                    ops[index].ty(),
                    ops[base],
                    ops[index]
                )
            }
            &Instruction_::ShiftIRef {
                is_ptr,
                base,
                offset
            } => {
                let ptr = select_value!(is_ptr, " PTR", "");
                format!(
                    "SHIFTIREF{}<{} {}>{} {}",
                    ptr,
                    ops[base].ty().get_referent_ty().unwrap(),
                    ops[offset].ty(),
                    ops[base],
                    ops[offset]
                )
            }
            &Instruction_::GetVarPartIRef { is_ptr, base } => {
                let ptr = select_value!(is_ptr, " PTR", "");
                format!(
                    "GETVARPARTIREF{}<{}> {}",
                    ptr,
                    ops[base].ty().get_referent_ty().unwrap(),
                    ops[base]
                )
            }

            &Instruction_::Fence(order) => format!("FENCE {}", order),

            &Instruction_::Return(ref vals) => {
                if vals.len() == 0 {
                    format!("RET")
                } else if vals.len() == 1 {
                    format!("RET {}", ops[vals[0]])
                } else {
                    format!("RET ({})", op_vector_str(vals, ops))
                }
            }
            &Instruction_::ThreadExit => "COMMINST @uvm.thread_exit".to_string(),
            &Instruction_::CurrentStack => "COMMINST @uvm.current_stack".to_string(),
            &Instruction_::KillStack(s) => format!("COMMINST @uvm.kill_stack({})", ops[s]),
            &Instruction_::Throw(exn_obj) => format!("THROW {}", ops[exn_obj]),
            &Instruction_::TailCall(ref call) => format!("TAILCALL{}", call.debug_str(ops)),
            &Instruction_::Branch1(ref dest) => format!("BRANCH {}", dest.debug_str(ops)),
            &Instruction_::Branch2 {
                cond,
                ref true_dest,
                ref false_dest,
                ..
            } => {
                format!(
                    "BRANCH2 {} {} {}",
                    ops[cond],
                    true_dest.debug_str(ops),
                    false_dest.debug_str(ops)
                )
            }
            &Instruction_::Select {
                cond,
                true_val,
                false_val
            } => {
                format!(
                    "SELECT<{} {}> {} {} {}",
                    ops[cond].ty(),
                    ops[true_val].ty(),
                    ops[cond],
                    ops[true_val],
                    ops[false_val]
                )
            }
            &Instruction_::Watchpoint {
                id,
                ref disable_dest,
                ref resume
            } => {
                match id {
                    Some(id) => {
                        format!(
                            // TODO: WPEXC should be optional
                            "WATCHPOINT {}<{}> {} {} WPEXC({})",
                            id,
                            format_value_types(&self.value),
                            disable_dest.as_ref().unwrap().debug_str(ops),
                            resume.normal_dest.debug_str(ops),
                            resume.exn_dest.debug_str(ops)
                        )
                    }
                    //TRAP < Ts > excClause keepAliveClause
                    None => {
                        format!(
                            "TRAP<{}> {}",
                            format_value_types(&self.value),
                            resume.debug_str(ops)
                        )
                    }
                }
            }
            &Instruction_::WPBranch {
                wp,
                ref disable_dest,
                ref enable_dest
            } => {
                format!(
                    "WPBRANCH {} {} {}",
                    wp,
                    disable_dest.debug_str(ops),
                    enable_dest.debug_str(ops)
                )
            }
            &Instruction_::Call {
                ref data,
                ref resume
            } => format!("CALL{} {}", data.debug_str(ops), resume.debug_str(ops)),
            &Instruction_::CCall {
                ref data,
                ref resume
            } => format!("CCALL{} {}", data.debug_str(ops), resume.debug_str(ops)),
            &Instruction_::SwapStackExpr {
                stack,
                is_exception,
                ref args
            } => {
                format!(
                    "SWAPSTACK {} RET_WITH<{}> {}",
                    ops[stack],
                    format_value_types(&self.value),
                    format_new_stack_clause(is_exception, args, ops)
                )
            }
            &Instruction_::SwapStackExc {
                stack,
                is_exception,
                ref args,
                ref resume
            } => {
                format!(
                    "SWAPSTACK {} RET_WITH<{}> {} {}",
                    ops[stack],
                    format_value_types(&self.value),
                    format_new_stack_clause(is_exception, args, ops),
                    resume.debug_str(ops)
                )
            }

            &Instruction_::SwapStackKill {
                stack,
                is_exception,
                ref args
            } => {
                format!(
                    "SWAPSTACK {} KILL_OLD {}",
                    ops[stack],
                    format_new_stack_clause(is_exception, args, ops)
                )
            }

            &Instruction_::Switch {
                cond,
                ref default,
                ref branches
            } => {
                //SWITCH < T > opnd default { ( value dest ) rep }
                let mut ret = format!(
                    "SWITCH<{}> {} {} {{",
                    ops[cond].ty(),
                    ops[cond],
                    default.debug_str(ops)
                );
                for i in 0..branches.len() {
                    let (op, ref dest) = branches[i];
                    ret.push_str(format!("{} {}", ops[op], dest.debug_str(ops)).as_str());
                    if i != branches.len() - 1 {
                        ret.push_str(" ");
                    }
                }
                ret.push_str("}");

                ret
            }
            &Instruction_::ExnInstruction {
                ref inner,
                ref resume
            } => format!("{} {}", inner.debug_str(ops), resume.debug_str(ops)),

            // common inst
            &Instruction_::CommonInst_GetThreadLocal => format!("COMMINST @uvm.get_threadlocal"),
            &Instruction_::CommonInst_SetThreadLocal(op) => {
                format!("COMMINST @uvm.set_threadlocal({})", ops[op])
            }

            &Instruction_::CommonInst_Pin(op) => {
                format!("COMMINST @uvm.native.pin<{}>({})", ops[op].ty(), ops[op])
            }
            &Instruction_::CommonInst_Unpin(op) => {
                format!("COMMINST @uvm.native.unpin<{}>({})", ops[op].ty(), ops[op])
            }
            &Instruction_::CommonInst_GetAddr(op) => {
                format!(
                    "COMMINST @uvm.native.get_addr<{}>({})",
                    ops[op].ty(),
                    ops[op]
                )
            }
            // Tagerf64
            &Instruction_::CommonInst_Tr64IsFp(op) => {
                format!("COMMINST @uvm.tr64.is_fp({})", ops[op])
            }
            &Instruction_::CommonInst_Tr64IsInt(op) => {
                format!("COMMINST @uvm.tr64.is_int({})", ops[op])
            }
            &Instruction_::CommonInst_Tr64IsRef(op) => {
                format!("COMMINST @uvm.tr64.is_ref({})", ops[op])
            }
            &Instruction_::CommonInst_Tr64FromFp(op) => {
                format!("COMMINST @uvm.tr64.from_fp({})", ops[op])
            }
            &Instruction_::CommonInst_Tr64FromInt(op) => {
                format!("COMMINST @uvm.tr64.from_int({})", ops[op])
            }
            &Instruction_::CommonInst_Tr64FromRef(op1, op2) => {
                format!("COMMINST @uvm.tr64.from_ref({} {})", ops[op1], ops[op2])
            }
            &Instruction_::CommonInst_Tr64ToFp(op) => {
                format!("COMMINST @uvm.tr64.to_fp({})", ops[op])
            }
            &Instruction_::CommonInst_Tr64ToInt(op) => {
                format!("COMMINST @uvm.tr64.to_int({})", ops[op])
            }
            &Instruction_::CommonInst_Tr64ToRef(op) => {
                format!("COMMINST @uvm.tr64.to_ref({})", ops[op])
            }
            &Instruction_::CommonInst_Tr64ToTag(op) => {
                format!("COMMINST @uvm.tr64.to_tag({})", ops[op])
            }

            // move
            &Instruction_::Move(from) => format!("MOVE<{}> {}", ops[from].ty(), ops[from]),
            // print hex
            &Instruction_::PrintHex(i) => format!("PRINTHEX<{}> {}", ops[i].ty(), ops[i]),
            // set retval
            &Instruction_::SetRetval(val) => format!("SETRETVAL {}", ops[val]),
            // get vm thread local
            &Instruction_::GetVMThreadLocal => format!("GETVMTHREADLOCAL")
        }
    }
}

const PRINT_INST_NAME: bool = false;

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ref ops = self.ops;
        let value = match &self.value {
            &Some(ref v) if v.len() == 0 => format!(""),
            &Some(ref v) if v.len() == 1 => format!("{} = ", v[0]),
            &Some(ref v) => format!("({}) = ", vec_utils::as_str_sp(&v)),
            &None => format!("")
        };

        if PRINT_INST_NAME {
            write!(f, "{}[{}]{}", value, self.hdr, self.debug_str(ops))
        } else {
            write!(f, "{}{}", value, self.debug_str(ops))
        }
    }
}

/// Instruction_ is used for pattern matching for Instruction
#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum Instruction_ {
    // non-terminal instruction
    /// binary operations
    BinOp(BinOp, OpIndex, OpIndex),
    /// binary operations with status flag (overflow, sign, etc. )
    BinOpWithStatus(BinOp, BinOpStatus, OpIndex, OpIndex),

    /// comparison operations
    CmpOp(CmpOp, OpIndex, OpIndex),

    /// conversion operations (casting)
    ConvOp {
        operation: ConvOp,
        from_ty: P<MuType>,
        to_ty: P<MuType>,
        operand: OpIndex
    },

    /// a non-terminating Call instruction (the call does not have an exceptional branch)
    /// This instruction is not in the Mu spec, but is documented in the HOL formal spec
    ExprCall {
        data: CallData,
        is_abort: bool // T to abort, F to rethrow
    },

    /// a non-terminating CCall instruction (the call does not have an exceptional branch)
    /// This instruction is not in the Mu spec, but is documented in the HOL formal spec
    ExprCCall { data: CallData, is_abort: bool },

    /// load instruction
    Load {
        is_ptr: bool,
        order: MemoryOrder,
        mem_loc: OpIndex
    },

    /// store instruction
    Store {
        is_ptr: bool,
        order: MemoryOrder,
        mem_loc: OpIndex,
        value: OpIndex
    },

    /// compare and exchange, yields a pair value (oldvalue, boolean (T = success, F = failure))
    CmpXchg {
        is_ptr: bool,
        is_weak: bool,
        success_order: MemoryOrder,
        fail_order: MemoryOrder,
        mem_loc: OpIndex,
        expected_value: OpIndex,
        desired_value: OpIndex
    },

    /// atomic read-modify-write, yields old memory value
    AtomicRMW {
        is_ptr: bool, // T for iref, F for ptr
        order: MemoryOrder,
        op: AtomicRMWOp,
        mem_loc: OpIndex,
        value: OpIndex // operand for op
    },

    /// allocate an object (non hybrid type) in the heap, yields a reference of the type
    New(P<MuType>),

    /// allocate an object (non hybrid type) on the stack, yields an iref of the type
    AllocA(P<MuType>),

    /// allocate a hybrid type object in the heap, yields ref
    /// args: the type of the hybrid, hybrid part length
    NewHybrid(P<MuType>, OpIndex),

    /// allocate a hybrid type object on the stack, yields iref
    /// args: the type of the hybrid, hybrid part length
    AllocAHybrid(P<MuType>, OpIndex),

    /// create a new Mu stack, yields stack ref
    /// args: functionref of the entry function
    NewStack(OpIndex),

    /// kill the given Mu stack
    KillStack(OpIndex),

    /// return stackref for the current stack
    CurrentStack,

    /// create a new Mu thread, yields thread reference
    /// args: stackref of a Mu stack, a list of arguments
    NewThread {
        stack: OpIndex,
        thread_local: Option<OpIndex>,
        is_exception: bool,
        args: Vec<OpIndex>
    },

    /// create a frame cursor reference
    /// args: stackref of a Mu stack
    NewFrameCursor(OpIndex), // stack

    /// get internal reference of a reference
    /// args: a reference
    GetIRef(OpIndex),

    /// get internal reference of an iref (or uptr) to a struct/hybrid fix part
    GetFieldIRef {
        is_ptr: bool,
        base: OpIndex, // iref or uptr
        index: usize   // constant
    },

    /// get internal reference of an element of an iref (or uptr) to an array
    GetElementIRef {
        is_ptr: bool,
        base: OpIndex,
        index: OpIndex // can be constant or ssa var
    },

    /// offset an iref (or uptr) (offset is an index)
    ShiftIRef {
        is_ptr: bool,
        base: OpIndex,
        offset: OpIndex
    },

    /// get internal reference to an element in hybrid var part
    GetVarPartIRef { is_ptr: bool, base: OpIndex },

    /// a fence of certain memory order
    Fence(MemoryOrder),

    // terminal instruction
    /// return instruction
    /// args: a list of return values
    Return(Vec<OpIndex>),

    /// thread exit
    ThreadExit,

    /// throw an exception
    Throw(OpIndex),

    /// tail call a function (reuse current frame)
    TailCall(CallData),

    /// unconditional branch
    Branch1(Destination),

    /// conditional branch
    Branch2 {
        cond: OpIndex,
        true_dest: Destination,
        false_dest: Destination,
        true_prob: f32
    },

    /// returns value1 if condition is true, otherwise returns value2
    Select {
        cond: OpIndex,
        true_val: OpIndex,
        false_val: OpIndex
    },

    /// a watchpoint
    /// * Watchpoint NONE ResumptionData: serves as an unconditional trap.
    ///   Trap to client, and resume with ResumptionData
    /// * Watchpoint (WPID dest) ResumptionData:
    ///   * when disabled, jump to dest
    ///   * when enabled, trap to client and resume
    Watchpoint {
        id: Option<WPID>,
        disable_dest: Option<Destination>,
        resume: ResumptionData
    },

    /// a watchpoint branch, branch to different destinations based on enabled/disabled
    WPBranch {
        wp: WPID,
        disable_dest: Destination,
        enable_dest: Destination
    },

    /// a call instruction that may throw an exception
    Call {
        data: CallData,
        resume: ResumptionData
    },

    /// a ccall instruction that may throw an exception
    CCall {
        data: CallData,
        resume: ResumptionData
    },

    /// A swap stack with an exception clause (i.e. uses the RET_WITH form)
    SwapStackExc {
        stack: OpIndex,
        is_exception: bool,
        args: Vec<OpIndex>,
        resume: ResumptionData
    },

    /// A swap stack without an exception clause that is not a terminator
    /// (i.e. uses the RET_WITH form)
    SwapStackExpr {
        stack: OpIndex,
        is_exception: bool,
        args: Vec<OpIndex>
    },

    /// A swapstack without an exception clause that is a terminator (i.e. one with KILL_OLD)
    SwapStackKill {
        stack: OpIndex,
        is_exception: bool,
        args: Vec<OpIndex>
    },

    /// a multiway branch
    Switch {
        cond: OpIndex,
        default: Destination,
        branches: Vec<(OpIndex, Destination)>
    },

    /// a wrapper for any instruction that may throw an exception
    //  This is not used at the moment
    ExnInstruction {
        inner: Box<Instruction>,
        resume: ResumptionData
    },

    /// common inst: get thread local
    CommonInst_GetThreadLocal,
    /// common inst: set thread local
    CommonInst_SetThreadLocal(OpIndex),

    /// common inst: pin an object (prevent it being moved and reclaimed by GC), yields a uptr
    CommonInst_Pin(OpIndex),
    /// common inst: unpin an object (the object is automatically managed by GC)
    CommonInst_Unpin(OpIndex),
    /// common inst: get address of a global cell or a pinned object
    CommonInst_GetAddr(OpIndex),

    /// common inst: is the tagref a floating point?
    CommonInst_Tr64IsFp(OpIndex),
    /// common inst: is the tagref an int?
    CommonInst_Tr64IsInt(OpIndex),
    /// common inst: is the tagref a ref?
    CommonInst_Tr64IsRef(OpIndex),
    /// common inst: creates a tagref from floating point (double)
    CommonInst_Tr64FromFp(OpIndex),
    /// common inst: creates a tagref from int<52>
    CommonInst_Tr64FromInt(OpIndex),
    /// common inst: creates a tagref from reference (a ref-void typed ref, 6 bits tag)
    CommonInst_Tr64FromRef(OpIndex, OpIndex),
    /// common inst: converts a tagref to floating point (double)
    CommonInst_Tr64ToFp(OpIndex),
    /// common inst: converts a tagref to integer (int<52>)
    CommonInst_Tr64ToInt(OpIndex),
    /// common inst: converts a tagref to reference
    CommonInst_Tr64ToRef(OpIndex),
    /// common inst: converts a tagref to a tag (int<64>)
    CommonInst_Tr64ToTag(OpIndex),

    /// internal use: move from value to value
    Move(OpIndex),
    /// internal use: print op as hex value
    PrintHex(OpIndex),
    /// internal use: set return value for main
    SetRetval(OpIndex),
    /// internal use: get zebu thread local
    GetVMThreadLocal
}

fn format_value_types(value: &Option<Vec<P<Value>>>) -> String {
    match value {
        &Some(ref v) => {
            let mut tys = format!("");
            for i in 0..v.len() {
                tys.push_str(format!("{}", v[i].ty).as_str());
                if i != v.len() - 1 {
                    tys.push_str(" ");
                }
            }
            tys
        }
        &None => "".to_string()
    }
}

fn format_new_stack_clause(
    is_exception: bool,
    args: &Vec<OpIndex>,
    ops: &Vec<P<TreeNode>>
) -> String {
    if is_exception {
        assert!(args.len() == 1);
        format!("THROW_EXC {}", ops[args[0]])
    } else {
        let mut arg_tys = format!("");
        let mut arg_vals = format!("");
        for i in 0..args.len() {
            let ref op = ops[args[i]];
            arg_tys.push_str(format!("{}", op.ty()).as_str());
            arg_vals.push_str(format!("{}", op).as_str());
            if i != args.len() - 1 {
                arg_tys.push_str(" ");
                arg_vals.push_str(" ");
            }
        }
        format!("PASS_VALUES<{}>({})", arg_tys, arg_vals)
    }
}
/// BinOpStatus represents status flags from a binary operation
#[derive(Copy, Clone)]
pub struct BinOpStatus {
    /// negative flag
    pub flag_n: bool,
    /// zero flag
    pub flag_z: bool,
    /// carry flag
    pub flag_c: bool,
    /// overflow flag
    pub flag_v: bool
}
impl fmt::Display for BinOpStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        if self.flag_n {
            write!(f, "#N ")?;
        }
        if self.flag_z {
            write!(f, "#Z ")?;
        }
        if self.flag_c {
            write!(f, "#C ")?;
        }
        if self.flag_v {
            write!(f, "#V ")?;
        }
        Ok(())
    }
}


impl BinOpStatus {
    pub fn none() -> BinOpStatus {
        BinOpStatus {
            flag_n: false,
            flag_z: false,
            flag_c: false,
            flag_v: false
        }
    }

    pub fn n() -> BinOpStatus {
        BinOpStatus {
            flag_n: true,
            flag_z: false,
            flag_c: false,
            flag_v: false
        }
    }

    pub fn z() -> BinOpStatus {
        BinOpStatus {
            flag_n: false,
            flag_z: true,
            flag_c: false,
            flag_v: false
        }
    }

    pub fn c() -> BinOpStatus {
        BinOpStatus {
            flag_n: false,
            flag_z: false,
            flag_c: true,
            flag_v: false
        }
    }

    pub fn v() -> BinOpStatus {
        BinOpStatus {
            flag_n: false,
            flag_z: false,
            flag_c: false,
            flag_v: true
        }
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

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum MemoryOrder {
    NotAtomic,
    Relaxed,
    Consume,
    Acquire,
    Release,
    AcqRel,
    SeqCst
}

impl fmt::Display for MemoryOrder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::MemoryOrder::*;
        write!(
            f,
            "{}",
            match *self {
                NotAtomic => "",
                Relaxed => "RELAXED",
                Consume => "CONSUME",
                Acquire => "ACQUIRE",
                Release => "RELEASE",
                AcqRel => "ACQ_REL",
                SeqCst => "SEQ_CST"
            }
        )
    }
}

pub const C_CALL_CONVENTION: CallConvention = CallConvention::Foreign(ForeignFFI::C);
pub const MU_CALL_CONVENTION: CallConvention = CallConvention::Mu;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum CallConvention {
    Mu,
    Foreign(ForeignFFI)
}
impl fmt::Display for CallConvention {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if *self == C_CALL_CONVENTION {
            write!(f, "#DEFAULT")
        } else if *self == MU_CALL_CONVENTION {
            write!(f, "#MU")
        } else {
            unimplemented!()
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
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
        let func = &ops[self.func];
        let conv = if self.convention == CallConvention::Mu {
            "".to_string()
        } else {
            format!(" {}", self.convention)
        };
        format!(
            "{}<{}> {} ({})",
            conv,
            func.ty().get_sig().unwrap(),
            func,
            op_vector_str(&self.args, ops)
        )
    }
}

#[derive(Clone, Debug)]
pub struct ResumptionData {
    pub normal_dest: Destination,
    pub exn_dest: Destination
}

impl ResumptionData {
    fn debug_str(&self, ops: &Vec<P<TreeNode>>) -> String {
        format!(
            "EXC ({} {})",
            self.normal_dest.debug_str(ops),
            self.exn_dest.debug_str(ops)
        )
    }
}

#[derive(Clone, Debug)]
pub struct Destination {
    pub target: MuEntityRef,
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
                ret.push_str(" ");
            }
        }
        ret.push(')');

        ret
    }

    pub fn get_arguments_as_node(&self, ops: &Vec<P<TreeNode>>) -> Vec<P<TreeNode>> {
        vec_utils::map(&self.args, |x| match x {
            &DestArg::Normal(i) => ops[i].clone(),
            &DestArg::Freshbound(_) => unimplemented!()
        })
    }

    pub fn get_arguments(&self, ops: &Vec<P<TreeNode>>) -> Vec<P<Value>> {
        vec_utils::map(&self.args, |x| match x {
            &DestArg::Normal(i) => ops[i].clone_value(),
            &DestArg::Freshbound(_) => unimplemented!()
        })
    }
}

#[derive(Clone, Debug)]
pub enum DestArg {
    /// a normal destination argument is an SSA value (appears in the ops field of the instruction)
    Normal(OpIndex),
    /// a freshbound argument is an undeclared/anonymous value (currently not support this)
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
            ret.push_str(" ");
        }
    }
    ret
}
