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

def test_taillcall_smaller_stack():
    compile_bundle(
        """
        .funcsig big_sig   = (int<128> int<128> int<128> int<128> int<128> int<128>)->(int<128> int<128>)
        .funcsig small_sig = (int<128> int<128> int<128> int<128> int<128>)         ->(int<128> int<128>)
        .funcdef test_taillcall_smaller_stack <main_sig>
        {
            entry(<int<32>>argc <uptr<uptr<char>>>argv):
                (res_013 res_24) = CALL <big_sig> bigger_stack(<int<128>>0 <int<128>>1 <int<128>>2 <int<128>>3 <int<128>>4 <int<128>>5)
                res_128 = ADD<int<128>> res_013 res_24
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
                RET (res_013 res_24)
        }
        """, "test_taillcall_smaller_stack");
    assert(execute("test_taillcall_smaller_stack") == 12);

@pytest.mark.xfail(reason = "unimplemented")
def test_taillcall_bigger_stack():
    compile_bundle(
        """
        .funcsig big_sig   = (int<128> int<128> int<128> int<128> int<128> int<128>)->(int<128> int<128>)
        .funcsig small_sig = (int<128> int<128> int<128> int<128> int<128>)         ->(int<128> int<128>)
        .funcdef test_taillcall_bigger_stack <main_sig>
        {
            entry(<int<32>>argc <uptr<uptr<char>>>argv):
                (res_013 res_245) = CALL <small_sig> smaller_stack(<int<128>>0 <int<128>>1 <int<128>>2 <int<128>>3 
                <int<128>>4)
                res_128 = ADD<int<128>> res_013 res_245
                res = TRUNC <int<128> int<32>> res_128
                RET res
        }
        
         .funcdef smaller_stack <small_sig>
        {
            entry(<int<128>>a0 <int<128>>a1 <int<128>>a2 <int<128>>a3 <int<128>>a4):
                TAILCALL <big_sig> bigger_stack(a0 a1 a2 a3 a4 <int<128>>5)
        }
        .funcdef bigger_stack <big_sig>
        {
            entry(<int<128>>a0 <int<128>>a1 <int<128>>a2 <int<128>>a3 <int<128>>a4 <int<128>>a5):
                res_01  = ADD<int<128>> a0 a1
                res_013 = ADD<int<128>> res_01 a3
                
                res_24 = MUL<int<128>> a2 a4
                res_245 = MUL<int<128>> res_24 a5
                RET (res_013 res_245)
        }
        """, "test_taillcall_bigger_stack");
    assert(execute("test_taillcall_bigger_stack") == 44);


def test_taillcall_exception():
    compile_bundle(
        """
        .funcdef tail_callee <(int<32>)->(int<32>)>
        {
            entry(<int<32>> arg):
                exc = NEW <int<32>>
                exc_iref = GETIREF <int<32>> exc
                STORE <int<32>> exc_iref arg
                THROW exc
        }
        .funcdef tail_caller <(int<32>)->(int<32>)>
        {
            entry(<int<32>> arg):
                TAILCALL <(int<32>)->(int<32>)> tail_callee(arg)
        }

        .funcdef test_taillcall_exception <main_sig>
        {
            entry(<int<32>>argc <uptr<uptr<char>>>argv):
                res = CALL <(int<32>)->(int<32>)> tail_caller(argc) EXC (norm(res) except())

            except()[@eparam]:
                eparam_cast = REFCAST <ref<void> ref<int<32>>> @eparam
                exc_iref = GETIREF <int<32>> eparam_cast
                val = LOAD <int<32>> exc_iref
                RET val
                
            norm(<int<32>> status):
                THROW <ref<void>>NULL // This should crash the program
        }
        """,
        "test_taillcall_exception");
    assert(execute("test_taillcall_exception", ["2", "3"]) == 3);