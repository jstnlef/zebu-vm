use ptr::P;
use types::*;
use inst::*;
use op::*;

use utils::vec_utils;
use utils::Address;

use std::collections::HashMap;
use std::fmt;
use std::default;
use std::sync::RwLock;
use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};

pub type WPID  = usize;
pub type MuID  = usize;
pub type MuName = String;
pub type CName  = MuName;

#[allow(non_snake_case)]
pub fn Mu(str: &'static str) -> MuName {str.to_string()}

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
pub const  MACHINE_ID_START : usize = 0;
pub const  MACHINE_ID_END   : usize = 100;

pub const  INTERNAL_ID_START: usize = 101;
pub const  INTERNAL_ID_END  : usize = 200;
pub const  USER_ID_START    : usize = 201;

#[deprecated]
#[allow(dead_code)]
/// it could happen that one same machine register get different IDs
/// during serialization and restoring
/// currently I hand-write fixed ID for each machine register
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

#[derive(Debug, RustcEncodable, RustcDecodable)]
pub struct MuFunction {
    pub hdr: MuEntityHeader,
    
    pub sig: P<MuFuncSig>,
    pub cur_ver: Option<MuID>,
    pub all_vers: Vec<MuID>
}

impl MuFunction {
    pub fn new(id: MuID, sig: P<MuFuncSig>) -> MuFunction {
        MuFunction {
            hdr: MuEntityHeader::unnamed(id),
            sig: sig,
            cur_ver: None,
            all_vers: vec![]
        }
    }
    
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

#[derive(RustcEncodable, RustcDecodable)]
pub struct MuFunctionVersion {
    pub hdr: MuEntityHeader,
         
    pub func_id: MuID,
    pub sig: P<MuFuncSig>,
    pub content: Option<FunctionContent>,
    pub context: FunctionContext,

    pub block_trace: Option<Vec<MuID>> // only available after Trace Generation Pass
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
            write!(f, "{:?}\n", self.block_trace.as_ref().unwrap())
        } else {
            write!(f, "Trace not available\n")
        }
    }
}

impl MuFunctionVersion {
    pub fn new(id: MuID, func: MuID, sig: P<MuFuncSig>) -> MuFunctionVersion {
        MuFunctionVersion{
            hdr: MuEntityHeader::unnamed(id),
            func_id: func,
            sig: sig,
            content: None,
            context: FunctionContext::new(),
            block_trace: None
        }
    }

    pub fn define(&mut self, content: FunctionContent) {
        self.content = Some(content);
    }

    pub fn new_ssa(&mut self, id: MuID, ty: P<MuType>) -> P<TreeNode> {
        let val = P(Value{
            hdr: MuEntityHeader::unnamed(id),
            ty: ty,
            v: Value_::SSAVar(id)
        });

        self.context.values.insert(id, SSAVarEntry::new(val.clone()));

        P(TreeNode {
            op: pick_op_code_for_ssa(&val.ty),
            v: TreeNode_::Value(val)
        })
    }

    pub fn new_constant(&mut self, v: P<Value>) -> P<TreeNode> {
        P(TreeNode{
            op: pick_op_code_for_value(&v.ty),
            v: TreeNode_::Value(v)
        })
    }
    
    pub fn new_global(&mut self, v: P<Value>) -> P<TreeNode> {
        P(TreeNode{
            op: pick_op_code_for_value(&v.ty),
            v: TreeNode_::Value(v)
        })
    }

    pub fn new_inst(&mut self, v: Instruction) -> Box<TreeNode> {
        Box::new(TreeNode{
            op: pick_op_code_for_inst(&v),
            v: TreeNode_::Instruction(v),
        })
    }
}

#[derive(RustcEncodable, RustcDecodable, Clone)]
pub struct FunctionContent {
    pub entry: MuID,
    pub blocks: HashMap<MuID, Block>
}

impl fmt::Debug for FunctionContent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let entry = self.get_entry_block();
        write!(f, "{:?}\n", entry).unwrap();
        
        for blk_id in self.blocks.keys() {
            let block = self.get_block(*blk_id);
            write!(f, "{:?}\n", block).unwrap();
        }
        Ok(())
    }
}

