"""
Performance comparison
"""
from time import time
from tempfile import mkdtemp
import py, os
import inspect
import subprocess as subp
import ctypes
import importlib

from rpython.rtyper.lltypesystem import lltype, rffi
from rpython.translator.interactive import Translation
from rpython.config.translationoption import set_opt_level

from util import fncptr_from_rpy_func

CPYTHON = os.environ.get('CPYTHON', 'python')
PYPY = os.environ.get('PYPY', 'pypy')
RPYTHON = os.environ.get('RPYTHON', None)


def run(cmd):
    print ' '.join(cmd)
    p = subp.Popen(cmd, stdout=subp.PIPE, stderr=subp.PIPE)
    return p.communicate()

def get_c_function(lib, f):
    from ctypes import CDLL
    name = f.__name__
    return getattr(CDLL(lib.strpath), 'pypy_g_' + name)

def perf_fibonacci():
    tmpdir = py.path.local(mkdtemp())
    print tmpdir

    file_str = \
"""
from time import time
from rpython.rlib import jit
driver = jit.JitDriver(greens = [], reds = 'auto')

def fib(n):
    if n in (0, 1):
        return n
    k = 2
    fib_k_2 = 0
    fib_k_1 = 1
    while k < n:
        driver.jit_merge_point()
        fib_k = fib_k_2 + fib_k_1
        fib_k_2 = fib_k_1
        fib_k_1 = fib_k
        k += 1
    return fib_k_2 + fib_k_1

def measure(N):
    t0 = time()
    fib(N)
    t1 = time()
    return t0, t1

def rpy_entry(N):
    t0, t1 = measure(N)
    # from rpython.rlib import rfloat
    # print rfloat.double_to_string(t1 - t0, 'e', %(fprec)d, rfloat.DTSF_ADD_DOT_0)
    return t1 - t0

if __name__ == '__main__':
    import sys
    t0, t1 = measure(int(sys.argv[1]))
    print '%%.%(fprec)df' %% (t1 - t0)



def target(*args):
    from rpython.rlib.entrypoint import export_symbol
    export_symbol(rpy_entry)
    return rpy_entry, [int]
""" % {'fprec': 10}

    py_file = tmpdir.join('fibonacci.py')
    with py_file.open('w') as fp:
        fp.write(file_str)


    N = 30

    def run_cpython():
        out, _ = run([CPYTHON, py_file.strpath, str(N)])
        print out
        return float(out)

    def run_pypy_nojit():
        out, _ = run([PYPY, '--jit', 'off', py_file.strpath, str(N)])
        print out
        return float(out)

    def run_pypy():
        out, _ = run([PYPY, '--jit', 'off', py_file.strpath, str(N)])
        print out
        return float(out)

    def compile_rpython_c():
        mod = {}
        exec(file_str, mod)
        rpy_fnc = mod['fib']
        t = Translation(rpy_fnc, [int],
                        gc='none')
        set_opt_level(t.config, '3')
        t.ensure_opt('gc', 'none')
        libpath = t.compile_c()
        fnp = getattr(ctypes.CDLL(libpath.strpath), 'pypy_g_' + rpy_fnc.__name__)
        return fnp

    def compile_rpython_c_jit():
        mod = {}
        exec (file_str, mod)
        rpy_fnc = mod['fib']
        t = Translation(rpy_fnc, [int],
                        gc='none')
        set_opt_level(t.config, 'jit')
        t.ensure_opt('gc', 'none')
        libpath = t.compile_c()
        fnp = getattr(ctypes.CDLL(libpath.strpath), 'pypy_g_' + rpy_fnc.__name__)
        return fnp

    def compile_rpython_mu():
        mod = {}
        exec (file_str, mod)
        rpy_fnc = mod['fib']
        fnp, _ = fncptr_from_rpy_func(rpy_fnc, [lltype.Signed], lltype.Signed)
        return fnp

    def run_funcptr(fnp):
        t0 = time()
        fnp(N)
        t1 = time()
        return t1 - t0


    t_cpython = run_cpython()
    t_pypy_nojit = run_pypy_nojit()
    t_pypy = run_pypy()
    t_rpyc = run_funcptr(compile_rpython_c())
    t_rpyc_jit = run_funcptr(compile_rpython_c_jit())
    t_rpyc_mu = run_funcptr(compile_rpython_mu())
    print "CPython:", t_cpython
    print "PyPy (no JIT):", t_pypy_nojit
    print "PyPy:", t_pypy
    print "RPython C:", t_rpyc
    print "RPython C (with JIT):", t_rpyc_jit
    print "RPython Mu Zebu:", t_rpyc_mu

if __name__ == '__main__':
    perf_fibonacci()
