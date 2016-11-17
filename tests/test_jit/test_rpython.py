from rpython.rtyper.lltypesystem import rffi, lltype
from rpython.rlib import rmu_fast as rmu
from util import fncptr_from_rpy_func, fncptr_from_py_script, proc_call, call_and_check
import ctypes


# -------------------
# helper functions
def rand_list_of(n):
    from random import getrandbits
    from struct import pack, unpack

    lst = []
    for i in range(n):
        lst.append(rffi.r_longlong(unpack('i', pack('I', getrandbits(32)))[0]))
    return lst


# --------------------------
# tests
def test_add():
    def add(a, b):
        return a + b

    fn, _ = fncptr_from_rpy_func(add, [rffi.LONGLONG, rffi.LONGLONG], rffi.LONGLONG)

    def check(s):
        assert s == 3
    call_and_check(fn, (1, 2), check)


def test_vec3prod():
    def prod(v1, v2):
        a = v1[0] * v2[0]
        b = v1[1] * v2[1]
        c = v1[2] * v2[2]
        return a + b + c

    fnc, (db, bdlgen) = fncptr_from_rpy_func(prod, [rffi.CArrayPtr(rffi.LONGLONG), rffi.CArrayPtr(rffi.LONGLONG)], rffi.LONGLONG)
    bdlgen.mu.current_thread_as_mu_thread(rmu.null(rmu.MuCPtr))
    with lltype.scoped_alloc(rffi.CArray(rffi.LONGLONG), 3) as vec1:
        vec1[0] = 1
        vec1[1] = 2
        vec1[2] = 3
        with lltype.scoped_alloc(rffi.CArray(rffi.LONGLONG), 3) as vec2:
            vec2[0] = 4
            vec2[1] = 5
            vec2[2] = 6

            def check(s):
                assert s == 32
            call_and_check(fnc, (vec1, vec2), check)


def test_find_min():
    def find_min(xs, sz):
        m = xs[0]
        for i in range(1, sz):
            x = xs[i]
            if x < m:
                m = x
        return m

    fnc, (db, bdlgen) = fncptr_from_rpy_func(find_min, [rffi.CArrayPtr(rffi.LONGLONG), rffi.INTPTR_T], rffi.LONGLONG)
    bdlgen.mu.current_thread_as_mu_thread(rmu.null(rmu.MuCPtr))
    with lltype.scoped_alloc(rffi.CArray(rffi.LONGLONG), 5) as arr:
        lst = [23, 100, 0, 78, -5]
        for i, k in enumerate(lst):
            arr[i] = k

        def check(m):
            assert m == -5
        call_and_check(fnc, (arr, 5), check)


def test_arraysum():
    from rpython.rlib.jit import JitDriver
    d = JitDriver(greens=[], reds='auto')
    def arraysum(arr, sz):
        sum = 0
        for i in range(sz):
            d.jit_merge_point()
            sum += arr[i]
        return sum

    fnc, (db, bdlgen) = fncptr_from_rpy_func(arraysum, [rffi.CArrayPtr(rffi.LONGLONG), rffi.SIZE_T], rffi.LONGLONG)
    bdlgen.mu.current_thread_as_mu_thread(rmu.null(rmu.MuCPtr))

    n = 100
    lst = rand_list_of(n)
    with lltype.scoped_alloc(rffi.CArray(rffi.LONGLONG), n) as arr:
        for i, k in enumerate(lst):
            arr[i] = k

        def check(s):
            assert s == sum(lst)
        call_and_check(fnc, (arr, rffi.cast(rffi.SIZE_T, n)), check)


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

    fnc, (db, bdlgen) = fncptr_from_rpy_func(quicksort, [rffi.CArrayPtr(rffi.LONGLONG), rffi.SIZE_T, rffi.SIZE_T], lltype.Void)
    bdlgen.mu.current_thread_as_mu_thread(rmu.null(rmu.MuCPtr))
    # fnc = quicksort

    n = 100
    lst = rand_list_of(n)
    with lltype.scoped_alloc(rffi.CArray(rffi.LONGLONG), n) as arr:
        for i, k in enumerate(lst):
            arr[i] = k

        def check():
            lst_s = sorted(lst)
            for i in range(n):
                assert lst_s[i] == arr[i], "%d != %d" % (lst_s[i], arr[i])
        call_and_check(fnc, (arr, rffi.cast(rffi.SIZE_T, 0), rffi.cast(rffi.SIZE_T, n - 1)), check)


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

    fnc, (db, bdlgen) = fncptr_from_rpy_func(reverse_linkedlist, [NodePtr], NodePtr)
    bdlgen.mu.current_thread_as_mu_thread(rmu.null(rmu.MuCPtr))
    # fnc = reverse_linkedlist

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

                    def check(h):
                        print '%s -> %s -> %s -> %s' % (h.val, h.nxt.val, h.nxt.nxt.val, h.nxt.nxt.nxt.val)
                        assert h.val == 'd'
                        assert h.nxt.val == 'c'
                        assert h.nxt.nxt.val == 'b'
                        assert h.nxt.nxt.nxt.val == 'a'
                        assert h.nxt.nxt.nxt.nxt == lltype.nullptr(Node)
                    call_and_check(fnc, (a,), check)


