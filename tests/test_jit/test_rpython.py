from rpython.rtyper.lltypesystem import rffi, lltype
from rpython.rlib.rmu import zebu as rmu
from rpython.translator.platform import platform
from util import fncptr_from_rpy_func, fncptr_from_py_script, may_spawn_proc
import ctypes, py, stat, os
import pytest

# -------------------
# helper functions
def rand_list_of(n):
    # 32 extend to 64-bit integers (to avoid overflow in summation
    from random import randrange, getstate
    init_state = getstate()
    return [rffi.r_longlong(randrange(-(1 << 31), (1 << 31) - 1)) for _ in range(n)], init_state


# --------------------------
# tests
@may_spawn_proc
def test_add():
    def add(a, b):
        return a + b

    fn, _ = fncptr_from_rpy_func(add, [rffi.LONGLONG, rffi.LONGLONG], rffi.LONGLONG)

    assert fn(1, 2) == 3

@may_spawn_proc
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

            assert fnc(vec1, vec2) == 32


@may_spawn_proc
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

        fnc(arr, 5) == -5


@may_spawn_proc
def test_arraysum():
    from rpython.rlib.jit import JitDriver
    d = JitDriver(greens=[], reds='auto')
    def arraysum(arr, sz):
        sum = rffi.r_longlong(0)
        for i in range(sz):
            d.jit_merge_point()
            sum += arr[i]
        return sum

    fnc, (db, bdlgen) = fncptr_from_rpy_func(arraysum, [rffi.CArrayPtr(rffi.LONGLONG), rffi.SIZE_T], rffi.LONGLONG)
    bdlgen.mu.current_thread_as_mu_thread(rmu.null(rmu.MuCPtr))

    n = 100
    lst, _ = rand_list_of(n)
    with lltype.scoped_alloc(rffi.CArray(rffi.LONGLONG), n) as arr:
        for i, k in enumerate(lst):
            arr[i] = k

        assert fnc(arr, rffi.cast(rffi.SIZE_T, n)) == arraysum(arr, rffi.cast(rffi.SIZE_T, n))


