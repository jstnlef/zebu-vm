pub extern crate immix_rust as gc;

pub use gc::common::Address;
pub use gc::common::ObjectReference;

pub type Word = usize;

pub mod thread;

pub enum RuntimeValue {
    Pointer(Address),
    Value(Word)
}