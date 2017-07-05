from util import execute, compile_bundle, load_bundle, get_function;
import pytest;
import ctypes, struct, math;

def transmute_float_to_int(f): # type: (float)->int
    return int(struct.unpack('P', struct.pack('d', float(f)))[0]);
def transmute_int_to_float(i): # type: (int)->float
    return float(struct.unpack('d', struct.pack('P', int(i)))[0]);

def test_is_int():
    lib = load_bundle("""
         .funcdef tr64_is_int <(tagref64)->(int<1>)>
        {
            entry(<tagref64> tr):
                res = COMMINST uvm.tr64.is_int (tr)
                RET res
        }    
    """, "tr64_is_int"); # type: ctypes.CDLL
    tr64_is_int = get_function(lib.tr64_is_int, [ctypes.c_uint64], ctypes.c_bool);
    assert(tr64_is_int(0x7ff0000000000001));
    assert(tr64_is_int(0xfff0000000000001));
    assert(tr64_is_int(0xffffffffffffffff));

def test_is_ref():
    lib = load_bundle("""
         .funcdef tr64_is_ref <(tagref64)->(int<1>)>
        {
            entry(<tagref64> tr):
                res = COMMINST uvm.tr64.is_ref (tr)
                RET res
        }    
    """, "tr64_is_ref"); # type: ctypes.CDLL
    tr64_is_ref = get_function(lib.tr64_is_ref, [ctypes.c_uint64], ctypes.c_bool);
    assert(tr64_is_ref(0x7ff0000000000002));
    assert(tr64_is_ref(0xfff0000000000002));
    assert(tr64_is_ref(0xfffffffffffffffe));

def test_is_fp():
    lib = load_bundle("""
         .funcdef tr64_is_fp <(tagref64)->(int<1>)>
        {
            entry(<tagref64> tr):
                res = COMMINST uvm.tr64.is_fp(tr)
                RET res
        }    
    """, "tr64_is_fp"); # type: ctypes.CDLL
    tr64_is_fp = get_function(lib.tr64_is_fp, [ctypes.c_uint64], ctypes.c_bool);
    assert(tr64_is_fp(0x0));
    assert(tr64_is_fp(0x123456789abcdef0));
    assert(tr64_is_fp(0x7ff123456789abcc));
    assert(tr64_is_fp(0xfffffffffffffffc));
    assert(tr64_is_fp(transmute_float_to_int(3.1415927)));

def test_from_int():
    lib = load_bundle("""
         .funcdef tr64_from_int <(int<52>)->(tagref64)>
        {
            entry(<int<52>> val):
                res = COMMINST uvm.tr64.from_int(val)
                RET res
        }    
    """, "tr64_from_int"); # type: ctypes.CDLL
    tr64_from_int = get_function(lib.tr64_from_int, [ctypes.c_uint64], ctypes.c_uint64);
    assert(tr64_from_int(0x0000000000000) == 0x7ff0000000000001);
    assert(tr64_from_int(0xfffffffffffff) == 0xffffffffffffffff);
    assert(tr64_from_int(0x5555555555555) == 0x7ffaaaaaaaaaaaab);
    assert(tr64_from_int(0xaaaaaaaaaaaaa) == 0xfff5555555555555);

def test_from_fp():
    lib = load_bundle("""
         .funcdef tr64_from_fp <(double)->(tagref64)>
        {
            entry(<double> val):
                res = COMMINST uvm.tr64.from_fp(val)
                RET res
        }    
    """, "tr64_from_fp"); # type: ctypes.CDLL
    tr64_from_fp = get_function(lib.tr64_from_fp, [ctypes.c_double], ctypes.c_uint64);
    assert(tr64_from_fp(3.14) == transmute_float_to_int(3.14));
    assert(tr64_from_fp(-3.14) == transmute_float_to_int(-3.14));
    assert(tr64_from_fp(float("inf")) == 0x7ff0000000000000);
    assert(tr64_from_fp(transmute_int_to_float(0x7ff123456789abcd)) == 0x7ff0000000000008);
    assert(math.isnan(transmute_int_to_float(tr64_from_fp(transmute_int_to_float(0x7ff123456789abcd)))));

