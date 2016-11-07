from test_milestones import get_fncptr
import ctypes


def test_uptr_bytestore_load():
    fn = get_fncptr("test_uptr_bytestore_load", "test_fnc",
                    argtypes=[ctypes.POINTER(ctypes.c_uint32)],
                    restype=ctypes.c_uint32)

    # allocate memory through ctypes
    ui32 = ctypes.c_uint32()
    assert fn(ctypes.byref(ui32)) == 0x8d9f9c1d
    assert ui32.value == 0x8d9f9c1d
