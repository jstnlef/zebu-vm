use super::common::*;
use ast::op::*;
use ast::inst::*;

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

pub type IdMap<T> = HashMap<MuID, Box<T>>;

/// A trantient bundle, i.e. the bundle being built, but not yet loaded into the MuVM.
#[derive(Default)]
pub struct TrantientBundle {
    types: IdMap<NodeType>,
    sigs: IdMap<NodeFuncSig>,
    consts: IdMap<NodeConst>,
    globals: IdMap<NodeGlobalCell>,
    funcs: IdMap<NodeFunc>,
    expfuncs: IdMap<NodeExpFunc>,
    funcvers: IdMap<NodeFuncVer>,
    bbs: IdMap<NodeBB>,
    insts: IdMap<NodeInst>,
    dest_clauses: IdMap<NodeDestClause>,
    exc_clauses: IdMap<NodeExcClause>,
    ka_clauses: IdMap<NodeKeepaliveClause>,
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
        panic!("Not implemented")
    }

    pub fn new_type_double(&mut self, id: MuID) {
        panic!("Not implemented")
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
        panic!("Not implemented")
    }

    pub fn new_type_array(&mut self, id: MuID, elemty: MuID, len: u64) {
        panic!("Not implemented")
    }

    pub fn new_type_vector(&mut self, id: MuID, elemty: MuID, len: u64) {
        panic!("Not implemented")
    }

    pub fn new_type_void(&mut self, id: MuID) {
        panic!("Not implemented")
    }

    pub fn new_type_ref(&mut self, id: MuID, ty: MuID) {
        panic!("Not implemented")
    }

    pub fn new_type_iref(&mut self, id: MuID, ty: MuID) {
        panic!("Not implemented")
    }

    pub fn new_type_weakref(&mut self, id: MuID, ty: MuID) {
        panic!("Not implemented")
    }

    pub fn new_type_funcref(&mut self, id: MuID, sig: MuID) {
        panic!("Not implemented")
    }

    pub fn new_type_tagref64(&mut self, id: MuID) {
        panic!("Not implemented")
    }

    pub fn new_type_threadref(&mut self, id: MuID) {
        panic!("Not implemented")
    }

    pub fn new_type_stackref(&mut self, id: MuID) {
        panic!("Not implemented")
    }

    pub fn new_type_framecursorref(&mut self, id: MuID) {
        panic!("Not implemented")
    }

    pub fn new_type_irbuilderref(&mut self, id: MuID) {
        panic!("Not implemented")
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
        panic!("Not implemented")
    }

    pub fn new_const_double(&mut self, id: MuID, ty: MuID, value: f64) {
        panic!("Not implemented")
    }

    pub fn new_const_null(&mut self, id: MuID, ty: MuID) {
        panic!("Not implemented")
    }

    pub fn new_const_seq(&mut self, id: MuID, ty: MuID, elems: Vec<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_const_extern(&mut self, id: MuID, ty: MuID, symbol: String) {
        panic!("Not implemented")
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
        panic!("Not implemented")
    }

    pub fn new_keepalive_clause(&mut self, id: MuID, vars: Vec<MuID>) {
        panic!("Not implemented")
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
    built_values: IdPMap<Value>,
    built_funcs: IdPMap<MuFunction>,
    built_funcvers: IdPMap<MuFunctionVersion>,
    struct_id_tags: Vec<(MuID, MuName)>,
    built_refi64: Option<P<MuType>>,
    built_i1: Option<P<MuType>>,
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
        built_values: Default::default(),
        built_funcs: Default::default(),
        built_funcvers: Default::default(),
        struct_id_tags: Default::default(),
        built_refi64: Default::default(),
        built_i1: Default::default(),
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
        if let Some(ref i1) = self.built_i1 {
            return i1.clone();
        }

        let id_i1 = self.vm.next_id();

        let impl_i1 = P(MuType {
            hdr: MuEntityHeader::unnamed(id_i1),
            v: MuType_::Int(1),
        });

        trace!("Ensure i1 is defined: {} {:?}", id_i1, impl_i1);

        self.built_types.insert(id_i1, impl_i1.clone());
        self.built_i1 = Some(impl_i1.clone());

        impl_i1
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
        // Make sure structs have names because struct names are used to resolve cyclic
        // dependencies.
        for (id, ty) in &self.b.bundle.types {
            match **ty {
                NodeType::TypeStruct { id: _, fieldtys: _ } => { 
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

        let struct_id_tags = self.struct_id_tags.drain(..).collect::<Vec<_>>();
        for (id, ref tag) in struct_id_tags {
            self.fill_struct(id, tag)
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
            NodeType::TypeInt { id: _, len: len } => {
                MuType_::Int(len as usize)
            },
            NodeType::TypeUPtr { id: _, ty: toty } => {
                let toty_i = self.ensure_type_rec(toty);
                MuType_::UPtr(toty_i)
            },
            NodeType::TypeUFuncPtr { id: _, sig: sig } => {
                let sig_i = self.ensure_sig_rec(sig);
                MuType_::UFuncPtr(sig_i)
            },
            NodeType::TypeStruct { id: _, fieldtys: _ } => { 
                let tag = self.get_name(id);
                self.struct_id_tags.push((id, tag.clone()));
                MuType_::Struct(tag)
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

    fn fill_struct(&mut self, id: MuID, tag: &MuName) {
        let ty = self.b.bundle.types.get(&id).unwrap();

        trace!("Filling struct {} {:?}", id, ty);

        match **ty {
            NodeType::TypeStruct { id: _, fieldtys: ref fieldtys } => { 
                let fieldtys_impl = fieldtys.iter().map(|fid| {
                    self.ensure_type_rec(*fid)
                }).collect::<Vec<_>>();

                let struct_ty_ = StructType_::new(fieldtys_impl);

                match STRUCT_TAG_MAP.read().unwrap().get(tag) {
                    Some(old_struct_ty_) => {
                        if struct_ty_ != *old_struct_ty_ {
                            panic!("trying to insert {:?} as {}, while the old struct is defined as {:?}",
                                   struct_ty_, tag, old_struct_ty_)
                        }
                    },
                    None => {}
                }
                STRUCT_TAG_MAP.write().unwrap().insert(tag.clone(), struct_ty_);

                trace!("Struct {} filled: {:?}", id,
                       STRUCT_TAG_MAP.read().unwrap().get(tag));
            },
            ref t => panic!("{} {:?} should be a Struct type", id, ty),
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
            NodeConst::ConstInt { id: _, ty: ty, value: value } => {
                let t = self.ensure_type_rec(ty);
                let c = Constant::Int(value);
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

        self.built_values.insert(id, P(impl_val));
    }

    fn build_func(&mut self, id: MuID) {
        self.visited.insert(id);

        let fun = self.b.bundle.funcs.get(&id).unwrap();

        trace!("Building function {} {:?}", id, fun);

        let hdr = self.make_mu_entity_header(id);
        let impl_sig = self.ensure_sig_rec(fun.sig);

        let impl_fun = MuFunction {
            hdr: hdr,
            sig: impl_sig,
            cur_ver: None,
            all_vers: Default::default(),
        };

        trace!("Function built: {} {:?}", id, impl_fun);

        self.built_funcs.insert(id, P(impl_fun));
    }

    fn build_funcver(&mut self, id: MuID) {
        let fv = self.b.bundle.funcvers.get(&id).unwrap();

        trace!("Building function version {} {:?}", id, fv);

        let hdr = self.make_mu_entity_header(id);
        let impl_sig = {
            let fun = self.built_funcs.get(&fv.func).unwrap();
            fun.sig.clone()
        };

        let mut fcb: FuncCtxBuilder = Default::default();

        let blocks = fv.bbs.iter().map(|bbid| {
            let block = self.build_block(&mut fcb, *bbid);
            (*bbid, block)
        }).collect::<HashMap<MuID, Block>>();

        let entry_id = *fv.bbs.first().unwrap();
        let ctn = FunctionContent {
            entry: entry_id,
            blocks: blocks,
        };

        let impl_fv = MuFunctionVersion {
            hdr: hdr,
            func_id: id,
            sig: impl_sig,
            content: Some(ctn),
            context: fcb.ctx,
            block_trace: None,
        };

        trace!("Function version built {} {:?}", id, impl_fv);

        self.built_funcvers.insert(id, P(impl_fv));
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
        } else if let Some(v) = self.built_values.get(&id) {
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
                exc_clause: _ } => {
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
                    let impl_rv = self.new_ssa(fcb, result_id, impl_ty);
                    let impl_rv_value = impl_rv.clone_value();

                    Instruction {
                        hdr: hdr,
                        value: Some(vec![impl_rv_value]),
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
                    let impl_rv = self.new_ssa(fcb, result_id, impl_i1);
                    let impl_rv_value = impl_rv.clone_value();

                    Instruction {
                        hdr: hdr,
                        value: Some(vec![impl_rv_value]),
                        ops: RwLock::new(vec![impl_opnd1, impl_opnd2]),
                        v: Instruction_::CmpOp(impl_optr, 0, 1),
                    }
                },
                NodeInst::NodeConv {
                    id: _, result_id, optr, from_ty, to_ty, opnd
                } => {
                    panic!("Conversion not implemented")
                    // let impl_optr = match optr {
                    //     CMU_CONV_TRUNC   => ComvOp::TRUNC,
                    //     CMU_CONV_ZEXT    => ComvOp::ZEXT,
                    //     CMU_CONV_SEXT    => ComvOp::SEXT,
                    //     CMU_CONV_FPTRUNC => ComvOp::FPTRUNC,
                    //     CMU_CONV_FPEXT   => ComvOp::FPEXT,
                    //     CMU_CONV_FPTOUI  => ComvOp::FPTOUI,
                    //     CMU_CONV_FPTOSI  => ComvOp::FPTOSI,
                    //     CMU_CONV_UITOFP  => ComvOp::UITOFP,
                    //     CMU_CONV_SITOFP  => ComvOp::SITOFP,
                    //     CMU_CONV_BITCAST => ComvOp::BITCAST,
                    //     CMU_CONV_REFCAST => ComvOp::REFCAST,
                    //     CMU_CONV_PTRCAST => ComvOp::PTRCAST,
                    //     _ => panic!("Illegal conversion operator {}", optr)
                    // };
                    // let impl_to_ty = self.get_built_type(to_ty);
                    // let impl_opnd = self.get_treenode(fcb, opnd);
                    // let impl_rv = self.new_ssa(fcb, result_id, impl_to_ty);
                    // let impl_rv_value = impl_rv.clone_value();

                    // Instruction {
                    //     hdr: hdr,
                    //     value: Some(vec![impl_rv_value]),
                    //     ops: RwLock::new(vec![impl_opnd]),
                    //     v: Instruction_::ComvOp(impl_optr, 0),
                    // }
                },
                NodeInst::NodeBranch { id: _, dest } => { 
                    let (impl_dest, ops) = self.build_destination(fcb, dest, 0, &[]);

                    Instruction {
                        hdr: hdr,
                        value: None,
                        ops: RwLock::new(ops),
                        v: Instruction_::Branch1(impl_dest),
                    }
                }
            NodeInst::NodeBranch2 { id: _, cond, if_true, if_false } => { 
                let mut ops: Vec<P<TreeNode>> = Vec::new();

                let impl_cond = self.get_treenode(fcb, cond);
                ops.push(impl_cond);

                let (impl_dest_true, mut true_ops) = self.build_destination(fcb, if_true, ops.len(), &[]);
                ops.append(&mut true_ops);

                let (impl_dest_false, mut false_ops) = self.build_destination(fcb, if_false, ops.len(), &[]);
                ops.append(&mut false_ops);

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
            }
            NodeInst::NodeRet { id: _, ref rvs } => {
                let ops = rvs.iter().map(|rvid| self.get_treenode(fcb, *rvid)).collect::<Vec<_>>();
                let op_indexes = (0..(ops.len())).collect::<Vec<_>>();

                Instruction {
                    hdr: hdr,
                    value: None,
                    ops: RwLock::new(ops),
                    v: Instruction_::Return(op_indexes),
                }
            }
            ref i => panic!("{:?} not implemented", i),
        };

        trace!("Instruction built {} {:?}", id, impl_inst);

        self.new_inst(impl_inst)
    }

    fn build_destination(&mut self, fcb: &mut FuncCtxBuilder, id: MuID,
                         next_op_index: usize, inst_result_ids: &[MuID],
                         ) -> (Destination, Vec<P<TreeNode>>) {
        let dest_clause = self.b.bundle.dest_clauses.get(&id).unwrap();

        let target = dest_clause.dest;

        let mut next_my_index = next_op_index;
        let mut var_treenodes: Vec<P<TreeNode>> = Vec::new();

        let dest_args = dest_clause.vars.iter().map(|vid| {
            if let Some(ind) = inst_result_ids.iter().position(|rid| *rid == *vid) {
                DestArg::Freshbound(ind)
            } else {
                let treenode = self.get_treenode(fcb, *vid);
                let my_index = next_my_index;
                next_my_index += 1;
                var_treenodes.push(treenode);
                DestArg::Normal(my_index)
            }
        }).collect::<Vec<_>>();

        let impl_dest = Destination {
            target: target,
            args: dest_args,
        };

        (impl_dest, var_treenodes)
    }
}
