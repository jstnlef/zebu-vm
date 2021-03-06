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

use ptr::P;
use types::*;
use inst::*;

use utils::vec_utils;
use utils::LinkedHashMap;
use utils::LinkedHashSet;

use std;
use std::fmt;
pub use std::sync::Arc;
use std::default;
use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};

pub type WPID = usize;
pub type MuID = usize;
pub type MuName = Arc<String>;
pub type CName = MuName;

#[allow(non_snake_case)]
pub fn Mu(str: &'static str) -> MuName {
    Arc::new(str.to_string())
}
#[allow(non_snake_case)]
pub fn C(str: &'static str) -> CName {
    Arc::new(str.to_string())
}

pub type OpIndex = usize;

lazy_static! {
    pub static ref MACHINE_ID : AtomicUsize = {
        let a = ATOMIC_USIZE_INIT;
        a.store(MACHINE_ID_START, Ordering::SeqCst);
        a
    };
    pub static ref INTERNAL_ID : AtomicUsize = {
        let a = ATOMIC_USIZE_INIT;
        a.store(INTERNAL_ID_START, Ordering::SeqCst);
        a
    };
}
/// MuID reserved for machine registers
pub const MACHINE_ID_START: usize = 0;
pub const MACHINE_ID_END: usize = 200;

/// MuID reserved for internal types, etc.
pub const INTERNAL_ID_START: usize = 201;
pub const INTERNAL_ID_END: usize = 500;
pub const USER_ID_START: usize = 1001;

#[deprecated]
#[allow(dead_code)]
// it could happen that one same machine register get different IDs
// during serialization and restoring
// currently I hand-write fixed ID for each machine register
pub fn new_machine_id() -> MuID {
    let ret = MACHINE_ID.fetch_add(1, Ordering::SeqCst);
    if ret >= MACHINE_ID_END {
        panic!("machine id overflow")
    }
    ret
}

pub fn new_internal_id() -> MuID {
    let ret = INTERNAL_ID.fetch_add(1, Ordering::SeqCst);
    if ret >= INTERNAL_ID_END {
        panic!("internal id overflow")
    }
    ret
}

/// MuFunction represents a Mu function (not a specific definition of a function)
/// This stores function signature, and a list of all versions of this function (as ID),
/// and its current version (as ID)
#[derive(Debug)]
pub struct MuFunction {
    pub hdr: MuEntityHeader,

    pub sig: P<MuFuncSig>,
    pub cur_ver: Option<MuID>,
    pub all_vers: Vec<MuID>
}

rodal_struct!(MuFunction {
    hdr,
    sig,
    cur_ver,
    all_vers
});

impl MuFunction {
    pub fn new(entity: MuEntityHeader, sig: P<MuFuncSig>) -> MuFunction {
        MuFunction {
            hdr: entity,
            sig: sig,
            cur_ver: None,
            all_vers: vec![]
        }
    }

    /// adds a new version to this function, and it becomes the current version
    pub fn new_version(&mut self, fv: MuID) {
        if self.cur_ver.is_some() {
            let obsolete_ver = self.cur_ver.unwrap();
            self.all_vers.push(obsolete_ver);
        }

        self.cur_ver = Some(fv);
    }
}

impl fmt::Display for MuFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Func {}", self.hdr)
    }
}

/// MuFunctionVersion represents a specific definition of a Mu function
/// It owns the tree structure of MuIRs for the function version

// FIXME: currently part of compilation information is also stored in this data structure
// we should move them (see Issue #18)
rodal_named!(MuFunctionVersion);
pub struct MuFunctionVersion {
    pub hdr: MuEntityHeader,

    pub func_id: MuID,
    pub sig: P<MuFuncSig>,
    orig_content: Option<FunctionContent>, // original IR
    pub content: Option<FunctionContent>,  // IR that may have been rewritten during compilation
    is_defined: bool,
    is_compiled: bool,
    pub context: FunctionContext,
    pub force_inline: bool,
    pub block_trace: Option<Vec<MuID>> // only available after Trace Generation Pass
}
rodal_struct!(Callsite {
    name,
    exception_destination,
    stack_arg_size
});
#[derive(Debug)]
pub struct Callsite {
    pub name: MuName,
    pub exception_destination: Option<MuName>,
    pub stack_arg_size: usize
}
impl Callsite {
    pub fn new(
        name: MuName,
        exception_destination: Option<MuName>,
        stack_arg_size: usize
    ) -> Callsite {
        Callsite {
            name: name,
            exception_destination: exception_destination,
            stack_arg_size: stack_arg_size
        }
    }
}
impl fmt::Display for MuFunctionVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FuncVer {} of Func #{}", self.hdr, self.func_id)
    }
}

impl fmt::Debug for MuFunctionVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FuncVer {} of Func #{}\n", self.hdr, self.func_id).unwrap();
        write!(f, "Signature: {}\n", self.sig).unwrap();
        write!(f, "IR:\n").unwrap();
        if self.content.is_some() {
            write!(f, "{:?}\n", self.content.as_ref().unwrap()).unwrap();
        } else {
            write!(f, "Empty\n").unwrap();
        }
        if self.block_trace.is_some() {
            write!(f, "Block Trace: {:?}\n", self.block_trace.as_ref().unwrap())
        } else {
            write!(f, "Trace not available\n")
        }
    }
}

impl MuFunctionVersion {
    /// creates an empty function version
    pub fn new(entity: MuEntityHeader, func: MuID, sig: P<MuFuncSig>) -> MuFunctionVersion {
        MuFunctionVersion {
            hdr: entity,
            func_id: func,
            sig: sig,
            orig_content: None,
            content: None,
            is_defined: false,
            is_compiled: false,
            context: FunctionContext::new(),
            block_trace: None,
            force_inline: false
        }
    }

