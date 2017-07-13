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

/// cross-platform mmap crate
pub extern crate memmap;
/// secured memory operations: memset, memzero, etc.
pub extern crate memsec;

use Word;
#[allow(unused_imports)]    // import both endianness (we may not use big endian though)
use byteorder::{LittleEndian, BigEndian, ReadBytesExt, WriteBytesExt, ByteOrder};

/// returns bit representations for u64
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub fn u64_to_raw(val: u64) -> Word {
    let mut ret = vec![];
    ret.write_u64::<LittleEndian>(val).unwrap();
    as_word(ret)
}

/// returns bit representations for f32
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub fn f32_to_raw(val: f32) -> Word {
    let mut ret = vec![];
    ret.write_f32::<LittleEndian>(val).unwrap();
    as_word(ret)
}

/// returns bit representations for f64
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub fn f64_to_raw(val: f64) -> Word {
    let mut ret = vec![];
    ret.write_f64::<LittleEndian>(val).unwrap();
    as_word(ret)
}

/// returns bit representations for Vec<u8>
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub fn as_word(mut u8_array: Vec<u8>) -> Word {
    LittleEndian::read_uint(&mut u8_array, 8) as Word
}

#[cfg(test)]
mod tests{
    use super::*;
    use Word;
    
    #[test]
    fn test_primitive_to_raw() {
        let a : Word = 0xabcd;
        let raw = u64_to_raw(a as u64);
        
        assert_eq!(raw, a);
    }
}
