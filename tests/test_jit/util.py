import subprocess as subp
import os, sys
import ctypes
import py

CC = os.environ.get('CC', 'clang')
proj_dir = py.path.local(__file__).join('..', '..', '..')
test_jit_dir = proj_dir.join('tests', 'test_jit')
testsuite_dir = test_jit_dir.join('suite')
bin_dir = test_jit_dir.join('temp')
if not bin_dir.exists():
    bin_dir.mkdir()
libmu_path = proj_dir.join('target', 'debug', 'libmu.dylib')


def compile_c_script(c_src_name):
    testname = c_src_name[:-2]
    src_c = testsuite_dir.join(c_src_name)
    bin_path = bin_dir.join(testname)
    CFLAGS = [
        "-std=c11",
        "-I%(proj_dir)s/src/vm/api" % globals(),
        "-L" + libmu_path.dirname,
        "-lmu",
    ]
    cmd = [CC] + CFLAGS + ['-o', bin_path.strpath] + [src_c.strpath]

    # compile
    p = subp.Popen(cmd, stdout=subp.PIPE, stderr=subp.PIPE, env=os.environ)
    out, err = p.communicate()
    if p.returncode != 0:  # failed
        sys.stdout.write(out + '\n')
        sys.stderr.write(err + '\n')
        raise subp.CalledProcessError(p.returncode, cmd)

    os.environ['LD_LIBRARY_PATH'] = "%s:%s" % ("%(proj_dir)s/target/debug" % globals(),
                                               os.environ['LD_LIBRARY_PATH'] if 'LD_LIBRARY_PATH' in os.environ else "")
    # run
    p = subp.Popen([bin_path.strpath], stdout=subp.PIPE, stderr=subp.PIPE, cwd=str(bin_dir), env=os.environ)
    out, err = p.communicate()
    if p.returncode != 0:  # failed
        sys.stdout.write(out + '\n')
        sys.stderr.write(err + '\n')
        raise subp.CalledProcessError(p.returncode, bin_path)

    return bin_dir.join('emit', '%(testname)s.dylib' % locals()).strpath


def fncptr_from_lib(lib, fnc_name, argtypes=[], restype=ctypes.c_longlong):
    fnp = getattr(lib, fnc_name)
    fnp.argtypes = argtypes
    fnp.restype = restype
    return fnp

def fncptr_from_c_script(c_src_name, name, argtypes=[], restype=ctypes.c_ulonglong):
    lib_path = compile_c_script(c_src_name)
    lib = ctypes.CDLL(lib_path)
    return fncptr_from_lib(lib, name, argtypes, restype), lib

def fncptr_from_py_script(py_fnc, name, argtypes=[], restype=ctypes.c_longlong):
    # NOTE: requires mu-client-pypy
    from rpython.rlib import rmu_fast as rmu

    mu = rmu.MuVM()
    ctx = mu.new_context()
    bldr = ctx.new_ir_builder()

    id_dict = py_fnc(bldr, rmu)
    bldr.load()
    libname = 'lib%(name)s.dylib' % locals()
    mu.compile_to_sharedlib(libname, [])

    lib = ctypes.CDLL('emit/%(libname)s' % locals())
    return fncptr_from_lib(lib, name, argtypes, restype), (mu, ctx, bldr)

def fncptr_from_rpy_func(rpy_fnc, llargtypes, llrestype, **kwargs):
    # NOTE: requires mu-client-pypy
    from rpython.rtyper.lltypesystem import rffi
    from rpython.translator.interactive import Translation

    kwargs.setdefault('backend', 'mu')
    kwargs.setdefault('muimpl', 'fast')
    kwargs.setdefault('mucodegen', 'api')
    kwargs.setdefault('mutestjit', True)

    t = Translation(rpy_fnc, llargtypes, **kwargs)
    if kwargs['backend'] == 'mu':
        db, bdlgen, fnc_name = t.compile_mu()
        libname = 'lib%(fnc_name)s.dylib' % locals()
        bdlgen.mu.compile_to_sharedlib(libname, [])
        eci = rffi.ExternalCompilationInfo(libraries=[test_jit_dir.join('emit', libname).strpath])
        extras = (db, bdlgen)
    else:
        libpath = t.compile_c()
        fnc_name = 'pypy_g_' + rpy_fnc.__name__
        eci = rffi.ExternalCompilationInfo(libraries=[libpath.strpath])
        extras = None

    return rffi.llexternal(fnc_name, llargtypes, llrestype, compilation_info=eci, _nowrapper=True), extras
