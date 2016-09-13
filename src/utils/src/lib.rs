extern crate byteorder;

pub type ByteSize = usize;
pub type Word = usize;

#[cfg(target_arch = "x86_64")]
pub const LOG_POINTER_SIZE : usize = 3;

pub const POINTER_SIZE     : ByteSize = 1 << LOG_POINTER_SIZE;
pub const WORD_SIZE        : ByteSize = 1 << LOG_POINTER_SIZE;

pub mod mem;

mod linked_hashset;
pub use linked_hashset::LinkedHashSet;
pub use linked_hashset::LinkedHashMap;

mod address;
pub use address::Address;
pub use address::ObjectReference;

// This porvides some missing operations on Vec.
// They are not included in the standard libarary.
// (because they are likely inefficient?)
pub mod vec_utils;

pub mod bit_utils;
pub mod string_utils;