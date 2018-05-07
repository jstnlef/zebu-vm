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

use ast::ir::*;
use ast::ptr::*;
use ast::types::*;
use compiler::backend::BackendType;
use compiler::backend::RegGroup;
use runtime::ValueLocation;
use runtime::thread::MuThread;
use utils::*;
use utils::ByteSize;
use utils::ObjectReference;
use utils::math;
use vm::VM;

/// we do not allocate hybrid into tiny object space (hybrid should be at least 32 bytes)
pub fn check_hybrid_size(size: ByteSize) -> ByteSize {
    if size < 32 {
        32
    } else {
        math::align_up(size, POINTER_SIZE)
    }
}

pub fn gen_object_encode(backend_ty: &BackendType, size: ByteSize, vm: &VM) -> ObjectEncode {
    let is_hybrid = backend_ty.ty.is_hybrid();
    let gc_tyid = {
        let ty_encode = backend_ty.gc_type.as_ref().unwrap();
        vm.get_gc_type_id(ty_encode)
    };
    let full_tyid = {
        match backend_ty.gc_type_hybrid_full {
            Some(ref enc) => vm.get_gc_type_id(enc),
            None => 0
        }
    };

    debug!("ENCODE: gen_object_encode: {:?}, size: {}", backend_ty, size);
    debug!("ENCODE: gc_ty: {}, full_gc_ty: {}", gc_tyid, full_tyid);

    gen_object_encode_internal(is_hybrid, gc_tyid, full_tyid, size, vm)
}

pub fn gen_object_encode_internal(
    is_hybrid: bool,
    gc_tyid: TypeID,
    full_tyid: TypeID,
    size: ByteSize,
    vm: &VM
) -> ObjectEncode {
    let size = math::align_up(size, POINTER_SIZE);
    if size <= MAX_TINY_OBJECT {
        if !is_hybrid {
            let size = if size <= 16 {
                16
            } else {
                assert!(size <= 24);
                24
            };
            let gc_type = vm.get_gc_type(gc_tyid);
            let enc = gc_type.as_short();
            let field0 = if enc.fix_len() > 0 {
                enc.fix_ty(0)
            } else {
                WordType::NonRef
            };
            let field1 = if enc.fix_len() > 1 {
                enc.fix_ty(1)
            } else {
                WordType::NonRef
            };
            let field2 = if enc.fix_len() > 2 {
                enc.fix_ty(2)
            } else {
                WordType::NonRef
            };
            ObjectEncode::Tiny(TinyObjectEncode::create(size, field0, field1, field2))
        } else {
            unreachable!()
        }
    } else if size <= MAX_SMALL_OBJECT {
        ObjectEncode::Small(SmallObjectEncode::create(size, gc_tyid))
    } else if size <= MAX_MEDIUM_OBJECT {
        ObjectEncode::Medium(MediumObjectEncode::create(size, gc_tyid))
    } else {
        if !is_hybrid {
            ObjectEncode::Large(LargeObjectEncode::new(size, gc_tyid))
        } else {
            ObjectEncode::Large(LargeObjectEncode::new(size, full_tyid))
        }
    }
}

#[no_mangle]
pub extern "C" fn muentry_alloc_var_size(
    fix_size: ByteSize,
    var_size: ByteSize,
    var_len: usize,
    align: ByteSize,
    tyid: TypeID,
    full_tyid: TypeID
) -> ObjectReference {
    debug_assert!(MuThread::has_current());
    let cur_thread = MuThread::current_mut();
    let mutator: *mut Mutator = &mut cur_thread.allocator as *mut Mutator;
    let size = check_hybrid_size(fix_size + var_size * var_len);

    // alloc
    let res = if size <= MAX_TINY_OBJECT {
        muentry_alloc_tiny(mutator, size, align)
    } else if size <= MAX_MEDIUM_OBJECT {
        muentry_alloc_normal(mutator, size, align)
    } else {
        muentry_alloc_large(mutator, size, align)
    };

    // get encoding
    let ref vm = cur_thread.vm;
    let encode = gen_object_encode_internal(true, tyid, full_tyid, size, vm);

    match encode {
        ObjectEncode::Tiny(_) => unreachable!(),
        ObjectEncode::Small(enc) => muentry_init_small_object(mutator, res, enc),
        ObjectEncode::Medium(enc) => muentry_init_medium_object(mutator, res, enc),
        ObjectEncode::Large(enc) => muentry_init_large_object(mutator, res, enc)
    }

    res
}

// the following functions are used by VM to allocate for the client (through API)

/// finds an allocator and allocates memory
/// if current thread has an allocator, use the allocator. Otherwise creates a new allocator,
/// allocates objects and drops the allocator
fn check_allocator(size: ByteSize, align: ByteSize, encode: ObjectEncode) -> ObjectReference {
    if MuThread::has_current() {
        // we have an allocator
        let allocator = (&mut MuThread::current_mut().allocator) as *mut Mutator;
        allocate(allocator, size, align, encode)
    } else {
        let allocator = new_mutator_ptr();
        let ret = allocate(allocator, size, align, encode);
        drop_mutator(allocator);

        ret
    }
}

/// allocates and initiates an object (hybrid or other types, large or small)
#[inline(always)]
fn allocate(allocator: *mut Mutator, size: ByteSize, align: ByteSize, encode: ObjectEncode) -> ObjectReference {
    let size = math::align_up(size, POINTER_SIZE);
    // allocate
    if size <= MAX_TINY_OBJECT {
        let res = muentry_alloc_tiny(allocator, size, align);
        muentry_init_tiny_object(allocator, res, encode.tiny());
        res
    } else if size <= MAX_SMALL_OBJECT {
        let res = muentry_alloc_normal(allocator, size, align);
        muentry_init_small_object(allocator, res, encode.small());
        res
    } else if size <= MAX_MEDIUM_OBJECT {
        let res = muentry_alloc_normal(allocator, size, align);
        muentry_init_medium_object(allocator, res, encode.medium());
        res
    } else {
        let res = muentry_alloc_large(allocator, size, align);
        muentry_init_large_object(allocator, res, encode.large());
        res
    }
}

/// allocates an object of fixed types
pub fn allocate_fixed(ty: P<MuType>, backendtype: Box<BackendType>, vm: &VM) -> Address {
    let encode = gen_object_encode(&backendtype, backendtype.size, vm);

    trace!("API: allocate fixed ty: {}", ty);
    check_allocator(backendtype.size, backendtype.alignment, encode).to_address()
}

/// allocates an object of hybrid types
pub fn allocate_hybrid(ty: P<MuType>, len: usize, backendtype: Box<BackendType>, vm: &VM) -> Address {
    let size = check_hybrid_size(backendtype.size + backendtype.elem_size.unwrap() * len);
    let encode = gen_object_encode(&backendtype, size, vm);

    trace!("API: allocate hybrd ty: {}", ty);
    check_allocator(size, backendtype.alignment, encode).to_address()
}

/// allocates a global cell
pub fn allocate_global(iref_global: P<Value>, backendtype: Box<BackendType>, vm: &VM) -> ValueLocation {
    let referenced_type = match iref_global.ty.get_referent_ty() {
        Some(ty) => ty,
        None => panic!("expected global to be an iref type, found {}", iref_global.ty)
    };

    assert!(!referenced_type.is_hybrid(), "global cell cannot be hybrid type");

    let addr = allocate_fixed(referenced_type, backendtype, vm);
    ValueLocation::Direct(RegGroup::GPR, addr)
}
