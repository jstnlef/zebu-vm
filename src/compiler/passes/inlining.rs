use ast::ir::*;
use ast::ptr::*;
use ast::inst::*;
use vm::VM;

use compiler::CompilerPass;
use std::any::Any;
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
        self.should_inline.clear();

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
        let n_insts  = estimate_insts(&fv);
        let out_calls = fv.get_static_call_edges();
        let has_throw = fv.has_throw();

        // now we use a simple heuristic here:
        // insts fewer than 10, no static out calls, no throw
        let should_inline = n_insts <= 25 && out_calls.len() == 0 && !has_throw;

        trace!("func {} has {} insts (estimated)", callee, n_insts);
        trace!("     has {} out calls", out_calls.len());
        trace!("     has throws? {}", has_throw);
        trace!("SO func should be inlined? {}", should_inline);

        self.should_inline.insert(callee, should_inline);

        should_inline
    }

    fn inline(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        debug!("inlining for Function {}", func);

        let call_edges = func.get_static_call_edges();

        let mut f_content = func.content.as_mut().unwrap();
        let ref mut f_context = func.context;

        let mut new_blocks : Vec<Block> = vec![];

        for (_, block) in f_content.blocks.iter() {
            // clone curent block, and clear its instructions
            let mut cur_block = block.clone();
            cur_block.content.as_mut().unwrap().body.clear();

            let block = block.clone();

            // iterate through instructions
            for inst in block.content.unwrap().body {
                trace!("check inst: {}", inst);
                let inst_id = inst.id();
                if call_edges.contains_key(&inst_id) {
                    let call_target = call_edges.get(&inst_id).unwrap();
                    if self.should_inline.contains_key(call_target) && *self.should_inline.get(call_target).unwrap() {

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

                        trace!("orig_content: {:?}", inlined_fv_guard.get_orig_ir().unwrap());
                        trace!("content     : {:?}", inlined_fv_guard.content.as_ref().unwrap());

                        let new_inlined_entry_id = vm.next_id();

                        // change current call insts to a branch
                        trace!("turning CALL instruction into a branch");
                        let ref ops = inst.ops;

                        match inst.v {
                            Instruction_::ExprCall {ref data, ..} => {
                                let arg_nodes  : Vec<P<TreeNode>> = data.args.iter().map(|x| ops[*x].clone()).collect();
                                let arg_indices: Vec<OpIndex> = (0..arg_nodes.len()).collect();

                                let branch = TreeNode::new_boxed_inst(Instruction{
                                    hdr: inst.hdr.clone(),
                                    value: None,
                                    ops: arg_nodes.clone(),
                                    v: Instruction_::Branch1(Destination{
                                        // this block doesnt exist yet, we will fix it later
                                        target: new_inlined_entry_id,
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
                                let new_name = format!("{}_cont_after_inline_{}", old_name, inst_id);
                                trace!("create continue block for EXPRCALL/CCALL: {}", &new_name);

                                cur_block = Block::new(MuEntityHeader::named(vm.next_id(), new_name));
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
                                vm.set_name(cur_block.as_entity());

                                // deal with the inlined function
                                copy_inline_blocks(&mut new_blocks, cur_block.id(),
                                                   inlined_fv_guard.get_orig_ir().unwrap(), new_inlined_entry_id,
                                                   vm);
                                copy_inline_context(f_context, &inlined_fv_guard.context);
                            },

                            Instruction_::Call {ref data, ref resume} => {
                                let arg_nodes  : Vec<P<TreeNode>> = data.args.iter().map(|x| ops[*x].clone()).collect();
                                let arg_indices: Vec<OpIndex> = (0..arg_nodes.len()).collect();

                                let branch = Instruction{
                                    hdr: inst.hdr.clone(),
                                    value: None,
                                    ops: arg_nodes,
                                    v: Instruction_::Branch1(Destination{
                                        target: new_inlined_entry_id,
                                        args: arg_indices.iter().map(|x| DestArg::Normal(*x)).collect()
                                    })
                                };

                                // add branch to current block
                                cur_block.content.as_mut().unwrap().body.push(TreeNode::new_boxed_inst(branch));

                                // next block
                                let mut next_block = resume.normal_dest.target;

                                // if normal_dest expects different number of arguments
                                // other than the inlined function returns, we need an intermediate block to pass extra arguments
                                if resume.normal_dest.args.len() != inlined_fv_guard.sig.ret_tys.len() {
                                    debug!("need an extra block for passing normal dest arguments");
                                    let int_block_name = format!("inline_{}_arg_pass", inst_id);
                                    let mut intermediate_block = Block::new(MuEntityHeader::named(vm.next_id(), int_block_name));
                                    vm.set_name(intermediate_block.as_entity());

                                    // branch to normal_dest with normal_dest arguments
                                    let normal_dest_args = resume.normal_dest.get_arguments_as_node(&ops);
                                    let normal_dest_args_len = normal_dest_args.len();

                                    let branch = Instruction {
                                        hdr: MuEntityHeader::unnamed(vm.next_id()),
                                        value: None,
                                        ops: normal_dest_args,
                                        v: Instruction_::Branch1(Destination {
                                            target: resume.normal_dest.target,
                                            args: (0..normal_dest_args_len).map(|x| DestArg::Normal(x)).collect()
                                        })
                                    };

                                    intermediate_block.content = Some(BlockContent {
                                        args: {
                                            match inst.value {
                                                Some(ref vec) => vec.clone(),
                                                None => vec![]
                                            }
                                        },
                                        exn_arg: None,
                                        body: vec![TreeNode::new_boxed_inst(branch)],
                                        keepalives: None
                                    });

                                    trace!("extra block: {:?}", intermediate_block);

                                    next_block = intermediate_block.id();
                                    new_blocks.push(intermediate_block);
                                }

                                // deal with inlined function
                                copy_inline_blocks(&mut new_blocks, next_block,
                                                   inlined_fv_guard.get_orig_ir().unwrap(), new_inlined_entry_id,
                                                   vm);
                                copy_inline_context(f_context, &inlined_fv_guard.context);
                            },

                            _ => panic!("unexpected callsite: {}", inst)
                        }
                    } else {
                        cur_block.content.as_mut().unwrap().body.push(inst.clone());
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

fn copy_inline_blocks(caller: &mut Vec<Block>, ret_block: MuID, callee: &FunctionContent, entry_block: MuID, vm: &VM) {
    trace!("trying to copy inlined function blocks to caller");

    // old id -> new id
    let mut block_map : HashMap<MuID, MuID> = HashMap::new();

    for block in callee.blocks.values() {
        if block.id() == callee.entry {
            block_map.insert(block.id(), entry_block);
        } else {
            block_map.insert(block.id(), vm.next_id());
        }
    }

    let fix_dest = |dest : Destination| {
        Destination {
            target: *block_map.get(&dest.target).unwrap(),
            args: dest.args
        }
    };

    let fix_resume = |resume : ResumptionData| {
        ResumptionData {
            normal_dest: fix_dest(resume.normal_dest),
            exn_dest: fix_dest(resume.exn_dest)
        }
    };

    for block in callee.blocks.values() {
        let old_id = block.id();
        let new_id = *block_map.get(&block.id()).unwrap();
        let mut block = Block {
            hdr: MuEntityHeader::named(new_id, format!("inlinedblock{}_for_{}", new_id, block.name().unwrap())),
            content: block.content.clone(),
            control_flow: ControlFlow::default()
        };

        trace!("starts copying instruction from {} to {}", old_id, new_id);

        // check its last instruction
        {
            let block_content = block.content.as_mut().unwrap();
            let last_inst = block_content.body.pop().unwrap();

            // every inst should have a unique ID
            let inst_new_id = vm.next_id();
            let last_inst_clone = match last_inst.v {
                TreeNode_::Instruction(ref inst) => {
                    TreeNode::new_boxed_inst(inst.clone_with_id(inst_new_id))
                }
                _ => panic!("expect instruction as block body")
            };

            match last_inst.v {
                TreeNode_::Instruction(inst) => {
                    trace!("last instruction: {}", inst);

                    let hdr = inst.hdr.clone_with_id(inst_new_id);
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

                            trace!("rewrite to: {}", branch);
                            block_content.body.push(TreeNode::new_boxed_inst(branch));
                        },

                        // fix destination
                        Instruction_::Branch1(dest) => {
                            let branch = Instruction {
                                hdr: hdr,
                                value: value,
                                ops: ops,
                                v: Instruction_::Branch1(fix_dest(dest))
                            };

                            trace!("rewrite to: {}", branch);
                            block_content.body.push(TreeNode::new_boxed_inst(branch));
                        }
                        Instruction_::Branch2{cond, true_dest, false_dest, true_prob} => {
                            let branch2 = Instruction {
                                hdr: hdr,
                                value: value,
                                ops: ops,
                                v: Instruction_::Branch2 {
                                    cond: cond,
                                    true_dest: fix_dest(true_dest),
                                    false_dest: fix_dest(false_dest),
                                    true_prob: true_prob
                                }
                            };

                            trace!("rewrite to: {}", branch2);
                            block_content.body.push(TreeNode::new_boxed_inst(branch2));
                        }
                        Instruction_::Call{data, resume} => {
                            let call = Instruction{
                                hdr: hdr,
                                value: value,
                                ops: ops,
                                v: Instruction_::Call {
                                    data: data,
                                    resume: fix_resume(resume)
                                }
                            };

                            trace!("rewrite to: {}", call);
                            block_content.body.push(TreeNode::new_boxed_inst(call));
                        }
                        Instruction_::CCall{data, resume} => {
                            let call = Instruction{
                                hdr: hdr,
                                value: value,
                                ops: ops,
                                v: Instruction_::CCall {
                                    data: data,
                                    resume: fix_resume(resume)
                                }
                            };

                            trace!("rewrite to: {}", call);
                            block_content.body.push(TreeNode::new_boxed_inst(call));
                        }
                        Instruction_::Switch {cond, default, mut branches} => {
                            let switch = Instruction {
                                hdr: hdr,
                                value: value,
                                ops: ops,
                                v: Instruction_::Switch {
                                    cond: cond,
                                    default: fix_dest(default),
                                    branches: branches.drain(..).map(|(op, dest)| (op, fix_dest(dest))).collect()
                                }
                            };

                            trace!("rewrite to: {}", switch);
                            block_content.body.push(TreeNode::new_boxed_inst(switch));
                        }

                        Instruction_::Watchpoint{..}
                        | Instruction_::WPBranch{..}
                        | Instruction_::SwapStack{..}
                        | Instruction_::ExnInstruction{..} => unimplemented!(),

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

fn estimate_insts(fv: &MuFunctionVersion) -> usize {
    let f_content = fv.content.as_ref().unwrap();

    let mut insts = 0;

    for block in f_content.blocks.values() {
        let ref body = block.content.as_ref().unwrap().body;

        for inst in body.iter() {
            use compiler::backend;

            match inst.v {
                TreeNode_::Value(_) => unreachable!(),
                TreeNode_::Instruction(ref inst) => {insts += backend::estimate_insts_for_ir(inst);}
            }
        }
    }

    insts
}

impl CompilerPass for Inlining {
    fn name(&self) -> &'static str {
        self.name
    }

    fn as_any(&self) -> &Any {
        self
    }

    fn visit_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        if vm.vm_options.flag_disable_inline {
            info!("inlining is disabled");
            return;
        }

        if self.check(vm, func) {
            self.inline(vm, func);

            debug!("after inlining: {:?}", func);
        }
    }
}
