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

use utils::LOG_POINTER_SIZE;
use utils::Address;
use common::bitmap::Bitmap;

#[derive(Clone)]
pub struct AddressBitmap {
    start: Address,
    end: Address,

    bitmap: Bitmap,
}

impl AddressBitmap {
    pub fn new(start: Address, end: Address) -> AddressBitmap {
        let bitmap_len = (end - start) >> LOG_POINTER_SIZE;
        let bitmap = Bitmap::new(bitmap_len);

        AddressBitmap {
            start: start,
            end: end,
            bitmap: bitmap,
        }
    }

    #[inline(always)]
    #[allow(mutable_transmutes)]
    pub unsafe fn set_bit(&self, addr: Address) {
        use std::mem;
        let mutable_bitmap: &mut Bitmap = mem::transmute(&self.bitmap);
        mutable_bitmap.set_bit((addr - self.start) >> LOG_POINTER_SIZE);
    }

    #[inline(always)]
    #[allow(mutable_transmutes)]
    pub unsafe fn clear_bit(&self, addr: Address) {
        use std::mem;
        let mutable_bitmap: &mut Bitmap = mem::transmute(&self.bitmap);
        mutable_bitmap.clear_bit((addr - self.start) >> LOG_POINTER_SIZE);
    }

    #[inline(always)]
    pub fn test_bit(&self, addr: Address) -> bool {
        self.bitmap
            .test_bit((addr - self.start) >> LOG_POINTER_SIZE)
    }

    #[inline(always)]
    pub fn length_until_next_bit(&self, addr: Address) -> usize {
        self.bitmap
            .length_until_next_bit((addr - self.start) >> LOG_POINTER_SIZE)
    }

    #[inline(always)]
    #[allow(mutable_transmutes)]
    pub unsafe fn set(&self, addr: Address, value: u64, length: usize) {
        use std::mem;

        if cfg!(debug_assertions) {
            assert!(addr >= self.start && addr <= self.end);
        }

        let index = (addr - self.start) >> LOG_POINTER_SIZE;
        let mutable_bitmap: &mut Bitmap = mem::transmute(&self.bitmap);
        mutable_bitmap.set(index, value, length);
    }

    #[inline(always)]
    pub fn get(&self, addr: Address, length: usize) -> u64 {
        if cfg!(debug_assertions) {
            assert!(addr >= self.start && addr <= self.end);
        }

        let index = (addr - self.start) >> LOG_POINTER_SIZE;
        self.bitmap.get(index, length)
    }

    pub fn print(&self) {
        self.bitmap.print();
    }
}
