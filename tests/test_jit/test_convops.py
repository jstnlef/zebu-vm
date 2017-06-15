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

@may_spawn_proc
def test_trunc():
    fn, _ = fncptr_from_c_script("test_trunc.c", "test_fnc", restype=ctypes.c_uint32)
    assert fn() == 0x58324b55

@may_spawn_proc
def test_sext():
    fn, _ = fncptr_from_c_script("test_sext.c", "test_fnc")
    assert fn() == 0xffffffffa8324b55

@may_spawn_proc
def test_zext():
    fn, _ = fncptr_from_c_script("test_zext.c", "test_fnc")
    assert fn() == 0x00000000a8324b55
