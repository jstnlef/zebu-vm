#![allow(dead_code)]

use ast::ptr::*;
use ast::ir::*;
use ast::types::*;
use vm::VM;
use vm::bundle::*;

use std::mem;
use std::os::raw;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

/// This module implements muapi.h

macro_rules! unimplemented_api {
    () => {
        // cast fn to usize before transmute, see: https://github.com/rust-lang/rust/issues/19925
        unsafe {mem::transmute(unimplemented_api as usize)}
    }
}

macro_rules! api {
    ($func: expr) => {
        unsafe {mem::transmute($func as usize)}
    }
}

fn unimplemented_api() {
    unimplemented!()
}

#[repr(C)]
pub struct MuVM {
    // void* header
    internal: Arc<VM>,
    
    pub new_context: fn (mvm: *mut MuVM) -> *mut MuCtx,
    
    id_of: fn (mvm: *const MuVM, name: MuName) -> MuID,
    name_of : fn (mvm: *const MuVM, id: MuID) -> MuName,
    
    // set_trap_handler: fn(mvm: *mut MuVM, trap_handler: MuTrapHandler, user_data: MuCPtr)
    // make_boot_image: 
}

impl MuVM {
    pub fn new() -> *mut MuVM {
        let vm = Box::new(MuVM {
            internal: Arc::new(VM::new()),
            new_context: api!(MuVM::new_context),
            id_of: api!(MuVM::id_of),
            name_of: api!(MuVM::name_of)
        });
        
        Box::into_raw(vm)
    }
    
    pub fn new_context(&mut self) -> *mut MuCtx {
        let ctx = Box::new(MuCtx::new(self.internal.clone()));
        
        Box::into_raw(ctx)
    }
    
    pub fn id_of(&self, name: MuName) -> MuID {
        self.internal.get_id_of(name)
    }
    
    pub fn name_of(&self, id: MuID) -> MuName {
        self.internal.get_name_of(id)
    }
}

pub type MuArraySize = usize;

pub type MuIntValue = P<Value>;

pub type MuBundleNode = *mut MuBundle;
pub type MuChildNode  = MuIRNode;
pub type MuTypeNode   = P<MuType>;
pub type MuFuncSigNode= P<MuFuncSig>;
pub type MuConstNode  = P<Value>;
pub type MuGlobalNode = P<Value>;
pub type MuFuncNode   = RefCell<MuFunction>;
pub type MuFuncVerNode= RefCell<MuFunctionVersion>;
pub type MuBBNode     = P<Block>;
pub type MuNorParamNode = P<Value>;
pub type MuExcParamNode = P<Value>;
pub type MuInstNode     = P<TreeNode>;
pub type MuInstResNode  = P<TreeNode>;
pub type MuLocalVarNode = P<TreeNode>;
pub type MuVarNode      = MuIRNode;

pub type MuFlag          = usize;
pub type MuDestKind      = MuFlag;
pub type MuBinOptr       = MuFlag;
pub type MuCmpOptr       = MuFlag;
pub type MuConvOptr      = MuFlag;
pub type MuAtomicRMWOptr = MuFlag;
pub type MuMemOrd        = MuFlag;
pub type MuCallConv      = MuFlag;
pub type MuCommInst      = MuFlag;

type MuBool = raw::c_int;
type MuWPID = u32;

#[repr(C)]
pub struct MuCtx {
    // void* header - current not planed to use this
    internal: Box<MuCtxInternal>,
    
    id_of: fn (ctx: *const MuCtx, name: MuName) -> MuID,
    name_of : fn (ctx: *const MuCtx, id: MuID) -> MuName,
    
    close_context: fn (ctx: *mut MuCtx) -> (),
    
    // load bundle/hail of text form, should be deprecates soon
    load_bundle: fn (ctx: *mut MuCtx, buf: *const raw::c_char, sz: MuArraySize),
    load_hail  : fn (ctx: *mut MuCtx, buf: *const raw::c_char, sz: MuArraySize),
    
