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

use heap::immix;
use heap::gc;
use utils::Address;
use common::AddressMap;
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

#[derive(Clone)]
pub struct LineMarkTable {
    space_start: Address,
    ptr: *mut immix::LineMark,
    len: usize
}

#[derive(Clone)]
pub struct LineMarkTableSlice {
    ptr: *mut immix::LineMark,
    len: usize
}

impl LineMarkTable {
    pub fn new(space_start: Address, space_end: Address) -> LineMarkTable {
        let line_mark_table_len = (space_end - space_start) / immix::BYTES_IN_LINE;
        let line_mark_table = {
            let ret = unsafe {
                malloc_zero(mem::size_of::<immix::LineMark>() * line_mark_table_len)
            } as *mut immix::LineMark;
            let mut cursor = ret;

            for _ in 0..line_mark_table_len {
                unsafe {
                    *cursor = immix::LineMark::Free;
                }
                cursor = unsafe { cursor.offset(1) };
            }

            ret
        };

        LineMarkTable {
            space_start: space_start,
            ptr: line_mark_table,
            len: line_mark_table_len
        }
    }

    pub fn take_slice(&mut self, start: usize, len: usize) -> LineMarkTableSlice {
        LineMarkTableSlice {
            ptr: unsafe { self.ptr.offset(start as isize) },
            len: len
        }
    }

    #[inline(always)]
    #[allow(dead_code)]
    fn get(&self, index: usize) -> immix::LineMark {
        debug_assert!(index <= self.len);
        unsafe { *self.ptr.offset(index as isize) }
    }

    #[inline(always)]
    fn set(&self, index: usize, value: immix::LineMark) {
        debug_assert!(index <= self.len);
        unsafe { *self.ptr.offset(index as isize) = value };
    }

    pub fn index_to_address(&self, index: usize) -> Address {
        self.space_start + (index << immix::LOG_BYTES_IN_LINE)
    }

    #[inline(always)]
    pub fn mark_line_live(&self, addr: Address) {
        let line_table_index = (addr - self.space_start) >> immix::LOG_BYTES_IN_LINE;

        self.set(line_table_index, immix::LineMark::Live);

        if line_table_index < self.len - 1 {
            self.set(line_table_index + 1, immix::LineMark::ConservLive);
        }
    }

    #[inline(always)]
    pub fn mark_line_live2(&self, space_start: Address, addr: Address) {
        let line_table_index = (addr - space_start) >> immix::LOG_BYTES_IN_LINE;

        self.set(line_table_index, immix::LineMark::Live);

        if line_table_index < self.len - 1 {
            self.set(line_table_index + 1, immix::LineMark::ConservLive);
        }
    }
}

impl Drop for LineMarkTable {
    fn drop(&mut self) {
        unsafe { memsec::free(self.ptr) }
    }
}

impl LineMarkTableSlice {
    #[inline(always)]
    pub fn get(&self, index: usize) -> immix::LineMark {
        debug_assert!(index <= self.len);
        unsafe { *self.ptr.offset(index as isize) }
    }
    #[inline(always)]
    pub fn set(&mut self, index: usize, value: immix::LineMark) {
        debug_assert!(index <= self.len);
        unsafe { *self.ptr.offset(index as isize) = value };
    }
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }
}

#[repr(C)]
pub struct ImmixSpace {
    start: Address,
    end: Address,

    // these maps are writable at allocation, read-only at collection
    pub alloc_map: Arc<AddressMap<u8>>,

    // these maps are only for collection
    pub trace_map: Arc<AddressMap<u8>>,

    // this table will be accessed through unsafe raw pointers. since Rust doesn't provide a
    // data structure for such guarantees:
    // 1. Non-overlapping segments of this table may be accessed parallelly from different mutator
    //    threads
    // 2. One element may be written into at the same time by different gc threads during tracing
    pub line_mark_table: LineMarkTable,

    total_blocks: usize, // for debug use

    #[allow(dead_code)]
    mmap: memmap::MmapMut,
    usable_blocks: Mutex<LinkedList<Box<ImmixBlock>>>,
    used_blocks: Mutex<LinkedList<Box<ImmixBlock>>>
}

pub struct ImmixBlock {
    id: usize,
    state: immix::BlockMark,
    start: Address,

    // a segment of the big line mark table in ImmixSpace
    line_mark_table: LineMarkTableSlice
}