def test_threadtran_fib():
    def build_test_bundle(bldr, rmu):
        """
        Builds the following test bundle.
            .typedef @i64 = int<64>
            .const @0_i64 <@i64> = 0
            .const @1_i64 <@i64> = 1
            .const @2_i64 <@i64> = 2
            .funcsig @sig_i64_i64 = (@i64) -> (@i64)
            .funcdef @fib VERSION @fib_v1 <@sig_i64_i64> {
                @fib_v1.blk0(<@i64> @fib_v1.blk0.k):
                    SWITCH <@i64> @fib_v1.blk0.k @fib_v1.blk2 (@fib_v1.blk0.k) {
                        @0_i64 @fib_v1.blk1 (@0_i64)
                        @1_i64 @fib_v1.blk1 (@1_i64)
                    }
                @fib_v1.blk1(<@i64> @fib_v1.blk1.rtn):
                    RET @fib_v1.blk1.rtn
                @fib_v1.blk2(<@i64> @fib_v1.blk1.k):
                    @fib_v1.blk2.k_1 = SUB <@i64> @fib_v1.blk2.k @1_i64
                    @fib_v1.blk2.res1 = CALL <@sig_i64_i64> @fib (@fib_v1.blk2.k_1)
                    @fib_v1.blk2.k_2 = SUB <@i64> @fib_v1.blk2.k @2_i64
                    @fib_v1.blk2.res2 = CALL <@sig_i64_i64> @fib (@fib_v1.blk2.k_2)
                    @fib_v1.blk2.res = ADD <@i64> @fib_v1.blk2.res1 @fib_v1.blk2.res2
                    RET @fib_v1.blk2.res2
            }
        :type bldr: rpython.rlib.rmu.MuIRBuilder
        :type rmu: rpython.rlib.rmu_fast
        :return: (rmu.MuVM(), rmu.MuCtx, rmu.MuIRBuilder, MuID, MuID)
        """
        i64 = bldr.gen_sym("@i64")
        bldr.new_type_int(i64, 64)

        c_0_i64 = bldr.gen_sym("@0_i64")
        bldr.new_const_int(c_0_i64, i64, 0)
        c_1_i64 = bldr.gen_sym("@1_i64")
        bldr.new_const_int(c_1_i64, i64, 1)
        c_2_i64 = bldr.gen_sym("@2_i64")
        bldr.new_const_int(c_2_i64, i64, 2)

        sig_i64_i64 = bldr.gen_sym("@sig_i64_i64")
        bldr.new_funcsig(sig_i64_i64, [i64], [i64])

        fib = bldr.gen_sym("@fib")
        bldr.new_func(fib, sig_i64_i64)

        # function body
        v1 = bldr.gen_sym("@fib_v1")
        blk0 = bldr.gen_sym("@fib_v1.blk0")
        blk1 = bldr.gen_sym("@fib_v1.blk1")
        blk2 = bldr.gen_sym("@fib_v1.blk2")

        # blk0
        blk0_k = bldr.gen_sym("@fib_v1.blk0.k")
        dest_defl = bldr.gen_sym()
        dest_0 = bldr.gen_sym()
        dest_1 = bldr.gen_sym()
        bldr.new_dest_clause(dest_defl, blk2, [blk0_k])
        bldr.new_dest_clause(dest_0, blk1, [c_0_i64])
        bldr.new_dest_clause(dest_1, blk1, [c_1_i64])
        op_switch = bldr.gen_sym()
        bldr.new_switch(op_switch, i64, blk0_k, dest_defl, [c_0_i64, c_1_i64], [dest_0, dest_1])
        bldr.new_bb(blk0, [blk0_k], [i64], rmu.MU_NO_ID, [op_switch])

        # blk1
        blk1_rtn = bldr.gen_sym("@fig_v1.blk1.rtn")
        blk1_op_ret = bldr.gen_sym()
        bldr.new_ret(blk1_op_ret, [blk1_rtn])
        bldr.new_bb(blk1, [blk1_rtn], [i64], rmu.MU_NO_ID, [blk1_op_ret])

        # blk2
        blk2_k = bldr.gen_sym("@fig_v1.blk2.k")
        blk2_k_1 = bldr.gen_sym("@fig_v1.blk2.k_1")
        blk2_k_2 = bldr.gen_sym("@fig_v1.blk2.k_2")
        blk2_res = bldr.gen_sym("@fig_v1.blk2.res")
        blk2_res1 = bldr.gen_sym("@fig_v1.blk2.res1")
        blk2_res2 = bldr.gen_sym("@fig_v1.blk2.res2")
        op_sub_1 = bldr.gen_sym()
        bldr.new_binop(op_sub_1, blk2_k_1, rmu.MuBinOptr.SUB, i64, blk2_k, c_1_i64)
        op_call_1 = bldr.gen_sym()
        bldr.new_call(op_call_1, [blk2_res1], sig_i64_i64, fib, [blk2_k_1])
        op_sub_2 = bldr.gen_sym()
        bldr.new_binop(op_sub_2, blk2_k_2, rmu.MuBinOptr.SUB, i64, blk2_k, c_2_i64)
        op_call_2 = bldr.gen_sym()
        bldr.new_call(op_call_2, [blk2_res2], sig_i64_i64, fib, [blk2_k_2])
        op_add = bldr.gen_sym()
        bldr.new_binop(op_add, blk2_res, rmu.MuBinOptr.ADD, i64, blk2_res1, blk2_res2)
        blk2_op_ret = bldr.gen_sym()
        bldr.new_ret(blk2_op_ret, [blk2_res])
        bldr.new_bb(blk2, [blk2_k], [i64], rmu.MU_NO_ID,
                    [op_sub_1, op_call_1, op_sub_2, op_call_2, op_add, blk2_op_ret])
        bldr.new_func_ver(v1, fib, [blk0, blk1, blk2])

        return {
            "@i64": i64,
            "test_fnc_sig": sig_i64_i64,
            "test_fnc": fib,
            "result_type": i64
        }

    fnp, (mu, ctx, bldr) = fncptr_from_py_script(build_test_bundle, 'fib', [ctypes.c_longlong])

    mu.current_thread_as_mu_thread(rmu.null(rmu.MuCPtr))

    def check(res):
        assert res == 6765
    call_and_check(fnp, (20,), check)


