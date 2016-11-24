from rpython.rtyper.lltypesystem import lltype, rffi
from rpython.rlib.jit import JitDriver
d = JitDriver(greens=[], reds='auto')


# algorithm taken from Wikipedia
def swap(arr, i, j):
    t = arr[i]
    arr[i] = arr[j]
    arr[j] = t


def partition(arr, idx_low, idx_high):
    pivot = arr[idx_high]
    i = idx_low
    for j in range(idx_low, idx_high):
        d.jit_merge_point()
        if arr[j] < pivot:
            swap(arr, i, j)
            i += 1
    swap(arr, i, idx_high)
    return i


def quicksort(arr, start, end):
    if start < end:
        p = partition(arr, start, end)
        quicksort(arr, start, p - 1)
        quicksort(arr, p + 1, end)


def setup(n):
    lst, _ = rand_list_of(n)
    arr = lltype.malloc(rffi.CArray(rffi.LONGLONG), n, flavor='raw')
    for i, k in enumerate(lst):
        arr[i] = k
    return arr, 0, n - 1


def teardown(arr, s, e):
    lltype.free(arr, 'raw')


def rand_list_of(n):
    # 32 extend to 64-bit integers (to avoid overflow in summation
    from random import randrange, getstate
    init_state = getstate()
    return [rffi.r_longlong(randrange(-(1 << 31), (1 << 31) - 1)) for _ in range(n)], init_state


def measure(N):
    args = setup(N)
    from time import time
    t0 = time()
    quicksort(*args)
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