from util import fncptr_from_c_script, may_spawn_proc
import ctypes

@may_spawn_proc
def test_add():
    fn, _ = fncptr_from_c_script("test_add.c", "test_fnc", restype=ctypes.c_uint8)
    assert fn() == 9

@may_spawn_proc
def test_sub():
    fn, _ = fncptr_from_c_script("test_sub.c", "test_fnc", restype=ctypes.c_uint8)
    assert fn() == 11

@may_spawn_proc
def test_mul():
    fn, _ = fncptr_from_c_script("test_mul.c", "test_fnc", restype=ctypes.c_uint8)
    assert fn() == 0xf6

@may_spawn_proc
def test_sdiv():
    fn, _ = fncptr_from_c_script("test_sdiv.c", "test_fnc", restype=ctypes.c_uint8)
    assert fn() == 0xf4

@may_spawn_proc
def test_udiv():
    fn, _ = fncptr_from_c_script("test_udiv.c", "test_fnc", restype=ctypes.c_uint8)
    assert fn() == 12

@may_spawn_proc
def test_srem():
    fn, _ = fncptr_from_c_script("test_srem.c", "test_fnc", restype=ctypes.c_uint8)
    assert fn() == 0xff     # -1

@may_spawn_proc
def test_urem():
    fn, _ = fncptr_from_c_script("test_urem.c", "test_fnc", restype=ctypes.c_uint8)
    assert fn() == 5

@may_spawn_proc
def test_shl():
    fn, _ = fncptr_from_c_script("test_shl.c", "test_fnc", restype=ctypes.c_uint64)
    assert fn() == 0x7e707560c92d5400

@may_spawn_proc
def test_ashr():
    fn, _ = fncptr_from_c_script("test_ashr.c", "test_fnc", restype=ctypes.c_uint64)
    assert fn() == 0xffe367e707560c92

@may_spawn_proc
def test_lshr():
    fn, _ = fncptr_from_c_script("test_lshr.c", "test_fnc", restype=ctypes.c_uint64)
    assert fn() == 0x2367e707560c92

@may_spawn_proc
def test_and():
    fn, _ = fncptr_from_c_script("test_and.c", "test_fnc", restype=ctypes.c_uint64)
    assert fn() == 0x8588901c10004b14

@may_spawn_proc
def test_or():
    fn, _ = fncptr_from_c_script("test_or.c", "test_fnc", restype=ctypes.c_uint64)
    assert fn() == 0xddbffedff83febf5

@may_spawn_proc
def test_xor():
    fn, _ = fncptr_from_c_script("test_xor.c", "test_fnc", restype=ctypes.c_uint64)
    assert fn() == 0x58376ec3e83fa0e1
