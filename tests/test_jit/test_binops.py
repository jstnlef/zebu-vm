from util import fncptr_from_c_script
import ctypes

def test_add():
    fn, _ = fncptr_from_c_script("test_add.c", "test_fnc", restype=ctypes.c_uint8)
    assert fn() == 9

def test_sub():
    fn, _ = fncptr_from_c_script("test_sub.c", "test_fnc", restype=ctypes.c_uint8)
    assert fn() == 11

def test_mul():
    fn, _ = fncptr_from_c_script("test_mul.c", "test_fnc", restype=ctypes.c_uint8)
    assert fn() == 0xf6

def test_sdiv():
    fn, _ = fncptr_from_c_script("test_sdiv.c", "test_fnc", restype=ctypes.c_uint8)
    assert fn() == 0xf4

def test_udiv():
    fn, _ = fncptr_from_c_script("test_udiv.c", "test_fnc", restype=ctypes.c_uint8)
    assert fn() == 12

def test_srem():
    fn, _ = fncptr_from_c_script("test_srem.c", "test_fnc", restype=ctypes.c_uint8)
    assert fn() == 0xff     # -1

def test_urem():
    fn, _ = fncptr_from_c_script("test_urem.c", "test_fnc", restype=ctypes.c_uint8)
    assert fn() == 5

def test_shl():
    fn, _ = fncptr_from_c_script("test_shl.c", "test_fnc", restype=ctypes.c_uint64)
    assert fn() == 0x7e707560c92d5400

def test_ashr():
    fn, _ = fncptr_from_c_script("test_ashr.c", "test_fnc", restype=ctypes.c_uint64)
    assert fn() == 0xffe367e707560c92

def test_lshr():
    fn, _ = fncptr_from_c_script("test_lshr.c", "test_fnc", restype=ctypes.c_uint64)
    assert fn() == 0x2367e707560c92

def test_and():
    fn, _ = fncptr_from_c_script("test_and.c", "test_fnc", restype=ctypes.c_uint64)
    assert fn() == 0x8588901c10004b14

def test_or():
    fn, _ = fncptr_from_c_script("test_or.c", "test_fnc", restype=ctypes.c_uint64)
    assert fn() == 0xddbffedff83febf5

def test_xor():
    fn, _ = fncptr_from_c_script("test_xor.c", "test_fnc", restype=ctypes.c_uint64)
    assert fn() == 0x58376ec3e83fa0e1
