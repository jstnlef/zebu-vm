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

def test_taillcall_simple():
    compile_bundle(
        """
         .funcdef test_taillcall_simple <main_sig>
        {
            entry(<int<32>>argc <uptr<uptr<char>>>argv):
                TAILCALL <main_sig>taillcallee(argc argv)
        }
        .funcdef taillcallee <main_sig>
        {
            entry(<int<32>>argc <uptr<uptr<char>>>argv):
                RET argc
        }
        """, "test_taillcall_simple");
    assert(execute("test_taillcall_simple", ["2", "3", "4"]) == 4);

#TODO: WHy does returning a pair of int<128>'s fail?
def test_taillcall_smaller_stack():
    compile_bundle(
            """
            .funcsig big_sig   = (int<128> int<128> int<128> int<128> int<128> int<128>)->(int<128>)
            .funcsig small_sig = (int<128> int<128> int<128> int<128> int<128>)         ->(int<128>)
            .funcdef test_taillcall_smaller_stack <main_sig>
            {
                entry(<int<32>>argc <uptr<uptr<char>>>argv):
                    res_128 = CALL <big_sig> bigger_stack(<int<128>>0 <int<128>>1 <int<128>>2 <int<128>>3 <int<128>>4 <int<128>>5)
                    res = TRUNC <int<128> int<32>> res_128
                    RET res
            }
            
             .funcdef bigger_stack <big_sig>
            {
                entry(<int<128>>a0 <int<128>>a1 <int<128>>a2 <int<128>>a3 <int<128>>a4 <int<128>>a5):
                    TAILCALL <small_sig> smaller_stack(a0 a1 a2 a3 a4)
            }
            .funcdef smaller_stack <small_sig>
            {
                entry(<int<128>>a0 <int<128>>a1 <int<128>>a2 <int<128>>a3 <int<128>>a4):
                    res_01  = ADD<int<128>> a0 a1
                    res_013 = ADD<int<128>> res_01 a3
                    
                    res_24 = MUL<int<128>> a2 a4
                    res_013_24 = ADD<int<128>> res_013 res_24
                    RET res_013_24
            }
            """, "test_taillcall_smaller_stack");
    assert(execute("test_taillcall_smaller_stack") == 12);