from rpython.rtyper.lltypesystem import lltype, rffi
from rpython.rlib.jit import JitDriver


d = JitDriver(greens=[], reds='auto')
def arraysum(arr, sz):
    sum = rffi.r_longlong(0)
    for i in range(sz):
        d.jit_merge_point()
        sum += arr[i]
    return sum


def setup(n):
    lst, _ = rand_list_of(n)
    arr = lltype.malloc(rffi.CArray(rffi.LONGLONG), n, flavor='raw')
    for i, k in enumerate(lst):
        arr[i] = k
    return rffi.ll2ctypes.lltype2ctypes(arr) , n


def teardown(carr, n):
    lltype.free(rffi.ll2ctypes.ctypes2lltype(rffi.CArray(rffi.LONGLONG), carr), 'raw')


def rand_list_of(n):
    # 32 extend to 64-bit integers (to avoid overflow in summation
    from random import randrange, getstate
    init_state = getstate()
    return [rffi.r_longlong(randrange(-(1 << 31), (1 << 31) - 1)) for _ in range(n)], init_state


def measure(N):
    args = setup(N)
    from time import time
    t0 = time()
    arraysum(*args)
    t1 = time()
    teardown(*args)
    return t0, t1


def rpy_entry(N):
    t0, t1 = measure(N)
    # from rpython.rlib import rfloat
    # print rfloat.double_to_string(t1 - t0, 'e', %(fprec)d, rfloat.DTSF_ADD_DOT_0)
    return t1 - t0

if __name__ == '__main__':
    import sys
    t0, t1 = measure(int(sys.argv[1]))
    print '%.15f' % (t1 - t0)


def target(*args):
    from rpython.rlib.entrypoint import export_symbol
    export_symbol(rpy_entry)
    return rpy_entry, [int]