@may_spawn_proc
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

    fnc, (db, bdlgen) = fncptr_from_rpy_func(quicksort, [rffi.CArrayPtr(rffi.LONGLONG), lltype.Signed, lltype.Signed], lltype.Void)
    bdlgen.mu.current_thread_as_mu_thread(rmu.null(rmu.MuCPtr))
    # fnc = quicksort

    n = 100
    from random import setstate
    init_state = (3, (
    2147483648L, 3430835514L, 2928424416L, 3147699060L, 2823572732L, 2905216632L, 1887281517L, 14272356L, 1356039141L,
    2741361235L, 1824725388L, 2228169284L, 2679861265L, 3150239284L, 657657570L, 1407124159L, 517316568L, 653526369L,
    139268705L, 3784719953L, 2212355490L, 3452491289L, 1232629882L, 1791207424L, 2898278956L, 1147783320L, 1824413680L,
    1993303973L, 2568444883L, 4228847642L, 4163974668L, 385627078L, 3663560714L, 320542554L, 1565882322L, 3416481154L,
    4219229298L, 315071254L, 778331393L, 3961037651L, 2951403614L, 3355970261L, 102946340L, 2509883952L, 215897963L,
    3361072826L, 689991350L, 3348092598L, 1763608447L, 2140226443L, 3813151178L, 2619956936L, 51244592L, 2130725065L,
    3867113849L, 1980820881L, 2600246771L, 3207535572L, 257556968L, 2223367443L, 3706150033L, 1711074250L, 4252385224L,
    3197142331L, 4139558716L, 748471849L, 2281163369L, 2596250092L, 2804492653L, 484240110L, 3726117536L, 2483815933L,
    2173995598L, 3765136999L, 3178931194L, 1237068319L, 3427263384L, 3958412830L, 2268556676L, 360704423L, 4113430429L,
    3758882140L, 3743971788L, 1685454939L, 488386L, 3511218911L, 3020688912L, 2168345327L, 3149651862L, 1472484695L,
    2011779229L, 1112533726L, 1873931730L, 2196153055L, 3806225492L, 1515074892L, 251489714L, 1958141723L, 2081062631L,
    3703490262L, 3211541213L, 1436109217L, 2664448365L, 2350764370L, 1285829042L, 3496997759L, 2306637687L, 1571644344L,
    1020052455L, 3114491401L, 2994766034L, 1518527036L, 994512437L, 1732585804L, 2089330296L, 2592371643L, 2377347339L,
    2617648350L, 1478066246L, 389918052L, 1126787130L, 2728695369L, 2921719205L, 3193658789L, 2101782606L, 4284039483L,
    2704867468L, 3843423543L, 119359906L, 1882384901L, 832276556L, 1862974878L, 1943541262L, 1823624942L, 2146680272L,
    333006125L, 929197835L, 639017219L, 1640196300L, 1424826762L, 2119569013L, 4259272802L, 2089277168L, 2030198981L,
    2950559216L, 621654826L, 3452546704L, 4085446289L, 3038316311L, 527272378L, 1679817853L, 450787204L, 3525043861L,
    3838351358L, 1558592021L, 3649888848L, 3328370698L, 3247166155L, 3855970537L, 1183088418L, 2778702834L, 2820277014L,
    1530905121L, 1434023607L, 3942716950L, 41643359L, 310637634L, 1537174663L, 4265200088L, 3126624846L, 2837665903L,
    446994733L, 85970060L, 643115053L, 1751804182L, 1480207958L, 2977093071L, 544778713L, 738954842L, 3370733859L,
    3242319053L, 2707786138L, 4041098196L, 1671493839L, 3420415077L, 2473516599L, 3949211965L, 3686186772L, 753757988L,
    220738063L, 772481263L, 974568026L, 3190407677L, 480257177L, 3620733162L, 2616878358L, 665763320L, 2808607644L,
    3851308236L, 3633157256L, 4240746864L, 1261222691L, 268963935L, 1449514350L, 4229662564L, 1342533852L, 1913674460L,
    1761163533L, 1974260074L, 739184472L, 3811507072L, 2880992381L, 3998389163L, 2673626426L, 2212222504L, 231447607L,
    2608719702L, 3509764733L, 2403318909L, 635983093L, 4233939991L, 2894463467L, 177171270L, 2962364044L, 1191007101L,
    882222586L, 1004217833L, 717897978L, 2125381922L, 626199402L, 3694698943L, 1373935523L, 762314613L, 2291077454L,
    2111081024L, 3758576304L, 2812129656L, 4067461097L, 3700761868L, 2281420733L, 197217625L, 460620692L, 506837624L,
    1532931238L, 3872395078L, 3629107738L, 2273221134L, 2086345980L, 1240615886L, 958420495L, 4059583254L, 3119201875L,
    3742950862L, 891360845L, 2974235885L, 87814219L, 4067521161L, 615939803L, 1881195074L, 2225917026L, 2775128741L,
    2996201447L, 1590546624L, 3960431955L, 1417477945L, 913935155L, 1610033170L, 3212701447L, 2545374014L, 2887105562L,
    2991635417L, 3194532260L, 1565555757L, 2142474733L, 621483430L, 2268177481L, 919992760L, 2022043644L, 2756890220L,
    881105937L, 2621060794L, 4262292201L, 480112895L, 2557060162L, 2367031748L, 2172434102L, 296539623L, 3043643256L,
    59166373L, 2947638193L, 1312917612L, 1798724013L, 75864164L, 339661149L, 289536004L, 422147716L, 1134944052L,
    1095534216L, 1231984277L, 239787072L, 923053211L, 1015393503L, 2558889580L, 4194512643L, 448088150L, 707905706L,
    2649061310L, 3081089715L, 3432955562L, 2217740069L, 1965789353L, 3320360228L, 3625802364L, 2420747908L, 3116949010L,
    442654625L, 2157578112L, 3603825090L, 3111995525L, 1124579902L, 101836896L, 3297125816L, 136981134L, 4253748197L,
    3809600572L, 1668193778L, 4146759785L, 3712590372L, 2998653463L, 3032597504L, 1046471011L, 2843821193L, 802959497L,
    3307715534L, 3226042258L, 1014478160L, 3105844949L, 3209150965L, 610876993L, 2563947590L, 2482526324L, 3913970138L,
    2812702315L, 4281779167L, 1026357391L, 2579486306L, 402208L, 3457975059L, 1714004950L, 2543595755L, 2421499458L,
    478932497L, 3117588180L, 1565800974L, 1757724858L, 1483685124L, 2262270397L, 3794544469L, 3986696110L, 2914756339L,
    1952061826L, 2672480198L, 3793151752L, 309930721L, 1861137379L, 94571340L, 1162935802L, 3681554226L, 4027302061L,
    21079572L, 446709644L, 1587253187L, 1845056582L, 3080553052L, 3575272255L, 2526224735L, 3569822959L, 2685900491L,
    918305237L, 1399881227L, 1554912161L, 703181091L, 738501299L, 269937670L, 1078548118L, 2313670525L, 3495159622L,
    2659487842L, 11394628L, 1222454456L, 3392065094L, 3426833642L, 1153231613L, 1234517654L, 3144547626L, 2148039080L,
    3790136587L, 684648337L, 3956093475L, 1384378197L, 2042781475L, 759764431L, 222267088L, 3187778457L, 3795259108L,
    2817237549L, 3494781277L, 3762880618L, 892345749L, 2153484401L, 721588894L, 779278769L, 3306398772L, 4221452913L,
    1981375723L, 379087895L, 1604791625L, 1426046977L, 4231163093L, 1344994557L, 1341041093L, 1072537134L, 1829925137L,
    3791772627L, 3176876700L, 2553745117L, 664821113L, 473469583L, 1076256869L, 2406012795L, 3141453822L, 4123012649L,
    3058620143L, 1785080140L, 1181483189L, 3587874749L, 1453504375L, 707249496L, 2022787257L, 2436320047L, 602521701L,
    483826957L, 821599664L, 3333871672L, 3024431570L, 3814441382L, 416508285L, 1217138244L, 3975201118L, 3077724941L,
    180118569L, 3754556886L, 4121534265L, 3495283397L, 700504668L, 3113972067L, 719371171L, 910731026L, 619936911L,
    2937105529L, 2039892965L, 3853404454L, 3783801801L, 783321997L, 1135195902L, 326690505L, 1774036419L, 3476057413L,
    1518029608L, 1248626026L, 427510490L, 3443223611L, 4087014505L, 2858955517L, 1918675812L, 3921514056L, 3929126528L,
    4048889933L, 1583842117L, 3742539544L, 602292017L, 3393759050L, 3929818519L, 3119818281L, 3472644693L, 1993924627L,
    4163228821L, 2943877721L, 3143487730L, 4087113198L, 1149082355L, 1713272081L, 1243627655L, 3511633996L, 3358757220L,
    3812981394L, 650044449L, 2143650644L, 3869591312L, 3719322297L, 386030648L, 2633538573L, 672966554L, 3498396042L,
    3907556L, 2308686209L, 2878779858L, 1475925955L, 2701537395L, 1448018484L, 2962578755L, 1383479284L, 3731453464L,
    3659512663L, 1521189121L, 843749206L, 2243090279L, 572717972L, 3400421356L, 3440777300L, 1393518699L, 1681924551L,
    466257295L, 568413244L, 3288530316L, 2951425105L, 2624424893L, 2410788864L, 2243174464L, 1385949609L, 2454100663L,
    1113953725L, 2127471443L, 1775715557L, 3874125135L, 1901707926L, 3152599339L, 2277843623L, 1941785089L, 3171888228L,
    802596998L, 3397391306L, 1743834429L, 395463904L, 2099329462L, 3761809163L, 262702111L, 1868879810L, 2887406426L,
    1160032302L, 4164116477L, 2287740849L, 3312176050L, 747117003L, 4048006270L, 3955419375L, 2724452926L, 3141695820L,
    791246424L, 524525849L, 1794277132L, 295485241L, 4125127474L, 825108028L, 1582794137L, 1259992755L, 2938829230L,
    912029932L, 1534496985L, 3075283272L, 4052041116L, 1125808104L, 2032938837L, 4008676545L, 1638361535L, 1649316497L,
    1302633381L, 4221627277L, 1206130263L, 3114681993L, 3409690900L, 3373263243L, 2922903613L, 349048087L, 4049532385L,
    3458779287L, 1737687814L, 287275672L, 645786941L, 1492233180L, 3925845678L, 3344829077L, 1669219217L, 665224162L,
    2679234088L, 1986576411L, 50610077L, 1080114376L, 1881648396L, 3818465156L, 1486861008L, 3824208930L, 1782008170L,
    4115911912L, 656413265L, 771498619L, 2709443211L, 1919820065L, 451888753L, 1449812173L, 2001941180L, 2997921765L,
    753032713L, 3011517640L, 2386888602L, 3181040472L, 1280522185L, 1036471598L, 1243809973L, 2985144032L, 2238294821L,
    557934351L, 347132246L, 1797956016L, 624L), None)
    setstate(init_state)
    lst, init_state = rand_list_of(n)

    print "original list:"
    print lst

    with lltype.scoped_alloc(rffi.CArray(rffi.LONGLONG), n) as arr:
        for i, k in enumerate(lst):
            arr[i] = k

        fnc(arr, 0, n - 1)  # inplace sort

        lst_s = sorted(lst)

        for i in range(n):
            assert lst_s[i] == arr[i], "%d != %d" % (lst_s[i], arr[i])


