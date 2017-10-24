# Copyright 2017 The Australian National University
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

from util import execute, compile_bundle, load_bundle, get_function;
import pytest;
import ctypes;

def test_gt_mem_r():
    lib = load_bundle(
        """
        .funcsig sig = (int<64> uptr<int<64>>) -> (int<64>)
        .funcdef test_gt_mem_r <sig>
        {
            entry(<int<64>> x <uptr<int<64>>> ptr):
                y = LOAD PTR <int<64>> ptr
                cond = SGT <int<64>> x y
                BRANCH2 cond ret1() ret0()
            ret1():
                RET <int<64>> 1
            ret0():
                RET <int<64>> 0
        }
        """, "test_gt_mem_r"
    )

    zero = ctypes.c_int64(0);
    ptr = ctypes.addressof(zero);
    cmp_gt_zero = get_function(lib.test_gt_mem_r, [ctypes.c_int64, ctypes.c_int64], ctypes.c_int64)
    assert(cmp_gt_zero(ctypes.c_int64(1), ptr) == 1)
    assert(cmp_gt_zero(ctypes.c_int64(0), ptr) == 0)
    assert(cmp_gt_zero(ctypes.c_int64(-1), ptr) == 0)

def test_gt_val_mem_r():
    lib = load_bundle(
        """
        .funcsig sig = (int<64> uptr<int<64>>) -> (int<8>)
        .funcdef test_gt_val_mem_r <sig>
        {
            entry(<int<64>> x <uptr<int<64>>> ptr):
                y = LOAD PTR <int<64>> ptr
                cond = SGT <int<64>> x y
                res = ZEXT <int<1> int<8>> cond
                RET res
        }
        """, "test_gt_val_mem_r"
    )

    zero = ctypes.c_int64(0);
    ptr = ctypes.addressof(zero);
    cmp_gt_zero = get_function(lib.test_gt_val_mem_r, [ctypes.c_int64, ctypes.c_voidp], ctypes.c_int8)
    assert(cmp_gt_zero(ctypes.c_int64(1), ptr) == 1)
    assert(cmp_gt_zero(ctypes.c_int64(0), ptr) == 0)
    assert(cmp_gt_zero(ctypes.c_int64(-1), ptr) == 0)

def test_gt_r_mem():
    lib = load_bundle(
        """
        .funcsig sig = (uptr<int<64>> int<64>) -> (int<64>)
        .funcdef test_gt_r_mem <sig>
        {
            entry(<uptr<int<64>>> ptr <int<64>> y):
                x = LOAD PTR <int<64>> ptr
                cond = SGT <int<64>> x y
                BRANCH2 cond ret1() ret0()
            ret1():
                RET <int<64>> 1
            ret0():
                RET <int<64>> 0
        }
        """, "test_gt_r_mem"
    )

    zero = ctypes.c_int64(0);
    ptr = ctypes.addressof(zero);
    cmp_gt_zero = get_function(lib.test_gt_r_mem, [ctypes.c_int64, ctypes.c_int64], ctypes.c_int64)
    assert(cmp_gt_zero(ptr, ctypes.c_int64(1)) == 0)
    assert(cmp_gt_zero(ptr, ctypes.c_int64(0)) == 0)
    assert(cmp_gt_zero(ptr, ctypes.c_int64(-1)) == 1)

def test_gt_mem_f():
    lib = load_bundle(
        """
        .funcsig sig = (double uptr<double>) -> (int<64>)
        .funcdef test_gt_mem_f <sig>
        {
            entry(<double> x <uptr<double>> ptr):
                y = LOAD PTR <double> ptr
                cond = FOGT <double> x y
                BRANCH2 cond ret1() ret0()
            ret1():
                RET <int<64>> 1
            ret0():
                RET <int<64>> 0
        }
        """, "test_gt_mem_f"
    )

    zero = ctypes.c_double(0);
    ptr = ctypes.addressof(zero);
    cmp_gt_zero = get_function(lib.test_gt_mem_f, [ctypes.c_double, ctypes.c_voidp], ctypes.c_int64)
    assert(cmp_gt_zero(ctypes.c_double(1), ptr) == 1)
    assert(cmp_gt_zero(ctypes.c_double(0), ptr) == 0)
    assert(cmp_gt_zero(ctypes.c_double(-1), ptr) == 0)

