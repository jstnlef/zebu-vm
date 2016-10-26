def test_add():
    src_c = "suite/test_add.c"
    entry_fnc = "test_fnc"

    lib = ctypes.CDLL(get_lib(src_c))
    fn = getattr(lib, entry_fnc)

    assert fn() == 9

def test_sub():
    src_c = "suite/test_sub.c"
    entry_fnc = "test_fnc"

    lib = ctypes.CDLL(get_lib(src_c))
    fn = getattr(lib, entry_fnc)

    assert fn() == 11

def test_mul():
    src_c = "suite/test_mul.c"
    entry_fnc = "test_fnc"

    lib = ctypes.CDLL(get_lib(src_c))
    fn = getattr(lib, entry_fnc)

    assert fn() == 0xf6

def test_sdiv():
    src_c = "suite/test_sdiv.c"
    entry_fnc = "test_fnc"

    lib = ctypes.CDLL(get_lib(src_c))
    fn = getattr(lib, entry_fnc)

    assert fn() == 0xf4

def test_urem():
    src_c = "suite/test_urem.c"
    entry_fnc = "test_fnc"

    lib = ctypes.CDLL(get_lib(src_c))
    fn = getattr(lib, entry_fnc)

    assert fn() == 5

def test_shl():
    src_c = "suite/test_shl.c"
    entry_fnc = "test_fnc"

    lib = ctypes.CDLL(get_lib(src_c))
    fn = getattr(lib, entry_fnc)

    assert fn() == 0x7e707560c92d5400

def test_lshr():
    src_c = "suite/test_shr.c"
    entry_fnc = "test_fnc"

    lib = ctypes.CDLL(get_lib(src_c))
    fn = getattr(lib, entry_fnc)

    assert fn() == 0x2367e707560c92

def test_and():
    src_c = "suite/test_and.c"
    entry_fnc = "test_fnc"

    lib = ctypes.CDLL(get_lib(src_c))
    fn = getattr(lib, entry_fnc)

    assert fn() == 0x8588901c10004b14

def test_xor():
    src_c = "suite/test_xor.c"
    entry_fnc = "test_fnc"

    lib = ctypes.CDLL(get_lib(src_c))
    fn = getattr(lib, entry_fnc)

    assert fn() == 0x58376ec3e83fa0e1
