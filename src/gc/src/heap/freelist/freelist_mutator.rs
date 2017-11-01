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

use common::ptr::*;
use heap::*;
use objectmodel::sidemap::*;
use std::ptr;

#[repr(C)]
pub struct FreelistAllocator {
    space: Raw<FreelistSpace>,
    mutator: *mut Mutator
}

impl FreelistAllocator {
    pub fn new(space: Raw<FreelistSpace>) -> FreelistAllocator {
        FreelistAllocator {
            space,
            mutator: ptr::null_mut()
        }
    }

    pub fn set_mutator(&mut self, mutator: *mut Mutator) {
        self.mutator = mutator;
    }

    pub fn alloc(&mut self, size: ByteSize, align: ByteSize) -> Address {
        loop {
            unsafe { &mut *self.mutator }.yieldpoint();

            let ret = self.space.alloc(size, align);

            if ret.is_zero() {
                gc::trigger_gc();
            } else {
                return ret;
            }
        }
    }

    pub fn init_object(&mut self, addr: Address, encode: LargeObjectEncode) {
        let slot = self.space.get_type_encode_slot(addr);
        unsafe {
            slot.store(encode);
        }
    }
}