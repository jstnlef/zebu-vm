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

use mu::vm::*;
use mu::vm::handle::*;
use mu::ast::types::*;
use mu::utils::Address;

use std::f64;
use std::mem::transmute;

/**
 * Helper functions to test VM:: methods with literal values
 *
 * Unfortunately these need to be called as e.g. &tr64(...)
 * due to the lifetime checker.
 */

fn tr64(val: u64) -> APIHandle {
    APIHandle {
        id: 0, // arbitrary
        v : APIHandleValue::TagRef64(val)
    }
}

fn double(val: f64) -> APIHandle {
    APIHandle {
        id: 0, // arbitrary
        v : APIHandleValue::Double(val)
    }
}

fn tag(val: u64) -> APIHandle {
    APIHandle {
        id: 0, // arbitrary
        v : APIHandleValue::Int(val, 6)
    }
}

fn int52(val: u64) -> APIHandle {
    APIHandle {
        id: 0, // arbitrary
        v : APIHandleValue::Int(val, 52)
    }
}

fn ref_void(val: u64) -> APIHandle {
    APIHandle {
        id: 0, // arbitrary
        v : APIHandleValue::Ref(REF_VOID_TYPE.clone(),
            unsafe { Address::from_usize(val as usize) })
    }
}

/**
 * These tests are translated from those in the reference implementation.
 * See: https://gitlab.anu.edu.au/mu/mu-impl-ref2/blob/master/src/test/scala/uvm/refimpl/itpr/UvmTagRef64OperationSpec.scala
 *
 * TODO: These test specific values. It would be nice if we had something
 * along the lines of QuickCheck properties, e.g. tr64ToInt . intToTr64 == id,
 * and ditto for floats except for NANs.
 */

#[test]
fn test_nan_with_suffix_1_is_integer() {
    let vm = VM::new();

    assert!(vm.handle_tr64_is_int(&tr64(0x7ff0000000000001u64)));
    assert!(vm.handle_tr64_is_int(&tr64(0xfff0000000000001u64)));
    assert!(vm.handle_tr64_is_int(&tr64(0xffffffffffffffffu64)));
}

#[test]
fn test_nan_with_suffix_10_is_ref() {
    let vm = VM::new();

    assert!(vm.handle_tr64_is_ref(&tr64(0x7ff0000000000002u64)));
    assert!(vm.handle_tr64_is_ref(&tr64(0xfff0000000000002u64)));
    assert!(vm.handle_tr64_is_ref(&tr64(0xfffffffffffffffeu64)));
}

#[test]
fn test_other_bit_pattern_is_double() {
    let vm = VM::new();

    assert!(vm.handle_tr64_is_fp(&tr64(0x0u64)));
    assert!(vm.handle_tr64_is_fp(&tr64(0x123456789abcdef0u64)));
    assert!(vm.handle_tr64_is_fp(&tr64(0x7ff123456789abccu64)));
    assert!(vm.handle_tr64_is_fp(&tr64(0xfffffffffffffffcu64)));
    unsafe { assert!(vm.handle_tr64_is_fp(&tr64(transmute(3.1415927f64)))); }
}

#[test]
fn test_encode_int() {
    let vm = VM::new();
    
    assert_eq!(vm.handle_tr64_from_int(&int52(0x0000000000000u64)).v.as_tr64(), 0x7ff0000000000001u64);
    assert_eq!(vm.handle_tr64_from_int(&int52(0xfffffffffffffu64)).v.as_tr64(), 0xffffffffffffffffu64);
    assert_eq!(vm.handle_tr64_from_int(&int52(0x5555555555555u64)).v.as_tr64(), 0x7ffaaaaaaaaaaaabu64);
    assert_eq!(vm.handle_tr64_from_int(&int52(0xaaaaaaaaaaaaau64)).v.as_tr64(), 0xfff5555555555555u64);
}

