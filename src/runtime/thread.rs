#![allow(dead_code)]

use ast::ir::*;
use ast::ptr::*;
use ast::types::*;
use vm::VM;
use runtime::ValueLocation;
use runtime::gc;

use utils::ByteSize;
use utils::Address;
use utils::mem::memmap;
use utils::mem::memsec;

use std::thread;
use std::thread::JoinHandle;

pub const STACK_SIZE : ByteSize = (4 << 20); // 4mb

#[cfg(target_arch = "x86_64")]
pub const PAGE_SIZE  : ByteSize = (4 << 10); // 4kb

impl_mu_entity!(MuThread);
impl_mu_entity!(MuStack);

pub struct MuStack {
    pub hdr: MuEntityHeader,
    
    func_addr: ValueLocation,
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
    
    // this frame pointers should only be used when stack is not active
    sp : Address,
    bp : Address,
    ip : Address,
    
    exception_obj  : Option<Address>,
    
    state: MuStackState,
    #[allow(dead_code)]
    mmap           : memmap::Mmap
}

impl MuStack {
    pub fn new(id: MuID, func_addr: ValueLocation, func: &MuFunction) -> MuStack {
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
            func_addr: func_addr,
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
    
    #[cfg(target_arch = "x86_64")]
    pub fn runtime_load_args(&mut self, vals: Vec<ValueLocation>) {
        use compiler::backend::Word;
        use compiler::backend::WORD_SIZE;
        use compiler::backend::RegGroup;
        use compiler::backend::x86_64;
        
        let mut gpr_used = vec![];
        let mut fpr_used = vec![];
        
        for i in 0..vals.len() {
            let ref val = vals[i];
            let (reg_group, word) = val.load_value();
            
            match reg_group {
                RegGroup::GPR => gpr_used.push(word),
                RegGroup::FPR => fpr_used.push(word),
            }
        }
        
        let mut stack_ptr = self.sp;
        for i in 0..x86_64::ARGUMENT_FPRs.len() {
            stack_ptr = stack_ptr.sub(WORD_SIZE);
            let val = {
                if i < fpr_used.len() {
                    fpr_used[i]
                } else {
                    0 as Word
                }
            };
            
            debug!("store {} to {}", val, stack_ptr);
            unsafe {stack_ptr.store(val);}
        }
        
        for i in 0..x86_64::ARGUMENT_GPRs.len() {
            stack_ptr = stack_ptr.sub(WORD_SIZE);
            let val = {
                if i < gpr_used.len() {
                    gpr_used[i]
                } else {
                    0 as Word
                }
            };
            
            debug!("store {} to {}", val, stack_ptr);
            unsafe {stack_ptr.store(val);}
        }
        
        // should have put 6 + 6 words on the stack
        debug_assert!(self.sp.diff(stack_ptr) == 12 * WORD_SIZE);
        // save it back
        self.sp = stack_ptr;
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

#[cfg(target_arch = "x86_64")]
#[cfg(target_os = "macos")]
#[link(name = "runtime")]
extern "C" {
    fn init_thread_local(thread: &Box<MuThread>) -> Address;
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
    
    #[no_mangle]
    #[allow(unused_variables)]
    pub extern fn mu_thread_launch(id: MuID, stack: Box<MuStack>, user_tls: Option<Address>, vm: &VM) -> JoinHandle<()> {
        match thread::Builder::new().name(format!("Mu Thread #{}", id)).spawn(move || {
            let muthread = Box::new(MuThread::new(id, gc::new_mutator(), stack, user_tls));
            
            // set thread local
            let addr = unsafe {init_thread_local(&muthread)};
            
            MuThread::thread_entry(muthread);
        }) {
            Ok(handle) => handle,
            Err(_) => panic!("failed to create a thread")
        }
    }
    
    /// entry function for launching a mu thread
    #[allow(unused_variables)]
    pub fn thread_entry(mu_thread: Box<MuThread>) -> ! {
        unimplemented!()
    }
}

#[derive(Debug, RustcEncodable, RustcDecodable)]
pub struct MuPrimordialThread {
    pub func_id: MuID,
    pub args: Vec<Constant>
}