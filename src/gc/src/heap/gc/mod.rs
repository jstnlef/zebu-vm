use heap::immix::MUTATORS;
use heap::immix::N_MUTATORS;
use heap::immix::ImmixMutatorLocal;
use heap::immix::ImmixSpace;
use heap::freelist::FreeListSpace;
use objectmodel;
use heap::Space;
use MY_GC;

use utils::{Address, ObjectReference};
use utils::POINTER_SIZE;
use utils::bit_utils;

use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::{Arc, Mutex, Condvar, RwLock};

use crossbeam::sync::chase_lev::*;
use std::sync::mpsc;
use std::sync::mpsc::channel;
use std::thread;

use std::sync::atomic;

lazy_static! {
    static ref STW_COND : Arc<(Mutex<usize>, Condvar)> = {
        Arc::new((Mutex::new(0), Condvar::new()))
    };
    
    static ref ROOTS : RwLock<Vec<ObjectReference>> = RwLock::new(vec![]);
}

static CONTROLLER : AtomicIsize = atomic::ATOMIC_ISIZE_INIT;
const  NO_CONTROLLER : isize    = -1;

pub fn init(n_gcthreads: usize) {
    CONTROLLER.store(NO_CONTROLLER, Ordering::SeqCst);

    GC_THREADS.store(n_gcthreads, Ordering::SeqCst);
}

pub fn trigger_gc() {
    trace!("Triggering GC...");
    
    for mut m in MUTATORS.write().unwrap().iter_mut() {
        if m.is_some() {
            m.as_mut().unwrap().set_take_yield(true);
        }
    }
}

use std::os::raw::c_void;
#[cfg(target_arch = "x86_64")]
#[link(name = "gc_clib_x64")]
extern "C" {
    pub fn malloc_zero(size: usize) -> *const c_void;
    fn immmix_get_stack_ptr() -> Address;
    pub fn set_low_water_mark();
    fn get_low_water_mark() -> Address;
    fn get_registers() -> *const Address;
    fn get_registers_count() -> i32;
}

pub fn stack_scan() -> Vec<ObjectReference> {
    trace!("stack scanning...");
    let stack_ptr : Address = unsafe {immmix_get_stack_ptr()};
    
    if cfg!(debug_assertions) {
        if !stack_ptr.is_aligned_to(8) {
            use std::process;
            println!("trying to scanning stack, however the current stack pointer is 0x{:x}, which is not aligned to 8bytes", stack_ptr);
            process::exit(102);
        }
    }
    
    let low_water_mark : Address = unsafe {get_low_water_mark()};
    
    let mut cursor = stack_ptr;
    let mut ret = vec![];

    let gccontext_guard = MY_GC.read().unwrap();
    let gccontext = gccontext_guard.as_ref().unwrap();

    let immix_space = gccontext.immix_space.clone();
    let lo_space = gccontext.lo_space.clone();
    
    while cursor < low_water_mark {
        let value : Address = unsafe {cursor.load::<Address>()};
        
        if immix_space.is_valid_object(value) || lo_space.is_valid_object(value) {
            ret.push(unsafe {value.to_object_reference()});
        }
        
        cursor = cursor.plus(POINTER_SIZE);
    }
    
    let roots_from_stack = ret.len();
    
    let registers_count = unsafe {get_registers_count()};
    let registers = unsafe {get_registers()};
    
    for i in 0..registers_count {
        let value = unsafe {*registers.offset(i as isize)};
        
        if immix_space.is_valid_object(value) || lo_space.is_valid_object(value){
            ret.push(unsafe {value.to_object_reference()});
        }
    }
    
    let roots_from_registers = ret.len() - roots_from_stack;
    
    trace!("roots: {} from stack, {} from registers", roots_from_stack, roots_from_registers);
    
    ret
}

#[inline(never)]
pub fn sync_barrier(mutator: &mut ImmixMutatorLocal) {
    let controller_id = CONTROLLER.compare_and_swap(-1, mutator.id() as isize, Ordering::SeqCst);
    
    trace!("Mutator{} saw the controller is {}", mutator.id(), controller_id);
    
    // prepare the mutator for gc - return current block (if it has)
    mutator.prepare_for_gc();
    
    // scan its stack
    let mut thread_roots = stack_scan();
    ROOTS.write().unwrap().append(&mut thread_roots);
    
    // user thread call back to prepare for gc
//    USER_THREAD_PREPARE_FOR_GC.read().unwrap()();
    
    if controller_id != NO_CONTROLLER {
        // this thread will block
        block_current_thread(mutator);
        
        // reset current mutator
        mutator.reset_after_gc();
    } else {
        // this thread is controller
        // other threads should block
        
        // wait for all mutators to be blocked
        let &(ref lock, ref cvar) = &*STW_COND.clone();
        let mut count = 0;
        
        trace!("expect {} mutators to park", *N_MUTATORS.read().unwrap() - 1);
        while count < *N_MUTATORS.read().unwrap() - 1 {
            let new_count = {*lock.lock().unwrap()};
            if new_count != count {				
                count = new_count;
                trace!("count = {}", count);
            }
        }
        
        trace!("everyone stopped, gc will start");
        
        // roots->trace->sweep
        gc();
        
        // mutators will resume
        CONTROLLER.store(NO_CONTROLLER, Ordering::SeqCst);
        for mut t in MUTATORS.write().unwrap().iter_mut() {
            if t.is_some() {
                let t_mut = t.as_mut().unwrap();
                t_mut.set_take_yield(false);
                t_mut.set_still_blocked(false);
            }
        }
        // every mutator thread will reset themselves, so only reset current mutator here
        mutator.reset_after_gc();

        // resume
        {
            let mut count = lock.lock().unwrap();
            *count = 0;
            cvar.notify_all();
        }
    }
}

