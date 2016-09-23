import injecttools
import os.path

_my_dir = os.path.dirname(__file__)
_mu_impl_fast_root = os.path.join(_my_dir, "..", "..", "..")

def _make_injectable_file_set(m):
    m2 = {os.path.join(_mu_impl_fast_root, k): v for k,v in m.items()}
    return InjectableFileSet(m2)

muapi_h_path = os.path.join(_my_dir, "muapi.h")

injectable_files = injecttools.make_injectable_file_set(_mu_impl_fast_root, [
    ("api_c.rs", "src/vm/api/api_c.rs",
        ["Types", "Structs", "Enums"]),
    ("api_bridge.rs", "src/vm/api/api_bridge.rs",
        ["Fillers", "Forwarders"]),
    ("__api_impl_stubs.rs", "src/vm/api/__api_impl_stubs.rs",
        ["StubImpls"]),
    ])
