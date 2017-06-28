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

use inst::*;
use inst::Instruction_::*;

pub fn is_terminal_inst(inst: &Instruction_) -> bool {
    match inst {
        &Return(_)
        | &ThreadExit
        | &Throw(_)
        | &TailCall(_)
        | &Branch1(_)
        | &Branch2{..}
        | &Watchpoint{..}
        | &WPBranch{..}
        | &Call{..}
        | &CCall{..}
        | &SwapStack{..}
        | &Switch{..}
        | &ExnInstruction{..} => true,
        _ => false,
    }
}

pub fn is_non_terminal_inst(inst: &Instruction_) -> bool {
    !is_terminal_inst(inst)
}

// FIXME: check the correctness
pub fn has_side_effect(inst: &Instruction_) -> bool {
    match inst {
          &ExprCall{..}
        | &ExprCCall{..}
        | &Load{..}
        | &Store{..}
        | &CmpXchg{..}
        | &AtomicRMW{..}
        | &New(_)
        | &AllocA(_)
        | &NewHybrid(_, _)
        | &AllocAHybrid(_, _)
        | &NewStack(_)
        | &NewThread(_, _)
        | &NewThreadExn(_, _)
        | &NewFrameCursor(_)
        | &Fence(_)
        | &Return(_)
        | &ThreadExit
        | &Throw(_)
        | &TailCall(_)
        | &Branch1(_)
        | &Branch2{..}
        | &Watchpoint{..}
        | &WPBranch{..}
        | &Call{..}
        | &CCall{..}
        | &SwapStack{..}
        | &Switch{..}
        | &ExnInstruction{..}
        | &CommonInst_GetThreadLocal
        | &CommonInst_SetThreadLocal(_)
        | &CommonInst_Pin(_)
        | &CommonInst_Unpin(_)
        | &PrintHex(_)
        | &SetRetval(_) => true,
        _ => false,
    }
}

pub fn is_potentially_excepting_instruction(inst: &Instruction_) -> bool {
    match inst {
        &Watchpoint{..}
        | &Call{..}
        | &CCall{..}
        | &SwapStack{..}
        | &ExnInstruction{..} => true,

        _ => false
    }
}