    /// creates a complete function version
    pub fn new_(
        hdr: MuEntityHeader,
        id: MuID,
        sig: P<MuFuncSig>,
        content: FunctionContent,
        context: FunctionContext
    ) -> MuFunctionVersion {
        MuFunctionVersion {
            hdr: hdr,
            func_id: id,
            sig: sig,
            orig_content: Some(content.clone()),
            content: Some(content),
            is_defined: true,
            is_compiled: false,
            context: context,
            block_trace: None,
            force_inline: false
        }
    }

    pub fn get_orig_ir(&self) -> Option<&FunctionContent> {
        self.orig_content.as_ref()
    }

    /// defines function content
    pub fn define(&mut self, content: FunctionContent) {
        if self.is_defined {
            panic!("alread defined the function: {}", self);
        }

        self.is_defined = true;
        self.orig_content = Some(content.clone());
        self.content = Some(content);
    }

    pub fn is_compiled(&self) -> bool {
        self.is_compiled
    }

    pub fn set_compiled(&mut self) {
        self.is_compiled = true;
    }

    pub fn new_ssa(&mut self, entity: MuEntityHeader, ty: P<MuType>) -> P<TreeNode> {
        let id = entity.id();
        let val = P(Value {
            hdr: entity,
            ty: ty,
            v: Value_::SSAVar(id)
        });

        self.context
            .values
            .insert(id, SSAVarEntry::new(val.clone()));

        P(TreeNode {
            v: TreeNode_::Value(val)
        })
    }

    pub fn new_machine_reg(&mut self, v: P<Value>) -> P<TreeNode> {
        self.context
            .values
            .insert(v.id(), SSAVarEntry::new(v.clone()));

        P(TreeNode {
            v: TreeNode_::Value(v)
        })
    }

    pub fn new_constant(&mut self, v: P<Value>) -> P<TreeNode> {
        P(TreeNode {
            v: TreeNode_::Value(v)
        })
    }

    pub fn new_global(&mut self, v: P<Value>) -> P<TreeNode> {
        P(TreeNode {
            v: TreeNode_::Value(v)
        })
    }

    pub fn new_inst(&mut self, v: Instruction) -> P<TreeNode> {
        P(TreeNode {
            v: TreeNode_::Instruction(v)
        })
    }

