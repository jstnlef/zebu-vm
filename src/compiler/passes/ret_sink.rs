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
use ast::inst::*;
use ast::ptr::*;
use vm::VM;
use compiler::CompilerPass;
use compiler::EPILOGUE_BLOCK_NAME;
use std::any::Any;

/// Mu IR the client gives us may contain several RET instructions. However,
/// internally we want a single exit point for a function. In this pass, we
/// create a return sink (a block), and rewrite all the RET instruction into
/// a BRANCH with return values.
pub struct RetSink {
    name: &'static str
}

impl RetSink {
    pub fn new() -> RetSink {
        RetSink {
            name: "Creating Return Sink"
        }
    }
}

impl CompilerPass for RetSink {
    fn name(&self) -> &'static str {
        self.name
    }

    fn as_any(&self) -> &Any {
        self
    }

    fn visit_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        let mut f_content = func.content.take().unwrap();

        // create a return sink
        let return_sink = {
            let block_name = format!("{}:{}", func.name(), EPILOGUE_BLOCK_NAME);
            trace!("created return sink {}", block_name);

            let mut block = Block::new(MuEntityHeader::named(vm.next_id(), block_name));
            // tell the compiler this is the return sink
            block.trace_hint = TraceHint::ReturnSink;
            vm.set_name(block.as_entity());

            let sig = func.sig.clone();
            let args : Vec<P<Value>> = sig.ret_tys.iter()
                .map(|ty| func.new_ssa(MuEntityHeader::unnamed(vm.next_id()), ty.clone()).clone_value()).collect();

            block.content = Some(BlockContent {
                args: args.clone(),
                exn_arg: None,
                body: vec![
                    func.new_inst(Instruction {
                        hdr: MuEntityHeader::unnamed(vm.next_id()),
                        value: None,
                        ops: args.iter().map(|val| TreeNode::new_value(val.clone())).collect(),
                        v: Instruction_::Return((0..args.len()).collect())
                    })
                ],
                keepalives: None
            });

            block
        };

        // rewrite existing RET instruction to a BRANCH
        // use RET values as BRANCH's goto values
        for (blk_id, mut block) in f_content.blocks.iter_mut() {
            trace!("block: {}", blk_id);

            // old block content
            let block_content = block.content.as_ref().unwrap().clone();

            let mut new_body = vec![];

            for node in block_content.body.iter() {
                match node.v {
                    TreeNode_::Instruction(Instruction {ref ops, v: Instruction_::Return(ref arg_index), ..}) => {
                        let branch_to_sink = func.new_inst(Instruction {
                            hdr: MuEntityHeader::unnamed(vm.next_id()),
                            value: None,
                            ops: ops.clone(),
                            v: Instruction_::Branch1(Destination {
                                target: return_sink.id(),
                                args: arg_index.iter().map(|i| DestArg::Normal(*i)).collect()
                            })
                        });
                        new_body.push(branch_to_sink);
                    }
                    _ => new_body.push(node.clone())
                }
            }

            block.content = Some(BlockContent {
                args      : block_content.args.to_vec(),
                exn_arg   : block_content.exn_arg.clone(),
                body      : new_body,
                keepalives: block_content.keepalives.clone()
            });
        }

        // insert return sink
        f_content.blocks.insert(return_sink.id(), return_sink);

        // put back the function content
        func.content = Some(f_content);
    }
}