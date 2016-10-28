from test_milestones import get_fncptr

def test_eq_int():
    fn = get_fncptr("test_eq_int", "test_fnc")
    assert fn() == 0

def test_eq_ref():
    fn = get_fncptr("test_eq_ref", "test_fnc")
    assert fn() == 0

def test_ne_int():
    fn = get_fncptr("test_ne_int", "test_fnc")
    assert fn() == 1

def test_ne_ref():
    fn = get_fncptr("test_ne_ref", "test_fnc")
    assert fn() == 1

def test_sge():
    fn = get_fncptr("test_sge", "test_fnc")
    assert fn() == 1

def test_sgt():
    fn = get_fncptr("test_sgt", "test_fnc")
    assert fn() == 0

def test_sle():
    fn = get_fncptr("test_sle", "test_fnc")
    assert fn() == 1

def test_slt():
    fn = get_fncptr("test_slt", "test_fnc")
    assert fn() == 0

def test_ult():
    fn = get_fncptr("test_ult", "test_fnc")
    assert fn() == 0