    handle_from_sint64: fn (ctx: *mut MuCtx, num: i64, len: raw::c_int) -> MuIntValue,
    handle_from_uint64: fn (ctx: *mut MuCtx, num: u64, len: raw::c_int) -> MuIntValue,
    // ... a lot more
    
    handle_to_sint64: fn (ctx: *mut MuCtx, opnd: MuIntValue) -> i64,
    handle_to_uint64: fn (ctx: *mut MuCtx, opnd: MuIntValue) -> u64,
    // ... a lot more
    
    // ignoring most of the runtime api for now
    // FIXME
    
    // IR Builder API
    
    new_bundle: fn (ctx: *mut MuCtx) -> MuBundleNode,
    
    load_bundle_from_node: fn (ctx: *mut MuCtx, b: MuBundleNode),
    abort_bundle_node    : fn (ctx: *mut MuCtx, b: MuBundleNode),
    
    get_node: fn (ctx: *mut MuCtx, b: MuBundleNode, id: MuID) -> MuChildNode,
    get_id  : fn (ctx: *mut MuCtx, b: MuBundleNode, node: MuChildNode) -> MuID,
    set_name: fn (ctx: *mut MuCtx, b: MuBundleNode, node: MuChildNode, name: MuName),
    
    // create types
    new_type_int   : fn (ctx: *mut MuCtx, b: MuBundleNode, len: raw::c_int) -> MuTypeNode,
    new_type_float : fn (ctx: *mut MuCtx, b: MuBundleNode) -> MuTypeNode,
    new_type_double: fn (ctx: *mut MuCtx, b: MuBundleNode) -> MuTypeNode,
    // ... a lot more 
    
    new_funcsig: fn (ctx: *mut MuCtx, b: MuBundleNode, 
        paramtys: *const MuTypeNode, nparamtys: MuArraySize,
        rettys: *const MuTypeNode, nrettys: MuArraySize) -> MuFuncSigNode,
    
    new_const_int   : fn (ctx: *mut MuCtx, b: MuBundleNode, ty: MuTypeNode, value: u64) -> MuConstNode,
    new_const_float : fn (ctx: *mut MuCtx, b: MuBundleNode, ty: MuTypeNode, value: f32) -> MuConstNode,
    new_const_double: fn (ctx: *mut MuCtx, b: MuBundleNode, ty: MuTypeNode, value: f64) -> MuConstNode,
    // ... a lot more
    
    new_global_cell : fn (ctx: *mut MuCtx, b: MuBundleNode, ty: MuTypeNode) -> MuGlobalNode,
    
    new_func: fn (ctx: *mut MuCtx, b: MuBundleNode, sig: MuFuncSigNode) -> MuFuncNode,
    new_func_ver: fn (ctx: *mut MuCtx, b: MuBundleNode, func: MuFuncNode) -> MuFuncVerNode,
    
    // create CFG
    new_bb: fn (ctx: *mut MuCtx, fv: MuFuncVerNode) -> MuBBNode,
    
    new_nor_param: fn (ctx: *mut MuCtx, bb: MuBBNode, ty: MuTypeNode) -> MuNorParamNode,
    new_exc_param: fn (ctx: *mut MuCtx, bb: MuBBNode) -> MuExcParamNode,
    
    get_inst_res    : fn (ctx: *mut MuCtx, inst: MuInstNode, index: raw::c_int) -> MuInstResNode,
    get_num_inst_res: fn (ctx: *mut MuCtx, inst: MuInstNode) -> raw::c_int, 
    
    add_dest: fn (ctx: *mut MuCtx, inst: MuInstNode, kind: MuDestKind, dest: MuBBNode, vars: *const MuVarNode, nvars: MuArraySize),
    add_keepalives: fn (ctx: *mut MuCtx, inst: MuInstNode, vars: *const MuLocalVarNode, nvars: MuArraySize),
    
