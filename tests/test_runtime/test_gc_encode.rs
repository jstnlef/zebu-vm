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

use mu::ast::ir::*;
use mu::ast::types::*;
use mu::ast::ptr::*;
use mu::vm::*;
use mu::compiler::backend::*;
use mu::runtime::mm::*;

#[test]
fn test_int() {
    let vm = VM::new();

    // int<1>
    {
        typedef!((vm) int1 = mu_int(1));
        let backend_ty = vm.get_backend_type_info(int1.id());
        assert_eq!(backend_ty.size, 1);
        assert_eq!(backend_ty.alignment, 1);
        assert!(backend_ty.gc_type.is_some());
        let gc_type = backend_ty.gc_type.as_ref().unwrap().as_short();
        assert_eq!(gc_type.align(), MINIMAL_ALIGNMENT);
        assert_eq!(gc_type.fix_len(), 1);
        assert_eq!(gc_type.fix_ty(0), WordType::NonRef);
        assert_eq!(gc_type.var_len(), 0);
    }

    // int<64>
    {
        typedef!((vm) int64 = mu_int(64));
        let backend_ty = vm.get_backend_type_info(int64.id());
        assert_eq!(backend_ty.size, 8);
        assert_eq!(backend_ty.alignment, 8);
        assert!(backend_ty.gc_type.is_some());
        let gc_type = backend_ty.gc_type.as_ref().unwrap().as_short();
        assert_eq!(gc_type.align(), MINIMAL_ALIGNMENT);
        assert_eq!(gc_type.fix_len(), 1);
        assert_eq!(gc_type.fix_ty(0), WordType::NonRef);
        assert_eq!(gc_type.var_len(), 0);
    }

    // int<128>
    {
        typedef!((vm) int128 = mu_int(128));
        let backend_ty = vm.get_backend_type_info(int128.id());
        assert_eq!(backend_ty.size, 16);
        assert_eq!(backend_ty.alignment, 16);
        assert!(backend_ty.gc_type.is_some());
        let gc_type = backend_ty.gc_type.as_ref().unwrap().as_short();
        assert_eq!(gc_type.align(), 16);
        assert_eq!(gc_type.fix_len(), 2);
        assert_eq!(gc_type.fix_ty(0), WordType::NonRef);
        assert_eq!(gc_type.fix_ty(1), WordType::NonRef);
        assert_eq!(gc_type.var_len(), 0);
    }
}

#[test]
fn test_floatpoint() {
    let vm = VM::new();

    // float
    {
        typedef!((vm) float = mu_float);
        let backend_ty = vm.get_backend_type_info(float.id());
        assert_eq!(backend_ty.size, 4);
        assert_eq!(backend_ty.alignment, 4);
        assert!(backend_ty.gc_type.is_some());
        let gc_type = backend_ty.gc_type.as_ref().unwrap().as_short();
        assert_eq!(gc_type.align(), MINIMAL_ALIGNMENT);
        assert_eq!(gc_type.fix_len(), 1);
        assert_eq!(gc_type.fix_ty(0), WordType::NonRef);
        assert_eq!(gc_type.var_len(), 0);
    }

    // double
    {
        typedef!((vm) double = mu_double);
        let backend_ty = vm.get_backend_type_info(double.id());
        assert_eq!(backend_ty.size, 8);
        assert_eq!(backend_ty.alignment, 8);
        assert!(backend_ty.gc_type.is_some());
        let gc_type = backend_ty.gc_type.as_ref().unwrap().as_short();
        assert_eq!(gc_type.align(), MINIMAL_ALIGNMENT);
        assert_eq!(gc_type.fix_len(), 1);
        assert_eq!(gc_type.fix_ty(0), WordType::NonRef);
        assert_eq!(gc_type.var_len(), 0);
    }
}

