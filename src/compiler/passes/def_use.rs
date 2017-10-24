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
use vm::VM;
use compiler::CompilerPass;
use std::any::Any;

pub struct DefUse {
    name: &'static str
}

impl DefUse {
    pub fn new() -> DefUse {
        DefUse {
            name: "Def-Use Pass"
        }
    }
}

impl CompilerPass for DefUse {
    fn name(&self) -> &'static str {
        self.name
    }

    fn as_any(&self) -> &Any {
        self
    }

    #[allow(unused_variables)]
    fn start_block(&mut self, vm: &VM, func_context: &mut FunctionContext, block: &mut Block) {}

    #[allow(unused_variables)]
    fn finish_block(&mut self, vm: &VM, func_context: &mut FunctionContext, block: &mut Block) {
        // if an SSA appears in keepalives, its use count increases
        let ref keepalives = block.content.as_ref().unwrap().keepalives;
        if let &Some(ref keepalives) = keepalives {
            for op in keepalives.iter() {
                use_value(op, func_context);
            }
        }
    }

    #[allow(unused_variables)]
    fn visit_inst(&mut self, vm: &VM, func_context: &mut FunctionContext, node: &P<TreeNode>) {
        let inst = node.as_inst();

        if let &Some(ref vals) = &inst.value {
            for val in vals {
                let id = val.extract_ssa_id().unwrap();
                func_context
                    .get_value_mut(id)
                    .unwrap()
                    .set_def(node.clone());
            }
        }

        // if an SSA appears in operands of instrs, its use count increases
        for op in inst.ops.iter() {
            use_op(op, func_context);
        }
    }

    #[allow(unused_variables)]
    fn start_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        for entry in func.context.values.values() {
            entry.reset_use_count();
        }
    }

    #[allow(unused_variables)]
    fn finish_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        debug!("check use count for variables");

        for entry in func.context.values.values() {
            debug!("{}: {}", entry, entry.use_count())
        }
    }
}

fn use_op(op: &P<TreeNode>, func_context: &mut FunctionContext) {
    match op.v {
        TreeNode_::Value(ref val) => {
            use_value(val, func_context);
        }
        _ => {} // dont worry about instruction
    }
}

fn use_value(val: &P<Value>, func_context: &mut FunctionContext) {
    match val.v {
        Value_::SSAVar(ref id) => {
            let entry = func_context.values.get_mut(id).unwrap();
            entry.increase_use_count();
        }
        _ => {} // dont worry about constants
    }
}
