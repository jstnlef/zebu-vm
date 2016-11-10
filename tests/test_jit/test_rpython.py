from rpython.rtyper.lltypesystem import rffi, lltype
from rpython.rlib import rmu_fast as rmu
from rpython.translator.interactive import Translation
import ctypes, sys


def getfncptr(entry_fnc, argtypes, **kwargs):
    kwargs.setdefault('backend', 'mu')
    kwargs.setdefault('muimpl', 'fast')
    kwargs.setdefault('mucodegen', 'api')
    kwargs.setdefault('mutestjit', True)

    t = Translation(entry_fnc, argtypes, **kwargs)
    if kwargs['backend'] == 'mu':
        db, bdlgen, fnc_name = t.compile_mu()
        bdlgen.mu.compile_to_sharedlib('libtesting.dylib', [])
        lib = ctypes.CDLL('emit/libtesting.dylib')
        fnp = getattr(lib, fnc_name)
        return fnp
    else:
        libpath = t.compile_c()
        return getattr(ctypes.CDLL(libpath.strpath), 'pypy_g_' + entry_fnc.__name__)


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


def rand_array_of(n):
    from random import getrandbits
    from struct import pack, unpack

    lst = [rffi.r_longlong(unpack('i', pack('I', getrandbits(32)))[0]) for i in range(n)]
    arr = (ctypes.c_longlong * n)()
    for i in range(n):
        arr[i] = lst[i]
    return arr, lst


def test_arraysum():
    from rpython.rlib.jit import JitDriver
    d = JitDriver(greens=[], reds='auto')
    def arraysum(arr, sz):
        sum = 0
        for i in range(sz):
            d.jit_merge_point()
            sum += arr[i]
        return sum

    fnc = getfncptr(arraysum, [rffi.CArrayPtr(rffi.LONGLONG), rffi.SIZE_T])
    # fnc = getfncptr(arraysum, [rffi.CArrayPtr(rffi.LONGLONG), rffi.SIZE_T],
    #                 backend='c', jit=False, gc='none')

    n = 1000000
    arr, lst = rand_array_of(n)

    import time
    tmr = time.time
    t0 = tmr()
    fnc(ctypes.pointer(arr), n)  # inplace sort
    t1 = tmr()
    print "took %f sec" % (t1 - t0)


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

    # fnc = getfncptr(quicksort, [rffi.CArrayPtr(rffi.LONGLONG), rffi.UINTPTR_T, rffi.UINTPTR_T],
    #                 backend='c', jit=False, gc='none')
    fnc = getfncptr(quicksort, [rffi.CArrayPtr(rffi.LONGLONG), rffi.UINTPTR_T, rffi.UINTPTR_T])

    n = 1000000
    arr, lst = rand_array_of(n)

    import time
    tmr = time.time
    t0 = tmr()
    fnc(ctypes.pointer(arr), 0, n - 1)    # inplace sort
    t1 = tmr()
    print "took %f sec" % (t1 - t0)

    lst_s = sorted(lst)
    for i in range(n):
        assert lst_s[i] == arr[i], "%d != %d" % (lst_s[i], arr[i])


def test_linkedlist_reversal():
    def reverse_linkedlist(head):
        h = head
        nxt = head.nxt
        while nxt:
            n = nxt.nxt
            nxt.nxt = h
            h = nxt
            nxt = n
        head.nxt = nxt
        return h

    Node = lltype.ForwardReference()
    NodePtr = lltype.Ptr(Node)
    Node.become(lltype.Struct("Node", ('val', rffi.CHAR), ('nxt', NodePtr)))

    # RPython RFFI
    t = Translation(reverse_linkedlist, [NodePtr],
                    backend='mu', muimpl='fast', mucodegen='api', mutestjit=True)
    db, bdlgen, fnc_name = t.compile_mu()
    bdlgen.mu.compile_to_sharedlib('libtesting.dylib', [])

    c_fnc = rffi.llexternal('reverse_linkedlist', [NodePtr], NodePtr,
                            compilation_info=rffi.ExternalCompilationInfo(libraries=['libtesting.dylib'],
                                                                          library_dirs=['emit']), _nowrapper=True)

    # linked list: a -> b -> c -> d
    with lltype.scoped_alloc(Node) as a:
        a.val = 'a'
        with lltype.scoped_alloc(Node) as b:
            a.nxt = b
            b.val = 'b'
            with lltype.scoped_alloc(Node) as c:
                b.nxt = c
                c.val = 'c'
                with lltype.scoped_alloc(Node) as d:
                    c.nxt = d
                    d.val = 'd'
                    d.nxt = lltype.nullptr(Node)

                    h = c_fnc(a)
                    assert h.val == 'd'
                    assert h.nxt.val == 'c'
                    assert h.nxt.nxt.val == 'b'
                    assert h.nxt.nxt.nxt.val == 'a'
                    assert h.nxt.nxt.nxt.nxt == lltype.nullptr(Node)

    # # ctypes
    # fnc = getfncptr(reverse_linkedlist, [NodePtr])
    #
    # class cNode(ctypes.Structure):
    #     pass
    # cNodePtr = ctypes.POINTER(cNode)
    # cNode._fields_ = [('val', ctypes.c_char),
    #                   ('nxt', cNodePtr)]
    #
    # # fnc.argtypes = [cNodePtr]
    # # fnc.restype = [cNodePtr]
    #
    # # ctypes
    # # linked list: a -> b -> c -> d
    # a = cNode()
    # a.val = 'a'
    # b = cNode()
    # b.val = 'b'
    # a.nxt = ctypes.pointer(b)
    # c = cNode()
    # c.val = 'c'
    # b.nxt = ctypes.pointer(c)
    # d = cNode()
    # d.val = 'd'
    # c.nxt = ctypes.pointer(d)
    # d.nxt = None
    #
    # h = fnc(a)
    #
    # assert h.val == 'd'
    # assert h.nxt.val == 'c'
    # assert h.nxt.nxt.val == 'b'
    # assert h.nxt.nxt.nxt.val == 'a'
    # assert h.nxt.nxt.nxt.nxt == None


