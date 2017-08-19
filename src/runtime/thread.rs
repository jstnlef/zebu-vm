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

use ast::ir::*;
use ast::ptr::*;
use ast::types::*;
use vm::VM;
use runtime::ValueLocation;
use runtime::mm;

use utils::ByteSize;
use utils::Address;
use utils::Word;
use utils::POINTER_SIZE;
use utils::mem::memmap;
use utils::mem::memsec;

use std;
use std::ptr;
use std::thread;
use std::thread::JoinHandle;
use std::sync::Arc;
use std::fmt;

/// a 4mb Mu stack
#[cfg(not(feature = "sel4-rumprun"))]
pub const STACK_SIZE: ByteSize = (4 << 20); // 4mb
/// a .25mb Mu stack for sel4-rumprun
#[cfg(feature = "sel4-rumprun")]
pub const STACK_SIZE: ByteSize = (4 << 16); // 256kb

/// operating system page size
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub const PAGE_SIZE: ByteSize = (4 << 10); // 4kb

// MuThread and MuStack are MuEntity (has MuID and an optional MuName)
impl_mu_entity!(MuThread);
impl_mu_entity!(MuStack);

/// MuStack represents metadata for a Mu stack.
/// A Mu stack is explicitly different from a native stack that comes with the thread from
/// OS, and is managed by the VM. A Mu stack is logically independent from a Mu thread,
/// as we allow creation of stacks to be independent of thread creation, we allow binding stack
/// to a new thread and swap stack to rebind stacks. A Mu stack is seen as a piece of memory
/// that contains function execution records.

/// Zebu stack has a layout as below:
///                              <- stack grows this way <-
///    lo addr                                                    hi addr
///     | overflow guard page | actual stack ..................... | underflow guard page|
///     |                     |                                    |                     |

/// We use guard page for overflow/underflow detection.
//  FIXME: we need to capture the signal (See Issue #50)
//  FIXME: this mechanics may ignore cases when frame size is larger than page size (Issue #49)
#[repr(C)]
pub struct MuStack {
    pub hdr: MuEntityHeader,

    /// stack size
    size: ByteSize,

    //    lo addr                                                    hi addr
    //     | overflow guard page | actual stack ..................... | underflow guard page|
    //     |                     |                                    |                     |
    // overflowGuard           lowerBound                           upperBound
    //                                                              underflowGuard
    /// start address of overflow guard page
    overflow_guard: Address,
    /// lower bound of the stack
    lower_bound: Address,
    /// upper bound of the stack
    upper_bound: Address,
    /// start address of underflow guard page
    underflow_guard: Address,

    // these frame/instruction pointers should only be used when the stack is not active
    /// stack pointer to the stack (before it becomes inactive)
    sp: Address,
    /// frame pointer to the stack (before it becomes inactive)
    bp: Address,
    /// instruction pointer (before the stack becomes inactive)
    ip: Address,

    /// state of this stack: ready (inactive), active, dead
    //  TODO: we are not using this for now
    state: MuStackState,

