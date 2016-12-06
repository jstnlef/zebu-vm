#![allow(dead_code)]

use utils::Address;
use utils::mem::memmap;
use common::AddressMap;

use std::ptr;
use std::sync::Arc;
use std::fmt;
use std::sync::Mutex;

const SPACE_ALIGN : usize = 1 << 19;
const BLOCK_SIZE  : usize = 1 << 12;    // 4kb

#[repr(C)]
pub struct FreeListSpace {
    start : Address,
    end   : Address,

    pub alloc_map : Arc<AddressMap<u8>>,
    pub trace_map : Arc<AddressMap<u8>>,

    #[allow(dead_code)]
    mmap : memmap::Mmap,

    treadmill: Mutex<Treadmill>
}

impl FreeListSpace {
    pub fn new(space_size: usize) -> FreeListSpace {
        let anon_mmap : memmap::Mmap = match memmap::Mmap::anonymous(space_size + SPACE_ALIGN, memmap::Protection::ReadWrite) {
            Ok(m) => m,
            Err(_) => panic!("failed to call mmap")
        };
        let start : Address = Address::from_ptr::<u8>(anon_mmap.ptr()).align_up(SPACE_ALIGN);
        let end   : Address = start.plus(space_size);

        let trace_map = AddressMap::new(start, end);
        let alloc_map = AddressMap::new(start, end);
        if cfg!(debug_assertions) {
            trace_map.init_all(0);
            alloc_map.init_all(0);
        }

        let treadmill = Treadmill::new(start, end);

        FreeListSpace {
            start: start,
            end: end,
            alloc_map: Arc::new(alloc_map),
            trace_map: Arc::new(trace_map),
            mmap: anon_mmap,
            treadmill: Mutex::new(treadmill)
        }
    }

    pub fn alloc(&self, size: usize, align: usize) -> Address {
        // every block is 'BLOCK_SIZE' aligned, usually we do not need to align
        assert!(BLOCK_SIZE % align == 0);

        let blocks_needed = if size % BLOCK_SIZE == 0 {
            size / BLOCK_SIZE
        } else {
            size / BLOCK_SIZE + 1
        };

        trace!("requiring {} bytes ({} blocks)", size, blocks_needed);
        let mut treadmill = self.treadmill.lock().unwrap();
        let res = treadmill.alloc_blocks(blocks_needed);

        res
    }

    pub fn sweep(&self) {
        let mut treadmill = self.treadmill.lock().unwrap();

        unimplemented!()
    }
}

use heap::Space;

impl Space for FreeListSpace {
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

unsafe impl Sync for FreeListSpace {}
unsafe impl Send for FreeListSpace {}

impl fmt::Display for FreeListSpace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FreeListSpace\n").unwrap();
        write!(f, "range={:#X} ~ {:#X}\n", self.start, self.end).unwrap();

        let treadmill : &Treadmill = &self.treadmill.lock().unwrap();
        write!(f, "treadmill: {}", treadmill)
    }
}

struct Treadmill{
    available_color: TreadmillNodeColor,

    free: *mut TreadmillNode,
    scan: *mut TreadmillNode,
    t   : *mut TreadmillNode,
    b   : *mut TreadmillNode
}

impl Treadmill {
    fn new(start: Address, end: Address) -> Treadmill {
        let mut addr = start;

        let free = TreadmillNode::singleton(addr);
        addr = addr.plus(BLOCK_SIZE);

        let mut tail = free;

        while addr < end {
            tail = unsafe {(&mut *tail)}.insert_after(addr);
            addr = addr.plus(BLOCK_SIZE);
        }

        Treadmill {
            available_color: TreadmillNodeColor::Ecru,
            free: free,
            scan: free,
            t: free,
            b: free
        }
    }

