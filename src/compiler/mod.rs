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

extern crate hprof;

use ast::ir::*;
use vm::VM;
use std::cell::RefCell;

/// compiler passes
pub mod passes;
/// compiler backends, include target description, and target dependent passes
pub mod backend;
/// a frame layout for a compiled function
pub mod frame;
/// machine code representation
pub mod machine_code;

pub use compiler::passes::CompilerPass;

/// name for prologue (this is not full name, but prologue name is generated from this)
pub const PROLOGUE_BLOCK_NAME: &'static str = "prologue";
/// name for epilogue (this is not full name, but epilogue name is generated from this)
pub const EPILOGUE_BLOCK_NAME: &'static str = "epilogue";

/// Zebu compiler
pub struct Compiler<'vm> {
    /// policy decides what passes to be executed
    policy: RefCell<CompilerPolicy>,
    /// a reference to vm, for compiler to query VM-wide info
    vm: &'vm VM
}

impl<'vm> Compiler<'vm> {
    /// creates a new compiler
    pub fn new(policy: CompilerPolicy, vm: &VM) -> Compiler {
        Compiler {
            policy: RefCell::new(policy),
            vm: vm
        }
    }

    /// compiles a certain function version
    pub fn compile(&self, func: &mut MuFunctionVersion) {
        info!("");
        info!("Start compiling {}", func);
        info!("");
        debug!("{:?}", func);

        // FIXME: should use function name here (however hprof::enter only accept &'static str)
        let _p = hprof::enter("Function Compilation");

        let ref mut passes = self.policy.borrow_mut().passes;
        for pass in passes.iter_mut() {
            let _p = hprof::enter(pass.name());

            pass.execute(self.vm, func);

            drop(_p);
        }

        drop(_p);
        hprof_print_timing(hprof::profiler().root());

        func.set_compiled();
        if self.vm.is_doing_jit() {
            // build exception table for this function
            unimplemented!()
        }
    }
}

/// CompilerPolicy specifies a list of ordered CompilerPasses
/// the compiler will follow the list to compile each function
pub struct CompilerPolicy {
    pub passes: Vec<Box<CompilerPass>>
}

impl CompilerPolicy {
    pub fn new(passes: Vec<Box<CompilerPass>>) -> CompilerPolicy {
        CompilerPolicy { passes: passes }
    }
}

impl Default for CompilerPolicy {
    fn default() -> Self {
        let mut passes: Vec<Box<CompilerPass>> = vec![];
        passes.push(Box::new(passes::DotGen::new(".orig")));

        // ir level passes
        passes.push(Box::new(passes::RetSink::new()));
        passes.push(Box::new(passes::Inlining::new()));
        passes.push(Box::new(passes::DefUse::new()));
        passes.push(Box::new(passes::TreeGen::new()));
        passes.push(Box::new(passes::GenMovPhi::new()));
        passes.push(Box::new(passes::ControlFlowAnalysis::new()));
        passes.push(Box::new(passes::TraceGen::new()));
        passes.push(Box::new(passes::DotGen::new(".transformed")));

        // compilation
        passes.push(Box::new(backend::inst_sel::InstructionSelection::new()));
        passes.push(Box::new(backend::reg_alloc::RegisterAllocation::new()));

        // machine code level passes
        passes.push(Box::new(backend::peephole_opt::PeepholeOptimization::new()));
        passes.push(Box::new(backend::code_emission::CodeEmission::new()));

        CompilerPolicy { passes: passes }
    }
}

// rewrite parts of the hprof crates to print via log (instead of print!())
use self::hprof::ProfileNode;
use std::rc::Rc;

fn hprof_print_timing(root: Rc<ProfileNode>) {
    info!("Timing information for {}:", root.name);
    for child in &*root.children.borrow() {
        hprof_print_child(child, 2);
    }
}

fn hprof_print_child(this: &ProfileNode, indent: usize) {
    let mut indent_str = "".to_string();
    for _ in 0..indent {
        indent_str += " ";
    }

    let parent_time = this.parent
        .as_ref()
        .map(|p| p.total_time.get())
        .unwrap_or(this.total_time.get()) as f64;
    let percent = 100.0 * (this.total_time.get() as f64 / parent_time);
    if percent.is_infinite() {
        info!(
            "{}{name} - {calls} * {each} = {total} @ {hz:.1}hz",
            indent_str,
            name = this.name,
            calls = this.calls.get(),
            each = Nanoseconds((this.total_time.get() as f64 / this.calls.get() as f64) as u64),
            total = Nanoseconds(this.total_time.get()),
            hz = this.calls.get() as f64 / this.total_time.get() as f64 * 1e9f64
        );
    } else {
        info!(
            "{}{name} - {calls} * {each} = {total} ({percent:.1}%)",
            indent_str,
            name = this.name,
            calls = this.calls.get(),
            each = Nanoseconds((this.total_time.get() as f64 / this.calls.get() as f64) as u64),
            total = Nanoseconds(this.total_time.get()),
            percent = percent
        );
    }
    for c in &*this.children.borrow() {
        hprof_print_child(c, indent + 2);
    }
}

// used to do a pretty printing of time
struct Nanoseconds(u64);

use std::fmt;
impl fmt::Display for Nanoseconds {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0 < 1_000 {
            write!(f, "{}ns", self.0)
        } else if self.0 < 1_000_000 {
            write!(f, "{:.1}us", self.0 as f64 / 1_000.)
        } else if self.0 < 1_000_000_000 {
            write!(f, "{:.1}ms", self.0 as f64 / 1_000_000.)
        } else {
            write!(f, "{:.1}s", self.0 as f64 / 1_000_000_000.)
        }
    }
}
