// Copyright 2017 The Australian National University
// 
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// 
//     http://www.apache.org/licenses/LICENSE-2.0
// 
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Utility crate that serves Zebu
//!
//! It includes:
//!
//! * data structures
//!   * double linked list
//!   * linked hashmap/set
//! * extra functions for existing types
//!   * string
//!   * vector
//! * Address/ObjectReference type
//! * utility functions for
//!   * memory
//!   * mathematics
//!   * bit operations

#[macro_use]
extern crate rodal;
extern crate byteorder;
extern crate doubly;

// these type aliases make source code easier to read

/// size in bits
pub type BitSize    = usize;
/// size in bytes
pub type ByteSize   = usize;
/// offset in byte
pub type ByteOffset = isize;
/// word value
pub type Word       = usize;

/// Zebu make an assumption that it will only support 64 bits architecture
/// However, ideally we should always use pointer size, or pointer-size type defined here.
/// But we may have hard coded u64 or 64 somewhere.
//  TODO: fix the hard code
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub const LOG_POINTER_SIZE : usize = 3;

/// pointer size in byte
pub const POINTER_SIZE     : ByteSize = 1 << LOG_POINTER_SIZE;
/// word size in byte
pub const WORD_SIZE        : ByteSize = 1 << LOG_POINTER_SIZE;

/// linked hashmap implementation copied from container-rs with modification
mod linked_hashmap;
/// linked hashset implementation based on LinkedHashMap
mod linked_hashset;

// re-export these data structures

pub use linked_hashmap::LinkedHashMap;
pub use linked_hashset::LinkedHashSet;
pub use self::doubly::DoublyLinkedList;

/// mem module:
/// * conversions of bit representations
/// * re-export memmap and memsec crate
pub mod mem;

/// mathematics utilities
pub mod math;

mod address;
/// Address represents an arbitrary memory address (valid or not)
pub use address::Address;
/// ObjectReference is a reference to an object (the address is guaranteed to be valid with an object)
pub use address::ObjectReference;

// These modules provide operations on Vector, and String.
// They are not found in the standard library.
// (maybe because they are likely inefficient?)
/// vector utilities
pub mod vec_utils;
/// string utilities
pub mod string_utils;
/// bit operations
pub mod bit_utils;

/// the macro to create LinkedHashMap
#[macro_export]
macro_rules! linked_hashmap {
    (@single $($x:tt)*) => (());
    (@count $($rest:expr),*) => (<[()]>::len(&[$(linked_hashmap!(@single $rest)),*]));

    ($($key:expr => $value:expr,)+) => { linked_hashmap!($($key => $value),+) };
    ($($key:expr => $value:expr),*) => {
        {
            let _cap = linked_hashmap!(@count $($key),*);
            let mut _map = LinkedHashMap::with_capacity(_cap);
            $(
                _map.insert($key, $value);
            )*
            _map
        }
    };
}

/// the macro to create LinkedHashSet
#[macro_export]
macro_rules! linked_hashset {
    (@single $($x:tt)*) => (());
    (@count $($rest:expr),*) => (<[()]>::len(&[$(linked_hashset!(@single $rest)),*]));

    ($($value:expr,)+) => { linked_hashset!($($value),+) };
    ($($value:expr),*) => {
        {
            let _cap = linked_hashset!(@count $($key),*);
            let mut _map = LinkedHashSet::with_capacity(_cap);
            $(
                _map.insert($value);
            )*
            _map
        }
    };
}

/// print trace!() log if condition is true (the condition should be a constant boolean)
#[macro_export]
macro_rules! trace_if {
    ($cond: expr, $($arg:tt)*) => {
        if $cond {
            trace!($($arg)*)
        }
    }
}

/// print info!() log if condition is true (the condition should be a constant boolean)
#[macro_export]
macro_rules! info_if {
    ($cond: expr, $($arg:tt)*) => {
        if $cond {
            info!($($arg)*)
        }
    }
}

/// print debug!() log if condition is true (the condition should be a constant boolean)
#[macro_export]
macro_rules! debug_if {
    ($cond: expr, $($arg:tt)*) => {
        if $cond {
            debug!($($arg)*)
        }
    }
}

/// print warn!() log if condition is true (the condition should be a constant boolean)
#[macro_export]
macro_rules! warn_if {
    ($cond: expr, $($arg:tt)*) => {
        if $cond {
            warn!($($arg)*)
        }
    }
}

/// print error!() log if condition is true (the condition should be a constant boolean)
#[macro_export]
macro_rules! error_if {
    ($cond: expr, $($arg:tt)*) => {
        if $cond {
            error!($($arg)*)
        }
    }
}
