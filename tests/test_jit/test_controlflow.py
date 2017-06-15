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
import ctypes, ctypes.util

@may_spawn_proc
def test_branch():
    fn, _ = fncptr_from_c_script("test_branch.c", "test_fnc")
    assert fn() == 30

@may_spawn_proc
def test_branch2():
    fn, _ = fncptr_from_c_script("test_branch2.c", "test_fnc", [ctypes.c_byte])
    assert fn(1) == 30
    assert fn(0) == 200

@may_spawn_proc
def test_ccall():
    fn, _ = fncptr_from_c_script("test_ccall.c", "test_ccall", [ctypes.c_ulonglong])
    assert fn(0x7e707560c92d5400) == 0x7e707560c92d5400

@may_spawn_proc
def test_extern_func():
    # libc = ctypes.CDLL(ctypes.util.find_library('c'), ctypes.RTLD_GLOBAL)
    fn, _ = fncptr_from_c_script("test_extern_func.c", "test_write", [ctypes.c_void_p, ctypes.c_size_t], ctypes.c_int64)
    buf = ctypes.create_string_buffer('hello world!\n')
    assert fn(ctypes.byref(buf), len(buf)) == len(buf)
