from util import fncptr_from_c_script, may_spawn_proc
import ctypes


@may_spawn_proc
def test_uptr_bytestore_load():
    fn, _ = fncptr_from_c_script("test_uptr_bytestore_load.c", "test_fnc",
                                 argtypes=[ctypes.POINTER(ctypes.c_uint32)],
                                 restype=ctypes.c_uint32)

    # allocate memory through ctypes
    ui32 = ctypes.c_uint32()
    assert fn(ctypes.byref(ui32)) == 0x8d9f9c1d
    assert ui32.value == 0x8d9f9c1d

@may_spawn_proc
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
    stt.ui64 = 0xabcdef0123456789
    stt.ui32 = 0xcafebabe

    res = fn(ctypes.byref(stt))
    assert res == 0xcafebabe, "result: %s" % hex(res)

@may_spawn_proc
def test_getelemiref():
    Arr = ctypes.ARRAY(ctypes.c_int64, 5)
    fn, _ = fncptr_from_c_script("test_getelemiref.c", "test_fnc",
                                 argtypes=[ctypes.POINTER(Arr)],
                                 restype=ctypes.c_int64)
    arr = Arr()
    arr[0] = -23
    arr[1] = 35
    arr[2] = 42
    arr[3] = 0
    arr[4] = 152

    res = fn(ctypes.byref(arr), 2)
    assert res == 42, "result: %d" % res

@may_spawn_proc
def test_getvarpartiref():
    class Stt(ctypes.Structure):
        _fields_ = [('ui8', ctypes.c_uint8),
                    ('ui64', ctypes.c_uint64),
                    ('ui32s', ctypes.ARRAY(ctypes.c_uint32, 5))]

    fn, _ = fncptr_from_c_script("test_getvarpartiref.c", "test_fnc",
                                 argtypes=[ctypes.POINTER(Stt)],
                                 restype=ctypes.c_uint32)
    stt = Stt()
    stt.ui8 = 25
    stt.ui64 = 0xabcdef0123456789
    stt.ui32s[0] = 0xcafebabe

    res = fn(ctypes.byref(stt))
    assert res == 0xcafebabe, "result: %s" % hex(res)

@may_spawn_proc
def test_getvarpartiref_nofix():
    Arr = ctypes.ARRAY(ctypes.c_uint32, 3)

    fn, _ = fncptr_from_c_script("test_getvarpartiref_nofix.c", "test_fnc",
                                 argtypes=[ctypes.POINTER(Arr)],
                                 restype=ctypes.c_uint32)
    arr = Arr()
    arr[0] = 0xcafebabe
    arr[1] = 0xbecca
    arr[2] = 0xfaebee

    res = fn(ctypes.byref(arr))
    assert res == 0xcafebabe, "result: %s" % hex(res)


