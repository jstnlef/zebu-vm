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
from util import fncptr_from_rpy_func, fncptr_from_py_script, may_spawn_proc, bin_dir
import ctypes, py, stat
import pytest

from test_rpython import run_boot_image

c_exit = rffi.llexternal('exit', [rffi.INT], lltype.Void, _nowrapper=True)

@may_spawn_proc
def test_rpython_list_new_empty():
    def new_empty():
        a = []
        return a

    fn, (db, bdlgen) = fncptr_from_rpy_func(new_empty, [], lltype.Void)
    bdlgen.mu.current_thread_as_mu_thread(rmu.null(rmu.MuCPtr))

    fn()

@may_spawn_proc
def test_rpython_list_new_5():
    def new_5():
        a = [1, 2, 3, 4, 5]
        return len(a)

    fn, (db, bdlgen) = fncptr_from_rpy_func(new_5, [], rffi.LONGLONG)
    bdlgen.mu.current_thread_as_mu_thread(rmu.null(rmu.MuCPtr))

    assert fn() == 5

@may_spawn_proc
def test_rpython_list_append():
    def list_append(n):
        a = []
        for i in range(0, n):
            a.append(i)
        return len(a)

    fn, (db, bdlgen) = fncptr_from_rpy_func(list_append, [rffi.LONGLONG], rffi.LONGLONG)
    bdlgen.mu.current_thread_as_mu_thread(rmu.null(rmu.MuCPtr))

    assert fn(5) == 5
    assert fn(10) == 10
    assert fn(100) == 100

@may_spawn_proc
def test_rpython_image_list_append():
    def main(argv):
        a = []
        for i in range(0, 10):
            a.append(i)
        c_exit(rffi.cast(rffi.INT, len(a)))
        return 0

    res = run_boot_image(main, str(bin_dir.join('test_rpython_image_list_append')))

    assert res.returncode == 10, 'returncode = %d\n%s' % (res.returncode, res.err)

@may_spawn_proc
def test_rpython_list_iter():
    def main(argv):
        a = []
        for i in range(0, 10):
            a.append(i)

        sum = 0
        for n in a:
            sum += n

        c_exit(rffi.cast(rffi.INT, sum))
        return 0

    res = run_boot_image(main, str(bin_dir.join('test_rpython_list_iter')))

    assert res.returncode == 45, 'returncode = %d\n%s' % (res.returncode, res.err)

@may_spawn_proc
def test_rpython_list_addr_check_length1():
    Int64Ptr = lltype.Ptr(lltype.Array(rffi.LONGLONG, hints={'nolength': True}))

    def check(actual, expect):
        if actual != expect:
            c_exit(rffi.cast(rffi.INT, actual))

    def main(argv):
        a = []
        a.append(42)

        from rpython.rtyper.lltypesystem.llmemory import cast_ptr_to_adr
        from rpython.rlib.objectmodel import keepalive_until_here

        addr = cast_ptr_to_adr(a)
        mem  = rffi.cast(Int64Ptr, addr)
        # ignore mem[0]
        check(mem[1], 1)
        keepalive_until_here(a)

        return 0

    res = run_boot_image(main, str(bin_dir.join('test_rpython_list_addr_check_length1')))

    assert res.returncode == 0, 'returncode = %d\n%s' % (res.returncode, res.err)

@may_spawn_proc
def test_rpython_list_addr_check_length2():
    Int64Ptr = lltype.Ptr(lltype.Array(rffi.LONGLONG, hints={'nolength': True}))

    def check(actual, expect):
        if actual != expect:
            c_exit(rffi.cast(rffi.INT, actual))

    def main(argv):
        a = []
        a.append(42)
        a.append(43)

        from rpython.rtyper.lltypesystem.llmemory import cast_ptr_to_adr
        from rpython.rlib.objectmodel import keepalive_until_here

        addr = cast_ptr_to_adr(a)
        mem  = rffi.cast(Int64Ptr, addr)
        # ignore mem[0]
        check(mem[1], 2)
        keepalive_until_here(a)

        return 0

    res = run_boot_image(main, str(bin_dir.join('test_rpython_list_addr_check_length2')))

    assert res.returncode == 0, 'returncode = %d\n%s' % (res.returncode, res.err)

@may_spawn_proc
def test_rpython_list_addr_check_length100():
    Int64Ptr = lltype.Ptr(lltype.Array(rffi.LONGLONG, hints={'nolength': True}))

    def check(actual, expect):
        if actual != expect:
            c_exit(rffi.cast(rffi.INT, actual))

    def main(argv):
        a = []
        for i in range(0, 100):
            a.append(i)

        from rpython.rtyper.lltypesystem.llmemory import cast_ptr_to_adr
        from rpython.rlib.objectmodel import keepalive_until_here

        addr = cast_ptr_to_adr(a)
        mem  = rffi.cast(Int64Ptr, addr)
        # ignore mem[0]
        check(mem[1], 100)
        keepalive_until_here(a)

        return 0

    res = run_boot_image(main, str(bin_dir.join('test_rpython_list_addr_check_length2')))

    assert res.returncode == 0, 'returncode = %d\n%s' % (res.returncode, res.err)

@may_spawn_proc
def test_rpython_list_addr_check_all10():
    Int64Ptr = lltype.Ptr(lltype.Array(rffi.LONGLONG, hints={'nolength': True}))

    N = 10

    def check(actual, expect):
        if actual != expect:
            c_exit(rffi.cast(rffi.INT, actual))

    def main(argv):
        a = []
        for i in range(0, N):
            a.append(i)

        from rpython.rtyper.lltypesystem.llmemory import cast_ptr_to_adr
        from rpython.rlib.objectmodel import keepalive_until_here

        addr = cast_ptr_to_adr(a)
        mem  = rffi.cast(Int64Ptr, addr)
        # ignore mem[0]
        check(mem[1], N)

        inner_addr = mem[2]
        inner = rffi.cast(Int64Ptr, inner_addr)
        # inner[0], inner[1] is ignored
        for i in range(0, N):
            check(inner[2 + i], i)

        keepalive_until_here(a)

        return 0

    res = run_boot_image(main, str(bin_dir.join('test_rpython_list_addr_check_all10')))

    assert res.returncode == 0, 'returncode = %d\n%s' % (res.returncode, res.err)

@may_spawn_proc
def test_rpython_list_addr_check_all100():
    Int64Ptr = lltype.Ptr(lltype.Array(rffi.LONGLONG, hints={'nolength': True}))

    N = 100

    def check(actual, expect):
        if actual != expect:
            c_exit(rffi.cast(rffi.INT, actual))

    def main(argv):
        a = []
        for i in range(0, N):
            a.append(i)

        from rpython.rtyper.lltypesystem.llmemory import cast_ptr_to_adr
        from rpython.rlib.objectmodel import keepalive_until_here

        addr = cast_ptr_to_adr(a)
        mem  = rffi.cast(Int64Ptr, addr)
        # ignore mem[0]
        check(mem[1], N)

        inner_addr = mem[2]
        inner = rffi.cast(Int64Ptr, inner_addr)
        # inner[0], inner[1] is ignored
        for i in range(0, N):
            check(inner[2 + i], i)

        keepalive_until_here(a)

        return 0

    res = run_boot_image(main, str(bin_dir.join('test_rpython_list_addr_check_all100')))

    assert res.returncode == 0, 'returncode = %d\n%s' % (res.returncode, res.err)
