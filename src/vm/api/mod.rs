pub mod api_c;      // This is pub because `api_c` can be used directly. It is just an interface.
mod api_bridge;     // This is mostly auto-generatd code, and should not be used externally.
mod api_impl;       // Mostly private. 

pub use self::api_impl::mu_fastimpl_new;
pub use self::api_impl::mu_fastimpl_new_with_opts;

mod deps {
    pub use ast::ir::WPID;
    pub use ast::ir::MuID;
    pub use ast::ir::MuName;
    pub use ast::ir::CName;
    pub use vm::handle::APIHandle;
    extern crate ast;
}