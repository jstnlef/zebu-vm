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

extern crate byteorder;
extern crate rustc_serialize;

pub type BitSize    = usize;
pub type ByteOffset = isize;
pub type ByteSize   = usize;
pub type Word       = usize;

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub const LOG_POINTER_SIZE : usize = 3;

pub const POINTER_SIZE     : ByteSize = 1 << LOG_POINTER_SIZE;
pub const WORD_SIZE        : ByteSize = 1 << LOG_POINTER_SIZE;

pub mod mem;

mod linked_hashmap;
mod linked_hashset;
mod doubly;

pub use linked_hashmap::LinkedHashMap;
pub use linked_hashset::LinkedHashSet;
pub use doubly::DoublyLinkedList;

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

#[macro_export]
macro_rules! trace_if {
    ($cond: expr, $($arg:tt)*) => {
        if $cond {
            trace!($($arg)*)
        }
    }
}

#[macro_export]
macro_rules! info_if {
    ($cond: expr, $($arg:tt)*) => {
        if $cond {
            info!($($arg)*)
        }
    }
}

#[macro_export]
macro_rules! debug_if {
    ($cond: expr, $($arg:tt)*) => {
        if $cond {
            debug!($($arg)*)
        }
    }
}

#[macro_export]
macro_rules! warn_if {
    ($cond: expr, $($arg:tt)*) => {
        if $cond {
            warn!($($arg)*)
        }
    }
}

#[macro_export]
macro_rules! error_if {
    ($cond: expr, $($arg:tt)*) => {
        if $cond {
            error!($($arg)*)
        }
    }
}

pub mod math;

mod address;
pub use address::Address;
pub use address::ObjectReference;

// This porvides some missing operations on Vec.
// They are not included in the standard libarary.
// (because they are likely inefficient?)
pub mod vec_utils;

pub mod bit_utils;
pub mod string_utils;
