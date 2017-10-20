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


def build_quicksort_bundle(bldr, rmu):
    """
    Builds the following test bundle.
        .typedef @i64 = int<64>
        .typedef @hi64 = hybrid<@i64>
        .typedef @phi64 = uptr<@hi64>
        .typedef @c_1 <@i64> = 1
        .funcsig @sig_quicksort = (@phi64 @i64 @i64) -> ()
        .funcsig @sig_partition = (@phi64 @i64 @i64) -> (@i64)
        .funcdef @quicksort VERSION @quicksort.v1 <@sig_quicksort> {
            %blk0(<@phi64> %parr <@i64> %start <@i64> %end):
                %cmpres = SLT <@i64> %start %end
                BRANCH2 %cmpres %blk2(%parr %start %end) %blk1()
            %blk1():
                RET ()
            %blk2(<@phi64> %parr <@i64> %start <@i64> %end):
                %p = CALL <@sig_partition> @partition (%parr %start %end)
                %ps1 = SUB <@i64> %p @c_1
                CALL <@sig_quicksort> @quicksort (%parr %start %ps1)
                %pp1 = ADD <@i64> %p @c_1
                CALL <@sig_quicksort> @quicksort (%parr %pp1 %end)
                BRANCH %blk1()
        }
        .funcdef @partition VERSION @partition.v1 <@sig_partition> {
            %blk0(<@phi64> %parr <@i64> %idx_low <@i64> %idx_high):
                %pelm = GETVARPARTIREF PTR <@hi64> %parr
                %pelm_idx_high = SHIFTIREF PTR <@i64 @i64> %pelm %idx_high
                %pivot = LOAD PTR <@i64> %pelm_idx_high
                BRANCH %blk1(%parr %idx_high %pivot %idx_low %idx_low %idx_high)

            %blk1(<@phi64> %parr  <@i64> %idx_high  <@i64> %pivot  <@i64> %i  <@i64> %j  <@i64> %end):
                %cmpres = SGE <@i64> %j %end
                BRANCH2 %cmpres %blk4(%i %parr %idx_high)
                                %blk2(%end %j %i %parr %idx_high %pivot)

            %blk2(<@i64> %end  <@i64> %j  <@i64> %i  <@phi64> %parr  <@i64> %idx_high  <@i64> %pivot):
                %jp1 = ADD <@i64> %j @c_1
                %pelm = GETVARPARTIREF PTR <@hi64> %parr
                %pelm_j = SHIFTIREF PTR <@i64 @i64> %pelm %j
                %elm_j = LOAD PTR <@i64> %pelm_j
                %cmpres = SLT <@i64> %elm_j %pivot
                BRANCH2 %cmpres %blk3(%jp1 %end %pivot %idx_high %j %i %parr)
                                %blk1(%parr %idx_high %pivot %i %jp1 %end)

            %blk3(<@i64> %jp1  <@i64> %end  <@i64> %pivot  <@i64> %idx_high  <@i64> %j  <@i64> %i  <@phi64> %parr):
                %pelm = GETVARPARTIREF PTR <@hi64> %parr
                %pelm_i = SHIFTIREF PTR <@i64 @i64> %pelm %i
                %t = LOAD PTR <@i64> %pelm_i
                %pelm_j = SHIFTIREF PTR <@i64 @i64> %pelm %j
                %elm_j = LOAD PTR <@i64> %pelm_j
                STORE PTR <@i64> %pelm_i %elm_j
                STORE PTR <@i64> %pelm_j %t
                %ip1 = ADD  <@i64> %i @c_1
                BRANCH %blk1(%parr %idx_high %pivot %ip1 %jp1 %end)

            %blk4(<@i64> %i  <@phi64> %parr  <@i64> %idx_high):
                %pelm = GETVARPARTIREF PTR <@hi64> %parr
                %pelm_i = SHIFTIREF PTR <@i64 @i64> @partition.blk0.pelm %i
                %t = LOAD PTR <@i64> %pelm_i
                %pelm_idx_high = SHIFTIREF PTR <@i64 @i64> %pelm %idx_high
                %elm_idx_high = LOAD PTR <@i64> %pelm_idx_high
                STORE PTR <@i64> %pelm_i %elm_idx_high
                STORE PTR <@i64> %pelm_idx_high %t
                BRANCH %blk5(%i)

            %blk5(<@i64> %i):
                RET (%i)

        }

    :type bldr: rpython.rlib.rmu.MuIRBuilder
    :type rmu: rpython.rlib.rmu
    :return: (rmu.MuVM(), rmu.MuCtx, rmu.MuIRBuilder, MuID, MuID)
    """
    NA = rmu.MuMemOrd.NOT_ATOMIC

    i64 = bldr.gen_sym("@i64"); bldr.new_type_int(i64, 64)
    hi64 = bldr.gen_sym("@hi64"); bldr.new_type_hybrid(hi64, [], i64)
    phi64 = bldr.gen_sym("@phi64"); bldr.new_type_uptr(phi64, hi64)
    c_1 = bldr.gen_sym("@c_1"); bldr.new_const_int(c_1, i64, 1)
    sig_quicksort = bldr.gen_sym("@sig_quicksort"); bldr.new_funcsig(sig_quicksort, [phi64, i64, i64], [])
    sig_partition = bldr.gen_sym("@sig_partition"); bldr.new_funcsig(sig_partition, [phi64, i64, i64], [i64])
    quicksort = bldr.gen_sym("@quicksort"); bldr.new_func(quicksort, sig_quicksort)
    partition = bldr.gen_sym("@partition"); bldr.new_func(partition, sig_partition)

    # quicksort
    blk0 = bldr.gen_sym("@quicksort.v1.blk0")
    blk1 = bldr.gen_sym("@quicksort.v1.blk1")
    blk2 = bldr.gen_sym("@quicksort.v1.blk2")

    # blk0
    parr = bldr.gen_sym("@quicksort.v1.blk0.parr")
    start = bldr.gen_sym("@quicksort.v1.blk0.start")
    end = bldr.gen_sym("@quicksort.v1.blk0.end")
    cmpres = bldr.gen_sym("@quicksort.v1.blk0.cmpres")
    op_slt = bldr.gen_sym(); bldr.new_cmp(op_slt, cmpres, rmu.MuCmpOptr.SLT, i64, start, end)
    dst_t = bldr.gen_sym(); bldr.new_dest_clause(dst_t, blk2, [parr, start, end])
    dst_f = bldr.gen_sym(); bldr.new_dest_clause(dst_f, blk1, [])
    op_br2 = bldr.gen_sym(); bldr.new_branch2(op_br2, cmpres, dst_t, dst_f)
    bldr.new_bb(blk0, [parr, start, end], [phi64, i64, i64], rmu.MU_NO_ID, [op_slt, op_br2])

    # blk1
    op_ret = bldr.gen_sym(); bldr.new_ret(op_ret, [])
    bldr.new_bb(blk1, [], [], rmu.MU_NO_ID, [op_ret])

    # blk2
    parr = bldr.gen_sym("@quicksort.v1.blk2.parr")
    start = bldr.gen_sym("@quicksort.v1.blk2.start")
    end = bldr.gen_sym("@quicksort.v1.blk2.end")
    p = bldr.gen_sym("@quicksort.v1.blk2.p")
    ps1 = bldr.gen_sym("@quicksort.v1.blk2.ps1")
    pp1 = bldr.gen_sym("@quicksort.v1.blk2.pp1")
    op_call1 = bldr.gen_sym(); bldr.new_call(op_call1, [p], sig_partition, partition, [parr, start, end])
    op_sub = bldr.gen_sym(); bldr.new_binop(op_sub, ps1, rmu.MuBinOptr.SUB, i64, p, c_1)
    op_call2 = bldr.gen_sym(); bldr.new_call(op_call2, [], sig_quicksort, quicksort, [parr, start, ps1])
    op_add = bldr.gen_sym(); bldr.new_binop(op_add, pp1, rmu.MuBinOptr.ADD, i64, p, c_1)
    op_call3 = bldr.gen_sym(); bldr.new_call(op_call3, [], sig_quicksort, quicksort, [parr, pp1, end])
    dst = bldr.gen_sym(); bldr.new_dest_clause(dst, blk1, [])
    op_br = bldr.gen_sym(); bldr.new_branch(op_br, dst)
    bldr.new_bb(blk2, [parr, start, end], [phi64, i64, i64], rmu.MU_NO_ID,
                [op_call1, op_sub, op_call2, op_add, op_call3, op_br])

    bldr.new_func_ver(bldr.gen_sym("@quicksort.v1"), quicksort, [blk0, blk1, blk2])

    # partition
    blk0 = bldr.gen_sym("@partition.v1.blk0")
    blk1 = bldr.gen_sym("@partition.v1.blk1")
    blk2 = bldr.gen_sym("@partition.v1.blk2")
    blk3 = bldr.gen_sym("@partition.v1.blk3")
    blk4 = bldr.gen_sym("@partition.v1.blk4")
    blk5 = bldr.gen_sym("@partition.v1.blk5")

    # blk0
    parr = bldr.gen_sym("@partition.v1.blk0.parr")
    idx_low = bldr.gen_sym("@partition.v1.blk0.idx_low")
    idx_high = bldr.gen_sym("@partition.v1.blk0.idx_high")
    pelm = bldr.gen_sym("@partition.v1.blk0.pelm")
    pelm_idx_high = bldr.gen_sym("@partition.v1.blk0.pelm_idx_high")
    pivot = bldr.gen_sym("@partition.v1.blk0.pivot")
    op_getvarpartiref = bldr.gen_sym(); bldr.new_getvarpartiref(op_getvarpartiref, pelm, True, hi64, parr)
    op_shiftiref = bldr.gen_sym(); bldr.new_shiftiref(op_shiftiref, pelm_idx_high, True, i64, i64, pelm, idx_high)
    op_load = bldr.gen_sym(); bldr.new_load(op_load, pivot, True, NA, i64, pelm_idx_high)
    dst = bldr.gen_sym(); bldr.new_dest_clause(dst, blk1, [parr, idx_high, pivot, idx_low, idx_low, idx_high])
    op_br = bldr.gen_sym(); bldr.new_branch(op_br, dst)
    bldr.new_bb(blk0, [parr, idx_low, idx_high], [phi64, i64, i64], rmu.MU_NO_ID, [op_getvarpartiref, op_shiftiref, op_load, op_br])

    # blk1
    parr = bldr.gen_sym("@partition.v1.blk1.parr")
    idx_high = bldr.gen_sym("@partition.v1.blk1.idx_high")
    pivot = bldr.gen_sym("@partition.v1.blk1.pivot")
    i = bldr.gen_sym("@partition.v1.blk1.i")
    j = bldr.gen_sym("@partition.v1.blk1.j")
    end = bldr.gen_sym("@partition.v1.blk1.end")
    cmpres = bldr.gen_sym("@partition.v1.blk1.cmpres")
    op_sge = bldr.gen_sym(); bldr.new_cmp(op_sge, cmpres, rmu.MuCmpOptr.SGE, i64, j, end)
    dst_t = bldr.gen_sym(); bldr.new_dest_clause(dst_t, blk4, [i, parr, idx_high])
    dst_f = bldr.gen_sym(); bldr.new_dest_clause(dst_f, blk2, [end, j, i, parr, idx_high, pivot])
    op_br2 = bldr.gen_sym(); bldr.new_branch2(op_br2, cmpres, dst_t, dst_f)
    bldr.new_bb(blk1, [parr, idx_high, pivot, i, j, end],
                      [phi64, i64, i64, i64, i64, i64], rmu.MU_NO_ID, [op_sge, op_br2])

    # blk2
    end = bldr.gen_sym("@partition.v1.blk2.end")
    j = bldr.gen_sym("@partition.v1.blk2.j")
    i = bldr.gen_sym("@partition.v1.blk2.i")
    parr = bldr.gen_sym("@partition.v1.blk2.parr")
    idx_high = bldr.gen_sym("@partition.v1.blk2.idx_high")
    pivot = bldr.gen_sym("@partition.v1.blk2.pivot")
    jp1 = bldr.gen_sym("@partition.v1.blk2.jp1")
    pelm = bldr.gen_sym("@partition.v1.blk2.pelm")
    pelm_j = bldr.gen_sym("@partition.v1.blk2.pelm_j")
    elm_j = bldr.gen_sym("@partition.v1.blk2.elm_j")
    cmpres = bldr.gen_sym("@partition.v1.blk2.cmpres")
    op_add = bldr.gen_sym(); bldr.new_binop(op_add, jp1, rmu.MuBinOptr.ADD, i64, j, c_1)
    op_getvarpartiref = bldr.gen_sym(); bldr.new_getvarpartiref(op_getvarpartiref, pelm, True, hi64, parr)
    op_shiftiref = bldr.gen_sym(); bldr.new_shiftiref(op_shiftiref, pelm_j, True, i64, i64, pelm, j)
    op_load = bldr.gen_sym(); bldr.new_load(op_load, elm_j, True, NA, i64, pelm_j)
    op_slt = bldr.gen_sym(); bldr.new_cmp(op_slt, cmpres, rmu.MuCmpOptr.SLT, i64, elm_j, pivot)
    dst_t = bldr.gen_sym(); bldr.new_dest_clause(dst_t, blk3, [jp1, end, pivot, idx_high, j, i, parr])
    dst_f = bldr.gen_sym(); bldr.new_dest_clause(dst_f, blk1, [parr, idx_high, pivot, i, jp1, end])
    op_br2 = bldr.gen_sym(); bldr.new_branch2(op_br2, cmpres, dst_t, dst_f)
    bldr.new_bb(blk2, [end, j, i, parr, idx_high, pivot], [i64, i64, i64, phi64, i64, i64], rmu.MU_NO_ID,
                [op_add, op_getvarpartiref, op_shiftiref, op_load, op_slt, op_br2])

    # blk3
    jp1 = bldr.gen_sym("@partition.v1.blk3.jp1")
    end = bldr.gen_sym("@partition.v1.blk3.end")
    pivot = bldr.gen_sym("@partition.v1.blk3.pivot")
    idx_high = bldr.gen_sym("@partition.v1.blk3.idx_high")
    j = bldr.gen_sym("@partition.v1.blk3.j")
    i = bldr.gen_sym("@partition.v1.blk3.i")
    parr = bldr.gen_sym("@partition.v1.blk3.parr")
    pelm = bldr.gen_sym("@partition.v1.blk3.pelm")
    pelm_i = bldr.gen_sym("@partition.v1.blk3.pelm_i")
    t = bldr.gen_sym("@partition.v1.blk3.t")
    pelm_j = bldr.gen_sym("@partition.v1.blk3.pelm_j")
    elm_j = bldr.gen_sym("@partition.v1.blk3.elm_j")
    ip1 = bldr.gen_sym("@partition.v1.blk3.ip1")
    op_getvarpartiref = bldr.gen_sym(); bldr.new_getvarpartiref(op_getvarpartiref, pelm, True, hi64, parr)
    op_shiftiref1 = bldr.gen_sym(); bldr.new_shiftiref(op_shiftiref1, pelm_i, True, i64, i64, pelm, i)
    op_load1 = bldr.gen_sym(); bldr.new_load(op_load1, t, True, NA, i64, pelm_i)
    op_shiftiref2 = bldr.gen_sym(); bldr.new_shiftiref(op_shiftiref2, pelm_j, True, i64, i64, pelm, j)
    op_load2 = bldr.gen_sym(); bldr.new_load(op_load2, elm_j, True, NA, i64, pelm_j)
    op_store1 = bldr.gen_sym(); bldr.new_store(op_store1, True, NA, i64, pelm_i, elm_j)
    op_store2 = bldr.gen_sym(); bldr.new_store(op_store2, True, NA, i64, pelm_j, t)
    op_add = bldr.gen_sym(); bldr.new_binop(op_add, ip1, rmu.MuBinOptr.ADD, i64, i, c_1)
    dst = bldr.gen_sym(); bldr.new_dest_clause(dst, blk1, [parr, idx_high, pivot, ip1, jp1, end])
    op_br = bldr.gen_sym(); bldr.new_branch(op_br, dst)
    bldr.new_bb(blk3, [jp1, end, pivot, idx_high, j, i, parr], [i64, i64, i64, i64, i64, i64, phi64], rmu.MU_NO_ID,
                [op_getvarpartiref, op_shiftiref1, op_load1, op_shiftiref2, op_load2, op_store1, op_store2, op_add, op_br])

    # blk4
    i = bldr.gen_sym("@partition.v1.blk4.i")
    parr = bldr.gen_sym("@partition.v1.blk4.parr")
    idx_high = bldr.gen_sym("@partition.v1.blk4.idx_high")
    pelm = bldr.gen_sym("@partition.v1.blk4.pelm")
    pelm_i = bldr.gen_sym("@partition.v1.blk4.pelm_i")
    t = bldr.gen_sym("@partition.v1.blk4.t")
    pelm_idx_high = bldr.gen_sym("@partition.v1.blk4.pelm_idx_high")
    elm_idx_high = bldr.gen_sym("@partition.v1.blk4.elm_idx_high")
    op_getvarpartiref = bldr.gen_sym(); bldr.new_getvarpartiref(op_getvarpartiref, pelm, True, hi64, parr)
    op_shiftiref1 = bldr.gen_sym(); bldr.new_shiftiref(op_shiftiref1, pelm_i, True, i64, i64, pelm, i)
    op_load1 = bldr.gen_sym(); bldr.new_load(op_load1, t, True, NA, i64, pelm_i)
    op_shiftiref2 = bldr.gen_sym(); bldr.new_shiftiref(op_shiftiref2, pelm_idx_high, True, i64, i64, pelm, idx_high)
    op_load2 = bldr.gen_sym(); bldr.new_load(op_load2, elm_idx_high, True, NA, i64, pelm_idx_high)
    op_store1 = bldr.gen_sym(); bldr.new_store(op_store1, True, NA, i64, pelm_i, elm_idx_high)
    op_store2 = bldr.gen_sym(); bldr.new_store(op_store2, True, NA, i64, pelm_idx_high, t)
    dst = bldr.gen_sym(); bldr.new_dest_clause(dst, blk5, [i])
    op_br = bldr.gen_sym(); bldr.new_branch(op_br, dst)
    bldr.new_bb(blk4, [i, parr, idx_high], [i64, phi64, i64], rmu.MU_NO_ID,
                [op_getvarpartiref, op_shiftiref1, op_load1, op_shiftiref2, op_load2, op_store1, op_store2, op_br])

    # blk5
    i = bldr.gen_sym("@partition.v1.blk5.i")
    op_ret = bldr.gen_sym(); bldr.new_ret(op_ret, [i])
    bldr.new_bb(blk5, [i], [i64], rmu.MU_NO_ID, [op_ret])

    bldr.new_func_ver(bldr.gen_sym("@partition.v1"), partition, [blk0, blk1, blk2, blk3, blk4, blk5])
    return None


