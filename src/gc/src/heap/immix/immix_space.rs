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
use heap::immix::*;
use heap::gc;
use utils::*;
use utils::mem::memmap;
use utils::mem::memsec;

use std::*;
use std::collections::LinkedList;
use std::sync::Mutex;

const TRACE_ALLOC: bool = true;

/// An ImmixSpace represents a memory area that is used for immix heap and also its meta data
///
/// Memory layout
/// |------------------| <- 16GB align
/// | metadata         |
/// | ...              | (64 KB)
/// |------------------|
/// | block mark table | (256 KB) - 256K blocks, 1 byte per block
/// |------------------|
/// | line mark table  | (64MB) - 64M lines, 1 byte per line
/// |------------------|
/// | gc byte table    | (1GB) - 1/16 of memory, 1 byte per 16 (min alignment/object size)
/// |------------------|
/// | type byte table  | (1GB) - 1/16 of memory, 1 byte per 16 (min alignment/object size)
/// |------------------|
/// | memory starts    |
/// | ......           |
/// | ......           |
/// |__________________|
#[repr(C)]
pub struct ImmixSpace {
    // 32 bytes - max space (as user defined)
    desc: SpaceDescriptor,
    start: Address,
    end: Address,
    size: ByteSize,

    // 32 bytes - current space (where we grow to at this point)
    // FIXME: should always hold 'usable_blocks' lock in order to change these fields along with
    // adding new usable blocks
    cur_end: Address,
    cur_size: ByteSize,
    cur_blocks: usize,
    // how many blocks we grow by last time
    cur_growth_rate: usize,

    // lists for managing blocks in current space
    // 88 bytes (8 + 40 * 2)
    total_blocks: usize, // for debug use
    usable_blocks: Mutex<LinkedList<Raw<ImmixBlock>>>,
    used_blocks: Mutex<LinkedList<Raw<ImmixBlock>>>,

    // some statistics
    // 32 bytes
    pub last_gc_free_lines: usize,
    pub last_gc_used_lines: usize,

    // 16 bytes
    #[allow(dead_code)]
    mmap: memmap::MmapMut,

    // padding to space metadata takes 64KB
    padding: [u64; ((BYTES_IN_BLOCK - 32 - 32 - 88 - 32 - 16) >> 3)],

    // block mark table
    block_mark_table: [BlockMark; BLOCKS_IN_SPACE],

    // line mark table
    line_mark_table: [LineMark; LINES_IN_SPACE],

    // gc byte table
    gc_byte_table: [u8; WORDS_IN_SPACE >> 1],
    // type byte table
    type_byte_table: [u8; WORDS_IN_SPACE >> 1],

    // memory starts here
    mem: [u8; 0]
}

impl RawMemoryMetadata for ImmixSpace {
    #[inline(always)]
    fn addr(&self) -> Address {
        Address::from_ptr(self as *const ImmixSpace)
    }
    #[inline(always)]
    fn mem_start(&self) -> Address {
        self.start
    }
}

#[repr(C, packed)]
pub struct ImmixBlock {}

impl RawMemoryMetadata for ImmixBlock {
    #[inline(always)]
    fn addr(&self) -> Address {
        Address::from_ptr(self as *const ImmixBlock)
    }
    #[inline(always)]
    fn mem_start(&self) -> Address {
        self.addr()
    }
}

impl ImmixSpace {
    pub fn new(desc: SpaceDescriptor, space_size: ByteSize) -> Raw<ImmixSpace> {
        // acquire memory through mmap
        let mut anon_mmap: memmap::MmapMut = match memmap::MmapMut::map_anon(
            BYTES_PREALLOC_SPACE * 2 // for alignment
        ) {
            Ok(m) => m,
            Err(_) => panic!("failed to reserve addresss pace for mmap")
        };
        let mmap_ptr = anon_mmap.as_mut_ptr();
        trace!("    mmap ptr: {:?}", mmap_ptr);

        let meta_start: Address = Address::from_ptr::<u8>(mmap_ptr).align_up(SPACE_ALIGN);
        let mem_start: Address = meta_start + OFFSET_MEM_START;
        let mem_end: Address = mem_start + space_size;
        trace!("    space metadata: {}", meta_start);
        trace!("    space: {} ~ {}", mem_start, mem_end);

        // initialize space metadata
        let mut space: Raw<ImmixSpace> = unsafe { Raw::from_addr(meta_start) };
        trace!("    acquired Raw<ImmixSpace>");

        space.desc = desc;
        space.start = mem_start;
        space.end = mem_end;
        space.size = space_size;
        trace!("    initialized desc/start/end/size");

        space.cur_end = space.start;
        space.cur_size = 0;
        space.cur_blocks = 0;
        trace!("    initialized cur_end/size/blocks");

        space.total_blocks = space_size >> LOG_BYTES_IN_BLOCK;
        unsafe {
            // use ptr::write to avoid destruction of the old values
            use std::ptr;
            ptr::write(
                &mut space.usable_blocks as *mut Mutex<LinkedList<Raw<ImmixBlock>>>,
                Mutex::new(LinkedList::new())
            );
            ptr::write(
                &mut space.used_blocks as *mut Mutex<LinkedList<Raw<ImmixBlock>>>,
                Mutex::new(LinkedList::new())
            );
        }
        trace!("    initialized total/usable/used blocks");

        unsafe {
            use std::ptr;
            ptr::write(&mut space.mmap as *mut memmap::MmapMut, anon_mmap);
        }
        trace!("    store mmap");

        space.last_gc_used_lines = 0;
        space.last_gc_free_lines = 0;

        trace!("    initializing blocks...");
        space.init_blocks();

        space.trace_details();

        space
    }

