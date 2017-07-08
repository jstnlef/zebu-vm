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

use ast::ir::*;
use ast::ptr::*;
use ast::inst::*;
use vm::VM;

use compiler::CompilerPass;
use std::any::Any;

pub struct GenMovPhi {
    name: &'static str,
}

impl GenMovPhi {
    pub fn new() -> GenMovPhi {
        GenMovPhi{name: "Generate Phi Moves"}
    }
}

struct IntermediateBlockInfo {
    blk_id: MuID,
    blk_name: MuName,
    target: MuID,
    from_args : Vec<P<TreeNode>>
}

impl CompilerPass for GenMovPhi {
    fn name(&self) -> &'static str {
        self.name
    }

    fn as_any(&self) -> &Any {
        self
    }

    fn visit_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        let mut f_content = func.content.take().unwrap();

        let mut new_blocks_to_insert : Vec<IntermediateBlockInfo> = vec![];

        // iteratio blocks
        for (blk_id, mut block) in f_content.blocks.iter_mut() {
            trace!("block: {}", blk_id);

            // old block content
            let block_content = block.content.as_ref().unwrap().clone();

            let mut new_body = vec![];

            let mut i = 0;
            let i_last = block_content.body.len() - 1;
            for node in block_content.body.iter() {
                // check if this is the last element
                if i != i_last {
                    new_body.push(node.clone());
                } else {
                    trace!("last instruction is {}", node);
                    let last_inst = node.clone();

                    match last_inst.v {
                        TreeNode_::Instruction(inst) => {
                            let ref ops = inst.ops;
                            let inst_name = inst.name().clone();
                            match inst.v {
                                Instruction_::Branch2{cond, true_dest, false_dest, true_prob} => {
                                    let true_dest  = process_dest(true_dest,  &mut new_blocks_to_insert, &ops, vm, &inst_name, "true");
                                    let false_dest = process_dest(false_dest, &mut new_blocks_to_insert, &ops, vm, &inst_name, "false");

                                    let new_inst = func.new_inst(Instruction{
                                        hdr: inst.hdr.clone(),
                                        value: inst.value.clone(),
                                        ops: ops.to_vec(),
                                        v: Instruction_::Branch2 {
                                            cond: cond,
                                            true_dest: true_dest,
                                            false_dest: false_dest,
                                            true_prob: true_prob
                                        }
                                    });

                                    trace!("rewrite to {}", new_inst);
                                    new_body.push(new_inst);
                                }
                                Instruction_::Call{data, resume} => {
                                    let norm_dest = process_dest(resume.normal_dest, &mut new_blocks_to_insert, &ops, vm, &inst_name, "norm");
                                    let exn_dest  = process_dest(resume.exn_dest,    &mut new_blocks_to_insert, &ops, vm, &inst_name, "exc");

                                    let new_inst = func.new_inst(Instruction{
                                        hdr: inst.hdr.clone(),
                                        value: inst.value.clone(),
                                        ops: ops.to_vec(),
                                        v: Instruction_::Call {
                                            data: data.clone(),
                                            resume: ResumptionData{
                                                normal_dest: norm_dest,
                                                exn_dest: exn_dest
                                            }
                                        }
                                    });

                                    trace!("rewrite to {}", new_inst);
                                    new_body.push(new_inst);
                                }
                                Instruction_::CCall{data, resume} => {
                                    let norm_dest = process_dest(resume.normal_dest, &mut new_blocks_to_insert, &ops, vm, &inst_name, "norm");
                                    let exn_dest  = process_dest(resume.exn_dest,    &mut new_blocks_to_insert, &ops, vm, &inst_name, "exc");

                                    let new_inst = func.new_inst(Instruction{
                                        hdr: inst.hdr.clone(),
                                        value: inst.value.clone(),
                                        ops: ops.to_vec(),
                                        v: Instruction_::Call {
                                            data: data.clone(),
                                            resume: ResumptionData{
                                                normal_dest: norm_dest,
                                                exn_dest: exn_dest
                                            }
                                        }
                                    });

                                    trace!("rewrite to {}", new_inst);
                                    new_body.push(new_inst);
                                },
                                Instruction_::Switch{cond, default, mut branches} => {
                                    let default_dest = process_dest(default, &mut new_blocks_to_insert, &ops, vm, &inst_name, "default");

                                    let new_branches = branches.drain(..).map(|pair| {
                                        let dest = process_dest(pair.1, &mut new_blocks_to_insert, &ops, vm, &inst_name, format!("case_{}", pair.0).as_str());
                                        (pair.0, dest)
                                    }).collect();

                                    let new_inst = func.new_inst(Instruction{
                                        hdr: inst.hdr.clone(),
                                        value: inst.value.clone(),
                                        ops: ops.to_vec(),
                                        v: Instruction_::Switch {
                                            cond: cond,
                                            default: default_dest,
                                            branches: new_branches
                                        }
                                    });

                                    trace!("rewrite to {}", new_inst);
                                    new_body.push(new_inst);
                                }
                                Instruction_::Watchpoint{..} => {
                                    unimplemented!()
                                },
                                Instruction_::WPBranch{..} => {
                                    unimplemented!()
                                },
                                Instruction_::SwapStack{..} => {
                                    unimplemented!()
                                },
                                Instruction_::ExnInstruction{..} => {
                                    unimplemented!()
                                },
                                _ => {
                                    trace!("no rewrite");
                                    new_body.push(node.clone())
                                }
                            }
                        }
                        _ => panic!("expect a terminal instruction")
                    }
                }

                i += 1;
            }

            block.content = Some(BlockContent{
                args      : block_content.args.to_vec(),
                exn_arg   : block_content.exn_arg.clone(),
                body      : new_body,
                keepalives: block_content.keepalives.clone()
            });
        }

        // insert new blocks here
        for block_info in new_blocks_to_insert {
            let block = {
                let target_id = block_info.target;
                let mut ret = Block::new(MuEntityHeader::named(block_info.blk_id, block_info.blk_name.clone()));
                vm.set_name(ret.as_entity());


                let mut target_block = f_content.get_block_mut(target_id);

                assert!(target_block.content.is_some());

                // if target_block is an exception block,
                // set its exn argument to None, and set this new block as an exception block
                let exn_arg = target_block.content.as_mut().unwrap().exn_arg.take();
                let ref target_args = target_block.content.as_ref().unwrap().args;

                ret.content = Some(BlockContent{
                    args: vec![],
                    exn_arg: exn_arg,
                    body: {
                        let mut vec = vec![];

                        // move every from_arg to target_arg
                        let mut i = 0;
                        for arg in block_info.from_args.iter() {
                            let m = func.new_inst(Instruction{
                                hdr: MuEntityHeader::unnamed(vm.next_id()),
                                value: Some(vec![target_args[i].clone()]),
                                ops: vec![arg.clone()],
                                v: Instruction_::Move(0)
                            });

                            vec.push(m);

                            i += 1;
                        }

                        // branch to target
                        let b = func.new_inst(Instruction{
                            hdr: MuEntityHeader::unnamed(vm.next_id()),
                            value: None,
                            ops: vec![],
                            v: Instruction_::Branch1(Destination{
                                target: target_id,
                                args: vec![]
                            })
                        });
                        vec.push(b);

                        vec
                    },
                    keepalives: None
                });

                trace!("inserting new intermediate block: {:?}", ret);

                ret
            };

            f_content.blocks.insert(block.id(), block);
        }

        func.content = Some(f_content);
    }
}

fn process_dest(dest: Destination, blocks_to_insert: &mut Vec<IntermediateBlockInfo>, ops: &Vec<P<TreeNode>>, vm: &VM, inst: &MuName, label: &str) -> Destination {
    if dest.args.is_empty() {
        dest
    } else {
        let target = dest.target;

        let mut from_args = vec![];
        for arg in dest.args.iter() {
            let from_arg = match arg {
                &DestArg::Normal(i) => ops[i].clone(),
                &DestArg::Freshbound(_) => unimplemented!()
            };

            from_args.push(from_arg);
        };

        let new_blk_id = vm.next_id();

        let dest = Destination {
            target: new_blk_id,
            args: vec![]
        };

        blocks_to_insert.push(IntermediateBlockInfo {
            blk_id: new_blk_id,
            blk_name: format!("{}:intermediate.{}", inst, label),
            target: target,
            from_args: from_args
        });

        dest
    }
}
