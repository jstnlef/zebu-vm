use super::common::*;
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

//        let maybe_name = self.consume_name_of(id);
//        let pty = P(MuType {
//            hdr: MuEntityHeader {
//                id: id,
//                name: RwLock::new(maybe_name),
//            },
//            v: MuType_::Int(len as usize),
//        });
//
//        self.bundle.types.push(pty);
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
            norParamIDs: nor_param_ids, norParamTys: nor_param_types,
            excParamID: exc_param_id, insts: insts }));
    }

    pub fn new_dest_clause(&mut self, id: MuID, dest: MuID, vars: Vec<MuID>) {
        panic!("Not implemented")
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

    pub fn new_binop(&mut self, id: MuID, result_id: MuID, optr: CMuBinOptr, ty: MuID, opnd1: MuID, opnd2: MuID, exc_clause: Option<MuID>) {
        self.bundle.insts.insert(id, Box::new(NodeInst::NodeBinOp {
            id: id, resultID: result_id, statusResultIDs: vec![],
            optr: optr, flags: 0, ty: ty, opnd1: opnd1, opnd2: opnd2,
            excClause: exc_clause}));
    }

    pub fn new_binop_with_status(&mut self, id: MuID, result_id: MuID, status_result_ids: Vec<MuID>, optr: CMuBinOptr, status_flags: CMuBinOpStatus, ty: MuID, opnd1: MuID, opnd2: MuID, exc_clause: Option<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_cmp(&mut self, id: MuID, result_id: MuID, optr: CMuCmpOptr, ty: MuID, opnd1: MuID, opnd2: MuID) {
        panic!("Not implemented")
    }

    pub fn new_conv(&mut self, id: MuID, result_id: MuID, optr: CMuConvOptr, from_ty: MuID, to_ty: MuID, opnd: MuID) {
        panic!("Not implemented")
    }

    pub fn new_select(&mut self, id: MuID, result_id: MuID, cond_ty: MuID, opnd_ty: MuID, cond: MuID, if_true: MuID, if_false: MuID) {
        panic!("Not implemented")
    }

    pub fn new_branch(&mut self, id: MuID, dest: MuID) {
        self.bundle.insts.insert(id, Box::new(NodeInst::NodeBranch {
            id: id, dest: dest }));
    }

    pub fn new_branch2(&mut self, id: MuID, cond: MuID, if_true: MuID, if_false: MuID) {
        panic!("Not implemented")
    }

    pub fn new_switch(&mut self, id: MuID, opnd_ty: MuID, opnd: MuID, default_dest: MuID, cases: Vec<MuID>, dests: Vec<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_call(&mut self, id: MuID, result_ids: Vec<MuID>, sig: MuID, callee: MuID, args: Vec<MuID>, exc_clause: Option<MuID>, keepalive_clause: Option<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_tailcall(&mut self, id: MuID, sig: MuID, callee: MuID, args: Vec<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_ret(&mut self, id: MuID, rvs: Vec<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_throw(&mut self, id: MuID, exc: MuID) {
        panic!("Not implemented")
    }

    pub fn new_extractvalue(&mut self, id: MuID, result_id: MuID, strty: MuID, index: c_int, opnd: MuID) {
        panic!("Not implemented")
    }

    pub fn new_insertvalue(&mut self, id: MuID, result_id: MuID, strty: MuID, index: c_int, opnd: MuID, newval: MuID) {
        panic!("Not implemented")
    }

    pub fn new_extractelement(&mut self, id: MuID, result_id: MuID, seqty: MuID, indty: MuID, opnd: MuID, index: MuID) {
        panic!("Not implemented")
    }

    pub fn new_insertelement(&mut self, id: MuID, result_id: MuID, seqty: MuID, indty: MuID, opnd: MuID, index: MuID, newval: MuID) {
        panic!("Not implemented")
    }

    pub fn new_shufflevector(&mut self, id: MuID, result_id: MuID, vecty: MuID, maskty: MuID, vec1: MuID, vec2: MuID, mask: MuID) {
        panic!("Not implemented")
    }

    pub fn new_new(&mut self, id: MuID, result_id: MuID, allocty: MuID, exc_clause: Option<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_newhybrid(&mut self, id: MuID, result_id: MuID, allocty: MuID, lenty: MuID, length: MuID, exc_clause: Option<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_alloca(&mut self, id: MuID, result_id: MuID, allocty: MuID, exc_clause: Option<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_allocahybrid(&mut self, id: MuID, result_id: MuID, allocty: MuID, lenty: MuID, length: MuID, exc_clause: Option<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_getiref(&mut self, id: MuID, result_id: MuID, refty: MuID, opnd: MuID) {
        panic!("Not implemented")
    }

    pub fn new_getfieldiref(&mut self, id: MuID, result_id: MuID, is_ptr: bool, refty: MuID, index: c_int, opnd: MuID) {
        panic!("Not implemented")
    }

    pub fn new_getelemiref(&mut self, id: MuID, result_id: MuID, is_ptr: bool, refty: MuID, indty: MuID, opnd: MuID, index: MuID) {
        panic!("Not implemented")
    }

    pub fn new_shiftiref(&mut self, id: MuID, result_id: MuID, is_ptr: bool, refty: MuID, offty: MuID, opnd: MuID, offset: MuID) {
        panic!("Not implemented")
    }

    pub fn new_getvarpartiref(&mut self, id: MuID, result_id: MuID, is_ptr: bool, refty: MuID, opnd: MuID) {
        panic!("Not implemented")
    }

    pub fn new_load(&mut self, id: MuID, result_id: MuID, is_ptr: bool, ord: CMuMemOrd, refty: MuID, loc: MuID, exc_clause: Option<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_store(&mut self, id: MuID, is_ptr: bool, ord: CMuMemOrd, refty: MuID, loc: MuID, newval: MuID, exc_clause: Option<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_cmpxchg(&mut self, id: MuID, value_result_id: MuID, succ_result_id: MuID, is_ptr: bool, is_weak: bool, ord_succ: CMuMemOrd, ord_fail: CMuMemOrd, refty: MuID, loc: MuID, expected: MuID, desired: MuID, exc_clause: Option<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_atomicrmw(&mut self, id: MuID, result_id: MuID, is_ptr: bool, ord: CMuMemOrd, optr: CMuAtomicRMWOptr, ref_ty: MuID, loc: MuID, opnd: MuID, exc_clause: Option<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_fence(&mut self, id: MuID, ord: CMuMemOrd) {
        panic!("Not implemented")
    }

    pub fn new_trap(&mut self, id: MuID, result_ids: Vec<MuID>, rettys: Vec<MuID>, exc_clause: Option<MuID>, keepalive_clause: Option<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_watchpoint(&mut self, id: MuID, wpid: CMuWPID, result_ids: Vec<MuID>, rettys: Vec<MuID>, dis: MuID, ena: MuID, exc: Option<MuID>, keepalive_clause: Option<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_wpbranch(&mut self, id: MuID, wpid: CMuWPID, dis: MuID, ena: MuID) {
        panic!("Not implemented")
    }

    pub fn new_ccall(&mut self, id: MuID, result_ids: Vec<MuID>, callconv: CMuCallConv, callee_ty: MuID, sig: MuID, callee: MuID, args: Vec<MuID>, exc_clause: Option<MuID>, keepalive_clause: Option<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_newthread(&mut self, id: MuID, result_id: MuID, stack: MuID, threadlocal: Option<MuID>, new_stack_clause: MuID, exc_clause: Option<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_swapstack(&mut self, id: MuID, result_ids: Vec<MuID>, swappee: MuID, cur_stack_clause: MuID, new_stack_clause: MuID, exc_clause: Option<MuID>, keepalive_clause: Option<MuID>) {
        panic!("Not implemented")
    }

    pub fn new_comminst(&mut self, id: MuID, result_ids: Vec<MuID>, opcode: CMuCommInst, flags: &[CMuFlag], tys: Vec<MuID>, sigs: Vec<MuID>, args: Vec<MuID>, exc_clause: Option<MuID>, keepalive_clause: Option<MuID>) {
        self.bundle.insts.insert(id, Box::new(NodeInst::NodeCommInst {
            id: id, resultIDs: result_ids,
            opcode: opcode, flags: vec![], tys: tys, sigs: sigs, args: args,
            excClause: exc_clause, keepaliveClause: keepalive_clause
        }));
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
    struct_id_tags: Vec<(MuID, MuName)>,
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
        struct_id_tags: Default::default(),
    };

    bl.load_bundle();
}

impl<'lb, 'lvm> BundleLoader<'lb, 'lvm> {
    fn load_bundle(&mut self) {
        self.ensure_names();
        self.build_toplevels();
    }

    fn name_from_id(id: MuID, hint: &str) -> String {
        format!("@uvm.unnamed{}{}", hint, id)
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
}

