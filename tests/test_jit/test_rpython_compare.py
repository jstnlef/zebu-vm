from rpython.rtyper.lltypesystem import rffi
from util import fncptr_from_rpy_func, may_spawn_proc


# disabled all tests on u64 MAX boundary
# rpython int is i64

@may_spawn_proc
def test_rpython_int_cmp():
    def int_cmp(a, b):
        if a > b:
            return 1
        elif a == b:
            return 0
        else:
            return -1
    
    mu_int_cmp, _ = fncptr_from_rpy_func(int_cmp, [rffi.LONGLONG, rffi.LONGLONG], rffi.LONGLONG)
    
    assert mu_int_cmp(1, 0) == 1
    assert mu_int_cmp(0, 1) == -1
    assert mu_int_cmp(0, 0) == 0
    assert mu_int_cmp(1, 1) == 0
    
    assert mu_int_cmp(1, -1) == 1
    assert mu_int_cmp(-1, 1) == -1
    
    assert mu_int_cmp(-1, -2) == 1
    assert mu_int_cmp(-2, -1) == -1
    assert mu_int_cmp(-1, -1) == 0

    assert mu_int_cmp(9223372036854775807, -9223372036854775808) == 1

#    assert mu_int_cmp(18446744073709551615, 9223372036854775807) == -1
#    assert mu_int_cmp(18446744073709551615, -9223372036854775808) == 1
#    assert mu_int_cmp(18446744073709551615, -1) == 0

@may_spawn_proc
def test_rpython_int_cmp_zero():
    def int_cmp_zero(a):
        if a > 0:
            return 1
        elif a == 0:
            return 0
        else:
            return -1
    
    mu_int_cmp_zero, _ = fncptr_from_rpy_func(int_cmp_zero, [rffi.LONGLONG], rffi.LONGLONG)
    
    assert mu_int_cmp_zero(1) == 1
    assert mu_int_cmp_zero(9223372036854775807) == 1
#    assert mu_int_cmp_zero(18446744073709551615) == -1
    assert mu_int_cmp_zero(0) == 0
    assert mu_int_cmp_zero(-1) == -1
    assert mu_int_cmp_zero(-9223372036854775808) == -1

@may_spawn_proc
def test_rpython_int_cmp_const():
    # these may get optimized away by Rpython compiler

    def int_cmp_zero_eq_zero():
        if 0 == 0:
            return 1
        else:
            return 0
    
    def int_cmp_zero_ne_zero():
        if 0 != 0:
            return 0
        else:
            return 1
    
    def int_cmp_zero_eq_one():
        if 0 == 1:
            return 0
        else:
            return 1
    
    def int_cmp_zero_ne_one():
        if 0 != 1:
            return 1
        else:
            return 0
    
    mu_int_cmp_zero_eq_zero, _ = fncptr_from_rpy_func(int_cmp_zero_eq_zero, [], rffi.LONGLONG)
    mu_int_cmp_zero_ne_zero, _ = fncptr_from_rpy_func(int_cmp_zero_ne_zero, [], rffi.LONGLONG)
    mu_int_cmp_zero_eq_one , _ = fncptr_from_rpy_func(int_cmp_zero_eq_one , [], rffi.LONGLONG)
    mu_int_cmp_zero_ne_one , _ = fncptr_from_rpy_func(int_cmp_zero_ne_one , [], rffi.LONGLONG)
    
    assert mu_int_cmp_zero_eq_zero() == 1
    assert mu_int_cmp_zero_ne_zero() == 1
    assert mu_int_cmp_zero_eq_one () == 1
    assert mu_int_cmp_zero_ne_one () == 1

@may_spawn_proc
def test_rpython_int_gt_value():
    def int_gt_value(a, b):
        ret = a > b
        return ret
    
    mu_int_gt_value, _ = fncptr_from_rpy_func(int_gt_value, [rffi.LONGLONG, rffi.LONGLONG], rffi.LONGLONG)
    
    assert mu_int_gt_value(1, 0) == 1
    assert mu_int_gt_value(0, 1) == 0
    assert mu_int_gt_value(1, 1) == 0

    assert mu_int_gt_value(1, -1) == 1
    assert mu_int_gt_value(-1, 1) == 0
    assert mu_int_gt_value(-1, -1) == 0
    
    assert mu_int_gt_value(9223372036854775807, -9223372036854775808) == 1
    assert mu_int_gt_value(-9223372036854775808, 9223372036854775807) == 0
    
#    assert mu_int_gt_value(18446744073709551615, 9223372036854775807) == 0
#    assert mu_int_gt_value(18446744073709551615, -9223372036854775808) == 1
#    assert mu_int_gt_value(18446744073709551615, -1) == 0

