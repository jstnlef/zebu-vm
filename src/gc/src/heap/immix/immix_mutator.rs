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
use heap::immix::ImmixSpace;
use heap::immix::immix_space::ImmixBlock;
use heap::gc;
use objectmodel;
use utils::Address;

use std::*;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::{AtomicBool, Ordering};

const MAX_MUTATORS : usize = 1024;
lazy_static! {
    pub static ref MUTATORS : RwLock<Vec<Option<Arc<ImmixMutatorGlobal>>>> = {
        let mut ret = Vec::with_capacity(MAX_MUTATORS);
        for _ in 0..MAX_MUTATORS {
            ret.push(None);
        }
        RwLock::new(ret)
    };
    
    pub static ref N_MUTATORS : RwLock<usize> = RwLock::new(0);
}

const TRACE_ALLOC_FASTPATH : bool = true;

#[repr(C)]
pub struct ImmixMutatorLocal {
    id        : usize,
    
    // use raw pointer here instead of AddressMapTable
    // to avoid indirection in fast path    
    alloc_map : *mut u8,
    trace_map : *mut u8,
    space_start: Address,
    
    // cursor might be invalid, but Option<Address> is expensive here
    // after every GC, we set both cursor and limit
    // to Address::zero() so that alloc will branch to slow path    
    cursor    : Address,
    limit     : Address,
    line      : usize,
    
    // globally accessible per-thread fields
    pub global    : Arc<ImmixMutatorGlobal>,
    
    space     : Arc<ImmixSpace>,
    block     : Option<Box<ImmixBlock>>,

    mark_state: u8
}

lazy_static! {
    pub static ref CURSOR_OFFSET : usize = offset_of!(ImmixMutatorLocal=>cursor).get_byte_offset();
    pub static ref LIMIT_OFFSET  : usize = offset_of!(ImmixMutatorLocal=>limit).get_byte_offset();
}

pub struct ImmixMutatorGlobal {
    take_yield : AtomicBool,
    still_blocked : AtomicBool
}

impl ImmixMutatorLocal {
    pub fn reset(&mut self) -> () {
        unsafe {
            // should not use Address::zero() other than initialization
            self.cursor = Address::zero();
            self.limit = Address::zero();
        }
        self.line = immix::LINES_IN_BLOCK;
        
        self.block = None;
    }

    pub fn reset_after_gc(&mut self) {
        self.reset();
        self.mark_state ^= 1;
    }
    
    pub fn new(space : Arc<ImmixSpace>) -> ImmixMutatorLocal {
        let global = Arc::new(ImmixMutatorGlobal::new());
        
        let mut id_lock = N_MUTATORS.write().unwrap();
        {
            let mut mutators_lock = MUTATORS.write().unwrap();
            mutators_lock.remove(*id_lock);
            mutators_lock.insert(*id_lock, Some(global.clone()));
        }
        
        let ret = ImmixMutatorLocal {
            id : *id_lock,
            cursor: unsafe {Address::zero()}, limit: unsafe {Address::zero()}, line: immix::LINES_IN_BLOCK,
            block: None,
            alloc_map: space.alloc_map.ptr,
            trace_map: space.trace_map.ptr,
            space_start: space.start(),
            global: global,
            space: space,
            mark_state: objectmodel::INIT_MARK_STATE as u8
        };
        *id_lock += 1;
        
        ret
    }
    
    pub fn destroy(&mut self) {
        {
            self.return_block();
        }
        
        let mut mutator_count_lock = N_MUTATORS.write().unwrap();
        
        let mut mutators_lock = MUTATORS.write().unwrap();
        mutators_lock.push(None);
        mutators_lock.swap_remove(self.id);
        
        *mutator_count_lock = *mutator_count_lock - 1;
        
        if cfg!(debug_assertions) {
            debug!("destroy mutator. Now live mutators = {}", *mutator_count_lock);
        }
    }
    
