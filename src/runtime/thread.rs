#![allow(dead_code)]

use ast::ir::*;
use ast::ptr::*;
use ast::types::*;
use vm::VM;
use runtime;
use runtime::ValueLocation;
use runtime::mm;

use utils::ByteSize;
use utils::Address;
use utils::mem::memmap;
use utils::mem::memsec;

use std::mem;
use std::thread;
use std::thread::JoinHandle;
use std::sync::Arc;

pub const STACK_SIZE : ByteSize = (4 << 20); // 4mb

#[cfg(target_arch = "x86_64")]
pub const PAGE_SIZE  : ByteSize = (4 << 10); // 4kb

impl_mu_entity!(MuThread);
impl_mu_entity!(MuStack);

#[repr(C)]
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
        
        self.print_stack(Some(20));
    }
    
    pub fn print_stack(&self, n_entries: Option<usize>) {
        use compiler::backend::Word;
        use compiler::backend::WORD_SIZE;
        
        let mut cursor = self.upper_bound.sub(WORD_SIZE);
        let mut count = 0;
        
        println!("0x{:x} | UPPER_BOUND", self.upper_bound); 
        while cursor >= self.lower_bound {
            let val = unsafe{cursor.load::<Word>()};
            print!("0x{:x} | 0x{:x} ({})", cursor, val, val);
            
            if cursor == self.sp {
                print!(" <- SP");
            }
            
            println!("");
            
            cursor = cursor.sub(WORD_SIZE);
            count += 1;
            
            if n_entries.is_some() && count > n_entries.unwrap() {
                println!("...");
                break;
            }
        }
        
        println!("0x{:x} | LOWER_BOUND", self.lower_bound); 
    }
}

pub enum MuStackState {
    Ready(Vec<P<MuType>>), // ready to resume when values of given types are supplied (can be empty)
    Active,
    Dead
}

#[repr(C)]
#[allow(improper_ctypes)]
// do not change the layout (unless change the offset of fields correspondingly)
pub struct MuThread {
    pub hdr: MuEntityHeader,
    allocator: mm::Mutator,
    pub stack: Option<Box<MuStack>>,
    
    native_sp_loc: Address,
    user_tls: Option<Address>,
    
    pub vm: Arc<VM>,
    pub exception_obj: Address
}

// this depends on the layout of MuThread
lazy_static! {
    pub static ref NATIVE_SP_LOC_OFFSET : usize = mem::size_of::<MuEntityHeader>() 
                + mem::size_of::<Box<mm::Mutator>>()
                + mem::size_of::<Option<Box<MuStack>>>();
    
    pub static ref ALLOCATOR_OFFSET : usize = mem::size_of::<MuEntityHeader>();
    
    pub static ref VM_OFFSET : usize = mem::size_of::<MuEntityHeader>() 
                + mem::size_of::<Box<mm::Mutator>>()
                + mem::size_of::<Option<Box<MuStack>>>()
                + mem::size_of::<Address>()
                + mem::size_of::<Option<Address>>();
}

#[cfg(target_arch = "x86_64")]
#[cfg(target_os = "macos")]
#[link(name = "runtime")]
extern "C" {
    pub fn set_thread_local(thread: *mut MuThread);
    pub fn muentry_get_thread_local() -> Address;
}

#[cfg(target_arch = "x86_64")]
#[cfg(target_os = "macos")]
#[link(name = "swap_stack")]
extern "C" {
    fn swap_to_mu_stack(new_sp: Address, entry: Address, old_sp_loc: Address);
    fn muentry_swap_back_to_native_stack(sp_loc: Address);
    pub fn get_current_frame_rbp() -> Address;
}

impl MuThread {
    pub fn new(id: MuID, allocator: mm::Mutator, stack: Box<MuStack>, user_tls: Option<Address>, vm: Arc<VM>) -> MuThread {
        MuThread {
            hdr: MuEntityHeader::unnamed(id),
            allocator: allocator,
            stack: Some(stack),
            native_sp_loc: unsafe {Address::zero()},
            user_tls: user_tls,
            vm: vm,
            exception_obj: unsafe {Address::zero()}
        }
    }
    
    #[inline(always)]
    pub fn current() -> &'static MuThread {
        unsafe{
            muentry_get_thread_local().to_ptr::<MuThread>().as_ref().unwrap()
        }
    }
    
    #[inline(always)]
    pub fn current_mut() -> &'static mut MuThread {
        unsafe{
            muentry_get_thread_local().to_ptr_mut::<MuThread>().as_mut().unwrap()
        }
    }
    
    pub fn new_thread_normal(mut stack: Box<MuStack>, threadlocal: Address, vals: Vec<ValueLocation>, vm: Arc<VM>) -> JoinHandle<()> {
        let user_tls = {
            if threadlocal.is_zero() {
                None
            } else {
                Some(threadlocal)
            }
        };
        
        // set up arguments on stack
        stack.runtime_load_args(vals);
        
        MuThread::mu_thread_launch(vm.next_id(), stack, user_tls, vm)
    }    
    
    #[no_mangle]
    #[allow(unused_variables)]
    pub extern fn mu_thread_launch(id: MuID, stack: Box<MuStack>, user_tls: Option<Address>, vm: Arc<VM>) -> JoinHandle<()> {
        let new_sp = stack.sp;
        let entry = runtime::resolve_symbol(vm.name_of(stack.func_id));
        debug!("entry : 0x{:x}", entry);
        
        match thread::Builder::new().name(format!("Mu Thread #{}", id)).spawn(move || {
            let muthread : *mut MuThread = Box::into_raw(Box::new(MuThread::new(id, mm::new_mutator(), stack, user_tls, vm)));
            
            // set thread local
            unsafe {set_thread_local(muthread)};
            
            let addr = unsafe {muentry_get_thread_local()};
            let sp_threadlocal_loc = addr.plus(*NATIVE_SP_LOC_OFFSET);
            
            debug!("new sp: 0x{:x}", new_sp);
            debug!("sp_store: 0x{:x}", sp_threadlocal_loc);
            
            unsafe {
                swap_to_mu_stack(new_sp, entry, sp_threadlocal_loc); 
            }
            
            debug!("returned to Rust stack. Going to quit");
        }) {
            Ok(handle) => handle,
            Err(_) => panic!("failed to create a thread")
        }
    }
}

#[derive(Debug, RustcEncodable, RustcDecodable)]
pub struct MuPrimordialThread {
    pub func_id: MuID,
    pub args: Vec<Constant>
}