    new_binop : fn (ctx: *mut MuCtx, bb: MuBBNode, optr: MuBinOptr,  ty: MuTypeNode, opnd1: MuVarNode, opnd2: MuVarNode) -> MuInstNode,
    new_cmp   : fn (ctx: *mut MuCtx, bb: MuBBNode, optr: MuCmpOptr,  ty: MuTypeNode, opnd1: MuVarNode, opnd2: MuVarNode) -> MuInstNode,
    new_conv  : fn (ctx: *mut MuCtx, bb: MuBBNode, optr: MuConvOptr, from_ty: MuTypeNode, to_ty: MuTypeNode, opnd: MuVarNode) -> MuInstNode,
    new_select: fn (ctx: *mut MuCtx, bb: MuBBNode, cond_ty: MuTypeNode, opnd_ty: MuTypeNode, cond: MuVarNode, if_true: MuVarNode, if_false: MuVarNode) -> MuInstNode,
    
    new_branch : fn (ctx: *mut MuCtx, bb: MuBBNode) -> MuInstNode,
    new_branch2: fn (ctx: *mut MuCtx, bb: MuBBNode, cond: MuVarNode) -> MuInstNode,
    new_switch : fn (ctx: *mut MuCtx, bb: MuBBNode, opnd_ty: MuTypeNode, opnd: MuVarNode) -> MuInstNode,
    add_switch_dest: fn (ctx: *mut MuCtx, sw: MuInstNode, key: MuConstNode, dest: MuBBNode, vars: *const MuVarNode, nvars: MuArraySize),
    
    new_call    : fn (ctx: *mut MuCtx, bb: MuBBNode, sig: MuFuncSigNode, callee: MuVarNode, args: *const MuVarNode, nargs: MuArraySize) -> MuInstNode,
    new_tailcall: fn (ctx: *mut MuCtx, bb: MuBBNode, sig: MuFuncSigNode, callee: MuVarNode, args: *const MuVarNode, nargs: MuArraySize) -> MuInstNode,
    new_ret     : fn (ctx: *mut MuCtx, bb: MuBBNode, rvs: *const MuVarNode, nrvs: MuArraySize) -> MuInstNode,
    new_throw   : fn (ctx: *mut MuCtx, bb: MuBBNode, exc: MuVarNode) -> MuInstNode,
    
    new_extractvalue  : fn (ctx: *mut MuCtx, bb: MuBBNode, strty: MuTypeNode, index: raw::c_int, opnd: MuVarNode) -> MuInstNode,
    new_insertvalue   : fn (ctx: *mut MuCtx, bb: MuBBNode, strty: MuTypeNode, index: raw::c_int, opnd: MuVarNode, newval: MuVarNode) -> MuInstNode,
    new_extractelement: fn (ctx: *mut MuCtx, bb: MuBBNode, seqty: MuTypeNode, indty: MuTypeNode, opnd: MuVarNode, index: MuVarNode) -> MuInstNode,
    new_insertelement : fn (ctx: *mut MuCtx, bb: MuBBNode, seqty: MuTypeNode, indty: MuTypeNode, opnd: MuVarNode, index: MuVarNode, newval: MuVarNode) -> MuInstNode,
    new_shufflevector : fn (ctx: *mut MuCtx, bb: MuBBNode, vecty: MuTypeNode, maskty: MuTypeNode, vec1: MuVarNode, vec2: MuVarNode, mask: MuVarNode) -> MuInstNode,
    
    new_new         : fn (ctx: *mut MuCtx, bb: MuBBNode, allocty: MuTypeNode) -> MuInstNode,
    new_newhybrid   : fn (ctx: *mut MuCtx, bb: MuBBNode, allocty: MuTypeNode, lenty: MuTypeNode, length: MuVarNode) -> MuInstNode,
    new_alloca      : fn (ctx: *mut MuCtx, bb: MuBBNode, allocty: MuTypeNode) -> MuInstNode,
    new_allocahybrid: fn (ctx: *mut MuCtx, bb: MuBBNode, allocty: MuTypeNode, lenty: MuTypeNode, length: MuVarNode) -> MuInstNode,
    
