"""
Harness JIT tests using py.test framework
"""
from util import fncptr_from_c_script, may_spawn_proc
import ctypes

@may_spawn_proc
def test_constant_function():
    fn, _ = fncptr_from_c_script("test_constfunc.c", 'test_fnc')
    assert fn() == 0

@may_spawn_proc
def test_milsum():
    fn, _ = fncptr_from_c_script("test_milsum.c", "milsum", [ctypes.c_ulonglong])
    assert fn(1000000) == 500000500000

@may_spawn_proc
def test_factorial():
    fn, _ = fncptr_from_c_script("test_fac.c", "fac", [ctypes.c_ulonglong])
    assert fn(20) == 2432902008176640000

@may_spawn_proc
def test_fibonacci():
    fn, _ = fncptr_from_c_script("test_fib.c", "fib", [ctypes.c_ulonglong])
    assert fn(20) == 6765

@may_spawn_proc
def test_multifunc():
    fn, _ = fncptr_from_c_script("test_multifunc.c", "entry")
    assert fn() == 6765