const SPACE_ALIGN: usize = 1 << 19;

impl ImmixSpace {
    pub fn new(space_size: usize) -> ImmixSpace {
        // acquire memory through mmap
        let mut anon_mmap: memmap::MmapMut =
            match memmap::MmapMut::map_anon(space_size + SPACE_ALIGN) {
                Ok(m) => m,
                Err(_) => panic!("failed to call mmap")
            };
        let start: Address = Address::from_ptr::<u8>(anon_mmap.as_mut_ptr()).align_up(SPACE_ALIGN);
        let end: Address = start + space_size;

        let line_mark_table = LineMarkTable::new(start, end);

        let trace_map = AddressMap::new(start, end);
        if cfg!(debug_assertions) {
            // access every of its cells
            trace_map.init_all(0);
        }
        let alloc_map = AddressMap::new(start, end);
        if cfg!(debug_assertions) {
            alloc_map.init_all(0);
        }

        let mut ret = ImmixSpace {
            start: start,
            end: end,
            mmap: anon_mmap,

            line_mark_table: line_mark_table,
            trace_map: Arc::new(trace_map),
            alloc_map: Arc::new(alloc_map),
            usable_blocks: Mutex::new(LinkedList::new()),
            used_blocks: Mutex::new(LinkedList::new()),
            total_blocks: 0
        };

        ret.init_blocks();

        ret
    }

    fn init_blocks(&mut self) -> () {
        let mut id = 0;
        let mut block_start = self.start;
        let mut line = 0;

        let mut usable_blocks_lock = self.usable_blocks.lock().unwrap();

        while block_start + immix::BYTES_IN_BLOCK <= self.end {
            usable_blocks_lock.push_back(Box::new(ImmixBlock {
                id: id,
                state: immix::BlockMark::Usable,
                start: block_start,
                line_mark_table: self.line_mark_table.take_slice(line, immix::LINES_IN_BLOCK)
            }));

            id += 1;
            block_start = block_start + immix::BYTES_IN_BLOCK;
            line += immix::LINES_IN_BLOCK;
        }

        self.total_blocks = id;
    }

    pub fn return_used_block(&self, old: Box<ImmixBlock>) {
        // Unsafe and raw pointers are used to transfer ImmixBlock to/from each Mutator.
        // This avoids explicit ownership transferring
        // If we explicitly transfer ownership, the function needs to own the Mutator in order to
        // move the ImmixBlock out of it (see ImmixMutatorLocal.alloc_from_global()),
        // and this will result in passing the Mutator object as value
        // (instead of a borrowed reference) all the way in the allocation
        self.used_blocks.lock().unwrap().push_front(old);
    }

    #[allow(unreachable_code)]
    pub fn get_next_usable_block(&self) -> Option<Box<ImmixBlock>> {
        let res_new_block: Option<Box<ImmixBlock>> =
            { self.usable_blocks.lock().unwrap().pop_front() };
        if res_new_block.is_none() {
            // should unlock, and call GC here
            gc::trigger_gc();

            None
        } else {
            res_new_block
        }
    }

