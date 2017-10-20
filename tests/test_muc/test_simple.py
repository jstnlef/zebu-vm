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
                CALL <stack_sig> stack_args(<int<64>>0 <int<64>>1 <int<64>>2 <int<64>>3 <int<64>>4 <int<64>>5 <int<64>>6 <int<64>>7 <int<64>>8)
                    EXC (exit(<int<32>> 0) exit(<int<32>> 1))

            exit(<int<32>> status):
                RET status
        }
        """,
        "test_except_stack_args");
    assert(execute("test_except_stack_args") == 1);

@pytest.mark.xfail(reason = "stack return values are not yet implemented on x86-64")
def test_stack_pass_and_return():
    compile_bundle(
        """
        .funcsig sig = (int<128> int<128> int<128> int<128> int<128> int<128>) ->(int<128> int<128>)
        .funcdef test_stack_pass_and_return <main_sig>
        {
            entry(<int<32>>argc <uptr<uptr<char>>>argv):
                (res_013 res_245) = CALL <sig> stacky(<int<128>>0 <int<128>>1 <int<128>>2 <int<128>>3 <int<128>>4 <int<128>>5)
                res_128 = ADD<int<128>> res_013 res_245
                res = TRUNC <int<128> int<32>> res_128
                RET res
        }
        
        .funcdef stacky <sig>
        {
            entry(<int<128>>a0 <int<128>>a1 <int<128>>a2 <int<128>>a3 <int<128>>a4 <int<128>>a5):
                res_01  = ADD<int<128>> a0 a1
                res_013 = ADD<int<128>> res_01 a3
                
                res_24 = MUL<int<128>> a2 a4
                res_245 = MUL<int<128>> res_24 a5
                RET (res_013 res_245)
        }
        """, "test_stack_pass_and_return");
    assert(execute("test_stack_pass_and_return") == 44);
def test_stack_args():
    lib = load_bundle(
        """
        .funcsig stack_sig = (double double double double double double double double double double)->(int<32>)
        .funcdef test_stack_args <stack_sig>
        {
            entry(<double>d0 <double>d1 <double>d2 <double>d3 <double>d4 <double>d5 <double>d6 <double>d7 <double> d8 <double> d9):
                ds0 = FMUL <double> d0 <double>0.0 d
                ds1 = FMUL <double> d1 <double>1.0 d
                ds2 = FMUL <double> d2 <double>2.0 d
                ds3 = FMUL <double> d3 <double>3.0 d
                ds4 = FMUL <double> d4 <double>4.0 d
                ds5 = FMUL <double> d5 <double>5.0 d
                ds6 = FMUL <double> d6 <double>6.0 d
                ds7 = FMUL <double> d7 <double>7.0 d
                ds8 = FMUL <double> d8 <double>8.0 d
                ds9 = FMUL <double> d9 <double>9.0 d
                s1  = FADD <double> ds0 ds1
                s2  = FADD <double> s1 ds2
                s3  = FADD <double> s2 ds3
                s4  = FADD <double> s3 ds4
                s5  = FADD <double> s4 ds5
                s6  = FADD <double> s5 ds6
                s7  = FADD <double> s6 ds7
                s8  = FADD <double> s7 ds8
                s9  = FADD <double> s8 ds9
                r   = FPTOSI <double int<32>> s9
                RET r 
        }
        """, "test_stack_args");
    test_stack_args = get_function(lib.test_stack_args, [ctypes.c_double, ctypes.c_double, ctypes.c_double, ctypes.c_double, ctypes.c_double, ctypes.c_double, ctypes.c_double, ctypes.c_double, ctypes.c_double, ctypes.c_double], ctypes.c_int32);

    args = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]
    assert(test_stack_args(*tuple(args)) == sum(map((lambda x: x**2), args)));

def test_double_inline():
    lib = load_bundle(
        """       
        .funcsig new_sig = ()->(ref<void>)
        .funcdef new_void <new_sig>
        {
            entry():
                //res = NEW <ref<void>>
                res = CCALL #DEFAULT <ufuncptr<new_sig> new_sig> <ufuncptr<new_sig>>EXTERN "malloc"()
                RET res
        }
        
        .funcdef double_inline <()->(ref<void> ref<void>)>
        {
            entry():
                a = CALL <()->(ref<void>)> new_void()
                b = CALL <()->(ref<void>)> new_void()
                RET (a b)
        }
        """, "test_double_inline");
def test_funcref():
    lib = load_bundle(
        """
      .funcdef triv<()->()>
      {
              entry():
                      RET
      }
        .funcdef ret_funcref <()->(funcref<()->()>)>
        {
            entry():
                RET triv
        }
        """, "test_funcref");

def test_load():
    lib = load_bundle(
        """
        .funcdef load <(iref<weakref<void>>)->(ref<void>)>
        {
            entry(<iref<weakref<void>>>a):
                r = LOAD<weakref<void>> a
		RET r
        }
        """, "test_load");
def test_xor():
    lib = load_bundle(
            """
            .funcdef xor<(int<8>)->(int<8>)>
            {
                entry(<int<8>>v):
                    r = XOR<int<8>> v <int<8>>1
                    RET r
            }
            """, "test_xor");

def test_exc_pass_values():
    compile_bundle(
        """
		.funcsig sig = (ref<void> int<64>)->(int<32>)
		.funcdef test_exc_pass_values <main_sig>
		{
			entry(<int<32>>argc <uptr<uptr<char>>>argv):
				BRANCH blk2(<int<64>>4 <ref<void>>NULL)
			blk2(<int<64>> v82555 <ref<void>> container_13):
				index_2893 = CALL <sig> excy(container_13 v82555) EXC(blk3(index_2893) blk4(v82555 container_13))

			blk3(<int<32>> v82556):
				RET v82556
	
			blk4(<int<64>> name_231 <ref<void>> container_14):
				a_32 = TRUNC <int<64> int<32>> name_231
				RET a_32
		}
		.funcdef excy <sig>
		{
			entry(<ref<void>>a0 <int<64>>a5):
				THROW <ref<void>>NULL
		}

        """, "test_exc_pass_values");
    assert(execute("test_exc_pass_values") == 4);

def test_nest_loop():
    lib = load_bundle(
        """
        .funcdef nest_loop<(int<64>) -> (int<64>)>
        {
            entry(<int<64>> n):
                BRANCH outer_header(n <int<64>> 0 <int<64>> 0)
            
            outer_header(<int<64>> n <int<64>> i <int<64>> sum):
                outer_cond = SLT <int<64>> i n
                BRANCH2 outer_cond outer_body(n i sum) exit(sum)
            
            outer_body(<int<64>> n <int<64>> i <int<64>> sum):
                BRANCH inner_header(n i <int<64>> 0 sum)
            
            outer_step(<int<64>> n <int<64>> i <int<64>> sum):
                i2 = ADD <int<64>> i <int<64>> 1
                BRANCH outer_header(n i2 sum)
            
            inner_header(<int<64>> n <int<64>> i <int<64>> j <int<64>> sum):
                inner_cond = SLT <int<64>> j i
                BRANCH2 inner_cond inner_body(n i j sum) outer_step(n i sum)
            
            inner_body(<int<64>> n <int<64>> i <int<64>> j <int<64>> sum):
                sum2 = ADD <int<64>> sum j
                j2 = ADD <int<64>> j <int<64>> 1
                BRANCH inner_header(n i j2 sum2)
            
            exit(<int<64>> sum):
                RET sum
        }
        """, "nest_loop"
    )

    nest_loop = get_function(lib.nest_loop, [ctypes.c_int64], ctypes.c_int64)
    assert(nest_loop(100) == 161700)