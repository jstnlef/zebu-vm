use inst::*;
use inst::Instruction_::*;

pub fn is_terminal_inst(inst: &Instruction_) -> bool {
    match inst {
        &BinOp(_, _, _)
        | &BinOpWithStatus(_, _, _, _)
        | &CmpOp(_, _, _)
        | &ConvOp{..}
        | &ExprCall{..}
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
        | &GetIRef(_)
        | &GetFieldIRef{..}
        | &GetElementIRef{..}
        | &ShiftIRef{..}
        | &GetVarPartIRef{..}
        | &Select{..}
        | &Fence(_)
        | &CommonInst_GetThreadLocal
        | &CommonInst_SetThreadLocal(_)
        | &CommonInst_Pin(_)
        | &CommonInst_Unpin(_)
        | &Move(_)
        | &PrintHex(_) => false,
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
        | &ExnInstruction{..} => true
    }
}

pub fn is_non_terminal_inst(inst: &Instruction_) -> bool {
    !is_terminal_inst(inst)
}

// FIXME: check the correctness
pub fn has_side_effect(inst: &Instruction_) -> bool {
    match inst {
        &BinOp(_, _, _) => false,
        &BinOpWithStatus(_, _, _, _) => false,
        &CmpOp(_, _, _) => false,
        &ConvOp{..} => false,
        &ExprCall{..} => true,
        &ExprCCall{..} => true,
        &Load{..} => true,
        &Store{..} => true,
        &CmpXchg{..} => true,
        &AtomicRMW{..} => true,
        &New(_) => true,
        &AllocA(_) => true,
        &NewHybrid(_, _) => true,
        &AllocAHybrid(_, _) => true,
        &NewStack(_) => true,
        &NewThread(_, _) => true,
        &NewThreadExn(_, _) => true,
        &NewFrameCursor(_) => true,
        &GetIRef(_) => false,
        &GetFieldIRef{..} => false,
        &GetElementIRef{..} => false,
        &ShiftIRef{..} => false,
        &GetVarPartIRef{..} => false,
        &Fence(_) => true,
        &Return(_) => true,
        &ThreadExit => true,
        &Throw(_) => true,
        &TailCall(_) => true,
        &Branch1(_) => true,
        &Branch2{..} => true,
        &Select{..} => false,
        &Watchpoint{..} => true,
        &WPBranch{..} => true,
        &Call{..} => true,
        &CCall{..} => true,
        &SwapStack{..} => true,
        &Switch{..} => true,
        &ExnInstruction{..} => true,
        &CommonInst_GetThreadLocal => true,
        &CommonInst_SetThreadLocal(_) => true,
        &CommonInst_Pin(_) => true,
        &CommonInst_Unpin(_) => true,
        &Move(_) => false,
        &PrintHex(_) => true
    }
}

pub fn is_potentially_excepting_instruction(inst: &Instruction_) -> bool {
    match inst {
        &Watchpoint{..}
        | &Call{..}
        | &CCall{..}
        | &SwapStack{..}
        | &ExnInstruction{..} => true,

        &BinOp(_, _, _)
        | &BinOpWithStatus(_, _, _, _)
        | &CmpOp(_, _, _)
        | &ConvOp{..}
        | &ExprCall{..}
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
        | &GetIRef(_)
        | &GetFieldIRef{..}
        | &GetElementIRef{..}
        | &ShiftIRef{..}
        | &GetVarPartIRef{..}
        | &Fence(_)
        | &Return(_)
        | &ThreadExit
        | &Throw(_)
        | &TailCall(_)
        | &Branch1(_)
        | &Branch2{..}
        | &Select{..}
        | &WPBranch{..}
        | &Switch{..}
        | &CommonInst_GetThreadLocal
        | &CommonInst_SetThreadLocal(_)
        | &CommonInst_Pin(_)
        | &CommonInst_Unpin(_)
        | &Move(_)
        | &PrintHex(_) => false
    }
}