    new_getiref       : fn (ctx: *mut MuCtx, bb: MuBBNode, refty: MuTypeNode, opnd: MuVarNode) -> MuInstNode,
    new_getfieldiref  : fn (ctx: *mut MuCtx, bb: MuBBNode, is_ptr: MuBool, refty: MuTypeNode, index: raw::c_int, opnd: MuVarNode) -> MuInstNode,
    new_getelemiref   : fn (ctx: *mut MuCtx, bb: MuBBNode, is_ptr: MuBool, refty: MuTypeNode, indty: MuTypeNode, opnd: MuVarNode, index: MuVarNode) -> MuInstNode,
    new_shiftiref     : fn (ctx: *mut MuCtx, bb: MuBBNode, is_ptr: MuBool, refty: MuTypeNode, offty: MuTypeNode, opnd: MuVarNode, offset: MuVarNode) -> MuInstNode,
    new_getvarpartiref: fn (ctx: *mut MuCtx, bb: MuBBNode, is_ptr: MuBool, refty: MuTypeNode, opnd: MuTypeNode) -> MuInstNode,
    
    new_load     : fn (ctx: *mut MuCtx, bb: MuBBNode, is_ptr: MuBool, ord: MuMemOrd, refty: MuTypeNode, loc: MuVarNode) -> MuInstNode,
    new_store    : fn (ctx: *mut MuCtx, bb: MuBBNode, is_ptr: MuBool, ord: MuMemOrd, refty: MuTypeNode, loc: MuVarNode, newval: MuVarNode) -> MuInstNode,
    new_cmpxchg  : fn (ctx: *mut MuCtx, bb: MuBBNode, is_ptr: MuBool, is_weak: MuBool, ord_succ: MuMemOrd, ord_fail: MuMemOrd, refty: MuTypeNode, loc: MuVarNode, expected: MuVarNode, desired: MuVarNode) -> MuInstNode,
    new_atomicrmw: fn (ctx: *mut MuCtx, bb: MuBBNode, is_ptr: MuBool, ord: MuMemOrd, optr: MuAtomicRMWOptr, refty: MuTypeNode, loc: MuVarNode, opnd: MuVarNode) -> MuInstNode,
    new_fence    : fn (ctx: *mut MuCtx, bb: MuBBNode, ord: MuMemOrd) -> MuInstNode,
    
    new_trap      : fn (ctx: *mut MuCtx, bb: MuBBNode, rettys: *const MuTypeNode, nrettys: MuArraySize) -> MuInstNode,
    new_watchpoint: fn (ctx: *mut MuCtx, bb: MuBBNode, wpid: MuWPID, rettys: *const MuTypeNode, nrettys: MuArraySize) -> MuInstNode,
    new_wpbranch  : fn (ctx: *mut MuCtx, bb: MuBBNode, wpid: MuWPID) -> MuInstNode,
    
    new_ccall     : fn (ctx: *mut MuCtx, bb: MuBBNode, callconv: MuCallConv, callee_ty: MuTypeNode, sig: MuFuncSigNode, callee: MuVarNode, args: *const MuVarNode, nargs: MuArraySize) -> MuInstNode,
    
    new_thread        : fn (ctx: *mut MuCtx, bb: MuBBNode, stack: MuVarNode, threadlocal: MuVarNode) -> MuInstNode,
    new_swapstack_ret : fn (ctx: *mut MuCtx, bb: MuBBNode, swappee: MuVarNode, ret_tys: *const MuTypeNode, nret_tys: MuArraySize) -> MuInstNode,
    new_swapstack_kill: fn (ctx: *mut MuCtx, bb: MuBBNode, swappee: MuVarNode) -> MuInstNode,
    