@may_spawn_proc
def test_rpython_int_ge_value():
    def int_ge_value(a, b):
        ret = a >= b
        return ret
    
    mu_int_ge_value, _ = fncptr_from_rpy_func(int_ge_value, [rffi.LONGLONG, rffi.LONGLONG], rffi.LONGLONG)
    
    assert mu_int_ge_value(1, 0) == 1
    assert mu_int_ge_value(0, 1) == 0
    assert mu_int_ge_value(1, 1) == 1

    assert mu_int_ge_value(1, -1) == 1
    assert mu_int_ge_value(-1, 1) == 0
    assert mu_int_ge_value(-1, -1) == 1
    
    assert mu_int_ge_value(9223372036854775807, -9223372036854775808) == 1
    assert mu_int_ge_value(-9223372036854775808, 9223372036854775807) == 0
    
#    assert mu_int_ge_value(18446744073709551615, 9223372036854775807) == 0
#    assert mu_int_ge_value(18446744073709551615, -9223372036854775808) == 1
#    assert mu_int_ge_value(18446744073709551615, -1) == 1

@may_spawn_proc
def test_rpython_int_lt_value():
    def int_lt_value(a, b):
        ret = a < b
        return ret
    
    mu_int_lt_value, _ = fncptr_from_rpy_func(int_lt_value, [rffi.LONGLONG, rffi.LONGLONG], rffi.LONGLONG)
    
    assert mu_int_lt_value(1, 0) == 0
    assert mu_int_lt_value(0, 1) == 1
    assert mu_int_lt_value(1, 1) == 0

    assert mu_int_lt_value(1, -1) == 0
    assert mu_int_lt_value(-1, 1) == 1
    assert mu_int_lt_value(-1, -1) == 0
    
    assert mu_int_lt_value(9223372036854775807, -9223372036854775808) == 0
    assert mu_int_lt_value(-9223372036854775808, 9223372036854775807) == 1
    
#    assert mu_int_lt_value(18446744073709551615, 9223372036854775807) == 1
#    assert mu_int_lt_value(18446744073709551615, -9223372036854775808) == 0
#    assert mu_int_lt_value(18446744073709551615, -1) == 0

@may_spawn_proc
def test_rpython_int_le_value():
    def int_le_value(a, b):
        ret = a <= b
        return ret
    
    mu_int_le_value, _ = fncptr_from_rpy_func(int_le_value, [rffi.LONGLONG, rffi.LONGLONG], rffi.LONGLONG)
    
    assert mu_int_le_value(1, 0) == 0
    assert mu_int_le_value(0, 1) == 1
    assert mu_int_le_value(1, 1) == 1

    assert mu_int_le_value(1, -1) == 0
    assert mu_int_le_value(-1, 1) == 1
    assert mu_int_le_value(-1, -1) == 1
    
    assert mu_int_le_value(9223372036854775807, -9223372036854775808) == 0
    assert mu_int_le_value(-9223372036854775808, 9223372036854775807) == 1
    
#    assert mu_int_le_value(18446744073709551615, 9223372036854775807) == 1
#    assert mu_int_le_value(18446744073709551615, -9223372036854775808) == 0
#    assert mu_int_le_value(18446744073709551615, -1) == 1

@may_spawn_proc
def test_rpython_int_eq_value():
    def int_eq_value(a, b):
        ret = a == b
        return ret
    
    mu_int_eq_value, _ = fncptr_from_rpy_func(int_eq_value, [rffi.LONGLONG, rffi.LONGLONG], rffi.LONGLONG)
    
    assert mu_int_eq_value(1, 0) == 0
    assert mu_int_eq_value(0, 1) == 0
    assert mu_int_eq_value(1, 1) == 1

    assert mu_int_eq_value(1, -1) == 0
    assert mu_int_eq_value(-1, 1) == 0
    assert mu_int_eq_value(-1, -1) == 1
    
    assert mu_int_eq_value(9223372036854775807, -9223372036854775808) == 0
    assert mu_int_eq_value(-9223372036854775808, 9223372036854775807) == 0
    
#    assert mu_int_eq_value(18446744073709551615, 9223372036854775807) == 0
#    assert mu_int_eq_value(18446744073709551615, -9223372036854775808) == 0
#    assert mu_int_eq_value(18446744073709551615, -1) == 1

@may_spawn_proc
def test_rpython_int_ne_value():
    def int_ne_value(a, b):
        ret = a != b
        return ret
    
    mu_int_ne_value, _ = fncptr_from_rpy_func(int_ne_value, [rffi.LONGLONG, rffi.LONGLONG], rffi.LONGLONG)
    
    assert mu_int_ne_value(1, 0) == 1
    assert mu_int_ne_value(0, 1) == 1
    assert mu_int_ne_value(1, 1) == 0

    assert mu_int_ne_value(1, -1) == 1
    assert mu_int_ne_value(-1, 1) == 1
    assert mu_int_ne_value(-1, -1) == 0
    
    assert mu_int_ne_value(9223372036854775807, -9223372036854775808) == 1
    assert mu_int_ne_value(-9223372036854775808, 9223372036854775807) == 1
    
#    assert mu_int_ne_value(18446744073709551615, 9223372036854775807) == 1
#    assert mu_int_ne_value(18446744073709551615, -9223372036854775808) == 1
#    assert mu_int_ne_value(18446744073709551615, -1) == 0