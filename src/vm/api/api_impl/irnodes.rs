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

#![allow(non_snake_case)]
#![allow(dead_code)]

use super::common::*;

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
    TypeVector { id: MuID, elemty: MuTypeNode, len: usize },

    TypeVoid            { id: MuID },
    TypeTagRef64        { id: MuID },

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
    ConstIntEx  { id: MuID, ty: MuTypeNode, value:  Vec<u64>},
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
pub struct NodeBB { pub id: MuID, pub nor_param_ids: Vec<MuID>, pub nor_param_types: Vec<MuTypeNode>, pub exc_param_id: Option<MuID>, pub insts: Vec<MuInstNode> }
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
    NodeBinOp { id: MuID, result_id: MuID, status_result_ids: Vec<MuID>, optr: MuBinOptr, flags: MuBinOpStatus, ty: MuTypeNode, opnd1: MuVarNode, opnd2: MuVarNode, exc_clause: Option<MuExcClause> },
    NodeCmp { id: MuID, result_id: MuID, optr: MuCmpOptr, ty: MuTypeNode, opnd1: MuVarNode, opnd2: MuVarNode },
    NodeConv { id: MuID, result_id: MuID, optr: MuConvOptr, from_ty: MuTypeNode, to_ty: MuTypeNode, opnd: MuVarNode },
    NodeSelect { id: MuID, result_id: MuID, cond_ty: MuTypeNode, opnd_ty: MuTypeNode, cond: MuVarNode, if_true: MuVarNode, if_false: MuVarNode },
    NodeBranch { id: MuID, dest: MuDestClause },
    NodeBranch2 { id: MuID, cond: MuVarNode, if_true: MuDestClause, if_false: MuDestClause },
    NodeSwitch { id: MuID, opnd_ty: MuTypeNode, opnd: MuVarNode, default_dest: MuDestClause, cases: Vec<MuConstNode>, dests: Vec<MuDestClause> },
    NodeCall { id: MuID, result_ids: Vec<MuID>, sig: MuFuncSigNode, callee: MuVarNode, args: Vec<MuVarNode>, exc_clause: Option<MuExcClause>, keepalive_clause: Option<MuKeepaliveClause> },
    NodeTailCall { id: MuID, sig: MuFuncSigNode, callee: MuVarNode, args: Vec<MuVarNode> },
    NodeRet { id: MuID, rvs: Vec<MuVarNode> },
    NodeThrow { id: MuID, exc: MuVarNode },
    NodeExtractValue { id: MuID, result_id: MuID, strty: MuTypeNode, index: i32, opnd: MuVarNode },
    NodeInsertValue { id: MuID, result_id: MuID, strty: MuTypeNode, index: i32, opnd: MuVarNode, newval: MuVarNode },
    NodeExtractElement { id: MuID, result_id: MuID, seqty: MuTypeNode, indty: MuTypeNode, opnd: MuVarNode, index: MuVarNode },
    NodeInsertElement { id: MuID, result_id: MuID, seqty: MuTypeNode, indty: MuTypeNode, opnd: MuVarNode, index: MuVarNode, newval: MuVarNode },
    NodeShuffleVector { id: MuID, result_id: MuID, vecty: MuTypeNode, maskty: MuTypeNode, vec1: MuVarNode, vec2: MuVarNode, mask: MuVarNode },
    NodeNew { id: MuID, result_id: MuID, allocty: MuTypeNode, exc_clause: Option<MuExcClause> },
    NodeNewHybrid { id: MuID, result_id: MuID, allocty: MuTypeNode, lenty: MuTypeNode, length: MuVarNode, exc_clause: Option<MuExcClause> },
    NodeAlloca { id: MuID, result_id: MuID, allocty: MuTypeNode, exc_clause: Option<MuExcClause> },
    NodeAllocaHybrid { id: MuID, result_id: MuID, allocty: MuTypeNode, lenty: MuTypeNode, length: MuVarNode, exc_clause: Option<MuExcClause> },
    NodeGetIRef { id: MuID, result_id: MuID, refty: MuTypeNode, opnd: MuVarNode },
    NodeGetFieldIRef { id: MuID, result_id: MuID, is_ptr: bool, refty: MuTypeNode, index: i32, opnd: MuVarNode },
    NodeGetElemIRef { id: MuID, result_id: MuID, is_ptr: bool, refty: MuTypeNode, indty: MuTypeNode, opnd: MuVarNode, index: MuVarNode },
    NodeShiftIRef { id: MuID, result_id: MuID, is_ptr: bool, refty: MuTypeNode, offty: MuTypeNode, opnd: MuVarNode, offset: MuVarNode },
    NodeGetVarPartIRef { id: MuID, result_id: MuID, is_ptr: bool, refty: MuTypeNode, opnd: MuVarNode },
    NodeLoad { id: MuID, result_id: MuID, is_ptr: bool, ord: MuMemoryOrder, refty: MuTypeNode, loc: MuVarNode, exc_clause: Option<MuExcClause> },
    NodeStore { id: MuID, is_ptr: bool, ord: MuMemoryOrder, refty: MuTypeNode, loc: MuVarNode, newval: MuVarNode, exc_clause: Option<MuExcClause> },
    NodeCmpXchg { id: MuID, value_result_id: MuID, succ_result_id: MuID, is_ptr: bool, is_weak: bool, ord_succ: MuMemoryOrder, ord_fail: MuMemoryOrder, refty: MuTypeNode, loc: MuVarNode, expected: MuVarNode, desired: MuVarNode, exc_clause: Option<MuExcClause> },
    NodeAtomicRMW { id: MuID, result_id: MuID, is_ptr: bool, ord: MuMemoryOrder, optr: MuAtomicRMWOptr, ref_ty: MuTypeNode, loc: MuVarNode, opnd: MuVarNode, exc_clause: Option<MuExcClause> },
    NodeFence { id: MuID, ord: MuMemoryOrder, },
    NodeTrap { id: MuID, result_ids: Vec<MuID>, rettys: Vec<MuTypeNode>, exc_clause: Option<MuExcClause>, keepalive_clause: Option<MuKeepaliveClause> },
    NodeWatchPoint { id: MuID, wpid: MuWPID, result_ids: Vec<MuID>, rettys: Vec<MuTypeNode>, dis: MuDestClause, ena: MuDestClause, exc: Option<MuDestClause>, keepalive_clause: Option<MuKeepaliveClause> },
    NodeWPBranch { id: MuID, wpid: MuWPID, dis: MuDestClause, ena: MuDestClause },
    NodeCCall { id: MuID, result_ids: Vec<MuID>, callconv: Flag, callee_ty: MuTypeNode, sig: MuFuncSigNode, callee: MuVarNode, args: Vec<MuVarNode>, exc_clause: Option<MuExcClause>, keepalive_clause: Option<MuKeepaliveClause> },
    NodeNewThread { id: MuID, result_id: MuID, stack: MuVarNode, threadlocal: Option<MuVarNode>, new_stack_clause: MuNewStackClause, exc_clause: Option<MuExcClause> },
    NodeSwapStack { id: MuID, result_ids: Vec<MuID>, swappee: MuVarNode, cur_stack_clause: MuCurStackClause, new_stack_clause: MuNewStackClause, exc_clause: Option<MuExcClause>, keepalive_clause: Option<MuKeepaliveClause> },
    NodeCommInst { id: MuID, result_ids: Vec<MuID>, opcode: MuCommInst, flags: Vec<Flag>, tys: Vec<MuTypeNode>, sigs: Vec<MuFuncSigNode>, args: Vec<MuVarNode>, exc_clause: Option<MuExcClause>, keepalive_clause: Option<MuKeepaliveClause> },
}
