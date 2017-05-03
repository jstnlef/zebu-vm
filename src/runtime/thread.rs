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
use utils::Word;
use utils::mem::memmap;
use utils::mem::memsec;

use std::ptr;
use std::thread;
use std::thread::JoinHandle;
use std::sync::Arc;
use std::fmt;

pub const STACK_SIZE : ByteSize = (4 << 20); // 4mb

#[cfg(target_arch = "x86_64")]
pub const PAGE_SIZE  : ByteSize = (4 << 10); // 4kb

impl_mu_entity!(MuThread);
impl_mu_entity!(MuStack);

#[repr(C)]
pub struct MuStack {
    pub hdr: MuEntityHeader,

    // address, id
    func: Option<(ValueLocation, MuID)>,
    
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
    
    state: MuStackState,
    #[allow(dead_code)]
    mmap           : Option<memmap::Mmap>
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
            func: Some((func_addr, func.id())),
            
            state: MuStackState::Ready(func.sig.arg_tys.clone()),
            
            size: STACK_SIZE,
            overflow_guard: overflow_guard,
            lower_bound: lower_bound,
            upper_bound: upper_bound,
            underflow_guard: upper_bound,
            
            sp: upper_bound,
            bp: upper_bound,
            ip: unsafe {Address::zero()},
            
            mmap: Some(anon_mmap)
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
                RegGroup::GPREX => unimplemented!(),
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

        // save it back
        self.sp = stack_ptr;
        
        self.print_stack(Some(20));
    }
    
    pub fn print_stack(&self, n_entries: Option<usize>) {
        use compiler::backend::Word;
        use compiler::backend::WORD_SIZE;
        
        let mut cursor = self.upper_bound.sub(WORD_SIZE);
        let mut count = 0;
        
        debug!("0x{:x} | UPPER_BOUND", self.upper_bound);
        while cursor >= self.lower_bound {
            let val = unsafe{cursor.load::<Word>()};
            
            if cursor == self.sp {
                debug!("0x{:x} | 0x{:x} ({}) <- SP", cursor, val, val);
            } else {
                debug!("0x{:x} | 0x{:x} ({})", cursor, val, val);
            }
            
            cursor = cursor.sub(WORD_SIZE);
            count += 1;
            
            if n_entries.is_some() && count > n_entries.unwrap() {
                debug!("...");
                break;
            }
        }
        
        debug!("0x{:x} | LOWER_BOUND", self.lower_bound);
    }

    pub fn print_backtrace(&self) {

    }
}

pub enum MuStackState {
    Ready(Vec<P<MuType>>), // ready to resume when values of given types are supplied (can be empty)
    Active,
    Dead
}

#[repr(C)]
pub struct MuThread {
    pub hdr: MuEntityHeader,
    pub allocator: mm::Mutator,
    pub stack: Option<Box<MuStack>>,
    
    pub native_sp_loc: Address,
    pub user_tls: Address, // can be zero

    pub exception_obj: Address,
    pub vm: Arc<VM>
}

// this depends on the layout of MuThread
lazy_static! {
    pub static ref ALLOCATOR_OFFSET     : usize = offset_of!(MuThread=>allocator).get_byte_offset();
    pub static ref NATIVE_SP_LOC_OFFSET : usize = offset_of!(MuThread=>native_sp_loc).get_byte_offset();
    pub static ref USER_TLS_OFFSET      : usize = offset_of!(MuThread=>user_tls).get_byte_offset();
    pub static ref EXCEPTION_OBJ_OFFSET : usize = offset_of!(MuThread=>exception_obj).get_byte_offset();
}

impl fmt::Display for MuThread {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MuThread    @{:?}: {}\n", self as *const MuThread, self.hdr).unwrap();
        write!(f, "- header    @{:?}\n",       &self.hdr as *const MuEntityHeader).unwrap();
        write!(f, "- allocator @{:?}\n",       &self.allocator as *const mm::Mutator).unwrap();
        write!(f, "- stack     @{:?}: {}\n", &self.stack as *const Option<Box<MuStack>>, self.stack.is_some()).unwrap();
        write!(f, "- native sp @{:?}: {}\n", &self.native_sp_loc as *const Address, self.native_sp_loc).unwrap();
        write!(f, "- user_tls  @{:?}: {}\n", &self.user_tls as *const Address, self.user_tls).unwrap();
        write!(f, "- exc obj   @{:?}: {}\n", &self.exception_obj as *const Address, self.exception_obj).unwrap();

        Ok(())
    }
}

#[cfg(target_arch = "x86_64")]
#[cfg(any(target_os = "macos", target_os = "linux"))]
#[link(name = "runtime")]
extern "C" {
    pub fn set_thread_local(thread: *mut MuThread);
    pub fn muentry_get_thread_local() -> Address;
}

