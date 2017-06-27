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

fn ref_void(val: usize) -> APIHandle {
    APIHandle {
        id: 0, // arbitrary
        v : APIHandleValue::Ref(REF_VOID_TYPE.clone(),
            unsafe { Address::from_usize(val) })
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

// FIXME: Convert the rest of these into Rust tests, as above

/*
  it should "treat a NaN with suffix 10 as a double." in {
    OpHelper.tr64IsRef(0x7ff0000000000002L) shouldBe true
    OpHelper.tr64IsRef(0xfff0000000000002L) shouldBe true
    OpHelper.tr64IsRef(0xfffffffffffffffeL) shouldBe true
  }

  it should "treat other bit patterns as double" in {
    OpHelper.tr64IsFP(0x0L) shouldBe true
    OpHelper.tr64IsFP(0x123456789abcdef0L) shouldBe true
    OpHelper.tr64IsFP(0x7ff123456789abccL) shouldBe true
    OpHelper.tr64IsFP(0xfffffffffffffffcL) shouldBe true
    OpHelper.tr64IsFP(doubleToRawLongBits(3.1415927)) shouldBe true
  }

  it should "encode integers" in {
    OpHelper.intToTr64(0x0000000000000L) shouldBe 0x7ff0000000000001L
    OpHelper.intToTr64(0xfffffffffffffL) shouldBe 0xffffffffffffffffL
    OpHelper.intToTr64(0x5555555555555L) shouldBe 0x7ffaaaaaaaaaaaabL
    OpHelper.intToTr64(0xaaaaaaaaaaaaaL) shouldBe 0xfff5555555555555L
  }

  it should "encode double" in {
    OpHelper.fpToTr64(3.14) shouldBe java.lang.Double.doubleToRawLongBits(3.14)
    OpHelper.fpToTr64(-3.14) shouldBe java.lang.Double.doubleToRawLongBits(-3.14)
    OpHelper.fpToTr64(java.lang.Double.POSITIVE_INFINITY) shouldBe 0x7ff0000000000000L
    OpHelper.fpToTr64(longBitsToDouble(0x7ff123456789abcdL)) shouldBe 0x7ff0000000000008L
    isNaN(longBitsToDouble(OpHelper.fpToTr64(longBitsToDouble(0x7ff123456789abcdL)))) shouldBe true
  }

  it should "encode ref and tag" in {
    OpHelper.refToTr64(0x000000000000L, 0x00L) shouldBe 0x7ff0000000000002L
    OpHelper.refToTr64(0x7ffffffffff8L, 0x00L) shouldBe 0x7ff07ffffffffffaL
    OpHelper.refToTr64(0xfffffffffffffff8L, 0x00L) shouldBe 0xfff07ffffffffffaL
    OpHelper.refToTr64(0x000000000000L, 0x3fL) shouldBe 0x7fff800000000006L
  }

  it should "decode integer" in {
    OpHelper.tr64ToInt(0x7ff0000000000001L) shouldBe 0
    OpHelper.tr64ToInt(0xfff0000000000001L) shouldBe 0x8000000000000L
    OpHelper.tr64ToInt(0xfff5555555555555L) shouldBe 0xaaaaaaaaaaaaaL
    OpHelper.tr64ToInt(0x7ffaaaaaaaaaaaabL) shouldBe 0x5555555555555L
  }

  it should "decode double" in {
    OpHelper.tr64ToFP(0x0000000000000000L) shouldBe +0.0
    OpHelper.tr64ToFP(0x8000000000000000L) shouldBe -0.0
    OpHelper.tr64ToFP(0x3ff0000000000000L) shouldBe 1.0
    isNaN(OpHelper.tr64ToFP(0x7ff0000000000008L)) shouldBe true
  }
  
  it should "decodde ref and tag" in {
    OpHelper.tr64ToRef(0x7ff0555555555552L) shouldBe 0x555555555550L
    OpHelper.tr64ToRef(0xfff02aaaaaaaaaaaL) shouldBe 0xffffaaaaaaaaaaa8L
    OpHelper.tr64ToTag(0x7ff0555555555552L) shouldBe 0
    OpHelper.tr64ToTag(0x7fff800000000006L) shouldBe 0x3f
    OpHelper.tr64ToTag(0x7ffa800000000002L) shouldBe 0x2a
    OpHelper.tr64ToTag(0x7ff5000000000006L) shouldBe 0x15
  }
  */
