#![allow(dead_code)]

extern crate doubly;

use utils::Address;
use utils::mem::memmap;
use utils::LOG_POINTER_SIZE;
use common::AddressMap;

use objectmodel;

use std::sync::Arc;
use std::fmt;
use std::sync::Mutex;

use self::doubly::DoublyLinkedList;

const SPACE_ALIGN : usize = 1 << 19;
const BLOCK_SIZE  : usize = 1 << 12;    // 4kb

const TRACE_TREADMILL : bool = false;

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

        let size = size + objectmodel::OBJECT_HEADER_SIZE;

        let blocks_needed = if size % BLOCK_SIZE == 0 {
            size / BLOCK_SIZE
        } else {
            size / BLOCK_SIZE + 1
        };

        if TRACE_TREADMILL {
            trace!("before allocation, space: {}", self);
        }

        trace!("requiring {} bytes ({} blocks)", size, blocks_needed);
        let res = {
            if blocks_needed > 1 {
                unimplemented!()
            }

            let mut treadmill = self.treadmill.lock().unwrap();
            treadmill.alloc_blocks(blocks_needed)
        };

        if TRACE_TREADMILL {
            trace!("after allocation, space: {}", self);
        }

        if res.is_zero() {
            res
        } else {
            res.offset(-objectmodel::OBJECT_HEADER_OFFSET)
        }
    }

    #[inline(always)]
    #[cfg(feature = "use-sidemap")]
    fn is_traced(&self, addr: Address, mark_state: u8) -> bool {
        objectmodel::is_traced(self.trace_map(), self.start, unsafe { addr.to_object_reference() }, mark_state)
    }

    #[inline(always)]
    #[cfg(not(feature = "use-sidemap"))]
    fn is_traced(&self, addr: Address, mark_state: u8) -> bool {
        objectmodel::is_traced(unsafe{addr.to_object_reference()}, mark_state)
    }

    pub fn sweep(&self) {
        trace!("going to sweep treadmill space");
        if TRACE_TREADMILL {
            trace!("{}", self);
        }

        let mut nodes_scanned = 0;
        let mut free_nodes_scanned = 0;
        let mut alive_nodes_scanned = 0;

        let mut treadmill = self.treadmill.lock().unwrap();

        {
            let trace_map = self.trace_map();
            let mark_state = objectmodel::load_mark_state();

            let from = treadmill.from;
            let to   = treadmill.to;

            let total_nodes = treadmill.spaces[from].len();
            let mut i = 0;
            while nodes_scanned < total_nodes {
                trace!("scanning {}", treadmill.spaces[from][i]);
                let addr = treadmill.spaces[from][i].payload;

                nodes_scanned += 1;

                let traced = self.is_traced(addr, mark_state);

                if traced {
                    // this object is alive
                    alive_nodes_scanned += 1;

                    // move to tospace
                    let node = treadmill.spaces[from].remove(i);
                    treadmill.spaces[to].push_front(node);

                    trace!("is alive");

                    // do not increment i
                } else {
                    free_nodes_scanned += 1;
                    i += 1;
                }
            }

            // check if we have any free nodes
            if free_nodes_scanned == 0 && treadmill.spaces[treadmill.to].len() == 0 {
                println!("didnt free up any memory in treadmill space");
                panic!("we ran out of memory in large object space")
            }
        }

        // next allocation in to_space will starts from alive_nodes_scanned
        treadmill.from_space_next = alive_nodes_scanned;

        // flip
        if treadmill.from == 0 {
            treadmill.from = 1;
            treadmill.to   = 0;
        } else {
            treadmill.from = 0;
            treadmill.to   = 1;
        }

        if cfg!(debug_assertions) {
            debug!("---tread mill space---");
            debug!("total nodes scanned: {}", nodes_scanned);
            debug!("alive nodes scanned: {}", alive_nodes_scanned);
            debug!("free  nodes scanned: {}", free_nodes_scanned);
        }
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
    from_space_next : usize, // next available node in from_space
    from: usize,
    to  : usize,
    spaces      : [DoublyLinkedList<TreadmillNode>; 2]
}

impl Treadmill {
    fn new(start: Address, end: Address) -> Treadmill {
        let half_space = start.plus(end.diff(start) / 2);

        let mut from_space = DoublyLinkedList::new();
        let mut to_space   = DoublyLinkedList::new();

        let mut addr = start;

        while addr < half_space {
            from_space.push_back(TreadmillNode::new(addr));
            addr = addr.plus(BLOCK_SIZE);
        }

        while addr < end {
            to_space.push_back(TreadmillNode::new(addr));
            addr = addr.plus(BLOCK_SIZE);
        }

        Treadmill {
            from_space_next: 0,
            from: 0,
            to  : 1,
            spaces: [from_space, to_space]
        }
    }

    fn alloc_blocks(&mut self, n_blocks: usize) -> Address {
        let ref from_space = self.spaces[self.from];
        if self.from_space_next + n_blocks <= from_space.len() {
            // zero blocks
            for i in 0..n_blocks {
                let block_i = self.from_space_next + i;
                let block_start = from_space[block_i].payload;

                Treadmill::zeroing_block(block_start);
            }

            // return first block
            // FIXME: the blocks may not be contiguous!!! we cannot allocate multiple blocks
            let ret = from_space[self.from_space_next].payload;
            self.from_space_next += n_blocks;

            ret
        } else {
            unsafe {Address::zero()}
        }
    }

    fn zeroing_block(start: Address) {
        use utils::mem::memsec;

        unsafe {
            memsec::memzero(start.to_ptr_mut::<u8>(), BLOCK_SIZE);
        }
    }
}

impl fmt::Display for Treadmill {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "next: {}\n", self.from_space_next).unwrap();
        write!(f, "from:").unwrap();
        for i in 0..self.spaces[self.from].len() {
            write!(f, "{}->", self.spaces[self.from][i]).unwrap();
        }
        write!(f, "\n").unwrap();
        write!(f, "to:").unwrap();
        for i in 0..self.spaces[self.to].len() {
            write!(f, "{}->", self.spaces[self.to][i]).unwrap();
        }
        write!(f, "\n")
    }
}

struct TreadmillNode {
    payload: Address
}

impl TreadmillNode {
    fn new(addr: Address) -> TreadmillNode {
        TreadmillNode {
            payload: addr
        }
    }
}


impl fmt::Display for TreadmillNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}]", self.payload)
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
        let space = FreeListSpace::new(BLOCK_SIZE * 10);

        for i in 0..10 {
            let ret = space.alloc(BLOCK_SIZE / 2, 8);
            println!("Allocation{}: {}", i, ret);
        }
    }

    #[test]
    #[ignore]
    fn test_treadmill_alloc_spanblock() {
        let space = FreeListSpace::new(BLOCK_SIZE * 10);

        for i in 0..5 {
            let ret = space.alloc(BLOCK_SIZE * 2, 8);
            println!("Allocation{}: {}", i, ret);
        }
    }

    #[test]
    fn test_treadmill_alloc_exhaust() {
        let space = FreeListSpace::new(BLOCK_SIZE * 10);

        for i in 0..20 {
            let ret = space.alloc(BLOCK_SIZE / 2, 8);
            println!("Allocation{}: {}", i, ret);
        }
    }
}