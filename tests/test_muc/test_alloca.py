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
                                    
                // sum all the *_int values togterh 
                res_0 = ADD <int<64>> ai_int ad_int
                res_1 = ADD <int<64>> res_0 ar_int
                res_2 = ADD <int<64>> res_1 argc_int
                RET res_2
        }
        """, "test_alloca");

    alloca = get_function(lib.alloca, [ctypes.c_int64], ctypes.c_int64);
    assert(alloca(-56) == -56);