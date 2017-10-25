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

use common::gctype::GCType;
use objectmodel::sidemap::TypeID;
use objectmodel::sidemap::type_encode::WordType;

use std::sync::atomic;
use utils::{Address, ObjectReference};
use utils::{LOG_POINTER_SIZE, POINTER_SIZE};
use utils::bit_utils;
use utils::{ByteSize, ByteOffset};
use std::mem::transmute;

/// Tiny object encoding - [16, 32) bytes
/// Stored in a tiny object space - by address, we can know it is a tiny object
/// hi         lo
/// |s|u|r2r1r0|
/// s,  1 bit  - size encode
/// u,  1 bit  - unused
/// ri, 2 bits - ref encode for ith word
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct TinyObjectEncode {
    b: u8
}
impl TinyObjectEncode {
    #[inline(always)]
    pub fn size(self) -> usize {
        let size = ((self.b >> 7) & 0b1u8) << 3;
        (16 + size) as usize
    }
    #[inline(always)]
    pub fn n_fields(self) -> usize {
        let n = (self.b >> 7) & 0b1u8;
        (2 + n) as usize
    }
    #[inline(always)]
    pub fn field_0(self) -> WordType {
        let f = self.b & 0b11u8;
        unsafe { transmute(f) }
    }
    #[inline(always)]
    pub fn field_1(self) -> WordType {
        let f = (self.b >> 2) & 0b11u8;
        unsafe { transmute(f) }
    }
    #[inline(always)]
    pub fn field_2(self) -> WordType {
        let f = (self.b >> 4) & 0b11u8;
        unsafe { transmute(f) }
    }
}

#[cfg(test)]
mod tiny_object_encoding {
    use super::*;
    use objectmodel::sidemap::type_encode::WordType;
    use std::mem::size_of;

    #[test]
    fn struct_size() {
        assert_eq!(size_of::<TinyObjectEncode>(), 1);
    }
    const encode1: TinyObjectEncode = TinyObjectEncode { b: 0b10111001 };
    const encode2: TinyObjectEncode = TinyObjectEncode { b: 0b01001000 };
    #[test]
    fn size() {
        assert_eq!(encode1.size(), 24);
        assert_eq!(encode2.size(), 16);
    }
    #[test]
    fn fields() {
        assert_eq!(encode1.n_fields(), 3);
        assert_eq!(encode1.field_0(), WordType::Ref);
        assert_eq!(encode1.field_1(), WordType::WeakRef);
        assert_eq!(encode1.field_2(), WordType::TaggedRef);

        assert_eq!(encode2.n_fields(), 2);
        assert_eq!(encode2.field_0(), WordType::NonRef);
        assert_eq!(encode2.field_1(), WordType::WeakRef);
    }
}

/// Small object encoding - [32, 64) bytes
/// Stored in a normal object space, along with medium objects
/// hi                lo
/// |f|sz|type_id.....|
/// f,  1 bit  - small(1) or medium(0)
/// sz, 2 bits - size encode (00: 32, 01:40, 10: 48, 11: 56)
/// type_id, 13 bits - type id
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct SmallObjectEncode {
    w: u16
}

impl SmallObjectEncode {
    #[inline(always)]
    pub fn is_small(self) -> bool {
        (self.w >> 15) == 1
    }
    #[inline(always)]
    pub fn size(self) -> usize {
        debug_assert!(self.is_small());
        let size = ((self.w >> 13) & 0b11u16) << 3;
        (32 + size) as usize
    }
    #[inline(always)]
    pub fn type_id(self) -> TypeID {
        debug_assert!(self.is_small());
        (self.w & 0b0001111111111111u16) as u32
    }
}

#[cfg(test)]
mod small_object_encoding {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn struct_size() {
        assert_eq!(size_of::<SmallObjectEncode>(), 2);
    }
    const encode1: SmallObjectEncode = SmallObjectEncode {
        w: 0b1000000000000001u16
    };
    const encode2: SmallObjectEncode = SmallObjectEncode {
        w: 0b1011000000000000u16
    };
    const encode3: SmallObjectEncode = SmallObjectEncode {
        w: 0b1111000000000001u16
    };
    const encode4: SmallObjectEncode = SmallObjectEncode {
        w: 0b0101010101110101u16
    };
    #[test]
    fn is_small() {
        assert!(encode1.is_small());
        assert!(encode2.is_small());
        assert!(encode3.is_small());
        assert!(!encode4.is_small());
    }
    #[test]
    fn size() {
        assert_eq!(encode1.size(), 32);
        assert_eq!(encode2.size(), 40);
        assert_eq!(encode3.size(), 56);
    }
    #[test]
    fn type_id() {
        assert_eq!(encode1.type_id(), 1);
        assert_eq!(encode2.type_id(), 4096);
        assert_eq!(encode3.type_id(), 4097);
    }
}

/// Medium object encoding - [64, 2k)
/// Stored in a normal object space, along with small objects
/// hi                  lo
/// |f|type_id.....|size|
/// f      , 1 bit   - small(1) or medium(0)
/// type_id, 23 bits - type id
/// size   , 8 bits  - size encode (sz -> 64 + sz * 8)
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct MediumObjectEncode {
    d: u32
}

impl MediumObjectEncode {
    #[inline(always)]
    pub fn is_medium(self) -> bool {
        (self.d >> 31) == 0
    }
    #[inline(always)]
    pub fn size(self) -> usize {
        debug_assert!(self.is_medium());
        let size = (self.d & 0xFFu32) << 3;
        (64 + size) as usize
    }
    #[inline(always)]
    pub fn type_id(self) -> TypeID {
        debug_assert!(self.is_medium());
        self.d >> 8
    }
}

#[cfg(test)]
mod medium_object_encoding {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn struct_size() {
        assert_eq!(size_of::<MediumObjectEncode>(), 4);
    }
    const encode1: MediumObjectEncode = MediumObjectEncode {
        d: 0b0000_0000_0000_0000_0000_0000_0000_0000u32
    };
    const encode2: MediumObjectEncode = MediumObjectEncode {
        d: 0b0100_0000_0000_0000_0000_0001_1000_0000u32
    };
    const encode3: MediumObjectEncode = MediumObjectEncode {
        d: 0b0111_1111_1111_1111_1111_1111_1111_1101u32
    };
    const encode4: MediumObjectEncode = MediumObjectEncode {
        d: 0b1100_0000_0000_0000_0000_0001_1111_1111u32
    };
    #[test]
    fn is_medium() {
        assert!(encode1.is_medium());
        assert!(encode2.is_medium());
        assert!(encode3.is_medium());
        assert!(!encode4.is_medium());
    }
    #[test]
    fn size() {
        assert_eq!(encode1.size(), 64);
        assert_eq!(encode2.size(), 1088);
        assert_eq!(encode3.size(), 2088);
    }
    #[test]
    fn type_id() {
        assert_eq!(encode1.type_id(), 0);
        assert_eq!(encode2.type_id(), 4194305);
        assert_eq!(encode3.type_id(), 8388607);
    }
}

/// Large object encoding - [2k, *)
/// Stored in a large object space - by address, we can know it is a large object
/// Header is used for it
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct LargeObjectEncode {
    size: u64,
    tyid: u32,
    unused: u32
}

impl LargeObjectEncode {
    #[inline(always)]
    pub fn size(self) -> usize {
        (self.size << 8) as usize
    }
    #[inline(always)]
    pub fn type_id(self) -> TypeID {
        self.tyid
    }
}

#[cfg(test)]
mod large_object_encoding {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn struct_size() {
        assert_eq!(size_of::<LargeObjectEncode>(), 16);
    }
}
