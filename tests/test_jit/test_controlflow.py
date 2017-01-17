from util import fncptr_from_c_script, preload_libmu
import ctypes, ctypes.util

def test_branch():
    fn, _ = fncptr_from_c_script("test_branch.c", "test_fnc")
    assert fn() == 30

def test_branch2():
    fn, _ = fncptr_from_c_script("test_branch2.c", "test_fnc", [ctypes.c_byte])
    assert fn(1) == 30
    assert fn(0) == 200

def test_ccall():
    fn, _ = fncptr_from_c_script("test_ccall.c", "test_ccall", [ctypes.c_ulonglong])
    assert fn(0x7e707560c92d5400) == 0x7e707560c92d5400

def test_extern_func():
    # libc = ctypes.CDLL(ctypes.util.find_library('c'), ctypes.RTLD_GLOBAL)
    fn, _ = fncptr_from_c_script("test_extern_func.c", "test_write", [ctypes.c_void_p, ctypes.c_size_t], ctypes.c_int64)
    buf = ctypes.create_string_buffer('hello world!\n')
    assert fn(ctypes.byref(buf), len(buf)) == len(buf)

def test_throw():
    # from rpython.rlib import rmu_fast as rmu
    preload_libmu()
    fn, _ = fncptr_from_c_script("test_throw.c", "test_fnc", [ctypes.c_int64], ctypes.c_int64)
    # mu = rmu.MuVM()
    # mu.current_thread_as_mu_thread(rmu.null(rmu.MuCPtr))
    assert fn(0) == 20
    assert fn(100) == 10

def test_exception_stack_unwind():
    # from rpython.rlib import rmu_fast as rmu
    preload_libmu()
    fn, _ = fncptr_from_c_script("test_exception_stack_unwind.c", "test_fnc", [ctypes.c_int64], ctypes.c_int64)
    # mu = rmu.MuVM()
    # mu.current_thread_as_mu_thread(rmu.null(rmu.MuCPtr))
    assert fn(0) == 20
    assert fn(100) == 10
