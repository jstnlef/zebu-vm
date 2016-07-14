use ast::ir::*;
use ast::inst::Instruction_::*;
use utils::vec_utils::as_str as vector_as_str;
use vm::context::VMContext;

use compiler::CompilerPass;

pub struct ControlFlowAnalysis {
    name: &'static str
}

impl ControlFlowAnalysis {
    pub fn new() -> ControlFlowAnalysis {
        ControlFlowAnalysis{name: "Control Flow Analysis"}
    }
}

fn check_edge_kind(target: MuTag, stack: &Vec<MuTag>) -> EdgeKind {
    if stack.contains(&target) {
        EdgeKind::Backward
    } else {
        EdgeKind::Forward
    }
}

fn new_edge(cur: MuTag, edge: BlockEdge, stack: &mut Vec<MuTag>, visited: &mut Vec<MuTag>, func: &mut MuFunctionVersion) {
    // add current block to target's predecessors
    {
        let target = func.content.as_mut().unwrap().get_block_mut(edge.target);
        target.control_flow.preds.push(cur);
    }

    // add target as current block's successors and start dfs
    let succ = edge.target;
    {
        let cur = func.content.as_mut().unwrap().get_block_mut(cur);
        cur.control_flow.succs.push(edge);
    }
    if !visited.contains(&succ) {
        dfs(succ, stack, visited, func);
    }

}

const WATCHPOINT_DISABLED_CHANCE : f32 = 0.9f32;

const NORMAL_RESUME_CHANCE       : f32 = 0.6f32;
const EXN_RESUME_CHANCE          : f32 = 1f32 - NORMAL_RESUME_CHANCE;

fn dfs(cur: MuTag, stack: &mut Vec<MuTag>, visited: &mut Vec<MuTag>, func: &mut MuFunctionVersion) {
    trace!("dfs visiting block {}", cur);
    trace!("current stack: {:?}", stack);
    trace!("current visited: {:?}", visited);

    stack.push(cur);
    visited.push(cur);

    // find all the successors for current block, and push them to the stack
    let out_edges : Vec<BlockEdge> = {
        let cur = func.content.as_mut().unwrap().get_block_mut(cur);
        let ref body = cur.content.as_ref().unwrap().body;
        let last_inst = body.last().unwrap();

        match last_inst.v {
            TreeNode_::Instruction(ref inst) => {
                match inst.v {
                    // unconditional branch, definitely branch to the target
                    Branch1(ref dest) => vec![BlockEdge{
                        target: dest.target,
                        kind: check_edge_kind(dest.target, stack),
                        is_exception: false,
                        probability: 1.0f32
                    }],

                    // conditional branch
                    Branch2{ref true_dest, ref false_dest, true_prob, ..} => vec![
                        BlockEdge{
                            target: true_dest.target,
                            kind: check_edge_kind(true_dest.target, stack),
                            is_exception: false,
                            probability: true_prob
                        },
                        BlockEdge{
                            target: false_dest.target,
                            kind: check_edge_kind(false_dest.target, stack),
                            is_exception: false,
                            probability: 1.0f32 - true_prob
                        }
                    ],

                    // watchpoints
                    Watchpoint{ref id, ref disable_dest, ref resume} => {
                        let ref normal = resume.normal_dest;
                        let ref exn    = resume.exn_dest;

                        if id.is_none() {
                            // unconditional trap
                            vec![
                                BlockEdge{
                                    target: normal.target,
                                    kind: check_edge_kind(normal.target, stack),
                                    is_exception: false,
                                    probability: 1.0f32 * NORMAL_RESUME_CHANCE
                                },
                                BlockEdge{
                                    target: exn.target,
                                    kind: check_edge_kind(exn.target, stack),
                                    is_exception: true,
                                    probability: 1.0f32 * EXN_RESUME_CHANCE
                                }
                            ]
                        } else {
                            // watchpoint. jump to disable_dest when disabled. otherwise trap
                            vec![
                                BlockEdge{
                                    target: disable_dest.as_ref().unwrap().target,
                                    kind: check_edge_kind(disable_dest.as_ref().unwrap().target, stack),
                                    is_exception: false,
                                    probability: WATCHPOINT_DISABLED_CHANCE
                                },
                                BlockEdge{
                                    target: normal.target,
                                    kind: check_edge_kind(normal.target, stack),
                                    is_exception: false,
                                    probability: (1.0f32 - WATCHPOINT_DISABLED_CHANCE) * NORMAL_RESUME_CHANCE
                                },
                                BlockEdge{
                                    target: exn.target,
                                    kind: check_edge_kind(exn.target, stack),
                                    is_exception: true,
                                    probability: (1.0f32 - WATCHPOINT_DISABLED_CHANCE) * EXN_RESUME_CHANCE
                                }
                            ]
                        }
                    },

                    // wpbranch
                    WPBranch{ref disable_dest, ref enable_dest, ..} => vec![
                        BlockEdge{
                            target: disable_dest.target,
                            kind: check_edge_kind(disable_dest.target, stack),
                            is_exception: false,
                            probability: WATCHPOINT_DISABLED_CHANCE
                        },
                        BlockEdge{
                            target: enable_dest.target,
                            kind: check_edge_kind(enable_dest.target, stack),
                            is_exception: false,
                            probability: 1.0f32 - WATCHPOINT_DISABLED_CHANCE
                        }
                    ],

                    // call
                    Call{ref resume, ..}
                    | SwapStack{ref resume, ..}
                    | ExnInstruction{ref resume, ..} => {
                        let ref normal = resume.normal_dest;
                        let ref exn    = resume.exn_dest;

                        vec![
                                BlockEdge{
                                    target: normal.target,
                                    kind: check_edge_kind(normal.target, stack),
                                    is_exception: false,
                                    probability: 1.0f32 * NORMAL_RESUME_CHANCE
                                },
                                BlockEdge{
                                    target: exn.target,
                                    kind: check_edge_kind(exn.target, stack),
                                    is_exception: true,
                                    probability: 1.0f32 * EXN_RESUME_CHANCE
                                }

                        ]
                    },

                    _ => vec![]
                }
            },
            _ => panic!("expected an instruction")
        }
    };

    trace!("out edges for {}: {}", cur, vector_as_str(&out_edges));

    for edge in out_edges {
        new_edge(cur, edge, stack, visited, func);
    }

    stack.pop();
}

impl CompilerPass for ControlFlowAnalysis {
    fn name(&self) -> &'static str {
        self.name
    }

    #[allow(unused_variables)]
    fn visit_function(&mut self, vm_context: &VMContext, func: &mut MuFunctionVersion) {
        let mut stack   : Vec<MuTag> = vec![];
        let mut visited : Vec<MuTag> = vec![];

        dfs(func.content.as_ref().unwrap().entry, &mut stack, &mut visited, func);
    }

    #[allow(unused_variables)]
    fn finish_function(&mut self, vm_context: &VMContext, func: &mut MuFunctionVersion) {
        debug!("check control flow for {}", func.fn_name);

        for entry in func.content.as_ref().unwrap().blocks.iter() {
            debug!("block {}", entry.0);

            debug!("{}", entry.1.control_flow);
        }
    }
}
