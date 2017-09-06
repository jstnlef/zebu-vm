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

def test_swapstack_kill_old():
    compile_bundle(
        """
        .funcdef test_swapstack_kill_old_swapee <()->()>
        {
            entry():
                CCALL #DEFAULT <exit_type exit_sig> exit(<int<32>>3) 
                RET
        }        
        .funcdef test_swapstack_kill_old <main_sig>
        {
            entry(<int<32>>argc <uptr<uptr<char>>>argv):
                s = COMMINST uvm.new_stack<[()->()]>(test_swapstack_kill_old_swapee)
                SWAPSTACK s KILL_OLD PASS_VALUES<>()
        }
        """, "test_swapstack_kill_old");
    assert(execute("test_swapstack_kill_old", []) == 3);

def test_swapstack_swap_back():
    compile_bundle(
        """
        .funcdef test_swapstack_swap_back_swapee <(stackref)->()>
        {
            entry(<stackref>s):
                SWAPSTACK s KILL_OLD PASS_VALUES<>()
        }        
        .funcdef test_swapstack_swap_back <main_sig>
        {
            entry(<int<32>>argc <uptr<uptr<char>>>argv):
                cs =  COMMINST uvm.current_stack()
                s = COMMINST uvm.new_stack<[(stackref)->()]>(test_swapstack_swap_back_swapee)
                SWAPSTACK s RET_WITH<> PASS_VALUES<stackref>(cs)
                RET <int<32>>3
                
        }
        """, "test_swapstack_swap_back");
    assert(execute("test_swapstack_swap_back", []) == 3);

def test_swapstack_ret_values():
    compile_bundle(
        """
        .funcdef test_swapstack_ret_values_swapee <(stackref)->()>
        {
            entry(<stackref>s):
                SWAPSTACK s KILL_OLD PASS_VALUES<int<32>>(<int<32>> 2) 
        }        
        .funcdef test_swapstack_ret_values <main_sig>
        {
            entry(<int<32>>argc <uptr<uptr<char>>>argv):
                cs =  COMMINST uvm.current_stack()
                s = COMMINST uvm.new_stack<[(stackref)->()]>(test_swapstack_ret_values_swapee)
                r = SWAPSTACK s RET_WITH<int<32>> PASS_VALUES<stackref>(cs)
                rv = ADD <int<32>> argc r
                RET rv
        }
        """, "test_swapstack_ret_values");
    assert(execute("test_swapstack_ret_values", []) == 3);

def test_swapstack_pass_stack_args():
    compile_bundle(
        """
        .funcsig stack_sig = (stackref double double double double double double double double double double)->()
        .funcdef test_swapstack_pass_stack_args_swapee <stack_sig>
        {
            entry(<stackref>s <double>d0 <double>d1 <double>d2 <double>d3 <double>d4 <double>d5 <double>d6 <double>d7 <double> d8 <double> d9):
                SWAPSTACK s KILL_OLD PASS_VALUES<double double double double double double double double double double>(d0 d1 d2 d3 d4 d5 d6 d7 d8 d9) 
        }
        .funcdef test_swapstack_pass_stack_args <main_sig>
        {
            entry(<int<32>>argc <uptr<uptr<char>>>argv):
                cs =  COMMINST uvm.current_stack()
                s = COMMINST uvm.new_stack<[stack_sig]>(test_swapstack_pass_stack_args_swapee)
                (d0 d1 d2 d3 d4 d5 d6 d7 d8 d9) = SWAPSTACK s RET_WITH<double double double double double double double double double double> PASS_VALUES<stackref double double double double double double double double double double>(cs <double>0.0 d <double>1.0 d <double>2.0 d <double>3.0 d <double>4.0 d <double>5.0 d <double>6.0 d <double>7.0 d <double>8.0 d <double>9.0 d)
                s1 = FADD <double> d0 d1
                s2 = FADD <double> s1 d2
                s3 = FADD <double> s2 d3
                s4 = FADD <double> s3 d4
                s5 = FADD <double> s4 d5
                s6 = FADD <double> s5 d6
                s7 = FADD <double> s6 d7
                s8 = FADD <double> s7 d8
                s9 = FADD <double> s8 d9
                r = FPTOSI <double int<32>> s9
                RET r
        }
        """, "test_swapstack_pass_stack_args");
    assert(execute("test_swapstack_pass_stack_args", []) == 45);

