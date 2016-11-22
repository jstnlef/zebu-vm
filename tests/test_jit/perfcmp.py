"""
Performance comparison
"""
from time import time
from tempfile import mkdtemp
import py, os, sys
import subprocess as subp
import ctypes
import math

from rpython.translator.interactive import Translation
from rpython.config.translationoption import set_opt_level

from util import libext, preload_libmu

perf_target_dir = py.path.local(__file__).dirpath().join('perftarget')


CPYTHON = os.environ.get('CPYTHON', 'python')
PYPY = os.environ.get('PYPY', 'pypy')
RPYTHON = os.environ.get('RPYTHON', None)
CC = os.environ.get('CC', 'clang')


def run(cmd):
    print ' '.join(cmd)
    p = subp.Popen(cmd, stdout=subp.PIPE, stderr=subp.PIPE)
    return p.communicate()


def get_c_function(lib, f):
    from ctypes import CDLL
    name = f.__name__
    return getattr(CDLL(lib.strpath), 'pypy_g_' + name)


def perf_fibonacci():
    from perftarget.fibonacci import fib
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
        t = Translation(fib, [int],
                        gc='none')
        set_opt_level(t.config, '3')
        t.ensure_opt('gc', 'none')
        libpath = t.compile_c()
        fnp = getattr(ctypes.CDLL(libpath.strpath), 'pypy_g_' + fib.__name__)
        return fnp

    def compile_rpython_c_jit():
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
        t = Translation(fib, [int],
                        backend='mu', muimpl='fast', mucodegen='api', mutestjit=True)
        set_opt_level(t.config, '3')
        db, bdlgen, fnc_name = t.compile_mu()
        libpath = tmpdir.join('lib%(fnc_name)s.dylib' % locals())
        bdlgen.mu.compile_to_sharedlib(libpath.strpath, [])
        fnp = getattr(ctypes.CDLL(libpath.strpath), fnc_name)
        return fnp

    def get_stat(run_fnc, args, warmup=5, iterations=100):
        for i in range(warmup):
            run_fnc(*args)

        times = []
        for i in range(iterations):
            times.append(run_fnc(*args))

        times.sort()
        avg = sum(times) / float(len(times))
        t_min = t_max = t_std = None
        if len(times) > 1:
            t_min = times[0]
            t_max = times[-1]
            squares = ((t - avg) ** 2 for t in times)
            t_std = math.sqrt(sum(squares) / (len(times) - 1))

        return {'average': avg, 't_min': t_min, 't_max': t_max, 'std_dev': t_std}

    def get_stat_compiled(compile_fnc, args, warmup=5, iterations=100):
        def run_funcptr(fnp, N):
            t0 = time()
            fnp(N)
            t1 = time()
            return t1 - t0

        fnp = compile_fnc()
        return get_stat(lambda *a: run_funcptr(fnp, *a), args, warmup, iterations)

    def get_display_str(stat):
        output = "average: %(average)s\n" \
                 "min: %(t_min)s\n" \
                 "max: %(t_max)s\n" \
                 "std_dev: %(std_dev)s\n"
        return output % stat

    N = 30
    warmup = 0
    iterations = 10

    results = {
        'cpython': get_stat(run_cpython, [N], warmup, iterations=iterations),
        'pypy_nojit': get_stat(run_pypy_nojit, [N], warmup, iterations=iterations),
        'pypy': get_stat(run_pypy, [N], warmup, iterations=iterations),
        'rpy_c': get_stat_compiled(compile_rpython_c, [N], warmup, iterations=iterations),
        'rpy_c_jit': get_stat_compiled(compile_rpython_c_jit, [N], warmup, iterations=iterations),
        'rpy_mu': get_stat_compiled(compile_rpython_mu, [N], warmup, iterations=iterations),
        'c': get_stat_compiled(compile_c, [N], warmup, iterations=iterations),
    }
    
    for python, result in results.items():
        print '\033[35m---- %(python)s ----\033[0m' % locals()
        print get_display_str(result)


if __name__ == '__main__':
    perf_fibonacci()