#[cfg(target_arch = "x86_64")]
#[cfg(any(target_os = "macos", target_os = "linux"))]
#[link(name = "swap_stack")]
extern "C" {
    fn swap_to_mu_stack(new_sp: Address, entry: Address, old_sp_loc: Address);
    fn fake_swap_mu_thread(old_sp_loc: Address);
    fn muentry_swap_back_to_native_stack(sp_loc: Address);
    pub fn get_current_frame_rbp() -> Address;
    pub fn exception_restore(dest: Address, callee_saved: *const Word, rsp: Address) -> !;
}

impl MuThread {
    pub fn new(id: MuID, allocator: mm::Mutator, stack: Box<MuStack>, user_tls: Address, vm: Arc<VM>) -> MuThread {
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
    pub fn has_current() -> bool {
        ! unsafe {muentry_get_thread_local()}.is_zero()
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

    #[allow(unused_unsafe)]
    // pieces of this function are not safe (we want to mark it unsafe)
    // this function is exposed as unsafe because it is not always safe to call it
    /// returns true if we have created MuThread on this call
    /// (false means we had MuThread for current thread before)
    pub unsafe fn current_thread_as_mu_thread(threadlocal: Address, vm: Arc<VM>) -> bool {
        use std::usize;

        if ! unsafe{muentry_get_thread_local()}.is_zero() {
            warn!("current thread has a thread local (has a muthread to it)");
            return false;
        }

        // fake a stack for current thread
        let fake_mu_stack_for_cur = Box::new(MuStack {
            hdr: MuEntityHeader::unnamed(vm.next_id()),
            func: None,

            state: MuStackState::Active,

            // we do not know anything about current stack
            // treat it as max size
            size: usize::MAX,
            overflow_guard: unsafe {Address::zero()},
            lower_bound: unsafe {Address::zero()},
            upper_bound: unsafe {Address::max()},
            underflow_guard: unsafe {Address::max()},

            // these will only be used when stack is not active (we still save to these fields)
            // their values do not matter now
            sp: unsafe {Address::zero()},
            bp: unsafe {Address::zero()},
            ip: unsafe {Address::zero()},

            // we are not responsible for keeping the memory alive
            mmap: None,
        });

        let fake_mu_thread = MuThread {
            hdr: MuEntityHeader::unnamed(vm.next_id()),

            // valid allocator and stack
            allocator: mm::new_mutator(),
            stack: Some(fake_mu_stack_for_cur),

            // we do not need native_sp_loc (we do not expect the thread to call
            native_sp_loc: unsafe {Address::zero()},
            user_tls: threadlocal,

            vm: vm,
            exception_obj: unsafe {Address::zero()}
        };

        let ptr_fake_mu_thread : *mut MuThread = Box::into_raw(Box::new(fake_mu_thread));

        // set thread local
        unsafe {set_thread_local(ptr_fake_mu_thread)};

//        let addr = unsafe {muentry_get_thread_local()};
//        let sp_threadlocal_loc = addr.plus(*NATIVE_SP_LOC_OFFSET);
//
//        unsafe {
//            fake_swap_mu_thread(sp_threadlocal_loc);
//        }

        true
    }

    /// turn this current mu thread back as normal thread
    #[allow(unused_variables)]
    pub unsafe fn cleanup_current_mu_thread() {
        let mu_thread_addr = muentry_get_thread_local();

        if !mu_thread_addr.is_zero() {
            let mu_thread : *mut MuThread = mu_thread_addr.to_ptr_mut();
            mm::drop_mutator(&mut (*mu_thread).allocator as *mut mm::Mutator);

            let mu_thread : Box<MuThread> = Box::from_raw(mu_thread);

            // set thread local to zero
            set_thread_local(ptr::null_mut())

            // drop mu_thread here
        }
    }
    
    pub fn new_thread_normal(mut stack: Box<MuStack>, threadlocal: Address, vals: Vec<ValueLocation>, vm: Arc<VM>) -> JoinHandle<()> {
        // set up arguments on stack
        stack.runtime_load_args(vals);
        
        MuThread::mu_thread_launch(vm.next_id(), stack, threadlocal, vm)
    }
    
    #[no_mangle]
    #[allow(unused_variables)]
    pub extern fn mu_thread_launch(id: MuID, stack: Box<MuStack>, user_tls: Address, vm: Arc<VM>) -> JoinHandle<()> {
        let new_sp = stack.sp;
        let entry = runtime::resolve_symbol(vm.name_of(stack.func.as_ref().unwrap().1));
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

    pub has_const_args: bool,
    pub args: Vec<Constant>
}