def test_swapstack_spill():
    compile_bundle(
        """
        .funcdef test_swapstack_spill_swapee <(stackref)->()>
        {
            entry(<stackref>s):
                SWAPSTACK s KILL_OLD PASS_VALUES<>()
        }
        .funcdef test_swapstack_spill <main_sig>
        {
            entry(<int<32>>argc <uptr<uptr<char>>>argv):
                BRANCH block(<double>0.0 d <double>1.0 d <double>2.0 d <double>3.0 d <double>4.0 d <double>5.0 d <double>6.0 d <double>7.0 d <double>8.0 d <double>9.0 d)
                
            block(<double>d0 <double>d1 <double>d2 <double>d3 <double>d4 <double>d5 <double>d6 <double>d7 <double> d8 <double> d9):
                cs =  COMMINST uvm.current_stack()
                s = COMMINST uvm.new_stack<[(stackref)->()]>(test_swapstack_spill_swapee)
                SWAPSTACK s RET_WITH<> PASS_VALUES<stackref>(cs)
                s1 = FADD <double> d0 d1
                s2 = FADD <double> s1 d2
                s3 = FADD <double> s2 d3
                s4 = FADD <double> s3 d4
                s5 = FADD <double> s4 d5
                s6 = FADD <double> s5 d6
                s7 = FADD <double> s6 d7
                s8 = FADD <double> s7 d8
                s9 = FADD <double> s8 d9
                r = FPTOSI <double int<32>> s9 
                RET r
        }
        """, "test_swapstack_spill");
    assert(execute("test_swapstack_spill", []) == 45);

