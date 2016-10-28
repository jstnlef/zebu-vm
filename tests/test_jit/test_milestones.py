"""
Harness JIT tests using py.test framework
"""
import subprocess as subp
import os, sys
import ctypes
import py

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

testsuite_dir = py.path.local('suite')

def compile_lib(testname):
    src_c = testsuite_dir.join(testname + '.c')
    bin_dir = py.path.local('tmp')
    if not bin_dir.exists():
        bin_dir.mkdir()

    bin_path = bin_dir.join(testname)
    cmd = [CC] + CFLAGS + ['-o', str(bin_path)] + [str(src_c)]

    # compile
    p = subp.Popen(cmd, stdout=subp.PIPE, stderr=subp.PIPE, env=os.environ)
    out, err = p.communicate()
    if p.returncode != 0:  # failed
        sys.stdout.write(out + '\n')
        sys.stderr.write(err + '\n')
        raise subp.CalledProcessError(p.returncode, cmd)

    # run
    p = subp.Popen([str(bin_path)], stdout=subp.PIPE, stderr=subp.PIPE,
                   cwd=str(bin_dir), env=os.environ)
    out, err = p.communicate()
    if p.returncode != 0:  # failed
        sys.stdout.write(out + '\n')
        sys.stderr.write(err + '\n')
        raise subp.CalledProcessError(p.returncode, bin_path)

    return str(bin_dir.join('emit', testname + '.dylib'))

def get_fncptr(testname, entry_fnc):
    lib_path = compile_lib(testname)
    lib = ctypes.CDLL(lib_path)
    return getattr(lib, entry_fnc)

def test_constant_function():
    fn = get_fncptr("test_constfunc", "test_fnc")
    assert fn() == 0

def test_fibonacci():
    fn = get_fncptr("test_fib", "fib")
    assert fn(20) == 6765

def test_multifunc():
    fn = get_fncptr("test_multifunc", "entry")
    assert fn() == 6765