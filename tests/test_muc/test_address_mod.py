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

# all the tests try to add 1 with 42, with a result of 43
# but the constants may be in the memory
# these tests are designed to test most of the cases for addressing mode on x86

def test_add_load_val():
    compile_bundle(
        """
        .funcdef test_add_load_val <main_sig>
        {
            entry(<int<32>>argc <uptr<uptr<char>>>argv):
                ref_y = NEW <int<64>>
                iref_y = GETIREF <int<64>> ref_y
                STORE <int<64>> iref_y <int<64>> 42
                BRANCH body(<int<64>> 1 ref_y)
            
            body(<int<64>> x <ref<int<64>>> ref_y):
                iref_y = GETIREF <int<64>> ref_y
                y = LOAD <int<64>> iref_y
                sum = ADD <int<64>> x y
                BRANCH exit(sum)
            
            exit(<int<64>> res):
                res32 = TRUNC <int<64> int<32>> res
                RET res32
        }
        """, "test_add_load_val"
    )
    assert(execute("test_add_load_val") == 43)

def test_add_load_cast():
    compile_bundle(
        """
        .funcdef test_add_load_cast <main_sig>
        {
            entry(<int<32>>argc <uptr<uptr<char>>>argv):
                ref_y = NEW <int<64>>
                iref_y = GETIREF <int<64>> ref_y
                STORE <int<64>> iref_y <int<64>> 42
                ref32_y = REFCAST <ref<int<64>> ref<int<32>>> ref_y
                BRANCH body(<int<64>> 1 ref32_y)
                
            body(<int<64>> x <ref<int<32>>> ref32_y):
                ref_y = REFCAST <ref<int<32>> ref<int<64>>> ref32_y
                iref_y = GETIREF <int<64>> ref_y
                y = LOAD <int<64>> iref_y
                sum = ADD <int<64>> x y
                BRANCH exit(sum)
            
            exit(<int<64>> res):
                res32 = TRUNC <int<64> int<32>> res
                RET res32
        }
        """, "test_add_load_cast"
    )
    assert(execute("test_add_load_cast") == 43)

def test_add_load_load():
    compile_bundle(
        """
        .funcdef test_add_load_load <main_sig>
        {
            entry(<int<32>>argc <uptr<uptr<char>>>argv):
                ref_y = NEW <int<64>>
                iref_y = GETIREF <int<64>> ref_y
                STORE <int<64>> iref_y <int<64>> 42
                
                ref_cell = NEW <ref<int<64>>>
                iref_cell = GETIREF <ref<int<64>>> ref_cell
                STORE <ref<int<64>>> iref_cell ref_y
                BRANCH body(<int<64>> 1 ref_cell)
                
            body(<int<64>> x <ref<ref<int<64>>>> ref_cell):
                iref_cell = GETIREF <ref<int<64>>> ref_cell
                ref_y = LOAD <ref<int<64>>> iref_cell
                iref_y = GETIREF <int<64>> ref_y
                y = LOAD <int<64>> iref_y
                sum = ADD <int<64>> x y
                BRANCH exit(sum)
            
            exit(<int<64>> res):
                res32 = TRUNC <int<64> int<32>> res
                RET res32
        }
        """, "test_add_load_load"
    )
    assert(execute("test_add_load_load") == 43)

def test_add_load_global():
    compile_bundle(
        """
        .global global_y <int<64>>
        .funcdef test_add_load_global <main_sig>
        {
            entry(<int<32>>argc <uptr<uptr<char>>>argv):
                STORE <int<64>> global_y <int<64>> 42
                BRANCH body(<int<64>> 1)
                
            body(<int<64>> x):
                y = LOAD <int<64>> global_y
                sum = ADD <int<64>> x y
                BRANCH exit(sum)
            
            exit(<int<64>> res):
                res32 = TRUNC <int<64> int<32>> res
                RET res32
        }
        """, "test_add_load_global"
    )
    assert(execute("test_add_load_global") == 43)

def test_add_load_cast_global():
    compile_bundle(
        """
        .global global_y <int<64>>
        .funcdef test_add_load_cast_global <main_sig>
        {
            entry(<int<32>>argc <uptr<uptr<char>>>argv):
                STORE <int<64>> global_y <int<64>> 42
                BRANCH body(<int<32>> 1)
                
            body(<int<32>> x):
                global32_y = REFCAST <iref<int<64>> iref<int<32>>> global_y
                y = LOAD <int<32>> global32_y
                sum = ADD <int<32>> x y
                BRANCH exit(sum)
            
            exit(<int<32>> res):
                RET res
        }
        """, "test_add_load_cast_global"
    )
    assert(execute("test_add_load_cast_global") == 43)