def test_gt_f_mem():
    lib = load_bundle(
        """
        .funcsig sig = (uptr<double> double) -> (int<64>)
        .funcdef test_gt_f_mem <sig>
        {
            entry(<uptr<double>> ptr <double> y):
                x = LOAD PTR <double> ptr
                cond = FOGT <double> x y
                BRANCH2 cond ret1() ret0()
            ret1():
                RET <int<64>> 1
            ret0():
                RET <int<64>> 0
        }
        """, "test_gt_f_mem"
    )

    zero = ctypes.c_double(0);
    ptr = ctypes.addressof(zero);
    cmp_gt_zero = get_function(lib.test_gt_f_mem, [ctypes.c_voidp, ctypes.c_double], ctypes.c_int64)
    assert(cmp_gt_zero(ptr, ctypes.c_double(1)) == 0)
    assert(cmp_gt_zero(ptr, ctypes.c_double(0)) == 0)
    assert(cmp_gt_zero(ptr, ctypes.c_double(-1)) == 1)

def test_eq_f_zero():
    lib = load_bundle(
        """
        .funcsig sig = (double) -> (int<8>)
        .funcdef test_eq_f_zero <sig>
        {
            entry(<double> x):
                cond = FOEQ <double> x <double> 0.00 d
                res = ZEXT <int<1> int<8>> cond
                RET res
        }
        """, "test_eq_f_zero"
    )

    eq_zero = get_function(lib.test_eq_f_zero, [ctypes.c_double], ctypes.c_int8)
    assert(eq_zero(ctypes.c_double(0)) == 1)
    assert(eq_zero(ctypes.c_double(1)) == 0)
    assert(eq_zero(ctypes.c_double(-1)) == 0)

def test_cmp_pattern1():
    lib = load_bundle(
        """
        .funcdef test_cmp_pattern1 <(int<64> int<64>) -> (int<64>)>
        {
            entry(<int<64>> x <int<64>> y):
                cond = EQ <int<64>> x y
                cond_ = ZEXT <int<1> int<8>> cond
                actual_cond = EQ <int<8>> cond_ <int<8>> 1
                sum = ADD <int<64>> x y
                BRANCH2 actual_cond ret_true(sum) ret_false(sum)
            
            ret_true(<int<64>> sum):
                RET <int<64>> 1
            
            ret_false(<int<64>> sum):
                RET <int<64>> 0
        }
        """, "test_cmp_pattern1"
    )

    eq = get_function(lib.test_cmp_pattern1, [ctypes.c_int64, ctypes.c_int64], ctypes.c_int64)
    assert(eq(1, 1) == 1)
    assert(eq(1, 0) == 0)

def test_cmp_pattern2():
    lib = load_bundle(
        """
        .funcdef test_cmp_pattern2 <(int<64> int<64>) -> (int<64>)>
        {
            entry(<int<64>> x <int<64>> y):
                cond = EQ <int<64>> x y
                sum = ADD <int<64>> x y
                BRANCH2 cond ret_true(sum) ret_false(sum)
            
            ret_true(<int<64>> sum):
                RET <int<64>> 1
            
            ret_false(<int<64>> sum):
                RET <int<64>> 0
        }
        """, "test_cmp_pattern2"
    )

    eq = get_function(lib.test_cmp_pattern2, [ctypes.c_int64, ctypes.c_int64], ctypes.c_int64)
    assert(eq(1, 1) == 1)
    assert(eq(1, 0) == 0)