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

use common::AddressMap;
use common::ptr::*;
use heap::*;
use heap::immix::*;
use heap::gc;
use objectmodel::sidemap::*;
use utils::*;
use utils::mem::malloc_zero;
use utils::mem::memmap;
use utils::mem::memsec;

use std::*;
use std::collections::LinkedList;
use std::sync::Mutex;
use std::sync::Arc;

// this table will be accessed through unsafe raw pointers. since Rust doesn't provide a
// data structure for such guarantees:
// 1. Non-overlapping segments of this table may be accessed parallelly from different mutator
//    threads
// 2. One element may be written into at the same time by different gc threads during tracing
//#[repr(C, packed)]
///// A global large line mark table. It facilitates iterating through all the line marks.
//pub struct GlobalLineMarkTable {
//    space_start: Address,
//    ptr: *mut LineMark,
//    len: usize,
//    mmap: memmap::Mmap
//}

#[repr(C, packed)]
#[derive(Copy, Clone)]
/// Every Immix block owns its own segment of the line mark table
pub struct BlockLineMarkTable {
    ptr: *mut LineMark
}

//impl GlobalLineMarkTable {
//    pub fn new(space_start: Address, space_end: Address) -> LineMarkTable {
//        let line_mark_table_len = (space_end - space_start) / BYTES_IN_LINE;
//
//        // mmap memory for the table
//        // we do not initialize it here
//        // we initialize it when a block needs to take a part of it
//        let mmap = match memmap::Mmap::anonymous(
//            mem::size_of::<LineMark>() * line_mark_table_len,
//            memmap::Protection::ReadWrite
//        ) {
//            Ok(m) => m,
//            Err(_) => panic!("failed to mmap for immix line mark table")
//        };
//
//        let ptr: *mut LineMark = mmap.ptr() as *mut LineMark;
//
//        LineMarkTable {
//            space_start,
//            ptr,
//            len: line_mark_table_len,
//            mmap
//        }
//    }
//
//    pub fn take_slice(&mut self, start: usize, len: usize) -> BlockLineMarkTable {
//        BlockLineMarkTable {
//            ptr: unsafe { self.ptr.offset(start as isize) }
//        }
//    }
//
//    #[inline(always)]
//    #[allow(dead_code)]
//    fn get(&self, index: usize) -> LineMark {
//        debug_assert!(index <= self.len);
//        unsafe { *self.ptr.offset(index as isize) }
//    }
//
//    #[inline(always)]
//    fn set(&self, index: usize, value: LineMark) {
//        debug_assert!(index <= self.len);
//        unsafe { *self.ptr.offset(index as isize) = value };
//    }
//
//    pub fn index_to_address(&self, index: usize) -> Address {
//        self.space_start + (index << LOG_BYTES_IN_LINE)
//    }
//
//    #[inline(always)]
//    pub fn mark_line_live(&self, addr: Address) {
//        self.mark_line_live2(self.space_start, addr)
//    }
//
//    #[inline(always)]
//    pub fn mark_line_live2(&self, space_start: Address, addr: Address) {
//        let line_table_index = (addr - space_start) >> LOG_BYTES_IN_LINE;
//
//        self.set(line_table_index, LineMark::Live);
//
//        if line_table_index < self.len - 1 {
//            self.set(line_table_index + 1, LineMark::ConservLive);
//        }
//    }
//}

impl BlockLineMarkTable {
    #[inline(always)]
    pub fn get(&self, index: u8) -> LineMark {
        unsafe { *self.ptr.offset(index as isize) }
    }
    #[inline(always)]
    pub fn set(&mut self, index: u8, value: LineMark) {
        unsafe { *self.ptr.offset(index as isize) = value };
    }
    #[inline(always)]
    pub fn len(&self) -> u8 {
        LINES_IN_BLOCK as u8
    }
}

const SPACE_ALIGN: usize = 1 << 19;

/// An Immix space represents a piece of raw memory as Immix heap
///
/// The memory layout looks like this
///
/// |-----------------------------------| <- aligned to 512K (1 << 19)
/// | ImmixSpace metadata (this struct) |
/// | ... (fields)                      |
/// | line_mark_table                   |
/// |-----------------------------------| <- aligned to 64K (1 << 16)
/// | Immix Block metadata              |
/// |- - - - - - - - - - - - - - - - - -|
/// | block memory                      |
/// | ...                               |
/// | ...                               |
/// |-----------------------------------|
/// | Immix Block metadata              |
/// |- - - - - - - - - - - - - - - - - -|
/// | block memory                      |
/// | ...                               |
/// | ...                               |
/// |-----------------------------------|
///   ......
///
#[repr(C)]
pub struct ImmixSpace {
    desc: SpaceDescriptor,
    start: Address,
    end: Address,

