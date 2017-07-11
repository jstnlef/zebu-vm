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

import os, subprocess, ctypes, sys;
import struct;

muc = os.environ.get('MUC', 'muc'); #type: str
emit = os.environ.get('MU_EMIT_DIR', 'emit'); #type: str
libext = '.dylib' if sys.platform.startswith('darwin') else \
         '.so'    if sys.platform.startswith('linux') else sys.exit("Unsupported platform");

prelude = """
        /*--------------------------------------------------------*/
        .funcsig exit_sig = (int<32>) -> ()
        .typedef exit_type = ufuncptr<exit_sig>
        .const exit <exit_type> = EXTERN "exit"
        .typedef char = int<8>
        .funcsig main_sig = (int<32> uptr<uptr<char>>)->(int<32>)
        /*--------------------------------------------------------*/
"""

""" Makes a primordial function that calls the given 'main' function (which should have the same signature as a C main function) """
def make_primordial(main): # type: (str) -> str
    return """
        /*--------------------------------------------------------*/
        .funcdef primordial <(int<32> uptr<uptr<char>>)->()>
        {
            entry(<int<32>>argc <uptr<uptr<char>>>argv):
                res = CALL <main_sig> """ + main + """(argc argv)
                CCALL #DEFAULT <exit_type exit_sig> exit(res) 
                RET // Unreachable
        }
        /*--------------------------------------------------------*/
        """;

def get_output_file(name): # type: (str) -> str
    return os.path.join(emit, name);

def execute_muc(bundle, name, primordial=None): # type: (str, str, Optional[str]) -> None
    sys.stderr.write(bundle);
    muc_proc = subprocess.Popen([muc, "-r"]
        + (["-f", primordial] if primordial is not None else [])
        +  ["/dev/stdin", get_output_file(name)],
        stdin = subprocess.PIPE); #type: subprocess.Popen
    muc_proc.communicate(bundle); # Send the bundle to muc
    assert (muc_proc.returncode == 0); # Check that muc worked

def compile_bundle(bundle, name, main = None): # type: (str, str, Optional[str]) -> None
    execute_muc(prelude + bundle + make_primordial(main if main is not None else name), name, "primordial");

def execute(name, args = []): # type: (str, Optional[List[str]]) -> int
    return subprocess.call([get_output_file(name)] + args);

def load_bundle(bundle, name): # type: (str, str) -> ctypes.CDLL
    execute_muc(prelude + bundle, name + libext);
    return ctypes.CDLL(get_output_file(name + libext));

def get_function(func, argtypes, restype): # type: (ctypes._FuncPtr) -> (ctypes._FuncPtr)
    func.argtypes = argtypes;
    func.restype = restype;
    return func;