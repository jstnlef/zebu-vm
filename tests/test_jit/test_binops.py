from test_milestones import get_fncptr

def test_add():
    fn = get_fncptr("test_add", "test_fnc")
    assert fn() == 9

def test_sub():
    fn = get_fncptr("test_sub", "test_fnc")
    assert fn() == 11

def test_mul():
    fn = get_fncptr("test_mul", "test_fnc")
    assert fn() == 0xf6

def test_sdiv():
    fn = get_fncptr("test_sdiv", "test_fnc")
    assert fn() == 0xf4

def test_urem():
    fn = get_fncptr("test_urem", "test_fnc")
    assert fn() == 5

def test_shl():
    fn = get_fncptr("test_shl", "test_fnc")
    assert fn() == 0x7e707560c92d5400

def test_lshr():
    fn = get_fncptr("test_lshr", "test_fnc")
    assert fn() == 0x2367e707560c92

def test_and():
    fn = get_fncptr("test_and", "test_fnc")
    assert fn() == 0x8588901c10004b14

def test_xor():
    fn = get_fncptr("test_xor", "test_fnc")
    assert fn() == 0x58376ec3e83fa0e1