    // lists for managing blocks in current space
    total_blocks: usize, // for debug use
    usable_blocks: Mutex<LinkedList<Raw<ImmixBlock>>>,
    used_blocks: Mutex<LinkedList<Raw<ImmixBlock>>>,

    #[allow(dead_code)]
    mmap: memmap::Mmap,

    // this table will be accessed through unsafe raw pointers. since Rust doesn't provide a
    // data structure for such guarantees:
    // 1. Non-overlapping segments of this table may be accessed parallelly from different mutator
    //    threads
    // 2. One element may be written into at the same time by different gc threads during tracing
    line_mark_table_len: usize,
    // do not directly access this field
    line_mark_table: [LineMark; 0]
}

impl RawMemoryMetadata for ImmixSpace {
    #[inline(always)]
    fn addr(&self) -> Address {
        Address::from_ptr(self as *const ImmixSpace)
    }
    #[inline(always)]
    fn mem_start(&self) -> Address {
        self.end - (self.total_blocks << LOG_BYTES_IN_BLOCK)
    }
}

#[repr(C, packed)]
pub struct ImmixBlock {
    // a segment of the big line mark table in ImmixSpace
    line_mark_table: BlockLineMarkTable,
    // state of current block
    state: BlockMark,
    // unused bytes in the header
    unused: [u8; 7],
    // gc map
    gc_map: [u8; BYTES_MEM_IN_BLOCK >> 3],
    // type map
    type_map: [u8; BYTES_MEM_IN_BLOCK >> 3]
}

impl RawMemoryMetadata for ImmixBlock {
    #[inline(always)]
    fn addr(&self) -> Address {
        Address::from_ptr(self as *const ImmixBlock)
    }
    #[inline(always)]
    fn mem_start(&self) -> Address {
        self.addr() + mem::size_of::<Self>()
    }
}

impl ImmixSpace {
    pub fn new(desc: SpaceDescriptor, space_size: ByteSize) -> Raw<ImmixSpace> {
        // acquire memory through mmap
        let anon_mmap: memmap::Mmap = match memmap::Mmap::anonymous(
            space_size + SPACE_ALIGN,
            memmap::Protection::ReadWrite
        ) {
            Ok(m) => m,
            Err(_) => panic!("failed to call mmap")
        };

        let meta_start: Address = Address::from_ptr::<u8>(anon_mmap.ptr()).align_up(SPACE_ALIGN);
        let end: Address = meta_start + space_size;

        // calculate how large the line mark table is needed
        // this memory chunk is used for
        // * ImmixSpace meta: constant size,
        // * line mark table: LINES_PER_BLOCK * sizeof(LineMark) (256 bytes per block)
        // * immix blocks (64kb per block, 256 lines)
        let n_blocks = (space_size - mem::size_of::<ImmixSpace>()) /
            (LINES_IN_BLOCK * mem::size_of::<LineMark>() + BYTES_IN_BLOCK);

        let mem_start = end - n_blocks * BYTES_IN_BLOCK;
        assert!(mem_start.is_aligned_to(IMMIX_BLOCK_ALIGN));

        // initialize space metadata
        let mut space: Raw<ImmixSpace> = unsafe { Raw::from_addr(meta_start) };

        space.desc = desc;
        space.start = mem_start;
        space.end = end;
        space.total_blocks = n_blocks;
        space.usable_blocks = Mutex::new(LinkedList::new());
        space.used_blocks = Mutex::new(LinkedList::new());
        space.mmap = anon_mmap;
        space.line_mark_table_len = n_blocks * LINES_IN_BLOCK;

        space.init_blocks();

        space
    }

    fn init_blocks(&mut self) {
        let mut block_start = self.start();
        let mut line = 0;

        let mut usable_blocks_lock = self.usable_blocks.lock().unwrap();

        while block_start + BYTES_IN_BLOCK <= self.end {
            let mut block: Raw<ImmixBlock> = unsafe { Raw::from_addr(block_start) };
            block.line_mark_table = self.get_block_line_mark_table(line);
            block.state = BlockMark::Uninitialized;

            usable_blocks_lock.push_back(block);

            block_start = block_start + BYTES_IN_BLOCK;
            line += LINES_IN_BLOCK;
        }
    }

