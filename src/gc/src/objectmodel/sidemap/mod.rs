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

//! # Sidemap Encoding
//!
//! Terminology
//!
//! * GC byte
//!   a byte for GC information (such as trace byte, ref count, etc)
//! * Type bytes
//!   several bytes to store type-related information for an object, GC
//!   needs the information to properly trace object
//! * Ref encode
//!   encode whether a word (8 bytes) is a reference or non-reference;
//!   and if it is a reference, which kind of reference it is (weakref,
//!   tagged ref, or normal ref)
//! * Size encode
//!   encode the size of the object. How the size is encoded depends on
//!   object sizes
//! * Type ID
//!   a type ID that allows us to indirectly find type information
//!
//! Design Goal
//!
//! We aim for a 1/8 constant space cost for the object encoding.
//! Min object size is 16 bytes. We always reserve 1 byte per 16 bytes as *GC byte*.
//! GC bytes are in a separate table opposed to *type bytes*.
//!
//! Ref Encode
//!
//! We need 2 bits per word to encode reference kinds
//! * 00: non ref
//! * 01: normal ref
//! * 10: weak ref
//! * 11: tagged ref
//!
//! Object Size and Categories
//!
//! We categorize objects into 4 kinds, we use different type bytes encoding
//! for different kinds
//!
//! * tiny object - [16, 32) bytes
//!   Stored in a tiny object space - by address, we can know it is a tiny object
//!
//!   1 type byte : 6 bits - ref encode (2 bits per word, 3 words at most (for 24 bytes objects))
//!                 1 bit  - size encode (00: 16 bytes, 01: 24 bytes)
//!                 1 bit  - unused
//!
//! * small object - [32, 64) bytes
//!   Stored in a normal object space, along with medium objects
//!
//!   2 type bytes: 1 bit   - small or medium object
//!                 2 bits  - size encode (32, 40, 48, 56 bytes)
//!                 13 bits - type ID
//!
//! * medium object - [64, 2k)
//!   Stored in a normal object space, along with small objects
//!
//!   4 type bytes: 1 bit   - small or medium object
//!                 8 bits  - size encode (64, 72, ... 2k-8 bytes)
//!                 23 bits - type ID
//!
//! * large object - [2k, *)
//!   Stored in a large object space - by address, we can know it is a large object
//!   We use header
//!
//!   16 type bytes: 8 bytes - object size (u32::MAX << 3 = ~12G)
//!                  4 bytes - type ID
//!                  4 bytes - unused

use utils::*;

pub const MINIMAL_ALIGNMENT: ByteSize = 16;
pub const MINIMAL_OBJECT_SIZE: ByteSize = 16;

pub const OBJECT_HEADER_SIZE: ByteSize = 0;
pub const OBJECT_HEADER_OFFSET: ByteOffset = 0;

/// Type ID (but we never use more than 23 bits of it)
pub type TypeID = usize;
pub const N_TYPES: usize = 1 << 23;

mod object_encode;
mod type_encode;
mod global_type_table;

pub use objectmodel::sidemap::object_encode::*;
pub use objectmodel::sidemap::type_encode::*;
pub use objectmodel::sidemap::global_type_table::*;