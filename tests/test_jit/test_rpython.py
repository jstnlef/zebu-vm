from rpython.rtyper.lltypesystem import rffi, lltype
from rpython.translator.interactive import Translation
import ctypes


def getfncptr(entry_fnc, argtypes):
    t = Translation(entry_fnc, argtypes,
                    backend='mu', muimpl='fast', mucodegen='api', mutestjit=True)
    db, bdlgen, fnc_name = t.compile_mu()
    bdlgen.mu.compile_to_sharedlib('libtesting.dylib', [])
    lib = ctypes.CDLL('emit/libtesting.dylib')
    fnp = getattr(lib, fnc_name)
    return fnp


def test_add():
    def add(a, b):
        return a + b

    fn = getfncptr(add, [rffi.LONGLONG, rffi.LONGLONG])
    assert fn(1, 2) == 3


def test_find_min():
    def find_min(xs, sz):
        m = xs[0]
        for i in range(1, sz):
            x = xs[i]
            if x < m:
                m = x
        return m

    fnc = getfncptr(find_min, [rffi.CArrayPtr(rffi.LONGLONG), rffi.UINTPTR_T])

    arr = (ctypes.c_longlong * 5)(23, 100, 0, 78, -5)
    assert fnc(ctypes.byref(arr), 5) == -5


def test_quicksort():
    # algorithm taken from Wikipedia
    def swap(arr, i, j):
        t = arr[i]
        arr[i] = arr[j]
        arr[j] = t

    def partition(arr, idx_low, idx_high):
        pivot = arr[idx_high]
        i = idx_low
        for j in range(idx_low, idx_high):
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

    fnc = getfncptr(quicksort, [rffi.CArrayPtr(rffi.LONGLONG), rffi.UINTPTR_T, rffi.UINTPTR_T])

    from random import getrandbits
    from struct import pack, unpack
    n = 20
    lst = [unpack('i', pack('I', getrandbits(32)))[0] for i in range(n)]
    arr = (ctypes.c_longlong * n)(*lst)
    fnc(ctypes.byref(arr), 0, n - 1)    # inplace sort
    lst_s = sorted(lst)
    for i in range(n):
        assert lst_s[i] == arr[i]

if __name__ == '__main__':
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument('testfnc', help="Test function name")
    opts = parser.parse_args()

    globals()[opts.testfnc]()
