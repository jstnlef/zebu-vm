from util import fncptr_from_c_script
import ctypes


def within_err(res, exp, err=1e15):
    return abs(res - exp) < err


def test_double_add():
    fnp = fncptr_from_c_script("test_double_add.c", "test_fnc", restype=ctypes.c_double)
    assert fnp() == 5.859873


def test_double_sub():
    fnp = fncptr_from_c_script("test_double_sub.c", "test_fnc", restype=ctypes.c_double)
    assert within_err(fnp(), 0.423313)


def test_double_mul():
    fnp = fncptr_from_c_script("test_double_mul.c", "test_fnc", restype=ctypes.c_double)
    assert fnp() == 8.53972942004


def test_double_div():
    fnp = fncptr_from_c_script("test_double_div.c", "test_fnc", restype=ctypes.c_double)
    assert fnp() == within_err(fnp, 1.1557282546316052)


def test_double_ordered_eq():
    fnp = fncptr_from_c_script("test_double_ordered_eq.c", "test_fnc")
    assert fnp() == 0


def test_double_ordered_ne():
    fnp = fncptr_from_c_script("test_double_ordered_ne.c", "test_fnc")
    assert fnp() == 1


def test_double_ordered_lt():
    fnp = fncptr_from_c_script("test_double_ordered_lt.c", "test_fnc")
    assert fnp() == 1


def test_double_ordered_le():
    fnp = fncptr_from_c_script("test_double_ordered_le.c", "test_fnc")
    assert fnp() == 1


def test_double_ordered_ge():
    fnp = fncptr_from_c_script("test_double_ordered_ge.c", "test_fnc")
    assert fnp() == 1


def test_double_ordered_gt():
    fnp = fncptr_from_c_script("test_double_ordered_gt.c", "test_fnc")
    assert fnp() == 1
