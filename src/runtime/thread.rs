extern crate memmap;

use gc::Mutator;
use ast::ir::*;
use utils::ByteSize;

pub const STACK_SIZE : ByteSize = (4 << 20); // 4mb

#[cfg(target_arch = "x86_64")]
pub const PAGE_SIZE  : ByteSize = (4 << 10); // 4kb

pub struct MuStack {
    hdr: MuEntityHeader,
    
    func_id: MuID,
    
    size: ByteSize,
    
    //    lo addr                                                    hi addr
    //     | overflow guard page | actual stack ..................... | underflow guard page|
    //     |                     |                                    |                     |
    // overflowGuard           lowerBound                           upperBound
    //                                                              underflowGuard    
    overflow_guard : Address,
    lower_bound    : Address,
    upper_bound    : Address,
    underflow_guard: Address,
    
    exception_obj  : Option<Address>,
    
    #[allow(dead_code)]
    mmap           : memmap::Mmap
}

impl MuStack {
    pub fn new(id: MuID, func_id: MuID) -> MuStack {
        let total_size = PAGE_SIZE * 2 + STACK_SIZE;
        
        let anon_mmap = match memmap::Mmap::anonymous(total_size, memmap::Protection::ReadWrite) {
            Ok(m) => m,
            Err(_) => panic!("failed to mmap for a stack"),
        };
        
        let overflow_guard = Address::from_ptr(anon_mmap.ptr());
        
        unimplemented!()
    }
}

pub struct MuThread {
    hdr: MuEntityHeader,
    allocator: Box<Mutator>,
    
    user_tls: Address
}