    /// gets call outedges in this function
    /// returns Map(CallSiteID -> (FuncID, has exception clause))
    pub fn get_static_call_edges(&self) -> LinkedHashMap<MuID, (MuID, bool)> {
        let mut ret = LinkedHashMap::new();

        let f_content = self.content.as_ref().unwrap();

        for (_, block) in f_content.blocks.iter() {
            let block_content = block.content.as_ref().unwrap();

            for inst in block_content.body.iter() {
                match inst.v {
                    TreeNode_::Instruction(ref inst) => {
                        let ref ops = inst.ops;
                        match inst.v {
                            Instruction_::ExprCall { ref data, .. } |
                            Instruction_::ExprCCall { ref data, .. } |
                            Instruction_::Call { ref data, .. } |
                            Instruction_::CCall { ref data, .. } => {
                                let ref callee = ops[data.func];

                                match callee.v {
                                    TreeNode_::Instruction(_) => {}
                                    TreeNode_::Value(ref pv) => {
                                        match &pv.v {
                                            &Value_::Constant(Constant::FuncRef(ref func)) => {
                                                ret.insert(
                                                    inst.id(),
                                                    (func.id(), inst.has_exception_clause())
                                                );
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                            _ => {
                                // do nothing
                            }
                        }
                    }
                    _ => unreachable!()
                }
            }
        }

        ret
    }

    // TODO: It may be more efficient to compute this when the instructions
    // are added to the function version and store the result in a field
    pub fn could_throw(&self) -> bool {
        let f_content = self.content.as_ref().unwrap();

        for (_, block) in f_content.blocks.iter() {
            let block_content = block.content.as_ref().unwrap();

            for inst in block_content.body.iter() {
                match inst.v {
                    TreeNode_::Instruction(ref inst) => {
                        if inst.is_potentially_throwing() {
                            // TODO: Do some smarter checking
                            // (e.g. if this is a CALL to a function where !could_throw,
                            // or a division where the divisor can't possibly be zero..)
                            return true;
                        }
                    }
                    _ => unreachable!()
                }
            }
        }

        false
    }

    pub fn has_tailcall(&self) -> bool {
        let f_content = self.content.as_ref().unwrap();

        for (_, block) in f_content.blocks.iter() {
            let block_content = block.content.as_ref().unwrap();

            for inst in block_content.body.iter() {
                match inst.v {
                    TreeNode_::Instruction(ref inst) => {
                        match inst.v {
                            Instruction_::TailCall(_) => {
                                return true;
                            }
                            _ => {
                                // do nothing
                            }
                        }
                    }
                    _ => unreachable!()
                }
            }
        }

        false
    }
}

/// FunctionContent contains all blocks (which include all instructions) for the function
#[derive(Clone)]
pub struct FunctionContent {
    pub entry: MuID,
    pub blocks: LinkedHashMap<MuID, Block>,
    pub exception_blocks: LinkedHashSet<MuID> // this field only valid after control flow analysis
}

impl fmt::Debug for FunctionContent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let entry = self.get_entry_block();
        write!(f, "Entry block: ").unwrap();
        write!(f, "{:?}\n", entry).unwrap();

        write!(f, "Body:").unwrap();
        for blk_id in self.blocks.keys() {
            let block = self.get_block(*blk_id);
            write!(f, "{:?}\n", block).unwrap();
        }
        Ok(())
    }
}

impl FunctionContent {
    pub fn new(entry: MuID, blocks: LinkedHashMap<MuID, Block>) -> FunctionContent {
        FunctionContent {
            entry: entry,
            blocks: blocks,
            exception_blocks: LinkedHashSet::new()
        }
    }

    pub fn get_entry_block(&self) -> &Block {
        self.get_block(self.entry)
    }

    pub fn get_entry_block_mut(&mut self) -> &mut Block {
        let entry = self.entry;
        self.get_block_mut(entry)
    }

    pub fn get_block(&self, id: MuID) -> &Block {
        let ret = self.blocks.get(&id);
        match ret {
            Some(b) => b,
            None => panic!("cannot find block #{}", id)
        }
    }

    pub fn get_block_mut(&mut self, id: MuID) -> &mut Block {
        let ret = self.blocks.get_mut(&id);
        match ret {
            Some(b) => b,
            None => panic!("cannot find block #{}", id)
        }
    }

    pub fn get_block_by_name(&self, name: MuName) -> &Block {
        for block in self.blocks.values() {
            if block.name() == name {
                return block;
            }
        }

        panic!("cannot find block {}", name)
    }
}

/// FunctionContext contains compilation information about the function
// FIXME: should move this out of ast crate and bind its lifetime with compilation (Issue #18)
#[derive(Default, Debug)]
pub struct FunctionContext {
    pub values: LinkedHashMap<MuID, SSAVarEntry>
}

impl FunctionContext {
    fn new() -> FunctionContext {
        FunctionContext {
            values: LinkedHashMap::new()
        }
    }

    /// makes a TreeNode of an SSA variable
    pub fn make_temporary(&mut self, id: MuID, ty: P<MuType>) -> P<TreeNode> {
        let val = P(Value {
            hdr: MuEntityHeader::unnamed(id),
            ty: ty,
            v: Value_::SSAVar(id)
        });

        self.values.insert(id, SSAVarEntry::new(val.clone()));

        P(TreeNode {
            v: TreeNode_::Value(val)
        })
    }

    /// shows the name for an SSA by ID
    pub fn get_temp_display(&self, id: MuID) -> String {
        match self.get_value(id) {
            Some(entry) => format!("{}", entry.value()),
            None => "CANT_FOUND_ID".to_string()
        }
    }

    /// returns a &SSAVarEntry for the given ID
    pub fn get_value(&self, id: MuID) -> Option<&SSAVarEntry> {
        self.values.get(&id)
    }

    /// returns a &mut SSAVarEntry for the given ID
    pub fn get_value_mut(&mut self, id: MuID) -> Option<&mut SSAVarEntry> {
        self.values.get_mut(&id)
    }
}

/// Block contains BlockContent, which includes all the instructions for the block
//  FIXME: control_flow field should be moved out of ast crate (Issue #18)
//  FIXME: trace_hint should also be moved
#[derive(Clone)]
pub struct Block {
    pub hdr: MuEntityHeader,
    /// the actual content of this block
    pub content: Option<BlockContent>,
    /// a trace scheduling hint about where to layout this block
    pub trace_hint: TraceHint,
    /// control flow info about this block (predecessors, successors, etc)
    pub control_flow: ControlFlow
}

impl fmt::Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Block {}", self.hdr).unwrap();
        writeln!(f, "with preds: {:?}", self.control_flow.preds).unwrap();
        writeln!(f, "     succs: {:?}", self.control_flow.succs).unwrap();
        if self.content.is_some() {
            writeln!(f, "{:?}", self.content.as_ref().unwrap()).unwrap();
        } else {
            writeln!(f, "Empty").unwrap();
        }
        Ok(())
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Block {
    pub fn new(entity: MuEntityHeader) -> Block {
        Block {
            hdr: entity,
            content: None,
            trace_hint: TraceHint::None,
            control_flow: ControlFlow::default()
        }
    }

    pub fn clear_insts(&mut self) {
        self.content.as_mut().unwrap().body.clear();
    }

    pub fn append_inst(&mut self, inst: P<TreeNode>) {
        self.content.as_mut().unwrap().body.push(inst);
    }

    /// does this block have an exception arguments?
    pub fn is_receiving_exception_arg(&self) -> bool {
        return self.content.as_ref().unwrap().exn_arg.is_some();
    }

    /// how many IR instruction does this block have?
    pub fn number_of_irs(&self) -> usize {
        if self.content.is_none() {
            0
        } else {
            let content = self.content.as_ref().unwrap();

            content.body.len()
        }
    }

    /// is this block ends with a conditional branch?
    pub fn ends_with_cond_branch(&self) -> bool {
        let block: &BlockContent = self.content.as_ref().unwrap();
        match block.body.last() {
            Some(node) => {
                match node.v {
                    TreeNode_::Instruction(Instruction {
                        v: Instruction_::Branch2 { .. },
                        ..
                    }) => true,
                    _ => false
                }
            }
            None => false
        }
    }

    /// is this block ends with a return?
    pub fn ends_with_return(&self) -> bool {
        let block: &BlockContent = self.content.as_ref().unwrap();
        match block.body.last() {
            Some(node) => {
                match node.v {
                    TreeNode_::Instruction(Instruction {
                        v: Instruction_::Return(_),
                        ..
                    }) => true,
                    _ => false
                }
            }
            None => false
        }
    }
}

/// TraceHint is a hint for the compiler to generate better trace for this block
//  Note: for a sequence of blocks that are supposed to be fast/slow path, only mark the
//  first block with TraceHint, and let the trace scheduler to normally layout other
//  blocks. Otherwise, the scheduler will take every TraceHint into consideration,
//  and may not generate the trace as expected.
//  FIXME: Issue #18
#[derive(Clone, PartialEq)]
pub enum TraceHint {
    /// no hint provided. Trace scheduler should use its own heuristics to decide
    None,
    /// this block is fast path, and should be put in straightline code where possible
    FastPath,
    /// this block is slow path, and should be kept out of hot loops
    SlowPath,
    /// this block is return sink, and should be put at the end of a function
    ReturnSink
}

/// ControlFlow stores compilation info about control flows of a block
//  FIXME: Issue #18
#[derive(Debug, Clone)]
pub struct ControlFlow {
    pub preds: Vec<MuID>,
    pub succs: Vec<BlockEdge>
}

impl ControlFlow {
    /// returns the successor with highest branching probability
    /// (in case of tie, returns first met successor)
    pub fn get_hottest_succ(&self) -> Option<MuID> {
        if self.succs.len() == 0 {
            None
        } else {
            let mut hot_blk = self.succs[0].target;
            let mut hot_prob = self.succs[0].probability;

            for edge in self.succs.iter() {
                if edge.probability > hot_prob {
                    hot_blk = edge.target;
                    hot_prob = edge.probability;
                }
            }

            Some(hot_blk)
        }
    }
}

impl fmt::Display for ControlFlow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "preds: [{}], ", vec_utils::as_str(&self.preds)).unwrap();
        write!(f, "succs: [{}]", vec_utils::as_str(&self.succs))
    }
}