@may_spawn_proc
def test_partition_in_quicksort():
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

    fnc, (db, bdlgen) = fncptr_from_rpy_func(partition, [rffi.CArrayPtr(rffi.LONGLONG), lltype.Signed, lltype.Signed],
                                             lltype.Signed)
    bdlgen.mu.current_thread_as_mu_thread(rmu.null(rmu.MuCPtr))
    # fnc = partition

    n = 100
    from random import setstate
    init_state = (3, (
    2147483648L, 3430835514L, 2928424416L, 3147699060L, 2823572732L, 2905216632L, 1887281517L, 14272356L, 1356039141L,
    2741361235L, 1824725388L, 2228169284L, 2679861265L, 3150239284L, 657657570L, 1407124159L, 517316568L, 653526369L,
    139268705L, 3784719953L, 2212355490L, 3452491289L, 1232629882L, 1791207424L, 2898278956L, 1147783320L, 1824413680L,
    1993303973L, 2568444883L, 4228847642L, 4163974668L, 385627078L, 3663560714L, 320542554L, 1565882322L, 3416481154L,
    4219229298L, 315071254L, 778331393L, 3961037651L, 2951403614L, 3355970261L, 102946340L, 2509883952L, 215897963L,
    3361072826L, 689991350L, 3348092598L, 1763608447L, 2140226443L, 3813151178L, 2619956936L, 51244592L, 2130725065L,
    3867113849L, 1980820881L, 2600246771L, 3207535572L, 257556968L, 2223367443L, 3706150033L, 1711074250L, 4252385224L,
    3197142331L, 4139558716L, 748471849L, 2281163369L, 2596250092L, 2804492653L, 484240110L, 3726117536L, 2483815933L,
    2173995598L, 3765136999L, 3178931194L, 1237068319L, 3427263384L, 3958412830L, 2268556676L, 360704423L, 4113430429L,
    3758882140L, 3743971788L, 1685454939L, 488386L, 3511218911L, 3020688912L, 2168345327L, 3149651862L, 1472484695L,
    2011779229L, 1112533726L, 1873931730L, 2196153055L, 3806225492L, 1515074892L, 251489714L, 1958141723L, 2081062631L,
    3703490262L, 3211541213L, 1436109217L, 2664448365L, 2350764370L, 1285829042L, 3496997759L, 2306637687L, 1571644344L,
    1020052455L, 3114491401L, 2994766034L, 1518527036L, 994512437L, 1732585804L, 2089330296L, 2592371643L, 2377347339L,
    2617648350L, 1478066246L, 389918052L, 1126787130L, 2728695369L, 2921719205L, 3193658789L, 2101782606L, 4284039483L,
    2704867468L, 3843423543L, 119359906L, 1882384901L, 832276556L, 1862974878L, 1943541262L, 1823624942L, 2146680272L,
    333006125L, 929197835L, 639017219L, 1640196300L, 1424826762L, 2119569013L, 4259272802L, 2089277168L, 2030198981L,
    2950559216L, 621654826L, 3452546704L, 4085446289L, 3038316311L, 527272378L, 1679817853L, 450787204L, 3525043861L,
    3838351358L, 1558592021L, 3649888848L, 3328370698L, 3247166155L, 3855970537L, 1183088418L, 2778702834L, 2820277014L,
    1530905121L, 1434023607L, 3942716950L, 41643359L, 310637634L, 1537174663L, 4265200088L, 3126624846L, 2837665903L,
    446994733L, 85970060L, 643115053L, 1751804182L, 1480207958L, 2977093071L, 544778713L, 738954842L, 3370733859L,
    3242319053L, 2707786138L, 4041098196L, 1671493839L, 3420415077L, 2473516599L, 3949211965L, 3686186772L, 753757988L,
    220738063L, 772481263L, 974568026L, 3190407677L, 480257177L, 3620733162L, 2616878358L, 665763320L, 2808607644L,
    3851308236L, 3633157256L, 4240746864L, 1261222691L, 268963935L, 1449514350L, 4229662564L, 1342533852L, 1913674460L,
    1761163533L, 1974260074L, 739184472L, 3811507072L, 2880992381L, 3998389163L, 2673626426L, 2212222504L, 231447607L,
    2608719702L, 3509764733L, 2403318909L, 635983093L, 4233939991L, 2894463467L, 177171270L, 2962364044L, 1191007101L,
    882222586L, 1004217833L, 717897978L, 2125381922L, 626199402L, 3694698943L, 1373935523L, 762314613L, 2291077454L,
    2111081024L, 3758576304L, 2812129656L, 4067461097L, 3700761868L, 2281420733L, 197217625L, 460620692L, 506837624L,
    1532931238L, 3872395078L, 3629107738L, 2273221134L, 2086345980L, 1240615886L, 958420495L, 4059583254L, 3119201875L,
    3742950862L, 891360845L, 2974235885L, 87814219L, 4067521161L, 615939803L, 1881195074L, 2225917026L, 2775128741L,
    2996201447L, 1590546624L, 3960431955L, 1417477945L, 913935155L, 1610033170L, 3212701447L, 2545374014L, 2887105562L,
    2991635417L, 3194532260L, 1565555757L, 2142474733L, 621483430L, 2268177481L, 919992760L, 2022043644L, 2756890220L,
    881105937L, 2621060794L, 4262292201L, 480112895L, 2557060162L, 2367031748L, 2172434102L, 296539623L, 3043643256L,
    59166373L, 2947638193L, 1312917612L, 1798724013L, 75864164L, 339661149L, 289536004L, 422147716L, 1134944052L,
    1095534216L, 1231984277L, 239787072L, 923053211L, 1015393503L, 2558889580L, 4194512643L, 448088150L, 707905706L,
    2649061310L, 3081089715L, 3432955562L, 2217740069L, 1965789353L, 3320360228L, 3625802364L, 2420747908L, 3116949010L,
    442654625L, 2157578112L, 3603825090L, 3111995525L, 1124579902L, 101836896L, 3297125816L, 136981134L, 4253748197L,
    3809600572L, 1668193778L, 4146759785L, 3712590372L, 2998653463L, 3032597504L, 1046471011L, 2843821193L, 802959497L,
    3307715534L, 3226042258L, 1014478160L, 3105844949L, 3209150965L, 610876993L, 2563947590L, 2482526324L, 3913970138L,
    2812702315L, 4281779167L, 1026357391L, 2579486306L, 402208L, 3457975059L, 1714004950L, 2543595755L, 2421499458L,
    478932497L, 3117588180L, 1565800974L, 1757724858L, 1483685124L, 2262270397L, 3794544469L, 3986696110L, 2914756339L,
    1952061826L, 2672480198L, 3793151752L, 309930721L, 1861137379L, 94571340L, 1162935802L, 3681554226L, 4027302061L,
    21079572L, 446709644L, 1587253187L, 1845056582L, 3080553052L, 3575272255L, 2526224735L, 3569822959L, 2685900491L,
    918305237L, 1399881227L, 1554912161L, 703181091L, 738501299L, 269937670L, 1078548118L, 2313670525L, 3495159622L,
    2659487842L, 11394628L, 1222454456L, 3392065094L, 3426833642L, 1153231613L, 1234517654L, 3144547626L, 2148039080L,
    3790136587L, 684648337L, 3956093475L, 1384378197L, 2042781475L, 759764431L, 222267088L, 3187778457L, 3795259108L,
    2817237549L, 3494781277L, 3762880618L, 892345749L, 2153484401L, 721588894L, 779278769L, 3306398772L, 4221452913L,
    1981375723L, 379087895L, 1604791625L, 1426046977L, 4231163093L, 1344994557L, 1341041093L, 1072537134L, 1829925137L,
    3791772627L, 3176876700L, 2553745117L, 664821113L, 473469583L, 1076256869L, 2406012795L, 3141453822L, 4123012649L,
    3058620143L, 1785080140L, 1181483189L, 3587874749L, 1453504375L, 707249496L, 2022787257L, 2436320047L, 602521701L,
    483826957L, 821599664L, 3333871672L, 3024431570L, 3814441382L, 416508285L, 1217138244L, 3975201118L, 3077724941L,
    180118569L, 3754556886L, 4121534265L, 3495283397L, 700504668L, 3113972067L, 719371171L, 910731026L, 619936911L,
    2937105529L, 2039892965L, 3853404454L, 3783801801L, 783321997L, 1135195902L, 326690505L, 1774036419L, 3476057413L,
    1518029608L, 1248626026L, 427510490L, 3443223611L, 4087014505L, 2858955517L, 1918675812L, 3921514056L, 3929126528L,
    4048889933L, 1583842117L, 3742539544L, 602292017L, 3393759050L, 3929818519L, 3119818281L, 3472644693L, 1993924627L,
    4163228821L, 2943877721L, 3143487730L, 4087113198L, 1149082355L, 1713272081L, 1243627655L, 3511633996L, 3358757220L,
    3812981394L, 650044449L, 2143650644L, 3869591312L, 3719322297L, 386030648L, 2633538573L, 672966554L, 3498396042L,
    3907556L, 2308686209L, 2878779858L, 1475925955L, 2701537395L, 1448018484L, 2962578755L, 1383479284L, 3731453464L,
    3659512663L, 1521189121L, 843749206L, 2243090279L, 572717972L, 3400421356L, 3440777300L, 1393518699L, 1681924551L,
    466257295L, 568413244L, 3288530316L, 2951425105L, 2624424893L, 2410788864L, 2243174464L, 1385949609L, 2454100663L,
    1113953725L, 2127471443L, 1775715557L, 3874125135L, 1901707926L, 3152599339L, 2277843623L, 1941785089L, 3171888228L,
    802596998L, 3397391306L, 1743834429L, 395463904L, 2099329462L, 3761809163L, 262702111L, 1868879810L, 2887406426L,
    1160032302L, 4164116477L, 2287740849L, 3312176050L, 747117003L, 4048006270L, 3955419375L, 2724452926L, 3141695820L,
    791246424L, 524525849L, 1794277132L, 295485241L, 4125127474L, 825108028L, 1582794137L, 1259992755L, 2938829230L,
    912029932L, 1534496985L, 3075283272L, 4052041116L, 1125808104L, 2032938837L, 4008676545L, 1638361535L, 1649316497L,
    1302633381L, 4221627277L, 1206130263L, 3114681993L, 3409690900L, 3373263243L, 2922903613L, 349048087L, 4049532385L,
    3458779287L, 1737687814L, 287275672L, 645786941L, 1492233180L, 3925845678L, 3344829077L, 1669219217L, 665224162L,
    2679234088L, 1986576411L, 50610077L, 1080114376L, 1881648396L, 3818465156L, 1486861008L, 3824208930L, 1782008170L,
    4115911912L, 656413265L, 771498619L, 2709443211L, 1919820065L, 451888753L, 1449812173L, 2001941180L, 2997921765L,
    753032713L, 3011517640L, 2386888602L, 3181040472L, 1280522185L, 1036471598L, 1243809973L, 2985144032L, 2238294821L,
    557934351L, 347132246L, 1797956016L, 624L), None)
    setstate(init_state)
    lst, init_state = rand_list_of(n)

    with lltype.scoped_alloc(rffi.CArray(rffi.LONGLONG), n) as arr:
        for i, k in enumerate(lst):
            arr[i] = k

        idx = fnc(arr, 0, n - 1)

        first_partition = [
            -562164038,
            -2071388465,
            -663526532,
            77489857,
            -343649111,
            -1660130362,
            -1364581753,
            -2038184925,
            -1165174475,
            -1849978230,
            -1236284585,
            -347764193,
            -415184763,
            -864996653,
            -1431147879,
            -254259567,
            -948603419,
            -777817366,
            -762104870,
            118960100,
            -982992600,
            -291431596,
            -1300455919,
            98312853,
            -451757010,
            -127589060,
            -1770428162,
            -1836098229,
            -918293874,
            -337375506,
            -1787719536,
            -2086483893,
            -730620516,
            -365703180,
            -1528919012,
            -1666015908,
            75036665,
            -1068382947,
            -2097740676,
            -158140475,
            181349155,
            1134943658,
            926214681,
            1436898456,
            1896535137,
            654725403,
            964722898,
            829972680,
            777329866,
            726385788,
            1050914914,
            1280292061,
            727975360,
            1023937016,
            640384790,
            637969418,
            1884043455,
            1925731670,
            1057772537,
            1322685888,
            1351410892,
            945183403,
            2014860171,
            1918531212,
            955471993,
            1075682797,
            238111242,
            1508508491,
            828291293,
            1789417882,
            1102829861,
            1435471727,
            1980476539,
            1344494232,
            1771547746,
            784699465,
            478704353,
            1664007571,
            511675340,
            1174338681,
            835473661,
            1039011592,
            1901271880,
            1983373831,
            782060246,
            1847820592,
            1751300194,
            558677750,
            1338238899,
            1313544470,
            232877310,
            599055646,
            873066597,
            1433425901,
            1192634012,
            1322616334,
            2026877877,
            1070749459,
            1899988061,
            632945766,
        ]

        assert idx == 40
        for i in range(n):
            assert arr[i] == first_partition[i], "%d != %d" % (arr[i], first_partition[i])


