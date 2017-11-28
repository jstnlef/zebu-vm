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

extern crate libc;

/// cross-platform mmap crate
pub extern crate memmap;
/// secured memory operations: memset, memzero, etc.
pub extern crate memsec;

#[allow(unused_imports)] // import both endianness (we may not use big endian though)
use byteorder::{LittleEndian, BigEndian, ReadBytesExt, WriteBytesExt, ByteOrder};

use Address;
use ByteSize;
use std::ptr;

#[cfg(target_os = "macos")]
fn mmap_flags() -> libc::c_int {
    libc::MAP_ANON | libc::MAP_PRIVATE | libc::MAP_NORESERVE
}
#[cfg(target_os = "linux")]
fn mmap_flags() -> libc::c_int {
    libc::MAP_ANONYMOUS | libc::MAP_PRIVATE | libc::MAP_NORESERVE
}

pub fn mmap_large(size: ByteSize) -> Address {
    use self::libc::*;

    let ret = unsafe {
        mmap(
            ptr::null_mut(),
            size as size_t,
            PROT_READ | PROT_WRITE,
            mmap_flags(),
            -1,
            0
        )
    };

    if ret == MAP_FAILED {
        panic!("failed to mmap {} bytes", size);
    }

    Address::from_mut_ptr(ret)
}

pub fn munmap(addr: Address, size: ByteSize) {
    use self::libc::*;
    unsafe {
        munmap(addr.to_ptr_mut() as *mut c_void, size as size_t);
    }
}

/// malloc's and zeroes the memory
pub unsafe fn malloc_zero(size: usize) -> *mut u8 {
    use self::memsec;
    match memsec::malloc(size) {
        Some(ptr) => {
            memsec::memzero(ptr, size);
            ptr
        }
        None => panic!("failed to malloc_zero() {} bytes", size)
    }
}

/// returns bit representations for u64
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub fn u64_to_raw(val: u64) -> u64 {
    let mut ret = vec![];
    ret.write_u64::<LittleEndian>(val).unwrap();
    LittleEndian::read_uint(&mut ret, 8)
}

/// returns bit representations for f32
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub fn f32_to_raw(val: f32) -> u32 {
    let mut ret = vec![];
    ret.write_f32::<LittleEndian>(val).unwrap();
    LittleEndian::read_uint(&mut ret, 4) as u32
}

/// returns bit representations for f64
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub fn f64_to_raw(val: f64) -> u64 {
    let mut ret = vec![];
    ret.write_f64::<LittleEndian>(val).unwrap();
    LittleEndian::read_uint(&mut ret, 8)
}

#[cfg(test)]
mod tests {
    use super::*;
    use Word;

    #[test]
    fn test_primitive_to_raw() {
        let a: u64 = 0xabcd;
        let raw = u64_to_raw(a);

        assert_eq!(raw, a);
    }
}
