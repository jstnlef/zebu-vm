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

use std::mem;
use utils::POINTER_SIZE;
use utils::LOG_POINTER_SIZE;
use utils::Address;
use heap::gc::malloc_zero;

#[derive(Clone)]
pub struct AddressMap<T: Copy> {
    start : Address,
    end   : Address,
    
    pub ptr   : *mut T,
    len   : usize
}

impl <T> AddressMap<T> where T: Copy{
    pub fn new(start: Address, end: Address) -> AddressMap<T> {
        let len = (end - start) >> LOG_POINTER_SIZE;
        let ptr = unsafe{malloc_zero(mem::size_of::<T>() * len)} as *mut T;
        
        AddressMap{start: start, end: end, ptr: ptr, len: len}
    }
    
    pub fn init_all (&self, init: T) {
        let mut cursor = self.start;
        
        while cursor < self.end {
            self.set(cursor, init);
            cursor = cursor + POINTER_SIZE;
        }
    }
    
    #[inline(always)]
    pub fn set(&self, addr: Address, value: T) {
        let index = ((addr - self.start) >> LOG_POINTER_SIZE) as isize;
        unsafe{*self.ptr.offset(index) = value};
    }

    #[inline(always)]
    pub fn get(&self, addr: Address) -> T {
        let index = ((addr - self.start) >> LOG_POINTER_SIZE) as isize;
        unsafe {*self.ptr.offset(index)}
    }
}
