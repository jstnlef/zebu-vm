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

use utils::Address;
use utils::ByteSize;
use std::ops::Deref;
use std::ops::DerefMut;
use std::fmt;
use std::mem::transmute;

#[repr(C)]
pub struct Raw<T: RawMemoryMetadata> {
    inner: *mut T
}

impl<T: RawMemoryMetadata> Raw<T> {
    pub unsafe fn from_ptr(ptr: *mut T) -> Raw<T> {
        debug_assert!(!ptr.is_null());
        Raw { inner: ptr }
    }
    pub unsafe fn from_addr(addr: Address) -> Raw<T> {
        debug_assert!(!addr.is_zero());
        Raw {
            inner: addr.to_ptr_mut()
        }
    }
    pub fn addr(&self) -> Address {
        Address::from_mut_ptr(self.inner)
    }
}

impl<T: RawMemoryMetadata> Clone for Raw<T> {
    fn clone(&self) -> Self {
        Raw { inner: self.inner }
    }
}

impl<T: RawMemoryMetadata> Deref for Raw<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { transmute(self.inner) }
    }
}

impl<T: RawMemoryMetadata> DerefMut for Raw<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { transmute(self.inner) }
    }
}

impl<T: fmt::Debug + RawMemoryMetadata> fmt::Debug for Raw<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:?}", &self)
    }
}

impl<T: fmt::Display + RawMemoryMetadata> fmt::Display for Raw<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", &self)
    }
}

unsafe impl<T: RawMemoryMetadata + Send> Send for Raw<T> {}
unsafe impl<T: RawMemoryMetadata + Sync> Sync for Raw<T> {}

pub trait RawMemoryMetadata {
    /// the address of the metadata
    fn addr(&self) -> Address;
    /// the start address of the memory area (after the metadata)
    fn mem_start(&self) -> Address;
}