@may_spawn_proc
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

                    h = fnc(a)
                    print '%s -> %s -> %s -> %s' % (h.val, h.nxt.val, h.nxt.nxt.val, h.nxt.nxt.nxt.val)
                    assert h.val == 'd'
                    assert h.nxt.val == 'c'
                    assert h.nxt.nxt.val == 'b'
                    assert h.nxt.nxt.nxt.val == 'a'
                    assert h.nxt.nxt.nxt.nxt == lltype.nullptr(Node)


@may_spawn_proc
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

    (fnp, _), (mu, ctx, bldr) = fncptr_from_py_script(build_test_bundle, None, 'fib', [ctypes.c_longlong])

    mu.current_thread_as_mu_thread(rmu.null(rmu.MuCPtr))

    assert fnp(20) == 6765


@may_spawn_proc
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

    (fnp, _), (mu, ctx, bldr) = fncptr_from_py_script(build_test_bundle, None, 'test_fnc')

    mu.current_thread_as_mu_thread(rmu.null(rmu.MuCPtr))
    assert fnp() == 1


@may_spawn_proc
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

    (fnp, _), (mu, ctx, bldr) = fncptr_from_py_script(build_test_bundle, None, 'test_fnc')

    mu.current_thread_as_mu_thread(rmu.null(rmu.MuCPtr))
    assert fnp() == 0