impl default::Default for ControlFlow {
    fn default() -> ControlFlow {
        ControlFlow {
            preds: vec![],
            succs: vec![]
        }
    }
}

/// BlockEdge represents an edge in control flow graph
#[derive(Copy, Clone, Debug)]
pub struct BlockEdge {
    pub target: MuID,
    pub kind: EdgeKind,
    pub is_exception: bool,
    pub probability: f32
}

impl fmt::Display for BlockEdge {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} ({:?}{} - {})",
            self.target,
            self.kind,
            select_value!(self.is_exception, ", exceptional", ""),
            self.probability
        )
    }
}

#[derive(Copy, Clone, Debug)]
pub enum EdgeKind {
    Forward,
    Backward
}

/// BlockContent describes arguments to this block, and owns all the IR instructions
#[derive(Clone)]
pub struct BlockContent {
    pub args: Vec<P<Value>>,
    pub exn_arg: Option<P<Value>>,
    pub body: Vec<P<TreeNode>>,
    pub keepalives: Option<Vec<P<Value>>>
}

impl fmt::Debug for BlockContent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "args: {}", vec_utils::as_str(&self.args)).unwrap();
        if self.exn_arg.is_some() {
            writeln!(f, "exception arg: {}", self.exn_arg.as_ref().unwrap()).unwrap();
        }
        if self.keepalives.is_some() {
            writeln!(
                f,
                "keepalives: {}",
                vec_utils::as_str(self.keepalives.as_ref().unwrap())
            ).unwrap();
        }
        for node in self.body.iter() {
            writeln!(f, "{}", node).unwrap();
        }
        Ok(())
    }
}

impl BlockContent {
    /// returns all the arguments passed to its successors
    pub fn get_out_arguments(&self) -> Vec<P<Value>> {
        let n_insts = self.body.len();
        let ref last_inst = self.body[n_insts - 1];

        let mut ret: Vec<P<Value>> = vec![];

        match last_inst.v {
            TreeNode_::Instruction(ref inst) => {
                let ref ops = inst.ops;
                match inst.v {
                    Instruction_::Return(_) |
                    Instruction_::ThreadExit |
                    Instruction_::Throw(_) |
                    Instruction_::TailCall(_) => {
                        // they do not have explicit liveouts
                    }
                    Instruction_::Branch1(ref dest) => {
                        let mut live_outs = dest.get_arguments(&ops);
                        vec_utils::add_all_unique(&mut ret, &mut live_outs);
                    }
                    Instruction_::Branch2 {
                        ref true_dest,
                        ref false_dest,
                        ..
                    } => {
                        let mut live_outs = true_dest.get_arguments(&ops);
                        live_outs.append(&mut false_dest.get_arguments(&ops));

                        vec_utils::add_all_unique(&mut ret, &mut live_outs);
                    }
                    Instruction_::Watchpoint {
                        ref disable_dest,
                        ref resume,
                        ..
                    } => {
                        let mut live_outs = vec![];

                        if disable_dest.is_some() {
                            live_outs
                                .append(&mut disable_dest.as_ref().unwrap().get_arguments(&ops));
                        }
                        live_outs.append(&mut resume.normal_dest.get_arguments(&ops));
                        live_outs.append(&mut resume.exn_dest.get_arguments(&ops));

                        vec_utils::add_all_unique(&mut ret, &mut live_outs);
                    }
                    Instruction_::WPBranch {
                        ref disable_dest,
                        ref enable_dest,
                        ..
                    } => {
                        let mut live_outs = vec![];
                        live_outs.append(&mut disable_dest.get_arguments(&ops));
                        live_outs.append(&mut enable_dest.get_arguments(&ops));
                        vec_utils::add_all_unique(&mut ret, &mut live_outs);
                    }
                    Instruction_::Call { ref resume, .. } |
                    Instruction_::CCall { ref resume, .. } |
                    Instruction_::SwapStackExc { ref resume, .. } |
                    Instruction_::ExnInstruction { ref resume, .. } => {
                        let mut live_outs = vec![];
                        live_outs.append(&mut resume.normal_dest.get_arguments(&ops));
                        live_outs.append(&mut resume.exn_dest.get_arguments(&ops));
                        vec_utils::add_all_unique(&mut ret, &mut live_outs);
                    }
                    Instruction_::Switch {
                        ref default,
                        ref branches,
                        ..
                    } => {
                        let mut live_outs = vec![];
                        live_outs.append(&mut default.get_arguments(&ops));
                        for &(_, ref dest) in branches {
                            live_outs.append(&mut dest.get_arguments(&ops));
                        }
                        vec_utils::add_all_unique(&mut ret, &mut live_outs);
                    }

                    _ => panic!("didn't expect last inst as {}", inst)
                }
            }
            _ => panic!("expect last treenode of block is a inst")
        }

        ret
    }

    pub fn clone_empty(&self) -> BlockContent {
        BlockContent {
            args: self.args.clone(),
            exn_arg: self.exn_arg.clone(),
            body: vec![],
            keepalives: self.keepalives.clone()
        }
    }
}

/// TreeNode represents a node in the AST, it could either be an instruction,
/// or an value (SSA, constant, global, etc)
#[derive(Debug, Clone)]
pub struct TreeNode {
    pub v: TreeNode_
}

