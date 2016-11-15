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


def test_getfieldiref():
    class Stt(ctypes.Structure):
        _fields_ = [('ui8', ctypes.c_uint8),
                    ('ui64', ctypes.c_uint64),
                    ('ui32', ctypes.c_uint32)]

    fn, _ = fncptr_from_c_script("test_getfieldiref.c", "test_fnc",
                                 argtypes=[ctypes.POINTER(Stt)],
                                 restype=ctypes.c_uint32)
    stt = Stt()
    stt.ui8 = 25
    stt.ui64 = 0xabcdef01234567890
    stt.ui32 = 0xcaffebabe

    res = fn(ctypes.byref(stt))
    assert res == 0xcaffebabe, "result: %s" % hex(res)