def test_new():
    def build_test_bundle(bldr, rmu):
        """
        Builds the following test bundle.
            .typedef @i64 = int<64>
            .typedef @refi64 = ref<@i64>
            .const @NULL_refi64 <@refi64> = NULL
            .funcsig @sig__i64 = () -> (@i64)
            .funcdef @test_fnc VERSION @test_fnc.v1 <@sig__i64> {
                @test_fnc.v1.blk0():
                    @test_fnc.v1.blk0.r = NEW <@i64>
                    @test_fnc.v1.blk0.cmpres = EQ <@refi64> @test_fnc.v1.blk0.r @NULL_refi64
                    @@test_fnc.v1.blk0.res = ZEXT <@i1 @i64> @test_fnc.v1.blk0.cmpres
                    RET @test_fnc.v1.blk0.res
            }
        :type bldr: rpython.rlib.rmu.MuIRBuilder
        :type rmu: rpython.rlib.rmu_fast
        :return: (rmu.MuVM(), rmu.MuCtx, rmu.MuIRBuilder, MuID, MuID)
        """
        i1 = bldr.gen_sym("@i1")
        bldr.new_type_int(i1, 1)
        i64 = bldr.gen_sym("@i64")
        bldr.new_type_int(i64, 64)
        refi64 = bldr.gen_sym("@refi64")
        bldr.new_type_ref(refi64, i64)

        NULL_refi64 = bldr.gen_sym("@NULL_refi64")
        bldr.new_const_null(NULL_refi64, refi64)

        sig__i64 = bldr.gen_sym("@sig__i64")
        bldr.new_funcsig(sig__i64, [], [i64])

        test_fnc = bldr.gen_sym("@test_fnc")
        bldr.new_func(test_fnc, sig__i64)

        test_fnc_v1 = bldr.gen_sym("@test_fnc.v1")
        blk0 = bldr.gen_sym("@test_fnc.v1.blk0")
        r = bldr.gen_sym("@test_fnc.v1.blk0.r")
        cmpres = bldr.gen_sym("@test_fnc.v1.blk0.cmpres")
        res = bldr.gen_sym("@test_fnc.v1.blk0.res")
        op_new = bldr.gen_sym()
        bldr.new_new(op_new, r, i64)
        op_eq = bldr.gen_sym()
        bldr.new_cmp(op_eq, cmpres, rmu.MuCmpOptr.EQ, refi64, r, NULL_refi64)
        op_zext = bldr.gen_sym()
        bldr.new_conv(op_zext, res, rmu.MuConvOptr.ZEXT, i1, i64, cmpres)
        op_ret = bldr.gen_sym()
        bldr.new_ret(op_ret, [res])
        bldr.new_bb(blk0, [], [], rmu.MU_NO_ID, [op_new, op_eq, op_zext, op_ret])

        bldr.new_func_ver(test_fnc_v1, test_fnc, [blk0])

        return {
            "test_fnc": test_fnc,
            "test_fnc_sig": sig__i64,
            "result_type": i64,
            "@i64": i64
        }

    mu = rmu.MuVM()
    ctx = mu.new_context()
    bldr = ctx.new_ir_builder()

    id_dict = build_test_bundle(bldr, rmu)
    bldr.load()
    mu.compile_to_sharedlib('libtesting.dylib', [])

    lib = ctypes.CDLL('emit/libtesting.dylib')
    fnp = lib.test_fnc

    mu.current_thread_as_mu_thread(rmu.null(rmu.MuCPtr))
    assert fnp() == 0

if __name__ == '__main__':
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument('testfnc', help="Test function name")
    opts = parser.parse_args()

    globals()[opts.testfnc]()
