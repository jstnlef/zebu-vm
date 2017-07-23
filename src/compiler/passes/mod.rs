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
use std::any::Any;

/// An inlining pass. Based on a certain criteria, the compiler chooses certain functions to be
/// inlined in their callsite by rewriting the call into a branch with several copied blocks from
/// the inlined function
mod inlining;
pub use compiler::passes::inlining::Inlining;

/// A Def-Use pass. Getting use info and count for SSA variables in the IR (we are not collecting
/// define info)
mod def_use;
pub use compiler::passes::def_use::DefUse;

/// A tree generation pass. Mu IR is a flat IR instruction sequence, this pass turns it into a
/// depth tree which is easier for instruction selection.
mod tree_gen;
pub use compiler::passes::tree_gen::TreeGen;

/// A phi node eliminating pass. Mu IR is SSA based with goto-with-values variants, it still has
/// phi node (implicitly). We get out of SSA form at this pass by removing phi nodes, and inserting
/// intermediate blocks for moving values around.
mod gen_mov_phi;
pub use compiler::passes::gen_mov_phi::GenMovPhi;

/// A control flow analysis pass at IR level.
mod control_flow;
pub use compiler::passes::control_flow::ControlFlowAnalysis;

/// A trace scheduling pass. It uses the CFA result from last pass, to schedule blocks that
/// favors hot path execution.
mod trace_gen;
pub use compiler::passes::trace_gen::TraceGen;

/// A pass to generate dot graph for current IR.
mod dot_gen;
pub use compiler::passes::dot_gen::DotGen;

/// A trait for implementing compiler passes.
///
/// A Mu function is supposed to be travelled in the following order:
/// * start_function()
/// * visit_function()
///   for each block
///   * start_block()
///   * visit_block()
///     for each instruction
///     * visit_inst()
///   * finish_block()
/// * finish_function()
///
/// functions can be overridden for each pass' own purpose.
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
