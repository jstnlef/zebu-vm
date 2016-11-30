use ast::ir::*;
use ast::ptr::*;
use ast::inst::*;
use vm::VM;

use compiler::CompilerPass;
use std::any::Any;
use std::sync::RwLock;
use std::collections::HashMap;

pub struct Inlining {
    name: &'static str,

    // whether a function version should be inlined
    should_inline: HashMap<MuID, bool>
}

impl Inlining {
    pub fn new() -> Inlining {
        Inlining{
            name: "Inlining",
            should_inline: HashMap::new()
        }
    }

    fn check(&mut self, vm: &VM, func: &mut MuFunctionVersion) -> bool {
        debug!("check inline");
        let mut inline_something = false;

        for func_id in func.get_static_call_edges().values() {
            let should_inline_this = self.check_should_inline_func(*func_id, func.func_id, vm);

            inline_something = inline_something || should_inline_this;
        }

        inline_something
    }

    #[allow(unused_variables)]
    fn check_should_inline_func(&mut self, callee: MuID, caller: MuID, vm: &VM) -> bool {
        // recursive call, do not inline
        if callee == caller {
            return false;
        }

        let funcs_guard = vm.funcs().read().unwrap();
        let func = match funcs_guard.get(&callee) {
            Some(func) => func.read().unwrap(),
            None => panic!("callee {} is undeclared", callee)
        };

        let fv_id = match func.cur_ver {
            Some(fv_id) => fv_id,
            None => {
                // the funtion is not defined
                info!("the function is undefined, we cannot inline it. ");
                return false;
            }
        };

        match self.should_inline.get(&fv_id) {
            Some(flag) => {
                trace!("func {} should be inlined (checked before)", callee);
                return *flag;
            }
            None => {}
        }

        let fv_guard = vm.func_vers().read().unwrap();
        let fv = fv_guard.get(&fv_id).unwrap().read().unwrap();

        // if the function is forced inline, then we inline it
        if fv.force_inline {
            trace!("func {} is forced as inline function", callee);
            return true;
        }

        // some heuristics here to decide if we should inline the function
        // to be more precise. we should be target specific
        let n_params = fv.sig.arg_tys.len();
        let n_insts  = fv.content.as_ref().unwrap().blocks.values().fold(0usize, |mut sum, ref block| {sum += block.number_of_irs(); sum});
        let out_calls = fv.get_static_call_edges();
        let has_throw = fv.has_throw();

        // now we use a simple heuristic here:
        // insts fewer than 10, no static out calls, no throw
        let should_inline = n_insts <= 10 && out_calls.len() == 0 && !has_throw;

        trace!("func has {} insts", n_insts);
        trace!("     has {} out calls", out_calls.len());
        trace!("     has throws? {}", has_throw);
        trace!("SO func should be inlined? {}", should_inline);

        self.should_inline.insert(fv_id, should_inline);

        should_inline
    }

