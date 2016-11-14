from util import fncptr_from_c_script
import ctypes

def test_select():
    fnp = fncptr_from_c_script('test_select.c', 'test_fnc', [ctypes.c_byte])
    assert fnp(0) == 20
    assert fnp(1) == 10
