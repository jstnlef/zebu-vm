use ast::ptr::P;
use ast::types::*;
use ast::inst::*;
use ast::op::*;
use utils::vec_utils::as_str as vector_as_str;
use utils::vec_utils;

use std::collections::HashMap;
use std::fmt;
use std::default;
use std::cell::Cell;

pub type WPID  = usize;
pub type MuID  = usize;
pub type MuName = &'static str;
pub type Address = usize; // TODO: replace this with Address(usize)

pub type OpIndex = usize;

#[derive(Debug)]
pub struct MuFunction {
    pub fn_name: MuName,
    pub sig: P<MuFuncSig>,
    pub cur_ver: Option<MuName>,
    pub all_vers: Vec<MuName>
}

impl MuFunction {
    pub fn new(fn_name: MuName, sig: P<MuFuncSig>) -> MuFunction {
        MuFunction {
            fn_name: fn_name,
            sig: sig,
            cur_ver: None,
            all_vers: vec![]
        }
    }
}

#[derive(Debug)]
pub struct MuFunctionVersion {
    pub fn_name: MuName,
    pub version: MuName,

    pub sig: P<MuFuncSig>,
    pub content: Option<FunctionContent>,
    pub context: FunctionContext,

    pub block_trace: Option<Vec<MuName>> // only available after Trace Generation Pass
}

pub const RESERVED_NODE_IDS_FOR_MACHINE : usize = 100;

impl MuFunctionVersion {
    pub fn new(fn_name: MuName, ver: MuName, sig: P<MuFuncSig>) -> MuFunctionVersion {
        MuFunctionVersion{
            fn_name: fn_name,
            version: ver,
            sig: sig,
            content: None,
            context: FunctionContext::new(),
            block_trace: None}
    }

    pub fn define(&mut self, content: FunctionContent) {
        self.content = Some(content)
    }

    pub fn new_ssa(&mut self, id: MuID, tag: MuName, ty: P<MuType>) -> P<TreeNode> {
        self.context.value_tags.insert(tag, id);
        self.context.values.insert(id, SSAVarEntry{id: id, tag: tag, ty: ty.clone(), use_count: Cell::new(0), expr: None});

        P(TreeNode {
            id: id,
            op: pick_op_code_for_ssa(&ty),
            v: TreeNode_::Value(P(Value{
                tag: tag,
                ty: ty,
                v: Value_::SSAVar(id)
            }))
        })
    }

    pub fn new_constant(&mut self, id: MuID, v: P<Value>) -> P<TreeNode> {
        P(TreeNode{
            id: id,
            op: pick_op_code_for_value(&v.ty),
            v: TreeNode_::Value(v)
        })
    }
    
    pub fn new_global(&mut self, id: MuID, v: P<Value>) -> P<TreeNode> {
        P(TreeNode{
            id: id,
            op: pick_op_code_for_value(&v.ty),
            v: TreeNode_::Value(v)
        })
    }

    pub fn new_inst(&mut self, id: MuID, v: Instruction) -> P<TreeNode> {
        P(TreeNode{
            id: id,
            op: pick_op_code_for_inst(&v),
            v: TreeNode_::Instruction(v),
        })
    }
}

#[derive(Debug)]
pub struct FunctionContent {
    pub entry: MuName,
    pub blocks: HashMap<MuName, Block>
}

impl FunctionContent {
    pub fn get_entry_block(&self) -> &Block {
        self.get_block(self.entry)
    }

    pub fn get_entry_block_mut(&mut self) -> &mut Block {
        let entry = self.entry;
        self.get_block_mut(entry)
    }

    pub fn get_block(&self, tag: MuName) -> &Block {
        let ret = self.blocks.get(tag);
        match ret {
            Some(b) => b,
            None => panic!("cannot find block {}", tag)
        }
    }

    pub fn get_block_mut(&mut self, tag: MuName) -> &mut Block {
        let ret = self.blocks.get_mut(tag);
        match ret {
            Some(b) => b,
            None => panic!("cannot find block {}", tag)
        }
    }
}

#[derive(Debug)]
pub struct FunctionContext {
    pub value_tags: HashMap<MuName, MuID>,
    pub values: HashMap<MuID, SSAVarEntry>
}

impl FunctionContext {
    fn new() -> FunctionContext {
        FunctionContext {
            value_tags: HashMap::new(),
            values: HashMap::new()
        }
    }

    pub fn get_value_by_tag(&self, tag: MuName) -> Option<&SSAVarEntry> {
        match self.value_tags.get(tag) {
            Some(id) => self.get_value(*id),
            None => None
        }
    }

    pub fn get_value_mut_by_tag(&mut self, tag: MuName) -> Option<&mut SSAVarEntry> {
        let id : MuID = match self.value_tags.get(tag) {
            Some(id) => *id,
            None => return None
        };

        self.get_value_mut(id)
    }

