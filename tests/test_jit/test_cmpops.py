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

from util import fncptr_from_c_script, may_spawn_proc, mu_instance_via_ctyeps
import ctypes

@may_spawn_proc
def test_eq_int():
    fn, _ = fncptr_from_c_script("test_eq_int.c", "test_fnc")
    assert fn() == 0

@may_spawn_proc
def test_eq_ref():
    mu = mu_instance_via_ctyeps()
    fn, _ = fncptr_from_c_script("test_eq_ref.c", "test_fnc")
    assert fn() == 0

@may_spawn_proc
def test_ne_int():
    fn, _ = fncptr_from_c_script("test_ne_int.c", "test_fnc")
    assert fn() == 1

@may_spawn_proc
def test_ne_ref():
    mu = mu_instance_via_ctyeps()
    fn, _ = fncptr_from_c_script("test_ne_ref.c", "test_fnc")
    assert fn() == 1

@may_spawn_proc
def test_sge():
    fn, _ = fncptr_from_c_script("test_sge.c", "test_fnc", restype=ctypes.c_uint8)
    assert fn() == 1

@may_spawn_proc
def test_sgt():
    fn, _ = fncptr_from_c_script("test_sgt.c", "test_fnc", restype=ctypes.c_uint8)
    assert fn() == 0

@may_spawn_proc
def test_sle():
    fn, _ = fncptr_from_c_script("test_sle.c", "test_fnc", restype=ctypes.c_uint8)
    assert fn() == 1

@may_spawn_proc
def test_ule():
    fn, _ = fncptr_from_c_script("test_ule.c", "test_fnc", restype=ctypes.c_uint8)
    assert fn() == 1

@may_spawn_proc
def test_slt():
    fn, _ = fncptr_from_c_script("test_slt.c", "test_fnc", restype=ctypes.c_uint8)
    assert fn() == 0

@may_spawn_proc
def test_ult():
    fn, _ = fncptr_from_c_script("test_ult.c", "test_fnc", restype=ctypes.c_uint8)
    assert fn() == 0
