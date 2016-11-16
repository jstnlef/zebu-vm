"""
Performance comparison
"""
from time import time
from tempfile import mkdtemp
import py, os
import subprocess as subp
import ctypes

from rpython.translator.interactive import Translation
from rpython.config.translationoption import set_opt_level

from util import libmu_path, libext

CPYTHON = os.environ.get('CPYTHON', 'python')
PYPY = os.environ.get('PYPY', 'pypy')
RPYTHON = os.environ.get('RPYTHON', None)
CC = os.environ.get('CC', 'clang')

def run(cmd):
    # print ' '.join(cmd)
    p = subp.Popen(cmd, stdout=subp.PIPE, stderr=subp.PIPE)
    return p.communicate()

def get_c_function(lib, f):
    from ctypes import CDLL
    name = f.__name__
    return getattr(CDLL(lib.strpath), 'pypy_g_' + name)

def perf_fibonacci():
    tmpdir = py.path.local(mkdtemp())
    print tmpdir

    py_code_str = \
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

    c_code_str = \
"""
#include <stdint.h>

uint64_t fib(uint64_t n) {
    uint64_t k, fib_k, fib_k_2, fib_k_1;

    if(n <= 1) return n;

    k = 2;
    fib_k_2 = 0;
    fib_k_1 = 1;

    while(k < n) {
        fib_k = fib_k_2 + fib_k_1;
        fib_k_2 = fib_k_1;
        fib_k_1 = fib_k;
        k += 1;
    }
    return fib_k_2 + fib_k_1;
}
"""

    py_file = tmpdir.join('fibonacci.py')
    with py_file.open('w') as fp:
        fp.write(py_code_str)
    c_file = tmpdir.join('fibonacci.c')
    with c_file.open('w') as fp:
        fp.write(c_code_str)

    def run_cpython(N):
        out, _ = run([CPYTHON, py_file.strpath, str(N)])
        return float(out)

    def run_pypy_nojit(N):
        out, _ = run([PYPY, '--jit', 'off', py_file.strpath, str(N)])
        return float(out)

    def run_pypy(N):
        out, _ = run([PYPY, py_file.strpath, str(N)])
        return float(out)

    def compile_rpython_c():
        mod = {}
        exec(py_code_str, mod)
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
        exec (py_code_str, mod)
        rpy_fnc = mod['fib']
        t = Translation(rpy_fnc, [int],
                        gc='none')
        set_opt_level(t.config, 'jit')
        t.ensure_opt('gc', 'none')
        libpath = t.compile_c()
        fnp = getattr(ctypes.CDLL(libpath.strpath), 'pypy_g_' + rpy_fnc.__name__)
        return fnp

    def compile_c():
        libpath = tmpdir.join('libfibonacci' + libext)
        run([CC, '-fpic', '--shared', '-o', libpath.strpath, c_file.strpath])
        lib = ctypes.CDLL(libpath.strpath)
        return lib.fib

    def compile_rpython_mu():
        mod = {}
        exec (py_code_str, mod)
        rpy_fnc = mod['fib']

        # load libmu before rffi so to load it with RTLD_GLOBAL
        libmu = ctypes.CDLL(libmu_path.strpath, ctypes.RTLD_GLOBAL)

        t = Translation(rpy_fnc, [int],
                        backend='mu', muimpl='fast', mucodegen='api', mutestjit=True)
        set_opt_level(t.config, '3')
        db, bdlgen, fnc_name = t.compile_mu()
        libname = 'lib%(fnc_name)s.dylib' % locals()
        bdlgen.mu.compile_to_sharedlib(libname, [])
        libpath = py.path.local().join('emit', libname)
        fnp = getattr(ctypes.CDLL(libpath.strpath), fnc_name)
        return fnp

    def get_average_time(run_fnc, args, warmup=5, iterations=100):
        for i in range(warmup):
            run_fnc(*args)

        total = 0.0
        for i in range(iterations):
            total += run_fnc(*args)
        return total / iterations

    def get_average_time_compiled(compile_fnc, args, warmup=5, iterations=100):
        def run_funcptr(fnp, N):
            t0 = time()
            fnp(N)
            t1 = time()
            return t1 - t0

        fnp = compile_fnc()
        return get_average_time(lambda *a: run_funcptr(fnp, *a), args, warmup, iterations)

    N = 100000
    iterations = 20

    t_cpython = get_average_time(run_cpython, [N], iterations=iterations)
    t_pypy_nojit = get_average_time(run_pypy_nojit, [N], iterations=iterations)
    t_pypy = get_average_time(run_pypy, [N], iterations=iterations)
    t_rpyc = get_average_time_compiled(compile_rpython_c, [N], iterations=iterations)
    t_rpyc_jit = get_average_time_compiled(compile_rpython_c_jit, [N], iterations=iterations)
    t_rpyc_mu = get_average_time_compiled(compile_rpython_mu, [N], iterations=iterations)
    t_c = get_average_time_compiled(compile_c, [N], iterations=iterations)
    print "CPython:", t_cpython
    print "PyPy (no JIT):", t_pypy_nojit
    print "PyPy:", t_pypy
    print "RPython C:", t_rpyc
    print "RPython C (with JIT):", t_rpyc_jit
    print "RPython Mu Zebu:", t_rpyc_mu
    print "C:", t_c

if __name__ == '__main__':
    perf_fibonacci()