    fn init_blocks(&mut self) {
        const N_INITIAL_BLOCKS: usize = 64;
        let n_blocks = if N_INITIAL_BLOCKS < self.total_blocks {
            N_INITIAL_BLOCKS
        } else {
            self.total_blocks
        };
        self.grow_blocks(n_blocks);
    }

    fn grow_blocks(&mut self, n_blocks: usize) {
        trace!("      grow space by {} blocks", n_blocks);
        debug_assert!(self.cur_blocks + n_blocks <= self.total_blocks);
        let mut lock = self.usable_blocks.lock().unwrap();

        // start address
        let mut cur_addr = self.cur_end;
        // start line/block index
        let line_start = (cur_addr - self.mem_start()) >> LOG_BYTES_IN_LINE;
        let block_start = self.cur_blocks;

        for _ in 0..n_blocks {
            let block: Raw<ImmixBlock> = unsafe { Raw::from_addr(cur_addr) };
            // add to usable blocks
            lock.push_back(block);

            cur_addr += BYTES_IN_BLOCK;
        }

        // zeroing block mark table (set blocks as Uninitialized)
        let block_table_ptr: *mut BlockMark =
            &mut self.block_mark_table[block_start] as *mut BlockMark;
        unsafe {
            memsec::memzero(block_table_ptr, n_blocks);
        }

        // zeroing line mark table (set lines as Free)
        let line_table_ptr: *mut LineMark = &mut self.line_mark_table[line_start] as *mut LineMark;
        unsafe {
            memsec::memzero(line_table_ptr, n_blocks * LINES_IN_BLOCK);
        }

        self.cur_end = cur_addr;
        self.cur_size += n_blocks * BYTES_IN_BLOCK;
        self.cur_blocks += n_blocks;
        self.cur_growth_rate = n_blocks;
    }

    pub fn cleanup(&self) {}

    #[inline(always)]
    pub fn get(addr: Address) -> Raw<ImmixSpace> {
        unsafe { Raw::from_addr(addr.mask(SPACE_LOWBITS_MASK)) }
    }

    // line mark table

    #[inline(always)]
    pub fn set_line_mark(&mut self, index: usize, mark: LineMark) {
        self.line_mark_table[index] = mark;
    }

    #[inline(always)]
    pub fn get_line_mark(&self, index: usize) -> LineMark {
        self.line_mark_table[index]
    }

    #[inline(always)]
    pub fn get_line_mark_index(&self, addr: Address) -> usize {
        (addr - self.mem_start()) >> LOG_BYTES_IN_LINE
    }

    #[inline(always)]
    pub fn mark_line_alive(addr: Address) {
        let mut space: Raw<ImmixSpace> = unsafe { Raw::from_addr(addr.mask(SPACE_LOWBITS_MASK)) };
        let index = space.get_line_mark_index(addr);
        space.set_line_mark(index, LineMark::Live);
        if index < (space.cur_blocks << LOG_LINES_IN_BLOCK) - 1 {
            space.set_line_mark(index + 1, LineMark::ConservLive);
        }
    }

    // block mark table

    #[inline(always)]
    pub fn set_block_mark(&mut self, index: usize, mark: BlockMark) {
        self.block_mark_table[index] = mark;
    }

    #[inline(always)]
    pub fn get_block_mark(&self, index: usize) -> BlockMark {
        self.block_mark_table[index]
    }

    #[inline(always)]
    pub fn get_block_mark_index(&self, addr: Address) -> usize {
        (addr - self.mem_start()) >> LOG_BYTES_IN_BLOCK
    }

    // gc/type byte table

    #[inline(always)]
    pub fn get_word_index(&self, addr: Address) -> usize {
        (addr - self.mem_start()) >> LOG_POINTER_SIZE
    }

