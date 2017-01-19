from util import fncptr_from_c_script, mu_instance_via_ctyeps, may_spawn_proc
import ctypes

def test_select():
    fnp, _ = fncptr_from_c_script('test_select.c', 'test_fnc', [ctypes.c_byte])
    assert fnp(0) == 20
    assert fnp(1) == 10

@may_spawn_proc
def test_commoninst_pin():
    mu = mu_instance_via_ctyeps()
    fnp, _ = fncptr_from_c_script("test_commoninst_pin.c", 'test_pin')
    assert fnp() == 6
