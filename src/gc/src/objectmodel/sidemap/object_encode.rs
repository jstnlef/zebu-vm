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

use objectmodel::sidemap::TypeID;
use objectmodel::sidemap::type_encode::WordType;

use utils::*;
use std::mem::size_of;
use std::mem::transmute;
use std::fmt;

#[derive(Copy, Clone)]
pub enum ObjectEncode {
    Tiny(TinyObjectEncode),
    Small(SmallObjectEncode),
    Medium(MediumObjectEncode),
    Large(LargeObjectEncode)
}

impl ObjectEncode {
    pub fn tiny(self) -> TinyObjectEncode {
        match self {
            ObjectEncode::Tiny(enc) => enc,
            _ => panic!()
        }
    }
    pub fn small(self) -> SmallObjectEncode {
        match self {
            ObjectEncode::Small(enc) => enc,
            _ => panic!()
        }
    }
    pub fn medium(self) -> MediumObjectEncode {
        match self {
            ObjectEncode::Medium(enc) => enc,
            _ => panic!()
        }
    }
    pub fn large(self) -> LargeObjectEncode {
        match self {
            ObjectEncode::Large(enc) => enc,
            _ => panic!()
        }
    }
    pub fn as_raw(&self) -> [u64; 3] {
        debug_assert_eq!(size_of::<ObjectEncode>(), 24);
        unsafe {
            let ptr: *const u64 = transmute(self as *const ObjectEncode);
            let word0 = *ptr;
            let word1 = *(ptr.offset(1));
            let word2 = *(ptr.offset(2));
            [word0, word1, word2]
        }
    }
}

mod object_encoding {
    #[test]
    fn struct_size() {
        println!("{:?}", size_of::<ObjectEncode>());
    }
}

// inclusive
pub const MAX_TINY_OBJECT: ByteSize = 24; // < 32
pub const MAX_SMALL_OBJECT: ByteSize = 56; // < 64
pub const MAX_MEDIUM_OBJECT: ByteSize = 2040; // < 2048

/// Tiny object encoding - [16, 32) bytes
/// Stored in a tiny object space - by address, we can know it is a tiny object
/// hi         lo
/// |s|u|r2r1r0|
/// s,  1 bit  - size encode
/// u,  1 bit  - unused
/// ri, 2 bits - ref encode for ith word
#[repr(C)]
#[derive(Copy, Clone)]
pub struct TinyObjectEncode {
    b: u8
}
impl TinyObjectEncode {
    pub fn new(b: u8) -> TinyObjectEncode {
        TinyObjectEncode { b }
    }
    pub fn create(sz: ByteSize, f0: WordType, f1: WordType, f2: WordType) -> TinyObjectEncode {
        let mut enc = TinyObjectEncode { b: 0 };
        enc.set_size(sz);
        enc.set_field(0, f0);
        enc.set_field(1, f1);
        enc.set_field(2, f2);
        enc
    }
    #[inline(always)]
    pub fn size(self) -> usize {
        let size = ((self.b >> 7) & 0b1u8) << 3;
        (16 + size) as usize
    }
    fn set_size(&mut self, size: ByteSize) {
        if size == 24 {
            self.b = bit_utils::set_bit_u8(self.b, 0b1000_0000u8);
        } else {
            self.b = bit_utils::clear_bit_u8(self.b, 0b1000_0000u8);
        }
    }
    #[inline(always)]
    pub fn n_fields(self) -> usize {
        let n = (self.b >> 7) & 0b1u8;
        (2 + n) as usize
    }
    #[inline(always)]
    pub fn field(self, i: usize) -> WordType {
        let f = (self.b >> (i << 1)) & 0b11u8;
        unsafe { transmute(f) }
    }
    fn set_field(&mut self, i: usize, ty: WordType) {
        self.b |= (ty as u8) << (i << 1);
    }
    pub fn as_u64(self) -> u64 {
        self.b as u64
    }
}

