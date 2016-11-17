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

from util import libext, preload_libmu

CPYTHON = os.environ.get('CPYTHON', 'python')
PYPY = os.environ.get('PYPY', 'pypy')
RPYTHON = os.environ.get('RPYTHON', None)
CC = os.environ.get('CC', 'clang')

perf_target_dir = py.path.local(__file__).join('perf_target')

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

    py_file = perf_target_dir.join('fibonacci.py')
    c_file = perf_target_dir.join('fibonacci.c')

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
        from perf_target.fibonacci import fib
        t = Translation(fib, [int],
                        gc='none')
        set_opt_level(t.config, '3')
        t.ensure_opt('gc', 'none')
        libpath = t.compile_c()
        fnp = getattr(ctypes.CDLL(libpath.strpath), 'pypy_g_' + fib.__name__)
        return fnp

    def compile_rpython_c_jit():
        from perf_target.fibonacci import fib
        t = Translation(fib, [int],
                        gc='none')
        set_opt_level(t.config, 'jit')
        t.ensure_opt('gc', 'none')
        libpath = t.compile_c()
        fnp = getattr(ctypes.CDLL(libpath.strpath), 'pypy_g_' + fib.__name__)
        return fnp

    def compile_c():
        libpath = tmpdir.join('libfibonacci' + libext)
        run([CC, '-fpic', '--shared', '-o', libpath.strpath, c_file.strpath])
        lib = ctypes.CDLL(libpath.strpath)
        return lib.fib

    def compile_rpython_mu():
        preload_libmu()

        from perf_target.fibonacci import fib
        t = Translation(fib, [int],
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

    N = 30
    iterations = 10

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
