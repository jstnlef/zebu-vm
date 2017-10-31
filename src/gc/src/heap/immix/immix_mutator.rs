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

use heap::Mutator;
use heap::immix::*;
use heap::immix::ImmixSpace;
use heap::immix::immix_space::ImmixBlock;
use heap::gc;
use objectmodel;
use objectmodel::sidemap::*;
use utils::Address;
use utils::ByteSize;
use common::ptr::*;

use std::*;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::{AtomicBool, Ordering};

const TRACE_ALLOC: bool = false;

#[repr(C)]
pub struct ImmixAllocator {
    // cursor might be invalid, but Option<Address> is expensive here
    // after every GC, we set both cursor and limit
    // to Address::zero() so that alloc will branch to slow path
    cursor: Address,
    limit: Address,
    line: usize,
    block: Option<Raw<ImmixBlock>>,

    large_cursor: Address,
    large_limit: Address,
    large_block: Option<Raw<ImmixBlock>>,

    space: Raw<ImmixSpace>,
    mutator: *mut Mutator
}

lazy_static! {
    pub static ref CURSOR_OFFSET : usize = offset_of!(ImmixAllocator=>cursor).get_byte_offset();
    pub static ref LIMIT_OFFSET  : usize = offset_of!(ImmixAllocator=>limit).get_byte_offset();
}

impl ImmixAllocator {
    pub fn reset(&mut self) -> () {
        unsafe {
            // should not use Address::zero() other than initialization
            self.cursor = Address::zero();
            self.limit = Address::zero();
            self.large_cursor = Address::zero();
            self.large_limit = Address::zero();
        }
        self.line = LINES_IN_BLOCK;
        self.block = None;
        self.large_block = None;
    }

    pub fn reset_after_gc(&mut self) {
        self.reset();
    }

    pub fn new(space: Raw<ImmixSpace>) -> ImmixAllocator {
        ImmixAllocator {
            cursor: unsafe { Address::zero() },
            limit: unsafe { Address::zero() },
            line: LINES_IN_BLOCK,
            block: None,
            large_cursor: unsafe { Address::zero() },
            large_limit: unsafe { Address::zero() },
            large_block: None,
            space,
            mutator: ptr::null_mut()
        }
    }

    pub fn set_mutator(&mut self, mutator: *mut Mutator) {
        self.mutator = mutator;
    }

    pub fn destroy(&mut self) {
        self.return_block(true);
        self.return_block(false);
    }

    #[inline(always)]
    pub fn alloc(&mut self, size: usize, align: usize) -> Address {
        // this part of code will slow down allocation
        let align = objectmodel::check_alignment(align);
        // end

        trace_if!(
            TRACE_ALLOC,
            "Mutator: fastpath alloc: size={}, align={}",
            size,
            align
        );

        let start = self.cursor.align_up(align);
        let end = start + size;

        trace_if!(
            TRACE_ALLOC,
            "Mutator: fastpath alloc: start=0x{:x}, end=0x{:x}",
            start,
            end
        );

        if end > self.limit {
            if size > BYTES_IN_LINE {
                trace_if!(TRACE_ALLOC, "Mutator: overflow alloc()");
                self.overflow_alloc(size, align)
            } else {
                trace_if!(
                    TRACE_ALLOC,
                    "Mutator: fastpath alloc: try_alloc_from_local()"
                );
                self.try_alloc_from_local(size, align)
            }
        } else {
            self.cursor = end;
            start
        }
    }

    #[inline(never)]
    pub fn overflow_alloc(&mut self, size: usize, align: usize) -> Address {
        let start = self.large_cursor.align_up(align);
        let end = start + size;

        trace_if!(
            TRACE_ALLOC,
            "Mutator: overflow alloc: start={}, end={}",
            start,
            end
        );

        if end > self.large_limit {
            self.alloc_from_global(size, align, true)
        } else {
            self.large_cursor = end;
            start
        }
    }

    #[inline(always)]
    #[cfg(feature = "use-sidemap")]
    pub fn init_object<T>(&mut self, addr: Address, encode: T) {
        let map_slot = ImmixSpace::get_type_byte_slot_static(addr);
        unsafe {
            map_slot.store(encode);
        }
    }

    #[inline(always)]
    #[cfg(not(feature = "use-sidemap"))]
    pub fn init_object(&mut self, addr: Address, encode: u64) {
        unsafe {
            (addr + objectmodel::OBJECT_HEADER_OFFSET).store(encode);
        }
    }

    #[inline(always)]
    #[cfg(feature = "use-sidemap")]
    pub fn init_hybrid<T>(&mut self, addr: Address, encode: T, len: u64) {
        unimplemented!()
    }

