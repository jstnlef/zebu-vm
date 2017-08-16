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

def test_swapstack_simple():
    compile_bundle(
        """
        .funcdef new_func <()->()>
        {
            entry():
                CCALL #DEFAULT <exit_type exit_sig> exit(<int<32>>3) 
                RET
        }        
        .funcdef test_swapstack_simple <main_sig>
        {
            entry(<int<32>>argc <uptr<uptr<char>>>argv):
                s = COMMINST uvm.new_stack<[()->()]>(new_func)
                SWAPSTACK s KILL_OLD PASS_VALUES<>()
        }
        """, "test_swapstack_simple");
    assert(execute("test_swapstack_simple", []) == 3);

def test_swapstack_swap_back():
    compile_bundle(
        """
        .funcdef new_func <(stackref)->()>
        {
            entry(<stackref>s):
                SWAPSTACK s KILL_OLD PASS_VALUES<>()
        }        
        .funcdef test_swapstack_swap_back <main_sig>
        {
            entry(<int<32>>argc <uptr<uptr<char>>>argv):
                cs =  COMMINST uvm.current_stack()
                s = COMMINST uvm.new_stack<[(stackref)->()]>(new_func)
                SWAPSTACK s RET_WITH<> PASS_VALUES<stackref>(cs)
                RET <int<32>>3
                
        }
        """, "test_swapstack_swap_back");
    assert(execute("test_swapstack_swap_back", []) == 3);

def test_swapstack_pass_vals():
    compile_bundle(
        """
        .funcdef new_func <(stackref)->()>
        {
            entry(<stackref>s):
                SWAPSTACK s KILL_OLD PASS_VALUES<int<32>>(<int<32>> 3) 
        }        
        .funcdef test_swapstack_pass_vals <main_sig>
        {
            entry(<int<32>>argc <uptr<uptr<char>>>argv):
                cs =  COMMINST uvm.current_stack()
                s = COMMINST uvm.new_stack<[(stackref)->()]>(new_func)
                r = SWAPSTACK s RET_WITH<int<32>> PASS_VALUES<stackref>(cs)
                RET r
        }
        """, "test_swapstack_pass_vals");
    assert(execute("test_swapstack_pass_vals", []) == 3);

def test_swapstack_throw():
    compile_bundle(
        """
        .funcdef new_func <(stackref)->()>
        {
            entry(<stackref>s):
                er = NEW <int<32>>
                eri = GETIREF <int<32>> er
                STORE <int<32>> eri <int<32>> 3
                ev = REFCAST <ref<int<32>> ref<void>> er
                SWAPSTACK s KILL_OLD THROW_EXC ev
        }
        .funcdef test_swapstack_throw <main_sig>
        {
            entry(<int<32>>argc <uptr<uptr<char>>>argv):
                cs =  COMMINST uvm.current_stack()
                s = COMMINST uvm.new_stack<[(stackref)->()]>(new_func)
                r = SWAPSTACK s RET_WITH<int<32>> PASS_VALUES<stackref>(cs) EXC(nor_dest(r) exc_dest())
            nor_dest(<int<32>> r):
                RET <int<32>>0
            exc_dest()[exc_param]:
                e = REFCAST <ref<void> ref<int<32>>> exc_param
                evi = GETIREF <int<32>> e
                ev = LOAD <int<32>> evi
                RET ev
        }
        """, "test_swapstack_throw");
    assert(execute("test_swapstack_throw", []) == 3);

def test_swapstack_throw_back():
    compile_bundle(
        """
        .funcdef new_func <(stackref)->()>
        {
            entry(<stackref>s):
                er = NEW <int<32>>
                eri = GETIREF <int<32>> er
                STORE <int<32>> eri <int<32>> 1
                ev = REFCAST <ref<int<32>> ref<void>> er
                r = SWAPSTACK s RET_WITH<int<32>> THROW_EXC ev EXC(nor_dest(r) exc_dest()) 
            
            nor_dest(<int<32>> r):
                CCALL #DEFAULT <exit_type exit_sig> exit(<int<32>>0)
                RET
            exc_dest()[exc_param]:
                e = REFCAST <ref<void> ref<int<32>>> exc_param
                evi = GETIREF <int<32>> e
                ev = LOAD <int<32>> evi
                CCALL #DEFAULT <exit_type exit_sig> exit(ev)
                RET
        }
        .funcdef test_swapstack_throw_back <main_sig>
        {
            entry(<int<32>>argc <uptr<uptr<char>>>argv):
                cs =  COMMINST uvm.current_stack()
                s = COMMINST uvm.new_stack<[(stackref)->()]>(new_func)
                r = SWAPSTACK s RET_WITH<int<32>> PASS_VALUES<stackref>(cs) EXC(nor_dest(r) exc_dest(s))
            nor_dest(<int<32>> r):
                RET <int<32>>0
            exc_dest(<stackref> s)[exc_param]:
                e = REFCAST <ref<void> ref<int<32>>> exc_param
                evi = GETIREF <int<32>> e
                ev = LOAD <int<32>> evi
                newv = ADD <int<32>> ev <int<32>> 2
                STORE <int<32>> evi newv
                // exc_param += 2
                
                // Throw back to new_func
                SWAPSTACK s KILL_OLD THROW_EXC exc_param
        }
        """, "test_swapstack_throw_back");
    assert(execute("test_swapstack_throw_back", []) == 3);
