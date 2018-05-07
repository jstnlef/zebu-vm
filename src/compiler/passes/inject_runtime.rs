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

use ast::inst::*;
use ast::ir::*;
use ast::op::*;
use ast::ptr::*;
use ast::types::*;
use compiler::CompilerPass;
use runtime::entrypoints;
use runtime::mm;
use runtime::mm::*;
use runtime::thread;
use std::any::Any;
use utils::*;
use utils::math;
use vm::VM;

pub struct InjectRuntime {
    name: &'static str
}

impl InjectRuntime {
    pub fn new() -> InjectRuntime {
        InjectRuntime {
            name: "Inject Runtime Code"
        }
    }
}

impl CompilerPass for InjectRuntime {
    fn name(&self) -> &'static str {
        self.name
    }

    fn as_any(&self) -> &Any {
        self
    }

    #[allow(unused_variables)]
    fn finish_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        debug!("after inject runtime: ");

        debug!("{:?}", func);
    }

    fn visit_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        // make a clone of the blocks
        let blocks = func.content.as_mut().unwrap().blocks.clone();
        let func_context = &mut func.context;

        let mut new_blocks = vec![];

        for (_, block) in blocks.into_iter() {
            // get all the instructions of this block, so we can iterate through them
            let body_copy = block.content.as_ref().unwrap().body.clone();

            // set cur block as current block
            // we may change cur block to some newly generated block, so cur block is mutable
            let mut cur_block = block;

            // clear the body of current block
            cur_block.clear_insts();

            for node in body_copy {
                let inst: &Instruction = node.as_inst();
                trace!("check instruction: {:?}", inst);
                match inst.v {
                    Instruction_::New(ref ty) => {
                        let ty_info = vm.get_backend_type_info(ty.id());
                        let size = math::align_up(mm::check_size(ty_info.size), POINTER_SIZE);
                        let align = mm::check_alignment(ty_info.alignment);

                        if size <= MAX_MEDIUM_OBJECT {
                            let block_after = gen_allocation_sequence(
                                size,
                                align,
                                &node,
                                &mut cur_block,
                                &mut new_blocks,
                                func_context,
                                vm
                            );

                            // alloc_end as cur_block
                            new_blocks.push(cur_block);
                            cur_block = block_after;
                        } else {
                            // large object allocation - keep the NEW inst
                            cur_block.append_inst(node.clone());
                        }
                    }
                    Instruction_::NewHybrid(ref ty, len_index) if inst.ops[len_index].is_const_value() => {
                        let len = inst.ops[len_index].as_value().extract_int_const().unwrap();

                        let ty_info = vm.get_backend_type_info(ty.id());
                        let size = ty_info.size + ty_info.elem_size.unwrap() * (len as usize);
                        let size = math::align_up(mm::check_hybrid_size(size), POINTER_SIZE);
                        let align = mm::check_alignment(ty_info.alignment);

                        if size <= MAX_MEDIUM_OBJECT {
                            let block_after = gen_allocation_sequence(
                                size,
                                align,
                                &node,
                                &mut cur_block,
                                &mut new_blocks,
                                func_context,
                                vm
                            );

                            // alloc_end as cur_block
                            new_blocks.push(cur_block);
                            cur_block = block_after;
                        } else {
                            // large object allocation - keep the NEW inst
                            cur_block.append_inst(node.clone());
                        }
                    }
                    _ => {
                        cur_block.append_inst(node.clone());
                    }
                }
            }

            new_blocks.push(cur_block);
        }

        // insert new blocks to the function
        let f_content = func.content.as_mut().unwrap();
        f_content.blocks.clear();
        for block in new_blocks.drain(..) {
            f_content.blocks.insert(block.id(), block);
        }
    }
}