@may_spawn_proc
def test_throw():
    def build_test_bundle(bldr, rmu):
        """
        Builds the following test bundle.
            .typedef @i64 = int<64>
            .typedef @void = void
            .typedef @refi64 = ref<@i64>
            .typedef @refvoid = ref<@void>
            .const @c_10 <@i64> = 10
            .const @c_20 <@i64> = 20
            .const @c_42 <@i64> = 42
            .funcsig @sig_i64_i64 = (@i64) -> (@i64)
            .funcdef @throw_fnc VERSION @throw_fnc.v1 <@sig_i64_i64> {
                %blk0(<@i64> %num):
                    %cmpres = SLT <@i64> %num @c_42
                    BRANCH2 %cmpres %blk1() %blk2()
                %blk1():
                    %excobj = NEW <@i64>
                    %iref_obj = GETIREF <@i64> %excobj
                    STORE <@i64> %iref_obj @c_20
                    THROW %excobj
                %blk2():
                    RET (@c_10)
            }
            .funcdef @test_fnc VERSION @test_fnc.v1 <@sig_i64_i64> {
                %blk0(<@i64> %num):
                    %res = CALL <@sig_i64_i64> @throw_fnc (%num) EXC(%blk1(%res) %blk2())
                %blk1(<@i64> %rtn):
                    RET %rtn
                %blk2()[%excobj]:
                    %ri64 = REFCAST <@refvoid @refi64> %excobj
                    %iri64 = GETIREF <@i64> %excobj
                    %obj = LOAD <@i64> %iri64
                    BRANCH %blk1(%obj)
            }
        :type bldr: rpython.rlib.rmu.MuIRBuilder
        :type rmu: rpython.rlib.rmu
        :return: (rmu.MuVM(), rmu.MuCtx, rmu.MuIRBuilder, MuID, MuID)
        """
        void = bldr.gen_sym("@void"); bldr.new_type_void(void)
        i64 = bldr.gen_sym("@i64"); bldr.new_type_int(i64, 64)
        refi64 = bldr.gen_sym("@refi64"); bldr.new_type_ref(refi64, i64)
        refvoid = bldr.gen_sym("@refvoid"); bldr.new_type_ref(refvoid, void)

        c_10 = bldr.gen_sym("@c_10"); bldr.new_const_int(c_10, i64, 10)
        c_20 = bldr.gen_sym("@c_20"); bldr.new_const_int(c_20, i64, 20)
        c_42 = bldr.gen_sym("@c_42"); bldr.new_const_int(c_42, i64, 42)

        sig_i64_i64 = bldr.gen_sym("@sig_i64_i64"); bldr.new_funcsig(sig_i64_i64, [i64], [i64])

        test_fnc = bldr.gen_sym("@test_fnc"); bldr.new_func(test_fnc, sig_i64_i64)
        throw_fnc = bldr.gen_sym("@throw_fnc"); bldr.new_func(throw_fnc, sig_i64_i64)

        throw_fnc_v1 = bldr.gen_sym("@throw_fnc.v1")
        blk0 = bldr.gen_sym("@throw_fnc.v1.blk0")
        blk1 = bldr.gen_sym("@throw_fnc.v1.blk1")
        blk2 = bldr.gen_sym("@throw_fnc.v1.blk2")

        # blk0
        num = bldr.gen_sym("@throw_fnc.v1.blk0.num")
        cmpres = bldr.gen_sym("@throw_fnc.v1.blk0.cmpres")
        op_slt = bldr.gen_sym(); bldr.new_cmp(op_slt, cmpres, rmu.MuCmpOptr.SLT, i64, num, c_42)
        dst_t = bldr.gen_sym(); bldr.new_dest_clause(dst_t, blk1, [])
        dst_f = bldr.gen_sym(); bldr.new_dest_clause(dst_f, blk2, [])
        op_br2 = bldr.gen_sym(); bldr.new_branch2(op_br2, cmpres, dst_t, dst_f)
        bldr.new_bb(blk0, [num], [i64], rmu.MU_NO_ID, [op_slt, op_br2])

        # blk1
        excobj = bldr.gen_sym("@throw_fnc.v1.blk1.excobj")
        iref_obj = bldr.gen_sym("@throw_fnc.v1.blk1.iref_obj")
        op_new = bldr.gen_sym(); bldr.new_new(op_new, excobj, i64)
        op_getiref = bldr.gen_sym(); bldr.new_getiref(op_getiref, iref_obj, i64, excobj)
        op_store = bldr.gen_sym(); bldr.new_store(op_store, False, rmu.MuMemOrd.NOT_ATOMIC, i64, iref_obj, c_20)
        op_throw = bldr.gen_sym(); bldr.new_throw(op_throw, excobj)
        bldr.new_bb(blk1, [], [], rmu.MU_NO_ID, [op_new, op_getiref, op_store, op_throw])

        # blk2
        op_ret = bldr.gen_sym(); bldr.new_ret(op_ret, [c_10])
        bldr.new_bb(blk2, [], [], rmu.MU_NO_ID, [op_ret])

        bldr.new_func_ver(throw_fnc_v1, throw_fnc, [blk0, blk1, blk2])

        test_fnc_v1 = bldr.gen_sym("@test_fnc.v1")
        blk0 = bldr.gen_sym("@test_fnc.v1.blk0")
        blk1 = bldr.gen_sym("@test_fnc.v1.blk1")
        blk2 = bldr.gen_sym("@test_fnc.v1.blk2")

        # blk0
        num = bldr.gen_sym("@test_fnc.v1.blk0.num")
        res = bldr.gen_sym("@test_fnc.v1.blk0.res")
        dst_nor = bldr.gen_sym(); bldr.new_dest_clause(dst_nor, blk1, [res])
        dst_exc = bldr.gen_sym(); bldr.new_dest_clause(dst_exc, blk2, [])
        exc = bldr.gen_sym(); bldr.new_exc_clause(exc, dst_nor, dst_exc)
        op_call = bldr.gen_sym(); bldr.new_call(op_call, [res], sig_i64_i64, throw_fnc, [num], exc)
        bldr.new_bb(blk0, [num], [i64], rmu.MU_NO_ID, [op_call])

        # blk1
        rtn = bldr.gen_sym("@test_fnc.v1.blk1.rtn")
        op_ret = bldr.gen_sym(); bldr.new_ret(op_ret, [rtn])
        bldr.new_bb(blk1, [rtn], [i64], rmu.MU_NO_ID, [op_ret])

        # blk2
        excobj = bldr.gen_sym("@test_fnc.v1.blk2.excobj")
        ri64 = bldr.gen_sym("@test_fnc.v1.blk2.ri64")
        iri64 = bldr.gen_sym("@test_fnc.v1.blk2.iri64")
        obj = bldr.gen_sym("@test_fnc.v1.blk2.obj")
        op_refcast = bldr.gen_sym(); bldr.new_conv(op_refcast, ri64, rmu.MuConvOptr.REFCAST, refvoid, refi64, excobj)
        op_getiref = bldr.gen_sym(); bldr.new_getiref(op_getiref, iri64, i64, ri64)
        op_load = bldr.gen_sym(); bldr.new_load(op_load, obj, False, rmu.MuMemOrd.NOT_ATOMIC, i64, iri64)
        dst = bldr.gen_sym(); bldr.new_dest_clause(dst, blk1, [obj])
        op_br = bldr.gen_sym(); bldr.new_branch(op_br, dst)
        bldr.new_bb(blk2, [], [], excobj, [op_refcast, op_getiref, op_load, op_br])

        bldr.new_func_ver(test_fnc_v1, test_fnc, [blk0, blk1, blk2])

        return {
            "test_fnc": test_fnc,
            "test_fnc_sig": sig_i64_i64,
            "result_type": i64,
            "@i64": i64,
            "@refi64": refi64
        }

    (fnp, _), (mu, ctx, bldr) = fncptr_from_py_script(build_test_bundle, None, 'test_fnc', [ctypes.c_int64],
                                                      ctypes.c_int64, mode=ctypes.RTLD_GLOBAL)
    mu.current_thread_as_mu_thread(rmu.null(rmu.MuCPtr))
    assert fnp(0) == 20
    assert fnp(100) == 10


