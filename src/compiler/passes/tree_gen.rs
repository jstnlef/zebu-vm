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

use vm::VM;
use compiler::CompilerPass;

use std::any::Any;

pub struct TreeGen {
    name: &'static str
} 

impl TreeGen {
    pub fn new() -> TreeGen {
        TreeGen{name: "Tree Geenration"}
    }
}

fn is_movable(inst: &Instruction) -> bool {
    !inst.has_side_effect()
}

impl CompilerPass for TreeGen {
    fn name(&self) -> &'static str {
        self.name
    }

    fn as_any(&self) -> &Any {
        self
    }
    
    fn execute(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        // We are trying to generate a depth tree from the original AST.
        // If an SSA variable is used only once, and the instruction that generates it is movable,
        // then we replace the use of the SSA with the actual variable

        // we are doing it in two steps
        // 1. if we see an expression that generates an SSA which is used only once, we take out
        //    the expression node
        // 2. if we see an SSA that is used only once (and it is this place for sure), we replace it
        //    with the expression node
        // because of SSA form,  it is guaranteed to see 1 before 2 for SSA variables.
        debug!("---CompilerPass {} for {}---", self.name(), func);
        
        {
            let ref mut func_content = func.content;
            let ref mut context = func.context;
            for (label, ref mut block) in func_content.as_mut().unwrap().blocks.iter_mut() {
                // take its content, we will need to put it back
                let mut content = block.content.take().unwrap();
                let body = content.body;
                
                let mut new_body = vec![];
                
                trace!("check block {}", label);
                trace!("");
                
                for mut node in body.into_iter() {
                    trace!("check inst: {}", node);
                    match &mut node.v {
                        &mut TreeNode_::Instruction(ref mut inst) => {
                            // check whether any operands (SSA) can be replaced by expression
                            {
                                trace!("check if we can replace any operand with inst");
                                
                                let ref mut ops = inst.ops;
                                for index in 0..ops.len() {
                                    let possible_ssa_id = ops[index].extract_ssa_id();
                                    if possible_ssa_id.is_some() {
                                        let entry_value = context.get_value_mut(possible_ssa_id.unwrap()).unwrap();
                                        
                                        if entry_value.has_expr() {
                                            // replace the node with its expr
                                            let expr = entry_value.take_expr();
                                            
                                            trace!("{} replaced by {}", ops[index], expr);
                                            ops[index] = TreeNode::new_inst(expr);
                                        }
                                    } else {
                                        trace!("{} cant be replaced", ops[index]);
                                    } 
                                }
                            }

                            // check whether the instruction generates an SSA that is used only once
                            // An instruction can replace its value if
                            // * it generates only one value
                            // * the value is used only once
                            // * the instruction is movable
                            trace!("check if we should fold the inst");
                            if inst.value.is_some() {
                                let left = inst.value.as_ref().unwrap();
                                
                                // if left is _one_ variable that is used once
                                // we can put the expression as a child node to its use
                                if left.len() == 1 {
                                    let lhs = context.get_value_mut(left[0].extract_ssa_id().unwrap()).unwrap(); 
                                    if lhs.use_count() == 1{
                                        if is_movable(&inst) {
                                            lhs.assign_expr(inst.clone()); // FIXME: should be able to move the inst here
                                            
                                            trace!("yes");
                                            trace!("");
                                            continue;
                                        } else {
                                            trace!("no, not movable");
                                        }
                                    } else {
                                        trace!("no, use count more than 1");
                                    }
                                } else {
                                    trace!("no, yields more than 1 SSA var");
                                }
                            } else {
                                trace!("no, no value yielded");
                            }
                        },
                        _ => panic!("expected an instruction node here")
                    }
                    
                    trace!("add {} back to block {}", node, label);
                    trace!("");
                    new_body.push(node);
                }
                
                content.body = new_body;
                trace!("block {} has {} insts", label, content.body.len());
                trace!("");
                            
                // put the content back
                block.content = Some(content);
            }
        }
        
        self.finish_function(vm, func);
        
        debug!("---finish---");
    }
    
    #[allow(unused_variables)]
    fn finish_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        debug!("check depth tree for {}", func);
        
        for entry in func.content.as_ref().unwrap().blocks.iter() {
            debug!("block {}", entry.1.name().unwrap());
            
            for inst in entry.1.content.as_ref().unwrap().body.iter() {
                debug!("{}", inst);
            }
        }
    }
}
