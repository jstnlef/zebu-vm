from rpython.rtyper.lltypesystem import rffi, lltype
from rpython.rlib.rmu import zebu as rmu
from rpython.translator.platform import platform
from util import fncptr_from_rpy_func, fncptr_from_py_script, may_spawn_proc
import ctypes, py, stat
import pytest

from test_rpython import run_boot_image

c_exit = rffi.llexternal('exit', [rffi.INT], lltype.Void, _nowrapper=True)

@may_spawn_proc
def test_rpython_list_new_empty():
    def new_empty():
        a = []
        return a

    fn, (db, bdlgen) = fncptr_from_rpy_func(new_empty, [], lltype.Void)
    bdlgen.mu.current_thread_as_mu_thread(rmu.null(rmu.MuCPtr))

    fn()

@may_spawn_proc
def test_rpython_list_new_5():
    def new_5():
        a = [1, 2, 3, 4, 5]
        return len(a)

    fn, (db, bdlgen) = fncptr_from_rpy_func(new_5, [], rffi.INT)
    bdlgen.mu.current_thread_as_mu_thread(rmu.null(rmu.MuCPtr))

    assert fn() == 5

@may_spawn_proc
def test_rpython_list_append():
    def main(argv):
        a = []
        for i in range(0, 10):
            a.append(i)

        c_exit(rffi.cast(rffi.INT, len(a)))
        return 0

    res = run_boot_image(main, '/tmp/test_rpython_list_append')

    assert res.returncode == 10, 'returncode = %d\n%s' % (res.returncode, res.err)

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
    
    assert res.returncode == 45, 'returncode = %d\n%s' % (res.returncode, res.err)

@may_spawn_proc
def test_rpython_list_addr_check_length1():
    Int64Ptr = lltype.Ptr(lltype.Array(rffi.LONGLONG, hints={'nolength': True}))

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
        keepalive_until_here(a)
        
        return 0
        
    res = run_boot_image(main, '/tmp/test_rpython_list_addr_check_length1')
    
    assert res.returncode == 0, 'returncode = %d\n%s' % (res.returncode, res.err)

@may_spawn_proc
def test_rpython_list_addr_check_length2():
    Int64Ptr = lltype.Ptr(lltype.Array(rffi.LONGLONG, hints={'nolength': True}))

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
        keepalive_until_here(a)
        
        return 0
        
    res = run_boot_image(main, '/tmp/test_rpython_list_addr_check_length2')
    
    assert res.returncode == 0, 'returncode = %d\n%s' % (res.returncode, res.err)
    
@may_spawn_proc
def test_rpython_list_addr_check_length100():
    Int64Ptr = lltype.Ptr(lltype.Array(rffi.LONGLONG, hints={'nolength': True}))

    def check(actual, expect):
        if actual != expect:
            c_exit(rffi.cast(rffi.INT, actual))
    
    def main(argv):
        a = []
        for i in range(0, 100):
            a.append(i)
        
        from rpython.rtyper.lltypesystem.llmemory import cast_ptr_to_adr
        from rpython.rlib.objectmodel import keepalive_until_here
        
        addr = cast_ptr_to_adr(a)
        mem  = rffi.cast(Int64Ptr, addr)
        # ignore mem[0]
        check(mem[1], 100)
        keepalive_until_here(a)
        
        return 0
        
    res = run_boot_image(main, '/tmp/test_rpython_list_addr_check_length2')
    
    assert res.returncode == 0, 'returncode = %d\n%s' % (res.returncode, res.err)

@may_spawn_proc
def test_rpython_list_addr_check_all10():
    Int64Ptr = lltype.Ptr(lltype.Array(rffi.LONGLONG, hints={'nolength': True}))

    N = 10

    def check(actual, expect):
        if actual != expect:
            c_exit(rffi.cast(rffi.INT, actual))
    
    def main(argv):
        a = []
        for i in range(0, N):
            a.append(i)
        
        from rpython.rtyper.lltypesystem.llmemory import cast_ptr_to_adr
        from rpython.rlib.objectmodel import keepalive_until_here
        
        addr = cast_ptr_to_adr(a)
        mem  = rffi.cast(Int64Ptr, addr)
        # ignore mem[0]
        check(mem[1], N)

        inner_addr = mem[2]
        inner = rffi.cast(Int64Ptr, inner_addr)
        # inner[0], inner[1] is ignored
        for i in range(0, N):
            check(inner[2 + i], i)
        
        keepalive_until_here(a)
        
        return 0
        
    res = run_boot_image(main, '/tmp/test_rpython_list_addr_check_all10')
    
    assert res.returncode == 0, 'returncode = %d\n%s' % (res.returncode, res.err)

@may_spawn_proc
def test_rpython_list_addr_check_all100():
    Int64Ptr = lltype.Ptr(lltype.Array(rffi.LONGLONG, hints={'nolength': True}))

    N = 100

    def check(actual, expect):
        if actual != expect:
            c_exit(rffi.cast(rffi.INT, actual))
    
    def main(argv):
        a = []
        for i in range(0, N):
            a.append(i)
        
        from rpython.rtyper.lltypesystem.llmemory import cast_ptr_to_adr
        from rpython.rlib.objectmodel import keepalive_until_here
        
        addr = cast_ptr_to_adr(a)
        mem  = rffi.cast(Int64Ptr, addr)
        # ignore mem[0]
        check(mem[1], N)

        inner_addr = mem[2]
        inner = rffi.cast(Int64Ptr, inner_addr)
        # inner[0], inner[1] is ignored
        for i in range(0, N):
            check(inner[2 + i], i)
        
        keepalive_until_here(a)
        
        return 0
        
    res = run_boot_image(main, '/tmp/test_rpython_list_addr_check_all100')
    
    assert res.returncode == 0, 'returncode = %d\n%s' % (res.returncode, res.err)
