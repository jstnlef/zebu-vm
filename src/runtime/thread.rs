#![allow(dead_code)]

use ast::ir::*;
use ast::ptr::*;
use ast::types::*;
use vm::VM;
use runtime::RuntimeValue;
use runtime::gc;

use utils::ByteSize;
use utils::Address;
use utils::mem::memmap;
use utils::mem::memsec;

use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

pub const STACK_SIZE : ByteSize = (4 << 20); // 4mb

#[cfg(target_arch = "x86_64")]
pub const PAGE_SIZE  : ByteSize = (4 << 10); // 4kb

impl_mu_entity!(MuThread);
impl_mu_entity!(MuStack);

pub struct MuStack {
    pub hdr: MuEntityHeader,
    
    func_id: MuID,
    
    state: MuStackState,
    
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
    
    // this frame pointers should only be used when stack is not active
    sp : Address,
    bp : Address,
    ip : Address,
    
    exception_obj  : Option<Address>,
    
    #[allow(dead_code)]
    mmap           : memmap::Mmap
}

impl MuStack {
    pub fn new(id: MuID, func: &MuFunction) -> MuStack {
        let total_size = PAGE_SIZE * 2 + STACK_SIZE;
        
        let anon_mmap = match memmap::Mmap::anonymous(total_size, memmap::Protection::ReadWrite) {
            Ok(m) => m,
            Err(_) => panic!("failed to mmap for a stack"),
        };
        
        let mmap_start = Address::from_ptr(anon_mmap.ptr());
        debug_assert!(mmap_start.is_aligned_to(PAGE_SIZE));
        
        let overflow_guard = mmap_start;
        let lower_bound = mmap_start.plus(PAGE_SIZE);
        let upper_bound = lower_bound.plus(STACK_SIZE);
        let underflow_guard = upper_bound;
        
        unsafe {
            memsec::mprotect(overflow_guard.to_ptr_mut::<u8>(),  PAGE_SIZE, memsec::Prot::NoAccess);
            memsec::mprotect(underflow_guard.to_ptr_mut::<u8>(), PAGE_SIZE, memsec::Prot::NoAccess);
        }
        
        debug!("creating stack {} with entry func {:?}", id, func);
        debug!("overflow_guard : {}", overflow_guard);
        debug!("lower_bound    : {}", lower_bound);
        debug!("upper_bound    : {}", upper_bound);
        debug!("underflow_guard: {}", underflow_guard);
        
        MuStack {
            hdr: MuEntityHeader::unnamed(id),
            func_id: func.id(),
            
            state: MuStackState::Ready(func.sig.arg_tys.clone()),
            
            size: STACK_SIZE,
            overflow_guard: overflow_guard,
            lower_bound: lower_bound,
            upper_bound: upper_bound,
            underflow_guard: upper_bound,
            
            sp: upper_bound,
            bp: upper_bound,
            ip: unsafe {Address::zero()},
            
            exception_obj: None,
            
            mmap: anon_mmap
        }
    }
}

pub enum MuStackState {
    Ready(Vec<P<MuType>>), // ready to resume when values of given types are supplied (can be empty)
    Active,
    Dead
}

pub struct MuThread {
    pub hdr: MuEntityHeader,
    allocator: Box<gc::Mutator>,
    stack: Option<Box<MuStack>>,
    
    user_tls: Option<Address>
}

impl MuThread {
    pub fn new(id: MuID, allocator: Box<gc::Mutator>, stack: Box<MuStack>, user_tls: Option<Address>) -> MuThread {
        MuThread {
            hdr: MuEntityHeader::unnamed(id),
            allocator: allocator,
            stack: Some(stack),
            user_tls: user_tls
        }
    }
    
    pub fn launch(id: MuID, stack: Box<MuStack>, user_tls: Option<Address>, vals: Vec<RuntimeValue>, vm: &VM) -> JoinHandle<()> {
        match thread::Builder::new().name(format!("Mu Thread #{}", id)).spawn(move || {
            let mut muthread = Box::new(MuThread::new(id, gc::new_mutator(), stack, user_tls));
            MuThread::thread_entry(muthread, vm);
        }) {
            Ok(handle) => handle,
            Err(_) => panic!("failed to create a thread")
        }
    }
    
    /// entry function for launching a mu thread
    pub fn thread_entry(mu_thread: Box<MuThread>, vm: &VM) -> ! {
        
    }
}