@may_spawn_proc
def test_exception_stack_unwind():
    def build_test_bundle(bldr, rmu):
        """
        Builds the following test bundle.
            .typedef @i64 = int<64>
            .typedef @void = void
            .typedef @refi64 = ref<@i64>
            .typedef @refvoid = ref<@void>
            .const @c_10 <@i64> = 10
            .const @c_20 <@i64> = 20
            .const @c_42 <@i64> = 42
            .funcsig @sig_i64_i64 = (@i64) -> (@i64)
            .funcdef @throw_fnc VERSION @throw_fnc.v1 <@sig_i64_i64> {
                %blk0(<@i64> %num):
                    %cmpres = SLT <@i64> %num @c_42
                    BRANCH2 %cmpres %blk1() %blk2()
                %blk1():
                    %excobj = NEW <@i64>
                    %iref_obj = GETIREF <@i64> %excobj
                    STORE <@i64> %iref_obj @c_20
                    THROW %excobj
                %blk2():
                    RET (@c_10)
            }
            .funcdef @intermediate_fnc VERSION @intermediate_fnc.v1 <@sig_i64_i64> {
                %blk0(<@i64> %num):
                    %res = CALL <@sig_i64_i64> @throw_fnc (%num)
                    RET (%res)
            }
            .funcdef @test_fnc VERSION @test_fnc.v1 <@sig_i64_i64> {
                %blk0(<@i64> %num):
                    %res = CALL <@sig_i64_i64> @intermediate_fnc (%num) EXC(%blk1(%res) %blk2())
                %blk1(<@i64> %rtn):
                    RET %rtn
                %blk2()[%excobj]:
                    %ri64 = REFCAST <@refvoid @refi64> %excobj
                    %iri64 = GETIREF <@i64> %excobj
                    %obj = LOAD <@i64> %iri64
                    BRANCH %blk1(%obj)
            }
        :type bldr: rpython.rlib.rmu.MuIRBuilder
        :type rmu: rpython.rlib.rmu
        :return: (rmu.MuVM(), rmu.MuCtx, rmu.MuIRBuilder, MuID, MuID)
        """
        void = bldr.gen_sym("@void"); bldr.new_type_void(void)
        i64 = bldr.gen_sym("@i64"); bldr.new_type_int(i64, 64)
        refi64 = bldr.gen_sym("@refi64"); bldr.new_type_ref(refi64, i64)
        refvoid = bldr.gen_sym("@refvoid"); bldr.new_type_ref(refvoid, void)

        c_10 = bldr.gen_sym("@c_10"); bldr.new_const_int(c_10, i64, 10)
        c_20 = bldr.gen_sym("@c_20"); bldr.new_const_int(c_20, i64, 20)
        c_42 = bldr.gen_sym("@c_42"); bldr.new_const_int(c_42, i64, 42)

        sig_i64_i64 = bldr.gen_sym("@sig_i64_i64"); bldr.new_funcsig(sig_i64_i64, [i64], [i64])

        test_fnc = bldr.gen_sym("@test_fnc"); bldr.new_func(test_fnc, sig_i64_i64)
        throw_fnc = bldr.gen_sym("@throw_fnc"); bldr.new_func(throw_fnc, sig_i64_i64)
        intermediate_fnc = bldr.gen_sym("@intermediate_fnc"); bldr.new_func(intermediate_fnc, sig_i64_i64)

        throw_fnc_v1 = bldr.gen_sym("@throw_fnc.v1")
        blk0 = bldr.gen_sym("@throw_fnc.v1.blk0")
        blk1 = bldr.gen_sym("@throw_fnc.v1.blk1")
        blk2 = bldr.gen_sym("@throw_fnc.v1.blk2")

        # blk0
        num = bldr.gen_sym("@throw_fnc.v1.blk0.num")
        cmpres = bldr.gen_sym("@throw_fnc.v1.blk0.cmpres")
        op_slt = bldr.gen_sym(); bldr.new_cmp(op_slt, cmpres, rmu.MuCmpOptr.SLT, i64, num, c_42)
        dst_t = bldr.gen_sym(); bldr.new_dest_clause(dst_t, blk1, [])
        dst_f = bldr.gen_sym(); bldr.new_dest_clause(dst_f, blk2, [])
        op_br2 = bldr.gen_sym(); bldr.new_branch2(op_br2, cmpres, dst_t, dst_f)
        bldr.new_bb(blk0, [num], [i64], rmu.MU_NO_ID, [op_slt, op_br2])

        # blk1
        excobj = bldr.gen_sym("@throw_fnc.v1.blk1.excobj")
        iref_obj = bldr.gen_sym("@throw_fnc.v1.blk1.iref_obj")
        op_new = bldr.gen_sym(); bldr.new_new(op_new, excobj, i64)
        op_getiref = bldr.gen_sym(); bldr.new_getiref(op_getiref, iref_obj, i64, excobj)
        op_store = bldr.gen_sym(); bldr.new_store(op_store, False, rmu.MuMemOrd.NOT_ATOMIC, i64, iref_obj, c_20)
        op_throw = bldr.gen_sym(); bldr.new_throw(op_throw, excobj)
        bldr.new_bb(blk1, [], [], rmu.MU_NO_ID, [op_new, op_getiref, op_store, op_throw])

        # blk2
        op_ret = bldr.gen_sym(); bldr.new_ret(op_ret, [c_10])
        bldr.new_bb(blk2, [], [], rmu.MU_NO_ID, [op_ret])

        bldr.new_func_ver(throw_fnc_v1, throw_fnc, [blk0, blk1, blk2])

        intermediate_fnc_v1 = bldr.gen_sym("@intermediate_fnc.v1")
        blk0 = bldr.gen_sym("@intermediate_fnc.v1.blk0")
        num = bldr.gen_sym("@intermediate_fnc.v1.blk0.num")
        res = bldr.gen_sym("@intermediate_fnc.v1.blk0.res")
        op_call = bldr.gen_sym(); bldr.new_call(op_call, [res], sig_i64_i64, throw_fnc, [num])
        op_ret = bldr.gen_sym(); bldr.new_ret(op_ret, [res])
        bldr.new_bb(blk0, [num], [i64], rmu.MU_NO_ID, [op_call, op_ret])

        bldr.new_func_ver(intermediate_fnc_v1, intermediate_fnc, [blk0])

        test_fnc_v1 = bldr.gen_sym("@test_fnc.v1")
        blk0 = bldr.gen_sym("@test_fnc.v1.blk0")
        blk1 = bldr.gen_sym("@test_fnc.v1.blk1")
        blk2 = bldr.gen_sym("@test_fnc.v1.blk2")

        # blk0
        num = bldr.gen_sym("@test_fnc.v1.blk0.num")
        res = bldr.gen_sym("@test_fnc.v1.blk0.res")
        dst_nor = bldr.gen_sym(); bldr.new_dest_clause(dst_nor, blk1, [res])
        dst_exc = bldr.gen_sym(); bldr.new_dest_clause(dst_exc, blk2, [])
        exc = bldr.gen_sym(); bldr.new_exc_clause(exc, dst_nor, dst_exc)
        op_call = bldr.gen_sym(); bldr.new_call(op_call, [res], sig_i64_i64, intermediate_fnc, [num], exc)
        bldr.new_bb(blk0, [num], [i64], rmu.MU_NO_ID, [op_call])

        # blk1
        rtn = bldr.gen_sym("@test_fnc.v1.blk1.rtn")
        op_ret = bldr.gen_sym(); bldr.new_ret(op_ret, [rtn])
        bldr.new_bb(blk1, [rtn], [i64], rmu.MU_NO_ID, [op_ret])

        # blk2
        excobj = bldr.gen_sym("@test_fnc.v1.blk2.excobj")
        ri64 = bldr.gen_sym("@test_fnc.v1.blk2.ri64")
        iri64 = bldr.gen_sym("@test_fnc.v1.blk2.iri64")
        obj = bldr.gen_sym("@test_fnc.v1.blk2.obj")
        op_refcast = bldr.gen_sym(); bldr.new_conv(op_refcast, ri64, rmu.MuConvOptr.REFCAST, refvoid, refi64, excobj)
        op_getiref = bldr.gen_sym(); bldr.new_getiref(op_getiref, iri64, i64, ri64)
        op_load = bldr.gen_sym(); bldr.new_load(op_load, obj, False, rmu.MuMemOrd.NOT_ATOMIC, i64, iri64)
        dst = bldr.gen_sym(); bldr.new_dest_clause(dst, blk1, [obj])
        op_br = bldr.gen_sym(); bldr.new_branch(op_br, dst)
        bldr.new_bb(blk2, [], [], excobj, [op_refcast, op_getiref, op_load, op_br])

        bldr.new_func_ver(test_fnc_v1, test_fnc, [blk0, blk1, blk2])

        return {
            "test_fnc": test_fnc,
            "test_fnc_sig": sig_i64_i64,
            "result_type": i64,
            "@i64": i64,
            "@refi64": refi64
        }
    (fnp, _), (mu, ctx, bldr) = fncptr_from_py_script(build_test_bundle, None, 'test_fnc', [ctypes.c_int64],
                                                      ctypes.c_int64, mode=ctypes.RTLD_GLOBAL)
    mu.current_thread_as_mu_thread(rmu.null(rmu.MuCPtr))
    assert fnp(0) == 20
    assert fnp(100) == 10


