#![allow(dead_code)]

use utils::Address;
use utils::mem::memmap;
use std::ptr;
use std::sync::Arc;
use common::AddressMap;

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

    treadmill: TreadMill
}

impl FreeListSpace {
    pub fn new(space_size: usize) -> FreeListSpace {
        let anon_mmap : memmap::Mmap = match memmap::Mmap::anonymous(space_size + SPACE_ALIGN, memmap::Protection::ReadWrite) {
            Ok(m) => m,
            Err(_) => panic!("failed to call mmap")
        };
        let start : Address = Address::from_ptr::<u8>(anon_mmap.ptr()).align_up(SPACE_ALIGN);
        let end   : Address = start.plus(space_size);

        unimplemented!()
    }
}

struct TreadMill{
    free: *mut TreadMillNode,
    scan: *mut TreadMillNode,
    t   : *mut TreadMillNode,
    b   : *mut TreadMillNode
}

impl TreadMill {
    fn new(start: Address, end: Address) -> TreadMill {
        let mut addr = start;

        let free = TreadMillNode::singleton(addr);
        let mut tail = free;

        while addr < end {
            tail = TreadMillNode::insert_after(tail, addr);
        }

        unimplemented!()
    }
}


struct TreadMillNode {
    payload: Address,

    prev: *mut TreadMillNode,
    next: *mut TreadMillNode
}

impl TreadMillNode {
    fn singleton(addr: Address) -> *mut TreadMillNode {
        let mut ptr = Box::into_raw(Box::new(TreadMillNode {
            payload: addr,
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
    fn insert_after(node: *mut TreadMillNode, addr: Address) -> *mut TreadMillNode {
        unsafe {
            // node <- ptr -> node.next
            let mut ptr = Box::into_raw(Box::new(TreadMillNode {
                payload: addr,
                // inserted between node and node.next
                prev: node,
                next: (*node).next
            }));


            // ptr <- node.next
            (*(*node).next).prev = ptr;
            // node -> ptr
            (*node).next = ptr;

            ptr
        }
    }
}