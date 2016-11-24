import subprocess as subp
import os, sys
import ctypes
import py
from multiprocessing import Process

CC = os.environ.get('CC', 'clang')
proj_dir = py.path.local(__file__).join('..', '..', '..')
test_jit_dir = proj_dir.join('tests', 'test_jit')
testsuite_dir = test_jit_dir.join('suite')
# testsuite_dir = py.path.local('/Users/johnz/Documents/Work/mu-client-pypy/rpython/translator/mu/test_impl')
bin_dir = py.path.local('/tmp')

if sys.platform.startswith('darwin'):
    libext = '.dylib'
elif sys.platform.startswith('linux'):
    libext = '.so'
else:
    libext = '.dll'
libmu_path = proj_dir.join('target', 'debug', 'libmu' + libext)


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
    p = subp.Popen([bin_path.strpath], stdout=subp.PIPE, stderr=subp.PIPE, env=os.environ)
    out, err = p.communicate()
    if p.returncode != 0:  # failed
        sys.stdout.write(out + '\n')
        sys.stderr.write(err + '\n')
        raise subp.CalledProcessError(p.returncode, bin_path)

    return py.path.local('emit').join('%(testname)s.dylib' % locals())


def fncptr_from_lib(lib, fnc_name, argtypes=[], restype=ctypes.c_longlong):
    fnp = getattr(lib, fnc_name)
    fnp.argtypes = argtypes
    fnp.restype = restype
    return fnp


def fncptr_from_c_script(c_src_name, name, argtypes=[], restype=ctypes.c_ulonglong):
    libpath = compile_c_script(c_src_name)
    lib = ctypes.CDLL(libpath.strpath)
    return fncptr_from_lib(lib, name, argtypes, restype), lib


def fncptr_from_py_script(py_fnc, name, argtypes=[], restype=ctypes.c_longlong):
    import os
    # NOTE: requires mu-client-pypy
    from rpython.rlib import rmu_fast as rmu

    # load libmu before rffi so to load it with RTLD_GLOBAL
    libmu = ctypes.CDLL(libmu_path.strpath, ctypes.RTLD_GLOBAL)

    loglvl = os.environ.get('MU_LOG_LEVEL', 'none')
    emit_dir = os.environ.get('MU_EMIT_DIR', 'emit')
    mu = rmu.MuVM("--log-level=%(loglvl)s --aot-emit-dir=%(emit_dir)s" % locals())
    ctx = mu.new_context()
    bldr = ctx.new_ir_builder()

    id_dict = py_fnc(bldr, rmu)
    bldr.load()
    libpath = py.path.local('lib%(name)s.dylib' % locals())
    mu.compile_to_sharedlib(libpath.strpath, [])

    lib = ctypes.CDLL(libpath.strpath)
    return fncptr_from_lib(lib, name, argtypes, restype), (mu, ctx, bldr)


def preload_libmu():
    # load libmu before rffi so to load it with RTLD_GLOBAL
    return ctypes.CDLL(libmu_path.strpath, ctypes.RTLD_GLOBAL)


spawn_proc = bool(int(os.environ.get('SPAWN_PROC', '1')))
def may_spawn_proc(test_fnc):
    def wrapper():
        if spawn_proc:
            p = Process(target=test_fnc, args=tuple())
            p.start()
            p.join()
            assert p.exitcode == 0
        else:
            test_fnc()
    return wrapper


def fncptr_from_rpy_func(rpy_fnc, llargtypes, llrestype, **kwargs):
    # NOTE: requires mu-client-pypy
    from rpython.rtyper.lltypesystem import rffi
    from rpython.translator.interactive import Translation
    from rpython.config.translationoption import set_opt_level

    preload_libmu()

    kwargs.setdefault('backend', 'mu')
    kwargs.setdefault('muimpl', 'fast')
    kwargs.setdefault('mucodegen', 'api')
    kwargs.setdefault('mutestjit', True)

    t = Translation(rpy_fnc, llargtypes, **kwargs)
    set_opt_level(t.config, '3')
    if kwargs['backend'] == 'mu':
        db, bdlgen, fnc_name = t.compile_mu()
        libpath = py.path.local('lib%(fnc_name)s.dylib' % locals())
        bdlgen.mu.compile_to_sharedlib(libpath.strpath, [])
        eci = rffi.ExternalCompilationInfo(libraries=[libpath.strpath])
        extras = (db, bdlgen)
    else:
        libpath = t.compile_c()
        fnc_name = 'pypy_g_' + rpy_fnc.__name__
        eci = rffi.ExternalCompilationInfo(libraries=[libpath.strpath])
        extras = None

    return rffi.llexternal(fnc_name, llargtypes, llrestype, compilation_info=eci, _nowrapper=True), extras
