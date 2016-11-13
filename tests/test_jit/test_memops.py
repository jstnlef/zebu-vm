from util import fncptr_from_c_script
import ctypes


def test_uptr_bytestore_load():
    fn, _ = fncptr_from_c_script("test_uptr_bytestore_load.c", "test_fnc",
                                 argtypes=[ctypes.POINTER(ctypes.c_uint32)],
                                 restype=ctypes.c_uint32)

    # allocate memory through ctypes
    ui32 = ctypes.c_uint32()
    assert fn(ctypes.byref(ui32)) == 0x8d9f9c1d
    assert ui32.value == 0x8d9f9c1d