    fn alloc_blocks(&mut self, n_blocks: usize) -> Address {
        // check if we have n_blocks available
        let mut cur = self.free;
        for _ in 0..n_blocks {
            if unsafe{&*cur}.color != self.available_color {
                return unsafe {Address::zero()};
            }

            cur = unsafe {&*cur}.next;
        }

        // we make sure that n_blocks are available, mark them as black
        let mut cur2 = self.free;
        for _ in 0..n_blocks {
            unsafe{&mut *cur2}.color = TreadmillNodeColor::Black;
            cur2 = unsafe {&*cur2}.next
        }

        let ret = self.free;
        self.free = cur;

        unsafe{&*ret}.payload
    }
}

impl fmt::Display for Treadmill {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut cursor = self.free;

        loop {
            write!(f, "{}", unsafe{&*cursor}).unwrap();

            if cursor == self.free {
                write!(f, "(free)").unwrap();
            }
            if cursor == self.scan {
                write!(f, "(scan)").unwrap();
            }
            if cursor == self.b {
                write!(f, "(bottom)").unwrap();
            }
            if cursor == self.t {
                write!(f, "(top)").unwrap();
            }

            if unsafe{&*cursor}.next() == self.free {
                break;
            } else {
                write!(f, "->").unwrap();
                cursor = unsafe{&*cursor}.next();
            }
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TreadmillNodeColor {
    Ecru,
    White,
    Black,
    Grey
}

struct TreadmillNode {
    payload: Address,
    color: TreadmillNodeColor,

    prev: *mut TreadmillNode,
    next: *mut TreadmillNode
}

impl fmt::Display for TreadmillNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}-{:?}]", self.payload, self.color)
    }
}

impl TreadmillNode {
    fn singleton(addr: Address) -> *mut TreadmillNode {
        let mut ptr = Box::into_raw(Box::new(TreadmillNode {
            payload: addr,
            color: TreadmillNodeColor::Ecru,
            prev: ptr::null_mut(),
            next: ptr::null_mut(),
        }));

        // first node in the cyclic doubly linked list
        unsafe {
            (*ptr).prev = ptr;
            (*ptr).next = ptr;
        }

        ptr
    }

    /// returns the inserted node
    fn insert_after(&mut self, addr: Address) -> *mut TreadmillNode {
        unsafe {
            // node <- ptr -> node.next
            let mut ptr = Box::into_raw(Box::new(TreadmillNode {
                payload: addr,
                color: TreadmillNodeColor::Ecru,
                // inserted between node and node.next
                prev: self as *mut TreadmillNode,
                next: self.next
            }));


            // ptr <- node.next
            unsafe{(&mut *self.next)}.prev = ptr;
            // node -> ptr
            self.next = ptr;

            ptr
        }
    }

    fn next(&self) -> *mut TreadmillNode {
        self.next
    }

    fn prev(&self) -> *mut TreadmillNode {
        self.prev
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::BLOCK_SIZE;

    #[test]
    fn test_new_treadmill_space() {
        let space = FreeListSpace::new(BLOCK_SIZE * 10);

        println!("{}", space);
    }

    #[test]
    fn test_treadmill_alloc() {
        let mut space = FreeListSpace::new(BLOCK_SIZE * 10);

        for i in 0..10 {
            let ret = space.alloc(BLOCK_SIZE, 8);
            println!("Allocation{}: {}", i, ret);
        }
    }

    #[test]
    fn test_treadmill_alloc_spanblock() {
        let mut space = FreeListSpace::new(BLOCK_SIZE * 10);

        for i in 0..5 {
            let ret = space.alloc(BLOCK_SIZE * 2, 8);
            println!("Allocation{}: {}", i, ret);
        }
    }

    #[test]
    fn test_treadmill_alloc_exhaust() {
        let mut space = FreeListSpace::new(BLOCK_SIZE * 10);

        for i in 0..20 {
            let ret = space.alloc(BLOCK_SIZE, 8);
            println!("Allocation{}: {}", i, ret);
        }
    }
}