    fn get_block_line_mark_table(&self, start_line: usize) -> BlockLineMarkTable {
        BlockLineMarkTable {
            ptr: self.get_line_mark_table_slot(start_line).to_ptr_mut()
        }
    }

    fn get_line_mark_table_slot(&self, index: usize) -> Address {
        Address::from_ptr(&self.line_mark_table as *const LineMark)
            .shift::<LineMark>(index as isize)
    }

    #[inline(always)]
    pub fn set_line_mark(&self, index: usize, mark: LineMark) {
        unsafe { self.get_line_mark_table_slot(index).store(mark) }
    }

    #[inline(always)]
    pub fn get_line_mark(&self, index: usize) -> LineMark {
        unsafe { self.get_line_mark_table_slot(index).load::<LineMark>() }
    }

    #[inline(always)]
    pub fn mark_line_alive(addr: Address) {
        let space: Raw<ImmixSpace> = unsafe { Raw::from_addr(addr.mask(SPACE_LOWBITS_MASK)) };
        let index = (addr - space.mem_start()) >> LOG_BYTES_IN_LINE;
        space.set_line_mark(index, LineMark::Live);
        if index < space.line_mark_table_len - 1 {
            space.set_line_mark(index + 1, LineMark::ConservLive);
        }
    }

    pub fn return_used_block(&self, old: Raw<ImmixBlock>) {
        self.used_blocks.lock().unwrap().push_front(old);
    }

    #[allow(unreachable_code)]
    pub fn get_next_usable_block(&self) -> Option<Raw<ImmixBlock>> {
        let new_block = self.usable_blocks.lock().unwrap().pop_front();
        match new_block {
            Some(block) => {
                if block.state == BlockMark::Uninitialized {
                    // we need to zero the block - zero anything other than metadata
                    let zero_start = block.addr() + BLOCK_META;
                    let zero_size = BYTES_IN_BLOCK - BLOCK_META;
                    unsafe {
                        memsec::memzero(zero_start.to_ptr_mut::<u8>(), zero_size);
                    }
                }
                Some(block)
            }
            None => {
                gc::trigger_gc();
                None
            }
        }
    }

    #[allow(unused_variables)]
    #[allow(unused_assignments)]
    pub fn sweep(&self) {
        // some statistics
        let mut free_lines = 0;
        let mut usable_blocks = 0;
        let mut full_blocks = 0;

        let mut used_blocks_lock = self.used_blocks.lock().unwrap();
        let mut usable_blocks_lock = self.usable_blocks.lock().unwrap();
        usable_blocks = usable_blocks_lock.len();

        let mut live_blocks: LinkedList<Raw<ImmixBlock>> = LinkedList::new();

        while !used_blocks_lock.is_empty() {
            let mut block = used_blocks_lock.pop_front().unwrap();

            let mut has_free_lines = false;
            // find free lines in the block, and set their line mark as free
            // (not zeroing the memory yet)
            {
                let mut cur_line_mark_table = block.line_mark_table_mut();
                for i in 0..cur_line_mark_table.len() {
                    if cur_line_mark_table.get(i) != LineMark::Live &&
                        cur_line_mark_table.get(i) != LineMark::ConservLive
                    {
                        has_free_lines = true;
                        cur_line_mark_table.set(i, LineMark::Free);
                        free_lines += 1;
                    }
                }
                // release the mutable borrow of 'block'
            }

            if has_free_lines {
                block.state = BlockMark::Usable;
                usable_blocks += 1;
                usable_blocks_lock.push_front(block);
            } else {
                block.state = BlockMark::Full;
                full_blocks += 1;
                live_blocks.push_front(block);
            }
        }

        used_blocks_lock.append(&mut live_blocks);

        if cfg!(debug_assertions) {
            debug!("---immix space---");
            debug!(
                "free lines    = {} of {} total ({} blocks)",
                free_lines,
                self.total_blocks * LINES_IN_BLOCK,
                self.total_blocks
            );
            debug!("usable blocks = {}", usable_blocks);
            debug!("full blocks   = {}", full_blocks);
        }

        if full_blocks == self.total_blocks {
            println!("Out of memory in Immix Space");
            process::exit(1);
        }

        debug_assert!(full_blocks + usable_blocks == self.total_blocks);
    }
}

use heap::Space;
impl Space for ImmixSpace {
    #[inline(always)]
    fn start(&self) -> Address {
        self.start
    }
    #[inline(always)]
    fn end(&self) -> Address {
        self.end
    }
    #[inline(always)]
    fn is_valid_object(&self, addr: Address) -> bool {
        // we cannot judge if it is a valid object, we always return true
        true
    }
}