def test_swapstack_throw():
    compile_bundle(
        """
        .funcdef test_swapstack_throw_swapee <(stackref)->()>
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
                s = COMMINST uvm.new_stack<[(stackref)->()]>(test_swapstack_throw_swapee)
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
        .funcdef test_swapstack_throw_back_swapee <(stackref)->()>
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
                s = COMMINST uvm.new_stack<[(stackref)->()]>(test_swapstack_throw_back_swapee)
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

def test_kill_stack():
    compile_bundle(
        """
        .funcdef test_kill_stack_swapee <(stackref)->()>
        {
            entry(<stackref>s):
                COMMINST uvm.kill_stack(s)            
                CCALL #DEFAULT <exit_type exit_sig> exit(<int<32>>3) 
                RET
        }        
        .funcdef test_kill_stack <main_sig>
        {
            entry(<int<32>>argc <uptr<uptr<char>>>argv):
                cs = COMMINST uvm.current_stack()
                s = COMMINST uvm.new_stack<[(stackref)->()]>(test_kill_stack_swapee)
                SWAPSTACK s RET_WITH<> PASS_VALUES<stackref>(cs)
                RET <int<32>>0
                
        }
        """, "test_kill_stack");
    assert(execute("test_kill_stack", []) == 3);

def test_newthread_simple():
    compile_bundle(
        """
        .funcdef test_newthread_simple_thread <()->()>
        {
            entry():
                CCALL #DEFAULT <exit_type exit_sig> exit(<int<32>>3) 
                RET
        }        
        .funcdef test_newthread_simple <main_sig>
        {
            entry(<int<32>>argc <uptr<uptr<char>>>argv):
                s = COMMINST uvm.new_stack<[()->()]>(test_newthread_simple_thread)
                t = NEWTHREAD s PASS_VALUES<>()
                COMMINST uvm.thread_exit()
        }
        """, "test_newthread_simple");
    assert(execute("test_newthread_simple", []) == 3);

def test_newthread_swapstack():
    compile_bundle(
        """
        .funcdef test_newthread_swapstack_thread <(stackref)->()>
        {
            entry(<stackref>s):
                t = NEWTHREAD s PASS_VALUES<int<32>>(<int<32>> 2)
                BRANCH loop()
            loop():
                BRANCH loop()
        }        
        .funcdef test_newthread_swapstack <main_sig>
        {
            entry(<int<32>>argc <uptr<uptr<char>>>argv):
                cs =  COMMINST uvm.current_stack()
                s = COMMINST uvm.new_stack<[(stackref)->()]>(test_newthread_swapstack_thread)
                r = SWAPSTACK s RET_WITH<int<32>> PASS_VALUES<stackref>(cs)
                rv = ADD <int<32>> argc r
                RET rv
                // argc = 1
        }
        """, "test_newthread_swapstack");
    assert(execute("test_newthread_swapstack", []) == 3);

def test_newthread_throw():
    compile_bundle(
        """
        .funcdef test_newthread_throw_thread <(stackref)->()>
        {
            entry(<stackref>s):
                er = NEW <int<32>>
                eri = GETIREF <int<32>> er
                STORE <int<32>> eri <int<32>> 3
                ev = REFCAST <ref<int<32>> ref<void>> er
                t = NEWTHREAD s THROW_EXC ev
                COMMINST uvm.thread_exit()
        }
        .funcdef test_newthread_throw <main_sig>
        {
            entry(<int<32>>argc <uptr<uptr<char>>>argv):
                cs =  COMMINST uvm.current_stack()
                s = COMMINST uvm.new_stack<[(stackref)->()]>(test_newthread_throw_thread)
                r = SWAPSTACK s RET_WITH<int<32>> PASS_VALUES<stackref>(cs) EXC(nor_dest(r) exc_dest())
            nor_dest(<int<32>> r):
                RET <int<32>>0
            exc_dest()[exc_param]:
                e = REFCAST <ref<void> ref<int<32>>> exc_param
                evi = GETIREF <int<32>> e
                ev = LOAD <int<32>> evi
                RET ev
        }
        """, "test_newthread_throw");
    assert(execute("test_newthread_throw", []) == 3);

def test_newthread_threadlocal():
    compile_bundle(
        """
        .funcdef test_newthread_threadlocal_thread <()->()>
        {
            entry():
                tv = COMMINST uvm.get_threadlocal()
                tr = REFCAST <ref<void> ref<int<32>>> tv
                tvi = GETIREF <int<32>> tr
                tv = LOAD <int<32>> tvi
                CCALL #DEFAULT <exit_type exit_sig> exit(tv)
                RET 
        }
        .funcdef test_newthread_threadlocal <main_sig>
        {
            entry(<int<32>>argc <uptr<uptr<char>>>argv):
                cs =  COMMINST uvm.current_stack()
                s = COMMINST uvm.new_stack<[()->()]>(test_newthread_threadlocal_thread)
                
                tr = NEW <int<32>>
                tri = GETIREF <int<32>> tr
                STORE <int<32>> tri <int<32>> 3
                tl = REFCAST <ref<int<32>> ref<void>> tr
                t = NEWTHREAD s THREADLOCAL (tl) PASS_VALUES<>()
                COMMINST uvm.thread_exit()
        }
        """, "test_newthread_threadlocal");
    assert(execute("test_newthread_threadlocal", []) == 3);

def test_newthread_stack_args():
    compile_bundle(
        """
        .funcsig stack_sig = (stackref double double double double double double double double double double)->()
        .funcdef test_newthread_stack_args_thread <stack_sig>
        {
            entry(<stackref>s <double>d0 <double>d1 <double>d2 <double>d3 <double>d4 <double>d5 <double>d6 <double>d7 <double> d8 <double> d9):
                s1 = FADD <double> d0 d1
                s2 = FADD <double> s1 d2
                s3 = FADD <double> s2 d3
                s4 = FADD <double> s3 d4
                s5 = FADD <double> s4 d5
                s6 = FADD <double> s5 d6
                s7 = FADD <double> s6 d7
                s8 = FADD <double> s7 d8
                s9 = FADD <double> s8 d9
                r = FPTOSI <double int<32>> s9
                CCALL #DEFAULT <exit_type exit_sig> exit(r)
                RET
        }
        .funcdef test_newthread_stack_args <main_sig>
        {
            entry(<int<32>>argc <uptr<uptr<char>>>argv):
                cs =  COMMINST uvm.current_stack()
                s = COMMINST uvm.new_stack<[stack_sig]>(test_newthread_stack_args_thread)
                t = NEWTHREAD s PASS_VALUES<stackref double double double double double double double double double double>(cs <double>0.0 d <double>1.0 d <double>2.0 d <double>3.0 d <double>4.0 d <double>5.0 d <double>6.0 d <double>7.0 d <double>8.0 d <double>9.0 d)
                COMMINST uvm.thread_exit()
        }
        """, "test_newthread_stack_args");
    assert(execute("test_newthread_stack_args", []) == 45);