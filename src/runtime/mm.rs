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

/// the garbage collection crate as in src/gc
/// we design the GC crate to be separate from other parts of the VM, and to be self-contained
/// as much as possible. We only expose limited interface (functions, data structures, constants)
/// from the GC crate, and those get re-exported in this module.
extern crate mu_gc as gc;
pub use self::gc::*;

use utils::ByteSize;
use utils::ObjectReference;
use ast::ir::*;
use ast::ptr::*;
use ast::types::*;
use utils::Address;
use compiler::backend::RegGroup;
use compiler::backend::BackendType;
use runtime::ValueLocation;
use runtime::thread::MuThread;

// the following functions are used by VM to allocate for the client (through API)

/// finds an allocator and allocates memory
/// if current thread has an allocator, use the allocator. Otherwise creates a new allocator,
/// allocates objects and drops the allocator
fn check_allocator(
    size: ByteSize,
    align: ByteSize,
    encode: u64,
    hybrid_len: Option<u64>
) -> ObjectReference {
    if MuThread::has_current() {
        // we have an allocator
        let allocator = (&mut MuThread::current_mut().allocator) as *mut Mutator;
        allocate(allocator, size, align, encode, hybrid_len)
    } else {
        let mut allocator = new_mutator();
        let ret = allocate(
            &mut allocator as *mut Mutator,
            size,
            align,
            encode,
            hybrid_len
        );
        drop_mutator(&mut allocator as *mut Mutator);

        ret
    }
}

/// allocates and initiates an object (hybrid or other types, large or small)
#[inline(always)]
fn allocate(
    allocator: *mut Mutator,
    size: ByteSize,
    align: ByteSize,
    encode: u64,
    hybrid_len: Option<u64>
) -> ObjectReference {
    // allocate
    let ret = if size > LARGE_OBJECT_THRESHOLD {
        muentry_alloc_large(allocator, size, align)
    } else {
        alloc(allocator, size, align)
    };

    // initiate
    if hybrid_len.is_none() {
        muentry_init_object(allocator, ret, encode);
    } else {
        muentry_init_hybrid(allocator, ret, encode, hybrid_len.unwrap());
    }

    ret
}

/// allocates an object of fixed types
pub fn allocate_fixed(ty: P<MuType>, backendtype: Box<BackendType>) -> Address {
    let gctype = backendtype.gc_type.clone();
    let encode = get_gc_type_encode(gctype.id);

    trace!("API: allocate fixed ty: {}", ty);
    trace!("API:          gc ty   : {:?}", gctype);
    trace!("API:          encode  : {:b}", encode);

    check_allocator(gctype.size(), gctype.alignment, encode, None).to_address()
}

/// allocates an object of hybrid types
pub fn allocate_hybrid(ty: P<MuType>, len: u64, backendtype: Box<BackendType>) -> Address {
    let gctype = backendtype.gc_type.clone();
    let encode = get_gc_type_encode(gctype.id);

    trace!("API: allocate hybrd ty: {}", ty);
    trace!("API:          gc ty   : {:?}", gctype);
    trace!("API:          encode  : {:b}", encode);

    check_allocator(
        gctype.size_hybrid(len as u32),
        gctype.alignment,
        encode,
        Some(len)
    ).to_address()
}

/// allocates a global cell
pub fn allocate_global(iref_global: P<Value>, backendtype: Box<BackendType>) -> ValueLocation {
    let referenced_type = match iref_global.ty.get_referent_ty() {
        Some(ty) => ty,
        None => {
            panic!(
                "expected global to be an iref type, found {}",
                iref_global.ty
            )
        }
    };

    assert!(
        !referenced_type.is_hybrid(),
        "global cell cannot be hybrid type"
    );

    let addr = allocate_fixed(referenced_type, backendtype);
    ValueLocation::Direct(RegGroup::GPR, addr)
}
