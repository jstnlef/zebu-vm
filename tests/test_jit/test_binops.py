from test_harness import get_fncptr

def test_add():
    fn = get_fncptr("suite/test_add.c", "test_fnc")
    assert fn() == 9

def test_sub():
    fn = get_fncptr("suite/test_sub.c", "test_fnc")
    assert fn() == 11

def test_mul():
    fn = get_fncptr("suite/test_mul.c", "test_fnc")
    assert fn() == 0xf6

def test_sdiv():
    fn = get_fncptr("suite/test_sdiv.c", "test_fnc")
    assert fn() == 0xf4

def test_urem():
    fn = get_fncptr("suite/test_urem.c", "test_fnc")
    assert fn() == 5

def test_shl():
    fn = get_fncptr("suite/test_shl.c", "test_fnc")
    assert fn() == 0x7e707560c92d5400

def test_lshr():
    fn = get_fncptr("suite/test_lshr.c", "test_fnc")
    assert fn() == 0x2367e707560c92

def test_and():
    fn = get_fncptr("suite/test_and.c", "test_fnc")
    assert fn() == 0x8588901c10004b14

def test_xor():
    fn = get_fncptr("suite/test_xor.c", "test_fnc")
    assert fn() == 0x58376ec3e83fa0e1
