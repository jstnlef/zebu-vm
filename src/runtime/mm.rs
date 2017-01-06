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

fn allocate(size: ByteSize, align: ByteSize) -> ObjectReference {
    let allocator = (&mut MuThread::current_mut().allocator) as *mut Mutator;

    if size > LARGE_OBJECT_THRESHOLD {
        muentry_alloc_large(allocator, size, align)
    } else {
        alloc(allocator, size, align)
    }
}

pub fn allocate_value(value: P<Value>, vm: &VM) -> ValueLocation {
    let tyid = value.ty.id();
    let backend_ty = vm.get_backend_type_info(tyid);

    let addr = allocate(backend_ty.size, backend_ty.alignment).to_address();

    ValueLocation::Direct(RegGroup::GPR, addr)
}