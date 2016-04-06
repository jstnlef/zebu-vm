use ast::ptr::P;
use ast::types::*;
use ast::inst::*;
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
    pub sig: P<MuFuncSig>,
    pub content: Option<FunctionContent>,
    pub context: FunctionContext
}

impl MuFunction {
    pub fn new(fn_name: MuTag, sig: P<MuFuncSig>) -> MuFunction {
        MuFunction{fn_name: fn_name, sig: sig, content: None, context: FunctionContext::new()}
    }
    
    pub fn define(&mut self, content: FunctionContent) {
        self.content = Some(content)
    }
    
    pub fn new_ssa(&mut self, id: MuID, tag: MuTag, ty: P<MuType>) -> P<TreeNode> {
        self.context.values.insert(id, ValueEntry{id: id, tag: tag, ty: ty.clone(), use_count: Cell::new(0), expr: None});
        
        P(TreeNode {
            v: TreeNode_::Value(P(Value{
                tag: tag,
                ty: ty,
                v: Value_::SSAVar(id)
            }))
        })
    }
    
    pub fn new_constant(&mut self, tag: MuTag, ty: P<MuType>, v: Constant) -> P<TreeNode> {
        P(TreeNode{
            v: TreeNode_::Value(P(Value{
                tag: tag,
                ty: ty, 
                v: Value_::Constant(v)
            }))
        })
    }
    
    pub fn new_value(&mut self, v: P<Value>) -> P<TreeNode> {
        P(TreeNode{
            v: TreeNode_::Value(v)
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
        self.get_block(self.entry).unwrap()
    }
    
    pub fn get_entry_block_mut(&mut self) -> &mut Block {
        self.get_block_mut(self.entry).unwrap()
    }
    
    pub fn get_block(&self, tag: MuTag) -> Option<&Block> {
        self.blocks.get(tag)
    }
    
    pub fn get_block_mut(&mut self, tag: MuTag) -> Option<&mut Block> {
        self.blocks.get_mut(tag)
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
    pub args: Vec<P<TreeNode>>,
    pub body: Vec<P<TreeNode>>,
    pub keepalives: Option<Vec<P<TreeNode>>>    
}

#[derive(Debug, Clone)]
/// always use with P<TreeNode>
pub struct TreeNode {
//    pub op: OpCode,
    pub v: TreeNode_
}

impl TreeNode {   
    pub fn new_inst(v: Instruction) -> P<TreeNode> {
        P(TreeNode{v: TreeNode_::Instruction(v)})
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
    FuncRef(Address),
    UFuncRef(Address),
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