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


def mu_instance_via_ctyeps():
    libmu = preload_libmu()
    class MuVM(ctypes.Structure):
        pass
    MuVM._fields_ = [
            ('header', ctypes.c_voidp),
            ('new_context', ctypes.c_voidp),    # function pointers should have the same size as c_voidp
            ('id_of', ctypes.c_voidp),
            ('name_of', ctypes.c_voidp),
            ('set_trap_handler', ctypes.c_voidp),
            ('compile_to_sharedlib', ctypes.c_voidp),
            ('current_thread_as_mu_thread', ctypes.CFUNCTYPE(None, ctypes.POINTER(MuVM), ctypes.c_voidp)),
        ]
    libmu.mu_fastimpl_new.restype = ctypes.POINTER(MuVM)
    mu = libmu.mu_fastimpl_new()
    mu.contents.current_thread_as_mu_thread(mu, None)
    return mu


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

    return py.path.local('emit').join('lib%(testname)s' % locals() + libext)


def ctypes_fncptr_from_lib(libpath, fnc_name, argtypes=[], restype=ctypes.c_longlong, mode=ctypes.RTLD_LOCAL):
    lib = ctypes.CDLL(libpath.strpath, mode)
    fnp = getattr(lib, fnc_name)
    fnp.argtypes = argtypes
    fnp.restype = restype
    return fnp, lib


def rffi_fncptr_from_lib(libpath, fnc_name, llargtypes, restype, mode=ctypes.RTLD_LOCAL):
    from rpython.rtyper.lltypesystem import rffi
    from rpython.translator.platform import platform
    if platform.name.startswith('linux'):
        link_extra = ['-Wl,-R' + libpath.dirpath().strpath]
    else:
        link_extra = []
    libname = libpath.basename[3:libpath.basename.index(libext)]

    if mode == ctypes.RTLD_GLOBAL:
        lib = ctypes.CDLL(libpath.strpath, mode)    # preload lib using RTLD_GLOBAL

    return rffi.llexternal(fnc_name, llargtypes, restype,
                           compilation_info=rffi.ExternalCompilationInfo(
                               libraries=[libname],
                               library_dirs=[libpath.dirpath().strpath],
                               link_extra=link_extra
                           ),
                           _nowrapper=True)


def fncptr_from_c_script(c_src_name, name, argtypes=[], restype=ctypes.c_ulonglong, mode=ctypes.RTLD_LOCAL):
    libpath = compile_c_script(c_src_name)
    return ctypes_fncptr_from_lib(libpath, name, argtypes, restype, mode)


def is_ctypes(t):
    return isinstance(t, type(ctypes.c_longlong))


def fncptr_from_py_script(py_fnc, heapinit_fnc, name, argtypes=[], restype=ctypes.c_longlong, mode=ctypes.RTLD_LOCAL, **kwargs):
    import os
    # NOTE: requires mu-client-pypy
    from rpython.rlib.rmu import zebu as rmu

    # load libmu before rffi so to load it with RTLD_GLOBAL
    libmu = preload_libmu()

    loglvl = os.environ.get('MU_LOG_LEVEL', 'none')
    emit_dir = kwargs.get('muemitdir', os.environ.get('MU_EMIT_DIR', 'emit'))
    mu = rmu.MuVM("--log-level=%(loglvl)s --aot-emit-dir=%(emit_dir)s" % locals())
    ctx = mu.new_context()
    bldr = ctx.new_ir_builder()

    id_dict = py_fnc(bldr, rmu)
    bldr.load()
    if heapinit_fnc:
        heapinit_fnc(ctx, id_dict, rmu)
    libpath = py.path.local(emit_dir).join('lib%(name)s' % locals() + libext)
    mu.compile_to_sharedlib(libpath.strpath, [])

    if (len(argtypes) > 0 and is_ctypes(argtypes[0])) or is_ctypes(restype):
        return ctypes_fncptr_from_lib(libpath, name, argtypes, restype, mode), (mu, ctx, bldr)
    else:
        return rffi_fncptr_from_lib(libpath, name, argtypes, restype, mode), (mu, ctx, bldr)


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


def fncptr_from_rpy_func(rpy_fnc, llargtypes, llrestype, mode=ctypes.RTLD_LOCAL, **kwargs):
    # NOTE: requires mu-client-pypy
    from rpython.rtyper.lltypesystem import rffi
    from rpython.translator.interactive import Translation
    from rpython.config.translationoption import set_opt_level

    preload_libmu()
    emit_dir = os.environ.get('MU_EMIT_DIR', 'emit')
    kwargs.setdefault('backend', 'mu')
    kwargs.setdefault('impl', 'zebu')
    kwargs.setdefault('codegen', 'api')
    kwargs.setdefault('testjit', True)
    kwargs.setdefault('vmargs', "--aot-emit-dir=" + emit_dir)

    t = Translation(rpy_fnc, llargtypes, **kwargs)
    set_opt_level(t.config, '3')
    if kwargs['backend'] == 'mu':
        db, bdlgen, fnc_name = t.compile_mu()
        emit_dir = py.path.local(emit_dir)
        libpath = emit_dir.join('lib%(fnc_name)s' % locals() + libext)
        bdlgen.mu.compile_to_sharedlib(libpath.strpath, [])
        extras = (db, bdlgen)
    else:
        libpath = t.compile_c()
        fnc_name = 'pypy_g_' + rpy_fnc.__name__
        extras = None
    return rffi_fncptr_from_lib(libpath, fnc_name, llargtypes, llrestype, mode), extras
