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

use objectmodel::*;
use std;
use std::mem::transmute;

/// Ref Encode
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum WordType {
    NonRef = 0,
    Ref = 1,
    WeakRef = 2,
    TaggedRef = 3
}

rodal_enum!(WordType {
    NonRef,
    Ref,
    WeakRef,
    TaggedRef
});

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum TypeEncode {
    Short(ShortTypeEncode),
    Full(FullTypeEncode)
}

rodal_enum!(TypeEncode{(Short: short_enc), (Full: full_enc)});

impl TypeEncode {
    pub fn short_noref(align: ByteSize, fix_len: u8) -> TypeEncode {
        let mut enc = ShortTypeEncode::empty();
        enc.align = align;
        enc.fix_len = fix_len;
        TypeEncode::Short(enc)
    }
    pub fn short_ref() -> TypeEncode {
        let mut enc = ShortTypeEncode::empty();
        enc.align = MINIMAL_ALIGNMENT;
        enc.fix_len = 1;
        enc.set_fix_ty(0, WordType::Ref);
        TypeEncode::Short(enc)
    }
    pub fn short_weakref() -> TypeEncode {
        let mut enc = ShortTypeEncode::empty();
        enc.align = MINIMAL_ALIGNMENT;
        enc.fix_len = 1;
        enc.set_fix_ty(0, WordType::WeakRef);
        TypeEncode::Short(enc)
    }
    pub fn short_tagref() -> TypeEncode {
        let mut enc = ShortTypeEncode::empty();
        enc.align = MINIMAL_ALIGNMENT;
        enc.fix_len = 1;
        enc.set_fix_ty(0, WordType::TaggedRef);
        TypeEncode::Short(enc)
    }
    pub fn short_aggregate_fix(align: ByteSize, fix: Vec<WordType>) -> TypeEncode {
        let mut enc = ShortTypeEncode::empty();
        enc.align = align;
        let fix_len = fix.len() as u8;
        enc.fix_len = fix_len;
        for i in 0..fix_len {
            enc.set_fix_ty(i, fix[i as usize]);
        }
        TypeEncode::Short(enc)
    }
    pub fn short_hybrid(align: ByteSize, fix: Vec<WordType>, var: Vec<WordType>) -> TypeEncode {
        let mut enc = ShortTypeEncode::empty();
        enc.align = check_alignment(align);
        let fix_len = fix.len() as u8;
        enc.fix_len = fix_len;
        for i in 0..fix_len {
            enc.set_fix_ty(i, fix[i as usize]);
        }
        let var_len = var.len() as u8;
        enc.var_len = var_len;
        for i in 0..var_len {
            enc.set_var_ty(i, var[i as usize]);
        }
        TypeEncode::Short(enc)
    }
    pub fn full(align: ByteSize, fix: Vec<WordType>, var: Vec<WordType>) -> TypeEncode {
        TypeEncode::Full(FullTypeEncode { align, fix, var })
    }
    pub fn as_short(&self) -> &ShortTypeEncode {
        match self {
            &TypeEncode::Short(ref enc) => enc,
            &TypeEncode::Full(_) => panic!("trying to cast TypeEncode as ShortTypeEncode")
        }
    }
    pub fn as_full(&self) -> &FullTypeEncode {
        match self {
            &TypeEncode::Short(_) => panic!("trying to cast TypeEncode to FullTypeEncode"),
            &TypeEncode::Full(ref enc) => enc
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FullTypeEncode {
    pub align: ByteSize,
    pub fix: Vec<WordType>,
    pub var: Vec<WordType>
}

rodal_struct!(FullTypeEncode { align, fix, var });

/// TypeEncode
#[repr(C, packed)]
// Clone, PartialEq, Eq, Hash, Debug
pub struct ShortTypeEncode {
    /// alignment requirement
    align: ByteSize,
    /// how many words in fixed part of the type (max 255 = ~2k bytes)
    fix_len: u8,
    /// types for each word (63 * 4 = 252 words)
    fix_ty: [u8; 63],
    /// how many words in var part of the type
    var_len: u8,
    /// types for each word
    var_ty: [u8; 63]
}

rodal_struct!(ShortTypeEncode { align, fix_len, fix_ty, var_len, var_ty });

// manually implement clone as [u8;63] does not have clone trait
// (rust std doesnt implement all array sizes with clone)
impl Clone for ShortTypeEncode {
    fn clone(&self) -> ShortTypeEncode {
        ShortTypeEncode {
            align: self.align,
            fix_len: self.fix_len,
            fix_ty: self.fix_ty,
            var_len: self.var_len,
            var_ty: self.var_ty
        }
    }
}

impl PartialEq for ShortTypeEncode {
    fn eq(&self, other: &ShortTypeEncode) -> bool {
        if self.align != other.align {
            return false;
        }
        if self.fix_len != other.fix_len {
            return false;
        }
        if self.var_len != other.var_len {
            return false;
        }
        for i in 0..self.fix_len as usize {
            if self.fix_ty[i] != other.fix_ty[i] {
                return false;
            }
        }
        for i in 0..self.var_len as usize {
            if self.var_ty[i] != other.var_ty[i] {
                return false;
            }
        }
        true
    }
}

impl Eq for ShortTypeEncode {}

use std::hash::*;
impl Hash for ShortTypeEncode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.align.hash(state);
        self.fix_len.hash(state);
        self.fix_ty.hash(state);
        self.var_len.hash(state);
        self.var_ty.hash(state);
    }
}

use std::fmt;
impl fmt::Debug for ShortTypeEncode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ShortTypeEncode {{ ").unwrap();
        write!(f, "fix_len: {:?}, fix_ty: ", self.fix_len).unwrap();
        self.fix_ty[0..self.fix_len as usize].fmt(f).unwrap();
        write!(f, ", var_len: {:?}, var_ty: ", self.var_len).unwrap();
        self.var_ty[0..self.var_len as usize].fmt(f).unwrap();
        write!(f, "}}")
    }
}

impl ShortTypeEncode {
    pub fn new(
        align: usize,
        fix_len: u8,
        fix_ty: [u8; 63],
        var_len: u8,
        var_ty: [u8; 63]
    ) -> ShortTypeEncode {
        ShortTypeEncode {
            align,
            fix_len,
            fix_ty,
            var_len,
            var_ty
        }
    }
    fn empty() -> ShortTypeEncode {
        ShortTypeEncode {
            align: 0,
            fix_len: 0,
            fix_ty: [0; 63],
            var_len: 0,
            var_ty: [0; 63]
        }
    }
    #[inline(always)]
    pub fn align(&self) -> ByteSize {
        self.align
    }
    #[inline(always)]
    pub fn fix_len(&self) -> u8 {
        self.fix_len
    }
    #[inline(always)]
    pub fn var_len(&self) -> u8 {
        self.var_len
    }
    #[inline(always)]
    fn extract_ty(vec: &[u8; 63], i: u8) -> WordType {
        let res = vec[(i >> 2) as usize] >> ((i & 0b11) << 1);
        unsafe { transmute(res & 0b11) }
    }
    #[inline(always)]
    fn set_ty(vec: &mut [u8; 63], i: u8, ty: WordType) {
        let index = (i >> 2) as usize;
        let orig: u8 = vec[index];
        let mask: u8 = (ty as u8 & 0b11) << ((i & 0b11) << 1);
        vec[index] = orig | mask
    }
    #[inline(always)]
    pub fn fix_ty(&self, i: u8) -> WordType {
        debug_assert!(i < self.fix_len);
        ShortTypeEncode::extract_ty(&self.fix_ty, i)
    }
    pub fn set_fix_ty(&mut self, i: u8, ty: WordType) {
        debug_assert!(i < self.fix_len);
        ShortTypeEncode::set_ty(&mut self.fix_ty, i, ty);
    }
    #[inline(always)]
    pub fn var_ty(&self, i: u8) -> WordType {
        debug_assert!(i < self.var_len);
        ShortTypeEncode::extract_ty(&self.var_ty, i)
    }
    pub fn set_var_ty(&mut self, i: u8, ty: WordType) {
        debug_assert!(i < self.var_len);
        ShortTypeEncode::set_ty(&mut self.var_ty, i, ty)
    }
}

#[cfg(test)]
mod type_encoding {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn struct_size() {
        assert_eq!(size_of::<ShortTypeEncode>(), 136);
    }

