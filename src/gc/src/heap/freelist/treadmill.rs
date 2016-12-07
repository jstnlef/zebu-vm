#![allow(dead_code)]

use utils::Address;
use utils::mem::memmap;
use utils::LOG_POINTER_SIZE;
use common::AddressMap;

use objectmodel;

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

        trace!("before allocation, space: {}", self);

        trace!("requiring {} bytes ({} blocks)", size, blocks_needed);
        let res = {
            let mut treadmill = self.treadmill.lock().unwrap();
            treadmill.alloc_blocks(blocks_needed)
        };

        trace!("after allocation, space: {}", self);

        res
    }

    pub fn init_object(&self, addr: Address, encode: u8) {
        unsafe {
            *self.alloc_map().offset((addr.diff(self.start) >> LOG_POINTER_SIZE) as isize) = encode;
            objectmodel::mark_as_untraced(self.trace_map(), self.start, addr, objectmodel::load_mark_state());
        }
    }

    pub fn sweep(&self) {
        trace!("going to sweep treadmill space");
        trace!("{}", self);

        let mut treadmill = self.treadmill.lock().unwrap();
        let trace_map = self.trace_map();
        let mark_state = objectmodel::load_mark_state();

        let mut resnapped_any = false;

        loop {
            trace!("scanning {}", unsafe{&*treadmill.scan});
            let addr = unsafe{&*treadmill.scan}.payload;

            if objectmodel::is_traced(trace_map, self.start, unsafe { addr.to_object_reference() }, mark_state) {
                // the object is alive, do not need to 'move' its node

                // but they will be alive, we will set them to opposite mark color
                // (meaning they are not available after flip)
                unsafe{&mut *treadmill.scan}.color = objectmodel::flip(mark_state);

                trace!("is alive, set color to {}", objectmodel::flip(mark_state));

                // advance cur backwards
                treadmill.scan = unsafe{&*treadmill.scan}.prev();
            } else {
                // this object is dead
                // we do not need to set their color

                // we resnap it after current 'free' pointer
                if treadmill.scan != treadmill.free {
                    // since we are going to move current node (scan), we get its prev first
                    let prev = unsafe{&*treadmill.scan}.prev();
                    trace!("get scan's prev before resnapping it: {}", unsafe{&*prev});

                    let alive_node = unsafe { &mut *treadmill.scan }.remove();

                    trace!("is dead, take it out of treadmill");
                    trace!("treadmill: {}", &treadmill as &Treadmill);

                    // insert alive node after free
                    unsafe{&mut *treadmill.free}.insert_after(alive_node);
                    trace!("insert after free");
                    trace!("treadmill: {}", &treadmill as &Treadmill);

                    // if this is the first object inserted, it is the 'bottom'
                    // then 1) all resnapped objects will be between 'free' and 'bottom'
                    //      2) the traversal can stop when scan meets bottom
                    if !resnapped_any {
                        treadmill.b = treadmill.scan;
                        resnapped_any = true;
                    }

                    treadmill.scan = prev;
                } else {
                    trace!("is dead and it is free pointer, do not move it");
                    treadmill.scan = unsafe{&*treadmill.scan}.prev();
                }
            }

            // check if we can stop
            if resnapped_any && treadmill.scan == treadmill.b {
                return;
            }
            if !resnapped_any && treadmill.scan == treadmill.free {
                // we never set bottom (meaning everything is alive)

                println!("didnt free up any memory in treadmill space");
                panic!("we ran out of memory in large object space")
            }
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
    free: *mut TreadmillNode,
    scan: *mut TreadmillNode,
    b   : *mut TreadmillNode
}

impl Treadmill {
    fn new(start: Address, end: Address) -> Treadmill {
        let mut addr = start;

        let free = TreadmillNode::singleton(addr);
        addr = addr.plus(BLOCK_SIZE);

        let mut tail = free;

        while addr < end {
            tail = unsafe {(&mut *tail)}.init_insert_after(addr);
            addr = addr.plus(BLOCK_SIZE);
        }

        Treadmill {
            free: free,
            scan: free,
            b: free
        }
    }

    fn alloc_blocks(&mut self, n_blocks: usize) -> Address {
        let unavailable_color = objectmodel::load_mark_state();

        // check if we have n_blocks available
        let mut cur = self.free;
        for _ in 0..n_blocks {
            if unsafe{&*cur}.color == unavailable_color {
                trace!("next block color is {}, no available blocks, return zero", unavailable_color);
                return unsafe {Address::zero()};
            }

            cur = unsafe {&*cur}.next;
        }

        // we make sure that n_blocks are available, mark them as black
        let mut cur2 = self.free;
        for _ in 0..n_blocks {
            unsafe{&mut *cur2}.color = unavailable_color;
            cur2 = unsafe {&*cur2}.next
        }

        debug_assert!(cur == cur2);

        let ret = self.free;
        self.free = cur;

        trace!("set free to {}", unsafe {&*cur});

        unsafe{&*ret}.payload
    }
}

impl fmt::Display for Treadmill {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut cursor = self.free;

        write!(f, "\n").unwrap();
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

            if unsafe{&*cursor}.next() == self.free {
                break;
            } else {
                write!(f, "\n->").unwrap();
                cursor = unsafe{&*cursor}.next();
            }
        }

        Ok(())
    }
}

struct TreadmillNode {
    payload: Address,
    color: u8,

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
            // starts as 0 (1, i.e. mark_state, means allocated/alive)
            color: objectmodel::flip(objectmodel::load_mark_state()),
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
    fn init_insert_after(&mut self, addr: Address) -> *mut TreadmillNode {
        unsafe {
            // node <- ptr -> node.next
            let mut ptr = Box::into_raw(Box::new(TreadmillNode {
                payload: addr,
                color: objectmodel::flip(objectmodel::load_mark_state()),
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

    fn insert_after(&mut self, node: *mut TreadmillNode) {
        unsafe {
            // self <- node -> self.next
            (&mut *node).next = self.next;
            (&mut *node).prev = self as *mut TreadmillNode;

            // self.next -> node
            self.next = node;
            // node <- node.next.prev
            (&mut *(&mut *node).next).prev = node;
        }
    }

    /// remove current node from treadmill, and returns the node
    fn remove(&mut self) -> *mut TreadmillNode {
        if self.next == self as *mut TreadmillNode && self.prev == self as *mut TreadmillNode {
            // if this is the only node, return itself
            self as *mut TreadmillNode
        } else {
            // we need to take it out from the list
            unsafe {
                use std::ptr;

                // its prev node's next will be its next node
                (&mut *self.prev).next = self.next as *mut TreadmillNode;

                // its next node' prev will be its prev node
                (&mut *self.next).prev = self.prev as *mut TreadmillNode;

                // clear current node prev and next
                self.prev = ptr::null_mut();
                self.next = ptr::null_mut();
            }

            // then return it
            self as *mut TreadmillNode
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