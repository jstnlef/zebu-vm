use ast::ir::*;
use ast::ptr::*;
use ast::inst::*;
use ast::op;
use ast::op::OpCode;
use ast::types;
use ast::types::*;
use vm::VM;
use runtime::mm;
use runtime::ValueLocation;
use runtime::thread;
use runtime::entrypoints;
use runtime::entrypoints::RuntimeEntrypoint;

use compiler::CompilerPass;
use compiler::backend::x86_64;
use compiler::backend::x86_64::CodeGenerator;
use compiler::backend::x86_64::ASMCodeGen;
use compiler::machine_code::CompiledFunction;
use compiler::frame::Frame;

use std::collections::HashMap;

pub struct InstructionSelection {
    name: &'static str,
    
    backend: Box<CodeGenerator>,
    
    current_callsite_id: usize,
    current_frame: Option<Frame>,
    current_block: Option<MuName>,
    current_func_start: Option<ValueLocation>,
    // key: block id, val: callsite that names the block as exception block
    current_exn_callsites: HashMap<MuID, Vec<ValueLocation>>,
    // key: block id, val: block location
    current_exn_blocks: HashMap<MuID, ValueLocation>     
}

impl <'a> InstructionSelection {
    pub fn new() -> InstructionSelection {
        InstructionSelection{
            name: "Instruction Selection (x64)",
            backend: Box::new(ASMCodeGen::new()),
            
            current_callsite_id: 0,
            current_frame: None,
            current_block: None,
            current_func_start: None,
            // key: block id, val: callsite that names the block as exception block
            current_exn_callsites: HashMap::new(), 
            current_exn_blocks: HashMap::new()
        }
    }
    
    // in this pass, we assume that
    // 1. all temporaries will use 64bit registers
    // 2. we do not need to backup/restore caller-saved registers
    // 3. we need to backup/restore all the callee-saved registers
    // if any of these assumption breaks, we will need to re-emit the code
    #[allow(unused_variables)]
    fn instruction_select(&mut self, node: &'a TreeNode, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) {
        trace!("instsel on node {}", node);
        
        match node.v {
            TreeNode_::Instruction(ref inst) => {
                match inst.v {
                    Instruction_::Branch2{cond, ref true_dest, ref false_dest, true_prob} => {
                        // move this to trace generation
                        // assert here
                        let (fallthrough_dest, branch_dest, branch_if_true) = {
                            if true_prob > 0.5f32 {
                                (true_dest, false_dest, false)
                            } else {
                                (false_dest, true_dest, true)
                            }
                        };
                        
                        let ops = inst.ops.read().unwrap();
                        
                        self.process_dest(&ops, fallthrough_dest, f_content, f_context, vm);
                        self.process_dest(&ops, branch_dest, f_content, f_context, vm);
                        
                        let branch_target = f_content.get_block(branch_dest.target).name().unwrap();
    
                        let ref cond = ops[cond];
                        
                        if self.match_cmp_res(cond) {
                            trace!("emit cmp_eq-branch2");
                            match self.emit_cmp_res(cond, f_content, f_context, vm) {
                                op::CmpOp::EQ => self.backend.emit_je(branch_target),
                                op::CmpOp::NE => self.backend.emit_jne(branch_target),
                                op::CmpOp::UGE => self.backend.emit_jae(branch_target),
                                op::CmpOp::UGT => self.backend.emit_ja(branch_target),
                                op::CmpOp::ULE => self.backend.emit_jbe(branch_target),
                                op::CmpOp::ULT => self.backend.emit_jb(branch_target),
                                op::CmpOp::SGE => self.backend.emit_jge(branch_target),
                                op::CmpOp::SGT => self.backend.emit_jg(branch_target),
                                op::CmpOp::SLE => self.backend.emit_jle(branch_target),
                                op::CmpOp::SLT => self.backend.emit_jl(branch_target),
                                _ => unimplemented!()
                            }
                        } else if self.match_ireg(cond) {
                            trace!("emit ireg-branch2");
                            
                            let cond_reg = self.emit_ireg(cond, f_content, f_context, vm);
                            
                            // emit: cmp cond_reg 1
                            self.backend.emit_cmp_r64_imm32(&cond_reg, 1);
                            // emit: je #branch_dest
                            self.backend.emit_je(branch_target);                            
                        } else {
                            unimplemented!();
                        }
                    },
                    
                    Instruction_::Branch1(ref dest) => {
                        let ops = inst.ops.read().unwrap();
                                            
                        self.process_dest(&ops, dest, f_content, f_context, vm);
                        
                        let target = f_content.get_block(dest.target).name().unwrap();
                        
                        trace!("emit branch1");
                        // jmp
                        self.backend.emit_jmp(target);
                    },
                    
                    Instruction_::ExprCall{ref data, is_abort} => {
                        if is_abort {
                            unimplemented!()
                        }
                        
                        self.emit_mu_call(
                            inst, // inst: &Instruction,
                            data, // calldata: &CallData,
                            None, // resumption: Option<&ResumptionData>,
                            node, // cur_node: &TreeNode, 
                            f_content, f_context, vm);                         
                    },
                    
                    Instruction_::Call{ref data, ref resume} => {
                        self.emit_mu_call(
                            inst, 
                            data, 
                            Some(resume), 
                            node, 
                            f_content, f_context, vm);
                    }
                    
                    Instruction_::Return(_) => {
                        self.emit_common_epilogue(inst, f_content, f_context, vm);
                        
                        self.backend.emit_ret();
                    },
                    
                    Instruction_::BinOp(op, op1, op2) => {
                        let ops = inst.ops.read().unwrap();
                        
                        match op {
                            op::BinOp::Add => {
                                if self.match_ireg(&ops[op1]) && self.match_ireg(&ops[op2]) {
                                    trace!("emit add-ireg-ireg");
                                    
                                    let reg_op1 = self.emit_ireg(&ops[op1], f_content, f_context, vm);
                                    let reg_op2 = self.emit_ireg(&ops[op2], f_content, f_context, vm);
                                    let res_tmp = self.emit_get_result(node);
                                    
                                    // mov op1, res
                                    self.backend.emit_mov_r64_r64(&res_tmp, &reg_op1);
                                    // add op2 res
                                    self.backend.emit_add_r64_r64(&res_tmp, &reg_op2);
                                } else if self.match_ireg(&ops[op1]) && self.match_iimm(&ops[op2]) {
                                    trace!("emit add-ireg-imm");
                                    
                                    let reg_op1 = self.emit_ireg(&ops[op1], f_content, f_context, vm);
                                    let reg_op2 = self.emit_get_iimm(&ops[op2]);
                                    let res_tmp = self.emit_get_result(node);
                                    
                                    // mov op1, res
                                    self.backend.emit_mov_r64_r64(&res_tmp, &reg_op1);
                                    // add op2, res
                                    self.backend.emit_add_r64_imm32(&res_tmp, reg_op2);
                                } else if self.match_iimm(&ops[op1]) && self.match_ireg(&ops[op2]) {
                                    trace!("emit add-imm-ireg");
                                    unimplemented!();
                                } else if self.match_ireg(&ops[op1]) && self.match_mem(&ops[op2]) {
                                    trace!("emit add-ireg-mem");
                                    
                                    let reg_op1 = self.emit_ireg(&ops[op1], f_content, f_context, vm);
                                    let reg_op2 = self.emit_mem(&ops[op2]);
                                    let res_tmp = self.emit_get_result(node);
                                    
                                    // mov op1, res
                                    self.backend.emit_mov_r64_r64(&res_tmp, &reg_op1);
                                    // add op2 res
                                    self.backend.emit_add_r64_mem64(&res_tmp, &reg_op2);
                                } else if self.match_mem(&ops[op1]) && self.match_ireg(&ops[op2]) {
                                    trace!("emit add-mem-ireg");
                                    unimplemented!();
                                } else {
                                    unimplemented!()
                                }
                            },
                            op::BinOp::Sub => {
                                if self.match_ireg(&ops[op1]) && self.match_ireg(&ops[op2]) {
                                    trace!("emit sub-ireg-ireg");
                                    
                                    let reg_op1 = self.emit_ireg(&ops[op1], f_content, f_context, vm);
                                    let reg_op2 = self.emit_ireg(&ops[op2], f_content, f_context, vm);
                                    let res_tmp = self.emit_get_result(node);
                                    
                                    // mov op1, res
                                    self.backend.emit_mov_r64_r64(&res_tmp, &reg_op1);
                                    // add op2 res
                                    self.backend.emit_sub_r64_r64(&res_tmp, &reg_op2);
                                } else if self.match_ireg(&ops[op1]) && self.match_iimm(&ops[op2]) {
                                    trace!("emit sub-ireg-imm");

                                    let reg_op1 = self.emit_ireg(&ops[op1], f_content, f_context, vm);
                                    let imm_op2 = self.emit_get_iimm(&ops[op2]);
                                    let res_tmp = self.emit_get_result(node);
                                    
                                    // mov op1, res
                                    self.backend.emit_mov_r64_r64(&res_tmp, &reg_op1);
                                    // add op2, res
                                    self.backend.emit_sub_r64_imm32(&res_tmp, imm_op2);
                                } else if self.match_iimm(&ops[op1]) && self.match_ireg(&ops[op2]) {
                                    trace!("emit sub-imm-ireg");
                                    unimplemented!();
                                } else if self.match_ireg(&ops[op1]) && self.match_mem(&ops[op2]) {
                                    trace!("emit sub-ireg-mem");
                                    
                                    let reg_op1 = self.emit_ireg(&ops[op1], f_content, f_context, vm);
                                    let mem_op2 = self.emit_mem(&ops[op2]);
                                    let res_tmp = self.emit_get_result(node);
                                    
                                    // mov op1, res
                                    self.backend.emit_mov_r64_r64(&res_tmp, &reg_op1);
                                    // sub op2 res
                                    self.backend.emit_sub_r64_mem64(&res_tmp, &mem_op2);
                                } else if self.match_mem(&ops[op1]) && self.match_ireg(&ops[op2]) {
                                    trace!("emit add-mem-ireg");
                                    unimplemented!();
                                } else {
                                    unimplemented!()
                                }
                            },
                            op::BinOp::Mul => {
                                // mov op1 -> rax
                                let rax = x86_64::RAX.clone();
                                let op1 = &ops[op1];
                                if self.match_ireg(op1) {
                                    let reg_op1 = self.emit_ireg(op1, f_content, f_context, vm);
                                    
                                    self.backend.emit_mov_r64_r64(&rax, &reg_op1);
                                } else if self.match_iimm(op1) {
                                    let imm_op1 = self.emit_get_iimm(op1);
                                    
                                    self.backend.emit_mov_r64_imm32(&rax, imm_op1);
                                } else if self.match_mem(op1) {
                                    let mem_op1 = self.emit_mem(op1);
                                    
                                    self.backend.emit_mov_r64_mem64(&rax, &mem_op1);
                                } else {
                                    unimplemented!();
                                }
                                
                                // mul op2 -> rax
                                let op2 = &ops[op2];
                                if self.match_ireg(op2) {
                                    let reg_op2 = self.emit_ireg(op2, f_content, f_context, vm);
                                    
                                    self.backend.emit_mul_r64(&reg_op2);
                                } else if self.match_iimm(op2) {
                                    let imm_op2 = self.emit_get_iimm(op2);
                                    
                                    // put imm in a temporary
                                    // here we use result reg as temporary
                                    let res_tmp = self.emit_get_result(node);
                                    self.backend.emit_mov_r64_imm32(&res_tmp, imm_op2);
                                    
                                    self.backend.emit_mul_r64(&res_tmp);
                                } else if self.match_mem(op2) {
                                    let mem_op2 = self.emit_mem(op2);
                                    
                                    self.backend.emit_mul_mem64(&mem_op2);
                                } else {
                                    unimplemented!();
                                }
                                
                                // mov rax -> result
                                let res_tmp = self.emit_get_result(node);
                                self.backend.emit_mov_r64_r64(&res_tmp, &rax);
                            },
                            
                            _ => unimplemented!()
                        }
                    }
                    
                    // load on x64 generates mov inst (no matter what order is specified)
                    // https://www.cl.cam.ac.uk/~pes20/cpp/cpp0xmappings.html
                    Instruction_::Load{is_ptr, order, mem_loc} => {
                        let ops = inst.ops.read().unwrap();
                        let ref loc_op = ops[mem_loc];
                        
                        // check order
                        match order {
                            MemoryOrder::Relaxed 
                            | MemoryOrder::Consume 
                            | MemoryOrder::Acquire
                            | MemoryOrder::SeqCst => {},
                            _ => panic!("didnt expect order {:?} with store inst", order)
                        }                        

                        let resolved_loc = self.emit_get_mem(loc_op, vm);                        
                        let res_temp = self.emit_get_result(node);
                        
                        if self.match_ireg(node) {
                            // emit mov(GPR)
                            self.backend.emit_mov_r64_mem64(&res_temp, &resolved_loc);
                        } else {
                            // emit mov(FPR)
                            unimplemented!()
                        }
                    }
                    
                    Instruction_::Store{is_ptr, order, mem_loc, value} => {
                        let ops = inst.ops.read().unwrap();
                        let ref loc_op = ops[mem_loc];
                        let ref val_op = ops[value];
                        
                        let generate_plain_mov : bool = {
                            match order {
                                MemoryOrder::Relaxed | MemoryOrder::Release => true,
                                MemoryOrder::SeqCst => false,
                                _ => panic!("didnt expect order {:?} with store inst", order)
                            }
                        };
                        
                        let resolved_loc = self.emit_get_mem(loc_op, vm);
                        
                        if self.match_ireg(val_op) {
                            let val = self.emit_ireg(val_op, f_content, f_context, vm);
                            if generate_plain_mov {
                                self.backend.emit_mov_mem64_r64(&resolved_loc, &val);
                            } else {
                                unimplemented!()
                            }
                        } else if self.match_iimm(val_op) {
                            let val = self.emit_get_iimm(val_op);
                            if generate_plain_mov {
                                self.backend.emit_mov_mem64_imm32(&resolved_loc, val);
                            } else {
                                unimplemented!()
                            }
                        } else {
                            // emit mov(FPR)
                            unimplemented!()
                        }
                    }
                    
                    Instruction_::GetIRef(op_index) => {
                        let ops = inst.ops.read().unwrap();
                        
                        let ref op = ops[op_index];
                        let res_tmp = self.emit_get_result(node);
                        
                        let hdr_size = mm::objectmodel::OBJECT_HEADER_SIZE;
                        if hdr_size == 0 {
                            self.emit_general_move(&op, &res_tmp, f_content, f_context, vm);
                        } else {
                            self.emit_lea_base_offset(&res_tmp, &op.clone_value(), hdr_size as i32, vm);
                        }
                    }
                    
                    Instruction_::ThreadExit => {
                        // emit a call to swap_back_to_native_stack(sp_loc: Address)
                        
                        // get thread local and add offset to get sp_loc
                        let tl = self.emit_get_threadlocal(Some(node), f_content, f_context, vm);
                        self.backend.emit_add_r64_imm32(&tl, *thread::NATIVE_SP_LOC_OFFSET as i32);
                        
                        self.emit_runtime_entry(&entrypoints::SWAP_BACK_TO_NATIVE_STACK, vec![tl.clone()], None, Some(node), f_content, f_context, vm);
                    }
                    
                    Instruction_::New(ref ty) => {
                        let ty_info = vm.get_backend_type_info(ty.id());
                        let ty_size = ty_info.size;
                        let ty_align= ty_info.alignment;
                        
                        if ty_size > mm::LARGE_OBJECT_THRESHOLD {
                            // emit large object allocation
                            unimplemented!()
                        } else {
                            // emit immix allocation fast path
                            
                            // ASM: %tl = get_thread_local()
                            let tmp_tl = self.emit_get_threadlocal(Some(node), f_content, f_context, vm);
                            
                            // ASM: mov [%tl + allocator_offset + cursor_offset] -> %cursor
                            let cursor_offset = *thread::ALLOCATOR_OFFSET + *mm::ALLOCATOR_CURSOR_OFFSET;
                            let tmp_cursor = self.make_temporary(f_context, ADDRESS_TYPE.clone(), vm);
                            self.emit_load_base_offset(&tmp_cursor, &tmp_tl, cursor_offset as i32, vm);
                            
                            // alignup cursor (cursor + align - 1 & !(align - 1))
                            // ASM: lea align-1(%cursor) -> %start
                            let align = ty_info.alignment as i32;
                            let tmp_start = self.make_temporary(f_context, ADDRESS_TYPE.clone(), vm);
                            self.emit_lea_base_offset(&tmp_start, &tmp_cursor, align - 1, vm);
                            // ASM: and %start, !(align-1) -> %start
                            self.backend.emit_and_r64_imm32(&tmp_start, !(align - 1));
                            
                            // bump cursor
                            // ASM: lea size(%start) -> %end
                            let tmp_end = self.make_temporary(f_context, ADDRESS_TYPE.clone(), vm);
                            self.emit_lea_base_offset(&tmp_end, &tmp_start, ty_size as i32, vm);
                            
                            // check with limit
                            // ASM: cmp %end, [%tl + allocator_offset + limit_offset]
                            let limit_offset = *thread::ALLOCATOR_OFFSET + *mm::ALLOCATOR_LIMIT_OFFSET;
                            let mem_limit = self.make_memory_op_base_offset(&tmp_tl, limit_offset as i32, ADDRESS_TYPE.clone(), vm);
                            self.backend.emit_cmp_r64_mem64(&tmp_end, &mem_limit);
                            
                            // branch to slow path if end > limit
                            // ASM: jl alloc_slow
                            let slowpath = format!("{}_allocslow", node.id());
                            self.backend.emit_jl(slowpath.clone());
                            
                            // update cursor
                            // ASM: mov %end -> [%tl + allocator_offset + cursor_offset]
                            self.emit_store_base_offset(&tmp_tl, cursor_offset as i32, &tmp_end, vm);
                            
                            // put start as result
                            // ASM: mov %start -> %result
                            let tmp_res = self.emit_get_result(node);
                            self.backend.emit_mov_r64_r64(&tmp_res, &tmp_start);
                            
                            // ASM jmp alloc_end
                            let allocend = format!("{}_allocend", node.id());
                            self.backend.emit_jmp(allocend.clone());
                            
                            // finishing current block
                            self.backend.end_block(self.current_block.as_ref().unwrap().clone());
                            
                            // alloc_slow: 
                            // call alloc_slow(size, align) -> %ret
                            // new block (no livein)
                            self.current_block = Some(slowpath.clone());
                            self.backend.start_block(slowpath.clone());
                            self.backend.set_block_livein(slowpath.clone(), &vec![]); 

                            // arg1: allocator address                            
                            let allocator_offset = *thread::ALLOCATOR_OFFSET;
                            let tmp_allocator = self.make_temporary(f_context, ADDRESS_TYPE.clone(), vm);
                            self.emit_lea_base_offset(&tmp_allocator, &tmp_tl, allocator_offset as i32, vm);
                            // arg2: size                            
                            let const_size = self.make_value_int_const(ty_size as u64, vm);
                            // arg3: align
                            let const_align= self.make_value_int_const(ty_align as u64, vm);
                            
                            let rets = self.emit_runtime_entry(
                                &entrypoints::ALLOC_SLOW,
                                vec![tmp_allocator, const_size, const_align],
                                Some(vec![
                                    tmp_res.clone()
                                ]),
                                Some(node), f_content, f_context, vm
                            );
                            
                            // end block (no liveout other than result)
                            self.backend.end_block(slowpath.clone());
                            self.backend.set_block_liveout(slowpath.clone(), &vec![tmp_res.clone()]);
                            
                            // block: alloc_end
                            self.backend.start_block(allocend.clone());
                            self.current_block = Some(allocend.clone());
                        }
                    }
                    
                    Instruction_::Throw(op_index) => {
                        let ops = inst.ops.read().unwrap();
                        let ref exception_obj = ops[op_index];
                        
                        self.emit_runtime_entry(
                            &entrypoints::THROW_EXCEPTION, 
                            vec![exception_obj.clone_value()], 
                            None,
                            Some(node), f_content, f_context, vm);
                    }
    
                    _ => unimplemented!()
                } // main switch
            },
            
            TreeNode_::Value(ref p) => {
        
            }
        }
    }
    
    fn make_temporary(&mut self, f_context: &mut FunctionContext, ty: P<MuType>, vm: &VM) -> P<Value> {
        f_context.make_temporary(vm.next_id(), ty).clone_value()
    }
    
    fn make_memory_op_base_offset (&mut self, base: &P<Value>, offset: i32, ty: P<MuType>, vm: &VM) -> P<Value> {
        P(Value{
            hdr: MuEntityHeader::unnamed(vm.next_id()),
            ty: ty.clone(),
            v: Value_::Memory(MemoryLocation::Address{
                base: base.clone(),
                offset: Some(self.make_value_int_const(offset as u64, vm)),
                index: None,
                scale: None
            })
        })
    }
    
    fn make_value_int_const (&mut self, val: u64, vm: &VM) -> P<Value> {
        P(Value{
            hdr: MuEntityHeader::unnamed(vm.next_id()),
            ty: UINT64_TYPE.clone(),
            v: Value_::Constant(Constant::Int(val))
        })
    } 
    
    fn emit_load_base_offset (&mut self, dest: &P<Value>, base: &P<Value>, offset: i32, vm: &VM) {
        let mem = self.make_memory_op_base_offset(base, offset, dest.ty.clone(), vm);
        
        self.backend.emit_mov_r64_mem64(dest, &mem);
    }
    
    fn emit_store_base_offset (&mut self, base: &P<Value>, offset: i32, src: &P<Value>, vm: &VM) {
        let mem = self.make_memory_op_base_offset(base, offset, src.ty.clone(), vm);
        
        self.backend.emit_mov_mem64_r64(&mem, src);
    }
    
    fn emit_lea_base_offset (&mut self, dest: &P<Value>, base: &P<Value>, offset: i32, vm: &VM) {
        let mem = self.make_memory_op_base_offset(base, offset, ADDRESS_TYPE.clone(), vm);
        
        self.backend.emit_lea_r64(dest, &mem);
    }
    
    fn emit_get_threadlocal (
        &mut self, 
        cur_node: Option<&TreeNode>,
        f_content: &FunctionContent, 
        f_context: &mut FunctionContext, 
        vm: &VM) -> P<Value> {
        let mut rets = self.emit_runtime_entry(&entrypoints::GET_THREAD_LOCAL, vec![], None, cur_node, f_content, f_context, vm);
        
        rets.pop().unwrap()
    }
    
    // ret: Option<Vec<P<Value>>
    // if ret is Some, return values will put stored in given temporaries
    // otherwise create temporaries
    // always returns result temporaries (given or created)
    fn emit_runtime_entry (
        &mut self, 
        entry: &RuntimeEntrypoint, 
        args: Vec<P<Value>>, 
        rets: Option<Vec<P<Value>>>,
        cur_node: Option<&TreeNode>, 
        f_content: &FunctionContent, 
        f_context: &mut FunctionContext, 
        vm: &VM) -> Vec<P<Value>> {
        let sig = entry.sig.clone();
        
        let entry_name = {
            if vm.is_running() {
                unimplemented!()
            } else {
                let ref entry_loc = entry.aot;
                
                match entry_loc {
                    &ValueLocation::Relocatable(_, ref name) => name.clone(),
                    _ => panic!("expecting a relocatable value")
                }
            }
        };
        
        self.emit_c_call(entry_name, sig, args, rets, cur_node, f_content, f_context, vm)
    }
    
    #[allow(unused_variables)]
    // ret: Option<Vec<P<Value>>
    // if ret is Some, return values will put stored in given temporaries
    // otherwise create temporaries
    // always returns result temporaries (given or created)
    fn emit_c_call (
        &mut self, 
        func_name: CName, 
        sig: P<CFuncSig>, 
        args: Vec<P<Value>>, 
        rets: Option<Vec<P<Value>>>,
        cur_node: Option<&TreeNode>,
        f_content: &FunctionContent, 
        f_context: &mut FunctionContext, 
        vm: &VM) -> Vec<P<Value>> 
    {
        let mut gpr_arg_count = 0;
        for arg in args.iter() {
            if arg.is_int_reg() {
                if gpr_arg_count < x86_64::ARGUMENT_GPRs.len() {
                    self.backend.emit_mov_r64_r64(&x86_64::ARGUMENT_GPRs[gpr_arg_count], &arg);
                    gpr_arg_count += 1;
                } else {
                    // use stack to pass argument
                    unimplemented!()
                }
            } else if arg.is_int_const() {
                if x86_64::is_valid_x86_imm(arg) {                
                    let int_const = arg.extract_int_const() as i32;
                    
                    if gpr_arg_count < x86_64::ARGUMENT_GPRs.len() {
                        self.backend.emit_mov_r64_imm32(&x86_64::ARGUMENT_GPRs[gpr_arg_count], int_const);
                        gpr_arg_count += 1;
                    } else {
                        // use stack to pass argument
                        unimplemented!()
                    }
                } else {
                    // put the constant to memory
                    unimplemented!()
                }
            } else if arg.is_mem() {
                if gpr_arg_count < x86_64::ARGUMENT_GPRs.len() {
                    self.backend.emit_mov_r64_mem64(&x86_64::ARGUMENT_GPRs[gpr_arg_count], &arg);
                    gpr_arg_count += 1;
                } else {
                    // use stack to pass argument
                    unimplemented!()
                }
            } else {
                // floating point
                unimplemented!()
            }
        }
        
        // make call
        if vm.is_running() {
            unimplemented!()
        } else {
            let callsite = self.new_callsite_label(cur_node);
            self.backend.emit_call_near_rel32(callsite, func_name);
            
            // record exception block (CCall may have an exception block)
            if cur_node.is_some() {
                let cur_node = cur_node.unwrap(); 
                if cur_node.op == OpCode::CCall {
                    unimplemented!()
                }
            }
        }
        
        // deal with ret vals
        let mut return_vals = vec![];
        
        let mut gpr_ret_count = 0;
        for ret_index in 0..sig.ret_tys.len() {
            let ref ty = sig.ret_tys[ret_index];
            
            let ret_val = match rets {
                Some(ref rets) => rets[ret_index].clone(),
                None => {
                    let tmp_node = f_context.make_temporary(vm.next_id(), ty.clone());
                    tmp_node.clone_value()
                }
            };
            
            if ret_val.is_int_reg() {
                if gpr_ret_count < x86_64::RETURN_GPRs.len() {
                    self.backend.emit_mov_r64_r64(&ret_val, &x86_64::RETURN_GPRs[gpr_ret_count]);
                    gpr_ret_count += 1;
                } else {
                    // get return value by stack
                    unimplemented!()
                }
            } else {
                // floating point register
                unimplemented!()
            }
            
            return_vals.push(ret_val);            
        }
        
        return_vals
    }
    
    fn emit_mu_call(
        &mut self,
        inst: &Instruction,
        calldata: &CallData,
        resumption: Option<&ResumptionData>,
        cur_node: &TreeNode, 
        f_content: &FunctionContent, 
        f_context: &mut FunctionContext, 
        vm: &VM) {
        trace!("deal with pre-call convention");
        
        let ops = inst.ops.read().unwrap();
        let ref func = ops[calldata.func];
        let ref func_sig = match func.v {
            TreeNode_::Value(ref pv) => {
                let ty : &MuType = &pv.ty;
                match ty.v {
                    MuType_::FuncRef(ref sig)
                    | MuType_::UFuncPtr(ref sig) => sig,
                    _ => panic!("expected funcref/ptr type")
                }
            },
            _ => panic!("expected funcref/ptr type")
        };
        
        debug_assert!(func_sig.ret_tys.len() == calldata.args.len());
        if cfg!(debug_assertions) {
            if inst.value.is_some() {
                assert!(func_sig.arg_tys.len() == inst.value.as_ref().unwrap().len());
            } else {
                assert!(func_sig.arg_tys.len() == 0);
            }
        }
                                
        let mut gpr_arg_count = 0;
        // TODO: let mut fpr_arg_count = 0;
        for arg_index in calldata.args.iter() {
            let ref arg = ops[*arg_index];
            trace!("arg {}", arg);
            
            if self.match_ireg(arg) {
                let arg = self.emit_ireg(arg, f_content, f_context, vm);
                
                if gpr_arg_count < x86_64::ARGUMENT_GPRs.len() {
                    self.backend.emit_mov_r64_r64(&x86_64::ARGUMENT_GPRs[gpr_arg_count], &arg);
                    gpr_arg_count += 1;
                } else {
                    // use stack to pass argument
                    unimplemented!();
                }
            } else if self.match_iimm(arg) {
                let arg = self.emit_get_iimm(arg);
                
                if gpr_arg_count < x86_64::ARGUMENT_GPRs.len() {
                    self.backend.emit_mov_r64_imm32(&x86_64::ARGUMENT_GPRs[gpr_arg_count], arg);
                    gpr_arg_count += 1;
                } else {
                    // use stack to pass argument
                    unimplemented!();
                }
            } else {
                unimplemented!();
            }
        }
        
        trace!("genearting call inst");
        // check direct call or indirect
        let callsite = {
            if self.match_funcref_const(func) {
                let target_id = self.emit_get_funcref_const(func);
                let funcs = vm.funcs().read().unwrap();
                let target = funcs.get(&target_id).unwrap().read().unwrap();
                                            
                if vm.is_running() {
                    unimplemented!()
                } else {
                    let callsite = self.new_callsite_label(Some(cur_node));
                    self.backend.emit_call_near_rel32(callsite, target.name().unwrap())
                }
            } else if self.match_ireg(func) {
                let target = self.emit_ireg(func, f_content, f_context, vm);
                
                let callsite = self.new_callsite_label(Some(cur_node));
                self.backend.emit_call_near_r64(callsite, &target)
            } else if self.match_mem(func) {
                let target = self.emit_mem(func);
                
                let callsite = self.new_callsite_label(Some(cur_node));
                self.backend.emit_call_near_mem64(callsite, &target)
            } else {
                unimplemented!()
            }
        };
        
        // record exception branch
        if resumption.is_some() {
            let ref exn_dest = resumption.as_ref().unwrap().exn_dest;
            let target_block = exn_dest.target;
            
            if self.current_exn_callsites.contains_key(&target_block) {
                let callsites = self.current_exn_callsites.get_mut(&target_block).unwrap();
                callsites.push(callsite);
            } else {
                let mut callsites = vec![];
                callsites.push(callsite);
                self.current_exn_callsites.insert(target_block, callsites);
            } 
        }
        
        // deal with ret vals
        if inst.value.is_some() {
            let rets = inst.value.as_ref().unwrap();
            trace!("deal with return values");
            let mut gpr_ret_count = 0;
            // TODO: let mut fpr_ret_count = 0;
            for val in rets {
                if val.is_int_reg() {
                    if gpr_ret_count < x86_64::RETURN_GPRs.len() {
                        self.backend.emit_mov_r64_r64(&val, &x86_64::RETURN_GPRs[gpr_ret_count]);
                        gpr_ret_count += 1;
                    } else {
                        // get return value by stack
                        unimplemented!();
                    }
                } else {
                    // floating point register
                    unimplemented!();
                }
            }
        } else {
            trace!("no return value");
        }

    }
    
    #[allow(unused_variables)]
    fn process_dest(&mut self, ops: &Vec<P<TreeNode>>, dest: &Destination, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) {
        for i in 0..dest.args.len() {
            let ref dest_arg = dest.args[i];
            match dest_arg {
                &DestArg::Normal(op_index) => {
                    let ref arg = ops[op_index];
//                    match arg.op {
//                        OpCode::RegI64 
//                        | OpCode::RegFP
//                        | OpCode::IntImmI64
//                        | OpCode::FPImm => {
//                            // do nothing
//                        },
//                        _ => {
//                            trace!("nested: compute arg for branch");
//                            // nested: compute arg
//                            self.instruction_select(arg, cur_func);
//                            
//                            self.emit_get_result(arg);
//                        }
//                    }
//                    
                    let ref target_args = f_content.get_block(dest.target).content.as_ref().unwrap().args;
                    let ref target_arg = target_args[i];
                    
                    self.emit_general_move(&arg, target_arg, f_content, f_context, vm);
                },
                &DestArg::Freshbound(_) => unimplemented!()
            }
        }
    }
    
    fn emit_common_prologue(&mut self, args: &Vec<P<Value>>, vm: &VM) {
        let block_name = "prologue".to_string();
        self.backend.start_block(block_name.clone());
        
        // no livein
        // liveout = entry block's args
        self.backend.set_block_livein(block_name.clone(), &vec![]);
        self.backend.set_block_liveout(block_name.clone(), args);
        
        // push rbp
        self.backend.emit_push_r64(&x86_64::RBP);
        // mov rsp -> rbp
        self.backend.emit_mov_r64_r64(&x86_64::RBP, &x86_64::RSP);
        
        // push all callee-saved registers
        {
            let frame = self.current_frame.as_mut().unwrap();
            for i in 0..x86_64::CALLEE_SAVED_GPRs.len() {
                let ref reg = x86_64::CALLEE_SAVED_GPRs[i];
                // not pushing rbp (as we have done taht)
                if reg.extract_ssa_id().unwrap() != x86_64::RBP.extract_ssa_id().unwrap() {
                    self.backend.emit_push_r64(&reg);
                    frame.alloc_slot_for_callee_saved_reg(reg.clone(), vm);
                }
            }
        }
        
        // unload arguments
        let mut gpr_arg_count = 0;
        // TODO: let mut fpr_arg_count = 0;
        for arg in args {
            if arg.is_int_reg() {
                if gpr_arg_count < x86_64::ARGUMENT_GPRs.len() {
                    self.backend.emit_mov_r64_r64(&arg, &x86_64::ARGUMENT_GPRs[gpr_arg_count]);
                    gpr_arg_count += 1;
                } else {
                    // unload from stack
                    unimplemented!();
                }
            } else if arg.is_fp_reg() {
                unimplemented!();
            } else {
                panic!("expect an arg value to be either int reg or fp reg");
            }
        }
        
        self.backend.end_block(block_name);
    }
    
    fn emit_common_epilogue(&mut self, ret_inst: &Instruction, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) {
        // epilogue is not a block (its a few instruction inserted before return)
        // FIXME: this may change in the future
        
        // prepare return regs
        let ref ops = ret_inst.ops.read().unwrap();
        let ret_val_indices = match ret_inst.v {
            Instruction_::Return(ref vals) => vals,
            _ => panic!("expected ret inst")
        };
        
        let mut gpr_ret_count = 0;
        // TODO: let mut fpr_ret_count = 0;
        for i in ret_val_indices {
            let ref ret_val = ops[*i];
            if self.match_ireg(ret_val) {
                let reg_ret_val = self.emit_ireg(ret_val, f_content, f_context, vm);
                
                self.backend.emit_mov_r64_r64(&x86_64::RETURN_GPRs[gpr_ret_count], &reg_ret_val);
                gpr_ret_count += 1;
            } else if self.match_iimm(ret_val) {
                let imm_ret_val = self.emit_get_iimm(ret_val);
                
                self.backend.emit_mov_r64_imm32(&x86_64::RETURN_GPRs[gpr_ret_count], imm_ret_val);
                gpr_ret_count += 1;
            } else {
                unimplemented!();
            }
        }        
        
        // pop all callee-saved registers - reverse order
        for i in (0..x86_64::CALLEE_SAVED_GPRs.len()).rev() {
            let ref reg = x86_64::CALLEE_SAVED_GPRs[i];
            if reg.extract_ssa_id().unwrap() != x86_64::RBP.extract_ssa_id().unwrap() {
                self.backend.emit_pop_r64(&reg);
            }
        }
        
        // pop rbp
        self.backend.emit_pop_r64(&x86_64::RBP);
    }
    
    fn match_cmp_res(&mut self, op: &P<TreeNode>) -> bool {
        match op.v {
            TreeNode_::Instruction(ref inst) => {
                match inst.v {
                    Instruction_::CmpOp(_, _, _) => true,
                    _ => false
                }
            }
            TreeNode_::Value(_) => false
        }
    }
    
    fn emit_cmp_res(&mut self, cond: &P<TreeNode>, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) -> op::CmpOp {
        match cond.v {
            TreeNode_::Instruction(ref inst) => {
                let ops = inst.ops.read().unwrap();                
                
                match inst.v {
                    Instruction_::CmpOp(op, op1, op2) => {
                        let op1 = &ops[op1];
                        let op2 = &ops[op2];
                        
                        if op::is_int_cmp(op) {                        
                            if self.match_ireg(op1) && self.match_ireg(op2) {
                                let reg_op1 = self.emit_ireg(op1, f_content, f_context, vm);
                                let reg_op2 = self.emit_ireg(op2, f_content, f_context, vm);
                                
                                self.backend.emit_cmp_r64_r64(&reg_op1, &reg_op2);
                            } else if self.match_ireg(op1) && self.match_iimm(op2) {
                                let reg_op1 = self.emit_ireg(op1, f_content, f_context, vm);
                                let iimm_op2 = self.emit_get_iimm(op2);
                                
                                self.backend.emit_cmp_r64_imm32(&reg_op1, iimm_op2);
                            } else {
                                unimplemented!()
                            }
                        } else {
                            unimplemented!()
                        }
                        
                        op
                    }
                    
                    _ => panic!("expect cmp res to emit")
                }
            }
            _ => panic!("expect cmp res to emit")
        }
    }    
    
    fn match_ireg(&mut self, op: &TreeNode) -> bool {
        match op.v {
            TreeNode_::Instruction(ref inst) => {
                if inst.value.is_some() {
                    if inst.value.as_ref().unwrap().len() > 1 {
                        return false;
                    }
                    
                    let ref value = inst.value.as_ref().unwrap()[0];
                    
                    if types::is_scalar(&value.ty) {
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            
            TreeNode_::Value(ref pv) => {
                pv.is_int_reg()
            }
        }
    }
    
    fn emit_ireg(&mut self, op: &P<TreeNode>, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) -> P<Value> {
        match op.v {
            TreeNode_::Instruction(_) => {
                self.instruction_select(op, f_content, f_context, vm);
                
                self.emit_get_result(op)
            },
            TreeNode_::Value(ref pv) => {
                match pv.v {
                    Value_::Constant(_)
                    | Value_::Global(_)
                    | Value_::Memory(_) => panic!("expected ireg"),
                    Value_::SSAVar(_) => {
                        pv.clone()
                    },
                }
            }
        }
    }
    
    #[allow(unused_variables)]
    fn match_fpreg(&mut self, op: &P<TreeNode>) -> bool {
        unimplemented!()
    }
    
    fn match_iimm(&mut self, op: &P<TreeNode>) -> bool {
        match op.v {
            TreeNode_::Value(ref pv) if x86_64::is_valid_x86_imm(pv) => true,
            _ => false
        }
    }
    
    fn emit_get_iimm(&mut self, op: &P<TreeNode>) -> i32 {
        match op.v {
            TreeNode_::Value(ref pv) => {
                match pv.v {
                    Value_::Constant(Constant::Int(val)) => {
                        val as i32
                    },
                    _ => panic!("expected iimm")
                }
            },
            _ => panic!("expected iimm")
        }
    }
    
    fn emit_get_mem(&mut self, op: &P<TreeNode>, vm: &VM) -> P<Value> {
        match op.v {
            TreeNode_::Value(ref pv) => {
                match pv.v {
                    Value_::SSAVar(_) => P(Value{
                        hdr: MuEntityHeader::unnamed(vm.next_id()),
                        ty: types::get_referent_ty(& pv.ty).unwrap(),
                        v: Value_::Memory(MemoryLocation::Address{
                            base: pv.clone(),
                            offset: None,
                            index: None,
                            scale: None
                        })
                    }),
                    Value_::Global(_) => {
                        if vm.is_running() {
                            // get address from vm
                            unimplemented!()
                        } else {
                            // symbolic
                            P(Value{
                                hdr: MuEntityHeader::unnamed(vm.next_id()),
                                ty: types::get_referent_ty(&pv.ty).unwrap(),
                                v: Value_::Memory(MemoryLocation::Symbolic{
                                    base: Some(x86_64::RIP.clone()),
                                    label: pv.name().unwrap()
                                })
                            })
                        }
                    },
                    Value_::Memory(_) => pv.clone(),
                    Value_::Constant(_) => unimplemented!()
                }
            }
            TreeNode_::Instruction(_) => self.emit_get_mem_from_inst(op, vm)
        }
    }
    
    fn emit_get_mem_from_inst(&mut self, op: &P<TreeNode>, vm: &VM) -> P<Value> {
        match op.v {
            TreeNode_::Instruction(ref inst) => {
                let ref ops = inst.ops.read().unwrap();
                
                match inst.v {
                    Instruction_::GetIRef(op_index) => {
                        let ref op = ops[op_index];
                        
                        self.make_memory_op_base_offset(&op.clone_value(), mm::objectmodel::OBJECT_HEADER_SIZE as i32, ADDRESS_TYPE.clone(), vm) 
                    }
                    _ => unimplemented!()
                }
            },
            _ => panic!("expecting a instruction that yields a memory address")
        }
    }
    
    fn match_funcref_const(&mut self, op: &P<TreeNode>) -> bool {
        match op.v {
            TreeNode_::Value(ref pv) => {
                match pv.v {
                    Value_::Constant(Constant::FuncRef(_)) => true,
                    Value_::Constant(Constant::UFuncRef(_)) => true,
                    _ => false
                }
            },
            _ => false 
        }
    }
    
    fn emit_get_funcref_const(&mut self, op: &P<TreeNode>) -> MuID {
        match op.v {
            TreeNode_::Value(ref pv) => {
                match pv.v {
                    Value_::Constant(Constant::FuncRef(id))
                    | Value_::Constant(Constant::UFuncRef(id)) => id,
                    _ => panic!("expected a (u)funcref const")
                }
            },
            _ => panic!("expected a (u)funcref const")
        }
    }
    
    #[allow(unused_variables)]
    fn match_mem(&mut self, op: &P<TreeNode>) -> bool {
        unimplemented!()
    }
    
    #[allow(unused_variables)]
    fn emit_mem(&mut self, op: &P<TreeNode>) -> P<Value> {
        unimplemented!()
    }
    
    fn emit_get_result(&mut self, node: &TreeNode) -> P<Value> {
        match node.v {
            TreeNode_::Instruction(ref inst) => {
                if inst.value.is_some() {
                    if inst.value.as_ref().unwrap().len() > 1 {
                        panic!("expected ONE result from the node {}", node);
                    }
                    
                    let ref value = inst.value.as_ref().unwrap()[0];
                    
                    value.clone()
                } else {
                    panic!("expected result from the node {}", node);
                }
            }
            
            TreeNode_::Value(ref pv) => {
                pv.clone()
            }
        }
    }
    
    fn emit_general_move(&mut self, src: &P<TreeNode>, dest: &P<Value>, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) {
        let ref dst_ty = dest.ty;
        
        if !types::is_fp(dst_ty) && types::is_scalar(dst_ty) {
            if self.match_ireg(src) {
                let src_reg = self.emit_ireg(src, f_content, f_context, vm);
                self.backend.emit_mov_r64_r64(dest, &src_reg);
            } else if self.match_iimm(src) {
                let src_imm = self.emit_get_iimm(src);
                self.backend.emit_mov_r64_imm32(dest, src_imm);
            } else {
                panic!("expected an int type op");
            }
        } else if !types::is_fp(dst_ty) && types::is_scalar(dst_ty) {
            unimplemented!()
        } else {
            panic!("unexpected type for move");
        } 
    }
    
    fn emit_landingpad(&mut self, exception_arg: &P<Value>, f_content: &FunctionContent, f_context: &mut FunctionContext, vm: &VM) {
        // get thread local and add offset to get exception_obj
        let tl = self.emit_get_threadlocal(None, f_content, f_context, vm);
        self.emit_load_base_offset(exception_arg, &tl, *thread::EXCEPTION_OBJ_OFFSET as i32, vm);
    }
    
    fn new_callsite_label(&mut self, cur_node: Option<&TreeNode>) -> String {
        let ret = {
            if cur_node.is_some() {
                format!("callsite_{}_{}", cur_node.unwrap().id(), self.current_callsite_id)
            } else {
                format!("callsite_anon_{}", self.current_callsite_id)
            }
        };
        self.current_callsite_id += 1;
        ret
    }
}

impl CompilerPass for InstructionSelection {
    fn name(&self) -> &'static str {
        self.name
    }

    #[allow(unused_variables)]
    fn start_function(&mut self, vm: &VM, func_ver: &mut MuFunctionVersion) {
        debug!("{}", self.name());
        
        self.current_frame = Some(Frame::new(func_ver.id()));
        self.current_func_start = Some({
            let funcs = vm.funcs().read().unwrap();
            let func = funcs.get(&func_ver.func_id).unwrap().read().unwrap();
            self.backend.start_code(func.name().unwrap())        
        });
        self.current_callsite_id = 0;
        self.current_exn_callsites.clear();
        self.current_exn_blocks.clear();
        
        // prologue (get arguments from entry block first)        
        let entry_block = func_ver.content.as_ref().unwrap().get_entry_block();
        let ref args = entry_block.content.as_ref().unwrap().args;
        self.emit_common_prologue(args, vm);
    }

    #[allow(unused_variables)]
    fn visit_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        let f_content = func.content.as_ref().unwrap();
        
        for block_id in func.block_trace.as_ref().unwrap() {
            let block = f_content.get_block(*block_id);
            let block_label = block.name().unwrap();
            self.current_block = Some(block_label.clone());            
            
            let block_content = block.content.as_ref().unwrap();
            
            if block.is_exception_block() {
                let loc = self.backend.start_exception_block(block_label.clone());
                self.current_exn_blocks.insert(block.id(), loc);
                
                let exception_arg = block_content.exn_arg.as_ref().unwrap();
                
                // live in is args of the block + exception arg
                let mut livein = block_content.args.to_vec();
                livein.push(exception_arg.clone());
                self.backend.set_block_livein(block_label.clone(), &livein);
                
                // need to insert a landing pad
                self.emit_landingpad(&exception_arg, f_content, &mut func.context, vm);
            } else {
                self.backend.start_block(block_label.clone());
                
                // live in is args of the block
                self.backend.set_block_livein(block_label.clone(), &block_content.args);                    
            }
            
            // live out is the union of all branch args of this block
            let live_out = block_content.get_out_arguments();

            for inst in block_content.body.iter() {
                self.instruction_select(&inst, f_content, &mut func.context, vm);
            }
            
            // we may start block a, and end with block b (instruction selection may create blocks)
            // we set liveout to current block 
            {
                let current_block = self.current_block.as_ref().unwrap();
                self.backend.set_block_liveout(current_block.clone(), &live_out);
                self.backend.end_block(current_block.clone());
            }            
            self.current_block = None;
        }
    }
    
    #[allow(unused_variables)]
    fn finish_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        self.backend.print_cur_code();
        
        let func_name = {
            let funcs = vm.funcs().read().unwrap();
            let func = funcs.get(&func.func_id).unwrap().read().unwrap();
            func.name().unwrap()
        };
        
        let (mc, func_end) = self.backend.finish_code(func_name);
        
        // insert exception branch info
        let mut frame = self.current_frame.take().unwrap();
        for block_id in self.current_exn_blocks.keys() {
            let block_loc = self.current_exn_blocks.get(&block_id).unwrap();
            let callsites = self.current_exn_callsites.get(&block_id).unwrap();
            
            for callsite in callsites {
                frame.add_exception_callsite(callsite.clone(), block_loc.clone());
            }
        }
        
        let compiled_func = CompiledFunction {
            func_id: func.func_id,
            func_ver_id: func.id(),
            temps: HashMap::new(),
            mc: Some(mc),
            frame: frame,
            start: self.current_func_start.take().unwrap(),
            end: func_end 
        };
        
        vm.add_compiled_func(compiled_func);
    }
}