def setup(n):
    lst = rand_list_of(n)
    arr = lltype.malloc(rffi.CArray(rffi.LONGLONG), n, flavor='raw')
    for i, k in enumerate(lst):
        arr[i] = k
    return arr, 0, n - 1


def teardown(arr, s, e):
    lltype.free(arr, 'raw')


def rand_list_of(n):
    # 32 extend to 64-bit integers (to avoid overflow in summation
    from random import randrange, setstate
    init_state = (3, (
        2147483648L, 3430835514L, 2928424416L, 3147699060L, 2823572732L, 2905216632L, 1887281517L, 14272356L,
        1356039141L,
        2741361235L, 1824725388L, 2228169284L, 2679861265L, 3150239284L, 657657570L, 1407124159L, 517316568L,
        653526369L,
        139268705L, 3784719953L, 2212355490L, 3452491289L, 1232629882L, 1791207424L, 2898278956L, 1147783320L,
        1824413680L,
        1993303973L, 2568444883L, 4228847642L, 4163974668L, 385627078L, 3663560714L, 320542554L, 1565882322L,
        3416481154L,
        4219229298L, 315071254L, 778331393L, 3961037651L, 2951403614L, 3355970261L, 102946340L, 2509883952L, 215897963L,
        3361072826L, 689991350L, 3348092598L, 1763608447L, 2140226443L, 3813151178L, 2619956936L, 51244592L,
        2130725065L,
        3867113849L, 1980820881L, 2600246771L, 3207535572L, 257556968L, 2223367443L, 3706150033L, 1711074250L,
        4252385224L,
        3197142331L, 4139558716L, 748471849L, 2281163369L, 2596250092L, 2804492653L, 484240110L, 3726117536L,
        2483815933L,
        2173995598L, 3765136999L, 3178931194L, 1237068319L, 3427263384L, 3958412830L, 2268556676L, 360704423L,
        4113430429L,
        3758882140L, 3743971788L, 1685454939L, 488386L, 3511218911L, 3020688912L, 2168345327L, 3149651862L, 1472484695L,
        2011779229L, 1112533726L, 1873931730L, 2196153055L, 3806225492L, 1515074892L, 251489714L, 1958141723L,
        2081062631L,
        3703490262L, 3211541213L, 1436109217L, 2664448365L, 2350764370L, 1285829042L, 3496997759L, 2306637687L,
        1571644344L,
        1020052455L, 3114491401L, 2994766034L, 1518527036L, 994512437L, 1732585804L, 2089330296L, 2592371643L,
        2377347339L,
        2617648350L, 1478066246L, 389918052L, 1126787130L, 2728695369L, 2921719205L, 3193658789L, 2101782606L,
        4284039483L,
        2704867468L, 3843423543L, 119359906L, 1882384901L, 832276556L, 1862974878L, 1943541262L, 1823624942L,
        2146680272L,
        333006125L, 929197835L, 639017219L, 1640196300L, 1424826762L, 2119569013L, 4259272802L, 2089277168L,
        2030198981L,
        2950559216L, 621654826L, 3452546704L, 4085446289L, 3038316311L, 527272378L, 1679817853L, 450787204L,
        3525043861L,
        3838351358L, 1558592021L, 3649888848L, 3328370698L, 3247166155L, 3855970537L, 1183088418L, 2778702834L,
        2820277014L,
        1530905121L, 1434023607L, 3942716950L, 41643359L, 310637634L, 1537174663L, 4265200088L, 3126624846L,
        2837665903L,
        446994733L, 85970060L, 643115053L, 1751804182L, 1480207958L, 2977093071L, 544778713L, 738954842L, 3370733859L,
        3242319053L, 2707786138L, 4041098196L, 1671493839L, 3420415077L, 2473516599L, 3949211965L, 3686186772L,
        753757988L,
        220738063L, 772481263L, 974568026L, 3190407677L, 480257177L, 3620733162L, 2616878358L, 665763320L, 2808607644L,
        3851308236L, 3633157256L, 4240746864L, 1261222691L, 268963935L, 1449514350L, 4229662564L, 1342533852L,
        1913674460L,
        1761163533L, 1974260074L, 739184472L, 3811507072L, 2880992381L, 3998389163L, 2673626426L, 2212222504L,
        231447607L,
        2608719702L, 3509764733L, 2403318909L, 635983093L, 4233939991L, 2894463467L, 177171270L, 2962364044L,
        1191007101L,
        882222586L, 1004217833L, 717897978L, 2125381922L, 626199402L, 3694698943L, 1373935523L, 762314613L, 2291077454L,
        2111081024L, 3758576304L, 2812129656L, 4067461097L, 3700761868L, 2281420733L, 197217625L, 460620692L,
        506837624L,
        1532931238L, 3872395078L, 3629107738L, 2273221134L, 2086345980L, 1240615886L, 958420495L, 4059583254L,
        3119201875L,
        3742950862L, 891360845L, 2974235885L, 87814219L, 4067521161L, 615939803L, 1881195074L, 2225917026L, 2775128741L,
        2996201447L, 1590546624L, 3960431955L, 1417477945L, 913935155L, 1610033170L, 3212701447L, 2545374014L,
        2887105562L,
        2991635417L, 3194532260L, 1565555757L, 2142474733L, 621483430L, 2268177481L, 919992760L, 2022043644L,
        2756890220L,
        881105937L, 2621060794L, 4262292201L, 480112895L, 2557060162L, 2367031748L, 2172434102L, 296539623L,
        3043643256L,
        59166373L, 2947638193L, 1312917612L, 1798724013L, 75864164L, 339661149L, 289536004L, 422147716L, 1134944052L,
        1095534216L, 1231984277L, 239787072L, 923053211L, 1015393503L, 2558889580L, 4194512643L, 448088150L, 707905706L,
        2649061310L, 3081089715L, 3432955562L, 2217740069L, 1965789353L, 3320360228L, 3625802364L, 2420747908L,
        3116949010L,
        442654625L, 2157578112L, 3603825090L, 3111995525L, 1124579902L, 101836896L, 3297125816L, 136981134L,
        4253748197L,
        3809600572L, 1668193778L, 4146759785L, 3712590372L, 2998653463L, 3032597504L, 1046471011L, 2843821193L,
        802959497L,
        3307715534L, 3226042258L, 1014478160L, 3105844949L, 3209150965L, 610876993L, 2563947590L, 2482526324L,
        3913970138L,
        2812702315L, 4281779167L, 1026357391L, 2579486306L, 402208L, 3457975059L, 1714004950L, 2543595755L, 2421499458L,
        478932497L, 3117588180L, 1565800974L, 1757724858L, 1483685124L, 2262270397L, 3794544469L, 3986696110L,
        2914756339L,
        1952061826L, 2672480198L, 3793151752L, 309930721L, 1861137379L, 94571340L, 1162935802L, 3681554226L,
        4027302061L,
        21079572L, 446709644L, 1587253187L, 1845056582L, 3080553052L, 3575272255L, 2526224735L, 3569822959L,
        2685900491L,
        918305237L, 1399881227L, 1554912161L, 703181091L, 738501299L, 269937670L, 1078548118L, 2313670525L, 3495159622L,
        2659487842L, 11394628L, 1222454456L, 3392065094L, 3426833642L, 1153231613L, 1234517654L, 3144547626L,
        2148039080L,
        3790136587L, 684648337L, 3956093475L, 1384378197L, 2042781475L, 759764431L, 222267088L, 3187778457L,
        3795259108L,
        2817237549L, 3494781277L, 3762880618L, 892345749L, 2153484401L, 721588894L, 779278769L, 3306398772L,
        4221452913L,
        1981375723L, 379087895L, 1604791625L, 1426046977L, 4231163093L, 1344994557L, 1341041093L, 1072537134L,
        1829925137L,
        3791772627L, 3176876700L, 2553745117L, 664821113L, 473469583L, 1076256869L, 2406012795L, 3141453822L,
        4123012649L,
        3058620143L, 1785080140L, 1181483189L, 3587874749L, 1453504375L, 707249496L, 2022787257L, 2436320047L,
        602521701L,
        483826957L, 821599664L, 3333871672L, 3024431570L, 3814441382L, 416508285L, 1217138244L, 3975201118L,
        3077724941L,
        180118569L, 3754556886L, 4121534265L, 3495283397L, 700504668L, 3113972067L, 719371171L, 910731026L, 619936911L,
        2937105529L, 2039892965L, 3853404454L, 3783801801L, 783321997L, 1135195902L, 326690505L, 1774036419L,
        3476057413L,
        1518029608L, 1248626026L, 427510490L, 3443223611L, 4087014505L, 2858955517L, 1918675812L, 3921514056L,
        3929126528L,
        4048889933L, 1583842117L, 3742539544L, 602292017L, 3393759050L, 3929818519L, 3119818281L, 3472644693L,
        1993924627L,
        4163228821L, 2943877721L, 3143487730L, 4087113198L, 1149082355L, 1713272081L, 1243627655L, 3511633996L,
        3358757220L,
        3812981394L, 650044449L, 2143650644L, 3869591312L, 3719322297L, 386030648L, 2633538573L, 672966554L,
        3498396042L,
        3907556L, 2308686209L, 2878779858L, 1475925955L, 2701537395L, 1448018484L, 2962578755L, 1383479284L,
        3731453464L,
        3659512663L, 1521189121L, 843749206L, 2243090279L, 572717972L, 3400421356L, 3440777300L, 1393518699L,
        1681924551L,
        466257295L, 568413244L, 3288530316L, 2951425105L, 2624424893L, 2410788864L, 2243174464L, 1385949609L,
        2454100663L,
        1113953725L, 2127471443L, 1775715557L, 3874125135L, 1901707926L, 3152599339L, 2277843623L, 1941785089L,
        3171888228L,
        802596998L, 3397391306L, 1743834429L, 395463904L, 2099329462L, 3761809163L, 262702111L, 1868879810L,
        2887406426L,
        1160032302L, 4164116477L, 2287740849L, 3312176050L, 747117003L, 4048006270L, 3955419375L, 2724452926L,
        3141695820L,
        791246424L, 524525849L, 1794277132L, 295485241L, 4125127474L, 825108028L, 1582794137L, 1259992755L, 2938829230L,
        912029932L, 1534496985L, 3075283272L, 4052041116L, 1125808104L, 2032938837L, 4008676545L, 1638361535L,
        1649316497L,
        1302633381L, 4221627277L, 1206130263L, 3114681993L, 3409690900L, 3373263243L, 2922903613L, 349048087L,
        4049532385L,
        3458779287L, 1737687814L, 287275672L, 645786941L, 1492233180L, 3925845678L, 3344829077L, 1669219217L,
        665224162L,
        2679234088L, 1986576411L, 50610077L, 1080114376L, 1881648396L, 3818465156L, 1486861008L, 3824208930L,
        1782008170L,
        4115911912L, 656413265L, 771498619L, 2709443211L, 1919820065L, 451888753L, 1449812173L, 2001941180L,
        2997921765L,
        753032713L, 3011517640L, 2386888602L, 3181040472L, 1280522185L, 1036471598L, 1243809973L, 2985144032L,
        2238294821L,
        557934351L, 347132246L, 1797956016L, 624L), None)
    setstate(init_state)
    return [rffi.r_longlong(randrange(-(1 << 31), (1 << 31) - 1)) for _ in range(n)]