    set_newstack_pass_values: fn (ctx: *mut MuCtx, inst: MuInstNode, tys: *const MuTypeNode, vars: *const MuVarNode, nvars: MuArraySize),
    set_newstack_throw_exc  : fn (ctx: *mut MuCtx, inst: MuInstNode, exc: MuVarNode),
    
    new_comminst: fn (ctx: *mut MuCtx, bb: MuBBNode, opcode: MuCommInst,
        flags: *const MuFlag,       nflags: MuArraySize,
        tys: *const MuTypeNode,     ntys: MuArraySize,
        sigs: *const MuFuncSigNode, nsigs: MuArraySize,
        args: *const MuVarNode,     nargs: MuArraySize) -> MuInstNode
}

struct MuCtxInternal {
    vm: Arc<VM>,
}

impl MuCtx {
    pub fn id_of(&self, name: MuName) -> MuID {
        self.internal.vm.get_id_of(name)
    }
    
    pub fn name_of(&self, id: MuID) -> MuName {
        self.internal.vm.get_name_of(id)
    }
    
    pub fn close_context(ctx: *mut MuCtx) {
        // Rust will reclaim the ctx
        unsafe {Box::from_raw(ctx)};
    }
    
    #[allow(unused_variables)]
    pub fn new_bundle(ctx: &mut MuCtx) -> MuBundleNode {
        let bundle = Box::new(MuBundle::new());
        
        Box::into_raw(bundle)
    }
    
    #[allow(unused_variables)]
    pub fn load_bundle_from_node(ctx: &mut MuCtx, b: MuBundleNode) {
        let bundle = unsafe{Box::from_raw(b)};
        
        // load it
        unimplemented!()
    }
    
    #[allow(unused_variables)]
    pub fn abort_bundle_node(ctx: &mut MuCtx, b: MuBundleNode) {
        let bundle = unsafe{Box::from_raw(b)};
    }
    
    pub fn get_node(ctx: &mut MuCtx, b: MuBundleNode, id: MuID) -> MuChildNode {
        let bundle = unsafe{b.as_mut()}.unwrap();
        
        if bundle.type_defs.contains_key(&id) {
            MuIRNode::new(id, MuIRNodeKind::Type)
        } else if bundle.func_sigs.contains_key(&id) {
            MuIRNode::new(id, MuIRNodeKind::FuncSig)
        } else if bundle.constants.contains_key(&id) {
            MuIRNode::new(id, MuIRNodeKind::Var(MuVarNodeKind::Global(MuGlobalVarNodeKind::Const)))
        } else if bundle.globals.contains_key(&id) {
            MuIRNode::new(id, MuIRNodeKind::Var(MuVarNodeKind::Global(MuGlobalVarNodeKind::Global)))
        } else if bundle.func_defs.contains_key(&id) {
            MuIRNode::new(id, MuIRNodeKind::Var(MuVarNodeKind::Global(MuGlobalVarNodeKind::Func)))
        } else if bundle.func_decls.contains_key(&id) {
            MuIRNode::new(id, MuIRNodeKind::FuncVer)
        } else {
            panic!("expecting ID of a top level definition")
        }
    }
    
    #[allow(unused_variables)]
    pub fn get_id(ctx: &mut MuCtx, b: MuBundleNode, node: MuChildNode) -> MuID {
        node.id
    }
    
