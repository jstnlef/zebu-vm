#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate rustc_serialize;
extern crate byteorder;
extern crate simple_logger;

#[macro_use]
pub mod utils;
#[macro_use]
pub mod ast;
pub mod vm;
pub mod compiler;
pub mod runtime;
