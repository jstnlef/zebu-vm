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

use utils::*;
use utils::bit_utils;
use objectmodel;
use common::ptr::*;
use heap::immix::*;

use std::sync::atomic::AtomicUsize;
use std::sync::RwLock;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub mod immix;
pub mod freelist;
pub mod gc;

pub const IMMIX_SPACE_RATIO: f64 = 1.0 - LO_SPACE_RATIO;
pub const LO_SPACE_RATIO: f64 = 0.2;
pub const DEFAULT_HEAP_SIZE: usize = 500 << 20;

lazy_static! {
    pub static ref IMMIX_SPACE_SIZE : AtomicUsize =
        AtomicUsize::new( (DEFAULT_HEAP_SIZE as f64 * IMMIX_SPACE_RATIO) as usize );
    pub static ref LO_SPACE_SIZE : AtomicUsize =
        AtomicUsize::new( (DEFAULT_HEAP_SIZE as f64 * LO_SPACE_RATIO) as usize );
}

pub const SPACE_ALIGN: ByteSize = (1 << 19); // 512K
pub const SPACE_LOWBITS_MASK: usize = !(SPACE_ALIGN - 1);

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum SpaceDescriptor {
    ImmixTiny,
    ImmixNormal,
    Freelist
}

impl SpaceDescriptor {
    pub fn get(obj: ObjectReference) -> SpaceDescriptor {
        unsafe {
            obj.to_address()
                .mask(SPACE_LOWBITS_MASK)
                .load::<SpaceDescriptor>()
        }
    }
}

pub trait Space {
    #[inline(always)]
    fn start(&self) -> Address;
    #[inline(always)]
    fn end(&self) -> Address;
    #[inline(always)]
    fn is_valid_object(&self, addr: Address) -> bool;
    #[inline(always)]
    fn addr_in_space(&self, addr: Address) -> bool {
        addr >= self.start() && addr < self.end()
    }
}

#[allow(dead_code)]
pub const ALIGNMENT_VALUE: u8 = 1;

#[inline(always)]
#[allow(dead_code)]
pub fn fill_alignment_gap(start: Address, end: Address) -> () {
    debug_assert!(end >= start);
    unsafe {
        start.memset(ALIGNMENT_VALUE, end - start);
    }
}

const MAX_MUTATORS: usize = 1024;
lazy_static! {
    pub static ref MUTATORS : RwLock<Vec<Option<Arc<MutatorGlobal>>>> = {
        let mut ret = Vec::with_capacity(MAX_MUTATORS);
        for _ in 0..MAX_MUTATORS {
            ret.push(None);
        }
        RwLock::new(ret)
    };

    pub static ref N_MUTATORS : RwLock<usize> = RwLock::new(0);
}

#[repr(C)]
pub struct Mutator {
    id: usize,
    pub tiny: ImmixAllocator,
    pub normal: ImmixAllocator,
    global: Arc<MutatorGlobal>
}

impl Mutator {
    pub fn new(
        tiny: ImmixAllocator,
        normal: ImmixAllocator,
        global: Arc<MutatorGlobal>
    ) -> Mutator {
        let mut id_lock = N_MUTATORS.write().unwrap();
        {
            let mut mutators_lock = MUTATORS.write().unwrap();
            mutators_lock.remove(*id_lock);
            mutators_lock.insert(*id_lock, Some(global.clone()));
        }

        let ret = Mutator {
            id: *id_lock,
            tiny,
            normal,
            global
        };
        *id_lock += 1;

        ret
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn reset(&mut self) {
        self.tiny.reset();
        self.normal.reset();
    }

    pub fn reset_after_gc(&mut self) {
        self.reset()
    }

    pub fn prepare_for_gc(&mut self) {
        self.tiny.prepare_for_gc();
        self.normal.prepare_for_gc();
    }

    pub fn destroy(&mut self) {
        let mut mutator_count_lock = N_MUTATORS.write().unwrap();

        let mut mutators_lock = MUTATORS.write().unwrap();
        mutators_lock.push(None);
        mutators_lock.swap_remove(self.id);

        *mutator_count_lock = *mutator_count_lock - 1;

        if cfg!(debug_assertions) {
            debug!(
                "destroy mutator. Now live mutators = {}",
                *mutator_count_lock
            );
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
}

pub struct MutatorGlobal {
    take_yield: AtomicBool,
    still_blocked: AtomicBool
}

impl MutatorGlobal {
    pub fn new() -> MutatorGlobal {
        MutatorGlobal {
            take_yield: AtomicBool::new(false),
            still_blocked: AtomicBool::new(false)
        }
    }

    #[inline(always)]
    pub fn is_still_blocked(&self) -> bool {
        self.still_blocked.load(Ordering::SeqCst)
    }
    pub fn set_still_blocked(&self, b: bool) {
        self.still_blocked.store(b, Ordering::SeqCst);
    }

    pub fn set_take_yield(&self, b: bool) {
        self.take_yield.store(b, Ordering::SeqCst);
    }
    #[inline(always)]
    pub fn take_yield(&self) -> bool {
        self.take_yield.load(Ordering::SeqCst)
    }
}