def test_from_ref():
    lib = load_bundle("""
         .funcdef tr64_from_ref <(ref<void> int<6>)->(tagref64)>
        {
            entry(<ref<void>>%ref <int<6>> tag):
                res = COMMINST uvm.tr64.from_ref(%ref tag)
                RET res
        }    
    """, "tr64_from_ref"); # type: ctypes.CDLL
    tr64_from_ref = get_function(lib.tr64_from_ref, [ctypes.c_void_p, ctypes.c_uint8], ctypes.c_uint64);
    assert(tr64_from_ref(0x000000000000, 0x00) == 0x7ff0000000000002);
    assert(tr64_from_ref(0x7ffffffffff8, 0x00) == 0x7ff07ffffffffffa);
    assert(tr64_from_ref(0xfffffffffffffff8, 0x00) == 0xfff07ffffffffffa);
    assert(tr64_from_ref(0x000000000000, 0x3f) == 0x7fff800000000006);

def test_to_int():
    lib = load_bundle("""
         .funcdef tr64_to_int <(tagref64)->(int<52>)>
        {
            entry(<tagref64> tr):
                res = COMMINST uvm.tr64.to_int(tr)
                RET res
        }    
    """, "tr64_to_int"); # type: ctypes.CDLL
    tr64_to_int = get_function(lib.tr64_to_int, [ctypes.c_uint64], ctypes.c_uint64);
    assert(tr64_to_int(0x7ff0000000000001) == 0);
    assert(tr64_to_int(0xfff0000000000001) == 0x8000000000000);
    assert(tr64_to_int(0xfff5555555555555) == 0xaaaaaaaaaaaaa);
    assert(tr64_to_int(0x7ffaaaaaaaaaaaab) == 0x5555555555555);

def test_to_fp():
    lib = load_bundle("""
         .funcdef tr64_to_fp <(tagref64)->(double)>
        {
            entry(<tagref64> tr):
                res = COMMINST uvm.tr64.to_fp(tr)
                RET res
        }    
    """, "tr64_to_fp"); # type: ctypes.CDLL
    tr64_to_fp = get_function(lib.tr64_to_fp, [ctypes.c_uint64], ctypes.c_double);
    assert(tr64_to_fp(0x0000000000000000) == 0.0);
    assert(tr64_to_fp(0x8000000000000000) == -0.0);
    assert(tr64_to_fp(0x3ff0000000000000) == 1.0);
    assert(math.isnan(tr64_to_fp(0x7ff0000000000008)));

def test_to_ref():
    lib = load_bundle("""
         .funcdef tr64_to_ref <(tagref64)->(ref<void>)>
        {
            entry(<tagref64> tr):
                res = COMMINST uvm.tr64.to_ref(tr)
                RET res
        }    
    """, "tr64_to_ref"); # type: ctypes.CDLL
    tr64_to_ref = get_function(lib.tr64_to_ref, [ctypes.c_uint64], ctypes.c_void_p);
    assert(tr64_to_ref(0x7ff0555555555552) == 0x555555555550);
    assert(tr64_to_ref(0xfff02aaaaaaaaaaa) == 0xffffaaaaaaaaaaa8);

def test_to_tag():
    lib = load_bundle("""
         .funcdef tr64_to_tag <(tagref64)->(int<6>)>
        {
            entry(<tagref64> tr):
                res = COMMINST uvm.tr64.to_tag(tr)
                RET res
        }    
    """, "tr64_to_tag"); # type: ctypes.CDLL
    tr64_to_tag = get_function(lib.tr64_to_tag, [ctypes.c_uint64], ctypes.c_uint8);
    assert(tr64_to_tag(0x7ff0555555555552) == 0);
    assert(tr64_to_tag(0x7fff800000000006) == 0x3f);
    assert(tr64_to_tag(0x7ffa800000000002) == 0x2a);
    assert(tr64_to_tag(0x7ff5000000000006) == 0x15);