fn block_current_thread(mutator: &mut ImmixMutatorLocal) {
    trace!("Mutator{} blocked", mutator.id());
    
    let &(ref lock, ref cvar) = &*STW_COND.clone();
    let mut count = lock.lock().unwrap();
    *count += 1;
    
    mutator.global.set_still_blocked(true);
    
    while mutator.global.is_still_blocked() {
        count = cvar.wait(count).unwrap();
    }
    
    trace!("Mutator{} unblocked", mutator.id());
}

pub static GC_COUNT : atomic::AtomicUsize = atomic::ATOMIC_USIZE_INIT;

fn gc() {
    GC_COUNT.store(GC_COUNT.load(atomic::Ordering::SeqCst) + 1, atomic::Ordering::SeqCst);
    
    trace!("GC starts");
    
    // creates root deque
    let mut roots : &mut Vec<ObjectReference> = &mut ROOTS.write().unwrap();
    
    // mark & trace
    {
        let gccontext_guard = MY_GC.read().unwrap();
        let gccontext = gccontext_guard.as_ref().unwrap();
        let (immix_space, lo_space) = (&gccontext.immix_space, &gccontext.lo_space);
        
        start_trace(&mut roots, immix_space.clone(), lo_space.clone());
    }
    
    trace!("trace done");
    
    // sweep
    {
        let gccontext_guard = MY_GC.read().unwrap();
        let gccontext = gccontext_guard.as_ref().unwrap();

        let ref immix_space = gccontext.immix_space;
        immix_space.sweep();

        let ref lo_space = gccontext.lo_space;
        lo_space.sweep();
    }
    
    objectmodel::flip_mark_state();
    trace!("GC finishes");
}

pub const MULTI_THREAD_TRACE_THRESHOLD : usize = 10;

pub const PUSH_BACK_THRESHOLD : usize = 50;
pub static GC_THREADS : atomic::AtomicUsize = atomic::ATOMIC_USIZE_INIT;

#[allow(unused_variables)]
#[inline(never)]
pub fn start_trace(work_stack: &mut Vec<ObjectReference>, immix_space: Arc<ImmixSpace>, lo_space: Arc<FreeListSpace>) {
    // creates root deque
    let (mut worker, stealer) = deque();
    
    while !work_stack.is_empty() {
        worker.push(work_stack.pop().unwrap());
    }

    loop {
        let (sender, receiver) = channel::<ObjectReference>();        
        
        let mut gc_threads = vec![];
        for _ in 0..GC_THREADS.load(atomic::Ordering::SeqCst) {
            let new_immix_space = immix_space.clone();
            let new_lo_space = lo_space.clone();
            let new_stealer = stealer.clone();
            let new_sender = sender.clone();
            let t = thread::spawn(move || {
                start_steal_trace(new_stealer, new_sender, new_immix_space, new_lo_space);
            });
            gc_threads.push(t);
        }
        
        // only stealers own sender, when all stealers quit, the following loop finishes
        drop(sender);
        
        loop {
            let recv = receiver.recv();
            match recv {
                Ok(obj) => worker.push(obj),
                Err(_) => break
            }
        }
        
        match worker.try_pop() {
            Some(obj_ref) => worker.push(obj_ref),
            None => break
        }
    }
}

#[allow(unused_variables)]
fn start_steal_trace(stealer: Stealer<ObjectReference>, job_sender:mpsc::Sender<ObjectReference>, immix_space: Arc<ImmixSpace>, lo_space: Arc<FreeListSpace>) {
    use objectmodel;
    
    let mut local_queue = vec![];
    let mark_state = objectmodel::load_mark_state();
    
    loop {
        let work = {
            if !local_queue.is_empty() {
                local_queue.pop().unwrap()
            } else {
                let work = stealer.steal();
                match work {
                    Steal::Empty => return,
                    Steal::Abort => continue,
                    Steal::Data(obj) => obj
                }
            }
        };
        
        steal_trace_object(work, &mut local_queue, &job_sender, mark_state, &immix_space, &lo_space);
    }
} 

