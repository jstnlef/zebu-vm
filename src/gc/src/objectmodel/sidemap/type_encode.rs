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

use std::mem::transmute;

/// Ref Encode
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum WordType {
    NonRef = 0,
    Ref = 1,
    WeakRef = 2,
    TaggedRef = 3
}

/// TypeEncode
#[repr(C, packed)]
pub struct TypeEncode {
    /// how many words in fixed part of the type (max 255 = ~2k bytes)
    fix_len: u8,
    /// types for each word (63 * 4 = 252 words)
    fix_ty: [u8; 63],
    /// how many words in var part of the type
    var_len: u8,
    /// types for each word
    var_ty: [u8; 63]
}

impl TypeEncode {
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
    pub fn fix_ty(&self, i: u8) -> WordType {
        debug_assert!(i < self.fix_len);
        TypeEncode::extract_ty(&self.fix_ty, i)
    }
    #[inline(always)]
    pub fn var_ty(&self, i: u8) -> WordType {
        debug_assert!(i < self.var_len);
        TypeEncode::extract_ty(&self.var_ty, i)
    }
}

#[cfg(test)]
mod type_encoding {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn struct_size() {
        assert_eq!(size_of::<TypeEncode>(), 128);
    }

    fn build_encode() -> TypeEncode {
        let mut ret = TypeEncode {
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