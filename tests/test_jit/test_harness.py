"""
Harness JIT tests using py.test framework
"""
import subprocess as subp
import os, sys
import ctypes

CC = 'clang'
# CI_PROJ_DIR = os.environ["CI_PROJECT_DIR"]
CI_PROJ_DIR = os.environ["MU_RUST"]
CFLAGS = [
    "-std=c99",
    "-I%(CI_PROJ_DIR)s/src/vm/api" % globals(),
    "-L%(CI_PROJ_DIR)s/target/debug" % globals(),
    "-lmu",
]
os.environ['RUST_BACKTRACE'] = '1'

def get_lib(src_c):
    bin_path = src_c[:-2]
    cmd = [CC] + CFLAGS + ['-o', bin_path, src_c]

    # compile
    p = subp.Popen(cmd, stdout=subp.PIPE, stderr=subp.PIPE, env=os.environ)
    out, err = p.communicate()
    if p.returncode != 0:  # failed
        sys.stdout.write(out + '\n')
        sys.stderr.write(err + '\n')
        raise subp.CalledProcessError(p.returncode, cmd)

    # run
    p = subp.Popen([bin_path], stdout=subp.PIPE, stderr=subp.PIPE, env=os.environ)
    out, err = p.communicate()
    if p.returncode != 0:  # failed
        sys.stdout.write(out + '\n')
        sys.stderr.write(err + '\n')
        raise subp.CalledProcessError(p.returncode, bin_path)

    return out.strip()

def get_fncptr(src_c, entry_fnc):
    lib = ctypes.CDLL(get_lib(src_c))
    return getattr(lib, entry_fnc)

def test_constant_function():
    fn = get_fncptr("suite/test_constfunc.c", "test_fnc")
    assert fn() == 0

def test_factorial():
    fn = get_fncptr("suite/test_fib.c", "fib")
    assert fn(20) == 6765
