from util import fncptr_from_c_script, may_spawn_proc
import ctypes

@may_spawn_proc
def test_trunc():
    fn, _ = fncptr_from_c_script("test_trunc.c", "test_fnc", restype=ctypes.c_uint32)
    assert fn() == 0x58324b55

@may_spawn_proc
def test_sext():
    fn, _ = fncptr_from_c_script("test_sext.c", "test_fnc")
    assert fn() == 0xffffffffa8324b55

@may_spawn_proc
def test_zext():
    fn, _ = fncptr_from_c_script("test_zext.c", "test_fnc")
    assert fn() == 0x00000000a8324b55