#[test]
fn test_ref() {
    let vm = VM::new();

    typedef!((vm) int64 = mu_int(64));

    // ref<int64>
    {
        typedef!((vm) my_ref = mu_ref(int64));
        let backend_ty = vm.get_backend_type_info(my_ref.id());
        assert_eq!(backend_ty.size, 8);
        assert_eq!(backend_ty.alignment, 8);
        assert!(backend_ty.gc_type.is_some());
        let gc_type = backend_ty.gc_type.as_ref().unwrap().as_short();
        assert_eq!(gc_type.align(), MINIMAL_ALIGNMENT);
        assert_eq!(gc_type.fix_len(), 1);
        assert_eq!(gc_type.fix_ty(0), WordType::Ref);
        assert_eq!(gc_type.var_len(), 0);
    }

    // iref<int64>
    {
        typedef!((vm) my_iref = mu_iref(int64));
        let backend_ty = vm.get_backend_type_info(my_iref.id());
        assert_eq!(backend_ty.size, 8);
        assert_eq!(backend_ty.alignment, 8);
        assert!(backend_ty.gc_type.is_some());
        let gc_type = backend_ty.gc_type.as_ref().unwrap().as_short();
        assert_eq!(gc_type.align(), MINIMAL_ALIGNMENT);
        assert_eq!(gc_type.fix_len(), 1);
        assert_eq!(gc_type.fix_ty(0), WordType::Ref);
        assert_eq!(gc_type.var_len(), 0);
    }
}

#[test]
fn test_struct1() {
    let vm = VM::new();

    // struct<int<64> ref<int<64>> int<64> ref<int<64>>>
    typedef!((vm) int64 = mu_int(64));
    typedef!((vm) ref_int64 = mu_ref(int64));
    typedef!((vm) my_struct = mu_struct(int64, ref_int64, int64, ref_int64));

    let backend_ty = vm.get_backend_type_info(my_struct.id());
    assert_eq!(backend_ty.size, 32);
    assert_eq!(backend_ty.alignment, 8);
    assert!(backend_ty.gc_type.is_some());
    let gc_type = backend_ty.gc_type.as_ref().unwrap().as_short();
    assert_eq!(gc_type.align(), MINIMAL_ALIGNMENT);
    assert_eq!(gc_type.fix_len(), 4);
    assert_eq!(gc_type.fix_ty(0), WordType::NonRef);
    assert_eq!(gc_type.fix_ty(1), WordType::Ref);
    assert_eq!(gc_type.fix_ty(2), WordType::NonRef);
    assert_eq!(gc_type.fix_ty(3), WordType::Ref);
    assert_eq!(gc_type.var_len(), 0);
}

#[test]
// test naturally aligned int32
fn test_struct2() {
    let vm = VM::new();

    // struct<int<32> int<32> ref<int<64>> int<64> ref<int<64>>>
    typedef!((vm) int32 = mu_int(32));
    typedef!((vm) int64 = mu_int(64));
    typedef!((vm) ref_int64 = mu_ref(int64));
    typedef!((vm) my_struct = mu_struct(int32, int32, ref_int64, int64, ref_int64));

    let backend_ty = vm.get_backend_type_info(my_struct.id());
    assert_eq!(backend_ty.size, 32);
    assert_eq!(backend_ty.alignment, 8);
    assert!(backend_ty.gc_type.is_some());
    let gc_type = backend_ty.gc_type.as_ref().unwrap().as_short();
    assert_eq!(gc_type.align(), MINIMAL_ALIGNMENT);
    assert_eq!(gc_type.fix_len(), 4);
    assert_eq!(gc_type.fix_ty(0), WordType::NonRef);
    assert_eq!(gc_type.fix_ty(1), WordType::Ref);
    assert_eq!(gc_type.fix_ty(2), WordType::NonRef);
    assert_eq!(gc_type.fix_ty(3), WordType::Ref);
    assert_eq!(gc_type.var_len(), 0);
}