    /// the Mmap that keeps this memory alive
    #[allow(dead_code)]
    mmap: Option<memmap::Mmap>
}
lazy_static!{
    pub static ref MUSTACK_SP_OFFSET : usize =
        offset_of!(MuStack=>sp).get_byte_offset();
}
impl MuStack {
    /// creates a new MuStack for given entry function and function address
    pub fn new(id: MuID, func_addr: Address, stack_arg_size: usize) -> MuStack {
        // allocate memory for the stack
        let anon_mmap = {
            // reserve two guard pages more than we need for the stack
            let total_size = PAGE_SIZE * 2 + STACK_SIZE;
            match memmap::Mmap::anonymous(total_size, memmap::Protection::ReadWrite) {
                Ok(m) => m,
                Err(_) => panic!("failed to mmap for a stack")
            }
        };

        let mmap_start = Address::from_ptr(anon_mmap.ptr());
        debug_assert!(mmap_start.is_aligned_to(PAGE_SIZE));

        // calculate the addresses
        let overflow_guard = mmap_start;
        let lower_bound = mmap_start + PAGE_SIZE;
        let upper_bound = lower_bound + STACK_SIZE;
        let underflow_guard = upper_bound;

        // protect the guard pages
        unsafe {
            memsec::mprotect(
                overflow_guard.to_ptr_mut::<u8>(),
                PAGE_SIZE,
                memsec::Prot::NoAccess
            );
            memsec::mprotect(
                underflow_guard.to_ptr_mut::<u8>(),
                PAGE_SIZE,
                memsec::Prot::NoAccess
            );
        }

        // Set up the stack
        let mut sp = upper_bound;
        sp -= stack_arg_size; // Allocate space for the arguments

        // Push entry as the return address
        sp -= POINTER_SIZE;
        unsafe {
            sp.store(func_addr);
        }

        // Push a null frame pointer
        sp -= POINTER_SIZE;
        unsafe {
            sp.store(Address::zero());
        }

        debug!("creating stack {} with entry address {:?}", id, func_addr);
        debug!("overflow_guard : {}", overflow_guard);
        debug!("lower_bound    : {}", lower_bound);
        debug!("stack_pointer  : {}", sp);
        debug!("upper_bound    : {}", upper_bound);
        debug!("underflow_guard: {}", underflow_guard);

        MuStack {
            hdr: MuEntityHeader::unnamed(id),
            state: MuStackState::Unknown,

            size: STACK_SIZE,
            overflow_guard: overflow_guard,
            lower_bound: lower_bound,
            upper_bound: upper_bound,
            underflow_guard: upper_bound,

            sp: sp,
            bp: upper_bound,
            ip: unsafe { Address::zero() },

            mmap: Some(anon_mmap)
        }
    }

    /// sets up arguments for the stack's entry function, so it is ready to be executed.
    /// We use a special calling convention for the entry function: we push all the argument
    /// registers for the platform to the stack. If the argument register is used, we get
    /// its value and push the value. Otherwise we push an empty value (0). swap_to_mu_stack
    /// will consume those values on the stack by popping them one by one.
    /// NOTE: any changes to here need to be reflected in swap_to_mu_stack, which consumes
    /// those values pushed to the stack
    pub fn setup_args(&mut self, vals: Vec<ValueLocation>) {
        use utils::Word;
        use utils::WORD_SIZE;
        use compiler::backend::RegGroup;
        use compiler::backend::{ARGUMENT_FPRS, ARGUMENT_GPRS};

        let mut gpr_used = vec![];
        let mut fpr_used = vec![];

        // load values for each argument
        for i in 0..vals.len() {
            let ref val = vals[i];
            let (reg_group, word) = val.load_value();

            match reg_group {
                RegGroup::GPR => gpr_used.push(word),
                RegGroup::FPR => fpr_used.push(word),
                RegGroup::GPREX => unimplemented!()
            }
        }

        // store floating point argument registers
        for i in 0..ARGUMENT_FPRS.len() {
            self.sp -= WORD_SIZE;
            let val = {
                if i < fpr_used.len() {
                    fpr_used[i]
                } else {
                    0 as Word
                }
            };

            debug!("store {} to {}", val, self.sp);
            unsafe {
                self.sp.store(val);
            }
        }

        // store general purpose argument registers
        for i in 0..ARGUMENT_GPRS.len() {
            self.sp -= WORD_SIZE;
            let val = {
                if i < gpr_used.len() {
                    gpr_used[i]
                } else {
                    0 as Word
                }
            };

            debug!("store {} to {}", val, self.sp);
            unsafe {
                self.sp.store(val);
            }
        }

        if cfg!(debug_assertions) {
            self.print_stack(Some(20));
        }
    }

    /// prints n * POINTER_SIZE slots from the stack top (upper bound)
    /// prints either n slots or until meet the stack bottom (lower bound)
    pub fn print_stack(&self, n_entries: Option<usize>) {
        use utils::Word;
        use utils::WORD_SIZE;

        let mut cursor = self.upper_bound - WORD_SIZE;
        let mut count = 0;

        debug!("0x{:x} | UPPER_BOUND", self.upper_bound);
        while cursor >= self.lower_bound {
            let val = unsafe { cursor.load::<Word>() };

            if cursor == self.sp {
                debug!("0x{:x} | 0x{:x} ({}) <- SP", cursor, val, val);
            } else {
                debug!("0x{:x} | 0x{:x} ({})", cursor, val, val);
            }

            cursor -= WORD_SIZE;
            count += 1;

            if n_entries.is_some() && count > n_entries.unwrap() {
                debug!("...");
                break;
            }
        }

        debug!("0x{:x} | LOWER_BOUND", self.lower_bound);
    }
}