#[inline(always)]
pub fn steal_trace_object(obj: ObjectReference, local_queue: &mut Vec<ObjectReference>, job_sender: &mpsc::Sender<ObjectReference>, mark_state: u8, immix_space: &ImmixSpace, lo_space: &FreeListSpace) {
    if cfg!(debug_assertions) {
        // check if this object in within the heap, if it is an object
        if !immix_space.is_valid_object(obj.to_address()) && !lo_space.is_valid_object(obj.to_address()){
            use std::process;
            
            println!("trying to trace an object that is not valid");
            println!("address: 0x{:x}", obj);
            println!("---");
            println!("immix space: {}", immix_space);
            println!("lo space: {}", lo_space);
            
            println!("invalid object during tracing");
            process::exit(101);
        }
    }
    
    let addr = obj.to_address();
    
    let (alloc_map, space_start) = if immix_space.addr_in_space(addr) {
        // mark object
        objectmodel::mark_as_traced(immix_space.trace_map(), immix_space.start(), obj, mark_state);

        // mark line
        immix_space.line_mark_table.mark_line_live(addr);

        (immix_space.alloc_map(), immix_space.start())
    } else if lo_space.addr_in_space(addr) {
        // mark object
        objectmodel::mark_as_traced(lo_space.trace_map(), lo_space.start(), obj, mark_state);
        trace!("mark object @ {} to {}", obj, mark_state);

        (lo_space.alloc_map(), lo_space.start())
    } else {
        println!("unexpected address: {}", addr);
        println!("immix space: {}", immix_space);
        println!("lo space   : {}", lo_space);

        panic!("error during tracing object")
    };
    
    let mut base = addr;
    loop {
        let value = objectmodel::get_ref_byte(alloc_map, space_start, obj);
        let (ref_bits, short_encode) = (bit_utils::lower_bits(value, objectmodel::REF_BITS_LEN), bit_utils::test_nth_bit(value, objectmodel::SHORT_ENCODE_BIT));
        match ref_bits {
            0b0000_0000 => {

            },
            0b0000_0001 => {
                steal_process_edge(base, 0, local_queue, job_sender, mark_state, immix_space, lo_space);
            },            
            0b0000_0011 => {
                steal_process_edge(base, 0, local_queue, job_sender, mark_state, immix_space, lo_space);
                steal_process_edge(base, 8, local_queue, job_sender, mark_state, immix_space, lo_space);
            },
            0b0000_1111 => {
                steal_process_edge(base, 0, local_queue, job_sender, mark_state, immix_space, lo_space);
                steal_process_edge(base, 8, local_queue, job_sender, mark_state, immix_space, lo_space);
                steal_process_edge(base, 16,local_queue, job_sender, mark_state, immix_space, lo_space);
                steal_process_edge(base, 24,local_queue, job_sender, mark_state, immix_space, lo_space);
            },            
            _ => {
                error!("unexpected ref_bits patterns: {:b}", ref_bits);
                unimplemented!()
            }
        }

        if short_encode {
            return;
        } else {
            base = base.plus(objectmodel::REF_BITS_LEN * POINTER_SIZE);
        } 
    }
}

#[inline(always)]
pub fn steal_process_edge(base: Address, offset: usize, local_queue:&mut Vec<ObjectReference>, job_sender: &mpsc::Sender<ObjectReference>, mark_state: u8, immix_space: &ImmixSpace, lo_space: &FreeListSpace) {
    let field_addr = base.plus(offset);
    let edge = unsafe{field_addr.load::<ObjectReference>()};
    
    if cfg!(debug_assertions) {
        use std::process;        
        // check if this object in within the heap, if it is an object
        if !edge.to_address().is_zero() && !immix_space.is_valid_object(edge.to_address()) && !lo_space.is_valid_object(edge.to_address()) {
            println!("trying to follow an edge that is not a valid object");
            println!("edge address: 0x{:x} from 0x{:x}", edge, field_addr);
            println!("base address: 0x{:x}", base);
            println!("---");
            if immix_space.addr_in_space(base) {
                objectmodel::print_object(base, immix_space.start(), immix_space.trace_map(), immix_space.alloc_map());
                println!("---");
                println!("immix space:{}", immix_space);
            } else if lo_space.addr_in_space(base) {
                objectmodel::print_object(base, lo_space.start(), lo_space.trace_map(), lo_space.alloc_map());
                println!("---");
                println!("lo space:{}", lo_space);
            } else {
                println!("not in immix/lo space")
            }
            
            println!("invalid object during tracing");
            process::exit(101);
        }
    }

    if !edge.to_address().is_zero() {
        if immix_space.addr_in_space(edge.to_address()) && !objectmodel::is_traced(immix_space.trace_map(), immix_space.start(), edge, mark_state) {
            if local_queue.len() >= PUSH_BACK_THRESHOLD {
                job_sender.send(edge).unwrap();
            } else {
                local_queue.push(edge);
            }
        } else if lo_space.addr_in_space(edge.to_address()) && !objectmodel::is_traced(lo_space.trace_map(), lo_space.start(), edge, mark_state) {
            if local_queue.len() >= PUSH_BACK_THRESHOLD {
                job_sender.send(edge).unwrap();
            } else {
                local_queue.push(edge);
            }
        }
    }
}