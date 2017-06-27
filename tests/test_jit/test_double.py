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

from util import fncptr_from_c_script, may_spawn_proc
import ctypes
import pytest

def within_err(res, exp, err=1e15):
    return abs(res - exp) < err


@may_spawn_proc
def test_double_add():
    fnp, _ = fncptr_from_c_script("test_double_add.c", "test_fnc", restype=ctypes.c_double)
    assert fnp() == 5.859873

@may_spawn_proc
def test_double_sub():
    fnp, _ = fncptr_from_c_script("test_double_sub.c", "test_fnc", restype=ctypes.c_double)
    assert within_err(fnp(), 0.423313)

@may_spawn_proc
def test_double_mul():
    fnp, _ = fncptr_from_c_script("test_double_mul.c", "test_fnc", restype=ctypes.c_double)
    assert fnp() == 8.53972942004

@may_spawn_proc
def test_double_div():
    fnp, _ = fncptr_from_c_script("test_double_div.c", "test_fnc", restype=ctypes.c_double)
    assert within_err(fnp(), 1.1557282546316052)

@may_spawn_proc
def test_double_ordered_eq():
    fnp, _ = fncptr_from_c_script("test_double_ordered_eq.c", "test_fnc")
    assert fnp() == 0

@may_spawn_proc
def test_double_ordered_ne():
    fnp, _ = fncptr_from_c_script("test_double_ordered_ne.c", "test_fnc")
    assert fnp() == 1

@may_spawn_proc
def test_double_ordered_lt():
    fnp, _ = fncptr_from_c_script("test_double_ordered_lt.c", "test_fnc")
    assert fnp() == 1

@may_spawn_proc
def test_double_ordered_le():
    fnp, _ = fncptr_from_c_script("test_double_ordered_le.c", "test_fnc")
    assert fnp() == 1

@may_spawn_proc
def test_double_ordered_ge():
    fnp, _ = fncptr_from_c_script("test_double_ordered_ge.c", "test_fnc")
    assert fnp() == 1

@may_spawn_proc
def test_double_ordered_gt():
    fnp, _ = fncptr_from_c_script("test_double_ordered_gt.c", "test_fnc")
    assert fnp() == 1

@may_spawn_proc
def test_double_arg_pass():
    fnp, _ = fncptr_from_c_script("test_double_arg_pass.c", "test_fnc",
                               [ctypes.c_double, ctypes.c_double], ctypes.c_double)
    assert fnp(3.141593, 2.71828) == 5.859873

@may_spawn_proc
def test_double_sitofp():
    fnp, _ = fncptr_from_c_script("test_double_sitofp.c", "test_fnc", restype=ctypes.c_double)
    assert fnp() == -42.0

@may_spawn_proc
def test_double_uitofp():
    fnp, _ = fncptr_from_c_script("test_double_uitofp.c", "test_fnc", restype=ctypes.c_double)
    assert fnp() == 42.0

@may_spawn_proc
def test_double_fptosi():
    fnp, _ = fncptr_from_c_script("test_double_fptosi.c", "test_fnc", restype=ctypes.c_int64)
    assert fnp() == -3

@may_spawn_proc
def test_double_fptoui():
    fnp, _ = fncptr_from_c_script("test_double_fptoui.c", "test_fnc", restype=ctypes.c_uint64)
    assert fnp() == 3