impl fmt::Debug for TinyObjectEncode {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "TinyObjectEncode ({:08b})", self.b)
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
    const ENCODE1: TinyObjectEncode = TinyObjectEncode { b: 0b10111001 };
    const ENCODE2: TinyObjectEncode = TinyObjectEncode { b: 0b01001000 };
    #[test]
    fn create() {
        let encode =
            TinyObjectEncode::create(24, WordType::Ref, WordType::WeakRef, WordType::TaggedRef);
        assert_eq!(encode.size(), 24);
        assert_eq!(encode.field(0), WordType::Ref);
        assert_eq!(encode.field(1), WordType::WeakRef);
        assert_eq!(encode.field(2), WordType::TaggedRef);

        let encode2 =
            TinyObjectEncode::create(16, WordType::Ref, WordType::NonRef, WordType::WeakRef);
        assert_eq!(encode2.size(), 16);
        assert_eq!(encode2.field(0), WordType::Ref);
        assert_eq!(encode2.field(1), WordType::NonRef);
        assert_eq!(encode2.field(2), WordType::WeakRef);
    }
    #[test]
    fn size() {
        assert_eq!(ENCODE1.size(), 24);
        assert_eq!(ENCODE2.size(), 16);
    }
    #[test]
    fn fields() {
        assert_eq!(ENCODE1.n_fields(), 3);
        assert_eq!(ENCODE1.field(0), WordType::Ref);
        assert_eq!(ENCODE1.field(1), WordType::WeakRef);
        assert_eq!(ENCODE1.field(2), WordType::TaggedRef);

        assert_eq!(ENCODE2.n_fields(), 2);
        assert_eq!(ENCODE2.field(0), WordType::NonRef);
        assert_eq!(ENCODE2.field(1), WordType::WeakRef);
    }
}

/// Small object encoding - [32, 64) bytes
/// Stored in a normal object space, along with medium objects
/// hi                lo
/// |f|sz|type_id.....|
/// f,  1 bit  - small(1) or medium(0)
/// sz, 2 bits - size encode (00: 32, 01:40, 10: 48, 11: 56)
/// type_id, 13 bits - type id
#[repr(C)]
#[derive(Copy, Clone)]
pub struct SmallObjectEncode {
    w: u16
}

pub const SMALL_ID_WIDTH: usize = 13;

impl SmallObjectEncode {
    pub fn new(w: u16) -> SmallObjectEncode {
        SmallObjectEncode { w }
    }
    pub fn create(size: ByteSize, type_id: TypeID) -> SmallObjectEncode {
        let mut enc = SmallObjectEncode {
            w: 0b1000_0000_0000_0000u16
        };
        enc.set_size(size);
        enc.set_type_id(type_id);
        enc
    }
    #[inline(always)]
    pub fn is_small(self) -> bool {
        (self.w >> 15) == 1
    }
    #[inline(always)]
    pub fn size(self) -> usize {
        debug_assert!(self.is_small());
        let size = ((self.w >> SMALL_ID_WIDTH) & 0b11u16) << 3;
        (32 + size) as usize
    }
    fn set_size(&mut self, size: ByteSize) {
        // set size
        self.w |= ((size as u16 - 32) >> 3) << SMALL_ID_WIDTH;
    }
    #[inline(always)]
    pub fn type_id(self) -> TypeID {
        debug_assert!(self.is_small());
        (self.w & ((1u16 << SMALL_ID_WIDTH) - 1)) as usize
    }
    fn set_type_id(&mut self, id: TypeID) {
        let masked_id = id as u16 & ((1 << SMALL_ID_WIDTH) - 1);
        debug_assert_eq!(masked_id, id as u16);
        self.w |= masked_id;
    }
    pub fn as_u64(self) -> u64 {
        self.w as u64
    }
}

impl fmt::Debug for SmallObjectEncode {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "SmallObjectEncode ({:016b})", self.w)
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
    const ENCODE1: SmallObjectEncode = SmallObjectEncode {
        w: 0b1000000000000001u16
    };
    const ENCODE2: SmallObjectEncode = SmallObjectEncode {
        w: 0b1011000000000000u16
    };
    const ENCODE3: SmallObjectEncode = SmallObjectEncode {
        w: 0b1111000000000001u16
    };
    const ENCODE4: SmallObjectEncode = SmallObjectEncode {
        w: 0b0101010101110101u16
    };
    #[test]
    fn create() {
        let enc = SmallObjectEncode::create(32, 1);
        assert!(enc.is_small());
        assert_eq!(enc.size(), 32);
        assert_eq!(enc.type_id(), 1);

        let max_id = (1 << SMALL_ID_WIDTH) - 1;
        let enc2 = SmallObjectEncode::create(56, max_id);
        assert!(enc2.is_small());
        assert_eq!(enc2.size(), 56);
        assert_eq!(enc2.type_id(), max_id);
    }
    #[test]
    fn is_small() {
        assert!(ENCODE1.is_small());
        assert!(ENCODE2.is_small());
        assert!(ENCODE3.is_small());
        assert!(!ENCODE4.is_small());
    }
    #[test]
    fn size() {
        assert_eq!(ENCODE1.size(), 32);
        assert_eq!(ENCODE2.size(), 40);
        assert_eq!(ENCODE3.size(), 56);
    }
    #[test]
    fn type_id() {
        assert_eq!(ENCODE1.type_id(), 1);
        assert_eq!(ENCODE2.type_id(), 4096);
        assert_eq!(ENCODE3.type_id(), 4097);
    }
}

