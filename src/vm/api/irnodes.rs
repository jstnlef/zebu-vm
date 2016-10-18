#![allow(non_snake_case)]
#![allow(dead_code)]

use super::deps::*;

//pub type MuID = usize;
pub type MuTypeNode = MuID;
pub type MuFuncSigNode = MuID;
pub type MuVarNode = MuID;
pub type MuGlobalVarNode = MuID;
pub type MuLocalVarNode = MuID;
pub type MuConstNode = MuID;
pub type MuConstIntNode = MuID;
pub type MuFuncNode = MuID;
pub type MuFuncVerNode = MuID;
pub type MuBBNode = MuID;
pub type MuInstNode = MuID;
pub type MuDestClause = MuID;
pub type MuExcClause = MuID;
pub type MuKeepaliveClause = MuID;
pub type MuCurStackClause = MuID;
pub type MuNewStackClause = MuID;
pub type MuWPID = MuID;

pub type Flag = u32;
pub type MuBinOptr = Flag;
pub type MuBinOpStatus = Flag;
pub type MuCmpOptr = Flag;
pub type MuConvOptr = Flag;
pub type MuMemoryOrder = Flag;
pub type MuAtomicRMWOptr = Flag;
pub type MuCommInst = Flag;

#[derive(Debug)]
pub enum NodeType {
    TypeInt         { id: MuID, len: i32 },
    TypeFloat       { id: MuID },
    TypeDouble      { id: MuID },
    TypeUPtr        { id: MuID, ty: MuTypeNode },
    TypeUFuncPtr    { id: MuID, sig: MuFuncSigNode },

    TypeStruct { id: MuID, fieldtys: Vec<MuTypeNode> },
    TypeHybrid { id: MuID, fixedtys: Vec<MuTypeNode>, varty: MuTypeNode },
    TypeArray  { id: MuID, elemty: MuTypeNode, len: usize },
    TypeVector { id: MuID, elemty: MuTypeNode, lem: usize },

    TypeRef             { id: MuID, ty: MuTypeNode },
    TypeIRef            { id: MuID, ty: MuTypeNode },
    TypeWeakRef         { id: MuID, ty: MuTypeNode },
    TypeFuncRef         { id: MuID, sig: MuFuncSigNode },
    TypeThreadRef       { id: MuID },
    TypeStackRef        { id: MuID },
    TypeFrameCursorRef  { id: MuID },
    TypeIRBuilderRef    { id: MuID },
}

#[derive(Debug)]
pub struct NodeFuncSig { pub id: MuID, pub paramtys: Vec<MuTypeNode>, pub rettys: Vec<MuTypeNode> }

#[derive(Debug)]
pub enum NodeConst {
    ConstInt    { id: MuID, ty: MuTypeNode, value:  u64 },
    ConstFloat  { id: MuID, ty: MuTypeNode, value:  f32 },
    ConstDouble { id: MuID, ty: MuTypeNode, value:  f64 },
    ConstNull   { id: MuID, ty: MuTypeNode },
    ConstSeq    { id: MuID, ty: MuTypeNode, elems: Vec<MuGlobalVarNode> },
    ConstExtern { id: MuID, ty: MuTypeNode, symbol: String },
}

#[derive(Debug)]
pub struct NodeGlobalCell { pub id: MuID, pub ty: MuTypeNode }

#[derive(Debug)]
pub struct NodeFunc { pub id: MuID, pub sig: MuFuncSigNode }

#[derive(Debug)]
pub struct NodeExpFunc { pub id: MuID, pub func: MuFuncNode, pub callconv: usize, pub cookie: MuConstIntNode }

#[derive(Debug)]
pub struct NodeFuncVer { pub id: MuID, pub func: MuFuncNode, pub bbs: Vec<MuBBNode> }

#[derive(Debug)]
pub struct NodeBB { pub id: MuID, pub norParamIDs: Vec<MuID>, pub norParamTys: Vec<MuTypeNode>, pub excParamID: Option<MuID>, pub insts: Vec<MuInstNode> }
#[derive(Debug)]
pub struct NodeDestClause { pub id: MuID, pub dest: MuBBNode, pub vars: Vec<MuVarNode> }

