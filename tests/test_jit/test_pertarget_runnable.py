from rpython.rtyper.lltypesystem import rffi, lltype
from rpython.rlib import rmu_fast as rmu
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