#[test]
// test alignment of int64 that follows int32
// test uncomplete word in the end
fn test_struct3() {
    let vm = VM::new();

    // struct<int<32> ref<int<64>> int<64> ref<int<64>> int<32>>
    typedef!((vm) int32 = mu_int(32));
    typedef!((vm) int64 = mu_int(64));
    typedef!((vm) ref_int64 = mu_ref(int64));
    typedef!((vm) my_struct = mu_struct(int32, ref_int64, int64, ref_int64, int32));

    let backend_ty = vm.get_backend_type_info(my_struct.id());
    assert_eq!(backend_ty.size, 40);
    assert_eq!(backend_ty.alignment, 8);
    assert!(backend_ty.gc_type.is_some());
    let gc_type = backend_ty.gc_type.as_ref().unwrap().as_short();
    assert_eq!(gc_type.align(), MINIMAL_ALIGNMENT);
    assert_eq!(gc_type.fix_len(), 5);
    assert_eq!(gc_type.fix_ty(0), WordType::NonRef);
    assert_eq!(gc_type.fix_ty(1), WordType::Ref);
    assert_eq!(gc_type.fix_ty(2), WordType::NonRef);
    assert_eq!(gc_type.fix_ty(3), WordType::Ref);
    assert_eq!(gc_type.fix_ty(4), WordType::NonRef);
    assert_eq!(gc_type.var_len(), 0);
}

#[test]
// test cyclic struct
fn test_struct4() {
    let vm = VM::new();

    // my_struct = struct<int<64> ref<my_struct>>
    typedef!((vm) int64 = mu_int(64));
    typedef!((vm) my_struct = mu_struct_placeholder());
    typedef!((vm) ref_my_struct = mu_ref(my_struct));
    typedef!((vm) mu_struct_put(my_struct, int64, ref_my_struct));

    let backend_ty = vm.get_backend_type_info(my_struct.id());
    assert_eq!(backend_ty.size, 16);
    assert_eq!(backend_ty.alignment, 8);
    assert!(backend_ty.gc_type.is_some());
    let gc_type = backend_ty.gc_type.as_ref().unwrap().as_short();
    assert_eq!(gc_type.align(), MINIMAL_ALIGNMENT);
    assert_eq!(gc_type.fix_len(), 2);
    assert_eq!(gc_type.fix_ty(0), WordType::NonRef);
    assert_eq!(gc_type.fix_ty(1), WordType::Ref);
    assert_eq!(gc_type.var_len(), 0);
}

#[test]
fn test_array1() {
    let vm = VM::new();
    typedef!((vm) int64 = mu_int(64));

    // array<int<64> 5>
    {
        typedef!((vm) array = mu_array(int64, 5));
        let backend_ty = vm.get_backend_type_info(array.id());
        assert_eq!(backend_ty.size, 40);
        assert_eq!(backend_ty.alignment, 8);
        assert!(backend_ty.gc_type.is_some());
        let gc_type = backend_ty.gc_type.as_ref().unwrap().as_short();
        assert_eq!(gc_type.align(), MINIMAL_ALIGNMENT);
        assert_eq!(gc_type.fix_len(), 5);
        assert_eq!(gc_type.fix_ty(0), WordType::NonRef);
        assert_eq!(gc_type.fix_ty(1), WordType::NonRef);
        assert_eq!(gc_type.fix_ty(2), WordType::NonRef);
        assert_eq!(gc_type.fix_ty(3), WordType::NonRef);
        assert_eq!(gc_type.fix_ty(4), WordType::NonRef);
        assert_eq!(gc_type.var_len(), 0);
    }

    // array<ref<int<64>> 5>
    {
        typedef!((vm) ref_int64 = mu_ref(int64));
        typedef!((vm) array = mu_array(ref_int64, 5));
        let backend_ty = vm.get_backend_type_info(array.id());
        assert_eq!(backend_ty.size, 40);
        assert_eq!(backend_ty.alignment, 8);
        assert!(backend_ty.gc_type.is_some());
        let gc_type = backend_ty.gc_type.as_ref().unwrap().as_short();
        assert_eq!(gc_type.align(), MINIMAL_ALIGNMENT);
        assert_eq!(gc_type.fix_len(), 5);
        assert_eq!(gc_type.fix_ty(0), WordType::Ref);
        assert_eq!(gc_type.fix_ty(1), WordType::Ref);
        assert_eq!(gc_type.fix_ty(2), WordType::Ref);
        assert_eq!(gc_type.fix_ty(3), WordType::Ref);
        assert_eq!(gc_type.fix_ty(4), WordType::Ref);
        assert_eq!(gc_type.var_len(), 0);
    }
}