    pub fn get_value(&self, id: MuID) -> Option<&SSAVarEntry> {
        self.values.get(&id)
    }

    pub fn get_value_mut(&mut self, id: MuID) -> Option<&mut SSAVarEntry> {
        self.values.get_mut(&id)
    }
}

#[derive(Debug)]
pub struct Block {
    pub label: MuName,
    pub content: Option<BlockContent>,
    pub control_flow: ControlFlow
}

impl Block {
    pub fn new(label: MuName) -> Block {
        Block{label: label, content: None, control_flow: ControlFlow::default()}
    }
}

#[derive(Debug)]
pub struct ControlFlow {
    pub preds : Vec<MuName>,
    pub succs : Vec<BlockEdge>
}

impl ControlFlow {
    pub fn get_hottest_succ(&self) -> Option<MuName> {
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
        write!(f, "preds: [{}], ", vector_as_str(&self.preds)).unwrap();
        write!(f, "succs: [{}]", vector_as_str(&self.succs))
    }
}

impl default::Default for ControlFlow {
    fn default() -> ControlFlow {
        ControlFlow {preds: vec![], succs: vec![]}
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BlockEdge {
    pub target: MuName,
    pub kind: EdgeKind,
    pub is_exception: bool,
    pub probability: f32
}

impl fmt::Display for BlockEdge {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({:?}{} - {})", self.target, self.kind, select_value!(self.is_exception, ", exceptional", ""), self.probability)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum EdgeKind {
    Forward, Backward
}

#[derive(Debug)]
pub struct BlockContent {
    pub args: Vec<P<Value>>,
    pub body: Vec<P<TreeNode>>,
    pub keepalives: Option<Vec<P<Value>>>
}

impl BlockContent {
    pub fn get_out_arguments(&self) -> Vec<P<Value>> {
        let n_insts = self.body.len();
        let ref last_inst = self.body[n_insts - 1];
        
        let mut ret : Vec<P<Value>> = vec![];
        
        match last_inst.v {
            TreeNode_::Instruction(ref inst) => {
                let ops = inst.ops.borrow();
                match inst.v {
                    Instruction_::Return(_)
                    | Instruction_::ThreadExit
                    | Instruction_::Throw(_)
                    | Instruction_::TailCall(_) => {
                        // they do not have explicit liveouts
                    }
                    Instruction_::Branch1(ref dest) => {
                        let mut live_outs = dest.get_arguments(&ops);
                        vec_utils::append_unique(&mut ret, &mut live_outs);
                    }
                    Instruction_::Branch2{ref true_dest, ref false_dest, ..} => {
                        let mut live_outs = true_dest.get_arguments(&ops);
                        live_outs.append(&mut false_dest.get_arguments(&ops));
                        
                        vec_utils::append_unique(&mut ret, &mut live_outs);
                    }
                    Instruction_::Watchpoint{ref disable_dest, ref resume, ..} => {
                        let mut live_outs = vec![];
                        
                        if disable_dest.is_some() {
                            live_outs.append(&mut disable_dest.as_ref().unwrap().get_arguments(&ops));
                        }
                        live_outs.append(&mut resume.normal_dest.get_arguments(&ops));
                        live_outs.append(&mut resume.exn_dest.get_arguments(&ops));
                        
                        vec_utils::append_unique(&mut ret, &mut live_outs);
                    }
                    Instruction_::WPBranch{ref disable_dest, ref enable_dest, ..} => {
                        let mut live_outs = vec![];
                        live_outs.append(&mut disable_dest.get_arguments(&ops));
                        live_outs.append(&mut enable_dest.get_arguments(&ops));
                        vec_utils::append_unique(&mut ret, &mut live_outs);
                    }
                    Instruction_::Call{ref resume, ..}
                    | Instruction_::SwapStack{ref resume, ..}
                    | Instruction_::ExnInstruction{ref resume, ..} => {
                        let mut live_outs = vec![];
                        live_outs.append(&mut resume.normal_dest.get_arguments(&ops));
                        live_outs.append(&mut resume.exn_dest.get_arguments(&ops));
                        vec_utils::append_unique(&mut ret, &mut live_outs);
                    }
                    Instruction_::Switch{ref default, ref branches, ..} => {
                        let mut live_outs = vec![];
                        live_outs.append(&mut default.get_arguments(&ops));
                        for &(_, ref dest) in branches {
                            live_outs.append(&mut dest.get_arguments(&ops));
                        }
                        vec_utils::append_unique(&mut ret, &mut live_outs);
                    }
                    
                    _ => panic!("didn't expect last inst as {:?}", inst) 
                }
            },
            _ => panic!("expect last treenode of block is a inst")
        }
        
        ret
    }
}

#[derive(Debug, Clone)]
/// always use with P<TreeNode>
pub struct TreeNode {
    pub id: MuID,
    pub op: OpCode,
    pub v: TreeNode_,
}

impl TreeNode {
    // this is a hack to allow creating TreeNode without using a &mut MuFunctionVersion
    pub fn new_inst(id: MuID, v: Instruction) -> P<TreeNode> {
        P(TreeNode{
            id: id,
            op: pick_op_code_for_inst(&v),
            v: TreeNode_::Instruction(v),
        })
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

    pub fn clone_value(&self) -> P<Value> {
        match self.v {
            TreeNode_::Value(ref val) => val.clone(),
            TreeNode_::Instruction(ref inst) => {
                info!("expecting a value, but we found an inst. Instead we use its first value");
                let vals = inst.value.as_ref().unwrap();
                if vals.len() != 1 {
                    panic!("we expect an inst with 1 value, but found multiple or zero (it should not be here - folded as a child)");
                }
                vals[0].clone()
            }
        }
    }

    pub fn into_value(self) -> Option<P<Value>> {
        match self.v {
            TreeNode_::Value(val) => Some(val),
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
                    },
                    Value_::Global(ref g) => {
                        write!(f, "+({} to GLOBAL {} @{})", pv.ty, g.ty, g.tag)
                    },
                    Value_::Memory(ref mem) => {
                        write!(f, "+({})", mem)
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
#[derive(Debug, Clone, PartialEq)]
pub struct Value {
    pub tag: MuName,
    pub ty: P<MuType>,
    pub v: Value_
}

impl Value {
    pub fn is_int_reg(&self) -> bool {
        match self.v {
            Value_::SSAVar(_) => {
                if is_scalar(&self.ty) && !is_fp(&self.ty) {
                    true
                } else {
                    false
                }
            }
            _ => false
        }
    }

    pub fn is_fp_reg(&self) -> bool {
        match self.v {
            Value_::SSAVar(_) => {
                if is_scalar(&self.ty) && is_fp(&self.ty) {
                    true
                } else {
                    false
                }
            },
            _ => false
        }
    }

    pub fn is_int_const(&self) -> bool {
        match self.v {
            Value_::Constant(_) => {
                let ty : &MuType_ = &self.ty;
                match ty {
                    &MuType_::Int(_) => true,
                    _ => false
                }
            }
            _ => false
        }
    }

    pub fn extract_ssa_id(&self) -> Option<MuID> {
        match self.v {
            Value_::SSAVar(id) => Some(id),
            _ => None
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.v {
            Value_::SSAVar(id) => {
                write!(f, "+({} %{}#{})", self.ty, self.tag, id)
            },
            Value_::Constant(ref c) => {
                write!(f, "+({} {})", self.ty, c)
            },
            Value_::Global(ref g) => {
                write!(f, "+({} to GLOBAL {} @{})", self.ty, g.ty, g.tag)
            },
            Value_::Memory(ref mem) => {
                write!(f, "+({})", mem)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value_ {
    SSAVar(MuID),
    Constant(Constant),
    Global(P<GlobalCell>),
    Memory(MemoryLocation)
}

#[derive(Debug, Clone)]
pub struct SSAVarEntry {
    pub id: MuID,
    pub tag: MuName,
    pub ty: P<MuType>,

    // how many times this entry is used
    // availalbe after DefUse pass
    pub use_count: Cell<usize>,

    // this field is only used during TreeGeneration pass
    pub expr: Option<Instruction>
}

impl SSAVarEntry {
    pub fn assign_expr(&mut self, expr: Instruction) {
        self.expr = Some(expr)
    }
}

impl fmt::Display for SSAVarEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}#{}", self.ty, self.tag, self.id)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    Int(usize),
    Float(f32),
    Double(f64),
    IRef(Address),
    FuncRef(MuName),
    UFuncRef(MuName),
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

#[derive(Debug, Clone, PartialEq)]
pub enum MemoryLocation {
    Address{
        base: P<Value>,
        offset: Option<P<Value>>,
        index: Option<P<Value>>,
        scale: Option<u8>
    },
    Symbolic{
        base: Option<P<Value>>,
        label: MuName
    }
}

impl fmt::Display for MemoryLocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &MemoryLocation::Address{ref base, ref offset, ref index, scale} => {
                write!(f, "{} + {} + {} * {}", base, offset.as_ref().unwrap(), index.as_ref().unwrap(), scale.unwrap())
            }
            &MemoryLocation::Symbolic{ref base, ref label} => {
                if base.is_some() {
                    write!(f, "{}({})", label, base.as_ref().unwrap())
                } else {
                    write!(f, "{}", label)
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalCell {
    pub tag: MuName,
    pub ty: P<MuType>
}

pub fn op_vector_str(vec: &Vec<OpIndex>, ops: &Vec<P<TreeNode>>) -> String {
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
