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
