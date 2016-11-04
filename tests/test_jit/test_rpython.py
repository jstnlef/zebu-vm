from rpython.rtyper.lltypesystem import rffi, lltype
from rpython.translator.interactive import Translation
import ctypes


def test_add():
    def add(a, b):
        return a + b

    t = Translation(add, [rffi.ULONGLONG, rffi.ULONGLONG],
                    backend='mu', muimpl='fast', mucodegen='api', mutestjit=True)
    db, bdlgen, fnc_name = t.compile_mu()
    bdlgen.mu.compile_to_sharedlib('libtesting.dylib', [])
    lib = ctypes.CDLL('emit/libtesting.dylib')
    fnc = getattr(lib, fnc_name)
    assert fnc(1, 2) == 3


def test_find_min():
    def find_min(xs, sz):
        m = xs[0]
        for i in range(1, sz):
            x = xs[i]
            if x < m:
                m = x
        return m

    t = Translation(find_min, [rffi.CArrayPtr(rffi.LONGLONG), rffi.INT],
                    backend='mu', muimpl='fast', mucodegen='api', mutestjit=True)

    db, bdlgen, fnc_name = t.compile_mu()
    bdlgen.mu.compile_to_sharedlib('libtesting.dylib', [])
    lib = ctypes.CDLL('emit/libtesting.dylib')
    fnc = getattr(lib, fnc_name)

    lst = [23, 100, 0, 78, -5]
    arr = lltype.malloc(rffi.CArray(rffi.LONGLONG), 5, flavor='raw')
    for i, n in enumerate(lst):
        arr[i] = n

    assert fnc(arr, 5) == -5
    lltype.free(arr, flavor='raw')

if __name__ == '__main__':
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument('testfnc', help="Test function name")
    opts = parser.parse_args()

    globals()[opts.testfnc]()