    #[inline(always)]
    pub fn get_gc_byte_slot(&self, index: usize) -> Address {
        Address::from_ptr(&self.gc_byte_table[index] as *const u8)
    }

    #[inline(always)]
    pub fn get_gc_byte_slot_static(addr: Address) -> Address {
        let space = ImmixSpace::get(addr);
        let index = space.get_word_index(addr);
        space.get_gc_byte_slot(index)
    }

    #[inline(always)]
    pub fn get_type_byte_slot(&self, index: usize) -> Address {
        Address::from_ptr(&self.type_byte_table[index] as *const u8)
    }

    #[inline(always)]
    pub fn get_type_byte_slot_static(addr: Address) -> Address {
        let space = ImmixSpace::get(addr);
        let index = space.get_word_index(addr);
        space.get_type_byte_slot(index)
    }

    pub fn return_used_block(&self, old: Raw<ImmixBlock>) {
        self.used_blocks.lock().unwrap().push_front(old);
    }

    #[allow(unreachable_code)]
    pub fn get_next_usable_block(&mut self) -> Option<Raw<ImmixBlock>> {
        if TRACE_ALLOC {
            debug!(
                "{} blocks usable, {} blocks used",
                self.n_usable_blocks(),
                self.n_used_blocks()
            );
        }
        let new_block = self.usable_blocks.lock().unwrap().pop_front();
        match new_block {
            Some(block) => {
                let block_index = self.get_block_mark_index(block.mem_start());
                if self.block_mark_table[block_index] == BlockMark::Uninitialized {
                    self.block_mark_table[block_index] = BlockMark::Usable;
                    // we lazily initialize the block in allocator
                }
                Some(block)
            }
            None => {
                // check if we can grow more
                if self.cur_blocks < self.total_blocks {
                    let next_growth = self.cur_growth_rate << 1;
                    let n_blocks = if self.cur_blocks + next_growth < self.total_blocks {
                        next_growth
                    } else {
                        self.total_blocks - self.cur_blocks
                    };
                    self.grow_blocks(n_blocks);
                    self.get_next_usable_block()
                } else {
                    gc::trigger_gc();
                    None
                }
            }
        }
    }

    pub fn prepare_for_gc(&mut self) {
        // erase lines marks
        let lines = self.cur_blocks << LOG_LINES_IN_BLOCK;
        unsafe {
            memsec::memzero(&mut self.line_mark_table[0] as *mut LineMark, lines);
        }

        // erase gc bytes
        let words = self.cur_size >> LOG_POINTER_SIZE;
        unsafe {
            memsec::memzero(&mut self.gc_byte_table[0] as *mut u8, words);
        }
    }

    #[allow(unused_variables)]
    #[allow(unused_assignments)]
    pub fn sweep(&mut self) {
        debug!("=== {:?} Sweep ===", self.desc);
        debug_assert_eq!(
            self.n_used_blocks() + self.n_usable_blocks(),
            self.cur_blocks
        );

        // some statistics
        let mut free_lines = 0;
        let mut used_lines = 0;

        {
            let mut used_blocks_lock = self.used_blocks.lock().unwrap();
            let mut usable_blocks_lock = self.usable_blocks.lock().unwrap();

            let mut all_blocks: LinkedList<Raw<ImmixBlock>> = {
                let mut ret = LinkedList::new();
                ret.append(&mut used_blocks_lock);
                ret.append(&mut usable_blocks_lock);
                ret
            };
            debug_assert_eq!(all_blocks.len(), self.cur_blocks);

            while !all_blocks.is_empty() {
                let block = all_blocks.pop_front().unwrap();
                let line_index = self.get_line_mark_index(block.mem_start());
                let block_index = self.get_block_mark_index(block.mem_start());

                let mut has_free_lines = false;
                // find free lines in the block, and set their line mark as free
                // (not zeroing the memory yet)

                for i in line_index..(line_index + LINES_IN_BLOCK) {
                    if self.line_mark_table[i] != LineMark::Live &&
                        self.line_mark_table[i] != LineMark::ConservLive
                    {
                        has_free_lines = true;
                        self.line_mark_table[i] = LineMark::Free;
                        free_lines += 1;
                    } else {
                        used_lines += 1;
                    }
                }

                if has_free_lines {
                    trace!("Block {} is usable", block.addr());
                    self.block_mark_table[block_index] = BlockMark::Usable;
                    usable_blocks_lock.push_front(block);
                } else {
                    trace!("Block {} is full", block.addr());
                    self.block_mark_table[block_index] = BlockMark::Full;
                    used_blocks_lock.push_front(block);
                }
            }
        }

        if cfg!(debug_assertions) {
            debug!(
                "free lines    = {} of {} total ({} blocks)",
                free_lines,
                self.cur_blocks * LINES_IN_BLOCK,
                self.cur_blocks
            );
            debug!(
                "used lines    = {} of {} total ({} blocks)",
                used_lines,
                self.cur_blocks * LINES_IN_BLOCK,
                self.cur_blocks
            );
            debug!("usable blocks = {}", self.n_usable_blocks());
            debug!("full blocks   = {}", self.n_used_blocks());
        }

        self.last_gc_free_lines = free_lines;
        self.last_gc_used_lines = used_lines;

        if self.n_used_blocks() == self.total_blocks && self.total_blocks != 0 {
            println!("Out of memory in Immix Space");
            process::exit(1);
        }

        debug_assert_eq!(
            self.n_used_blocks() + self.n_usable_blocks(),
            self.cur_blocks
        );

        trace!("=======================");
    }

