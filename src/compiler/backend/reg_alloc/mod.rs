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

//! This module is for register allocation.
//! We should encapsulate the details of register allocation within this module,
//! and expose RegisterAllocation (which implements CompilerPass) to the compiler.
//! Outside of this module, the compiler should not assume a specific register
//! allocation algorithm.
//! We currently implemented graph coloring. We may have other algorithms in
//! the future.

/// a graph coloring implementation (mostly based on Appel's compiler book).
pub mod graph_coloring;
/// a register allocation validation pass. Design is discussed in Issue #19.
/// This pass is controlled by --disable-regalloc-validate option
/// (currently disabled for all cases due to bugs)
mod validate;

/// exposing graph coloring register allocation pass.
pub use compiler::backend::reg_alloc::graph_coloring::RegisterAllocation;
