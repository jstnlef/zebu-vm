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

use mu::ast::types::*;
use mu::ast::ir::*;
use mu::vm::*;

use std::sync::Arc;

#[test]
fn test_array_layout() {
    let vm = Arc::new(VM::new());

    typedef!    ((vm) int8  = mu_int(8));
    typedef!    ((vm) int64 = mu_int(64));

    typedef!    ((vm) struct1 = mu_struct(int64, int8));

    typedef!    ((vm) array1  = mu_array(struct1, 5));

    let array1_backend_ty = vm.get_backend_type_info(array1.id());
    assert_eq!(array1_backend_ty.size, 16 * 5);
}

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
