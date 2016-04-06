use ast::ir::*;
use ast::inst::*;
use ast::ir_semantics::*;

use vm::context::VMContext;
use compiler::CompilerPass;

pub struct TreeGen {
    name: &'static str
} 

impl TreeGen {
    pub fn new() -> TreeGen {
        TreeGen{name: "Tree Geenration"}
    }
}

fn is_movable(expr: &Instruction_) -> bool {
    !has_side_effect(expr)
}

impl CompilerPass for TreeGen {
    fn name(&self) -> &'static str {
        self.name
    }
    
    fn execute(&mut self, vm_context: &VMContext, func: &mut MuFunction) {
        debug!("---CompilerPass {} for {}---", self.name(), func.fn_name);
        
        for (label, ref mut block) in func.content.as_mut().unwrap().blocks.iter_mut() {
            // take its content, we will need to put it back
            let mut content = block.content.take().unwrap();
            let body = content.body;
            
            let mut new_body = vec![];
            
            trace!("check block {}", label);
            trace!("");
            
            for node in body.into_iter() {
                trace!("check inst: {}", node);
                match &node.v {
                    &TreeNode_::Instruction(ref inst) => {
                        // check if any operands can be replaced by expression
                        {
                            trace!("check if we can replace any operand with inst");
                            
                            let mut ops = inst.ops.borrow_mut();
                            for index in 0..ops.len() {
                                let possible_ssa_id = ops[index].extract_ssa_id();
                                if possible_ssa_id.is_some() {
                                    let entry_value = func.context.get_value_mut(possible_ssa_id.unwrap()).unwrap();
                                    
                                    if entry_value.expr.is_some() {
                                        // replace the node with its expr
                                        let expr = entry_value.expr.take().unwrap();
                                        
                                        trace!("{} replaced by {}", ops[index], expr);
                                        ops[index] = TreeNode::new_inst(expr);
                                    }
                                } else {
                                    trace!("{} cant be replaced", ops[index]);
                                }
                            }
                        }
                        
                        // check if left hand side of an assignment has a single use
                        trace!("check if we should fold the inst");
                        if inst.value.is_some() {
                            let left = inst.value.as_ref().unwrap();
                            
                            // if left is _one_ variable that is used once
                            // we can put the expression as a child node to its use
                            if left.len() == 1 {
                                let lhs = func.context.get_value_mut(left[0].extract_ssa_id().unwrap()).unwrap(); 
                                if lhs.use_count.get() == 1{
                                    if is_movable(&inst.v) {
                                        lhs.expr = Some(inst.clone()); // FIXME: should be able to move the inst here 
                                        
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
        
        self.finish_function(vm_context, func);
        
        debug!("---finish---");
    }
    
    #[allow(unused_variables)]
    fn finish_function(&mut self, vm_context: &VMContext, func: &mut MuFunction) {
        debug!("check depth tree for {}", func.fn_name);
        
        for entry in func.content.as_ref().unwrap().blocks.iter() {
            debug!("block {}", entry.0);
            
            for inst in entry.1.content.as_ref().unwrap().body.iter() {
                debug!("{}", inst);
            }
        }
    }
}