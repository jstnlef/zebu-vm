from test_milestones import get_fncptr
import ctypes


def test_uptr_bytestore_load():
    fn = get_fncptr("test_uptr_bytestore_load", "entry", restype=ctypes.c_uint32)
    assert fn() == 0x8d9f9c1d
