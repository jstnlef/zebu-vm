#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

#[macro_use]
mod common;

pub mod ast;
pub mod vm;
pub mod compiler;
mod utils;

#[allow(dead_code)]
fn main() {
    println!("Hello, world!");
}
