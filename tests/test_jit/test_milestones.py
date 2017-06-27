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

"""
Harness JIT tests using py.test framework
"""
from util import fncptr_from_c_script, may_spawn_proc
import ctypes

@may_spawn_proc
def test_constant_function():
    fn, _ = fncptr_from_c_script("test_constfunc.c", 'test_fnc')
    assert fn() == 0

@may_spawn_proc
def test_milsum():
    fn, _ = fncptr_from_c_script("test_milsum.c", "milsum", [ctypes.c_ulonglong])
    assert fn(1000000) == 500000500000

@may_spawn_proc
def test_factorial():
    fn, _ = fncptr_from_c_script("test_fac.c", "fac", [ctypes.c_ulonglong])
    assert fn(20) == 2432902008176640000

@may_spawn_proc
def test_fibonacci():
    fn, _ = fncptr_from_c_script("test_fib.c", "fib", [ctypes.c_ulonglong])
    assert fn(20) == 6765

@may_spawn_proc
def test_multifunc():
    fn, _ = fncptr_from_c_script("test_multifunc.c", "entry")
    assert fn() == 6765
