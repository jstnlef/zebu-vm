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

from rpython.rtyper.lltypesystem import rffi, lltype
from rpython.rlib.rmu import zebu as rmu
from rpython.translator.platform import platform
from util import fncptr_from_rpy_func, fncptr_from_py_script, may_spawn_proc
import ctypes, py, stat
import pytest

from test_rpython import run_boot_image, check

c_exit = rffi.llexternal('exit', [rffi.INT], lltype.Void, _nowrapper=True)

@may_spawn_proc
def test_rpython_dict_new_empty():
    def new_empty():
        a = {}

    fn, (db, bdlgen) = fncptr_from_rpy_func(new_empty, [], lltype.Void)
    bdlgen.mu.current_thread_as_mu_thread(rmu.null(rmu.MuCPtr))

    fn()

@may_spawn_proc
def test_rpython_dict_new_1():
    def new_1():
        a = {0: 42}

    fn, (db, bdlgen) = fncptr_from_rpy_func(new_1, [], lltype.Void)
    bdlgen.mu.current_thread_as_mu_thread(rmu.null(rmu.MuCPtr))

    fn()

@may_spawn_proc
def test_rpython_dict_new_100():
    def new_100():
        a = {}
        for i in range(0, 100):
            a[i] = i

    fn, (db, bdlgen) = fncptr_from_rpy_func(new_100, [], lltype.Void)
    bdlgen.mu.current_thread_as_mu_thread(rmu.null(rmu.MuCPtr))

    fn()


@may_spawn_proc
def test_rpython_dict_lookup():
    def test_lookup():
        a = {0: 42}
        return a[0]

    fn, (db, bdlgen) = fncptr_from_rpy_func(test_lookup, [], lltype.Signed)
    bdlgen.mu.current_thread_as_mu_thread(rmu.null(rmu.MuCPtr))

    assert fn() == 42

@may_spawn_proc
def test_rpython_dict_update():
    def test_update():
        a = {0: 42}
        a[0] = 43
        return a[0]

    fn, (db, bdlgen) = fncptr_from_rpy_func(test_update, [], lltype.Signed)
    bdlgen.mu.current_thread_as_mu_thread(rmu.null(rmu.MuCPtr))

    assert fn() == 43