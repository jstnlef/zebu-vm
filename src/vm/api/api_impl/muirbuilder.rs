use super::common::*;
use ast::op::*;
use ast::inst::*;
use utils::LinkedHashMap;
use std;

pub struct MuIRBuilder {
    /// ref to MuVM
    mvm: *const MuVM,

    /// Point to the C-visible CMuIRBuilder so that `load` and `abort` can deallocate itself.
    pub c_struct: *mut CMuIRBuilder,

    /// Map IDs to names. Items are inserted during `gen_sym`. MuIRBuilder is supposed to be used
    /// by one thread, so there is no need for locking.
    id_name_map: HashMap<MuID, MuName>,

    /// The "trantient bundle" includes everything being built here.
    bundle: TrantientBundle,
}

pub type IdBMap<T> = HashMap<MuID, Box<T>>;

/// A trantient bundle, i.e. the bundle being built, but not yet loaded into the MuVM.
#[derive(Default)]
pub struct TrantientBundle {
    types: IdBMap<NodeType>,
    sigs: IdBMap<NodeFuncSig>,
    consts: IdBMap<NodeConst>,
    globals: IdBMap<NodeGlobalCell>,
    funcs: IdBMap<NodeFunc>,
    expfuncs: IdBMap<NodeExpFunc>,
    funcvers: IdBMap<NodeFuncVer>,
    bbs: IdBMap<NodeBB>,
    insts: IdBMap<NodeInst>,
    dest_clauses: IdBMap<NodeDestClause>,
    exc_clauses: IdBMap<NodeExcClause>,
    ka_clauses: IdBMap<NodeKeepaliveClause>,
}

impl MuIRBuilder {
    pub fn new(mvm: *const MuVM) -> Box<MuIRBuilder> {
        Box::new(MuIRBuilder {
            mvm: mvm,
            c_struct: ptr::null_mut(),
            id_name_map: Default::default(),
            bundle: Default::default(),
        })
    }