// returns new block after the allocation
fn gen_allocation_sequence(
    size: ByteSize,
    align: ByteSize,
    node: &P<TreeNode>,
    cur_block: &mut Block,
    new_blocks: &mut Vec<Block>,
    func_context: &mut FunctionContext,
    vm: &VM
) -> Block {
    use runtime::mm::heap::immix::*;

    // are we allocating tiny object? otherwise allocate small/medium into normal
    // immix space
    let is_alloc_tiny = size <= MAX_TINY_OBJECT;

    // tl = GETVMTHREADLOCAL
    let tmp_tl = func_context.make_temporary(vm.next_id(), UPTR_U8_TYPE.clone());
    cur_block.append_inst(TreeNode::new_inst(Instruction {
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![tmp_tl.clone_value()]),
        ops: vec![],
        v: Instruction_::GetVMThreadLocal
    }));

    // cursor_loc = SHIFTIREF tl CURSOR_OFFSET
    let cursor_offset = if is_alloc_tiny {
        *thread::ALLOCATOR_OFFSET + *TINY_ALLOCATOR_OFFSET + *CURSOR_OFFSET
    } else if size <= MAX_MEDIUM_OBJECT {
        *thread::ALLOCATOR_OFFSET + *NORMAL_ALLOCATOR_OFFSET + *CURSOR_OFFSET
    } else {
        unreachable!()
    };
    let tmp_cursor_loc = func_context.make_temporary(vm.next_id(), UPTR_U8_TYPE.clone());
    cur_block.append_inst(TreeNode::new_inst(Instruction {
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![tmp_cursor_loc.clone_value()]),
        ops: vec![
            tmp_tl.clone(),
            TreeNode::new_value(Value::make_int64_const(vm.next_id(), cursor_offset as u64)),
        ],
        v: Instruction_::ShiftIRef {
            is_ptr: true,
            base: 0,
            offset: 1
        }
    }));

    // cursor_loc_u64 = PTRCAST cursor_loc
    let tmp_cursor_loc_u64 = func_context.make_temporary(vm.next_id(), UPTR_U64_TYPE.clone());
    cur_block.append_inst(TreeNode::new_inst(Instruction {
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![tmp_cursor_loc_u64.clone_value()]),
        ops: vec![tmp_cursor_loc],
        v: Instruction_::ConvOp {
            operation: ConvOp::PTRCAST,
            from_ty: UPTR_U8_TYPE.clone(),
            to_ty: UPTR_U64_TYPE.clone(),
            operand: 0
        }
    }));

    // cursor = LOAD cursor_loc_u64
    let tmp_cursor = func_context.make_temporary(vm.next_id(), UINT64_TYPE.clone());
    cur_block.append_inst(TreeNode::new_inst(Instruction {
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![tmp_cursor.clone_value()]),
        ops: vec![tmp_cursor_loc_u64.clone()],
        v: Instruction_::Load {
            is_ptr: true,
            order: MemoryOrder::NotAtomic,
            mem_loc: 0
        }
    }));

    // align up the cursor: (cursor + align - 1) & !(align - 1)
    // cursor_t1 = cursor + (align - 1)
    let tmp_cursor_t1 = func_context.make_temporary(vm.next_id(), UINT64_TYPE.clone());
    let tmp_align_minus_one = TreeNode::new_value(Value::make_int64_const(vm.next_id(), (align - 1) as u64));
    cur_block.append_inst(TreeNode::new_inst(Instruction {
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![tmp_cursor_t1.clone_value()]),
        ops: vec![tmp_cursor.clone(), tmp_align_minus_one.clone()],
        v: Instruction_::BinOp(BinOp::Add, 0, 1)
    }));

    // start = cursor_t1 & !(align - 1)
    let tmp_start = func_context.make_temporary(vm.next_id(), UINT64_TYPE.clone());
    let tmp_not_align_minus_one = TreeNode::new_value(Value::make_int64_const(vm.next_id(), !(align - 1) as u64));
    cur_block.append_inst(TreeNode::new_inst(Instruction {
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![tmp_start.clone_value()]),
        ops: vec![tmp_cursor_t1.clone(), tmp_not_align_minus_one],
        v: Instruction_::BinOp(BinOp::And, 0, 1)
    }));

    // end = start + size
    let tmp_end = func_context.make_temporary(vm.next_id(), UINT64_TYPE.clone());
    let tmp_size = TreeNode::new_value(Value::make_int64_const(vm.next_id(), size as u64));
    cur_block.append_inst(TreeNode::new_inst(Instruction {
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![tmp_end.clone_value()]),
        ops: vec![tmp_start.clone(), tmp_size.clone()],
        v: Instruction_::BinOp(BinOp::Add, 0, 1)
    }));

    // limit_loc = SHIFTIREF tl LIMIT_OFFSET
    let limit_offset = if is_alloc_tiny {
        *thread::ALLOCATOR_OFFSET + *TINY_ALLOCATOR_OFFSET + *LIMIT_OFFSET
    } else if size <= MAX_MEDIUM_OBJECT {
        *thread::ALLOCATOR_OFFSET + *NORMAL_ALLOCATOR_OFFSET + *LIMIT_OFFSET
    } else {
        unreachable!()
    };
    let tmp_limit_loc = func_context.make_temporary(vm.next_id(), UPTR_U8_TYPE.clone());
    cur_block.append_inst(TreeNode::new_inst(Instruction {
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![tmp_limit_loc.clone_value()]),
        ops: vec![
            tmp_tl.clone(),
            TreeNode::new_value(Value::make_int64_const(vm.next_id(), limit_offset as u64)),
        ],
        v: Instruction_::ShiftIRef {
            is_ptr: true,
            base: 0,
            offset: 1
        }
    }));

    // limit_loc_u64 = PTRCAST limit_loc
    let tmp_limit_loc_u64 = func_context.make_temporary(vm.next_id(), UPTR_U64_TYPE.clone());
    cur_block.append_inst(TreeNode::new_inst(Instruction {
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![tmp_limit_loc_u64.clone_value()]),
        ops: vec![tmp_limit_loc],
        v: Instruction_::ConvOp {
            operation: ConvOp::PTRCAST,
            from_ty: UPTR_U8_TYPE.clone(),
            to_ty: UPTR_U64_TYPE.clone(),
            operand: 0
        }
    }));

    // limit = LOAD limit_loc_u64
    let tmp_limit = func_context.make_temporary(vm.next_id(), UINT64_TYPE.clone());
    cur_block.append_inst(TreeNode::new_inst(Instruction {
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![tmp_limit.clone_value()]),
        ops: vec![tmp_limit_loc_u64.clone()],
        v: Instruction_::Load {
            is_ptr: true,
            order: MemoryOrder::NotAtomic,
            mem_loc: 0
        }
    }));

    // exceed_limit = UGT tmp_end tmp_limit
    let tmp_exceed_limit = func_context.make_temporary(vm.next_id(), UINT1_TYPE.clone());
    cur_block.append_inst(TreeNode::new_inst(Instruction {
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![tmp_exceed_limit.clone_value()]),
        ops: vec![tmp_end.clone(), tmp_limit.clone()],
        v: Instruction_::CmpOp(CmpOp::UGT, 0, 1)
    }));

    // alloc_end
    let alloc_end = {
        let block_name = Arc::new(format!("new:{}:end", node.id()));
        let mut block = Block::new(MuEntityHeader::named(vm.next_id(), block_name));
        block.trace_hint = TraceHint::None;
        block.content = Some(BlockContent {
            args: vec![],
            exn_arg: None,
            body: vec![],
            keepalives: None
        });
        block
    };

    // result of the allocation
    let tmp_res = node.as_value().clone();

    // fastpath and slowpath - they both jumps to alloc_end
    let fastpath = {
        let block_name = Arc::new(format!("new:{}:fastpath", node.id()));
        let mut block = Block::new(MuEntityHeader::named(vm.next_id(), block_name));
        block.trace_hint = TraceHint::FastPath;
        block.content = Some(BlockContent {
            args: vec![],
            exn_arg: None,
            body: vec![
                // STORE end cursor_loc_u64
                TreeNode::new_inst(Instruction {
                    hdr: MuEntityHeader::unnamed(vm.next_id()),
                    value: None,
                    ops: vec![tmp_cursor_loc_u64.clone(), tmp_end.clone()],
                    v: Instruction_::Store {
                        is_ptr: true,
                        order: MemoryOrder::NotAtomic,
                        mem_loc: 0,
                        value: 1
                    }
                }),
                // MOVE tmp_start -> tmp_res
                TreeNode::new_inst(Instruction {
                    hdr: MuEntityHeader::unnamed(vm.next_id()),
                    value: Some(vec![tmp_res.clone()]),
                    ops: vec![tmp_start.clone()],
                    v: Instruction_::Move(0)
                }),
                // BRANCH alloc_end
                TreeNode::new_inst(Instruction {
                    hdr: MuEntityHeader::unnamed(vm.next_id()),
                    value: None,
                    ops: vec![],
                    v: Instruction_::Branch1(Destination {
                        target: alloc_end.hdr.clone(),
                        args: vec![]
                    })
                }),
            ],
            keepalives: None
        });
        block
    };
    let slowpath = {
        let block_name = Arc::new(format!("new:{}:slowpath", node.id()));
        let mut block = Block::new(MuEntityHeader::named(vm.next_id(), block_name));
        block.trace_hint = TraceHint::SlowPath;
        block.content = Some(BlockContent {
            args: vec![],
            exn_arg: None,
            body: {
                let mutator_offset = *thread::ALLOCATOR_OFFSET;
                let tmp_mutator_loc = func_context.make_temporary(vm.next_id(), UPTR_U8_TYPE.clone());
                let tmp_align = TreeNode::new_value(Value::make_int64_const(vm.next_id(), align as u64));

                let func: &entrypoints::RuntimeEntrypoint = if is_alloc_tiny {
                    &entrypoints::ALLOC_TINY_SLOW
                } else {
                    &entrypoints::ALLOC_NORMAL_SLOW
                };
                let tmp_alloc_slow = TreeNode::new_value(P(Value {
                    hdr: MuEntityHeader::unnamed(vm.next_id()),
                    ty: P(MuType::new(vm.next_id(), MuType_::UFuncPtr(func.sig.clone()))),
                    v: Value_::Constant(Constant::ExternSym(func.aot.to_relocatable()))
                }));
                vec![
                    // tmp_mutator_loc = SHIFTIREF tmp_tl MUTATOR_OFFSET
                    TreeNode::new_inst(Instruction {
                        hdr: MuEntityHeader::unnamed(vm.next_id()),
                        value: Some(vec![tmp_mutator_loc.clone_value()]),
                        ops: vec![
                            tmp_tl.clone(),
                            TreeNode::new_value(Value::make_int64_const(vm.next_id(), mutator_offset as u64)),
                        ],
                        v: Instruction_::ShiftIRef {
                            is_ptr: true,
                            base: 0,
                            offset: 1
                        }
                    }),
                    // CCALL alloc_slow(mutator, size, align)
                    TreeNode::new_inst(Instruction {
                        hdr: MuEntityHeader::unnamed(vm.next_id()),
                        value: Some(vec![tmp_res.clone()]),
                        ops: vec![tmp_alloc_slow, tmp_mutator_loc.clone(), tmp_size.clone(), tmp_align],
                        v: Instruction_::ExprCCall {
                            data: CallData {
                                func: 0,
                                args: vec![1, 2, 3],
                                convention: C_CALL_CONVENTION
                            },
                            is_abort: false
                        }
                    }),
                    // BRANCH alloc_end
                    TreeNode::new_inst(Instruction {
                        hdr: MuEntityHeader::unnamed(vm.next_id()),
                        value: None,
                        ops: vec![],
                        v: Instruction_::Branch1(Destination {
                            target: alloc_end.hdr.clone(),
                            args: vec![]
                        })
                    }),
                ]
            },
            keepalives: None
        });
        block
    };

    // BRANCH2 exceed_limit slowpath fastpath
    cur_block.append_inst(TreeNode::new_inst(Instruction {
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: None,
        ops: vec![tmp_exceed_limit.clone()],
        v: Instruction_::Branch2 {
            cond: 0,
            true_dest: Destination {
                target: slowpath.hdr.clone(),
                args: vec![]
            },
            false_dest: Destination {
                target: fastpath.hdr.clone(),
                args: vec![]
            },
            true_prob: 0.1f32
        }
    }));

    // put alloc_end, slowpath, fastpath to new blocks
    new_blocks.push(slowpath);
    new_blocks.push(fastpath);

    alloc_end
}
