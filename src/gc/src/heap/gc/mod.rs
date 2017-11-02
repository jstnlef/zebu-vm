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

use heap::*;
use objectmodel;
use objectmodel::sidemap::*;
use MY_GC;

use std::sync::atomic::{AtomicIsize, AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Condvar, RwLock};

use crossbeam::sync::chase_lev::*;
use std::sync::mpsc;
use std::sync::mpsc::channel;
use std::thread;
use std::sync::atomic;
use std::mem::transmute;

lazy_static! {
    static ref STW_COND : Arc<(Mutex<usize>, Condvar)> = {
        Arc::new((Mutex::new(0), Condvar::new()))
    };

    static ref ROOTS : RwLock<Vec<ObjectReference>> = RwLock::new(vec![]);
}

pub static ENABLE_GC: AtomicBool = atomic::ATOMIC_BOOL_INIT;

static CONTROLLER: AtomicIsize = atomic::ATOMIC_ISIZE_INIT;
const NO_CONTROLLER: isize = -1;

pub fn init(n_gcthreads: usize) {
    CONTROLLER.store(NO_CONTROLLER, Ordering::SeqCst);
    GC_THREADS.store(n_gcthreads, Ordering::SeqCst);
    GC_COUNT.store(0, Ordering::SeqCst);
}

pub fn trigger_gc() {
    trace!("Triggering GC...");

    for mut m in MUTATORS.write().unwrap().iter_mut() {
        if m.is_some() {
            m.as_mut().unwrap().set_take_yield(true);
        }
    }
}

#[cfg(target_arch = "x86_64")]
#[link(name = "gc_clib_x64")]
extern "C" {
    fn immmix_get_stack_ptr() -> Address;
    pub fn set_low_water_mark();
    fn get_low_water_mark() -> Address;
    fn get_registers() -> *const Address;
    fn get_registers_count() -> i32;
}

#[cfg(target_arch = "aarch64")]
#[link(name = "gc_clib_aarch64")]
extern "C" {
    fn immmix_get_stack_ptr() -> Address;
    pub fn set_low_water_mark();
    fn get_low_water_mark() -> Address;
    fn get_registers() -> *const Address;
    fn get_registers_count() -> i32;
}

pub fn stack_scan() -> Vec<ObjectReference> {
    trace!("stack scanning...");
    let stack_ptr: Address = unsafe { immmix_get_stack_ptr() };

    if cfg!(debug_assertions) {
        if !stack_ptr.is_aligned_to(8) {
            use std::process;
            println!(
                "trying to scanning stack, however the current stack pointer is 0x{:x}, \
                 which is not aligned to 8bytes",
                stack_ptr
            );
            process::exit(102);
        }
    }

    let low_water_mark: Address = unsafe { get_low_water_mark() };

    let mut cursor = stack_ptr;
    let mut ret = vec![];

    let gccontext_guard = MY_GC.read().unwrap();
    let gccontext = gccontext_guard.as_ref().unwrap();

    while cursor < low_water_mark {
        let value: Address = unsafe { cursor.load::<Address>() };

        if gccontext.is_heap_object(value) {
            ret.push(unsafe { value.to_object_reference() });
        }

        cursor = cursor + POINTER_SIZE;
    }

    let roots_from_stack = ret.len();
    let registers_count = unsafe { get_registers_count() };
    let registers = unsafe { get_registers() };

    for i in 0..registers_count {
        let value = unsafe { *registers.offset(i as isize) };

        if gccontext.is_heap_object(value) {
            ret.push(unsafe { value.to_object_reference() });
        }
    }

    let roots_from_registers = ret.len() - roots_from_stack;

    trace!(
        "roots: {} from stack, {} from registers",
        roots_from_stack,
        roots_from_registers
    );

    ret
}