impl FunctionContent {
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
}

#[derive(Default, Debug, RustcEncodable, RustcDecodable)]
pub struct FunctionContext {
    pub values: HashMap<MuID, SSAVarEntry>
}

impl FunctionContext {
    fn new() -> FunctionContext {
        FunctionContext {
            values: HashMap::new()
        }
    }
    
    pub fn make_temporary(&mut self, id: MuID, ty: P<MuType>) -> P<TreeNode> {
        let val = P(Value{
            hdr: MuEntityHeader::unnamed(id),
            ty: ty,
            v: Value_::SSAVar(id)
        });

        self.values.insert(id, SSAVarEntry::new(val.clone()));

        P(TreeNode {
            op: pick_op_code_for_ssa(&val.ty),
            v: TreeNode_::Value(val)
        })
    }

    pub fn get_temp_display(&self, id: MuID) -> String {
        match self.get_value(id) {
            Some(entry) => format!("{}", entry.value()),
            None => "CANT_FOUND_ID".to_string()
        }
    }

    pub fn get_value(&self, id: MuID) -> Option<&SSAVarEntry> {
        self.values.get(&id)
    }

    pub fn get_value_mut(&mut self, id: MuID) -> Option<&mut SSAVarEntry> {
        self.values.get_mut(&id)
    }
}

#[derive(RustcEncodable, RustcDecodable, Clone)]
pub struct Block {
    pub hdr: MuEntityHeader,
    pub content: Option<BlockContent>,
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

impl Block {
    pub fn new(id: MuID) -> Block {
        Block{hdr: MuEntityHeader::unnamed(id), content: None, control_flow: ControlFlow::default()}
    }
    
    pub fn is_exception_block(&self) -> bool {
        return self.content.as_ref().unwrap().exn_arg.is_some()
    }
}

#[derive(Debug, RustcEncodable, RustcDecodable, Clone)]
pub struct ControlFlow {
    pub preds : Vec<MuID>,
    pub succs : Vec<BlockEdge>
}

impl ControlFlow {
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
        ControlFlow {preds: vec![], succs: vec![]}
    }
}

#[derive(Copy, Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct BlockEdge {
    pub target: MuID,
    pub kind: EdgeKind,
    pub is_exception: bool,
    pub probability: f32
}

impl fmt::Display for BlockEdge {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({:?}{} - {})", self.target, self.kind, select_value!(self.is_exception, ", exceptional", ""), self.probability)
    }
}

#[derive(Copy, Clone, Debug, RustcEncodable, RustcDecodable)]
pub enum EdgeKind {
    Forward, Backward
}

#[derive(RustcEncodable, RustcDecodable, Clone)]
pub struct BlockContent {
    pub args: Vec<P<Value>>,
    pub exn_arg: Option<P<Value>>,
    pub body: Vec<Box<TreeNode>>,
    pub keepalives: Option<Vec<P<Value>>>
}

impl fmt::Debug for BlockContent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "args: {}", vec_utils::as_str(&self.args)).unwrap();
        if self.exn_arg.is_some() {
            writeln!(f, "exception arg: {}", self.exn_arg.as_ref().unwrap()).unwrap();
        }
        if self.keepalives.is_some() {
            writeln!(f, "keepalives: {}", vec_utils::as_str(self.keepalives.as_ref().unwrap())).unwrap();
        }
        for node in self.body.iter() {
            writeln!(f, "{}", node).unwrap();
        }
        Ok(())
    }
}