def test_from_int_imm():
    bundle_template = """
         .funcdef tr64_from_int <()->(tagref64)>
        {{
            entry():
                res = COMMINST uvm.tr64.from_int(<int<52>>{})
                RET res
        }}   
        """; # type: str

    def tr64_from_int(val): # type: (str) -> int
        lib = load_bundle(bundle_template.format(val), "tr64_from_int_{}".format(val));
        return get_function(lib.tr64_from_int, [], ctypes.c_uint64)();

    assert(tr64_from_int("0x0000000000000") == 0x7ff0000000000001);
    assert(tr64_from_int("0xfffffffffffff") == 0xffffffffffffffff);
    assert(tr64_from_int("0x5555555555555") == 0x7ffaaaaaaaaaaaab);
    assert(tr64_from_int("0xaaaaaaaaaaaaa") == 0xfff5555555555555);
def test_from_fp_imm():
    bundle_template = """
         .funcdef tr64_from_fp <()->(tagref64)>
        {{
            entry():
                res = COMMINST uvm.tr64.from_fp(<double>{})
                RET res
        }}    
    """; # type: str
    def tr64_from_fp(val): # type: (str) -> int
        lib = load_bundle(bundle_template.format(val), "tr64_from_fp_{}".format(val));
        return get_function(lib.tr64_from_fp, [], ctypes.c_uint64)();

    assert(tr64_from_fp("3.14 d") == transmute_float_to_int(3.14));
    assert(tr64_from_fp("-3.14 d") == transmute_float_to_int(-3.14));
    assert(tr64_from_fp("+inf d") == 0x7ff0000000000000);
    assert(tr64_from_fp("bitsd(0x7ff123456789abcd)") == 0x7ff0000000000008);
    assert(math.isnan(transmute_int_to_float(tr64_from_fp("bitsd(0x7ff123456789abcd)"))));

def test_from_ref_imm():
    lib = load_bundle("""
     .funcdef tr64_from_tag <(int<6>)->(tagref64)>
    {
        entry(<int<6>> tag):
            res = COMMINST uvm.tr64.from_ref(<ref<void>>NULL tag)
            RET res
    }""", "tr64_from_tag"); # type: ctypes.CDLL
    tr64_from_tag = get_function(lib.tr64_from_tag, [ctypes.c_uint8], ctypes.c_uint64);
    assert(tr64_from_tag(0x00) == 0x7ff0000000000002);
    assert(tr64_from_tag(0x3f) == 0x7fff800000000006);

def test_from_tag_imm():
    bundle_template = """
         .funcdef tr64_from_ref <(ref<void>)->(tagref64)>
        {{
            entry(<ref<void>>%ref):
                res = COMMINST uvm.tr64.from_ref(%ref <int<6>>{})
                RET res
        }}    
    """; # type: str

    def tr64_from_ref(ref, val): # type: (int, str) -> int
        lib = load_bundle(bundle_template.format(val), "tr64_from_ref_{}".format(val));
        return get_function(lib.tr64_from_ref, [ctypes.c_void_p], ctypes.c_uint64)(ref);

    assert(tr64_from_ref(0x000000000000, "0x00") == 0x7ff0000000000002);
    assert(tr64_from_ref(0x7ffffffffff8, "0x00") == 0x7ff07ffffffffffa);
    assert(tr64_from_ref(0xfffffffffffffff8, "0x00") == 0xfff07ffffffffffa);
    assert(tr64_from_ref(0x000000000000, "0x3f") == 0x7fff800000000006);
    
def test_from_tag_ref_imm():
    bundle_template = """
         .funcdef tr64_from_tag_ref <()->(tagref64)>
        {{
            entry():
                res = COMMINST uvm.tr64.from_ref(<ref<void>>NULL <int<6>>{})
                RET res
        }}    
    """; # type: str

    def tr64_from_tag_ref(val): # type: (str) -> int
        lib = load_bundle(bundle_template.format(val), "tr64_from_tag_ref_{}".format(val));
        return get_function(lib.tr64_from_tag_ref, [], ctypes.c_uint64)();

    assert(tr64_from_tag_ref("0x00") == 0x7ff0000000000002);
    assert(tr64_from_tag_ref("0x3f") == 0x7fff800000000006);