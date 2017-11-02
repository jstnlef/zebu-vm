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
use utils::mem::memmap;
use utils::mem::memsec::memzero;

use std::sync::Mutex;
use std::mem;

const LOG_BYTES_IN_PAGE: usize = 12;
const BYTES_IN_PAGE: ByteSize = 1 << LOG_BYTES_IN_PAGE; // 4KB

// 4M pages
const PAGES_IN_SPACE: usize = 1 << (LOG_BYTES_PREALLOC_SPACE - LOG_BYTES_IN_PAGE);

#[repr(C)]
pub struct FreelistSpace {
    // 32 bytes
    desc: SpaceDescriptor,
    start: Address,
    end: Address,
    size: ByteSize,

    // 24 bytes
    cur_end: Address,
    cur_size: ByteSize,
    cur_pages: usize,

    // 88 bytes (8 + 40 * 2)
    total_pages: usize,
    usable_nodes: Mutex<Vec<FreelistNode>>,
    used_nodes: Mutex<Vec<FreelistNode>>,

    // some statistics
    // 32 bytes
    pub last_gc_free_pages: usize,
    pub last_gc_used_pages: usize,

    // 16 bytes
    #[allow(dead_code)]
    mmap: memmap::MmapMut,

    padding: [u64; (BYTES_IN_PAGE - 32 - 24 - 88 - 32) >> 3],

    // page tables
    page_encode_table: [LargeObjectEncode; PAGES_IN_SPACE],
    page_mark_table: [PageMark; PAGES_IN_SPACE],

    mem: [u8; 0]
}

impl RawMemoryMetadata for FreelistSpace {
    #[inline(always)]
    fn addr(&self) -> Address {
        Address::from_ptr(self as *const FreelistSpace)
    }
    #[inline(always)]
    fn mem_start(&self) -> Address {
        self.start
    }
}

impl Space for FreelistSpace {
    #[inline(always)]
    fn start(&self) -> Address {
        self.start
    }

    #[inline(always)]
    fn end(&self) -> Address {
        self.cur_end
    }

    #[inline(always)]
    #[allow(unused_variables)]
    fn is_valid_object(&self, addr: Address) -> bool {
        true
    }

    fn destroy(&mut self) {}

    fn prepare_for_gc(&mut self) {
        // erase page mark
        unsafe {
            memzero(
                &mut self.page_mark_table[0] as *mut PageMark,
                self.cur_pages
            );
        }
    }

    fn sweep(&mut self) {
        debug!("=== {:?} Sweep ===", self.desc);
        debug_assert_eq!(self.n_used_pages() + self.n_usable_pages(), self.cur_pages);

        let mut free_pages = 0;
        let mut used_pages = 0;

        {
            let mut used_nodes = self.used_nodes.lock().unwrap();
            let mut usable_nodes = self.usable_nodes.lock().unwrap();

            let mut all_nodes: Vec<FreelistNode> = {
                let mut ret = vec![];
                ret.append(&mut used_nodes);
                ret.append(&mut usable_nodes);
                ret
            };
            debug_assert_eq!(all_nodes.len(), self.cur_pages);

            while !all_nodes.is_empty() {
                let node: FreelistNode = all_nodes.pop().unwrap();
                let index = self.get_page_index(node.addr);
                if self.page_mark_table[index] == PageMark::Live {
                    used_pages += node.size >> LOG_BYTES_IN_PAGE;
                    used_nodes.push(node);
                } else {
                    free_pages += node.size >> LOG_BYTES_IN_PAGE;
                    usable_nodes.push(node);
                }
            }
        }

        if cfg!(debug_assertions) {
            debug!("free pages = {} of {} total", free_pages, self.cur_pages);
            debug!("used pages = {} of {} total", used_pages, self.cur_pages);
        }

        self.last_gc_free_pages = free_pages;
        self.last_gc_used_pages = used_pages;

        if self.n_used_pages() == self.total_pages && self.total_pages != 0 {
            use std::process;
            println!("Out of memory in Freelist Space");
            process::exit(1);
        }

        debug_assert_eq!(self.n_used_pages() + self.n_usable_pages(), self.cur_pages);

        trace!("=======================");
    }

    #[inline(always)]
    fn mark_object_traced(&mut self, obj: ObjectReference) {
        let index = self.get_page_index(obj.to_address());
        self.page_mark_table[index] = PageMark::Live;
    }

    #[inline(always)]
    fn is_object_traced(&self, obj: ObjectReference) -> bool {
        let index = self.get_page_index(obj.to_address());
        self.page_mark_table[index] == PageMark::Live
    }
}

