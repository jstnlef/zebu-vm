#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

pub mod ast;
pub mod vm;
pub mod compiler;

#[allow(dead_code)]
fn main() {
    println!("Hello, world!");
}