def run_boot_image(entry, output, has_c_main_sig = False, args = [], impl=os.getenv('MU_IMPL', 'zebu'), vmargs = ""):
    from rpython.translator.interactive import Translation
    from rpython.translator.platform import log as log_platform
    if has_c_main_sig:
        t = Translation(entry, [rffi.INT, rffi.CCHARPP], backend='mu', impl=impl, codegen='api', vmargs=vmargs)
        t.driver.disable(['entrypoint_mu'])
    else:
        t = Translation(entry, None, backend='mu', impl=impl, codegen='c', vmargs=vmargs)

    t.driver.standalone = True  # force standalone
    t.driver.exe_name = output
   
    #t.backendopt(inline=True, mallocs=True)
    #t.view()
    #t.mutype()
    #t.view()

    db, mugen, epf_name = t.compile_mu()
    exe = py.path.local(output)

    if impl == 'zebu':
        # zebu
        exe.chmod(stat.S_IRWXU)
        eci = rffi.ExternalCompilationInfo(library_dirs=[str(db.libsupport_path.dirpath())])
        res = platform.execute(str(exe), args, compilation_info=eci)
    else:
        from rpython.rlib.rmu import holstein
        runmu = py.path.local(holstein.mu_dir).join('..', 'tools', 'runmu.sh')
        flags = ['--vmLog=ERROR', '--losSize=780M', '--sosSize=780M']
        log_platform.execute(' '.join([str(runmu)] + flags + [str(exe)] + args))
        res = platform.execute(runmu, flags + [str(exe)] + args)

    return res

# not using this function at the moment
def check(actual, expect):
    c_exit = rffi.llexternal('exit', [rffi.INT], lltype.Void, _nowrapper=True)

    if actual != expect:
        print 'actual: %d' % actual
        print 'expect: %d' % expect
        print 'assertion fails'
        c_exit(rffi.cast(rffi.INT, actual))

@may_spawn_proc
def test_make_boot_image_simple():
    c_printf = rffi.llexternal('printf', [rffi.CCHARP], rffi.INT, _nowrapper=True)
    c_putchar = rffi.llexternal('putchar', [rffi.CHAR], rffi.INT, _nowrapper=True)
    c_exit = rffi.llexternal('exit', [rffi.INT], lltype.Void, _nowrapper=True)

    def pypy_mu_entry(argc, argv):
        for i in range(argc):
            c_printf(argv[i])
            c_putchar('\n')
        c_exit(rffi.cast(rffi.INT, 0))
    return 0

    res = run_boot_image(pypy_mu_entry, '/tmp/test_make_boot_image_mu', True, ['abc', '123'])
    exe = '/tmp/test_make_boot_image_mu'
    
    assert res.returncode == 0, res.err
    assert res.out == '%s\nabc\n123\n' % exe

@may_spawn_proc
def test_rpytarget_print_argv():
    def main(argv):
        print argv
        return 0

    res = run_boot_image(main, '/tmp/test_printargv_mu', args = ['abc', '123'])
    exe = '/tmp/test_printargv_mu'

    assert res.returncode == 0, res.err
    assert res.out == '[%s, abc, 123]\n' % exe

@may_spawn_proc
def test_rpython_helloworld():
    def main(argv):
        print "hello world"
        return 0

    res = run_boot_image(main, '/tmp/test_helloworld_mu')

    assert res.returncode == 0, res.err
    assert res.out == 'hello world\n'

@may_spawn_proc
def test_rpython_print_number():

    def main(argv):
        print 233
        return 0

    res = run_boot_image(main, '/tmp/test_print_number_mu')

    assert res.returncode == 0, res.err
    assert res.out == '233\n'

@may_spawn_proc
def test_rpython_print_fmt():
    def main(argv):
        print "hello world %s" % argv[1]
        return 0

    res = run_boot_image(main, '/tmp/test_print_fmt', args = ['mu'])

    assert res.returncode == 0, res.err
    assert res.out == 'hello world mu\n'

@may_spawn_proc
def test_rpython_main():
    def main(argv):
        return 0

    res = run_boot_image(main, '/tmp/test_main')

    assert res.returncode == 0, res.err

@may_spawn_proc
def test_rpytarget_sha1sum():
    john1 = \
'''
In the beginning was the Word, and the Word was with God, and the Word was God.
He was in the beginning with God.
All things were made through him, and without him was not any thing made that was made.
In him was life, and the life was the light of men.
The light shines in the darkness, and the darkness has not overcome it.
'''
    test_file = py.path.local('/tmp/john1.txt')
    with test_file.open('w') as fp:
        fp.write(john1)

    from rpython.translator.goal.targetsha1sum import entry_point
    res = run_boot_image(entry_point, '/tmp/test_sha1sum_mu', args=['/tmp/john1.txt'])

    assert res.returncode == 0, res.err
    assert res.out == '53b45a7e3fb6ccb2d9e43c45cb57b6b56c784def /tmp/john1.txt\n'

@may_spawn_proc
def test_linked_list():
    class Node:
        def __init__(self, data, nxt):
            self.data = data
            self.nxt = nxt

    l = Node(3, Node(2, Node(1, Node(0, None))))

    def main(argv):
        idx = int(argv[1])
        if idx >= 4:
            raise IndexError
        nd = l
        while idx > 0:
            nd = nd.nxt
            idx -= 1
        print nd.data
        return 0

    res = run_boot_image(main, '/tmp/test_linked_list-mu', args=['2'])
    assert res.returncode == 0, res.err
    assert res.out == '1\n'

@may_spawn_proc
def test_rpytarget_richards0():
    from rpython.translator.goal.richards import entry_point
    def main(argv):
        res, t0, t1 = entry_point(int(argv[1]))
        return 0

    res = run_boot_image(main, '/tmp/test_richards-mu', args=['5'])
    assert res.returncode == 0, res.err

@may_spawn_proc
def test_rpytarget_richards_measure_time():
    from rpython.translator.goal.richards import entry_point
    def main(argv):
        iterations = int(argv[1])
        res, t0, t1 = entry_point(iterations)
        print 'result =', res
        print 'avg time =', (t1 - t0) / iterations
        return 0

    res = run_boot_image(main, '/tmp/test_richards_measure_time-mu', args=['5'])
    assert res.returncode == 0, res.err

@may_spawn_proc
def test_rpython_print_time():
    import time
    def main(argv):
        print time.time()
        return 0

    res = run_boot_image(main, '/tmp/test_print_time')
    assert res.returncode == 0, res.err

@may_spawn_proc
def test_rpython_time_diff():
    import time
    def main(argv):
        t1 = time.time()
        t2 = time.time()
        if t2 >= t1:
            return 0
        else:
            return 1

    res = run_boot_image(main, '/tmp/test_time_diff')
    assert res.returncode == 0, res.err

@may_spawn_proc
def test_dtoa():
    from rpython.rlib.rdtoa import dtoa
    from rpython.translator.mu.tool.debug_print import print_
    def main(argv):
        print_(dtoa(3.14))
        return 0

    res = run_boot_image(main, '/tmp/test_print_float-mu', args=['2'])
    assert res.returncode == 0, res.err
    assert res.out == '3.14\n'

@may_spawn_proc
def test_rpytarget_testdicts():
    from rpython.translator.goal.targettestdicts import entry_point

    res = run_boot_image(entry_point, '/tmp/test_testdicts-mu',
                         args=['d', '1534'], vmargs="--gc-immixspace-size=536870912 --gc-lospace-size=536870912")
    assert res.returncode == 0, res.err
    assert res.out == '0x5fe\n'

