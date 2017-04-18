use mu::ast::types::*;
use mu::ast::ir::*;
use mu::ast::inst::*;
use mu::ast::op::*;
use mu::vm::*;

use utils::Address;
use mu::runtime::mm;
use mu::runtime::thread;
use mu::runtime::thread::MuThread;
use mu::vm::VM;

use std::sync::Arc;

#[test]
fn test_struct_layout() {
    let vm = Arc::new(VM::new());

    typedef!    ((vm) int8  = mu_int(8));
    typedef!    ((vm) int16 = mu_int(16));
    typedef!    ((vm) int32 = mu_int(32));
    typedef!    ((vm) int64 = mu_int(64));

    typedef!    ((vm) struct1 = mu_struct(int64, int8, int64));

    let struct1_backend_ty = vm.get_backend_type_info(struct1.id());
    assert_eq!(struct1_backend_ty.get_field_offset(0), 0);
    assert_eq!(struct1_backend_ty.get_field_offset(1), 8);
    assert_eq!(struct1_backend_ty.get_field_offset(2), 16);

    typedef!    ((vm) struct2 = mu_struct(int64, int8, int8, int16, int32, int64));
    let struct2_backend_ty = vm.get_backend_type_info(struct2.id());
    assert_eq!(struct2_backend_ty.get_field_offset(0), 0);
    assert_eq!(struct2_backend_ty.get_field_offset(1), 8);
    assert_eq!(struct2_backend_ty.get_field_offset(2), 9);
    assert_eq!(struct2_backend_ty.get_field_offset(3), 10);
    assert_eq!(struct2_backend_ty.get_field_offset(4), 12);
    assert_eq!(struct2_backend_ty.get_field_offset(5), 16);

    typedef!    ((vm) struct3 = mu_struct(int64, int8, int16, int32, int64));
    let struct3_backend_ty = vm.get_backend_type_info(struct3.id());
    assert_eq!(struct3_backend_ty.get_field_offset(0), 0);
    assert_eq!(struct3_backend_ty.get_field_offset(1), 8);
    assert_eq!(struct3_backend_ty.get_field_offset(2), 10);
    assert_eq!(struct3_backend_ty.get_field_offset(3), 12);
    assert_eq!(struct3_backend_ty.get_field_offset(4), 16);
}