#[test]
// array with unaligned element
fn test_array2() {
    let vm = VM::new();

    // array<struct<ref<int<32>> int<32>> 5>
    typedef!((vm) int32 = mu_int(32));
    typedef!((vm) ref_int32 = mu_ref(int32));
    typedef!((vm) my_struct = mu_struct(ref_int32, int32));
    typedef!((vm) array = mu_array(my_struct, 5));
    let backend_ty = vm.get_backend_type_info(array.id());
    assert_eq!(backend_ty.size, 80);
    assert_eq!(backend_ty.alignment, 8);
    let gc_type = backend_ty.gc_type.as_ref().unwrap().as_short();
    assert_eq!(gc_type.align(), MINIMAL_ALIGNMENT);
    assert_eq!(gc_type.fix_len(), 10);
    assert_eq!(gc_type.fix_ty(0), WordType::Ref);
    assert_eq!(gc_type.fix_ty(1), WordType::NonRef);
    assert_eq!(gc_type.fix_ty(2), WordType::Ref);
    assert_eq!(gc_type.fix_ty(3), WordType::NonRef);
    assert_eq!(gc_type.fix_ty(4), WordType::Ref);
    assert_eq!(gc_type.fix_ty(5), WordType::NonRef);
    assert_eq!(gc_type.fix_ty(6), WordType::Ref);
    assert_eq!(gc_type.fix_ty(7), WordType::NonRef);
    assert_eq!(gc_type.fix_ty(8), WordType::Ref);
    assert_eq!(gc_type.fix_ty(9), WordType::NonRef);
    assert_eq!(gc_type.var_len(), 0);
}

#[test]
fn test_hybrid() {
    let vm = VM::new();

    typedef!((vm) int32 = mu_int(32));
    typedef!((vm) ref_int32 = mu_ref(int32));
    typedef!((vm) my_struct = mu_struct(ref_int32, int32));
    typedef!((vm) hybrid = mu_hybrid(my_struct)(my_struct));

    let backend_ty = vm.get_backend_type_info(hybrid.id());
    assert_eq!(backend_ty.size, 16); // only fix part
    assert_eq!(backend_ty.alignment, 8);

    let gc_type = backend_ty.gc_type.as_ref().unwrap().as_short();
    assert_eq!(gc_type.align(), MINIMAL_ALIGNMENT);
    assert_eq!(gc_type.fix_len(), 2);
    assert_eq!(gc_type.fix_ty(0), WordType::Ref);
    assert_eq!(gc_type.fix_ty(1), WordType::NonRef);
    assert_eq!(gc_type.var_len(), 2);
    assert_eq!(gc_type.var_ty(0), WordType::Ref);
    assert_eq!(gc_type.var_ty(1), WordType::NonRef);

    let full_gc_type = backend_ty.gc_type_hybrid_full.as_ref().unwrap().as_full();
    assert_eq!(full_gc_type.align, MINIMAL_ALIGNMENT);
    assert_eq!(full_gc_type.fix.len(), 2);
    assert_eq!(full_gc_type.fix[0], WordType::Ref);
    assert_eq!(full_gc_type.fix[1], WordType::NonRef);
    assert_eq!(full_gc_type.var.len(), 2);
    assert_eq!(full_gc_type.var[0], WordType::Ref);
    assert_eq!(full_gc_type.var[1], WordType::NonRef);
}