impl TreeNode {
    /// creates a sharable Instruction TreeNode
    pub fn new_inst(v: Instruction) -> P<TreeNode> {
        P(TreeNode {
            v: TreeNode_::Instruction(v)
        })
    }

    /// creates a sharable Value TreeNode
    pub fn new_value(v: P<Value>) -> P<TreeNode> {
        P(TreeNode {
            v: TreeNode_::Value(v)
        })
    }

    /// is instruction
    pub fn is_inst(&self) -> bool {
        match self.v {
            TreeNode_::Instruction(_) => true,
            _ => false
        }
    }

    /// is value
    pub fn is_value(&self) -> bool {
        match self.v {
            TreeNode_::Value(_) => true,
            _ => false
        }
    }

    /// is constant value
    pub fn is_const_value(&self) -> bool {
        match self.v {
            TreeNode_::Value(ref val) => val.is_const(),
            _ => false
        }
    }

    /// extracts the MuID of an SSA TreeNode
    /// if the node is not an SSA, returns None
    pub fn extract_ssa_id(&self) -> Option<MuID> {
        match self.v {
            TreeNode_::Value(ref pv) => {
                match pv.v {
                    Value_::SSAVar(id) => Some(id),
                    _ => None
                }
            }
            _ => None
        }
    }

    /// clones the value from the TreeNode
    /// * if this is a Instruction TreeNode, returns its first result value
    /// * if this is a value, returns a clone of it
    pub fn clone_value(&self) -> P<Value> {
        self.as_value().clone()
    }

    /// returns the value from the TreeNode
    /// * if this is a Instruction TreeNode, returns its first result value
    /// * if this is a value, returns a clone of it
    pub fn as_value(&self) -> &P<Value> {
        match self.v {
            TreeNode_::Value(ref val) => val,
            TreeNode_::Instruction(ref inst) => {
                let vals = inst.value.as_ref().unwrap();
                if vals.len() != 1 {
                    panic!(
                        "we expect an inst with 1 value, but found multiple or zero \
                         (it should not be here - folded as a child)"
                    );
                }
                &vals[0]
            }
        }
    }

    /// consumes the TreeNode, returns the value in it (or None if it is not a value)
    pub fn into_value(self) -> Option<P<Value>> {
        match self.v {
            TreeNode_::Value(val) => Some(val),
            _ => None
        }
    }

    /// consumes the TreeNode, returns the instruction in it (or None if it is not an instruction)
    pub fn into_inst(self) -> Option<Instruction> {
        match self.v {
            TreeNode_::Instruction(inst) => Some(inst),
            _ => None
        }
    }

    /// consumes the TreeNode, returns the instruction in it (or None if it is not an instruction)
    pub fn as_inst(&self) -> &Instruction {
        match &self.v {
            &TreeNode_::Instruction(ref inst) => inst,
            _ => panic!("expected inst")
        }
    }

    // The type of the node (for a value node)
    pub fn ty(&self) -> P<MuType> {
        match self.v {
            TreeNode_::Instruction(ref inst) => {
                if inst.value.is_some() {
                    let ref value = inst.value.as_ref().unwrap();
                    if value.len() != 1 {
                        panic!("the node {} does not have one result value", self);
                    }

                    value[0].ty.clone()
                } else {
                    panic!("expected result from the node {}", self);
                }
            }
            TreeNode_::Value(ref pv) => pv.ty.clone()
        }
    }
}

impl fmt::Display for TreeNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.v {
            TreeNode_::Value(ref pv) => pv.fmt(f),
            TreeNode_::Instruction(ref inst) => write!(f, "({})", inst)
        }
    }
}

/// TreeNode_ is used for pattern matching for TreeNode
#[derive(Debug, Clone)]
pub enum TreeNode_ {
    Value(P<Value>),
    Instruction(Instruction)
}

/// Value represents a value in the tree, it could be SSA variables, constants, globals,
/// which all will appear in Mu IR. Value may also represent a memory (as in transformed tree,
/// we need to represent memory as well)
///
/// Value should always be used with P<Value> (sharable)
#[derive(PartialEq)]
pub struct Value {
    pub hdr: MuEntityHeader,
    pub ty: P<MuType>,
    pub v: Value_
}

rodal_struct!(Value { hdr, ty, v });

impl Value {
    /// creates an int constant value
    pub fn make_int32_const(id: MuID, val: u64) -> P<Value> {
        Value::make_int_const_ty(id, UINT32_TYPE.clone(), val)
    }

    pub fn make_int64_const(id: MuID, val: u64) -> P<Value> {
        Value::make_int_const_ty(id, UINT64_TYPE.clone(), val)
    }

    pub fn make_int_const_ty(id: MuID, ty: P<MuType>, val: u64) -> P<Value> {
        P(Value {
            hdr: MuEntityHeader::unnamed(id),
            ty: ty,
            v: Value_::Constant(Constant::Int(val))
        })
    }

    pub fn is_mem(&self) -> bool {
        match self.v {
            Value_::Memory(_) => true,
            _ => false
        }
    }

    pub fn is_reg(&self) -> bool {
        match self.v {
            Value_::SSAVar(_) => true,
            _ => false
        }
    }

    pub fn is_const(&self) -> bool {
        match self.v {
            Value_::Constant(_) => true,
            _ => false
        }
    }

    pub fn is_const_zero(&self) -> bool {
        match self.v {
            Value_::Constant(Constant::Int(val)) if val == 0 => true,
            Value_::Constant(Constant::Double(val)) if val == 0f64 => true,
            Value_::Constant(Constant::Float(val)) if val == 0f32 => true,
            Value_::Constant(Constant::IntEx(ref vec)) => {
                if vec.iter().all(|x| *x == 0) {
                    true
                } else {
                    false
                }
            }
            Value_::Constant(Constant::NullRef) => true,
            _ => false
        }
    }