    fn new(vm: Arc<VM>) -> MuCtx {
        MuCtx {
            internal: Box::new(MuCtxInternal::new(vm)),

            id_of: api!(MuCtx::id_of),
            name_of: api!(MuCtx::name_of),
            
            close_context: api!(MuCtx::close_context),
            
            // load bundle/hail of text form, should be deprecates soon
            load_bundle: unimplemented_api!(),
            load_hail  : unimplemented_api!(),
            
            handle_from_sint64: unimplemented_api!(),
            handle_from_uint64: unimplemented_api!(),
            // ... a lot more
            
            handle_to_sint64: unimplemented_api!(),
            handle_to_uint64: unimplemented_api!(),
            // ... a lot more
            
            // ignoring most of the runtime api for now
            // FIXME
            
            // IR Builder API
            
            new_bundle: api!(MuCtx::new_bundle),
            
            load_bundle_from_node: api!(MuCtx::load_bundle_from_node),
            abort_bundle_node    : api!(MuCtx::abort_bundle_node),
            
            get_node: api!(MuCtx::get_node),
            get_id  : api!(MuCtx::get_id),
            set_name: unimplemented_api!(),
            
            // create types
            new_type_int   : unimplemented_api!(),
            new_type_float : unimplemented_api!(),
            new_type_double: unimplemented_api!(),
            // ... a lot more 
            
            new_funcsig: unimplemented_api!(),
            
            new_const_int   : unimplemented_api!(),
            new_const_float : unimplemented_api!(),
            new_const_double: unimplemented_api!(),
            // ... a lot more
            
            new_global_cell : unimplemented_api!(),
            
            new_func: unimplemented_api!(),
            new_func_ver: unimplemented_api!(),
            
            // create CFG
            new_bb: unimplemented_api!(),
            
            new_nor_param: unimplemented_api!(),
            new_exc_param: unimplemented_api!(),
            
            get_inst_res    : unimplemented_api!(),
            get_num_inst_res: unimplemented_api!(), 
            
            add_dest: unimplemented_api!(),
            add_keepalives: unimplemented_api!(),
            
            new_binop : unimplemented_api!(),
            new_cmp   : unimplemented_api!(),
            new_conv  : unimplemented_api!(),
            new_select: unimplemented_api!(),
            
            new_branch : unimplemented_api!(),
            new_branch2: unimplemented_api!(),
            new_switch : unimplemented_api!(),
            add_switch_dest: unimplemented_api!(),
            
            new_call    : unimplemented_api!(),
            new_tailcall: unimplemented_api!(),
            new_ret     : unimplemented_api!(),
            new_throw   : unimplemented_api!(),
            
            new_extractvalue  : unimplemented_api!(),
            new_insertvalue   : unimplemented_api!(),
            new_extractelement: unimplemented_api!(),
            new_insertelement : unimplemented_api!(),
            new_shufflevector : unimplemented_api!(),
            
            new_new         : unimplemented_api!(),
            new_newhybrid   : unimplemented_api!(),
            new_alloca      : unimplemented_api!(),
            new_allocahybrid: unimplemented_api!(),
            
            new_getiref       : unimplemented_api!(),
            new_getfieldiref  : unimplemented_api!(),
            new_getelemiref   : unimplemented_api!(),
            new_shiftiref     : unimplemented_api!(),
            new_getvarpartiref: unimplemented_api!(),
            
            new_load     : unimplemented_api!(),
            new_store    : unimplemented_api!(),
            new_cmpxchg  : unimplemented_api!(),
            new_atomicrmw: unimplemented_api!(),
            new_fence    : unimplemented_api!(),
            
            new_trap      : unimplemented_api!(),
            new_watchpoint: unimplemented_api!(),
            new_wpbranch  : unimplemented_api!(),
            
            new_ccall     : unimplemented_api!(),
            
            new_thread        : unimplemented_api!(),
            new_swapstack_ret : unimplemented_api!(),
            new_swapstack_kill: unimplemented_api!(),
            
            set_newstack_pass_values: unimplemented_api!(),
            set_newstack_throw_exc  : unimplemented_api!(),
            
            new_comminst: unimplemented_api!(),       
        }
    } 
}

impl MuCtxInternal {
    fn new(vm: Arc<VM>) -> MuCtxInternal {
        MuCtxInternal {
            vm: vm
        }
    }
}