def measure(N):
    args = setup(N)
    from time import time
    t0 = time()
    quicksort(*args)
    t1 = time()
    teardown(*args)
    return t0, t1


def rpy_entry(arr, start, end):
    from time import time
    t0 = time()
    quicksort(arr, start, end)
    t1 = time()
    return t1 - t0


def setup2(n):
    lst = rand_list_of(n)
    arr = lltype.malloc(rffi.CArray(rffi.LONGLONG), n, flavor='raw')
    for i, k in enumerate(lst):
        arr[i] = k

    lst2 = rand_list_of(n)
    arr2 = lltype.malloc(rffi.CArray(rffi.LONGLONG), n, flavor='raw')
    for i, k in enumerate(lst2):
        arr2[i] = k
    return arr, arr2, 0, n - 1


def teardown2(arr, arr2, s, e):
    lltype.free(arr, 'raw')
    lltype.free(arr2, 'raw')


def rpy_entry2(arr, arr2, start, end):
    from time import time
    t0 = time()
    quicksort(arr, start, end)
    quicksort(arr2, start, end)
    t1 = time()
    return t1 - t0

if __name__ == '__main__':
    import sys
    t0, t1 = measure(int(sys.argv[1]))
    print '%.15f' % (t1 - t0)


def target(*args):
    from rpython.rlib.entrypoint import export_symbol
    export_symbol(rpy_entry)
    return rpy_entry, [int]
