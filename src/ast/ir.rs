use ast::ptr::P;
use ast::types::*;
use ast::inst::*;
use ast::op::*;
use common::vector_as_str;

use std::collections::HashMap;
use std::fmt;
use std::default;
use std::cell::Cell;

pub type WPID  = usize;
pub type MuID  = usize;
pub type MuTag = &'static str;
pub type Address = usize; // TODO: replace this with Address(usize)

pub type OpIndex = usize;

#[derive(Debug)]
pub struct MuFunction {
    pub fn_name: MuTag,
    
    pub next_id: MuID,
    
    pub sig: P<MuFuncSig>,
    pub content: Option<FunctionContent>,
    pub context: FunctionContext,

    pub block_trace: Option<Vec<MuTag>> // only available after Trace Generation Pass
}

pub const RESERVED_NODE_IDS_FOR_MACHINE : usize = 100;

impl MuFunction {
    pub fn new(fn_name: MuTag, sig: P<MuFuncSig>) -> MuFunction {
        MuFunction{
            fn_name: fn_name,
            next_id: RESERVED_NODE_IDS_FOR_MACHINE, 
            sig: sig, 
            content: None, 
            context: FunctionContext::new(), 
            block_trace: None}
    }
    
    fn get_id(&mut self) -> MuID {
        let ret = self.next_id;
        self.next_id += 1;
        ret
    }

    pub fn define(&mut self, content: FunctionContent) {
        self.content = Some(content)
    }

    pub fn new_ssa(&mut self, tag: MuTag, ty: P<MuType>) -> P<TreeNode> {
        let id = self.get_id();
        
        self.context.values.insert(id, ValueEntry{id: id, tag: tag, ty: ty.clone(), use_count: Cell::new(0), expr: None});

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

    pub fn new_constant(&mut self, v: P<Value>) -> P<TreeNode> {
        P(TreeNode{
            id: self.get_id(),
            op: pick_op_code_for_const(&v.ty),
            v: TreeNode_::Value(v)
        })
    }
    
    pub fn new_inst(&mut self, v: Instruction) -> P<TreeNode> {
        P(TreeNode{
            id: self.get_id(),
            op: pick_op_code_for_inst(&v), 
            v: TreeNode_::Instruction(v),
        })
    }      
}

#[derive(Debug)]
pub struct FunctionContent {
    pub entry: MuTag,
    pub blocks: HashMap<MuTag, Block>
}

impl FunctionContent {
    pub fn get_entry_block(&self) -> &Block {
        self.get_block(self.entry)
    }

    pub fn get_entry_block_mut(&mut self) -> &mut Block {
        self.get_block_mut(self.entry)
    }

    pub fn get_block(&self, tag: MuTag) -> &Block {
        let ret = self.blocks.get(tag);
        match ret {
            Some(b) => b,
            None => panic!("cannot find block {}", tag)
        }
    }

    pub fn get_block_mut(&mut self, tag: MuTag) -> &mut Block {
        let ret = self.blocks.get_mut(tag);
        match ret {
            Some(b) => b,
            None => panic!("cannot find block {}", tag)
        }
    }
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

#[derive(Debug)]
pub struct Block {
    pub label: MuTag,
    pub content: Option<BlockContent>,
    pub control_flow: ControlFlow
}

impl Block {
    pub fn new(label: MuTag) -> Block {
        Block{label: label, content: None, control_flow: ControlFlow::default()}
    }
}

#[derive(Debug)]
pub struct ControlFlow {
    pub preds : Vec<MuTag>,
    pub succs : Vec<BlockEdge>
}

impl ControlFlow {
    pub fn get_hottest_succ(&self) -> Option<MuTag> {
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
    pub target: MuTag,
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

#[derive(Debug, Clone)]
/// always use with P<TreeNode>
pub struct TreeNode {
    pub id: MuID,
    pub op: OpCode,
    pub v: TreeNode_,
}

impl TreeNode {
    // this is a hack to allow creating TreeNode without using a &mut MuFunction
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
            _ => panic!("expecting a value") 
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
            }
        }
    }
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

    // how many times this entry is used
    // availalbe after DefUse pass
    pub use_count: Cell<usize>,

    // this field is only used during TreeGeneration pass
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
    FuncRef(MuTag),
    UFuncRef(MuTag),
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
