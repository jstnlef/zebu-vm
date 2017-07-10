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
use vm::VM;

mod def_use;
mod tree_gen;
mod control_flow;
mod trace_gen;
mod gen_mov_phi;
mod inlining;
mod dot_gen;

pub use compiler::passes::inlining::Inlining;
pub use compiler::passes::def_use::DefUse;
pub use compiler::passes::tree_gen::TreeGen;
pub use compiler::passes::control_flow::ControlFlowAnalysis;
pub use compiler::passes::trace_gen::TraceGen;
pub use compiler::passes::gen_mov_phi::GenMovPhi;
pub use compiler::passes::dot_gen::DotGen;

use std::any::Any;

#[allow(unused_variables)]
pub trait CompilerPass {
    fn name(&self) -> &'static str;
    fn as_any(&self) -> &Any;

    fn execute(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        info!("---CompilerPass {} for {}---", self.name(), func);

        self.start_function(vm, func);
        self.visit_function(vm, func);
        self.finish_function(vm, func);

        info!("---finish---");
    }

    fn visit_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        for (label, ref mut block) in func.content.as_mut().unwrap().blocks.iter_mut() {
            trace!("block: {}", label);

            self.start_block(vm, &mut func.context, block);
            self.visit_block(vm, &mut func.context, block);
            self.finish_block(vm, &mut func.context, block);
        }
    }

    fn visit_block(&mut self, vm: &VM, func_context: &mut FunctionContext, block: &mut Block) {
        for inst in block.content.as_mut().unwrap().body.iter_mut() {
            trace!("{}", inst);

            self.visit_inst(vm, func_context, inst);
        }
    }

    fn start_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {}
    fn finish_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {}

    fn start_block(&mut self, vm: &VM, func_context: &mut FunctionContext, block: &mut Block) {}
    fn finish_block(&mut self, vm: &VM, func_context: &mut FunctionContext, block: &mut Block) {}

    fn visit_inst(&mut self, vm: &VM, func_context: &mut FunctionContext, node: &TreeNode) {}
}
