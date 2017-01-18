from util import fncptr_from_c_script, preload_libmu
import ctypes

def test_select():
    fnp, _ = fncptr_from_c_script('test_select.c', 'test_fnc', [ctypes.c_byte])
    assert fnp(0) == 20
    assert fnp(1) == 10


def test_commoninst_pin():
    fnp, _ = fncptr_from_c_script("test_commoninst_pin.c", 'test_pin')
    assert fnp() == 6
