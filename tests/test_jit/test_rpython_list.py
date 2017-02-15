from rpython.rtyper.lltypesystem import rffi, lltype
from rpython.rlib import rmu_fast as rmu
from rpython.translator.platform import platform
from util import fncptr_from_rpy_func, fncptr_from_py_script, may_spawn_proc
import ctypes, py, stat
import pytest

from test_rpython import run_boot_image

c_exit = rffi.llexternal('exit', [rffi.INT], lltype.Void, _nowrapper=True)

@may_spawn_proc
def test_rpython_list_new_empty():
    def main(argv):
        a = []
        c_exit(rffi.cast(rffi.INT, len(a)))
        return 0
    
    res = run_boot_image(main, '/tmp/test_rpython_list_new_empty')
    
    assert res.returncode == 0, res.err

@may_spawn_proc
def test_rpython_list_new_5():
    def main(argv):
        a = [1, 2, 3, 4, 5]
        c_exit(rffi.cast(rffi.INT, len(a)))
        return 0
    
    res = run_boot_image(main, '/tmp/test_rpython_list_new_5')
    
    assert res.returncode == 5, res.err

@may_spawn_proc
def test_rpython_list_append():
    def main(argv):
        a = []
        for i in range(0, 10):
            a.append(i)
        
        c_exit(rffi.cast(rffi.INT, len(a)))
        return 0
    
    res = run_boot_image(main, '/tmp/test_rpython_list_append')
    
    assert res.returncode == 10, res.err

@may_spawn_proc
def test_rpython_list_iter():
    def main(argv):
        a = []
        for i in range(0, 10):
            a.append(i)
        
        sum = 0
        for n in a:
            sum += n
        
        c_exit(rffi.cast(rffi.INT, sum))
        return 0
    
    res = run_boot_image(main, '/tmp/test_rpython_list_iter')
    
    assert res.returncode == 45, res.err

@pytest.mark.xfail(reason = "bug")
@may_spawn_proc
def test_rpython_list_addr_check_length1():
    Int64Ptr = lltype.Ptr(lltype.Array(rffi.LONGLONG))

    def check(actual, expect):
        if actual != expect:
            c_exit(rffi.cast(rffi.INT, actual))
    
    def main(argv):
        a = []
        a.append(42)
        
        from rpython.rtyper.lltypesystem.llmemory import cast_ptr_to_adr
        from rpython.rlib.objectmodel import keepalive_until_here
        
        addr = cast_ptr_to_adr(a)
        mem  = rffi.cast(Int64Ptr, addr)
        # ignore mem[0]
        check(mem[1], 1)
        keepalive_until_here(argv)
        
        return 0
        
    res = run_boot_image(main, '/tmp/test_rpython_list_addr_check_length1')
    
    assert res.returncode == 0, 'returncode = %d\n%s' % (res.returncode, res.err)

@pytest.mark.xfail(reason = "bug")
@may_spawn_proc
def test_rpython_list_addr_check_length2():
    Int64Ptr = lltype.Ptr(lltype.Array(rffi.LONGLONG))

    def check(actual, expect):
        if actual != expect:
            c_exit(rffi.cast(rffi.INT, actual))
    
    def main(argv):
        a = []
        a.append(42)
        a.append(43)
        
        from rpython.rtyper.lltypesystem.llmemory import cast_ptr_to_adr
        from rpython.rlib.objectmodel import keepalive_until_here
        
        addr = cast_ptr_to_adr(a)
        mem  = rffi.cast(Int64Ptr, addr)
        # ignore mem[0]
        check(mem[1], 2)
        keepalive_until_here(argv)
        
        return 0
        
    res = run_boot_image(main, '/tmp/test_rpython_list_addr_check_length2')
    
    assert res.returncode == 0, 'returncode = %d\n%s' % (res.returncode, res.err)

@pytest.mark.xfail(reason = "mem[1] is not 10? but mem[0] is 10, need to look into this")
@may_spawn_proc
def test_rpython_list_addr_check_all():
    Int64Ptr = lltype.Ptr(lltype.Array(rffi.LONGLONG))

    def check(a, b):
        if a != b:
            c_exit(rffi.cast(rffi.INT, 23))
    
    def main(argv):
        a = []
        for i in range(0, 10):
            a.append(i)
        
        from rpython.rtyper.lltypesystem.llmemory import cast_ptr_to_adr
        from rpython.rlib.objectmodel import keepalive_until_here
        
        addr = cast_ptr_to_adr(a)
        mem  = rffi.cast(Int64Ptr, addr)
        # ignore mem[0]
        check(mem[1], 10)
        keepalive_until_here(argv)
        
        return 0
        
    res = run_boot_image(main, '/tmp/test_rpython_list_addr_check_all')
    
    assert res.returncode == 0, 'returncode = %d\n%s' % (res.returncode, res.err)
