from test_milestones import get_fncptr
import ctypes


def test_ccall():
    fn = get_fncptr("test_ccall", "test_ccall", [ctypes.c_ulonglong])
    assert fn(0x7e707560c92d5400) == 0x7e707560c92d5400
