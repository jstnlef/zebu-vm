#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate rustc_serialize;
extern crate simple_logger;
#[macro_use]
extern crate maplit;

#[macro_use]
pub extern crate ast;
#[macro_use]
pub extern crate utils;
pub mod vm;
pub mod compiler;
pub mod runtime;
pub mod testutil;