    /// disguises a value as another type.
    /// This is usually used for treat an integer type as an integer of a different length
    /// This method is unsafe
    pub unsafe fn as_type(&self, ty: P<MuType>) -> P<Value> {
        P(Value {
            hdr: self.hdr.clone(),
            ty: ty,
            v: self.v.clone()
        })
    }

    pub fn is_int_ex_const(&self) -> bool {
        match self.v {
            Value_::Constant(Constant::IntEx(_)) => true,
            _ => false
        }
    }


    pub fn is_func_const(&self) -> bool {
        match self.v {
            Value_::Constant(Constant::FuncRef(_)) => true,
            _ => false
        }
    }

    pub fn is_int_const(&self) -> bool {
        match self.v {
            Value_::Constant(Constant::Int(_)) => true,
            Value_::Constant(Constant::NullRef) => true,
            _ => false
        }
    }

    pub fn is_fp_const(&self) -> bool {
        match self.v {
            Value_::Constant(Constant::Float(_)) => true,
            Value_::Constant(Constant::Double(_)) => true,
            _ => false
        }
    }

    pub fn extract_int_const(&self) -> Option<u64> {
        match self.v {
            Value_::Constant(Constant::Int(val)) => Some(val),
            Value_::Constant(Constant::NullRef) => Some(0),
            _ => None
        }
    }

    pub fn extract_int_ex_const(&self) -> Vec<u64> {
        match self.v {
            Value_::Constant(Constant::IntEx(ref val)) => val.clone(),
            _ => panic!("expect int ex const")
        }
    }

    pub fn extract_ssa_id(&self) -> Option<MuID> {
        match self.v {
            Value_::SSAVar(id) => Some(id),
            _ => None
        }
    }

    pub fn extract_memory_location(&self) -> Option<MemoryLocation> {
        match self.v {
            Value_::Memory(ref loc) => Some(loc.clone()),
            _ => None
        }
    }
}

const DISPLAY_ID: bool = true;
const DISPLAY_TYPE: bool = true;
const PRINT_ABBREVIATE_NAME: bool = true;

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if DISPLAY_TYPE {
            match self.v {
                Value_::SSAVar(_) => write!(f, "/*<{}>*/{}", self.ty, self.hdr),
                Value_::Constant(ref c) => {
                    if self.is_func_const() {
                        write!(f, "/*<{}>*/{}", self.ty, c)
                    } else {
                        write!(f, "<{}>{}", self.ty, c)
                    }
                }
                Value_::Global(ref ty) => write!(f, "/*<{}>*/@{}", ty, self.hdr),
                Value_::Memory(ref mem) => write!(f, "/*<{}>*/{}{}", self.ty, self.hdr, mem)
            }
        } else {
            match self.v {
                Value_::SSAVar(_) => write!(f, "{}", self.hdr),
                Value_::Constant(ref c) => {
                    if self.is_func_const() {
                        write!(f, "{}", c)
                    } else {
                        write!(f, "<{}>{}", self.ty, c)
                    }
                }
                Value_::Global(_) => write!(f, "@{}", self.hdr),
                Value_::Memory(ref mem) => write!(f, "{}{}", self.hdr, mem)
            }
        }
    }
}

/// Value_ is used for pattern matching for Value
#[derive(Debug, Clone, PartialEq)]
pub enum Value_ {
    SSAVar(MuID),
    Constant(Constant),
    Global(P<MuType>), // what type is this global (without IRef)
    Memory(MemoryLocation)
}

rodal_enum!(Value_{(SSAVar: id), (Constant: val), (Global: ty), (Memory: location)});

/// SSAVarEntry represent compilation info for an SSA variable
//  FIXME: Issue#18
#[derive(Debug)]
pub struct SSAVarEntry {
    val: P<Value>,

    // how many times this entry is used
    // available after DefUse pass
    use_count: AtomicUsize,

    // this field is only used during TreeGeneration pass
    expr: Option<Instruction>,

    // some ssa vars (such as int128) needs to be split into smaller vars
    split: Option<Vec<P<Value>>>,

    // which instruction defines this value
    def: Option<P<TreeNode>>
}

impl SSAVarEntry {
    pub fn new(val: P<Value>) -> SSAVarEntry {
        let ret = SSAVarEntry {
            val: val,
            use_count: ATOMIC_USIZE_INIT,
            expr: None,
            split: None,
            def: None
        };

        ret.use_count.store(0, Ordering::SeqCst);

        ret
    }

    pub fn ty(&self) -> &P<MuType> {
        &self.val.ty
    }

    pub fn value(&self) -> &P<Value> {
        &self.val
    }

    pub fn use_count(&self) -> usize {
        self.use_count.load(Ordering::SeqCst)
    }
    pub fn increase_use_count(&self) {
        self.use_count.fetch_add(1, Ordering::SeqCst);
    }
    pub fn reset_use_count(&self) {
        self.use_count.store(0, Ordering::SeqCst);
    }

    pub fn has_expr(&self) -> bool {
        self.expr.is_some()
    }
    pub fn assign_expr(&mut self, expr: Instruction) {
        self.expr = Some(expr)
    }
    pub fn take_expr(&mut self) -> Instruction {
        debug_assert!(self.has_expr());
        self.expr.take().unwrap()
    }

    pub fn has_split(&self) -> bool {
        self.split.is_some()
    }
    pub fn set_split(&mut self, vec: Vec<P<Value>>) {
        self.split = Some(vec);
    }
    pub fn get_split(&self) -> &Option<Vec<P<Value>>> {
        &self.split
    }

    pub fn has_def(&self) -> bool {
        self.def.is_some()
    }
    pub fn set_def(&mut self, d: P<TreeNode>) {
        self.def = Some(d);
    }
    pub fn get_def(&self) -> &Option<P<TreeNode>> {
        &self.def
    }
}

impl fmt::Display for SSAVarEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.val)
    }
}