    fn inline(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        debug!("inlining for Function {}", func);

        let call_edges = func.get_static_call_edges();

        let mut f_content = func.content.as_mut().unwrap();
        let ref mut f_context = func.context;

        let mut new_blocks : Vec<Block> = vec![];

        for (blk_id, mut block) in f_content.blocks.drain() {
            // clone curent block, and clear its instructions
            let mut cur_block = block.clone();
            cur_block.content.as_mut().unwrap().body.clear();

            // iterate through instructions
            for inst in block.content.unwrap().body {
                trace!("check inst: {}", inst);
                let inst_id = inst.id();
                if call_edges.contains_key(&inst_id) {
                    trace!("inserting inlined function at {}", inst);

                    // from TreeNode into Inst (we do not need old TreeNode)
                    let inst = inst.into_inst().unwrap();

                    // (inline expansion)

                    let inlined_func = *call_edges.get(&inst.id()).unwrap();
                    trace!("function being inlined is {}", inlined_func);

                    let inlined_fvid = match vm.get_cur_version_of(inlined_func) {
                        Some(fvid) => fvid,
                        None => panic!("cannot resolve current version of Func {}, which is supposed to be inlined", inlined_func)
                    };

                    let inlined_fvs_guard = vm.func_vers().read().unwrap();
                    let inlined_fv_lock   = inlined_fvs_guard.get(&inlined_fvid).unwrap();
                    let inlined_fv_guard  = inlined_fv_lock.read().unwrap();

                    let inlined_entry = inlined_fv_guard.content.as_ref().unwrap().entry;

                    // change current call insts to a branch
                    trace!("turning CALL instruction into a branch");
                    let ops = inst.ops.read().unwrap();

                    match inst.v {
                        Instruction_::ExprCall {ref data, ..} => {
                            let arg_nodes  : Vec<P<TreeNode>> = data.args.iter().map(|x| ops[*x].clone()).collect();
                            let arg_indices: Vec<OpIndex> = (0..arg_nodes.len()).collect();

                            let branch = TreeNode::new_boxed_inst(Instruction{
                                hdr: inst.hdr.clone(),
                                value: None,
                                ops: RwLock::new(arg_nodes.clone()),
                                v: Instruction_::Branch1(Destination{
                                    target: inlined_entry,
                                    args: arg_indices.iter().map(|x| DestArg::Normal(*x)).collect()
                                })
                            });

                            trace!("branch inst: {}", branch);

                            // add branch to current block
                            cur_block.content.as_mut().unwrap().body.push(branch);

                            // finish current block
                            new_blocks.push(cur_block.clone());
                            let old_name = cur_block.name().unwrap();

                            // start a new block
                            cur_block = Block::new(vm.next_id());
                            cur_block.content = Some(BlockContent{
                                args: {
                                    if inst.value.is_none() {
                                        vec![]
                                    } else {
                                        inst.value.unwrap()
                                    }
                                },
                                exn_arg: None,
                                body: vec![],
                                keepalives: None
                            });
                            let new_name = format!("{}_cont_after_inline_{}", old_name, inst_id);
                            trace!("create continue block for EXPRCALL/CCALL: {}", &new_name);
                            vm.set_name(cur_block.as_entity(), new_name);

                            // deal with the inlined function
                            copy_inline_blocks(&mut new_blocks, cur_block.id(), inlined_fv_guard.content.as_ref().unwrap());
                            copy_inline_context(f_context, &inlined_fv_guard.context);
                        },

                        Instruction_::Call {ref data, ref resume} => {
                            let arg_nodes  : Vec<P<TreeNode>> = data.args.iter().map(|x| ops[*x].clone()).collect();
                            let arg_indices: Vec<OpIndex> = (0..arg_nodes.len()).collect();

                            let branch = Instruction{
                                hdr: inst.hdr.clone(),
                                value: None,
                                ops: RwLock::new(arg_nodes),
                                v: Instruction_::Branch1(Destination{
                                    target: inlined_entry,
                                    args: arg_indices.iter().map(|x| DestArg::Normal(*x)).collect()
                                })
                            };

                            // add branch to current block
                            cur_block.content.as_mut().unwrap().body.push(TreeNode::new_boxed_inst(branch));

                            // if normal_dest expects different number of arguments
                            // other than the inlined function returns, we need an intermediate block to pass extra arguments
                            if resume.normal_dest.args.len() != inlined_fv_guard.sig.ret_tys.len() {
                                unimplemented!()
                            }

                            // deal with inlined function
                            let next_block = resume.normal_dest.target;

                            copy_inline_blocks(&mut new_blocks, next_block, inlined_fv_guard.content.as_ref().unwrap());
                            copy_inline_context(f_context, &inlined_fv_guard.context);
                        },

                        _ => panic!("unexpected callsite: {}", inst)
                    }
                } else {
                    cur_block.content.as_mut().unwrap().body.push(inst.clone());
                }
            }

            new_blocks.push(cur_block);
        }

        f_content.blocks.clear();
        for blk in new_blocks {
            f_content.blocks.insert(blk.id(), blk);
        }
    }
}

fn copy_inline_blocks(caller: &mut Vec<Block>, ret_block: MuID, callee: &FunctionContent) {
    trace!("trying to copy inlined function blocks to caller");
    for block in callee.blocks.values() {
        let mut block = block.clone();

        // check its last instruction
        {
            let block_content = block.content.as_mut().unwrap();
            let last_inst = block_content.body.pop().unwrap();
            let last_inst_clone = last_inst.clone();

            match last_inst.v {
                TreeNode_::Instruction(inst) => {
                    let hdr = inst.hdr;
                    let value = inst.value;
                    let ops = inst.ops;
                    let v = inst.v;

                    match v {
                        Instruction_::Return(vec) => {
                            // change RET to a branch
                            let branch = Instruction {
                                hdr: hdr,
                                value: value,
                                ops: ops,
                                v: Instruction_::Branch1(Destination {
                                    target: ret_block,
                                    args: vec.iter().map(|x| DestArg::Normal(*x)).collect()
                                })
                            };

                            block_content.body.push(TreeNode::new_boxed_inst(branch));
                        },
                        _ => {block_content.body.push(last_inst_clone);}
                    }
                },
                _ => {
                    // do nothing, and directly push the instruction back
                    block_content.body.push(last_inst_clone)
                }
            }
        }

        caller.push(block);
    }
}

fn copy_inline_context(caller: &mut FunctionContext, callee: &FunctionContext) {
    trace!("trying to copy inlined function context to caller");
    for (id, entry) in callee.values.iter() {
        caller.values.insert(*id, SSAVarEntry::new(entry.value().clone()));
    }
}

impl CompilerPass for Inlining {
    fn name(&self) -> &'static str {
        self.name
    }

    fn as_any(&self) -> &Any {
        self
    }

    fn visit_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        if self.check(vm, func) {
            self.inline(vm, func);

            debug!("after inlining: {:?}", func);
        }
    }
}