    #[allow(unused_variables)]
    #[allow(unused_assignments)]
    pub fn sweep(&self) {
        let mut free_lines = 0;
        let mut usable_blocks = 0;
        let mut full_blocks = 0;

        let mut used_blocks_lock = self.used_blocks.lock().unwrap();

        let mut usable_blocks_lock = self.usable_blocks.lock().unwrap();
        usable_blocks = usable_blocks_lock.len();

        let mut live_blocks: LinkedList<Box<ImmixBlock>> = LinkedList::new();

        while !used_blocks_lock.is_empty() {
            let mut block = used_blocks_lock.pop_front().unwrap();

            let mut has_free_lines = false;

            {
                let mut cur_line_mark_table = block.line_mark_table_mut();
                for i in 0..cur_line_mark_table.len() {
                    if cur_line_mark_table.get(i) != immix::LineMark::Live &&
                        cur_line_mark_table.get(i) != immix::LineMark::ConservLive
                    {
                        has_free_lines = true;
                        cur_line_mark_table.set(i, immix::LineMark::Free);

                        free_lines += 1;
                    }
                }

                // release the mutable borrow of 'block'
            }

            if has_free_lines {
                block.set_state(immix::BlockMark::Usable);
                usable_blocks += 1;

                usable_blocks_lock.push_front(block);
            } else {
                block.set_state(immix::BlockMark::Full);
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
                self.total_blocks * immix::LINES_IN_BLOCK,
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

    pub fn start(&self) -> Address {
        self.start
    }
    pub fn end(&self) -> Address {
        self.end
    }

    pub fn line_mark_table(&self) -> &LineMarkTable {
        &self.line_mark_table
    }

    #[inline(always)]
    pub fn addr_in_space(&self, addr: Address) -> bool {
        addr >= self.start && addr < self.end
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
    fn alloc_map(&self) -> *mut u8 {
        self.alloc_map.ptr
    }

    #[inline(always)]
    fn trace_map(&self) -> *mut u8 {
        self.trace_map.ptr
    }
}

impl ImmixBlock {
    pub fn get_next_available_line(&self, cur_line: usize) -> Option<usize> {
        let mut i = cur_line;
        while i < self.line_mark_table.len {
            match self.line_mark_table.get(i) {
                immix::LineMark::Free => {
                    return Some(i);
                }
                _ => {
                    i += 1;
                }
            }
        }
        None
    }

    pub fn get_next_unavailable_line(&self, cur_line: usize) -> usize {
        let mut i = cur_line;
        while i < self.line_mark_table.len {
            match self.line_mark_table.get(i) {
                immix::LineMark::Free => {
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
        for i in 0..line_mark_table.len {
            if line_mark_table.get(i) == immix::LineMark::Free {
                let line_start: Address = self.start + (i << immix::LOG_BYTES_IN_LINE);

                // zero the line
                unsafe {
                    memsec::memzero(line_start.to_ptr_mut::<u8>(), immix::BYTES_IN_LINE);
                }
            }
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }
    pub fn start(&self) -> Address {
        self.start
    }
    pub fn set_state(&mut self, mark: immix::BlockMark) {
        self.state = mark;
    }
    #[inline(always)]
    pub fn line_mark_table(&self) -> &LineMarkTableSlice {
        &self.line_mark_table
    }
    #[inline(always)]
    pub fn line_mark_table_mut(&mut self) -> &mut LineMarkTableSlice {
        &mut self.line_mark_table
    }
}

/// Using raw pointers forbid the struct being shared between threads
/// we ensure the raw pointers won't be an issue, so we allow Sync/Send on ImmixBlock
unsafe impl Sync for ImmixBlock {}
unsafe impl Send for ImmixBlock {}
unsafe impl Sync for ImmixSpace {}
unsafe impl Send for ImmixSpace {}

impl fmt::Display for ImmixSpace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ImmixSpace\n").unwrap();
        write!(f, "range={:#X} ~ {:#X}\n", self.start, self.end).unwrap();

        // print table by vec
        //        write!(f, "table={{\n").unwrap();
        //        for i in 0..self.line_mark_table_len {
        //            write!(f, "({})", i).unwrap();
        //            write!(f, "{:?},", unsafe{*self.line_mark_table.offset(i as isize)}).unwrap();
        //            if i % immix::BYTES_IN_LINE == immix::BYTES_IN_LINE - 1 {
        //                write!(f, "\n").unwrap();
        //            }
        //        }
        //        write!(f, "\n}}\n").unwrap();


        write!(f, "t_ptr={:?}\n", self.line_mark_table.ptr).unwrap();
        //        write!(f, "usable blocks:\n").unwrap();
        //        for b in self.usable_blocks.iter() {
        //            write!(f, "  {}\n", b).unwrap();
        //        }
        //        write!(f, "used blocks:\n").unwrap();
        //        for b in self.used_blocks.iter() {
        //            write!(f, "  {}\n", b).unwrap();
        //        }
        write!(f, "done\n")
    }
}

impl fmt::Display for ImmixBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "ImmixBlock#{}(state={:?}, address=0x{:X})",
            self.id,
            self.state,
            self.start
        )
    }
}

impl fmt::Debug for ImmixBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "ImmixBlock#{}(state={:?}, address={:#X}, line_table={:?}",
            self.id,
            self.state,
            self.start,
            self.line_mark_table.ptr
        ).unwrap();

        write!(f, "[").unwrap();
        for i in 0..immix::LINES_IN_BLOCK {
            write!(f, "{:?},", self.line_mark_table.get(i)).unwrap();
        }
        write!(f, "]")
    }
}