/// Constant presents all kinds of constant that can appear in MuIR
#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    /// all integer constants are stored as u64
    Int(u64),
    IntEx(Vec<u64>),
    /// float constants
    Float(f32),
    /// double constants
    Double(f64),
    /// function reference
    FuncRef(MuEntityRef),
    /// vector constant (currently not used)
    Vector(Vec<Constant>),
    /// null reference
    NullRef,
    /// external symbol
    ExternSym(CName),
    /// a composite type of several constants (currently not used)
    List(Vec<P<Value>>)
}

rodal_enum!(Constant{(Int: val), (IntEx: val), (Float: val), (Double: val), (FuncRef: val),
    (Vector: val), NullRef, (ExternSym: val), (List: val)});

impl fmt::Display for Constant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Constant::Int(v) => write!(f, "{}", v as i64),
            &Constant::IntEx(ref v) => {
                let mut res = format!("");
                // Stored in little-endian order, but we need to display it in big-endian order
                for i in 1..v.len() + 1 {
                    res.push_str(format!("{:016X}", v[v.len() - i]).to_string().as_str());
                }
                write!(f, "0x{}", res)
            }
            &Constant::Float(v) => write!(f, "{}", v),
            &Constant::Double(v) => write!(f, "{}", v),
            //            &Constant::IRef(v) => write!(f, "{}", v),
            &Constant::FuncRef(ref v) => write!(f, "{}", v.name),
            &Constant::Vector(ref v) => {
                // TODO: Make this Muc compatible?
                write!(f, "[").unwrap();
                for i in 0..v.len() {
                    write!(f, "{}", v[i]).unwrap();
                    if i != v.len() - 1 {
                        write!(f, ", ").unwrap();
                    }
                }
                write!(f, "]")
            }
            &Constant::NullRef => write!(f, "NULL"),
            &Constant::ExternSym(ref name) => write!(f, "EXTERN \"{}\"", name),

            &Constant::List(ref vec) => {
                write!(f, "List(").unwrap();
                for val in vec.iter() {
                    write!(f, "{}, ", val).unwrap();
                }
                write!(f, ")")
            }
        }
    }
}

/// MemoryLocation represents a memory value
/// This enumerate type is target dependent
#[cfg(target_arch = "x86_64")]
#[derive(Debug, Clone, PartialEq)]
pub enum MemoryLocation {
    /// addr = base + offset + index * scale
    Address {
        base: P<Value>, // +8
        offset: Option<P<Value>>,
        index: Option<P<Value>>,
        scale: Option<u8>
    },
    /// addr = base + label(offset)
    Symbolic {
        base: Option<P<Value>>,
        label: MuName,
        is_global: bool,
        is_native: bool
    }
}

#[cfg(target_arch = "x86_64")]
rodal_enum!(MemoryLocation{{Address: scale, base, offset, index},
    {Symbolic: is_global, is_native, base, label}});

#[cfg(target_arch = "x86_64")]
impl fmt::Display for MemoryLocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &MemoryLocation::Address {
                ref base,
                ref offset,
                ref index,
                scale
            } => {
                // base
                write!(f, "[{}", base).unwrap();
                // offset
                if offset.is_some() {
                    write!(f, " + {}", offset.as_ref().unwrap()).unwrap();
                }
                // index/scale
                if index.is_some() && scale.is_some() {
                    write!(f, " + {} * {}", index.as_ref().unwrap(), scale.unwrap()).unwrap();
                }
                write!(f, "]")
            }
            &MemoryLocation::Symbolic {
                ref base,
                ref label,
                ..
            } => {
                if base.is_some() {
                    write!(f, "{}({})", label, base.as_ref().unwrap())
                } else {
                    write!(f, "{}", label)
                }
            }
        }
    }
}

/// MemoryLocation represents a memory value
/// This enumerate type is target dependent
#[cfg(target_arch = "aarch64")]
#[derive(Debug, Clone, PartialEq)]
pub enum MemoryLocation {
    /// Represents how an address should be computed,
    /// will need to be converted to a real Address before being used
    VirtualAddress {
        /// Represents base + offset*scale
        /// With offset being interpreted as signed if 'signed' is true
        base: P<Value>, //+8
        offset: Option<P<Value>>, //+16
        signed: bool,             //+1
        scale: u64                //+24
    },
    Address {
        /// Must be a normal 64-bit register or SP
        base: P<Value>,
        /// Can be any GPR or a 12-bit unsigned immediate << n
        offset: Option<P<Value>>,
        /// valid values are 0, log2(n)
        shift: u8,
        /// Whether offset is signed or not (only set this if offset is a register)
        /// Note: n is the number of bytes the adress refers two
        signed: bool
    },
    Symbolic {
        label: MuName,
        is_global: bool,
        is_native: bool
    }
}

#[cfg(target_arch = "aarch64")]
rodal_enum!(MemoryLocation{
    {VirtualAddress: signed, base, offset, scale},
    {Address: base, offset, shift, signed},
    {Symbolic: is_global, is_native, label}}
);

#[cfg(target_arch = "aarch64")]
impl fmt::Display for MemoryLocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &MemoryLocation::VirtualAddress {
                ref base,
                ref offset,
                scale,
                signed
            } => {
                write!(f, "[{}", base).unwrap();

                if offset.is_some() {
                    let sign_type = if signed { "SInt" } else { "UInt" };
                    write!(f, " + {}({})", sign_type, offset.as_ref().unwrap()).unwrap();
                }

                write!(f, " * {}", scale).unwrap();
                write!(f, "]")
            }
            &MemoryLocation::Address {
                ref base,
                ref offset,
                shift,
                signed
            } => {
                write!(f, "[{}", base).unwrap();

                if offset.is_some() {
                    let sign_type = if signed { "SInt" } else { "UInt" };
                    write!(f, " + {}({})", sign_type, offset.as_ref().unwrap()).unwrap();
                }

                if shift != 0 {
                    write!(f, " LSL {}", shift).unwrap();
                }
                write!(f, "]")
            }
            &MemoryLocation::Symbolic { ref label, .. } => write!(f, "{}", label)
        }
    }
}

