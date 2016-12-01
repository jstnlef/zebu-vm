from util import fncptr_from_c_script, preload_libmu
import ctypes

def test_eq_int():
    fn, _ = fncptr_from_c_script("test_eq_int.c", "test_fnc")
    assert fn() == 0

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

def test_eq_ref():
    mu = mu_instance_via_ctyeps()
    fn, _ = fncptr_from_c_script("test_eq_ref.c", "test_fnc")
    assert fn() == 0

def test_ne_int():
    fn, _ = fncptr_from_c_script("test_ne_int.c", "test_fnc")
    assert fn() == 1

def test_ne_ref():
    mu = mu_instance_via_ctyeps()
    fn, _ = fncptr_from_c_script("test_ne_ref.c", "test_fnc")
    assert fn() == 1

def test_sge():
    fn, _ = fncptr_from_c_script("test_sge.c", "test_fnc", restype=ctypes.c_uint8)
    assert fn() == 1

def test_sgt():
    fn, _ = fncptr_from_c_script("test_sgt.c", "test_fnc", restype=ctypes.c_uint8)
    assert fn() == 0

def test_sle():
    fn, _ = fncptr_from_c_script("test_sle.c", "test_fnc", restype=ctypes.c_uint8)
    assert fn() == 1

def test_ule():
    fn, _ = fncptr_from_c_script("test_ule.c", "test_fnc", restype=ctypes.c_uint8)
    assert fn() == 1

def test_slt():
    fn, _ = fncptr_from_c_script("test_slt.c", "test_fnc", restype=ctypes.c_uint8)
    assert fn() == 0

def test_ult():
    fn, _ = fncptr_from_c_script("test_ult.c", "test_fnc", restype=ctypes.c_uint8)
    assert fn() == 0
