import subprocess as subp
import os, sys
import ctypes
import py
from multiprocessing import Process, Queue, ProcessError

CC = os.environ.get('CC', 'clang')
proj_dir = py.path.local(__file__).join('..', '..', '..')
test_jit_dir = proj_dir.join('tests', 'test_jit')
testsuite_dir = test_jit_dir.join('suite')
bin_dir = test_jit_dir.join('temp')
if not bin_dir.exists():
    bin_dir.mkdir()
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

    # load libmu before rffi so to load it with RTLD_GLOBAL
    libmu = ctypes.CDLL(libmu_path.strpath, ctypes.RTLD_GLOBAL)

    mu = rmu.MuVM()
    ctx = mu.new_context()
    bldr = ctx.new_ir_builder()

    id_dict = py_fnc(bldr, rmu)
    bldr.load()
    libname = 'lib%(name)s.dylib' % locals()
    mu.compile_to_sharedlib(libname, [])

    lib = ctypes.CDLL('emit/%(libname)s' % locals())
    return fncptr_from_lib(lib, name, argtypes, restype), (mu, ctx, bldr)


def preload_libmu():
    # load libmu before rffi so to load it with RTLD_GLOBAL
    return ctypes.CDLL(libmu_path.strpath, ctypes.RTLD_GLOBAL)


def proc_call(fnc, args, block=True, timeout=1):
    # call function with an extra Queue parameter to pass the return value in a separate process
    q = Queue()
    rtn = None
    proc = Process(target=lambda *args: args[-1].put(fnc(*args[:-1])), args=args + (q,))
    proc.start()
    from Queue import Empty
    while proc.is_alive():
        try:
            rtn = q.get(False)
            break
        except Empty:
            pass

    if proc.is_alive():
        proc.join()
    if proc.exitcode != 0:
        proc.join()
        raise ProcessError("calling %(fnc)s with args %(args)s crashed with " % locals() + str(proc.exitcode))
    return rtn


def fncptr_from_rpy_func(rpy_fnc, llargtypes, llrestype, spawn_proc=True, **kwargs):
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
        libname = 'lib%(fnc_name)s.dylib' % locals()
        # run in a different process
        proc_call(bdlgen.mu.compile_to_sharedlib, args=(libname, []), block=False)

        eci = rffi.ExternalCompilationInfo(libraries=[test_jit_dir.join('emit', libname).strpath])
        extras = (db, bdlgen)
    else:
        libpath = t.compile_c()
        fnc_name = 'pypy_g_' + rpy_fnc.__name__
        eci = rffi.ExternalCompilationInfo(libraries=[libpath.strpath])
        extras = None

    return rffi.llexternal(fnc_name, llargtypes, llrestype, compilation_info=eci, _nowrapper=True), extras
