mod liveness;
mod coloring;

pub use compiler::backend::reg_alloc::graph_coloring::liveness::InterferenceGraph;
//pub use compiler::backend::reg_alloc::graph_coloring::liveness::build as build_inteference_graph;
pub use compiler::backend::reg_alloc::graph_coloring::liveness::build_chaitin_briggs as build_inteference_graph;
pub use compiler::backend::reg_alloc::graph_coloring::coloring::GraphColoring;