@may_spawn_proc
def test_nbody():
    """N-body benchmark from the Computer Language Benchmarks Game.

    This is intended to support Unladen Swallow's perf.py. Accordingly, it has been
    modified from the Shootout version:
    - Accept standard Unladen Swallow benchmark options.
    - Run report_energy()/advance() in a loop.
    - Reimplement itertools.combinations() to work with older Python versions.
    """

    # Pulled from http://shootout.alioth.debian.org/u64q/benchmark.php?test=nbody&lang=python&id=4
    # Contributed by Kevin Carson.
    # Modified by Tupteq, Fredrik Johansson, and Daniel Nanz.

    import math

    def combinations(l):
        """Pure-Python implementation of itertools.combinations(l, 2)."""
        result = []
        for x in xrange(len(l) - 1):
            ls = l[x+1:]
            for y in ls:
                result.append((l[x],y))
        return result


    PI = 3.14159265358979323
    SOLAR_MASS = 4 * PI * PI
    DAYS_PER_YEAR = 365.24

    BODIES = {
        'sun': ([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], SOLAR_MASS),

        'jupiter': ([4.84143144246472090e+00,
                     -1.16032004402742839e+00,
                     -1.03622044471123109e-01],
                    [1.66007664274403694e-03 * DAYS_PER_YEAR,
                     7.69901118419740425e-03 * DAYS_PER_YEAR,
                     -6.90460016972063023e-05 * DAYS_PER_YEAR],
                    9.54791938424326609e-04 * SOLAR_MASS),

        'saturn': ([8.34336671824457987e+00,
                    4.12479856412430479e+00,
                    -4.03523417114321381e-01],
                   [-2.76742510726862411e-03 * DAYS_PER_YEAR,
                    4.99852801234917238e-03 * DAYS_PER_YEAR,
                    2.30417297573763929e-05 * DAYS_PER_YEAR],
                   2.85885980666130812e-04 * SOLAR_MASS),

        'uranus': ([1.28943695621391310e+01,
                    -1.51111514016986312e+01,
                    -2.23307578892655734e-01],
                   [2.96460137564761618e-03 * DAYS_PER_YEAR,
                    2.37847173959480950e-03 * DAYS_PER_YEAR,
                    -2.96589568540237556e-05 * DAYS_PER_YEAR],
                   4.36624404335156298e-05 * SOLAR_MASS),

        'neptune': ([1.53796971148509165e+01,
                     -2.59193146099879641e+01,
                     1.79258772950371181e-01],
                    [2.68067772490389322e-03 * DAYS_PER_YEAR,
                     1.62824170038242295e-03 * DAYS_PER_YEAR,
                     -9.51592254519715870e-05 * DAYS_PER_YEAR],
                    5.15138902046611451e-05 * SOLAR_MASS) }


    SYSTEM = list(BODIES.values())
    PAIRS = combinations(SYSTEM)

    def advance(dt, n, bodies=SYSTEM, pairs=PAIRS):
        for i in xrange(n):
            for (([x1, y1, z1], v1, m1),
                 ([x2, y2, z2], v2, m2)) in pairs:
                dx = x1 - x2
                dy = y1 - y2
                dz = z1 - z2
                mag = dt * math.pow((dx * dx + dy * dy + dz * dz), (-1.5))
                b1m = m1 * mag
                b2m = m2 * mag
                v1[0] -= dx * b2m
                v1[1] -= dy * b2m
                v1[2] -= dz * b2m
                v2[0] += dx * b1m
                v2[1] += dy * b1m
                v2[2] += dz * b1m
            for (r, [vx, vy, vz], m) in bodies:
                r[0] += dt * vx
                r[1] += dt * vy
                r[2] += dt * vz

    def report_energy(bodies=SYSTEM, pairs=PAIRS, e=0.0):
        for (((x1, y1, z1), v1, m1),
             ((x2, y2, z2), v2, m2)) in pairs:
            dx = x1 - x2
            dy = y1 - y2
            dz = z1 - z2
            e -= (m1 * m2) / math.pow((dx * dx + dy * dy + dz * dz), 0.5)
        for (r, [vx, vy, vz], m) in bodies:
            e += m * (vx * vx + vy * vy + vz * vz) / 2.
        return e

    def offset_momentum(ref, bodies=SYSTEM, px=0.0, py=0.0, pz=0.0):
        for (r, [vx, vy, vz], m) in bodies:
            px -= vx * m
            py -= vy * m
            pz -= vz * m
        (r, v, m) = ref
        v[0] = px / m
        v[1] = py / m
        v[2] = pz / m

    def test_nbody(iterations):
        offset_momentum(BODIES['sun'])
        e = report_energy()
        for i in xrange(iterations):
            advance(0.01, 20000)
            e = report_energy()
        return e

    def main(argv):
        print test_nbody(int(argv[1]))
        return 0

    res = run_boot_image(main, '/tmp/test_nbody-mu', args=['5'])
    assert res.returncode == 0, res.err
    assert res.out == '-0.169080\n'


@may_spawn_proc
def test_float():
    from math import sin, cos, sqrt

    class Point(object):

        def __init__(self, i):
            self.x = x = sin(i)
            self.y = cos(i) * 3
            self.z = (x * x) / 2

        def __repr__(self):
            return "<Point: x=%s, y=%s, z=%s>" % (self.x, self.y, self.z)

        def normalize(self):
            x = self.x
            y = self.y
            z = self.z
            norm = sqrt(x * x + y * y + z * z)
            self.x /= norm
            self.y /= norm
            self.z /= norm

        def maximize(self, other):
            self.x = self.x if self.x > other.x else other.x
            self.y = self.y if self.y > other.y else other.y
            self.z = self.z if self.z > other.z else other.z
            return self


    def maximize(points):
        next = points[0]
        for p in points[1:]:
            next = next.maximize(p)
        return next

    def benchmark(n):
        points = [None] * n
        for i in xrange(n):
            points[i] = Point(i)
        for p in points:
            p.normalize()
        return maximize(points)

    POINTS = 100

    def main(argv):
        o = None
        for i in xrange(int(argv[1])):
            o = benchmark(POINTS)
        if o:
            print (o.x, o.y, o.z)
        else:
            print 'NULL'
        return 0

    res = run_boot_image(main, '/tmp/test_float-mu', args=['5'])
    assert res.returncode == 0, res.err
    assert res.out == '(0.893876, 1.000000, 0.447179)\n'

@pytest.mark.xfail(reason='int128 not implemented')
@may_spawn_proc
def test_RPySOM():
    from som.vm.universe import main, Exit

    def entry_point(argv):
        try:
            main(argv)
        except Exit, e:
            return e.code
        except Exception, e:
            os.write(2, "ERROR: %s thrown during execution.\n" % e)
            return 1
        return 1

    RPYSOM = os.environ.get('RPYSOM', str(py.path.local(__file__).join('..', 'RPySOM')))

    res = run_boot_image(entry_point, '/tmp/RPySOM-no-jit-mu',
                         args=['-cp', '%(RPYSOM)s/Smalltalk' % locals(),
                               '%(RPYSOM)s/TestSuite/TestHarness.som' % locals()])
    assert res.returncode == 0, res.err
    expected_out = \
"""\
Testing...
Running test EmptyTest
Running test SystemTest
Running test ArrayTest
Running test ClassLoadingTest
Running test ClosureTest
Running test CoercionTest
Running test CompilerReturnTest
Running test DoubleTest
Running test HashTest
Running test IntegerTest
Warning: Test instance of IntegerTest failed: Identity failed. Expected: true, but Actual: false
Warning: Test instance of IntegerTest failed: Identity failed. Expected: true, but Actual: false
Running test ObjectSizeTest
Warning: Test instance of ObjectSizeTest failed: Plain object does not have size 1.
Warning: Test instance of ObjectSizeTest failed: Integer object does not have size 1.
Warning: Test instance of ObjectSizeTest failed: hello String object does not have size 1.
Warning: Test instance of ObjectSizeTest failed: Empty array object does not have size 1.
Warning: Test instance of ObjectSizeTest failed: Array object (length 4) does not have size 5.
Running test PreliminaryTest
Running test ReflectionTest
Running test SelfBlockTest
Running test SuperTest
Running test SymbolTest
Running test VectorTest
Running test BlockTest
Running test StringTest
Running test ClassStructureTest
Definition of Class changed. Testcase needs to be updated.
Running test DoesNotUnderstandTest
...done
"""
    assert res.out == expected_out


if __name__ == '__main__':
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument('testfnc', help="Test function name")
    opts = parser.parse_args()

    globals()[opts.testfnc]()