#[derive(Debug)]
pub struct NodeExcClause { pub id: MuID, pub nor: MuDestClause, pub exc: MuDestClause }

#[derive(Debug)]
pub struct NodeKeepaliveClause { pub id: MuID, pub vars: Vec<MuLocalVarNode> }

#[derive(Debug)]
pub struct NodeCscRetWith { pub id: MuID, pub rettys: Vec<MuVarNode> }
#[derive(Debug)]
pub struct NodeCscKillOld { pub id: MuID }

#[derive(Debug)]
pub struct NodeNscPassValues { pub id: MuID, pub tys: Vec<MuTypeNode>, pub vars: Vec<MuVarNode> }
#[derive(Debug)]
pub struct NodeNscThrowExc { pub id: MuID, pub exc: MuVarNode }

#[derive(Debug)]
pub enum NodeInst {
    NodeBinOp { id: MuID, resultID: MuID, statusResultIDs: Vec<MuID>, optr: MuBinOptr, flags: MuBinOpStatus, ty: MuTypeNode, opnd1: MuVarNode, opnd2: MuVarNode, excClause: Option<MuExcClause> },
    NodeCmp { id: MuID, resultID: MuID, optr: MuCmpOptr, ty: MuTypeNode, opnd1: MuVarNode, opnd2: MuVarNode },
    NodeConv { id: MuID, resultID: MuID, optr: MuConvOptr, fromTy: MuTypeNode, toTy: MuTypeNode, opnd: MuVarNode },
    NodeSelect { id: MuID, resultID: MuID, condTy: MuTypeNode, opndTy: MuTypeNode, cond: MuVarNode, ifTrue: MuVarNode, ifFalse: MuVarNode },
    NodeBranch { id: MuID, dest: MuDestClause },
    NodeBranch2 { id: MuID, cond: MuVarNode, ifTrue: MuDestClause, ifFalse: MuDestClause },
    NodeSwitch { id: MuID, opndTy: MuTypeNode, opnd: MuVarNode, defaultDest: MuDestClause, cases: Vec<MuConstNode>, dests: Vec<MuDestClause> },
    NodeCall { id: MuID, resultIDs: Vec<MuID>, sig: MuFuncSigNode, callee: MuVarNode, args: Vec<MuVarNode>, excClause: Option<MuExcClause>, keepaliveClause: Option<MuKeepaliveClause> },
    NodeTailCall { id: MuID, sig: MuFuncSigNode, callee: MuVarNode, args: Vec<MuVarNode> },
    NodeRet { id: MuID, rvs: Vec<MuVarNode> },
    NodeThrow { id: MuID, exc: MuVarNode },
    NodeExtractValue { id: MuID, resultID: MuID, strty: MuTypeNode, index: i32, opnd: MuVarNode },
    NodeInsertValue { id: MuID, resultID: MuID, strty: MuTypeNode, index: i32, opnd: MuVarNode, newval: MuVarNode },
    NodeExtractElement { id: MuID, resultID: MuID, seqty: MuTypeNode, indty: MuTypeNode, opnd: MuVarNode, index: MuVarNode },
    NodeInsertElement { id: MuID, resultID: MuID, seqty: MuTypeNode, indty: MuTypeNode, opnd: MuVarNode, index: MuVarNode, newval: MuVarNode },
    NodeShuffleVector { id: MuID, resultID: MuID, vecty: MuTypeNode, maskty: MuTypeNode, vec1: MuVarNode, vec2: MuVarNode, mask: MuVarNode },
    NodeNew { id: MuID, resultID: MuID, allocty: MuTypeNode, excClause: Option<MuExcClause> },
    NodeNewHybrid { id: MuID, resultID: MuID, allocty: MuTypeNode, lenty: MuTypeNode, length: MuVarNode, excClause: Option<MuExcClause> },
    NodeAlloca { id: MuID, resultID: MuID, allocty: MuTypeNode, excClause: Option<MuExcClause> },
    NodeAllocaHybrid { id: MuID, resultID: MuID, allocty: MuTypeNode, lenty: MuTypeNode, length: MuVarNode, excClause: Option<MuExcClause> },
    NodeGetIRef { id: MuID, resultID: MuID, refty: MuTypeNode, opnd: MuVarNode },
    NodeGetFieldIRef { id: MuID, resultID: MuID, isPtr: bool, refty: MuTypeNode, index: i32, opnd: MuVarNode },
    NodeGetElemIRef { id: MuID, resultID: MuID, isPtr: bool, refty: MuTypeNode, indty: MuTypeNode, opnd: MuVarNode, index: MuVarNode },
    NodeShiftIRef { id: MuID, resultID: MuID, isPtr: bool, refty: MuTypeNode, offty: MuTypeNode, opnd: MuVarNode, offset: MuVarNode },
    NodeGetVarPartIRef { id: MuID, resultID: MuID, isPtr: bool, refty: MuTypeNode, opnd: MuVarNode },
    NodeLoad { id: MuID, resultID: MuID, isPtr: bool, ord: MuMemoryOrder, refty: MuTypeNode, loc: MuVarNode, excClause: Option<MuExcClause> },
    NodeStore { id: MuID, isPtr: bool, ord: MuMemoryOrder, refty: MuTypeNode, loc: MuVarNode, newval: MuVarNode, excClause: Option<MuExcClause> },
    NodeCmpXchg { id: MuID, valueResultID: MuID, succResultID: MuID, isPtr: bool, isWeak: bool, ordSucc: MuMemoryOrder, ordFail: MuMemoryOrder, refty: MuTypeNode, loc: MuVarNode, expected: MuVarNode, desired: MuVarNode, excClause: Option<MuExcClause> },
    NodeAtomicRMW { id: MuID, resultID: MuID, isPtr: bool, ord: MuMemoryOrder, optr: MuAtomicRMWOptr, refTy: MuTypeNode, loc: MuVarNode, opnd: MuVarNode, excClause: Option<MuExcClause> },
    NodeFence { id: MuID, ord: MuMemoryOrder, },
    NodeTrap { id: MuID, resultIDs: Vec<MuID>, rettys: Vec<MuTypeNode>, excClause: Option<MuExcClause>, keepaliveClause: Option<MuKeepaliveClause> },
    NodeWatchPoint { id: MuID, wpid: MuWPID, resultIDs: Vec<MuID>, rettys: Vec<MuTypeNode>, dis: MuDestClause, ena: MuDestClause, exc: Option<MuDestClause>, keepaliveClause: Option<MuKeepaliveClause> },
    NodeWPBranch { id: MuID, wpid: MuWPID, dis: MuDestClause, ena: MuDestClause },
    NodeCCall { id: MuID, resultIDs: Vec<MuID>, callconv: Flag, calleeTy: MuTypeNode, sig: MuFuncSigNode, callee: MuVarNode, args: Vec<MuVarNode>, excClause: Option<MuExcClause>, keepaliveClause: Option<MuKeepaliveClause> },
    NodeNewThread { id: MuID, resultID: MuID, stack: MuVarNode, threadlocal: Option<MuVarNode>, newStackClause: MuNewStackClause, excClause: Option<MuExcClause> },
    NodeSwapStack { id: MuID, resultIDs: Vec<MuID>, swappee: MuVarNode, curStackClause: MuCurStackClause, newStackClause: MuNewStackClause, excClause: Option<MuExcClause>, keepaliveClause: Option<MuKeepaliveClause> },
    NodeCommInst { id: MuID, resultIDs: Vec<MuID>, opcode: MuCommInst, flags: Vec<Flag>, tys: Vec<MuTypeNode>, sigs: Vec<MuFuncSigNode>, args: Vec<MuVarNode>, excClause: Option<MuExcClause>, keepaliveClause: Option<MuKeepaliveClause> },
}