def test_new():
    def build_test_bundle(bldr, rmu):
        """
        Builds the following test bundle.
            .typedef @i64 = int<64>
            .typedef @refi64 = ref<@i64>
            .const @1_i64 <@i64> = 1
            .const @NULL_refi64 <@refi64> = NULL
            .funcsig @sig__i64 = () -> (@i64)
            .funcdef @test_fnc VERSION @test_fnc.v1 <@sig__i64> {
                %blk0():
                    %r = NEW <@i64>
                    %ir = GETIREF <@refi64> %r
                    STORE <@i64> %ir @1_i64
                    %res = LOAD <@i64> %ir
                    RET %res
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

        c_1_i64 = bldr.gen_sym("@1_64")
        bldr.new_const_int(c_1_i64, i64, 1)

        sig__i64 = bldr.gen_sym("@sig__i64")
        bldr.new_funcsig(sig__i64, [], [i64])

        test_fnc = bldr.gen_sym("@test_fnc")
        bldr.new_func(test_fnc, sig__i64)

        test_fnc_v1 = bldr.gen_sym("@test_fnc.v1")
        blk0 = bldr.gen_sym("@test_fnc.v1.blk0")
        r = bldr.gen_sym("@test_fnc.v1.blk0.r")
        ir = bldr.gen_sym("@test_fnc.v1.blk0.ir")
        res = bldr.gen_sym("@test_fnc.v1.blk0.res")
        op_new = bldr.gen_sym()
        bldr.new_new(op_new, r, i64)
        op_getiref = bldr.gen_sym()
        bldr.new_getiref(op_getiref, ir, refi64, r)
        op_store = bldr.gen_sym()
        bldr.new_store(op_store, False, rmu.MuMemOrd.NOT_ATOMIC, i64, ir, c_1_i64)
        op_load = bldr.gen_sym()
        bldr.new_load(op_load, res, False, rmu.MuMemOrd.NOT_ATOMIC, i64, ir)
        op_ret = bldr.gen_sym()
        bldr.new_ret(op_ret, [res])
        bldr.new_bb(blk0, [], [], rmu.MU_NO_ID, [op_new, op_getiref, op_store, op_load, op_ret])

        bldr.new_func_ver(test_fnc_v1, test_fnc, [blk0])

        return {
            "test_fnc": test_fnc,
            "test_fnc_sig": sig__i64,
            "result_type": i64,
            "@i64": i64
        }

    fnp, (mu, ctx, bldr) = fncptr_from_py_script(build_test_bundle, 'test_fnc')

    mu.current_thread_as_mu_thread(rmu.null(rmu.MuCPtr))
    def check(res):
        assert res == 1
    call_and_check(fnp, tuple(), check)


def test_new_cmpeq():
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

    fnp, (mu, ctx, bldr) = fncptr_from_py_script(build_test_bundle, 'test_fnc')

    mu.current_thread_as_mu_thread(rmu.null(rmu.MuCPtr))
    def check(res):
        assert res == 0
    call_and_check(fnp, tuple(), check)

if __name__ == '__main__':
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument('testfnc', help="Test function name")
    opts = parser.parse_args()

    globals()[opts.testfnc]()