    fn trace_details(&self) {
        trace!("=== {:?} ===", self.desc);
        trace!(
            "-range: {} ~ {} (size: {})",
            self.start,
            self.end,
            self.size
        );
        trace!(
            "-cur  : {} ~ {} (size: {})",
            self.start,
            self.cur_end,
            self.cur_size
        );
        trace!(
            "-block: current {} (usable: {}, used: {}), total {}",
            self.cur_blocks,
            self.usable_blocks.lock().unwrap().len(),
            self.used_blocks.lock().unwrap().len(),
            self.total_blocks
        );
        trace!(
            "-block mark table starts at {}",
            Address::from_ptr(&self.block_mark_table as *const BlockMark)
        );
        trace!(
            "-line mark table starts at {}",
            Address::from_ptr(&self.line_mark_table as *const LineMark)
        );
        trace!(
            "-gc byte table starts at {}",
            Address::from_ptr(&self.gc_byte_table as *const u8)
        );
        trace!(
            "-type byte table starts at {}",
            Address::from_ptr(&self.type_byte_table as *const u8)
        );
        trace!("-memory starts at {}", self.mem_start());
        trace!("=== {:?} ===", self.desc);
    }

    // for debug use
    pub fn n_used_blocks(&self) -> usize {
        self.used_blocks.lock().unwrap().len()
    }

    pub fn n_usable_blocks(&self) -> usize {
        self.usable_blocks.lock().unwrap().len()
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
        self.cur_end
    }
    #[inline(always)]
    fn is_valid_object(&self, addr: Address) -> bool {
        // we cannot judge if it is a valid object, we always return true
        true
    }
}

impl ImmixBlock {
    pub fn get_next_available_line(&self, cur_line: usize) -> Option<usize> {
        let space: Raw<ImmixSpace> = ImmixSpace::get(self.mem_start());
        let base_line = space.get_line_mark_index(self.mem_start());
        let base_end = base_line + LINES_IN_BLOCK;

        let mut i = base_line + cur_line;
        while i < base_end {
            match space.get_line_mark(i) {
                LineMark::Free => {
                    return Some(i - base_line);
                }
                _ => {
                    i += 1;
                }
            }
        }
        None
    }

    pub fn get_next_unavailable_line(&self, cur_line: usize) -> usize {
        let space: Raw<ImmixSpace> = ImmixSpace::get(self.mem_start());
        let base_line = space.get_line_mark_index(self.mem_start());
        let base_end = base_line + LINES_IN_BLOCK;

        let mut i = base_line + cur_line;
        while i < base_end {
            match space.get_line_mark(i) {
                LineMark::Free => {
                    i += 1;
                }
                _ => {
                    return i - base_line;
                }
            }
        }
        i - base_line
    }

    #[inline(always)]
    pub fn set_line_mark(&self, line: usize, mark: LineMark) {
        let mut space: Raw<ImmixSpace> = ImmixSpace::get(self.mem_start());
        let base_line = space.get_line_mark_index(self.mem_start());

        space.set_line_mark(base_line + line as usize, mark);
    }
}

#[inline(always)]
pub fn mark_object_traced(obj: ObjectReference) {
    let obj_addr = obj.to_address();

    // mark object
    let addr = ImmixSpace::get_gc_byte_slot_static(obj_addr);
    unsafe { addr.store(1u8) }

    // mark line
    ImmixSpace::mark_line_alive(obj_addr);
}

#[inline(always)]
pub fn is_object_traced(obj: ObjectReference) -> bool {
    // gc byte
    let gc_byte = unsafe { ImmixSpace::get_gc_byte_slot_static(obj.to_address()).load::<u8>() };
    gc_byte == 1
}

/// Using raw pointers forbid the struct being shared between threads
/// we ensure the raw pointers won't be an issue, so we allow Sync/Send on ImmixBlock
unsafe impl Sync for ImmixBlock {}
unsafe impl Send for ImmixBlock {}
unsafe impl Sync for ImmixSpace {}
unsafe impl Send for ImmixSpace {}