    #[inline(always)]
    pub fn yieldpoint(&mut self) {
        if self.global.take_yield() {
            self.yieldpoint_slow();
        }
    }
    
    #[inline(never)]
    pub fn yieldpoint_slow(&mut self) {
        trace!("Mutator{}: yieldpoint triggered, slow path", self.id);
        gc::sync_barrier(self);
    }
    
    #[inline(always)]
    pub fn alloc(&mut self, size: usize, align: usize) -> Address {
        // this part of code will slow down allocation
        let align = objectmodel::check_alignment(align);
        let size = size + objectmodel::OBJECT_HEADER_SIZE;
        // end

        if TRACE_ALLOC_FASTPATH {
            trace!("Mutator{}: fastpath alloc: size={}, align={}", self.id, size, align);
        }

        let start = self.cursor.align_up(align);
        let end = start.plus(size);

        if TRACE_ALLOC_FASTPATH {
            trace!("Mutator{}: fastpath alloc: start=0x{:x}, end=0x{:x}", self.id, start, end);
        }

        if end > self.limit {
            let ret = self.try_alloc_from_local(size, align);
            if TRACE_ALLOC_FASTPATH {
                trace!("Mutator{}: fastpath alloc: try_alloc_from_local()=0x{:x}", self.id, ret);
            }
            
            if cfg!(debug_assertions) {
                if !ret.is_aligned_to(align) {
                    use std::process;
                    println!("wrong alignment on 0x{:x}, expected align: {}", ret, align);
                    process::exit(102);
                }
            }

            // this offset should be removed as well (for performance)
            ret.offset(-objectmodel::OBJECT_HEADER_OFFSET)
        } else {
            if cfg!(debug_assertions) {
                if !start.is_aligned_to(align) {
                    use std::process;
                    println!("wrong alignment on 0x{:x}, expected align: {}", start, align);
                    process::exit(102);
                }
            }
            self.cursor = end;
            
            start.offset(-objectmodel::OBJECT_HEADER_OFFSET)
        } 
    }
    
    #[inline(always)]
    #[cfg(feature = "use-sidemap")]
    pub fn init_object(&mut self, addr: Address, encode: u64) {
//        unsafe {
//            *self.alloc_map.offset((addr.diff(self.space_start) >> LOG_POINTER_SIZE) as isize) = encode as u8;
//            objectmodel::mark_as_untraced(self.trace_map, self.space_start, addr, self.mark_state);
//        }

        unimplemented!()
    }
    #[inline(always)]
    #[cfg(not(feature = "use-sidemap"))]
    pub fn init_object(&mut self, addr: Address, encode: u64) {
        unsafe {
            addr.offset(objectmodel::OBJECT_HEADER_OFFSET).store(encode);
        }
    }

    #[inline(always)]
    #[cfg(feature = "use-sidemap")]
    pub fn init_hybrid(&mut self, addr: Address, encode: u64, len: u64) {
        unimplemented!()
    }
    #[inline(always)]
    #[cfg(not(feature = "use-sidemap"))]
    pub fn init_hybrid(&mut self, addr: Address, encode: u64, len: u64) {
        let encode = encode | ((len << objectmodel::SHR_HYBRID_LENGTH) & objectmodel::MASK_HYBRID_LENGTH);
        unsafe {
            addr.offset(objectmodel::OBJECT_HEADER_OFFSET).store(encode);
        }
    }
    
