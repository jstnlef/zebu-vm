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

const TRACE_ALLOC_FASTPATH: bool = true;

#[repr(C)]
pub struct ImmixAllocator {
    // cursor might be invalid, but Option<Address> is expensive here
    // after every GC, we set both cursor and limit
    // to Address::zero() so that alloc will branch to slow path
    cursor: Address,
    limit: Address,
    line: u8,
    space: Raw<ImmixSpace>,
    block: Option<Raw<ImmixBlock>>,
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
        }
        self.line = LINES_IN_BLOCK as u8;
        self.block = None;
    }

    pub fn reset_after_gc(&mut self) {
        self.reset();
    }

    pub fn new(space: Raw<ImmixSpace>) -> ImmixAllocator {
        ImmixAllocator {
            cursor: unsafe { Address::zero() },
            limit: unsafe { Address::zero() },
            line: LINES_IN_BLOCK as u8,
            block: None,
            space,
            mutator: ptr::null_mut()
        }
    }

    pub fn set_mutator(&mut self, mutator: *mut Mutator) {
        self.mutator = mutator;
    }

    pub fn destroy(&mut self) {
        self.return_block();
    }

    #[inline(always)]
    pub fn alloc(&mut self, size: usize, align: usize) -> Address {
        // this part of code will slow down allocation
        let align = objectmodel::check_alignment(align);
        let size = size + objectmodel::OBJECT_HEADER_SIZE;
        // end

        if TRACE_ALLOC_FASTPATH {
            trace!("Mutator: fastpath alloc: size={}, align={}", size, align);
        }

        let start = self.cursor.align_up(align);
        let end = start + size;

        if TRACE_ALLOC_FASTPATH {
            trace!(
                "Mutator: fastpath alloc: start=0x{:x}, end=0x{:x}",
                start,
                end
            );
        }

        if end > self.limit {
            let ret = self.try_alloc_from_local(size, align);
            if TRACE_ALLOC_FASTPATH {
                trace!(
                    "Mutator: fastpath alloc: try_alloc_from_local()=0x{:x}",
                    ret
                );
            }

            if cfg!(debug_assertions) {
                if !ret.is_aligned_to(align) {
                    use std::process;
                    println!("wrong alignment on 0x{:x}, expected align: {}", ret, align);
                    process::exit(102);
                }
            }

            // this offset should be removed as well (for performance)
            ret + (-objectmodel::OBJECT_HEADER_OFFSET)
        } else {
            if cfg!(debug_assertions) {
                if !start.is_aligned_to(align) {
                    use std::process;
                    println!(
                        "wrong alignment on 0x{:x}, expected align: {}",
                        start,
                        align
                    );
                    process::exit(102);
                }
            }
            self.cursor = end;

            start + (-objectmodel::OBJECT_HEADER_OFFSET)
        }
    }

    #[inline(always)]
    #[cfg(feature = "use-sidemap")]
    pub fn init_object<T>(&mut self, addr: Address, encode: T) {
        let map_slot = ImmixBlock::get_type_map_slot_static(addr);
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
        if self.line < LINES_IN_BLOCK as u8 {
            let opt_next_available_line = {
                let cur_line = self.line;
                self.block().get_next_available_line(cur_line)
            };

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
                        self.block()
                            .line_mark_table_mut()
                            .set(line, LineMark::FreshAlloc);
                    }

                    // allocate fast path
                    let start = self.cursor.align_up(align);
                    let end = start + size;

                    self.cursor = end;
                    start
                }
                None => self.alloc_from_global(size, align)
            }
        } else {
            // we need to alloc from global space
            self.alloc_from_global(size, align)
        }
    }

    fn alloc_from_global(&mut self, size: usize, align: usize) -> Address {
        trace!("Mutator: slowpath: alloc_from_global");

        self.return_block();

        loop {
            // check if yield
            unsafe { &mut *self.mutator }.yieldpoint();

            let new_block: Option<Raw<ImmixBlock>> = self.space.get_next_usable_block();

            match new_block {
                Some(b) => {
                    // zero the block - do not need to zero the block here
                    // we zero lines that get used in try_alloc_from_local()
                    //                    b.lazy_zeroing();

                    self.block = Some(b);
                    self.cursor = self.block().mem_start();
                    self.limit = self.block().mem_start();
                    self.line = 0;

                    trace!(
                        "Mutator: slowpath: new block starting from 0x{:x}",
                        self.cursor
                    );

                    return self.try_alloc_from_local(size, align);
                }
                None => {
                    continue;
                }
            }
        }
    }

    pub fn prepare_for_gc(&mut self) {
        self.return_block();
    }

    fn return_block(&mut self) {
        if self.block.is_some() {
            trace!("finishing block {:?}", self.block.as_ref().unwrap());

            if cfg!(debug_assertions) {
                let block = self.block.as_ref().unwrap();
                ImmixAllocator::sanity_check_finished_block(block);
            }

            self.space.return_used_block(self.block.take().unwrap());
        }
    }

    #[cfg(feature = "use-sidemap")]
    #[allow(unused_variables)]
    fn sanity_check_finished_block(block: &ImmixBlock) {}

    #[cfg(not(feature = "use-sidemap"))]
    #[allow(unused_variables)]
    fn sanity_check_finished_block(block: &ImmixBlock) {}

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
            write!(f, "Mutator (not initialized)")
        } else {
            write!(f, "Mutator:\n").unwrap();
            write!(f, "cursor= {:#X}\n", self.cursor).unwrap();
            write!(f, "limit = {:#X}\n", self.limit).unwrap();
            write!(f, "line  = {}\n", self.line).unwrap();
            write!(f, "block = {}", self.block.as_ref().unwrap())
        }
    }
}