#[test]
fn test_encode_double() {
    let vm = VM::new();
    
    unsafe {
        assert_eq!(vm.handle_tr64_from_fp(&double(3.14_f64)).v.as_tr64(), transmute(3.14_f64));
        assert_eq!(vm.handle_tr64_from_fp(&double(-3.14_f64)).v.as_tr64(), transmute(-3.14_f64));
        assert_eq!(vm.handle_tr64_from_fp(&double(f64::INFINITY)).v.as_tr64(), 0x7ff0000000000000u64);
        assert_eq!(vm.handle_tr64_from_fp(&double(transmute(0x7ff123456789abcdu64))).v.as_tr64(), 0x7ff0000000000008u64);
        assert!(transmute::<u64, f64>((vm.handle_tr64_from_fp(&double(transmute(0x7ff123456789abcdu64))).v.as_tr64())).is_nan());
    }
}

#[test]
fn test_encode_tagref() {
    let vm = VM::new();
    
    assert_eq!(vm.handle_tr64_from_ref(&ref_void(0x000000000000u64), &tag(0x00u64)).v.as_tr64(), 0x7ff0000000000002u64);
    assert_eq!(vm.handle_tr64_from_ref(&ref_void(0x7ffffffffff8u64), &tag(0x00u64)).v.as_tr64(), 0x7ff07ffffffffffau64);
    assert_eq!(vm.handle_tr64_from_ref(&ref_void(0xfffffffffffffff8u64), &tag(0x00u64)).v.as_tr64(), 0xfff07ffffffffffau64);
    assert_eq!(vm.handle_tr64_from_ref(&ref_void(0x000000000000u64), &tag(0x3fu64)).v.as_tr64(), 0x7fff800000000006u64);
}

#[test]
fn test_decode_integer() {
    let vm = VM::new();
    
    assert_eq!(vm.handle_tr64_to_int(&tr64(0x7ff0000000000001u64)).v.as_int(), 0u64);
    assert_eq!(vm.handle_tr64_to_int(&tr64(0xfff0000000000001u64)).v.as_int(), 0x8000000000000u64);
    assert_eq!(vm.handle_tr64_to_int(&tr64(0xfff5555555555555u64)).v.as_int(), 0xaaaaaaaaaaaaau64);
    assert_eq!(vm.handle_tr64_to_int(&tr64(0x7ffaaaaaaaaaaaabu64)).v.as_int(), 0x5555555555555u64);
}

#[test]
fn test_decode_double() {
    let vm = VM::new();
    
    assert_eq!(vm.handle_tr64_to_fp(&tr64(0x0000000000000000u64)).v.as_double(),  0.0_f64);
    assert_eq!(vm.handle_tr64_to_fp(&tr64(0x8000000000000000u64)).v.as_double(), -0.0_f64);
    assert_eq!(vm.handle_tr64_to_fp(&tr64(0x3ff0000000000000u64)).v.as_double(),  1.0_f64);
    assert!(vm.handle_tr64_to_fp(&tr64(0x7ff0000000000008)).v.as_double().is_nan());
}

#[test]
fn test_decode_tagref() {
    let vm = VM::new();

    assert_eq!(vm.handle_tr64_to_ref(&tr64(0x7ff0555555555552u64)).v.as_ref().1.as_usize() as u64, 0x555555555550u64);
    assert_eq!(vm.handle_tr64_to_ref(&tr64(0xfff02aaaaaaaaaaau64)).v.as_ref().1.as_usize() as u64, 0xffffaaaaaaaaaaa8u64);
    assert_eq!(vm.handle_tr64_to_tag(&tr64(0x7ff0555555555552u64)).v.as_int(), 0u64);
    assert_eq!(vm.handle_tr64_to_tag(&tr64(0x7fff800000000006u64)).v.as_int(), 0x3fu64);
    assert_eq!(vm.handle_tr64_to_tag(&tr64(0x7ffa800000000002u64)).v.as_int(), 0x2au64);
    assert_eq!(vm.handle_tr64_to_tag(&tr64(0x7ff5000000000006u64)).v.as_int(), 0x15u64);
}