/// Medium object encoding - [64, 2k)
/// Stored in a normal object space, along with small objects
/// hi                  lo
/// |f|type_id.....|size|
/// f      , 1 bit   - small(1) or medium(0)
/// type_id, 23 bits - type id
/// size   , 8 bits  - size encode (sz -> 64 + sz * 8)
#[repr(C)]
#[derive(Copy, Clone)]
pub struct MediumObjectEncode {
    d: u32
}

impl MediumObjectEncode {
    pub fn new(d: u32) -> MediumObjectEncode {
        MediumObjectEncode { d }
    }
    pub fn create(size: ByteSize, type_id: TypeID) -> MediumObjectEncode {
        let mut enc = MediumObjectEncode { d: 0 };
        enc.set_size(size);
        enc.set_type_id(type_id);
        enc
    }
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
    fn set_size(&mut self, size: ByteSize) {
        // set size
        self.d |= (size as u32 - 64) >> 3;
    }
    #[inline(always)]
    pub fn type_id(self) -> TypeID {
        debug_assert!(self.is_medium());
        (self.d >> 8) as usize
    }
    fn set_type_id(&mut self, id: TypeID) {
        let masked_id = id as u32 & 0x7FFFFFu32;
        debug_assert_eq!(masked_id, id as u32);
        self.d |= masked_id << 8;
    }
    pub fn as_u64(self) -> u64 {
        self.d as u64
    }
}

impl fmt::Debug for MediumObjectEncode {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "MediumObjectEncode ({:032b})", self.d)
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
    const ENCODE1: MediumObjectEncode = MediumObjectEncode {
        d: 0b0000_0000_0000_0000_0000_0000_0000_0000u32
    };
    const ENCODE2: MediumObjectEncode = MediumObjectEncode {
        d: 0b0100_0000_0000_0000_0000_0001_1000_0000u32
    };
    const ENCODE3: MediumObjectEncode = MediumObjectEncode {
        d: 0b0111_1111_1111_1111_1111_1111_1111_1101u32
    };
    const ENCODE4: MediumObjectEncode = MediumObjectEncode {
        d: 0b1100_0000_0000_0000_0000_0001_1111_1111u32
    };
    #[test]
    fn create() {
        let enc = MediumObjectEncode::create(64, 1);
        assert!(enc.is_medium());
        assert_eq!(enc.size(), 64);
        assert_eq!(enc.type_id(), 1);
    }
    #[test]
    fn is_medium() {
        assert!(ENCODE1.is_medium());
        assert!(ENCODE2.is_medium());
        assert!(ENCODE3.is_medium());
        assert!(!ENCODE4.is_medium());
    }
    #[test]
    fn size() {
        assert_eq!(ENCODE1.size(), 64);
        assert_eq!(ENCODE2.size(), 1088);
        assert_eq!(ENCODE3.size(), 2088);
    }
    #[test]
    fn type_id() {
        assert_eq!(ENCODE1.type_id(), 0);
        assert_eq!(ENCODE2.type_id(), 4194305);
        assert_eq!(ENCODE3.type_id(), 8388607);
    }
}

/// Large object encoding - [2k, *)
/// Stored in a large object space - by address, we can know it is a large object
/// Header is used for it
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct LargeObjectEncode {
    size: usize,
    tyid: usize
}

impl LargeObjectEncode {
    #[inline(always)]
    pub fn new(size: usize, tyid: usize) -> LargeObjectEncode {
        LargeObjectEncode { size, tyid }
    }
    #[inline(always)]
    pub fn size(self) -> usize {
        self.size
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
