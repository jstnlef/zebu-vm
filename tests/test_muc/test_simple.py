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

# Tests that zebu can handle wierd, but valid mu names
def test_name():
    compile_bundle(
            """
            .global @-0.a5-1_5 <void>
            .const @0 <int<32>> = 0
            .funcdef @0-main.func <main_sig>
            {
                entry(<int<32>>%1.3 <uptr<uptr<char>>>%-):
                     RET @0
            }
            """, "test_name", "0-main.func");
    assert(execute("test_name") == 0);


def test_argc():
    compile_bundle(
        """
         .funcdef test_argc <main_sig>
        {
            entry(<int<32>>argc <uptr<uptr<char>>>argv):
                RET argc
        }
        """, "test_argc");
    assert(execute("test_argc", ["2", "3", "4"]) == 4);

@pytest.mark.xfail(reason = "1 bit division is not implemented on x86-64")
def test_int1():
    compile_bundle(
        """
         .funcdef test_int <main_sig>
        {
            entry(<int<32>>argc <uptr<uptr<char>>>argv):
                res10 = ADD <int<1>> <int<1>>1 <int<1>>1 // = 1
                res11 = ADD <int<1>> res10 <int<1>>1 // = 1
                res1  = UDIV <int<1>> res11 <int<1>>1 // = 1
                res   = ZEXT <int<1> int<32>> res1 // = 1
                RET res
        }
        """,
        "test_int")
    assert(execute("test_int") == 1);


def test_add():
    lib = load_bundle(
        """
         .funcdef test_add <(int<64> int<64>)->(int<64>)>
        {
            entry(<int<64>>a <int<64>>b):
                res = ADD <int<64>> a b
                RET res
        }
        """,
        "test_add");
    test_add = get_function(lib.test_add, [ctypes.c_int64, ctypes.c_int64], ctypes.c_int64);
    assert(test_add(1, 2) == 3);
    assert(test_add(-40, 60) == 20);

def test_except_stack_args():
    compile_bundle(
        """
        .funcsig stack_sig = (int<64> int<64> int<64> int<64> int<64> int<64> int<64> int<64> int<64>)->()
        .funcdef stack_args <stack_sig>
        {
            entry(<int<64>> v0 <int<64>> v1 <int<64>> v2 <int<64>> v3 <int<64>> v4 <int<64>> v5 <int<64>> v6 <int<64>> v7 <int<64>> v8):
                THROW <ref<void>> NULL
        }
        .funcdef test_except_stack_args <main_sig>
        {
            entry(<int<32>>argc <uptr<uptr<char>>>argv):
                CALL <stack_sig> stack_args(<int<32>>0 <int<32>>1 <int<32>>2 <int<32>>3 <int<32>>4 <int<32>>5 <int<32>>6 <int<32>>7 <int<32>>8)
                    EXC (exit(<int<32>> 0) exit(<int<32>> 1))

            exit(<int<32>> status):
                RET status
        }
        """,
        "test_except_stack_args");
    assert(execute("test_except_stack_args") == 1);


def test_ldp_bug():
    compile_bundle(
        """
        .funcdef foo <(int<128> int<128> int<128> int<128> int<128> int<128>)->(int<128>)>
        {
            entry(<int<128>>a0 <int<128>>a1 <int<128>>a2 <int<128>>a3 <int<128>>a4 <int<128>>a5):
                RET a5
        }
        """, "test_taillcall_smaller_stack");
assert(execute("test_taillcall_smaller_stack") == 12);