pub mod graph_coloring;

pub enum RegAllocFailure {
    FailedForSpilling,
}

pub use compiler::backend::reg_alloc::graph_coloring::RegisterAllocation;