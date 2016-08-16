#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate immix_rust as gc;
extern crate rustc_serialize;

#[macro_use]
pub mod utils;
#[macro_use]
pub mod ast;
pub mod vm;
pub mod compiler;
pub mod runtime;