impl FreelistSpace {
    pub fn new(desc: SpaceDescriptor, space_size: ByteSize) -> Raw<FreelistSpace> {
        let mut anon_mmap = match memmap::MmapMut::map_anon(
            BYTES_PREALLOC_SPACE * 2 // for alignment
        ) {
            Ok(m) => m,
            Err(_) => panic!("failed to reserve address space for mmap")
        };
        let mmap_ptr = anon_mmap.as_mut_ptr();
        trace!("    mmap ptr: {:?}", mmap_ptr);

        let space_size = math::align_up(space_size, BYTES_IN_PAGE);

        let meta_start = Address::from_ptr::<u8>(mmap_ptr).align_up(SPACE_ALIGN);
        let mem_start = meta_start + BYTES_IN_PAGE +
            mem::size_of::<LargeObjectEncode>() * PAGES_IN_SPACE +
            mem::size_of::<PageMark>() * PAGES_IN_SPACE;
        let mem_end = mem_start + space_size;
        trace!("    space metadata: {}", meta_start);
        trace!("    space: {} ~ {}", mem_start, mem_end);

        let mut space: Raw<FreelistSpace> = unsafe { Raw::from_addr(meta_start) };
        trace!("    acquired Raw<FreelistSpace>");

        space.desc = desc;
        space.start = mem_start;
        space.end = mem_end;
        space.size = space_size;
        trace!("    initialized desc/start/end/size");

        space.cur_end = space.start;
        space.cur_size = 0;
        space.cur_pages = 0;
        trace!("    initialized cur_end/size/pages");

        space.total_pages = space_size >> LOG_BYTES_IN_PAGE;
        unsafe {
            use std::ptr;
            ptr::write(
                &mut space.usable_nodes as *mut Mutex<Vec<FreelistNode>>,
                Mutex::new(Vec::new())
            );
            ptr::write(
                &mut space.used_nodes as *mut Mutex<Vec<FreelistNode>>,
                Mutex::new(Vec::new())
            );
        }
        trace!("    initialized total/usable/used_nodes");

        unsafe {
            use std::ptr;
            ptr::write(&mut space.mmap as *mut memmap::MmapMut, anon_mmap);
        }
        trace!("    store mmap");

        debug_assert_eq!(Address::from_ptr(&space.mem as *const [u8; 0]), mem_start);

        space.trace_details();

        space
    }

    #[inline(always)]
    pub fn get_page_index(&self, obj: Address) -> usize {
        (obj - self.mem_start()) >> LOG_BYTES_IN_PAGE
    }

    pub fn alloc(&mut self, size: ByteSize, align: ByteSize) -> Address {
        assert!(align <= BYTES_IN_PAGE);
        let size = math::align_up(size, BYTES_IN_PAGE);

        // check if we can find any usable nodes that will fit the allocation
        let mut usable_nodes = self.usable_nodes.lock().unwrap();
        let mut candidate = None;
        for i in 0..usable_nodes.len() {
            let ref node = usable_nodes[i];
            if node.size >= size {
                candidate = Some(i);
                break;
            }
        }

        let opt_node = if let Some(index) = candidate {
            Some(usable_nodes.remove(index))
        } else {
            // we will need to allocate new memory
            let pages_required = size >> LOG_BYTES_IN_PAGE;
            if self.cur_pages + pages_required <= self.total_pages {
                // we have enough pages
                let start = self.cur_end;

                self.cur_end += size;
                self.cur_size += size;
                self.cur_pages += pages_required;

                Some(FreelistNode { size, addr: start })
            } else {
                None
            }
        };

        if let Some(node) = opt_node {
            let res = node.addr;
            self.used_nodes.lock().unwrap().push(node);

            // zero the pages
            unsafe {
                memzero(res.to_ptr_mut::<u8>(), size);
            }

            res
        } else {
            unsafe { Address::zero() }
        }
    }

    pub fn n_used_pages(&self) -> usize {
        let lock = self.used_nodes.lock().unwrap();
        let mut ret = 0;
        for node in lock.iter() {
            ret += node.size;
        }
        ret = ret >> LOG_BYTES_IN_PAGE;
        ret
    }

    pub fn n_usable_pages(&self) -> usize {
        let lock = self.usable_nodes.lock().unwrap();
        let mut ret = 0;
        for node in lock.iter() {
            ret += node.size;
        }
        ret = ret >> LOG_BYTES_IN_PAGE;
        ret
    }

    pub fn get_type_encode(&self, obj: ObjectReference) -> LargeObjectEncode {
        let index = self.get_page_index(obj.to_address());
        self.page_encode_table[index]
    }

    pub fn get_type_encode_slot(&self, addr: Address) -> Address {
        let index = self.get_page_index(addr);
        Address::from_ptr(&self.page_encode_table[index] as *const LargeObjectEncode)
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
            "-pages: current {} (usable: {}, used: {}), total {}",
            self.cur_pages,
            self.usable_nodes.lock().unwrap().len(),
            self.used_nodes.lock().unwrap().len(),
            self.total_pages
        );
        trace!(
            "-page type encode starts at {}",
            Address::from_ptr(&self.page_encode_table as *const LargeObjectEncode)
        );
        trace!(
            "-page mark table starts at {}",
            Address::from_ptr(&self.page_mark_table as *const PageMark)
        );
        trace!("-memory starts at {}", self.mem_start());
        trace!("=== {:?} ===", self.desc);
    }
}

#[repr(C)]
pub struct FreelistNode {
    size: ByteSize,
    addr: Address
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
#[allow(dead_code)] // we do not explicitly use Free, but we zero the page marks
pub enum PageMark {
    Free = 0,
    Live
}
