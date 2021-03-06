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
from util import fncptr_from_rpy_func, fncptr_from_py_script, may_spawn_proc
import ctypes


@may_spawn_proc
def test_fibonacci():
    from perftarget.fibonacci import fib
    fnc, (db, bdlgen) = fncptr_from_rpy_func(fib, [rffi.ULONGLONG], rffi.ULONGLONG)
    bdlgen.mu.current_thread_as_mu_thread(rmu.null(rmu.MuCPtr))

    fnc(rffi.cast(rffi.ULONGLONG, 20)) == 6765

@may_spawn_proc
def test_fibonacci_iterative():
    def fib_iter(n):
        if n <= 1:
            return n
        k = 2
        fib_k_2 = 0
        fib_k_1 = 1
        fib_k = 0
        while k <= n:
            fib_k = fib_k_2 + fib_k_1
            fib_k_2 = fib_k_1
            fib_k_1 = fib_k
            k += 1
        return fib_k

    fnc, (db, bdlgen) = fncptr_from_rpy_func(fib_iter, [rffi.ULONGLONG], rffi.ULONGLONG)
    bdlgen.mu.current_thread_as_mu_thread(rmu.null(rmu.MuCPtr))

    assert fnc(rffi.cast(rffi.ULONGLONG, 20))


@may_spawn_proc
def test_quicksort():
    from perftarget.quicksort import quicksort, setup, teardown
    fnc, (db, bdlgen) = fncptr_from_rpy_func(quicksort,
                                             [rffi.CArrayPtr(rffi.LONGLONG), lltype.Signed, lltype.Signed],
                                             lltype.Void)
    bdlgen.mu.current_thread_as_mu_thread(rmu.null(rmu.MuCPtr))

    N = 100
    arr, s, e = setup(N)
    lst = list(arr)
    fnc(arr, s, e)
    lst.sort()
    for i in range(len(lst)):
        assert lst[i] == arr[i]
    teardown(arr, s, e)


@may_spawn_proc
def test_quicksort_handcraft():
    from perftarget.quicksort import build_quicksort_bundle, setup, teardown
    fnc, (mu, ctx, bldr) = fncptr_from_py_script(build_quicksort_bundle, None, 'quicksort',
                                                 [rffi.CArrayPtr(rffi.LONGLONG), lltype.Signed, lltype.Signed],
                                                 lltype.Void)
    mu.current_thread_as_mu_thread(rmu.null(rmu.MuCPtr))

    N = 100
    arr, s, e = setup(N)
    lst = list(arr)
    fnc(arr, s, e)
    lst.sort()
    for i in range(len(lst)):
        assert lst[i] == arr[i]
    teardown(arr, s, e)