impl BlockContent {
    pub fn get_out_arguments(&self) -> Vec<P<Value>> {
        let n_insts = self.body.len();
        let ref last_inst = self.body[n_insts - 1];
        
        let mut ret : Vec<P<Value>> = vec![];
        
        match last_inst.v {
            TreeNode_::Instruction(ref inst) => {
                let ops = inst.ops.read().unwrap();
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

#[derive(Debug, RustcEncodable, RustcDecodable, Clone)]
/// always use with P<TreeNode>
pub struct TreeNode {
    pub op: OpCode,
    pub v: TreeNode_,
}

impl TreeNode {
    // this is a hack to allow creating TreeNode without using a &mut MuFunctionVersion
    pub fn new_inst(v: Instruction) -> P<TreeNode> {
        P(TreeNode{
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
                warn!("expecting a value, but we found an inst. Instead we use its first value");
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
            TreeNode_::Value(ref pv) => pv.fmt(f),
            TreeNode_::Instruction(ref inst) => {
                write!(f, "+({})", inst)
            }
        }
    }
}

#[derive(Debug, RustcEncodable, RustcDecodable, Clone)]
pub enum TreeNode_ {
    Value(P<Value>),
    Instruction(Instruction)
}

/// always use with P<Value>
#[derive(Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub struct Value {
    pub hdr: MuEntityHeader,
    pub ty: P<MuType>,
    pub v: Value_
}

impl Value {
    pub fn make_int_const(id: MuID, val: u64) -> P<Value> {
        P(Value{
            hdr: MuEntityHeader::unnamed(id),
            ty: UINT32_TYPE.clone(),
            v: Value_::Constant(Constant::Int(val))
        })
    }
    
    pub fn is_mem(&self) -> bool {
        match self.v {
            Value_::Memory(_) => true,
            _ => false
        }
    }
    
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
                let ty : &MuType = &self.ty;
                match ty.v {
                    MuType_::Int(_) => true,
                    _ => false
                }
            }
            _ => false
        }
    }
    
    pub fn extract_int_const(&self) -> u64 {
        match self.v {
            Value_::Constant(ref c) => {
                match c {
                    &Constant::Int(val) => val,
                    _ => panic!("expect int const")
                }
            },
            _ => panic!("expect int const")
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
            Value_::SSAVar(_) => {
                write!(f, "+({} %{})", self.ty, self.hdr)
            },
            Value_::Constant(ref c) => {
                write!(f, "+({} {} @{})", self.ty, c, self.hdr)
            },
            Value_::Global(ref ty) => {
                write!(f, "+(GLOBAL {} @{})", ty, self.hdr)
            },
            Value_::Memory(ref mem) => {
                write!(f, "+(MEM {} %{})", mem, self.hdr)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, RustcEncodable, RustcDecodable)]
pub enum Value_ {
    SSAVar(MuID),
    Constant(Constant),
    Global(P<MuType>), // what type is this global (without IRef)
    Memory(MemoryLocation)
}

#[derive(Debug)]
pub struct SSAVarEntry {
    val: P<Value>,

    // how many times this entry is used
    // availalbe after DefUse pass
    use_count: AtomicUsize,

    // this field is only used during TreeGeneration pass
    expr: Option<Instruction>
}

impl Encodable for SSAVarEntry {
    fn encode<S: Encoder> (&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_struct("SSAVarEntry", 3, |s| {
            try!(s.emit_struct_field("val", 0, |s| self.val.encode(s)));
            let count = self.use_count.load(Ordering::SeqCst);
            try!(s.emit_struct_field("use_count", 1, |s| s.emit_usize(count)));
            try!(s.emit_struct_field("expr", 2, |s| self.expr.encode(s)));
            Ok(())
        })
    }
}

impl Decodable for SSAVarEntry {
    fn decode<D: Decoder>(d: &mut D) -> Result<SSAVarEntry, D::Error> {
        d.read_struct("SSAVarEntry", 3, |d| {
            let val = try!(d.read_struct_field("val", 0, |d| Decodable::decode(d)));
            let count = try!(d.read_struct_field("use_count", 1, |d| d.read_usize()));
            let expr = try!(d.read_struct_field("expr", 2, |d| Decodable::decode(d)));
            
            let ret = SSAVarEntry {
                val: val,
                use_count: ATOMIC_USIZE_INIT,
                expr: expr
            };
            
            ret.use_count.store(count, Ordering::SeqCst);
            
            Ok(ret)
        })
    }
}

impl SSAVarEntry {
    pub fn new(val: P<Value>) -> SSAVarEntry {
        let ret = SSAVarEntry {
            val: val,
            use_count: ATOMIC_USIZE_INIT,
            expr: None
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
}

impl fmt::Display for SSAVarEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.val)
    }
}

#[derive(Debug, Clone, PartialEq, RustcEncodable, RustcDecodable)]
pub enum Constant {
    Int(u64),
    Float(f32),
    Double(f64),
//    IRef(Address),
    FuncRef(MuID),
    UFuncRef(MuID),
    Vector(Vec<Constant>),
    //Pointer(Address),
    NullRef,
}

impl fmt::Display for Constant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Constant::Int(v) => write!(f, "{}", v),
            &Constant::Float(v) => write!(f, "{}", v),
            &Constant::Double(v) => write!(f, "{}", v),
//            &Constant::IRef(v) => write!(f, "{}", v),
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
            &Constant::NullRef => write!(f, "NullRef"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, RustcEncodable, RustcDecodable)]
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

#[repr(C)]
#[derive(Debug)] // Display, PartialEq, Clone
pub struct MuEntityHeader {
    id: MuID,
    name: RwLock<Option<MuName>>
}

impl Clone for MuEntityHeader {
    fn clone(&self) -> Self {
        MuEntityHeader {
            id: self.id,
            name: RwLock::new(self.name.read().unwrap().clone())
        }
    }
}

use rustc_serialize::{Encodable, Encoder, Decodable, Decoder};
impl Encodable for MuEntityHeader {
    fn encode<S: Encoder> (&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_struct("MuEntityHeader", 2, |s| {
            try!(s.emit_struct_field("id", 0, |s| self.id.encode(s)));
            
            let name = &self.name.read().unwrap();
            try!(s.emit_struct_field("name", 1, |s| name.encode(s)));
            
            Ok(())
        })
    }
}

impl Decodable for MuEntityHeader {
    fn decode<D: Decoder>(d: &mut D) -> Result<MuEntityHeader, D::Error> {
        d.read_struct("MuEntityHeader", 2, |d| {
            let id = try!(d.read_struct_field("id", 0, |d| {d.read_usize()}));
            let name = try!(d.read_struct_field("name", 1, |d| Decodable::decode(d)));
            
            Ok(MuEntityHeader{
                    id: id,
                    name: RwLock::new(name)
                })
        })
    }
}

impl MuEntityHeader {
    pub fn unnamed(id: MuID) -> MuEntityHeader {
        MuEntityHeader {
            id: id,
            name: RwLock::new(None)
        }
    }
    
