extern crate gc;

pub use self::gc::*;

use utils::ByteSize;
use utils::ObjectReference;
use ast::ir::*;
use ast::ptr::*;
use ast::types::*;
use utils::Address;
use compiler::backend::RegGroup;
use vm::VM;
use runtime::ValueLocation;
use runtime::thread::MuThread;

fn allocate(size: ByteSize, align: ByteSize, encode: u64, hybrid_len: Option<u64>) -> ObjectReference {
    let allocator = (&mut MuThread::current_mut().allocator) as *mut Mutator;

    let ret = if size > LARGE_OBJECT_THRESHOLD {
        muentry_alloc_large(allocator, size, align)
    } else {
        alloc(allocator, size, align)
    };

    if hybrid_len.is_none() {
        muentry_init_object(allocator, ret, encode);
    } else {
        muentry_init_hybrid(allocator, ret, encode, hybrid_len.unwrap());
    }

    ret
}

pub fn allocate_fixed(ty: P<MuType>, vm: &VM) -> Address {
    let backendtype = vm.get_backend_type_info(ty.id());
    let gctype = backendtype.gc_type.clone();
    let encode = get_gc_type_encode(gctype.id);

    trace!("API: allocate fixed ty: {}", ty);
    trace!("API:          gc ty   : {:?}", gctype);
    trace!("API:          encode  : {:b}", encode);

    allocate(gctype.size(), gctype.alignment, encode, None).to_address()
}

pub fn allocate_hybrid(ty: P<MuType>, len: u64, vm: &VM) -> Address {
    let backendtype = vm.get_backend_type_info((ty.id()));
    let gctype = backendtype.gc_type.clone();
    let encode = get_gc_type_encode(gctype.id);

    trace!("API: allocate fixed ty: {}", ty);
    trace!("API:          gc ty   : {:?}", gctype);
    trace!("API:          encode  : {:b}", encode);

    allocate(gctype.size_hybrid(len as u32), gctype.alignment, encode, Some(len)).to_address()
}

pub fn allocate_global(iref_global: P<Value>, vm: &VM) -> ValueLocation {
    let referenced_type = match iref_global.ty.get_referenced_ty() {
        Some(ty) => ty,
        None => panic!("expected global to be an iref type, found {}", iref_global.ty)
    };

    let addr = allocate_fixed(referenced_type, vm);
    ValueLocation::Direct(RegGroup::GPR, addr)
}
