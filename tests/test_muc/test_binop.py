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

def test_div():
    lib = load_bundle(
        """
        .funcdef div<(int<64> int<64>) -> (int<64>)>
        {
            entry(<int<64>> x <int<64>> y):
                res = UDIV <int<64>> x y
                RET res
        }
        """, "div"
    )

    div = get_function(lib.div, [ctypes.c_uint64], ctypes.c_uint64)
    assert(div(6, 2) == 3)

def test_div2():
    lib = load_bundle(
        """
        .funcdef div2<(int<64>) -> (int<64>)>
        {
            entry(<int<64>> x):
                res = UDIV <int<64>> x <int<64>> 2
                RET res
        }
        """, "div2"
    )

    div2 = get_function(lib.div2, [ctypes.c_uint64], ctypes.c_uint64)
    assert(div2(6) == 3)

def test_sdiv():
    lib = load_bundle(
        """
        .funcdef sdiv<(int<64> int<64>) -> (int<64>)>
        {
            entry(<int<64>> x <int<64>> y):
                res = SDIV <int<64>> x y
                RET res
        }
        """, "sdiv"
    )

    sdiv = get_function(lib.sdiv, [ctypes.c_int64], ctypes.c_int64)
    assert(sdiv(ctypes.c_int64(6), ctypes.c_int64(2)) == 3)
    assert(sdiv(ctypes.c_int64(-6), ctypes.c_int64(2)) == -3)
    assert(sdiv(ctypes.c_int64(6), ctypes.c_int64(-2)) == -3)
    assert(sdiv(ctypes.c_int64(-6), ctypes.c_int64(-2)) == 3)

def test_sdiv2():
    lib = load_bundle(
        """
        .funcdef sdiv2<(int<64>) -> (int<64>)>
        {
            entry(<int<64>> x):
                res = SDIV <int<64>> x <int<64>> 2
                RET res
        }
        """, "sdiv2"
    )

    sdiv2 = get_function(lib.sdiv2, [ctypes.c_int64], ctypes.c_int64)
    assert(sdiv2(ctypes.c_int64(6)) == 3)
    assert(sdiv2(ctypes.c_int64(-6)) == -3)

def test_mul():
    lib = load_bundle(
        """
        .funcdef mul<(int<64> int<64>) -> (int<64>)>
        {
            entry(<int<64>> x <int<64>> y):
                res = MUL <int<64>> x y
                RET res
        }
        """, "mul"
    )

    mul = get_function(lib.mul, [ctypes.c_int64, ctypes.c_int64], ctypes.c_int64)
    assert(mul(3, 2) == 6)
    assert(mul(-3, 2) == -6)
    assert(mul(-3, -2) == 6)

def test_mul2():
    lib = load_bundle(
        """
        .funcdef mul2<(int<64>) -> (int<64>)>
        {
            entry(<int<64>> x):
                res = MUL <int<64>> x <int<64>> 2
                RET res
        }
        """, "mul2"
    )

    mul2 = get_function(lib.mul2, [ctypes.c_int64], ctypes.c_int64)
    assert(mul2(3) == 6)
    assert(mul2(-3) == -6)