/// MuEntityHeader is a prefix struct for all Mu Entities (who have an Mu ID, and possibly a name)
#[repr(C)]
#[derive(Debug)] // Display, PartialEq, Clone
pub struct MuEntityHeader {
    id: MuID,
    name: MuName
}
rodal_struct!(MuEntityHeader{id, name});
pub type MuEntityRef = MuEntityHeader;

impl Clone for MuEntityHeader {
    fn clone(&self) -> Self {
        MuEntityHeader {
            id: self.id,
            name: self.name.clone()
        }
    }
}

/// returns true if name is a valid_c identifier
/// (i.e. it contains only ASCII letters, digits and underscores
/// and does not start with "__" or an digit.
pub fn is_valid_c_identifier(name: &MuName) -> bool {
    let mut i = 0;
    let mut underscore = false; // whether the first character is an underscore
    for c in name.chars() {
        match c {
            '_' => {
                if i == 0 {
                    underscore = true;
                } else if i == 1 && underscore {
                    return false;
                }
            }
            '0'...'9' => {
                if i == 0 {
                    return false;
                }
            }
            'a'...'z' | 'A'...'Z' => {}
            _ => {
                return false;
            }
        }
        i += 1;
    }
    return true;
}

/// changes name to mangled name
/// This will always return a valid C identifier
pub fn mangle_name(name: MuName) -> String {
    let name = name.replace('@', "");
    if name.starts_with("__mu_") {
        // TODO: Get rid of this, since it will be triggered if a client provides a name
        // starting with "__mu" which is totally valid, it's only here for the moment to
        // debug name handling
        panic!("Trying to mangle \"{}\", which is already mangled", name.clone());
    }

    assert!(!name.starts_with("%"));

    // Note: a ':'  and '#' is only used by names generated by zebu itself
    let name = name.replace('Z', "ZZ")
        .replace('.', "Zd")
        .replace('-', "Zh")
        .replace(':', "Zc")
        .replace('#', "Za");
    "__mu_".to_string() + name.as_str()
}

/// demangles a Mu name
//  WARNING: This only reverses mangle_name above when no warning is issued)
pub fn demangle_name(mut name: String) -> MuName {
    let name = if cfg!(target_os = "macos") && name.starts_with("___mu_") {
        name.split_off(1)
    } else {
        name
    };

    if name.starts_with("%") {
        panic!("The name '{}'' is local", name);
    }
    if !name.starts_with("__mu_") {
        panic!("Trying to demangle \"{}\", which is not mangled", name.clone());
    }
    let name = name.split_at("__mu_".len()).1.to_string();
    let name = name.replace("Za", "#")
        .replace("Zc", ":")
        .replace("Zh", "-")
        .replace("Zd", ".")
        .replace("ZZ", "Z");
    Arc::new(name)
}

extern crate regex;

/// identifies mu names and demangles them
pub fn demangle_text(text: &String) -> String {
    use self::regex::Regex;

    lazy_static!{
        static ref IDENT_NAME: Regex = if cfg!(target_os = "macos") {
            Regex::new(r"___mu_\w+").unwrap()
        } else {
            Regex::new(r"__mu_\w+").unwrap()
        };
    }

    let mut res = text.clone();
    for cap in IDENT_NAME.captures_iter(&text) {
        let name = cap.get(0).unwrap().as_str().to_string();
        let demangled = demangle_name(name.clone());
        res = res.replacen(&name, &demangled, 1);
    }

    res
}


impl MuEntityHeader {
    pub fn unnamed(id: MuID) -> MuEntityHeader {
        MuEntityHeader {
            id: id,
            name: Arc::new(format!("#{}", id))
        }
    }

    pub fn named(id: MuID, name: MuName) -> MuEntityHeader {
        MuEntityHeader {
            id: id,
            name: Arc::new(name.replace('@', ""))
        }
    }

    pub fn id(&self) -> MuID {
        self.id
    }

    pub fn name(&self) -> MuName {
        self.name.clone()
    }

    /// an abbreviate (easy reading) version of the name
    pub fn abbreviate_name(&self) -> String {
        if PRINT_ABBREVIATE_NAME {
            self.name.split('.').last().unwrap().to_string()
        } else {
            (*self.name()).clone()
        }
    }

    pub fn clone_with_id(&self, new_id: MuID) -> MuEntityHeader {
        let mut clone = self.clone();
        clone.id = new_id;
        clone.name = Arc::new(format!("{}-#{}", clone.name, clone.id));
        clone
    }
}

impl PartialEq for MuEntityHeader {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl fmt::Display for MuEntityHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if DISPLAY_ID {
            write!(f, "{}/*{}*/", self.abbreviate_name(), self.id)
        } else {
            write!(f, "{}", self.abbreviate_name())
        }
    }
}

/// MuEntity trait allows accessing id and name on AST data structures
pub trait MuEntity {
    fn id(&self) -> MuID;
    fn name(&self) -> MuName;
    fn as_entity(&self) -> &MuEntity;
}

// The following structs defined in this module implement MuEntity
// TreeNode implements MuEntity in a different way

impl_mu_entity!(MuFunction);
impl_mu_entity!(MuFunctionVersion);
impl_mu_entity!(Block);
impl_mu_entity!(MuType);
impl_mu_entity!(Value);
impl_mu_entity!(MuFuncSig);

impl MuEntity for TreeNode {
    fn id(&self) -> MuID {
        match self.v {
            TreeNode_::Instruction(ref inst) => inst.id(),
            TreeNode_::Value(ref pv) => pv.id()
        }
    }

    fn name(&self) -> MuName {
        match self.v {
            TreeNode_::Instruction(ref inst) => inst.name(),
            TreeNode_::Value(ref pv) => pv.name()
        }
    }

    fn as_entity(&self) -> &MuEntity {
        match self.v {
            TreeNode_::Instruction(ref inst) => inst.as_entity(),
            TreeNode_::Value(ref pv) => pv.as_entity()
        }
    }
}
