#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate immix_rust as gc;

#[macro_use]
pub mod utils;
pub mod ast;
pub mod vm;
pub mod compiler;
pub mod runtime;