/// MuStackState represents the state for a mu stack
pub enum MuStackState {
    /// ready to resume when values of given types are supplied (can be empty)
    Ready(Vec<P<MuType>>),
    /// running mu code
    Active,
    /// can be destroyed
    Dead,
    Unknown
}

/// MuThread represents metadata for a Mu thread.
/// A Mu thread in Zebu is basically an OS thread (pthread). However, we need to maintain our own
/// thread local info, such as allocator, stack, user-level thread local pointer, exception object,
/// and an Arc reference to the VM.
/// We keep the pointer to MuThread for each thread, so that we can query our MuThread metadata.
/// The user-level thread local pointer can be found within MuThread.
/// The compiler emits code that uses offsets to some fields in this struct.
#[repr(C)]
pub struct MuThread {
    pub hdr: MuEntityHeader,
    /// the allocator from memory manager
    pub allocator: mm::Mutator,
    /// current stack (a thread can execute different stacks, but one stack at a time)
    pub stack: *mut MuStack,
    /// native stack pointer before we switch to this mu stack
    /// (when the thread exits, we restore to native stack, and allow proper destruction)
    pub native_sp_loc: Address,
    /// user supplied thread local address, can be zero
    pub user_tls: Address,
    /// exception object being thrown by the thread
    pub exception_obj: Address,
    /// a pointer to the virtual machine
    pub vm: Arc<VM>
}

// a few field offsets the compiler uses
lazy_static! {
    pub static ref ALLOCATOR_OFFSET     : usize =
        offset_of!(MuThread=>allocator).get_byte_offset();
    pub static ref NATIVE_SP_LOC_OFFSET : usize =
        offset_of!(MuThread=>native_sp_loc).get_byte_offset();
    pub static ref USER_TLS_OFFSET      : usize =
        offset_of!(MuThread=>user_tls).get_byte_offset();
    pub static ref STACK_OFFSET      : usize =
        offset_of!(MuThread=>stack).get_byte_offset();
    pub static ref EXCEPTION_OBJ_OFFSET : usize =
        offset_of!(MuThread=>exception_obj).get_byte_offset();
}

impl fmt::Display for MuThread {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "MuThread    @{:?}: {}\n",
            self as *const MuThread,
            self.hdr
        ).unwrap();
        write!(f, "- header    @{:?}\n", &self.hdr as *const MuEntityHeader).unwrap();
        write!(
            f,
            "- allocator @{:?}\n",
            &self.allocator as *const mm::Mutator
        ).unwrap();
        write!(f, "- stack     @{:?}\n", &self.stack as *const *mut MuStack).unwrap();
        write!(
            f,
            "- native sp @{:?}: {}\n",
            &self.native_sp_loc as *const Address,
            self.native_sp_loc
        ).unwrap();
        write!(
            f,
            "- user_tls  @{:?}: {}\n",
            &self.user_tls as *const Address,
            self.user_tls
        ).unwrap();
        write!(
            f,
            "- exc obj   @{:?}: {}\n",
            &self.exception_obj as *const Address,
            self.exception_obj
        ).unwrap();

        Ok(())
    }
}

use std::os::raw::c_int;

#[cfg(not(feature = "sel4-rumprun-target-side"))]
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(any(target_os = "macos", target_os = "linux"))]
#[link(name = "runtime_asm")]
extern "C" {
    /// swaps from a native stack to a mu stack
    /// we create OS threads with native stack, then before executing any mu code,
    /// we swap to mu stack and execute the entry function
    /// args:
    /// new_sp: stack pointer for the mu stack
    /// old_sp_loc: the location to store native stack pointer so we can later swap back
    fn muthread_start_normal(new_sp: Address, old_sp_loc: Address);

    /// gets base poniter for current frame
    pub fn get_current_frame_bp() -> Address;

    /// restores exception: restore callee saved registers, then set execution to certain point
    /// (this function will not return)
    /// args:
    /// dest: code address to execute (catch block)
    /// callee_saved: a sequence of value that will be restored in order
    /// sp: stack pointer for the new execution
    pub fn exception_restore(dest: Address, callee_saved: *const Word, sp: Address) -> !;
}