    #[inline(never)]
    pub fn try_alloc_from_local(&mut self, size : usize, align: usize) -> Address {
        if self.line < immix::LINES_IN_BLOCK {
            let opt_next_available_line = {
                let cur_line = self.line;
                self.block().get_next_available_line(cur_line)
            };
    
            match opt_next_available_line {
                Some(next_available_line) => {
                    // we can alloc from local blocks
                    let end_line = self.block().get_next_unavailable_line(next_available_line);

                    self.cursor = self.block().start().plus(next_available_line << immix::LOG_BYTES_IN_LINE);
                    self.limit  = self.block().start().plus(end_line << immix::LOG_BYTES_IN_LINE);
                    self.line   = end_line;
                    
                    self.cursor.memset(0, self.limit.diff(self.cursor));
                    
                    for line in next_available_line..end_line {
                        self.block().line_mark_table_mut().set(line, immix::LineMark::FreshAlloc);
                    }

                    // allocate fast path
                    let start = self.cursor.align_up(align);
                    let end = start.plus(size);

                    self.cursor = end;
                    start
                },
                None => {
                    self.alloc_from_global(size, align)
                }
            }
        } else {
            // we need to alloc from global space
            self.alloc_from_global(size, align)
        }
    }
    
    fn alloc_from_global(&mut self, size: usize, align: usize) -> Address {
        trace!("Mutator{}: slowpath: alloc_from_global", self.id);
        
        self.return_block();

        loop {
            // check if yield
            self.yieldpoint();
            
            let new_block : Option<Box<ImmixBlock>> = self.space.get_next_usable_block();
            
            match new_block {
                Some(b) => {
                    // zero the block - do not need to zero the block here
                    // we zero lines that get used in try_alloc_from_local()
//                    b.lazy_zeroing();

                    self.block    = Some(b);
                    self.cursor   = self.block().start();
                    self.limit    = self.block().start();
                    self.line     = 0;

                    trace!("Mutator{}: slowpath: new block starting from 0x{:x}", self.id, self.cursor);

                    return self.try_alloc_from_local(size, align);
                },
                None => {continue; }
            }
        }
    }
    
    pub fn prepare_for_gc(&mut self) {
        self.return_block();
    }
    
    pub fn id(&self) -> usize {
        self.id
    }

    fn return_block(&mut self) {
        if self.block.is_some() {
            trace!("finishing block {:?}", self.block.as_ref().unwrap());

            if cfg!(debug_assertions) {
                let block = self.block.as_ref().unwrap();
                ImmixMutatorLocal::sanity_check_finished_block(block);
            }

            self.space.return_used_block(self.block.take().unwrap());
        }        
    }

    #[cfg(feature = "use-sidemap")]
    #[allow(unused_variables)]
    fn sanity_check_finished_block(block: &ImmixBlock) {

    }

    #[cfg(not(feature = "use-sidemap"))]
    #[allow(unused_variables)]
    fn sanity_check_finished_block(block: &ImmixBlock) {

    }

    fn block(&mut self) -> &mut ImmixBlock {
        self.block.as_mut().unwrap()
    }
    
    pub fn print_object(&self, obj: Address, length: usize) {
        ImmixMutatorLocal::print_object_static(obj, length);
    }
    
    pub fn print_object_static(obj: Address, length: usize) {
        debug!("===Object {:#X} size: {} bytes===", obj, length);
        let mut cur_addr = obj;
        while cur_addr < obj.plus(length) {
            debug!("Address: {:#X}   {:#X}", cur_addr, unsafe {cur_addr.load::<u64>()});
            cur_addr = cur_addr.plus(8);
        }
        debug!("----");
        debug!("=========");        
    }
}

impl ImmixMutatorGlobal {
    pub fn new() -> ImmixMutatorGlobal {
        ImmixMutatorGlobal {
            take_yield: AtomicBool::new(false),
            still_blocked: AtomicBool::new(false)
        }
    }
    
    #[inline(always)]
    pub fn is_still_blocked(&self) -> bool {
        self.still_blocked.load(Ordering::SeqCst)
    }
    pub fn set_still_blocked(&self, b : bool) {
        self.still_blocked.store(b, Ordering::SeqCst);
    }
    
    pub fn set_take_yield(&self, b : bool) {
        self.take_yield.store(b, Ordering::SeqCst);
    }
    #[inline(always)]
    pub fn take_yield(&self) -> bool{
        self.take_yield.load(Ordering::SeqCst)
    }
}

impl fmt::Display for ImmixMutatorLocal {
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