impl ImmixBlock {
    pub fn get_next_available_line(&self, cur_line: u8) -> Option<u8> {
        let mut i = cur_line;
        while i < self.line_mark_table.len() {
            match self.line_mark_table.get(i) {
                LineMark::Free => {
                    return Some(i);
                }
                _ => {
                    i += 1;
                }
            }
        }
        None
    }

    pub fn get_next_unavailable_line(&self, cur_line: u8) -> u8 {
        let mut i = cur_line;
        while i < self.line_mark_table.len() {
            match self.line_mark_table.get(i) {
                LineMark::Free => {
                    i += 1;
                }
                _ => {
                    return i;
                }
            }
        }
        i
    }

    pub fn lazy_zeroing(&mut self) {
        let line_mark_table = self.line_mark_table();
        for i in 0..line_mark_table.len() {
            if line_mark_table.get(i) == LineMark::Free {
                let line_start: Address = self.mem_start() + ((i as usize) << LOG_BYTES_IN_LINE);
                // zero the line
                unsafe {
                    memsec::memzero(line_start.to_ptr_mut::<u8>(), BYTES_IN_LINE);
                }
            }
        }
    }
    #[inline(always)]
    pub fn line_mark_table(&self) -> &BlockLineMarkTable {
        &self.line_mark_table
    }
    #[inline(always)]
    pub fn line_mark_table_mut(&mut self) -> &mut BlockLineMarkTable {
        &mut self.line_mark_table
    }

    #[inline(always)]
    pub fn get_type_map_slot(&self, addr: Address) -> Address {
        let index = (addr - self.mem_start()) >> 3;
        Address::from_ptr(&self.type_map[index] as *const u8)
    }

    #[inline(always)]
    pub fn get_type_map_slot_static(addr: Address) -> Address {
        let block_start = addr.mask(IMMIX_BLOCK_LOWBITS_MASK);
        let block: Raw<ImmixBlock> = unsafe { Raw::from_addr(block_start) };
        block.get_type_map_slot(addr)
    }

    #[inline(always)]
    pub fn get_gc_map_slot(&self, addr: Address) -> Address {
        let index = (addr - self.mem_start()) >> 3;
        Address::from_ptr(&self.type_map[index] as *const u8)
    }

    #[inline(always)]
    pub fn get_gc_map_slot_static(addr: Address) -> Address {
        let block_start = addr.mask(IMMIX_BLOCK_LOWBITS_MASK);
        let block: Raw<ImmixBlock> = unsafe { Raw::from_addr(block_start) };
        block.get_gc_map_slot(addr)
    }
}

#[inline(always)]
pub fn mark_object_traced(obj: ObjectReference) {
    let obj_addr = obj.to_address();

    // mark object
    let addr = ImmixBlock::get_gc_map_slot_static(obj_addr);
    unsafe { addr.store(1u8) }

    // mark line
    ImmixSpace::mark_line_alive(obj_addr);
}

#[inline(always)]
pub fn is_object_traced(obj: ObjectReference) -> bool {
    // gc byte
    let gc_byte = unsafe { ImmixBlock::get_gc_map_slot_static(obj.to_address()).load::<u8>() };
    gc_byte == 1
}

/// Using raw pointers forbid the struct being shared between threads
/// we ensure the raw pointers won't be an issue, so we allow Sync/Send on ImmixBlock
unsafe impl Sync for ImmixBlock {}
unsafe impl Send for ImmixBlock {}
unsafe impl Sync for ImmixSpace {}
unsafe impl Send for ImmixSpace {}

impl fmt::Display for ImmixSpace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "ImmixSpace").unwrap();
        writeln!(f, "  range=0x{:#X} ~ 0x{:#X}", self.start, self.end).unwrap();
        writeln!(
            f,
            "  line_mark_table=0x{:?}",
            &self.line_mark_table as *const LineMark
        )
    }
}

impl fmt::Display for ImmixBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ImmixBlock({}, state={:?}", self.addr(), self.state)
    }
}

impl fmt::Debug for ImmixBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "ImmixBlock({}, state={:?}, line_table={:?}",
            self.addr(),
            self.state,
            self.line_mark_table.ptr
        ).unwrap();

        write!(f, "[").unwrap();
        for i in 0..self.line_mark_table.len() {
            write!(f, "{:?},", self.line_mark_table.get(i)).unwrap();
        }
        write!(f, "]")
    }
}