    #[inline(always)]
    fn get_mvm<'a, 'b>(&'a mut self) -> &'b MuVM {
        //self.mvm
        unsafe { & *self.mvm }
    }

    #[inline(always)]
    fn get_mvm_immutable<'a, 'b>(&'a self) -> &'b MuVM {
        unsafe { & *self.mvm }
    }

    #[inline(always)]
    fn get_vm<'a, 'b>(&'a mut self) -> &'b VM {
        &self.get_mvm().vm
    }

    #[inline(always)]
    fn next_id(&mut self) -> MuID {
        self.get_vm().next_id()
    }

    fn deallocate(&mut self) {
        let c_struct = self.c_struct;
        let b_ptr = self as *mut MuIRBuilder;
        debug!("Deallocating MuIRBuilder {:?} and CMuIRBuilder {:?}...", b_ptr, c_struct);
        unsafe {
            Box::from_raw(c_struct);
            Box::from_raw(b_ptr);
        }
    }

    /// Get the Mu name of the `id`. This will consume the entry in the `id_name_map`. For this
    /// reason, this function is only called when the actual MuEntity that has this ID is created
    /// (such as `new_type_int`).
    fn consume_name_of(&mut self, id: MuID) -> Option<MuName> {
        self.id_name_map.remove(&id)
    }

    pub fn load(&mut self) {
        load_bundle(self);
        self.deallocate();
    }

    pub fn abort(&mut self) {
        info!("Aborting boot image building...");
        self.deallocate();
    }

    pub fn gen_sym(&mut self, name: Option<String>) -> MuID {
        let my_id = self.next_id();

        debug!("gen_sym({:?}) -> {}", name, my_id);

        match name {
            None => {},
            Some(the_name) => {
                let old = self.id_name_map.insert(my_id, the_name);
                debug_assert!(old.is_none(), "ID already exists: {}, new name: {}, old name: {}",
                my_id, self.id_name_map.get(&my_id).unwrap(), old.unwrap());
            },
        };

        my_id
    }

    pub fn new_type_int(&mut self, id: MuID, len: c_int) {
        self.bundle.types.insert(id, Box::new(NodeType::TypeInt { id: id, len: len }));
    }

    pub fn new_type_float(&mut self, id: MuID) {
        self.bundle.types.insert(id, Box::new(NodeType::TypeFloat { id: id }));
    }

    pub fn new_type_double(&mut self, id: MuID) {
        self.bundle.types.insert(id, Box::new(NodeType::TypeDouble { id: id }));
    }

    pub fn new_type_uptr(&mut self, id: MuID, ty: MuID) {
        self.bundle.types.insert(id, Box::new(NodeType::TypeUPtr{ id: id,
            ty: ty }));
    }

    pub fn new_type_ufuncptr(&mut self, id: MuID, sig: MuID) {
        self.bundle.types.insert(id, Box::new(NodeType::TypeUFuncPtr{ id: id,
            sig: sig }));
    }

    pub fn new_type_struct(&mut self, id: MuID, fieldtys: Vec<MuID>) {
        self.bundle.types.insert(id, Box::new(NodeType::TypeStruct { id: id,
            fieldtys: fieldtys }));
    }

    pub fn new_type_hybrid(&mut self, id: MuID, fixedtys: Vec<MuID>, varty: MuID) {
        self.bundle.types.insert(id, Box::new(NodeType::TypeHybrid { id: id,
            fixedtys: fixedtys, varty: varty }));
    }

    pub fn new_type_array(&mut self, id: MuID, elemty: MuID, len: u64) {
        self.bundle.types.insert(id, Box::new(NodeType::TypeArray { id: id,
            elemty: elemty, len: len as usize }));
    }

    pub fn new_type_vector(&mut self, id: MuID, elemty: MuID, len: u64) {
        self.bundle.types.insert(id, Box::new(NodeType::TypeVector { id: id,
            elemty: elemty, len: len as usize }));
    }

    pub fn new_type_void(&mut self, id: MuID) {
        self.bundle.types.insert(id, Box::new(NodeType::TypeVoid { id: id }));
    }

    pub fn new_type_ref(&mut self, id: MuID, ty: MuID) {
        self.bundle.types.insert(id, Box::new(NodeType::TypeRef{ id: id,
            ty: ty }));
    }

    pub fn new_type_iref(&mut self, id: MuID, ty: MuID) {
        self.bundle.types.insert(id, Box::new(NodeType::TypeIRef{ id: id,
            ty: ty }));
    }

    pub fn new_type_weakref(&mut self, id: MuID, ty: MuID) {
        self.bundle.types.insert(id, Box::new(NodeType::TypeWeakRef{ id: id,
            ty: ty }));
    }

    pub fn new_type_funcref(&mut self, id: MuID, sig: MuID) {
        self.bundle.types.insert(id, Box::new(NodeType::TypeFuncRef{ id: id,
            sig: sig }));
    }

    pub fn new_type_tagref64(&mut self, id: MuID) {
        self.bundle.types.insert(id, Box::new(NodeType::TypeTagRef64 { id: id }));
    }

    pub fn new_type_threadref(&mut self, id: MuID) {
        self.bundle.types.insert(id, Box::new(NodeType::TypeThreadRef { id: id }));
    }

    pub fn new_type_stackref(&mut self, id: MuID) {
        self.bundle.types.insert(id, Box::new(NodeType::TypeStackRef { id: id }));
    }

    pub fn new_type_framecursorref(&mut self, id: MuID) {
        self.bundle.types.insert(id, Box::new(NodeType::TypeFrameCursorRef { id: id }));
    }

    pub fn new_type_irbuilderref(&mut self, id: MuID) {
        self.bundle.types.insert(id, Box::new(NodeType::TypeIRBuilderRef { id: id }));
    }

    pub fn new_funcsig(&mut self, id: MuID, paramtys: Vec<MuID>, rettys: Vec<MuID>) {
        self.bundle.sigs.insert(id, Box::new(NodeFuncSig { id: id,
            paramtys: paramtys, rettys: rettys }));
    }

    pub fn new_const_int(&mut self, id: MuID, ty: MuID, value: u64) {
        self.bundle.consts.insert(id, Box::new(NodeConst::ConstInt { id: id,
            ty: ty, value: value }));
    }

    pub fn new_const_int_ex(&mut self, id: MuID, ty: MuID, values: &[u64]) {
        panic!("Not implemented")
    }

    pub fn new_const_float(&mut self, id: MuID, ty: MuID, value: f32) {
        self.bundle.consts.insert(id, Box::new(NodeConst::ConstFloat { id: id,
            ty: ty, value: value }));
    }

    pub fn new_const_double(&mut self, id: MuID, ty: MuID, value: f64) {
        self.bundle.consts.insert(id, Box::new(NodeConst::ConstDouble { id: id,
            ty: ty, value: value }));
    }

    pub fn new_const_null(&mut self, id: MuID, ty: MuID) {
        self.bundle.consts.insert(id, Box::new(NodeConst::ConstNull { id: id,
            ty: ty }));
    }

    pub fn new_const_seq(&mut self, id: MuID, ty: MuID, elems: Vec<MuID>) {
        self.bundle.consts.insert(id, Box::new(NodeConst::ConstSeq { id: id,
            ty: ty, elems: elems }));
    }

    pub fn new_const_extern(&mut self, id: MuID, ty: MuID, symbol: String) {
        self.bundle.consts.insert(id, Box::new(NodeConst::ConstExtern { id: id,
            ty: ty, symbol: symbol }));
    }

    pub fn new_global_cell(&mut self, id: MuID, ty: MuID) {
        self.bundle.globals.insert(id, Box::new(NodeGlobalCell { id: id,
            ty: ty }));
    }

    pub fn new_func(&mut self, id: MuID, sig: MuID) {
        self.bundle.funcs.insert(id, Box::new(NodeFunc { id: id,
            sig: sig }));
    }

    pub fn new_exp_func(&mut self, id: MuID, func: MuID, callconv: CMuCallConv, cookie: MuID) {
        panic!("Not implemented")
    }

    pub fn new_func_ver(&mut self, id: MuID, func: MuID, bbs: Vec<MuID>) {
        self.bundle.funcvers.insert(id, Box::new(NodeFuncVer { id: id,
            func: func, bbs: bbs }));
    }

    pub fn new_bb(&mut self, id: MuID, nor_param_ids: Vec<MuID>, nor_param_types: Vec<MuID>, exc_param_id: Option<MuID>, insts: Vec<MuID>) {
        self.bundle.bbs.insert(id, Box::new(NodeBB { id: id,
            nor_param_ids: nor_param_ids, nor_param_types: nor_param_types,
            exc_param_id: exc_param_id, insts: insts }));
    }

    pub fn new_dest_clause(&mut self, id: MuID, dest: MuID, vars: Vec<MuID>) {
        self.bundle.dest_clauses.insert(id, Box::new(NodeDestClause { id: id,
            dest: dest, vars: vars }));
    }

    pub fn new_exc_clause(&mut self, id: MuID, nor: MuID, exc: MuID) {
        self.bundle.exc_clauses.insert(id, Box::new(NodeExcClause { id: id,
            nor: nor, exc: exc }));
    }

    pub fn new_keepalive_clause(&mut self, id: MuID, vars: Vec<MuID>) {
        self.bundle.ka_clauses.insert(id, Box::new(NodeKeepaliveClause { id: id,
            vars: vars }));
    }

    pub fn new_csc_ret_with(&mut self, id: MuID, rettys: Vec<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_csc_kill_old(&mut self, id: MuID) {
        panic!("Not implemented")
    }

    pub fn new_nsc_pass_values(&mut self, id: MuID, tys: Vec<MuID>, vars: Vec<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_nsc_throw_exc(&mut self, id: MuID, exc: MuID) {
        panic!("Not implemented")
    }

    #[inline(always)]
    fn add_inst(&mut self, id: MuID, inst: NodeInst) {
        self.bundle.insts.insert(id, Box::new(inst));
    }

    pub fn new_binop(&mut self, id: MuID, result_id: MuID, optr: CMuBinOptr, ty: MuID, opnd1: MuID, opnd2: MuID, exc_clause: Option<MuID>) {
        self.add_inst(id, NodeInst::NodeBinOp {
            id: id, result_id: result_id, status_result_ids: vec![],
            optr: optr, flags: 0, ty: ty, opnd1: opnd1, opnd2: opnd2,
            exc_clause: exc_clause
        });
    }

    pub fn new_binop_with_status(&mut self, id: MuID, result_id: MuID, status_result_ids: Vec<MuID>, optr: CMuBinOptr, status_flags: CMuBinOpStatus, ty: MuID, opnd1: MuID, opnd2: MuID, exc_clause: Option<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_cmp(&mut self, id: MuID, result_id: MuID, optr: CMuCmpOptr, ty: MuID, opnd1: MuID, opnd2: MuID) {
        self.add_inst(id, NodeInst::NodeCmp {
            id: id, result_id: result_id,
            optr: optr, ty: ty, opnd1: opnd1, opnd2: opnd2
        });
    }

    pub fn new_conv(&mut self, id: MuID, result_id: MuID, optr: CMuConvOptr, from_ty: MuID, to_ty: MuID, opnd: MuID) {
        self.add_inst(id, NodeInst::NodeConv {
            id: id, result_id: result_id,
            optr: optr, from_ty: from_ty, to_ty: to_ty, opnd: opnd
        });
    }

    pub fn new_select(&mut self, id: MuID, result_id: MuID, cond_ty: MuID, opnd_ty: MuID, cond: MuID, if_true: MuID, if_false: MuID) {
        self.add_inst(id, NodeInst::NodeSelect {
            id: id, result_id: result_id,
            cond_ty: cond_ty, opnd_ty: opnd_ty, cond: cond,
            if_true: if_true, if_false: if_false
        });
    }

    pub fn new_branch(&mut self, id: MuID, dest: MuID) {
        self.add_inst(id, NodeInst::NodeBranch {
            id: id, dest: dest
        });
    }

    pub fn new_branch2(&mut self, id: MuID, cond: MuID, if_true: MuID, if_false: MuID) {
        self.add_inst(id, NodeInst::NodeBranch2 {
            id: id, cond: cond, if_true: if_true, if_false: if_false
        });
    }

    pub fn new_switch(&mut self, id: MuID, opnd_ty: MuID, opnd: MuID, default_dest: MuID, cases: Vec<MuID>, dests: Vec<MuID>) {
        self.add_inst(id, NodeInst::NodeSwitch {
            id: id, opnd_ty: opnd_ty, opnd: opnd, default_dest: default_dest,
            cases: cases, dests: dests
        });
    }

    pub fn new_call(&mut self, id: MuID, result_ids: Vec<MuID>, sig: MuID, callee: MuID, args: Vec<MuID>, exc_clause: Option<MuID>, keepalive_clause: Option<MuID>) {
        self.add_inst(id, NodeInst::NodeCall {
            id: id, result_ids: result_ids,
            sig: sig, callee: callee, args: args,
            exc_clause: exc_clause, keepalive_clause: keepalive_clause
        });
    }

    pub fn new_tailcall(&mut self, id: MuID, sig: MuID, callee: MuID, args: Vec<MuID>) {
        self.add_inst(id, NodeInst::NodeTailCall {
            id: id, sig: sig, callee: callee, args: args
        });
    }

    pub fn new_ret(&mut self, id: MuID, rvs: Vec<MuID>) {
        self.add_inst(id, NodeInst::NodeRet {
            id: id, rvs: rvs
        });
    }

    pub fn new_throw(&mut self, id: MuID, exc: MuID) {
        self.add_inst(id, NodeInst::NodeThrow {
            id: id, exc: exc
        });
    }

    pub fn new_extractvalue(&mut self, id: MuID, result_id: MuID, strty: MuID, index: c_int, opnd: MuID) {
        self.add_inst(id, NodeInst::NodeExtractValue {
            id: id, result_id: result_id, strty: strty, index: index, opnd: opnd
        });
    }

    pub fn new_insertvalue(&mut self, id: MuID, result_id: MuID, strty: MuID, index: c_int, opnd: MuID, newval: MuID) {
        self.add_inst(id, NodeInst::NodeInsertValue {
            id: id, result_id: result_id, strty: strty, index: index, opnd: opnd, newval: newval
        });
    }

    pub fn new_extractelement(&mut self, id: MuID, result_id: MuID, seqty: MuID, indty: MuID, opnd: MuID, index: MuID) {
        self.add_inst(id, NodeInst::NodeExtractElement {
            id: id, result_id: result_id, seqty: seqty, indty: indty, opnd: opnd, index: index
        });
    }

    pub fn new_insertelement(&mut self, id: MuID, result_id: MuID, seqty: MuID, indty: MuID, opnd: MuID, index: MuID, newval: MuID) {
        self.add_inst(id, NodeInst::NodeInsertElement {
            id: id, result_id: result_id, seqty: seqty, indty: indty, opnd: opnd, index: index, newval: newval
        });
    }

    pub fn new_shufflevector(&mut self, id: MuID, result_id: MuID, vecty: MuID, maskty: MuID, vec1: MuID, vec2: MuID, mask: MuID) {
        self.add_inst(id, NodeInst::NodeShuffleVector {
            id: id, result_id: result_id, vecty: vecty, maskty: maskty, vec1: vec1, vec2: vec2, mask: mask
        });
    }

    pub fn new_new(&mut self, id: MuID, result_id: MuID, allocty: MuID, exc_clause: Option<MuID>) {
        self.add_inst(id, NodeInst::NodeNew {
            id: id, result_id: result_id, allocty: allocty, exc_clause: exc_clause
        });
    }

    pub fn new_newhybrid(&mut self, id: MuID, result_id: MuID, allocty: MuID, lenty: MuID, length: MuID, exc_clause: Option<MuID>) {
        self.add_inst(id, NodeInst::NodeNewHybrid {
            id: id, result_id: result_id, allocty: allocty, lenty: lenty, length: length, exc_clause: exc_clause
        });
    }

    pub fn new_alloca(&mut self, id: MuID, result_id: MuID, allocty: MuID, exc_clause: Option<MuID>) {
        self.add_inst(id, NodeInst::NodeAlloca {
            id: id, result_id: result_id, allocty: allocty, exc_clause: exc_clause
        });
    }

    pub fn new_allocahybrid(&mut self, id: MuID, result_id: MuID, allocty: MuID, lenty: MuID, length: MuID, exc_clause: Option<MuID>) {
        self.add_inst(id, NodeInst::NodeAllocaHybrid {
            id: id, result_id: result_id, allocty: allocty, lenty: lenty, length: length, exc_clause: exc_clause
        });
    }

    pub fn new_getiref(&mut self, id: MuID, result_id: MuID, refty: MuID, opnd: MuID) {
        self.add_inst(id, NodeInst::NodeGetIRef {
            id: id, result_id: result_id, refty: refty, opnd: opnd
        });
    }

    pub fn new_getfieldiref(&mut self, id: MuID, result_id: MuID, is_ptr: bool, refty: MuID, index: c_int, opnd: MuID) {
        self.add_inst(id, NodeInst::NodeGetFieldIRef {
            id: id, result_id: result_id, is_ptr: is_ptr, refty: refty, index: index, opnd: opnd
        });
    }

    pub fn new_getelemiref(&mut self, id: MuID, result_id: MuID, is_ptr: bool, refty: MuID, indty: MuID, opnd: MuID, index: MuID) {
        self.add_inst(id, NodeInst::NodeGetElemIRef {
            id: id, result_id: result_id, is_ptr: is_ptr, refty: refty, indty: indty, opnd: opnd, index: index
        });
    }

    pub fn new_shiftiref(&mut self, id: MuID, result_id: MuID, is_ptr: bool, refty: MuID, offty: MuID, opnd: MuID, offset: MuID) {
        self.add_inst(id, NodeInst::NodeShiftIRef {
            id: id, result_id: result_id, is_ptr: is_ptr, refty: refty, offty: offty, opnd: opnd, offset: offset
        });
    }

    pub fn new_getvarpartiref(&mut self, id: MuID, result_id: MuID, is_ptr: bool, refty: MuID, opnd: MuID) {
        self.add_inst(id, NodeInst::NodeGetVarPartIRef {
            id: id, result_id: result_id, is_ptr: is_ptr, refty: refty, opnd: opnd
        });
    }

    pub fn new_load(&mut self, id: MuID, result_id: MuID, is_ptr: bool, ord: CMuMemOrd, refty: MuID, loc: MuID, exc_clause: Option<MuID>) {
        self.add_inst(id, NodeInst::NodeLoad {
            id: id, result_id: result_id, is_ptr: is_ptr, ord: ord, refty: refty, loc: loc, exc_clause: exc_clause
        });
    }

    pub fn new_store(&mut self, id: MuID, is_ptr: bool, ord: CMuMemOrd, refty: MuID, loc: MuID, newval: MuID, exc_clause: Option<MuID>) {
        self.add_inst(id, NodeInst::NodeStore {
            id: id, is_ptr: is_ptr, ord: ord, refty: refty, loc: loc, newval: newval, exc_clause: exc_clause
        });
    }

    pub fn new_cmpxchg(&mut self, id: MuID, value_result_id: MuID, succ_result_id: MuID, is_ptr: bool, is_weak: bool, ord_succ: CMuMemOrd, ord_fail: CMuMemOrd, refty: MuID, loc: MuID, expected: MuID, desired: MuID, exc_clause: Option<MuID>) {
        self.add_inst(id, NodeInst::NodeCmpXchg {
            id: id, value_result_id: value_result_id, succ_result_id: succ_result_id, is_ptr: is_ptr, is_weak: is_weak, ord_succ: ord_succ, ord_fail: ord_fail, refty: refty, loc: loc, expected: expected, desired: desired, exc_clause: exc_clause
        });
    }

    pub fn new_atomicrmw(&mut self, id: MuID, result_id: MuID, is_ptr: bool, ord: CMuMemOrd, optr: CMuAtomicRMWOptr, ref_ty: MuID, loc: MuID, opnd: MuID, exc_clause: Option<MuID>) {
        self.add_inst(id, NodeInst::NodeAtomicRMW {
            id: id, result_id: result_id, is_ptr: is_ptr, ord: ord, optr: optr, ref_ty: ref_ty, loc: loc, opnd: opnd, exc_clause: exc_clause
        });
    }

    pub fn new_fence(&mut self, id: MuID, ord: CMuMemOrd) {
        self.add_inst(id, NodeInst::NodeFence {
            id: id, ord: ord,
        });
    }

    pub fn new_trap(&mut self, id: MuID, result_ids: Vec<MuID>, rettys: Vec<MuID>, exc_clause: Option<MuID>, keepalive_clause: Option<MuID>) {
        self.add_inst(id, NodeInst::NodeTrap {
            id: id, result_ids: result_ids, rettys: rettys, exc_clause: exc_clause, keepalive_clause: keepalive_clause
        });
    }

    pub fn new_watchpoint(&mut self, id: MuID, wpid: CMuWPID, result_ids: Vec<MuID>, rettys: Vec<MuID>, dis: MuID, ena: MuID, exc: Option<MuID>, keepalive_clause: Option<MuID>) {
        self.add_inst(id, NodeInst::NodeWatchPoint {
            id: id, wpid: wpid as MuID, result_ids: result_ids, rettys: rettys, dis: dis, ena: ena, exc: exc, keepalive_clause: keepalive_clause
        });
    }

    pub fn new_wpbranch(&mut self, id: MuID, wpid: CMuWPID, dis: MuID, ena: MuID) {
        self.add_inst(id, NodeInst::NodeWPBranch {
            id: id, wpid: wpid as MuID, dis: dis, ena: ena
        });
    }

    pub fn new_ccall(&mut self, id: MuID, result_ids: Vec<MuID>, callconv: CMuCallConv, callee_ty: MuID, sig: MuID, callee: MuID, args: Vec<MuID>, exc_clause: Option<MuID>, keepalive_clause: Option<MuID>) {
        self.add_inst(id, NodeInst::NodeCCall {
            id: id, result_ids: result_ids, callconv: callconv, callee_ty: callee_ty, sig: sig, callee: callee, args: args, exc_clause: exc_clause, keepalive_clause: keepalive_clause
        });
    }

    pub fn new_newthread(&mut self, id: MuID, result_id: MuID, stack: MuID, threadlocal: Option<MuID>, new_stack_clause: MuID, exc_clause: Option<MuID>) {
        self.add_inst(id, NodeInst::NodeNewThread {
            id: id, result_id: result_id, stack: stack, threadlocal: threadlocal, new_stack_clause: new_stack_clause, exc_clause: exc_clause
        });
    }

    pub fn new_swapstack(&mut self, id: MuID, result_ids: Vec<MuID>, swappee: MuID, cur_stack_clause: MuID, new_stack_clause: MuID, exc_clause: Option<MuID>, keepalive_clause: Option<MuID>) {
        self.add_inst(id, NodeInst::NodeSwapStack {
            id: id, result_ids: result_ids, swappee: swappee, cur_stack_clause: cur_stack_clause, new_stack_clause: new_stack_clause, exc_clause: exc_clause, keepalive_clause: keepalive_clause
        });
    }

    pub fn new_comminst(&mut self, id: MuID, result_ids: Vec<MuID>, opcode: CMuCommInst, flags: &[CMuFlag], tys: Vec<MuID>, sigs: Vec<MuID>, args: Vec<MuID>, exc_clause: Option<MuID>, keepalive_clause: Option<MuID>) {
        self.add_inst(id, NodeInst::NodeCommInst {
            id: id, result_ids: result_ids,
            opcode: opcode, flags: vec![], tys: tys, sigs: sigs, args: args,
            exc_clause: exc_clause, keepalive_clause: keepalive_clause
        });
    }
}

type IdPMap<T> = HashMap<MuID, P<T>>;

struct BundleLoader<'lb, 'lvm> {
    b: &'lb MuIRBuilder,
    vm: &'lvm VM,
    id_name_map: HashMap<MuID, MuName>,
    visited: HashSet<MuID>,
    built_types: IdPMap<MuType>,
    built_sigs: IdPMap<MuFuncSig>,
    built_constants: IdPMap<Value>,
    built_globals: IdPMap<Value>,
    built_funcs: IdBMap<MuFunction>,
    built_funcvers: IdBMap<MuFunctionVersion>,
    struct_hybrid_id_tags: Vec<(MuID, MuName)>,

    built_void: Option<P<MuType>>,
    built_refvoid: Option<P<MuType>>,
    built_refi64: Option<P<MuType>>,
    built_i1: Option<P<MuType>>,
    built_i64: Option<P<MuType>>,

    built_funcref_of: IdPMap<MuType>,
    built_ref_of: IdPMap<MuType>,
    built_iref_of: IdPMap<MuType>,
    built_uptr_of: IdPMap<MuType>,

    built_constint_of: HashMap<u64, P<Value>>,
}

fn load_bundle(b: &mut MuIRBuilder) {
    let vm = b.get_vm();

    let new_map = b.id_name_map.drain().collect::<HashMap<_,_>>();

    let mut bl = BundleLoader {
        b: b,
        vm: vm,
        id_name_map: new_map,
        visited: Default::default(),
        built_types: Default::default(),
        built_sigs: Default::default(),
        built_constants: Default::default(),
        built_globals: Default::default(),
        built_funcs: Default::default(),
        built_funcvers: Default::default(),
        struct_hybrid_id_tags: Default::default(),
        built_void: Default::default(),
        built_refvoid: Default::default(),
        built_refi64: Default::default(),
        built_i1: Default::default(),
        built_i64: Default::default(),
        built_funcref_of: Default::default(),
        built_ref_of: Default::default(),
        built_iref_of: Default::default(),
        built_uptr_of: Default::default(),
        built_constint_of: Default::default(),
    };

    bl.load_bundle();
}

#[derive(Default)]
struct FuncCtxBuilder {
    ctx: FunctionContext,
    tree_nodes: IdPMap<TreeNode>,
}

const DEFAULT_TRUE_PROB: f32 = 0.6f32;

impl<'lb, 'lvm> BundleLoader<'lb, 'lvm> {
    fn load_bundle(&mut self) {
        self.ensure_names();
        self.build_toplevels();
        self.add_everything_to_vm();
    }

    fn ensure_void(&mut self) -> P<MuType> {
        if let Some(ref void) = self.built_void {
            return void.clone();
        }

        let id_void = self.vm.next_id();

        let impl_void = P(MuType {
            hdr: MuEntityHeader::unnamed(id_void),
            v: MuType_::Void
        });

        trace!("Ensure void is defined: {} {:?}", id_void, impl_void);

        self.built_types.insert(id_void, impl_void.clone());
        self.built_void = Some(impl_void.clone());

        impl_void
    }

    fn ensure_refvoid(&mut self) -> P<MuType> {
        if let Some(ref refvoid) = self.built_refi64 {
            return refvoid.clone();
        }

        let id_refvoid = self.vm.next_id();

        let id_void = self.ensure_void().id();
        let impl_refvoid = self.ensure_ref(id_void);

        trace!("Ensure refvoid is defined: {} {:?}", id_refvoid, impl_refvoid);

        self.built_types.insert(id_refvoid, impl_refvoid.clone());
        self.built_refvoid = Some(impl_refvoid.clone());

        impl_refvoid
    }

    fn ensure_refi64(&mut self) -> P<MuType> {
        if let Some(ref refi64) = self.built_refi64 {
            return refi64.clone();
        }

        let id_i64 = self.vm.next_id();
        let id_ref = self.vm.next_id();

        let impl_i64 = P(MuType {
            hdr: MuEntityHeader::unnamed(id_i64),
            v: MuType_::Int(64),
        });

        let impl_ref = P(MuType {
            hdr: MuEntityHeader::unnamed(id_ref),
            v: MuType_::Ref(impl_i64.clone()),
        });

        trace!("Ensure i64 is defined: {} {:?}", id_i64, impl_i64);
        trace!("Ensure ref is defined: {} {:?}", id_ref, impl_ref);

        self.built_types.insert(id_i64, impl_i64);
        self.built_types.insert(id_ref, impl_ref.clone());

        self.built_refi64 = Some(impl_ref.clone());

        impl_ref
    }

    fn ensure_i1(&mut self) -> P<MuType> {
        if let Some(ref impl_ty) = self.built_i1 {
            return impl_ty.clone();
        }

        let id = self.vm.next_id();

        let impl_ty = P(MuType {
            hdr: MuEntityHeader::unnamed(id),
            v: MuType_::Int(1),
        });

        trace!("Ensure i1 is defined: {} {:?}", id, impl_ty);

        self.built_types.insert(id, impl_ty.clone());
        self.built_i1 = Some(impl_ty.clone());

        impl_ty
    }

    fn ensure_i64(&mut self) -> P<MuType> {
        if let Some(ref impl_ty) = self.built_i64 {
            return impl_ty.clone();
        }

        let id = self.vm.next_id();

        let impl_ty = P(MuType {
            hdr: MuEntityHeader::unnamed(id),
            v: MuType_::Int(64),
        });

        trace!("Ensure i64 is defined: {} {:?}", id, impl_ty);

        self.built_types.insert(id, impl_ty.clone());
        self.built_i64 = Some(impl_ty.clone());

        impl_ty
    }

    fn ensure_constint_of(&mut self, value: u64) -> P<TreeNode> {
        if let Some(c) = self.built_constint_of.get(&value) {
            return self.new_global(c.clone());
        }

        let id = self.vm.next_id();

        let impl_ty = self.ensure_i64();

        let impl_val = P(Value {
            hdr: MuEntityHeader::unnamed(id),
            ty: impl_ty,
            v: Value_::Constant(Constant::Int(value)),
        });

        trace!("Ensure const int is defined: {} {:?}", value, impl_val);

        self.built_constants.insert(id, impl_val.clone());
        self.built_constint_of.insert(value, impl_val.clone());

        self.new_global(impl_val)
    }
    
    fn ensure_funcref(&mut self, sig_id: MuID) -> P<MuType> {
        if let Some(funcref) = self.built_funcref_of.get(&sig_id) {
            return funcref.clone();
        }

        let sig = self.built_sigs.get(&sig_id).unwrap().clone();

        let id_funcref = self.vm.next_id();

        let impl_funcref = P(MuType {
            hdr: MuEntityHeader::unnamed(id_funcref),
            v: MuType_::FuncRef(sig),
        });

        trace!("Ensure funcref of {} is defined: {} {:?}", sig_id, id_funcref, impl_funcref);

        self.built_types.insert(id_funcref, impl_funcref.clone());
        self.built_funcref_of.insert(sig_id, impl_funcref.clone());

        impl_funcref
    }

    fn ensure_type_generic<F>(id: MuID, hint: &str,
                         vm: &VM, cache_map: &mut IdPMap<MuType>, storage_map: &mut IdPMap<MuType>,
                         factory: F) -> P<MuType> where F: Fn(P<MuType>) -> MuType_ {
        if let Some(obj) = cache_map.get(&id) {
            return obj.clone();
        }

        let new_id = vm.next_id();

        let old_obj = storage_map.get(&id).unwrap().clone();

        let impl_type_ = factory(old_obj);

        let new_obj = P(MuType {
            hdr: MuEntityHeader::unnamed(new_id),
            v: impl_type_,
        });

        storage_map.insert(new_id, new_obj.clone());

        trace!("Ensure {} of {} is defined: {:?}", hint, id, new_obj);

        cache_map.insert(new_id, new_obj.clone());

        new_obj
    }

    fn ensure_ref(&mut self, ty_id: MuID) -> P<MuType> {
        BundleLoader::ensure_type_generic(ty_id, "ref", &self.vm, &mut self.built_ref_of, &mut self.built_types, |impl_ty| {
            MuType_::Ref(impl_ty)
        })
    }

    fn ensure_iref(&mut self, ty_id: MuID) -> P<MuType> {
        BundleLoader::ensure_type_generic(ty_id, "iref", &self.vm, &mut self.built_iref_of, &mut self.built_types, |impl_ty| {
            MuType_::IRef(impl_ty)
        })
    }

    fn ensure_uptr(&mut self, ty_id: MuID) -> P<MuType> {
        BundleLoader::ensure_type_generic(ty_id, "uptr", &self.vm, &mut self.built_iref_of, &mut self.built_types, |impl_ty| {
            MuType_::UPtr(impl_ty)
        })
    }

    fn ensure_iref_or_uptr(&mut self, ty_id: MuID, is_ptr: bool) -> P<MuType> {
        if is_ptr {
            self.ensure_uptr(ty_id)
        } else {
            self.ensure_iref(ty_id)
        }
    }

    fn name_from_id(id: MuID, hint: &str) -> String {
        format!("@uvm.unnamed.{}{}", hint, id)
    }

    fn ensure_name(&mut self, id: MuID, hint: &str) {
        self.id_name_map.entry(id).or_insert_with(|| {
            let name = BundleLoader::name_from_id(id, hint);
            trace!("Making name for ID {} : {}", id, name);
            name
        });
    }

    fn ensure_names(&mut self) {
        // Make sure structs and hybrids have names because names are used to resolve cyclic
        // dependencies.
        for (id, ty) in &self.b.bundle.types {
            match **ty {
                NodeType::TypeStruct { id: _, fieldtys: _ } => { 
                    self.ensure_name(*id, "struct");
                },
                NodeType::TypeHybrid { id: _, fixedtys: _, varty: _ } => { 
                    self.ensure_name(*id, "struct");
                },
                _ => {}
            }
        }

        for id in self.b.bundle.funcvers.keys() {
            self.ensure_name(*id, "funcver");
        }

        for id in self.b.bundle.bbs.keys() {
            self.ensure_name(*id, "funcver");
        }
    }

    fn get_name(&self, id: MuID) -> String {
        self.id_name_map.get(&id).unwrap().clone()
    }

    fn maybe_get_name(&self, id: MuID) -> Option<String> {
        self.id_name_map.get(&id).cloned()
    }

    fn make_mu_entity_header(&self, id: MuID) -> MuEntityHeader {
        match self.maybe_get_name(id) {
            None => MuEntityHeader::unnamed(id),
            Some(name) => MuEntityHeader::named(id, name),
        }
    }

    fn build_toplevels(&mut self) {
        for id in self.b.bundle.types.keys() {
            if !self.visited.contains(id) {
                self.build_type(*id)
            }
        }

        let struct_hybrid_id_tags = self.struct_hybrid_id_tags.drain(..).collect::<Vec<_>>();
        for (id, ref tag) in struct_hybrid_id_tags {
            self.fill_struct_hybrid(id, tag)
        }

        for id in self.b.bundle.sigs.keys() {
            if !self.visited.contains(id) {
                self.build_sig(*id)
            }
        }

        for id in self.b.bundle.consts.keys() {
            if !self.visited.contains(id) {
                self.build_const(*id)
            }
        }

        for id in self.b.bundle.globals.keys() {
            if !self.visited.contains(id) {
                self.build_global(*id)
            }
        }

        for id in self.b.bundle.funcs.keys() {
            if !self.visited.contains(id) {
                self.build_func(*id)
            }
        }

        for id in self.b.bundle.funcvers.keys() {
            self.build_funcver(*id)
        }
    }

    fn build_type(&mut self, id: MuID) {
        self.visited.insert(id);

        let ty = self.b.bundle.types.get(&id).unwrap();

        trace!("Building type {} {:?}", id, ty);

        let hdr = self.make_mu_entity_header(id);

        let impl_ty_ = match **ty {
            NodeType::TypeInt { id: _, len } => {
                MuType_::Int(len as usize)
            },
            NodeType::TypeFloat { id: _ } => {
                MuType_::Float
            },
            NodeType::TypeDouble { id: _ } => {
                MuType_::Double
            },
            NodeType::TypeUPtr { id: _, ty: toty } => {
                let impl_toty = self.ensure_type_rec(toty);
                MuType_::UPtr(impl_toty)
            },
            NodeType::TypeUFuncPtr { id: _, sig } => {
                let impl_sig = self.ensure_sig_rec(sig);
                MuType_::UFuncPtr(impl_sig)
            },
            NodeType::TypeStruct { id: _, fieldtys: _ } => { 
                let tag = self.get_name(id);
                self.struct_hybrid_id_tags.push((id, tag.clone()));
                MuType_::mustruct_empty(tag)
                // MuType_::Struct(tag)
            },
            NodeType::TypeHybrid { id: _, fixedtys: _, varty: _ } => {
                let tag = self.get_name(id);
                self.struct_hybrid_id_tags.push((id, tag.clone()));
                MuType_::hybrid_empty(tag)
//                let impl_fixedtys = fixedtys.iter().map(|t| self.ensure_type_rec(*t)).collect::<Vec<_>>();
//                let impl_varty = self.ensure_type_rec(varty);
//                MuType_::Hybrid(impl_fixedtys, impl_varty)
            },
            NodeType::TypeArray { id: _, elemty, len } => { 
                let impl_elemty = self.ensure_type_rec(elemty);
                MuType_::Array(impl_elemty, len)
            },
            NodeType::TypeVector { id: _, elemty, len } => { 
                let impl_elemty = self.ensure_type_rec(elemty);
                MuType_::Vector(impl_elemty, len)
            },
            NodeType::TypeVoid { id: _ } => {
                MuType_::Void
            },
            NodeType::TypeTagRef64 { id: _ } => {
                MuType_::Tagref64
            },
            NodeType::TypeRef { id: _, ty: toty } => {
                let impl_toty = self.ensure_type_rec(toty);
                MuType_::Ref(impl_toty)
            },
            NodeType::TypeIRef { id: _, ty: toty } => {
                let impl_toty = self.ensure_type_rec(toty);
                MuType_::IRef(impl_toty)
            },
            NodeType::TypeWeakRef { id: _, ty: toty } => {
                let impl_toty = self.ensure_type_rec(toty);
                MuType_::WeakRef(impl_toty)
            },
            NodeType::TypeFuncRef { id: _, sig } => {
                let impl_sig = self.ensure_sig_rec(sig);
                MuType_::FuncRef(impl_sig)
            },
            NodeType::TypeThreadRef { id: _ } => {
                MuType_::ThreadRef
            },
            NodeType::TypeStackRef { id: _ } => {
                MuType_::StackRef
            },
            ref t => panic!("{:?} not implemented", t),
        };

        let impl_ty = MuType { hdr: hdr, v: impl_ty_ };

        trace!("Type built: {} {:?}", id, impl_ty);

        self.built_types.insert(id, P(impl_ty));
    }

    fn ensure_type_rec(&mut self, id: MuID) -> P<MuType> {
        if self.b.bundle.types.contains_key(&id) {
            if self.visited.contains(&id) {
                match self.built_types.get(&id) {
                    Some(t) => t.clone(),
                    None => panic!("Cyclic types found. id: {}", id)
                }
            } else {
                self.build_type(id);
                self.built_types.get(&id).unwrap().clone()
            }
        } else {
            self.vm.get_type(id)
        }
    }

    fn get_built_type(&self, id: MuID) -> P<MuType> {
        match self.built_types.get(&id) {
            Some(t) => t.clone(),
            None => self.vm.get_type(id)
        }
    }

    fn fill_struct_hybrid(&mut self, id: MuID, tag: &MuName) {
        let ty = self.b.bundle.types.get(&id).unwrap();

        trace!("Filling struct or hybrid {} {:?}", id, ty);

        match **ty {
            NodeType::TypeStruct { id: _, ref fieldtys } => { 
                let fieldtys_impl = fieldtys.iter().map(|fid| {
                    self.ensure_type_rec(*fid)
                }).collect::<Vec<_>>();

                MuType_::mustruct_put(tag, fieldtys_impl);

                trace!("Struct {} filled: {:?}", id,
                       STRUCT_TAG_MAP.read().unwrap().get(tag));
            },
            NodeType::TypeHybrid { id: _, ref fixedtys, varty } => { 
                let fixedtys_impl = fixedtys.iter().map(|fid| {
                    self.ensure_type_rec(*fid)
                }).collect::<Vec<_>>();

                let varty_impl = self.ensure_type_rec(varty);

                MuType_::hybrid_put(tag, fixedtys_impl, varty_impl);

                trace!("Hybrid {} filled: {:?}", id,
                       HYBRID_TAG_MAP.read().unwrap().get(tag));
            },
            ref t => panic!("{} {:?} should be a Struct or Hybrid type", id, ty),
        }
    }

    fn build_sig(&mut self, id: MuID) {
        self.visited.insert(id);

        let sig = self.b.bundle.sigs.get(&id).unwrap();

        trace!("Building function signature {} {:?}", id, sig);

        let hdr = self.make_mu_entity_header(id);

        let impl_sig = MuFuncSig{
            hdr: hdr,
            ret_tys: sig.rettys.iter().map(|i| self.ensure_type_rec(*i)).collect::<Vec<_>>(),
            arg_tys: sig.paramtys.iter().map(|i| self.ensure_type_rec(*i)).collect::<Vec<_>>(),
        };

        trace!("Function signature built: {} {:?}", id, impl_sig);

        self.built_sigs.insert(id, P(impl_sig));
    }

    fn ensure_sig_rec(&mut self, id: MuID) -> P<MuFuncSig> {
        if self.b.bundle.sigs.contains_key(&id) {
            if self.visited.contains(&id) {
                match self.built_sigs.get(&id) {
                    Some(t) => t.clone(),
                    None => panic!("Cyclic signature found. id: {}", id)
                }
            } else {
                self.build_sig(id);
                self.built_sigs.get(&id).unwrap().clone()
            }
        } else {
            self.vm.get_func_sig(id)
        }
    }

    fn build_const(&mut self, id: MuID) {
        self.visited.insert(id);

        let con = self.b.bundle.consts.get(&id).unwrap();

        trace!("Building constant {} {:?}", id, con);

        let hdr = self.make_mu_entity_header(id);

        let (impl_con, impl_ty) = match **con {
            NodeConst::ConstInt { id: _, ty, value } => {
                let t = self.ensure_type_rec(ty);
                let c = Constant::Int(value);
                (c, t)
            },
            NodeConst::ConstFloat { id: _, ty, value } => {
                let t = self.ensure_type_rec(ty);
                let c = Constant::Float(value);
                (c, t)
            },
            NodeConst::ConstDouble { id: _, ty, value } => {
                let t = self.ensure_type_rec(ty);
                let c = Constant::Double(value);
                (c, t)
            },
            NodeConst::ConstNull { id: _, ty } => {
                let t = self.ensure_type_rec(ty);
                let c = Constant::NullRef;
                (c, t)
            },
            NodeConst::ConstExtern { id: _, ty, ref symbol } => {
                let t = self.ensure_type_rec(ty);
                let c = Constant::ExternSym(symbol.clone());
                (c, t)
            },
            ref c => panic!("{:?} not implemented", c),
        };

        let impl_val = Value {
            hdr: hdr,
            ty: impl_ty,
            v: Value_::Constant(impl_con),
        };

        trace!("Constant built: {} {:?}", id, impl_val);

        self.built_constants.insert(id, P(impl_val));
    }

    fn build_global(&mut self, id: MuID) {
        self.visited.insert(id);

        let global = self.b.bundle.globals.get(&id).unwrap();

        trace!("Building global {} {:?}", id, global);

        let hdr = self.make_mu_entity_header(id);
        let impl_ty = self.ensure_type_rec(global.ty); // global type

        let impl_val = Value {
            hdr: hdr,
            ty: self.ensure_iref(impl_ty.id()), // iref to global
            v: Value_::Global(impl_ty)
        };

        trace!("Global built: {} {:?}", id, impl_val);

        self.built_globals.insert(id, P(impl_val));
    }

    fn build_func(&mut self, id: MuID) {
        self.visited.insert(id);

        let fun = self.b.bundle.funcs.get(&id).unwrap();

        trace!("Building function {} {:?}", id, fun);

        let hdr = self.make_mu_entity_header(id);
        let impl_sig = self.ensure_sig_rec(fun.sig);

        let impl_fun = MuFunction {
            hdr: hdr.clone(),
            sig: impl_sig,
            cur_ver: None,
            all_vers: Default::default(),
        };

        trace!("Function built: {} {:?}", id, impl_fun);

        self.built_funcs.insert(id, Box::new(impl_fun));

        let impl_ty = self.ensure_funcref(fun.sig);

        let impl_val = Value {
            hdr: hdr,
            ty: impl_ty,
            v: Value_::Constant(Constant::FuncRef(id)),
        };

        trace!("Function value built: {} {:?}", id, impl_val);

        self.built_constants.insert(id, P(impl_val));
    }

    fn get_sig_for_func(&mut self, id: MuID) -> P<MuFuncSig> {
        if let Some(impl_func) = self.built_funcs.get(&id) {
            impl_func.sig.clone()
        } else {
            self.vm.get_func_sig_for_func(id)
        }
    }


    fn build_funcver(&mut self, id: MuID) {
        let fv = self.b.bundle.funcvers.get(&id).unwrap();

        trace!("Building function version {} {:?}", id, fv);

        let hdr = self.make_mu_entity_header(id);
        let func_id = fv.func;
        let impl_sig = self.get_sig_for_func(func_id);

        let mut fcb: FuncCtxBuilder = Default::default();

        let blocks = fv.bbs.iter().map(|bbid| {
            let block = self.build_block(&mut fcb, *bbid);
            (*bbid, block)
        }).collect::<LinkedHashMap<MuID, Block>>();

        let entry_id = *fv.bbs.first().unwrap();
        let ctn = FunctionContent {
            entry: entry_id,
            blocks: blocks,
        };

        let impl_fv = MuFunctionVersion::new_(hdr, func_id, impl_sig, ctn, fcb.ctx);

        trace!("Function version built {} {:?}", id, impl_fv);

        self.built_funcvers.insert(id, Box::new(impl_fv));
    }

    /// Copied from ast::ir::*. That was implemented for the previous API which implies mutability.
    /// When we migrate later, we can assume the AST is almost fully immutable, and can be
    /// constructed in a functional recursive-descendent style.
    fn new_ssa(&self, fcb: &mut FuncCtxBuilder, id: MuID, ty: P<MuType>) -> P<TreeNode> {
        let hdr = self.make_mu_entity_header(id);
        let val = P(Value{
            hdr: hdr,
            ty: ty,
            v: Value_::SSAVar(id)
        });

        fcb.ctx.values.insert(id, SSAVarEntry::new(val.clone()));

        let tn = P(TreeNode {
            op: pick_op_code_for_ssa(&val.ty),
            v: TreeNode_::Value(val)
        });

        fcb.tree_nodes.insert(id, tn.clone());

        tn
    }

    pub fn new_inst(&self, v: Instruction) -> Box<TreeNode> {
        Box::new(TreeNode{
            op: pick_op_code_for_inst(&v),
            v: TreeNode_::Instruction(v),
        })
    }

    pub fn new_global(&self, v: P<Value>) -> P<TreeNode> {
        P(TreeNode{
            op: pick_op_code_for_value(&v.ty),
            v: TreeNode_::Value(v)
        })
    }

    fn get_treenode(&self, fcb: &FuncCtxBuilder, id: MuID) -> P<TreeNode> {
        if let Some(tn) = fcb.tree_nodes.get(&id) {
            tn.clone()
        } else if let Some(v) = self.built_constants.get(&id) {
            self.new_global(v.clone())
        } else if let Some(v) = self.built_globals.get(&id) {
            self.new_global(v.clone())
        } else {
            panic!("Operand {} is neither a local var or a global var", id)
        }
    }

    fn build_block(&mut self, fcb: &mut FuncCtxBuilder, id: MuID) -> Block {
        let bb = self.b.bundle.bbs.get(&id).unwrap();

        trace!("Building basic block {} {:?}", id, bb);

        let nor_ids = &bb.nor_param_ids;
        let nor_tys = &bb.nor_param_types;

        let args = nor_ids.iter().zip(nor_tys).map(|(arg_id, arg_ty_id)| {
            let arg_ty = self.get_built_type(*arg_ty_id);
            self.new_ssa(fcb, *arg_id, arg_ty).clone_value()
        }).collect::<Vec<_>>();

        let exn_arg = bb.exc_param_id.map(|arg_id| {
            let arg_ty = self.ensure_refi64();
            self.new_ssa(fcb, arg_id, arg_ty).clone_value()
        });

        let hdr = self.make_mu_entity_header(id);

        let body = self.build_block_content(fcb, &bb.insts);

        let ctn = BlockContent {
            args: args,
            exn_arg: exn_arg,
            body: body,
            keepalives: None,
        };

        Block {
            hdr: hdr,
            content: Some(ctn),
            control_flow: Default::default(),
        }
    }

    fn build_block_content(&mut self, fcb: &mut FuncCtxBuilder, insts: &Vec<MuID>) -> Vec<Box<TreeNode>> {
        insts.iter().map(|iid| self.build_inst(fcb, *iid)).collect::<Vec<_>>()
    }

    fn build_inst(&mut self, fcb: &mut FuncCtxBuilder, id: MuID) -> Box<TreeNode> {
        let inst = self.b.bundle.insts.get(&id).unwrap();

        trace!("Building instruction {} {:?}", id, inst);

        let hdr = self.make_mu_entity_header(id);

        let impl_inst = match **inst {
            NodeInst::NodeBinOp {
                id: _, result_id, status_result_ids: _,
                optr, flags: _, ty, opnd1, opnd2,
                exc_clause: _
            } => {
                let impl_optr = match optr {
                    CMU_BINOP_ADD  => BinOp::Add,
                    CMU_BINOP_SUB  => BinOp::Sub,
                    CMU_BINOP_MUL  => BinOp::Mul,
                    CMU_BINOP_SDIV => BinOp::Sdiv,
                    CMU_BINOP_SREM => BinOp::Srem,
                    CMU_BINOP_UDIV => BinOp::Udiv,
                    CMU_BINOP_UREM => BinOp::Urem,
                    CMU_BINOP_SHL  => BinOp::Shl,
                    CMU_BINOP_LSHR => BinOp::Lshr,
                    CMU_BINOP_ASHR => BinOp::Ashr,
                    CMU_BINOP_AND  => BinOp::And,
                    CMU_BINOP_OR   => BinOp::Or,
                    CMU_BINOP_XOR  => BinOp::Xor,
                    CMU_BINOP_FADD => BinOp::FAdd,
                    CMU_BINOP_FSUB => BinOp::FSub,
                    CMU_BINOP_FMUL => BinOp::FMul,
                    CMU_BINOP_FDIV => BinOp::FDiv,
                    CMU_BINOP_FREM => BinOp::FRem,
                    _ => panic!("Illegal binary operator {}", optr)
                };
                let impl_ty = self.get_built_type(ty);
                let impl_opnd1 = self.get_treenode(fcb, opnd1);
                let impl_opnd2 = self.get_treenode(fcb, opnd2);
                let impl_rv = self.new_ssa(fcb, result_id, impl_ty).clone_value();

                Instruction {
                    hdr: hdr,
                    value: Some(vec![impl_rv]),
                    ops: RwLock::new(vec![impl_opnd1, impl_opnd2]),
                    v: Instruction_::BinOp(impl_optr, 0, 1),
                }
            },
            NodeInst::NodeCmp {
                id: _, result_id, optr, ty, opnd1, opnd2
            } => {
                let impl_optr = match optr {
                    CMU_CMP_EQ  => CmpOp::EQ,
                    CMU_CMP_NE  => CmpOp::NE,
                    CMU_CMP_SGE    => CmpOp::SGE,
                    CMU_CMP_SGT    => CmpOp::SGT,
                    CMU_CMP_SLE    => CmpOp::SLE,
                    CMU_CMP_SLT    => CmpOp::SLT,
                    CMU_CMP_UGE    => CmpOp::UGE,
                    CMU_CMP_UGT    => CmpOp::UGT,
                    CMU_CMP_ULE    => CmpOp::ULE,
                    CMU_CMP_ULT    => CmpOp::ULT,
                    CMU_CMP_FFALSE => CmpOp::FFALSE,
                    CMU_CMP_FTRUE  => CmpOp::FTRUE,
                    CMU_CMP_FUNO   => CmpOp::FUNO,
                    CMU_CMP_FUEQ   => CmpOp::FUEQ,
                    CMU_CMP_FUNE   => CmpOp::FUNE,
                    CMU_CMP_FUGT   => CmpOp::FUGT,
                    CMU_CMP_FUGE   => CmpOp::FUGE,
                    CMU_CMP_FULT   => CmpOp::FULT,
                    CMU_CMP_FULE   => CmpOp::FULE,
                    CMU_CMP_FORD   => CmpOp::FORD,
                    CMU_CMP_FOEQ   => CmpOp::FOEQ,
                    CMU_CMP_FONE   => CmpOp::FONE,
                    CMU_CMP_FOGT   => CmpOp::FOGT,
                    CMU_CMP_FOGE   => CmpOp::FOGE,
                    CMU_CMP_FOLT   => CmpOp::FOLT,
                    CMU_CMP_FOLE   => CmpOp::FOLE,
                    _ => panic!("Illegal comparing operator {}", optr)
                };
                // NOTE: vectors not implemented. Otherwise the result would be a vector of
                // int<1>
                let impl_i1 = self.ensure_i1();
                let impl_opnd1 = self.get_treenode(fcb, opnd1);
                let impl_opnd2 = self.get_treenode(fcb, opnd2);
                let impl_rv = self.new_ssa(fcb, result_id, impl_i1).clone_value();

                Instruction {
                    hdr: hdr,
                    value: Some(vec![impl_rv]),
                    ops: RwLock::new(vec![impl_opnd1, impl_opnd2]),
                    v: Instruction_::CmpOp(impl_optr, 0, 1),
                }
            },
            NodeInst::NodeConv {
                id: _, result_id, optr, from_ty, to_ty, opnd
            } => {
                let impl_optr = match optr {
                    CMU_CONV_TRUNC   => ConvOp::TRUNC,
                    CMU_CONV_ZEXT    => ConvOp::ZEXT,
                    CMU_CONV_SEXT    => ConvOp::SEXT,
                    CMU_CONV_FPTRUNC => ConvOp::FPTRUNC,
                    CMU_CONV_FPEXT   => ConvOp::FPEXT,
                    CMU_CONV_FPTOUI  => ConvOp::FPTOUI,
                    CMU_CONV_FPTOSI  => ConvOp::FPTOSI,
                    CMU_CONV_UITOFP  => ConvOp::UITOFP,
                    CMU_CONV_SITOFP  => ConvOp::SITOFP,
                    CMU_CONV_BITCAST => ConvOp::BITCAST,
                    CMU_CONV_REFCAST => ConvOp::REFCAST,
                    CMU_CONV_PTRCAST => ConvOp::PTRCAST,
                    _ => panic!("Illegal conversion operator {}", optr)
                };
                let impl_from_ty = self.get_built_type(from_ty);
                let impl_to_ty = self.get_built_type(to_ty);
                let impl_opnd = self.get_treenode(fcb, opnd);
                let impl_rv = self.new_ssa(fcb, result_id, impl_to_ty.clone()).clone_value();

                Instruction {
                    hdr: hdr,
                    value: Some(vec![impl_rv]),
                    ops: RwLock::new(vec![impl_opnd]),
                    v: Instruction_::ConvOp {
                        operation: impl_optr,
                        from_ty: impl_from_ty,
                        to_ty: impl_to_ty,
                        operand: 0,
                    },
                }
            },
            NodeInst::NodeSelect { id: _, result_id, cond_ty, opnd_ty, cond, if_true, if_false } => {
                let impl_cond_ty = self.get_built_type(cond_ty);
                let impl_opnd_ty = self.get_built_type(opnd_ty);
                let impl_cond = self.get_treenode(fcb, cond);
                let impl_if_true = self.get_treenode(fcb, if_true);
                let impl_if_false = self.get_treenode(fcb, if_false);

                // NOTE: only implemented scalar SELECT. Vector select is not implemented yet.
                let impl_rv = self.new_ssa(fcb, result_id, impl_opnd_ty.clone()).clone_value();

                Instruction {
                    hdr: hdr,
                    value: Some(vec![impl_rv]),
                    ops: RwLock::new(vec![impl_cond, impl_if_true, impl_if_false]),
                    v: Instruction_::Select {
                        cond: 0,
                        true_val: 1,
                        false_val: 2,
                    },
                }
            },
            NodeInst::NodeBranch { id: _, dest } => { 
                let mut ops: Vec<P<TreeNode>> = Vec::new();

                let impl_dest = self.build_destination(fcb, dest, &mut ops, &[]);

                Instruction {
                    hdr: hdr,
                    value: None,
                    ops: RwLock::new(ops),
                    v: Instruction_::Branch1(impl_dest),
                }
            },
            NodeInst::NodeBranch2 { id: _, cond, if_true, if_false } => { 
                let mut ops: Vec<P<TreeNode>> = Vec::new();

                self.add_opnd(fcb, &mut ops, cond);

                let impl_dest_true = self.build_destination(fcb, if_true, &mut ops, &[]);

                let impl_dest_false = self.build_destination(fcb, if_false, &mut ops, &[]);

                Instruction {
                    hdr: hdr,
                    value: None,
                    ops: RwLock::new(ops),
                    v: Instruction_::Branch2 {
                        cond: 0,
                        true_dest: impl_dest_true,
                        false_dest: impl_dest_false,
                        true_prob: DEFAULT_TRUE_PROB,
                    },
                }
            },
            NodeInst::NodeSwitch {
                id: _, opnd_ty, opnd, default_dest, ref cases, ref dests
            } => {
                let mut ops: Vec<P<TreeNode>> = Vec::new();

                self.add_opnd(fcb, &mut ops, opnd);

                let impl_dest_def = self.build_destination(fcb, default_dest, &mut ops, &[]);

                let impl_branches = cases.iter().zip(dests).map(|(cid, did)| {
                    let case_opindex = ops.len();
                    self.add_opnd(fcb, &mut ops, *cid);

                    let impl_dest = self.build_destination(fcb, *did, &mut ops, &[]);

                    (case_opindex, impl_dest)
                }).collect::<Vec<_>>();

                Instruction {
                    hdr: hdr,
                    value: None,
                    ops: RwLock::new(ops),
                    v: Instruction_::Switch {
                        cond: 0,
                        default: impl_dest_def,
                        branches: impl_branches,
                    },
                }
            },
            NodeInst::NodeCall {
                id: _, ref result_ids, sig, callee, ref args, exc_clause, keepalive_clause
            } => {
                self.build_call_or_ccall(fcb, hdr, result_ids, sig, callee, args,
                                         exc_clause, keepalive_clause,
                                         false, CallConvention::Mu)
            },
            NodeInst::NodeTailCall {
                id: _, sig, callee, ref args
            } => {
                let mut ops: Vec<P<TreeNode>> = Vec::new();

                let call_data = self.build_call_data(fcb, &mut ops, callee, args, CallConvention::Mu);

                Instruction {
                    hdr: hdr,
                    value: None,
                    ops: RwLock::new(ops),
                    v: Instruction_::TailCall(call_data),
                }
            },
            NodeInst::NodeRet { id: _, ref rvs } => {
                let ops = rvs.iter().map(|rvid| self.get_treenode(fcb, *rvid)).collect::<Vec<_>>();
                let op_indexes = (0..(ops.len())).collect::<Vec<_>>();

                Instruction {
                    hdr: hdr,
                    value: None,
                    ops: RwLock::new(ops),
                    v: Instruction_::Return(op_indexes),
                }
            },
            NodeInst::NodeThrow { id: _, exc } => {
                let impl_exc = self.get_treenode(fcb, exc);

                Instruction {
                    hdr: hdr,
                    value: None,
                    ops: RwLock::new(vec![impl_exc]),
                    v: Instruction_::Throw(0),
                }
            },
            NodeInst::NodeNew { id: _, result_id, allocty, exc_clause } => {
                assert!(exc_clause.is_none(), "exc_clause is not implemented for NEW");
                let impl_allocty = self.get_built_type(allocty);
                let impl_rvtype = self.ensure_ref(allocty);
                let impl_rv = self.new_ssa(fcb, result_id, impl_rvtype).clone_value();
                Instruction {
                    hdr: hdr,
                    value: Some(vec![impl_rv]),
                    ops: RwLock::new(vec![]),
                    v: Instruction_::New(impl_allocty),
                }
            },
            NodeInst::NodeNewHybrid { id: _, result_id, allocty, lenty, length, exc_clause } => {
                assert!(exc_clause.is_none(), "exc_clause is not implemented for NEWHYBRID");
                let impl_allocty = self.get_built_type(allocty);
                let impl_length = self.get_treenode(fcb, length);
                let impl_rvtype = self.ensure_ref(allocty);
                let impl_rv = self.new_ssa(fcb, result_id, impl_rvtype).clone_value();
                Instruction {
                    hdr: hdr,
                    value: Some(vec![impl_rv]),
                    ops: RwLock::new(vec![impl_length]),
                    v: Instruction_::NewHybrid(impl_allocty, 0),
                }
            },
            NodeInst::NodeAlloca { id: _, result_id, allocty, exc_clause } => {
                assert!(exc_clause.is_none(), "exc_clause is not implemented for ALLOCA");
                let impl_allocty = self.get_built_type(allocty);
                let impl_rvtype = self.ensure_iref(allocty);
                let impl_rv = self.new_ssa(fcb, result_id, impl_rvtype).clone_value();
                Instruction {
                    hdr: hdr,
                    value: Some(vec![impl_rv]),
                    ops: RwLock::new(vec![]),
                    v: Instruction_::AllocA(impl_allocty),
                }
            },
            NodeInst::NodeAllocaHybrid { id: _, result_id, allocty, lenty, length, exc_clause } => {
                assert!(exc_clause.is_none(), "exc_clause is not implemented for ALLOCAHYBRID");
                let impl_allocty = self.get_built_type(allocty);
                let impl_length = self.get_treenode(fcb, length);
                let impl_rvtype = self.ensure_iref(allocty);
                let impl_rv = self.new_ssa(fcb, result_id, impl_rvtype).clone_value();
                Instruction {
                    hdr: hdr,
                    value: Some(vec![impl_rv]),
                    ops: RwLock::new(vec![impl_length]),
                    v: Instruction_::AllocAHybrid(impl_allocty, 0),
                }
            },
            NodeInst::NodeGetIRef { id: _, result_id, refty, opnd } => {
                let impl_opnd = self.get_treenode(fcb, opnd);
                let impl_rvtype = self.ensure_iref(refty);
                let impl_rv = self.new_ssa(fcb, result_id, impl_rvtype).clone_value();
                Instruction {
                    hdr: hdr,
                    value: Some(vec![impl_rv]),
                    ops: RwLock::new(vec![impl_opnd]),
                    v: Instruction_::GetIRef(0),
                }
            },
            NodeInst::NodeGetFieldIRef { id: _, result_id, is_ptr, refty, index, opnd } => {
                let impl_opnd = self.get_treenode(fcb, opnd);
                let index = index as usize;
                let refty_node = self.b.bundle.types.get(&refty).unwrap();
                let field_ty_id = match **refty_node {
                    NodeType::TypeStruct { id: _, ref fieldtys } => {
                        fieldtys[index]
                    },
                    NodeType::TypeHybrid { id: _, ref fixedtys, varty: _ } => {
                        fixedtys[index]
                    },
                    ref t => panic!("GETFIELDIREF {}: Expected struct or hybrid type. actual: {:?}", id, t)
                };
                let impl_rvtype = self.ensure_iref_or_uptr(field_ty_id, is_ptr);
                let impl_rv = self.new_ssa(fcb, result_id, impl_rvtype).clone_value();
                Instruction {
                    hdr: hdr,
                    value: Some(vec![impl_rv]),
                    ops: RwLock::new(vec![impl_opnd]),
                    v: Instruction_::GetFieldIRef {
                        is_ptr: is_ptr,
                        base: 0,
                        index: index,
                    },
                }
            },
            NodeInst::NodeGetElemIRef { id: _, result_id, is_ptr, refty, indty: _, opnd, index } => {
                let impl_opnd = self.get_treenode(fcb, opnd);
                let impl_index = self.get_treenode(fcb, index);
                let refty_node = self.b.bundle.types.get(&refty).unwrap();
                let elem_ty_id = match **refty_node {
                    NodeType::TypeArray { id: _, elemty, len: _ } => { 
                        elemty
                    },
                    NodeType::TypeVector { id: _, elemty, len: _ } => { 
                        elemty
                    },
                    ref t => panic!("GETELEMIREF {}: Expected array or vector type. actual: {:?}", id, t)
                };
                let impl_rvtype = self.ensure_iref_or_uptr(elem_ty_id, is_ptr);
                let impl_rv = self.new_ssa(fcb, result_id, impl_rvtype).clone_value();
                Instruction {
                    hdr: hdr,
                    value: Some(vec![impl_rv]),
                    ops: RwLock::new(vec![impl_opnd, impl_index]),
                    v: Instruction_::GetElementIRef {
                        is_ptr: is_ptr,
                        base: 0,
                        index: 1,
                    },
                }
            },
            NodeInst::NodeShiftIRef { id: _, result_id, is_ptr, refty, offty: _, opnd, offset } => {
                let impl_opnd = self.get_treenode(fcb, opnd);
                let impl_offset = self.get_treenode(fcb, offset);
                let impl_rvtype = self.ensure_iref_or_uptr(refty, is_ptr);
                let impl_rv = self.new_ssa(fcb, result_id, impl_rvtype).clone_value();
                Instruction {
                    hdr: hdr,
                    value: Some(vec![impl_rv]),
                    ops: RwLock::new(vec![impl_opnd, impl_offset]),
                    v: Instruction_::ShiftIRef {
                        is_ptr: is_ptr,
                        base: 0,
                        offset: 1,
                    },
                }
            },
            NodeInst::NodeGetVarPartIRef { id: _, result_id, is_ptr, refty, opnd } => {
                let impl_opnd = self.get_treenode(fcb, opnd);
                let refty_node = self.b.bundle.types.get(&refty).unwrap();
                let elem_ty_id = match **refty_node {
                    NodeType::TypeHybrid { id: _, fixedtys: _, varty } => { 
                        varty
                    },
                    ref t => panic!("GETVARPARTIREF {}: Expected hybrid type. actual: {:?}", id, t)
                };
                let impl_rvtype = self.ensure_iref_or_uptr(elem_ty_id, is_ptr);
                let impl_rv = self.new_ssa(fcb, result_id, impl_rvtype).clone_value();
                Instruction {
                    hdr: hdr,
                    value: Some(vec![impl_rv]),
                    ops: RwLock::new(vec![impl_opnd]),
                    v: Instruction_::GetVarPartIRef {
                        is_ptr: is_ptr,
                        base: 0,
                    },
                }
            },
            NodeInst::NodeLoad { id: _, result_id, is_ptr, ord, refty, loc, exc_clause } => {
                let impl_ord = self.build_mem_ord(ord);
                let impl_loc = self.get_treenode(fcb, loc);
                let impl_rvtype = self.get_built_type(refty);
                let impl_rv = self.new_ssa(fcb, result_id, impl_rvtype).clone_value();
                Instruction {
                    hdr: hdr,
                    value: Some(vec![impl_rv]),
                    ops: RwLock::new(vec![impl_loc]),
                    v: Instruction_::Load {
                        is_ptr: is_ptr,
                        order: impl_ord,
                        mem_loc: 0,
                    },
                }
            },
            NodeInst::NodeStore { id: _, is_ptr, ord, refty, loc, newval, exc_clause } => {
                let impl_ord = self.build_mem_ord(ord);
                let impl_loc = self.get_treenode(fcb, loc);
                let impl_newval = self.get_treenode(fcb, newval);
                let impl_rvtype = self.get_built_type(refty);
                Instruction {
                    hdr: hdr,
                    value: None,
                    ops: RwLock::new(vec![impl_loc, impl_newval]),
                    v: Instruction_::Store {
                        is_ptr: is_ptr,
                        order: impl_ord,
                        mem_loc: 0,
                        value: 1,
                    },
                }
            },
            NodeInst::NodeCCall {
                id: _, ref result_ids, callconv: _, callee_ty: _,
                sig, callee, ref args, exc_clause, keepalive_clause
            } => {
                self.build_call_or_ccall(fcb, hdr, result_ids, sig, callee, args,
                                         exc_clause, keepalive_clause,
                                         true, CallConvention::Foreign(ForeignFFI::C))
            },
            NodeInst::NodeCommInst {
                id, ref result_ids, opcode,
                ref flags, ref tys, ref sigs, ref args,
                ref exc_clause, ref keepalive_clause
            } => {
                self.build_comm_inst(fcb, hdr, result_ids, opcode, flags, tys, sigs, args, exc_clause, keepalive_clause)
            },

            ref i => panic!("{:?} not implemented", i),
        };

        trace!("Instruction built {} {:?}", id, impl_inst);

        self.new_inst(impl_inst)
    }

    fn build_destination(&mut self, fcb: &mut FuncCtxBuilder, id: MuID,
                         ops: &mut Vec<P<TreeNode>>, inst_result_ids: &[MuID],
                         ) -> Destination {
        let dest_clause = self.b.bundle.dest_clauses.get(&id).unwrap();

        let target = dest_clause.dest;

        let dest_args = dest_clause.vars.iter().map(|vid| {
//            if let Some(ind) = inst_result_ids.iter().position(|rid| *rid == *vid) {
//                DestArg::Freshbound(ind)
//            } else {
//                let my_index = ops.len();
//                self.add_opnd(fcb, ops, *vid);
//                DestArg::Normal(my_index)
//            }
            let my_index = ops.len();
            self.add_opnd(fcb, ops, *vid);
            DestArg::Normal(my_index)
        }).collect::<Vec<_>>();

        let impl_dest = Destination {
            target: target,
            args: dest_args,
        };

        impl_dest
    }

    fn add_opnd(&mut self, fcb: &mut FuncCtxBuilder, ops: &mut Vec<P<TreeNode>>, opnd: MuID) {
        let impl_opnd = self.get_treenode(fcb, opnd);
        ops.push(impl_opnd);
    }

    fn add_opnds(&mut self, fcb: &mut FuncCtxBuilder, ops: &mut Vec<P<TreeNode>>, opnds: &[MuID]) {
        for opnd in opnds {
            self.add_opnd(fcb, ops, *opnd)
        }
    }

    fn build_call_data(&mut self, fcb: &mut FuncCtxBuilder, ops: &mut Vec<P<TreeNode>>,
                       callee: MuID, args: &[MuID], call_conv: CallConvention) -> CallData {
        let func_index = ops.len();
        self.add_opnd(fcb, ops, callee);

        let args_begin_index = ops.len();
        self.add_opnds(fcb, ops, args);

        let args_opindexes = (args_begin_index..(args.len()+1)).collect::<Vec<_>>();

        let call_data = CallData {
            func: func_index,
            args: args_opindexes,
            convention: call_conv,
        };

        call_data
    }

    fn build_call_or_ccall(&mut self, fcb: &mut FuncCtxBuilder, hdr: MuEntityHeader,
                           result_ids: &[MuID], sig: MuID, callee: MuID, args: &[MuID],
                           exc_clause: Option<MuID>, keepalive_claue: Option<MuID>,
                           is_ccall: bool, call_conv: CallConvention) -> Instruction {
        let mut ops: Vec<P<TreeNode>> = Vec::new();

        let call_data = self.build_call_data(fcb, &mut ops, callee, args, CallConvention::Mu);

        let signode = self.b.bundle.sigs.get(&sig).unwrap();
        let rettys_ids = &signode.rettys;

        let rvs = result_ids.iter().zip(rettys_ids).map(|(rvid, rvty)| {
            let impl_rvty = self.get_built_type(*rvty);
            self.new_ssa(fcb, *rvid, impl_rvty).clone_value()
        }).collect::<Vec<_>>();

        if let Some(ecid) = exc_clause {
            // terminating inst
            let ecnode = self.b.bundle.exc_clauses.get(&ecid).unwrap();

            let impl_normal_dest = {
                self.build_destination(fcb, ecnode.nor, &mut ops, result_ids)
            };

            let impl_exn_dest = {
                self.build_destination(fcb, ecnode.exc, &mut ops, &[])
            };

            let resumption_data = ResumptionData {
                normal_dest: impl_normal_dest,
                exn_dest: impl_exn_dest,
            };

            let impl_inst_ = if is_ccall {
                Instruction_::CCall{
                    data: call_data,
                    resume: resumption_data,
                }
            } else {
                Instruction_::Call{
                    data: call_data,
                    resume: resumption_data,
                }
            };

            Instruction {
                hdr: hdr,
                value: Some(rvs),
                ops: RwLock::new(ops),
                v: impl_inst_,
            }
        } else {
            // non-terminating inst
            Instruction {
                hdr: hdr,
                value: Some(rvs),
                ops: RwLock::new(ops),
                v: if is_ccall {
                    Instruction_::ExprCCall {
                        data: call_data,
                        is_abort: false,
                    }
                } else {
                    Instruction_::ExprCall {
                        data: call_data,
                        is_abort: false,
                    }
                },
            }
        }
    }

    #[allow(unused_variables)]
    fn build_comm_inst(&mut self, fcb: &mut FuncCtxBuilder, hdr: MuEntityHeader,
                       result_ids: &Vec<MuID>, opcode: MuCommInst, flags: &Vec<Flag>,
                       tys: &Vec<MuTypeNode>, sigs: &Vec<MuFuncSigNode>, args: &Vec<MuVarNode>,
                       exc_clause: &Option<MuExcClause>, keepalives: &Option<MuKeepaliveClause>) -> Instruction {
        match opcode {
            CMU_CI_UVM_GET_THREADLOCAL => {
                assert!(result_ids.len() == 1);

                let rv_ty = self.ensure_refvoid();
                let rv = self.new_ssa(fcb, result_ids[0], rv_ty).clone_value();

                Instruction {
                    hdr: hdr,
                    value: Some(vec![rv]),
                    ops: RwLock::new(vec![]),
                    v: Instruction_::CommonInst_GetThreadLocal
                }
            }
            CMU_CI_UVM_SET_THREADLOCAL => {
                assert!(args.len() == 1);

                let op = self.get_treenode(fcb, args[0]);

                Instruction {
                    hdr: hdr,
                    value: None,
                    ops: RwLock::new(vec![op]),
                    v: Instruction_::CommonInst_SetThreadLocal(0)
                }
            }
            CMU_CI_UVM_NATIVE_PIN => {
                assert!(result_ids.len() == 1);
                assert!(args.len() == 1);
                assert!(tys.len()  == 1);

                let op_ty = self.ensure_type_rec(tys[0]);
                let op = self.get_treenode(fcb, args[0]);

                let referent_ty = match op_ty.get_referenced_ty() {
                    Some(ty) => ty,
                    _ => panic!("expected ty in PIN to be ref/iref, found {}", op_ty)
                };

                let rv_ty = self.ensure_uptr(referent_ty.id());
                let rv = self.new_ssa(fcb, result_ids[0], rv_ty).clone_value();

                Instruction {
                    hdr: hdr,
                    value: Some(vec![rv]),
                    ops: RwLock::new(vec![op]),
                    v: Instruction_::CommonInst_Pin(0)
                }
            }
            CMU_CI_UVM_NATIVE_UNPIN => {
                assert!(args.len() == 1);
                assert!(tys.len()  == 1);

                let op_ty = self.ensure_type_rec(tys[0]);
                let op = self.get_treenode(fcb, args[0]);

                Instruction {
                    hdr: hdr,
                    value: None,
                    ops: RwLock::new(vec![op]),
                    v: Instruction_::CommonInst_Unpin(0)
                }
            }
            CMU_CI_UVM_THREAD_EXIT => {
                Instruction {
                    hdr: hdr,
                    value: None,
                    ops: RwLock::new(vec![]),
                    v: Instruction_::ThreadExit
                }
            }
            _ => unimplemented!()
        }
    }

    fn build_mem_ord(&self, ord: MuMemoryOrder) -> MemoryOrder {
        match ord {
            CMU_ORD_NOT_ATOMIC => MemoryOrder::NotAtomic,
            CMU_ORD_RELAXED => MemoryOrder::Relaxed,
            CMU_ORD_CONSUME => MemoryOrder::Consume,
            CMU_ORD_ACQUIRE => MemoryOrder::Acquire,
            CMU_ORD_RELEASE => MemoryOrder::Release,
            CMU_ORD_ACQ_REL => MemoryOrder::AcqRel,
            CMU_ORD_SEQ_CST => MemoryOrder::SeqCst,
            o => panic!("Illegal memory order {}", o),
        }
    }

    fn add_everything_to_vm(&mut self) {
        let vm = self.b.get_mvm_immutable().vm.clone();
        let arc_vm = vm.clone();

        trace!("Loading bundle to the VM...");

        vm.declare_many(
            &mut self.id_name_map,
            &mut self.built_types,
            &mut self.built_sigs,
            &mut self.built_constants,
            &mut self.built_globals,
            &mut self.built_funcs,
            &mut self.built_funcvers,
            arc_vm
            );

        trace!("Bundle loaded to the VM!");
    }
}
