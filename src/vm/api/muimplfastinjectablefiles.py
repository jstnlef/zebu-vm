# Copyright 2017 The Australian National University
# 
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
# 
#     http://www.apache.org/licenses/LICENSE-2.0
# 
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

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
