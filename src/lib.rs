#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate rustc_serialize;
extern crate stderrlog;
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate field_offset;
extern crate extprim;

#[macro_use]
pub extern crate ast;
#[macro_use]
pub extern crate utils;
pub mod vm;
pub mod compiler;
pub mod runtime;
pub mod testutil;