#[cfg(not(feature = "sel4-rumprun-target-side"))]
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(any(target_os = "macos", target_os = "linux"))]
#[link(name = "runtime_c")]
#[allow(improper_ctypes)]
extern "C" {
    /// sets thread local for Zebu
    /// Note: this thread local points to a MuThread, which contains all the thread local info
    /// for a Mu thread and further contains a pointer to the user-level thread local
    pub fn set_thread_local(thread: *mut MuThread);
    /// gets thread local for Zebu
    /// compiler may emit calls to this
    pub fn muentry_get_thread_local() -> Address;
    /// sets return value for a Zebu executable image
    /// compiler emits calls to this for SetRetval instruction (internal use only)
    pub fn muentry_set_retval(val: u32);
    /// a C wrapper for checking results written in asm
    fn c_check_result() -> c_int;
}

#[cfg(feature = "sel4-rumprun-target-side")]
#[cfg(target_arch = "x86_64")]
#[link(name = "runtime_asm")]
extern "C" {
    fn swap_to_mu_stack(new_sp: Address, entry: Address, old_sp_loc: Address);
    #[allow(dead_code)]
    fn muentry_swap_back_to_native_stack(sp_loc: Address);
    pub fn get_current_frame_bp() -> Address;
    pub fn exception_restore(dest: Address, callee_saved: *const Word, sp: Address) -> !;
}

#[cfg(feature = "sel4-rumprun-target-side")]
#[cfg(target_arch = "x86_64")]
#[link(name = "runtime_c")]
#[allow(improper_ctypes)]
extern "C" {
    pub fn set_thread_local(thread: *mut MuThread);
    pub fn muentry_get_thread_local() -> Address;
    pub fn muentry_set_retval(val: u32);
    fn c_check_result() -> c_int;
}

/// a Rust wrapper for the C function which returns the last set result
pub fn check_result() -> c_int {
    let result = unsafe { c_check_result() };
    result
}

impl MuThread {
    /// creates a new Mu thread with normal execution
    pub fn new_thread_normal(
        mut stack: Box<MuStack>,
        threadlocal: Address,
        vals: Vec<ValueLocation>,
        vm: Arc<VM>
    ) -> JoinHandle<()> {
        // set up arguments on stack
        stack.setup_args(vals);

        MuThread::mu_thread_launch(vm.next_id(), stack, threadlocal, vm)
    }

    /// creates and launches a mu thread, returns a JoinHandle
    #[no_mangle]
    pub extern "C" fn mu_thread_launch(
        id: MuID,
        stack: Box<MuStack>,
        user_tls: Address,
        vm: Arc<VM>
    ) -> JoinHandle<()> {
        let new_sp = stack.sp;

        match thread::Builder::new()
            .name(format!("Mu Thread #{}", id))
            .spawn(move || {
                let mut muthread = MuThread::new(id, mm::new_mutator(), stack, user_tls, vm);

                // set thread local
                unsafe { set_thread_local(&mut muthread) };

                let addr = unsafe { muentry_get_thread_local() };
                let sp_threadlocal_loc = addr + *NATIVE_SP_LOC_OFFSET;
                debug!("new sp: 0x{:x}", new_sp);
                debug!("sp_store: 0x{:x}", sp_threadlocal_loc);

                unsafe {
                    muthread_start_normal(new_sp, sp_threadlocal_loc);
                }

                debug!("returned to Rust stack. Going to quit");
            }) {
            Ok(handle) => handle,
            Err(_) => panic!("failed to create a thread")
        }
    }

    /// creates metadata for a Mu thread
    fn new(
        id: MuID,
        allocator: mm::Mutator,
        stack: Box<MuStack>,
        user_tls: Address,
        vm: Arc<VM>
    ) -> MuThread {
        MuThread {
            hdr: MuEntityHeader::unnamed(id),
            allocator: allocator,
            stack: Box::into_raw(stack),
            native_sp_loc: unsafe { Address::zero() },
            user_tls: user_tls,
            vm: vm,
            exception_obj: unsafe { Address::zero() }
        }
    }