    pub fn named(id: MuID, name: MuName) -> MuEntityHeader {
        MuEntityHeader {
            id: id,
            name: RwLock::new(Some(MuEntityHeader::name_check(name)))
        }
    }
    
    pub fn id(&self) -> MuID {
        self.id
    }
    
    pub fn name(&self) -> Option<MuName> {
        self.name.read().unwrap().clone()
    }

    pub fn set_name(&self, name: MuName) {
        let mut name_guard = self.name.write().unwrap();
        *name_guard = Some(MuEntityHeader::name_check(name));
    }

    fn name_check(name: MuName) -> MuName {
        if name.starts_with("@") || name.starts_with("%") {
            let (_, name) = name.split_at(1);

            return name.to_string();
        }

        name
    }
}

impl PartialEq for MuEntityHeader {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl fmt::Display for MuEntityHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.name().is_none() {
            write!(f, "UNNAMED #{}", self.id)
        } else {
            write!(f, "{} #{}", self.name().unwrap(), self.id)
        }
    }
}

pub trait MuEntity {
    fn id(&self) -> MuID;
    fn name(&self) -> Option<MuName>;
    fn set_name(&self, name: MuName);
    fn as_entity(&self) -> &MuEntity;
}

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

    fn name(&self) -> Option<MuName> {
        match self.v {
            TreeNode_::Instruction(ref inst) => inst.name(),
            TreeNode_::Value(ref pv) => pv.name()
        }
    }

    fn set_name(&self, name: MuName) {
        match self.v {
            TreeNode_::Instruction(ref inst) => inst.set_name(name),
            TreeNode_::Value(ref pv) => pv.set_name(name)
        }
    }

    fn as_entity(&self) -> &MuEntity {
        match self.v {
            TreeNode_::Instruction(ref inst) => inst.as_entity(),
            TreeNode_::Value(ref pv) => pv.as_entity()
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
