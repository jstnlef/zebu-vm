extern crate gc;

pub use self::gc::*;

use utils::ByteSize;
use utils::ObjectReference;
use ast::ir::*;
use ast::ptr::*;
use compiler::backend::RegGroup;
use vm::VM;
use runtime::ValueLocation;
use runtime::thread::MuThread;

fn allocate(size: ByteSize, align: ByteSize, encode: u64) -> ObjectReference {
    let allocator = (&mut MuThread::current_mut().allocator) as *mut Mutator;

    let ret = if size > LARGE_OBJECT_THRESHOLD {
        muentry_alloc_large(allocator, size, align)
    } else {
        alloc(allocator, size, align)
    };

    muentry_init_object(allocator, ret, encode);

    ret
}

pub fn allocate_global(value: P<Value>, vm: &VM) -> ValueLocation {
    let tyid = value.ty.id();

    let referenced_type = match value.ty.get_referenced_ty() {
        Some(ty) => ty,
        None => panic!("expected global to be an iref type, found {}", value.ty)
    };

    let backendtype = vm.get_backend_type_info(referenced_type.id());
    let gctype = backendtype.gc_type.clone();
    let gctype_id = gctype.id;
    let encode = get_gc_type_encode(gctype_id);

    trace!("allocating global as gctype {:?}", gctype);
    let addr = allocate(backendtype.size, backendtype.alignment, encode).to_address();

    ValueLocation::Direct(RegGroup::GPR, addr)
}
