from test_harness import get_fncptr

def test_eq_int():
    fn = get_fncptr("suite/test_eq_int.c", "test_fnc")
    assert fn() == 0

def test_eq_ref():
    fn = get_fncptr("suite/test_eq_ref.c", "test_fnc")
    assert fn() == 0

def test_ne_int():
    fn = get_fncptr("suite/test_ne_int.c", "test_fnc")
    assert fn() == 1

def test_ne_ref():
    fn = get_fncptr("suite/test_ne_ref.c", "test_fnc")
    assert fn() == 1

def test_sge():
    fn = get_fncptr("suite/test_sge.c", "test_fnc")
    assert fn() == 1

def test_sgt():
    fn = get_fncptr("suite/test_sgt.c", "test_fnc")
    assert fn() == 0

def test_sle():
    fn = get_fncptr("suite/test_sle.c", "test_fnc")
    assert fn() == 1

def test_slt():
    fn = get_fncptr("suite/test_slt.c", "test_fnc")
    assert fn() == 0

def test_ult():
    fn = get_fncptr("suite/test_ult.c", "test_fnc")
    assert fn() == 0
