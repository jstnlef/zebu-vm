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

def test_alloca_simple():
    compile_bundle(
        """
         .funcdef test_alloca_simple <main_sig>
        {
            entry(<int<32>>argc <uptr<uptr<char>>>argv):
                a = ALLOCA <struct<int<64> double ref<void>>>
                RET <int<32>>0
        }
        """, "test_alloca_simple");
    assert(execute("test_alloca_simple") == 0);

def test_alloca():
    lib = load_bundle(
        """
        .typedef type = struct<int<64> double ref<void>>
        .funcdef alloca <(int<64>)->(int<64>)>
        {
            entry(<int<64>>arg):
                a = ALLOCA <type>
                
                // Load the int field to ai_int
                ai_ref = GETFIELDIREF <type 0> a
                ai_int = LOAD <int<64>> ai_ref
                
                // Load the double field to ad_int (converting it to an int<64>)
                ad_ref = GETFIELDIREF <type 1> a
                ad = LOAD <double> ad_ref
                ad_int = BITCAST <double int<64>> ad 
                
                // Load the ref field to ar_int (which will be '0' for a null ref, and '1' otherwise)
                ar_ref = GETFIELDIREF <type 2> a
                ar = LOAD <ref<void>> ar_ref
                ar_null = NE <ref<void>> ar <ref<void>>NULL
                ar_int = ZEXT <int<1> int<64>> ar_null
                
                // Store arg into the ALLOCA'd area
                STORE <type> ai_ref arg
                argc_int = LOAD <int<64>> ai_ref
                                    
                // or all the *_int values together
                res_0 = OR <int<64>> ai_int ad_int
                res_1 = OR <int<64>> res_0 ar_int
                res_2 = OR <int<64>> res_1 argc_int
                RET res_2
        }
        """, "test_alloca");

    alloca = get_function(lib.alloca, [ctypes.c_int64], ctypes.c_int64);
    assert(alloca(-56) == -56);

def test_allocahybrid_simple():
    compile_bundle(
        """
         .funcdef test_allocahybrid_simple <main_sig>
        {
            entry(<int<32>>argc <uptr<uptr<char>>>argv):
                a = ALLOCAHYBRID <hybrid<int<1>> int<32>> argc
                RET argc
        }
        """, "test_allocahybrid_simple");
    assert(execute("test_allocahybrid_simple", ["1", "2", "3"]) == 4);


def test_allocahybrid():
    lib = load_bundle(
        """
        .typedef type = hybrid<int<1> int<64>>
        .funcdef allocahybrid <(int<8>)->(int<64>)>
        {
            entry(<int<8>>n):
                a = ALLOCAHYBRID <type int<64>> n 

                // Load the int<1> field to ai_int (as a 64-bit integer)
                ai_ref = GETFIELDIREF <type 0> a
                ai     = LOAD <int<64>> ai_ref
                ai_int = ZEXT <int<1> int<64>> ai

                a_var = GETVARPARTIREF <type> a
                n_zero = EQ <int<8>> n <int<8>>0
                // If the hybrid is non empty, sum all of it's variable elements
                BRANCH2 n_zero exit(ai_int) sum(a_var n ai_int)

            // Sum 'sum' and the n elements of pos
            // branch to exit with sum once finished
            sum(<iref<int<64>>>pos <int<8>>n <int<64>>sum):
                val     = LOAD <int<64>> pos
                new_pos = SHIFTIREF <int<64> int<1>> pos <int<1>>1
                new_sum = OR <int<64>> sum val
                new_n   = SUB <int<8>> n <int<8>>1
                n_zero  = EQ <int<8>> n <int<8>>1
                BRANCH2 n_zero exit(new_sum) sum(new_pos new_n new_sum)
                
            exit(<int<64>> sum):
                RET sum
        }
        """, "test_allocahybrid");

    allocahybrid = get_function(lib.allocahybrid, [ctypes.c_uint8], ctypes.c_uint64);
    assert(allocahybrid(56) == 0);

def test_allocahybrid_imm():
    bundle_template = """
        .typedef type = hybrid<int<1> int<64>>
        .const n <int<64>> = {}
        .funcdef allocahybrid_imm <(int<64>)->(int<64>)>
        {{
            entry():
                a = ALLOCAHYBRID <type int<64>> n 

                // Load the int<1> field to ai_int (as a 64-bit integer)
                ai_ref = GETFIELDIREF <type 0> a
                ai     = LOAD <int<64>> ai_ref
                ai_int = ZEXT <int<1> int<64>> ai

                a_var = GETVARPARTIREF <type> a
                n_zero = EQ <int<64>> n <int<64>>0
                // If the hybrid is non empty, sum all of it's variable elements
                BRANCH2 n_zero exit(ai_int) sum(a_var n ai_int)

            // Sum 'sum' and the n elements of pos
            // branch to exit with sum once finished
            sum(<iref<int<64>>>pos <int<64>>n <int<64>>sum):
                val     = LOAD <int<64>> pos
                new_pos = SHIFTIREF <int<64> int<1>> pos <int<1>>1
                new_sum = OR <int<64>> sum val
                new_n   = SUB <int<64>> n <int<64>>1
                n_zero  = EQ <int<64>> n <int<64>>1
                BRANCH2 n_zero exit(new_sum) sum(new_pos new_n new_sum)
                
            exit(<int<64>> sum):
                RET sum
        }}
        """;
    def allocahybrid_imm(n): # type: (str) -> int
        lib = load_bundle(bundle_template.format(n), "test_allocahybrid_{}".format(n));
        return get_function(lib.allocahybrid_imm, [], ctypes.c_uint64)();

    assert(allocahybrid_imm("16") == 0);
    assert(allocahybrid_imm("0") == 0);
