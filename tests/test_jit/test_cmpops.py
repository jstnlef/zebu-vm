from util import fncptr_from_c_script

def test_eq_int():
    fn, _ = fncptr_from_c_script("test_eq_int.c", "test_fnc")
    assert fn() == 0

def test_eq_ref():
    fn, _ = fncptr_from_c_script("test_eq_ref.c", "test_fnc")
    assert fn() == 0

def test_ne_int():
    fn, _ = fncptr_from_c_script("test_ne_int.c", "test_fnc")
    assert fn() == 1

def test_ne_ref():
    fn, _ = fncptr_from_c_script("test_ne_ref.c", "test_fnc")
    assert fn() == 1

def test_sge():
    fn, _ = fncptr_from_c_script("test_sge.c", "test_fnc")
    assert fn() == 1

def test_sgt():
    fn, _ = fncptr_from_c_script("test_sgt.c", "test_fnc")
    assert fn() == 0

def test_sle():
    fn, _ = fncptr_from_c_script("test_sle.c", "test_fnc")
    assert fn() == 1

def test_slt():
    fn, _ = fncptr_from_c_script("test_slt.c", "test_fnc")
    assert fn() == 0

def test_ult():
    fn, _ = fncptr_from_c_script("test_ult.c", "test_fnc")
    assert fn() == 0