    /// is current thread a Mu thread?
    #[inline(always)]
    pub fn has_current() -> bool {
        !unsafe { muentry_get_thread_local() }.is_zero()
    }

    /// gets a reference to MuThread for current MuThread
    #[inline(always)]
    pub fn current() -> &'static MuThread {
        unsafe {
            muentry_get_thread_local()
                .to_ptr::<MuThread>()
                .as_ref()
                .unwrap()
        }
    }

    /// gets a mutable reference to MuThread for current MuThread
    #[inline(always)]
    pub fn current_mut() -> &'static mut MuThread {
        unsafe {
            muentry_get_thread_local()
                .to_ptr_mut::<MuThread>()
                .as_mut()
                .unwrap()
        }
    }

    /// disguises current thread as a Mu thread (setup MuThread metadata, thread local for it),
    /// returns true if we have created a MuThread on this call, false means we had MuThread
    /// for current thread before.
    pub unsafe fn current_thread_as_mu_thread(threadlocal: Address, vm: Arc<VM>) -> bool {
        use std::usize;

        // build exception table as we may execute mu function
        vm.build_callsite_table();

        if !muentry_get_thread_local().is_zero() {
            warn!("current thread has a thread local (has a muthread to it)");
            return false;
        }

        // fake a stack for current thread
        let fake_mu_stack_for_cur = Box::new(MuStack {
            hdr: MuEntityHeader::unnamed(vm.next_id()),
            // active state
            state: MuStackState::Active,
            // we do not know anything about current stack
            // treat it as max size
            size: usize::MAX,
            overflow_guard: Address::zero(),
            lower_bound: Address::zero(),
            upper_bound: Address::max(),
            underflow_guard: Address::max(),
            // these will only be used when stack is not active (we still save to these fields)
            // their values do not matter now
            sp: Address::zero(),
            bp: Address::zero(),
            ip: Address::zero(),
            // we are not responsible for keeping the memory alive
            mmap: None
        });

        // fake a thread for current thread
        let fake_mu_thread = MuThread {
            hdr: MuEntityHeader::unnamed(vm.next_id()),
            // we need a valid allocator and stack
            allocator: mm::new_mutator(),
            stack: Box::into_raw(fake_mu_stack_for_cur),
            // we do not need native_sp_loc (we do not expect the thread to call THREADEXIT)
            native_sp_loc: Address::zero(),
            // valid thread local from user
            user_tls: threadlocal,
            vm: vm,
            exception_obj: Address::zero()
        };

        // set thread local
        let ptr_fake_mu_thread: *mut MuThread = Box::into_raw(Box::new(fake_mu_thread));
        set_thread_local(ptr_fake_mu_thread);

        true
    }

    /// turn this current mu thread back as normal thread
    pub unsafe fn cleanup_current_mu_thread() {
        let mu_thread_addr = muentry_get_thread_local();

        if !mu_thread_addr.is_zero() {
            let mu_thread: *mut MuThread = mu_thread_addr.to_ptr_mut();

            // drop allocator
            mm::drop_mutator(&mut (*mu_thread).allocator as *mut mm::Mutator);

            // set thread local to zero
            set_thread_local(ptr::null_mut());

            // get mu thread back to Box (and will get dropped)
            Box::from_raw(mu_thread);
        }
    }
}

/// PrimordialThreadInfo stores information about primordial thread/entry function for a boot image
#[derive(Debug)]
pub struct PrimordialThreadInfo {
    /// entry function id
    pub func_id: MuID,
    /// does user supply some contant arguments to start the primordial thread?
    pub has_const_args: bool,
    /// arguments
    pub args: Vec<Constant>
}

rodal_struct!(PrimordialThreadInfo {
    func_id,
    args,
    has_const_args
});

#[no_mangle]
pub unsafe extern "C" fn muentry_new_stack(entry: Address, stack_size: usize) -> *mut MuStack {
    let ref vm = MuThread::current_mut().vm;
    let stack = Box::new(MuStack::new(vm.next_id(), entry, stack_size));
    Box::into_raw(stack)
}

// Kills the given stack. WARNING! do not call this whilst on the given stack
#[no_mangle]
pub unsafe extern "C" fn muentry_kill_stack(stack: *mut MuStack) {
    // This new box will be destroyed upon returning
    Box::from_raw(stack);
}