    #[inline(always)]
    #[cfg(not(feature = "use-sidemap"))]
    pub fn init_hybrid(&mut self, addr: Address, encode: u64, len: u64) {
        let encode =
            encode | ((len << objectmodel::SHR_HYBRID_LENGTH) & objectmodel::MASK_HYBRID_LENGTH);
        unsafe {
            (addr + objectmodel::OBJECT_HEADER_OFFSET).store(encode);
        }
    }

    #[inline(never)]
    pub fn try_alloc_from_local(&mut self, size: usize, align: usize) -> Address {
        if self.line < LINES_IN_BLOCK {
            let opt_next_available_line = {
                let cur_line = self.line;
                self.block().get_next_available_line(cur_line)
            };
            trace_if!(
                TRACE_ALLOC,
                "Mutator: alloc from local, next available line: {:?}",
                opt_next_available_line
            );

            match opt_next_available_line {
                Some(next_available_line) => {
                    // we can alloc from local blocks
                    let end_line = self.block().get_next_unavailable_line(next_available_line);

                    self.cursor = self.block().mem_start() +
                        ((next_available_line as usize) << LOG_BYTES_IN_LINE);
                    self.limit =
                        self.block().mem_start() + ((end_line as usize) << LOG_BYTES_IN_LINE);
                    self.line = end_line;

                    unsafe {
                        self.cursor.memset(0, self.limit - self.cursor);
                    }

                    for line in next_available_line..end_line {
                        self.block().set_line_mark(line, LineMark::FreshAlloc);
                    }

                    // allocate fast path
                    let start = self.cursor.align_up(align);
                    let end = start + size;

                    self.cursor = end;
                    start
                }
                None => self.alloc_from_global(size, align, false)
            }
        } else {
            // we need to alloc from global space
            self.alloc_from_global(size, align, false)
        }
    }

    fn alloc_from_global(&mut self, size: usize, align: usize, request_large: bool) -> Address {
        trace!("Mutator: slowpath: alloc_from_global()");
        self.return_block(request_large);

        loop {
            // check if yield
            unsafe { &mut *self.mutator }.yieldpoint();

            let new_block: Option<Raw<ImmixBlock>> = self.space.get_next_usable_block();

            match new_block {
                Some(b) => {
                    // zero the block - do not need to zero the block here
                    // we zero lines that get used in try_alloc_from_local()
                    //                    b.lazy_zeroing();

                    if request_large {
                        self.large_cursor = b.mem_start();
                        self.limit = b.mem_start() + BYTES_IN_BLOCK;
                        self.large_block = Some(b);

                        return self.alloc(size, align);
                    } else {
                        self.cursor = b.mem_start();
                        self.limit = b.mem_start();
                        self.line = 0;
                        self.block = Some(b);

                        trace!(
                            "Mutator: slowpath: new block starting from 0x{:x}",
                            self.cursor
                        );

                        return self.try_alloc_from_local(size, align);
                    }
                }
                None => {
                    continue;
                }
            }
        }
    }

    pub fn prepare_for_gc(&mut self) {
        self.return_block(true);
        self.return_block(false);
    }

    fn return_block(&mut self, request_large: bool) {
        if request_large {
            if self.large_block.is_some() {
                trace!(
                    "finishing large block {}",
                    self.large_block.as_ref().unwrap().addr()
                );
                self.space
                    .return_used_block(self.large_block.take().unwrap());
            }
        } else {
            if self.block.is_some() {
                trace!("finishing block {}", self.block.as_ref().unwrap().addr());
                self.space.return_used_block(self.block.take().unwrap());
            }
        }
    }

    fn block(&mut self) -> &mut ImmixBlock {
        self.block.as_mut().unwrap()
    }

    pub fn print_object(&self, obj: Address, length: usize) {
        ImmixAllocator::print_object_static(obj, length);
    }

    pub fn print_object_static(obj: Address, length: usize) {
        debug!("===Object {:#X} size: {} bytes===", obj, length);
        let mut cur_addr = obj;
        while cur_addr < obj + length {
            debug!("Address: {:#X}   {:#X}", cur_addr, unsafe {
                cur_addr.load::<u64>()
            });
            cur_addr = cur_addr + 8 as ByteSize;
        }
        debug!("----");
        debug!("=========");
    }
}

impl fmt::Display for ImmixAllocator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.cursor.is_zero() {
            write!(f, "Mutator (not initialized)").unwrap();
        } else {
            write!(f, "Mutator:\n").unwrap();
            write!(f, "cursor= {:#X}\n", self.cursor).unwrap();
            write!(f, "limit = {:#X}\n", self.limit).unwrap();
            write!(f, "line  = {}\n", self.line).unwrap();
            write!(f, "large cursor = {}\n", self.large_cursor).unwrap();
            write!(f, "large limit  = {}\n", self.large_limit).unwrap();
        }
        Ok(())
    }
}
