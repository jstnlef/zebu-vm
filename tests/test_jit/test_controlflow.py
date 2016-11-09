from test_milestones import get_fncptr, compile_lib
import ctypes, ctypes.util


def test_branch():
    fn = get_fncptr("test_branch", "test_fnc")
    assert fn() == 30

def test_branch2():
    fn = get_fncptr("test_branch2", "test_fnc", [ctypes.c_byte])
    assert fn(1) == 30
    assert fn(0) == 200

def test_ccall():
    fn = get_fncptr("test_ccall", "test_ccall", [ctypes.c_ulonglong])
    assert fn(0x7e707560c92d5400) == 0x7e707560c92d5400

def test_extern_func():
    # libc = ctypes.CDLL(ctypes.util.find_library('c'), ctypes.RTLD_GLOBAL)
    fn = get_fncptr("test_extern_func", "test_write", [ctypes.c_void_p, ctypes.c_size_t], ctypes.c_int64)
    buf = ctypes.create_string_buffer('hello world!\n')
    assert fn(ctypes.byref(buf), len(buf)) == len(buf)