    fn build_encode() -> ShortTypeEncode {
        let mut ret = ShortTypeEncode {
            align: MINIMAL_ALIGNMENT,
            fix_len: 12,
            fix_ty: [0; 63],
            var_len: 8,
            var_ty: [0; 63]
        };
        ret.fix_ty[0] = 0b11100100u8;
        ret.fix_ty[1] = 0b00011011u8;
        ret.fix_ty[2] = 0b11100100u8;
        ret
    }

    #[test]
    fn len() {
        let encode = build_encode();
        assert_eq!(encode.fix_len(), 12);
        assert_eq!(encode.var_len(), 8);
    }
    #[test]
    fn ty() {
        use super::WordType::*;
        let encode = build_encode();
        assert_eq!(encode.fix_ty(0), NonRef);
        assert_eq!(encode.fix_ty(1), Ref);
        assert_eq!(encode.fix_ty(2), WeakRef);
        assert_eq!(encode.fix_ty(3), TaggedRef);
        assert_eq!(encode.fix_ty(4), TaggedRef);
        assert_eq!(encode.fix_ty(5), WeakRef);
        assert_eq!(encode.fix_ty(6), Ref);
        assert_eq!(encode.fix_ty(7), NonRef);
        assert_eq!(encode.fix_ty(8), NonRef);
        assert_eq!(encode.fix_ty(9), Ref);
        assert_eq!(encode.fix_ty(10), WeakRef);
        assert_eq!(encode.fix_ty(11), TaggedRef);
    }
}
