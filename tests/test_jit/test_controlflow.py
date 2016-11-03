from test_milestones import get_fncptr
import ctypes


def test_branch():
    fn = get_fncptr("test_branch", "test_fnc")
    assert fn() == 30

def test_branch2():
    fn = get_fncptr("test_branch", "test_fnc", [ctypes.c_byte])
    assert fn(1) == 30
    assert fn(0) == 200

def test_ccall():
    fn = get_fncptr("test_ccall", "test_ccall", [ctypes.c_ulonglong])
    assert fn(0x7e707560c92d5400) == 0x7e707560c92d5400