#[inline(never)]
pub fn sync_barrier(mutator: &mut Mutator) {
    let controller_id = CONTROLLER.compare_and_swap(-1, mutator.id() as isize, Ordering::SeqCst);

    trace!(
        "Mutator{} saw the controller is {}",
        mutator.id(),
        controller_id
    );

    // prepare the mutator for gc - return current block (if it has)
    mutator.prepare_for_gc();

    // user thread call back to prepare for gc
    //    USER_THREAD_PREPARE_FOR_GC.read().unwrap()();

    if controller_id != NO_CONTROLLER {
        // scan its stack
        {
            let mut thread_roots = stack_scan();
            ROOTS.write().unwrap().append(&mut thread_roots);
        }

        // this thread will block
        block_current_thread(mutator);

        // reset current mutator
        mutator.reset_after_gc();
    } else {
        // this thread is controller
        // other threads should block

        // init roots
        {
            // scan its stack
            let mut thread_roots = stack_scan();
            ROOTS.write().unwrap().append(&mut thread_roots);
        }

        // wait for all mutators to be blocked
        let &(ref lock, ref cvar) = &*STW_COND.clone();
        let mut count = 0;

        trace!(
            "expect {} mutators to park",
            *N_MUTATORS.read().unwrap() - 1
        );
        while count < *N_MUTATORS.read().unwrap() - 1 {
            let new_count = { *lock.lock().unwrap() };
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

fn block_current_thread(mutator: &mut Mutator) {
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

pub static GC_COUNT: atomic::AtomicUsize = atomic::ATOMIC_USIZE_INIT;

fn gc() {
    if !ENABLE_GC.load(Ordering::SeqCst) {
        panic!("Triggering GC when GC is disabled");
    }

    GC_COUNT.store(
        GC_COUNT.load(atomic::Ordering::SeqCst) + 1,
        atomic::Ordering::SeqCst
    );

    // each space prepares for GC
    {
        let mut gccontext_guard = MY_GC.write().unwrap();
        let mut gccontext = gccontext_guard.as_mut().unwrap();
        gccontext.immix_tiny.prepare_for_gc();
        gccontext.immix_normal.prepare_for_gc();
        gccontext.lo.prepare_for_gc();
    }

    trace!("GC starts");

    // mark & trace
    {
        // creates root deque
        let mut roots: &mut Vec<ObjectReference> = &mut ROOTS.write().unwrap();

        let gccontext_guard = MY_GC.read().unwrap();
        let gccontext = gccontext_guard.as_ref().unwrap();
        for obj in gccontext.roots.iter() {
            roots.push(*obj);
        }

        trace!("total roots: {}", roots.len());

        start_trace(&mut roots);
    }

    trace!("trace done");

    // sweep
    {
        let mut gccontext_guard = MY_GC.write().unwrap();
        let mut gccontext = gccontext_guard.as_mut().unwrap();

        gccontext.immix_tiny.sweep();
        gccontext.immix_normal.sweep();
        gccontext.lo.sweep();
    }

    objectmodel::flip_mark_state();

    // clear existing roots (roots from last gc)
    ROOTS.write().unwrap().clear();

    trace!("GC finishes");
}

pub const PUSH_BACK_THRESHOLD: usize = 50;
pub static GC_THREADS: atomic::AtomicUsize = atomic::ATOMIC_USIZE_INIT;

const TRACE_GC: bool = true;

#[allow(unused_variables)]
#[inline(never)]
pub fn start_trace(work_stack: &mut Vec<ObjectReference>) {
    // creates root deque
    let (worker, stealer) = deque();

    while !work_stack.is_empty() {
        worker.push(work_stack.pop().unwrap());
    }

    loop {
        let (sender, receiver) = channel::<ObjectReference>();

        let mut gc_threads = vec![];
        let n_gcthreads = GC_THREADS.load(atomic::Ordering::SeqCst);
        trace!("launching {} gc threads...", n_gcthreads);
        for _ in 0..n_gcthreads {
            let new_stealer = stealer.clone();
            let new_sender = sender.clone();
            let t = thread::spawn(move || { start_steal_trace(new_stealer, new_sender); });
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
fn start_steal_trace(stealer: Stealer<ObjectReference>, job_sender: mpsc::Sender<ObjectReference>) {
    use objectmodel;

    let mut local_queue = vec![];
    let mark_state = objectmodel::load_mark_state();

    loop {
        let work = {
            if !local_queue.is_empty() {
                let ret = local_queue.pop().unwrap();
                trace_if!(TRACE_GC, "got object {} from local queue", ret);
                ret
            } else {
                let work = stealer.steal();
                let ret = match work {
                    Steal::Empty => return,
                    Steal::Abort => continue,
                    Steal::Data(obj) => obj
                };
                trace_if!(TRACE_GC, "got object {} from global queue", ret);
                ret
            }
        };

        steal_trace_object(work, &mut local_queue, &job_sender, mark_state);
    }
}

#[inline(always)]
#[cfg(feature = "use-sidemap")]
pub fn steal_trace_object(
    obj: ObjectReference,
    local_queue: &mut Vec<ObjectReference>,
    job_sender: &mpsc::Sender<ObjectReference>,
    mark_state: u8
) {
    match SpaceDescriptor::get(obj) {
        SpaceDescriptor::ImmixTiny => {
            let mut space = ImmixSpace::get::<ImmixSpace>(obj.to_address());
            // mark current object traced
            space.mark_object_traced(obj);

            let encode = unsafe {
                space
                    .get_type_byte_slot(space.get_word_index(obj.to_address()))
                    .load::<TinyObjectEncode>()
            };

            trace_if!(TRACE_GC, "  trace tiny obj: {} ({:?})", obj, encode);
            for i in 0..encode.n_fields() {
                trace_word(
                    encode.field(i),
                    obj,
                    (i << LOG_POINTER_SIZE) as ByteOffset,
                    local_queue,
                    job_sender
                )
            }
        }
        SpaceDescriptor::ImmixNormal => {
            let mut space = ImmixSpace::get::<ImmixSpace>(obj.to_address());
            //mark current object traced
            space.mark_object_traced(obj);

            // get type encode
            let (type_encode, type_size): (&TypeEncode, ByteOffset) = {
                let type_slot = space.get_type_byte_slot(space.get_word_index(obj.to_address()));
                let encode = unsafe { type_slot.load::<MediumObjectEncode>() };
                let small_encode: &SmallObjectEncode = unsafe { transmute(&encode) };

                let (type_id, type_size) = if small_encode.is_small() {
                    trace_if!(TRACE_GC, "  trace small obj: {} ({:?})", obj, small_encode);
                    trace_if!(
                        TRACE_GC,
                        "        id {}, size {}",
                        small_encode.type_id(),
                        small_encode.size()
                    );
                    (small_encode.type_id(), small_encode.size())
                } else {
                    trace_if!(TRACE_GC, "  trace medium obj: {} ({:?})", obj, encode);
                    trace_if!(
                        TRACE_GC,
                        "        id {}, size {}",
                        encode.type_id(),
                        encode.size()
                    );
                    (encode.type_id(), encode.size())
                };
                (&GlobalTypeTable::table()[type_id], type_size as ByteOffset)
            };

            let mut offset: ByteOffset = 0;
            trace_if!(TRACE_GC, "  -fix part-");
            for i in 0..type_encode.fix_len() {
                trace_word(type_encode.fix_ty(i), obj, offset, local_queue, job_sender);
                offset += POINTER_SIZE as ByteOffset;
            }
            // for variable part
            if type_encode.var_len() != 0 {
                trace_if!(TRACE_GC, "  -var part-");
                while offset < type_size {
                    for i in 0..type_encode.var_len() {
                        trace_word(type_encode.var_ty(i), obj, offset, local_queue, job_sender);
                        offset += POINTER_SIZE as ByteOffset;
                    }
                }
            }
            trace_if!(TRACE_GC, "  -done-");
        }
        SpaceDescriptor::Freelist => {
            let mut space = FreelistSpace::get::<FreelistSpace>(obj.to_address());
            space.mark_object_traced(obj);

            let encode = space.get_type_encode(obj);
            let tyid = encode.type_id();
            let ty = GlobalTypeTable::get_full_type(tyid);

            let mut offset: ByteOffset = 0;
            // fix part
            for &word_ty in ty.fix.iter() {
                trace_word(word_ty, obj, offset, local_queue, job_sender);
                offset += POINTER_SIZE as ByteOffset;
            }
            if encode.hybrid_len() != 0 {
                // for every hybrid element
                for _ in 0..encode.hybrid_len() {
                    for &word_ty in ty.var.iter() {
                        trace_word(word_ty, obj, offset, local_queue, job_sender);
                        offset += POINTER_SIZE as ByteOffset;
                    }
                }
            }
        }
    }
}

#[inline(always)]
#[cfg(feature = "use-sidemap")]
fn trace_word(
    word_ty: WordType,
    obj: ObjectReference,
    offset: ByteOffset,
    local_queue: &mut Vec<ObjectReference>,
    job_sender: &mpsc::Sender<ObjectReference>
) {
    trace_if!(
        TRACE_GC,
        "  follow field (offset: {}) of {} with type {:?}",
        offset,
        obj,
        word_ty
    );
    match word_ty {
        WordType::NonRef => {}
        WordType::Ref => {
            let field_addr = obj.to_address() + offset;
            let edge = unsafe { field_addr.load::<ObjectReference>() };

            if edge.to_address().is_zero() {
                return;
            }

            match SpaceDescriptor::get(edge) {
                SpaceDescriptor::ImmixTiny | SpaceDescriptor::ImmixNormal => {
                    let space = ImmixSpace::get::<ImmixSpace>(edge.to_address());
                    if !space.is_object_traced(edge) {
                        steal_process_edge(edge, local_queue, job_sender);
                    }
                }
                SpaceDescriptor::Freelist => {
                    let space = FreelistSpace::get::<FreelistSpace>(edge.to_address());
                    if !space.is_object_traced(edge) {
                        debug!("edge {} is not traced, trace it", edge);
                        steal_process_edge(edge, local_queue, job_sender);
                    } else {
                        debug!("edge {} is traced, skip", edge);
                    }
                }
            }
        }
        WordType::WeakRef | WordType::TaggedRef => {
            use std::process;
            error!("unimplemented");
            process::exit(1);
        }
    }
}

#[inline(always)]
#[cfg(feature = "use-sidemap")]
fn steal_process_edge(
    edge: ObjectReference,
    local_queue: &mut Vec<ObjectReference>,
    job_sender: &mpsc::Sender<ObjectReference>
) {
    if local_queue.len() >= PUSH_BACK_THRESHOLD {
        job_sender.send(edge).unwrap();
    } else {
        local_queue.push(edge);
    }
}

#[inline(always)]
#[cfg(not(feature = "use-sidemap"))]
pub fn steal_trace_object(
    obj: ObjectReference,
    local_queue: &mut Vec<ObjectReference>,
    job_sender: &mpsc::Sender<ObjectReference>,
    mark_state: u8,
    immix_space: &ImmixSpace,
    lo_space: &FreeListSpace
) {
    if cfg!(debug_assertions) {
        // check if this object in within the heap, if it is an object
        if !immix_space.is_valid_object(obj.to_address()) &&
            !lo_space.is_valid_object(obj.to_address())
        {
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

    // mark object
    objectmodel::mark_as_traced(obj, mark_state);

    if immix_space.addr_in_space(addr) {
        // mark line
        immix_space.line_mark_table.mark_line_live(addr);
    } else if lo_space.addr_in_space(addr) {
        // do nothing
    } else {
        println!("unexpected address: {}", addr);
        println!("immix space: {}", immix_space);
        println!("lo space   : {}", lo_space);

        panic!("error during tracing object")
    }

    // this part of code has some duplication with code in objectdump
    // FIXME: remove the duplicate code - use 'Tracer' trait

    let hdr = unsafe { (addr + objectmodel::OBJECT_HEADER_OFFSET).load::<u64>() };

    if objectmodel::header_is_fix_size(hdr) {
        // fix sized type
        if objectmodel::header_has_ref_map(hdr) {
            // has ref map
            let ref_map = objectmodel::header_get_ref_map(hdr);

            match ref_map {
                0 => {}
                0b0000_0001 => {
                    steal_process_edge(
                        addr,
                        0,
                        local_queue,
                        job_sender,
                        mark_state,
                        immix_space,
                        lo_space
                    );
                }
                0b0000_0011 => {
                    steal_process_edge(
                        addr,
                        0,
                        local_queue,
                        job_sender,
                        mark_state,
                        immix_space,
                        lo_space
                    );
                    steal_process_edge(
                        addr,
                        8,
                        local_queue,
                        job_sender,
                        mark_state,
                        immix_space,
                        lo_space
                    );
                }
                0b0000_1111 => {
                    steal_process_edge(
                        addr,
                        0,
                        local_queue,
                        job_sender,
                        mark_state,
                        immix_space,
                        lo_space
                    );
                    steal_process_edge(
                        addr,
                        8,
                        local_queue,
                        job_sender,
                        mark_state,
                        immix_space,
                        lo_space
                    );
                    steal_process_edge(
                        addr,
                        16,
                        local_queue,
                        job_sender,
                        mark_state,
                        immix_space,
                        lo_space
                    );
                    steal_process_edge(
                        addr,
                        24,
                        local_queue,
                        job_sender,
                        mark_state,
                        immix_space,
                        lo_space
                    );
                }
                _ => {
                    warn!("ref bits fall into slow path: {:b}", ref_map);

                    let mut i = 0;
                    while i < objectmodel::REF_MAP_LENGTH {
                        let has_ref: bool = ((ref_map >> i) & 1) == 1;

                        if has_ref {
                            steal_process_edge(
                                addr,
                                i * POINTER_SIZE,
                                local_queue,
                                job_sender,
                                mark_state,
                                immix_space,
                                lo_space
                            );
                        }

                        i += 1;
                    }
                }
            }
        } else {
            // by type ID
            let gctype_id = objectmodel::header_get_gctype_id(hdr);

            let gc_lock = MY_GC.read().unwrap();
            let gctype: Arc<GCType> =
                gc_lock.as_ref().unwrap().gc_types[gctype_id as usize].clone();

            for offset in gctype.gen_ref_offsets() {
                steal_process_edge(
                    addr,
                    offset,
                    local_queue,
                    job_sender,
                    mark_state,
                    immix_space,
                    lo_space
                );
            }
        }
    } else {
        // hybrids
        let gctype_id = objectmodel::header_get_gctype_id(hdr);
        let var_length = objectmodel::header_get_hybrid_length(hdr);

        let gc_lock = MY_GC.read().unwrap();
        let gctype: Arc<GCType> = gc_lock.as_ref().unwrap().gc_types[gctype_id as usize].clone();

        for offset in gctype.gen_hybrid_ref_offsets(var_length) {
            steal_process_edge(
                addr,
                offset,
                local_queue,
                job_sender,
                mark_state,
                immix_space,
                lo_space
            );
        }
    }
}

#[inline(always)]
#[cfg(not(feature = "use-sidemap"))]
pub fn steal_process_edge(
    base: Address,
    offset: usize,
    local_queue: &mut Vec<ObjectReference>,
    job_sender: &mpsc::Sender<ObjectReference>,
    mark_state: u8,
    immix_space: &ImmixSpace,
    lo_space: &FreeListSpace
) {
    let field_addr = base + offset;
    let edge = unsafe { field_addr.load::<ObjectReference>() };

    if cfg!(debug_assertions) {
        use std::process;
        // check if this object in within the heap, if it is an object
        if !edge.to_address().is_zero() && !immix_space.is_valid_object(edge.to_address()) &&
            !lo_space.is_valid_object(edge.to_address())
        {
            println!("trying to follow an edge that is not a valid object");
            println!("edge address: 0x{:x} from 0x{:x}", edge, field_addr);
            println!("base address: 0x{:x}", base);
            println!("---");
            if immix_space.addr_in_space(base) {
                objectmodel::print_object(base);
                objectmodel::print_object(edge.to_address());
                println!("---");
                println!("immix space:{}", immix_space);
            } else if lo_space.addr_in_space(base) {
                objectmodel::print_object(base);
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
        if !objectmodel::is_traced(edge, mark_state) {
            if local_queue.len() >= PUSH_BACK_THRESHOLD {
                job_sender.send(edge).unwrap();
            } else {
                local_queue.push(edge);
            }
        }
    }
}
