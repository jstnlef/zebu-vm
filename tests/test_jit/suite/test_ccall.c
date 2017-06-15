// Copyright 2017 The Australian National University
// 
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// 
//     http://www.apache.org/licenses/LICENSE-2.0
// 
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
#ifdef __APPLE__
    #define LIB_EXT ".dylib"
#elif __linux__
    #define LIB_EXT ".so"
#elif _WIN32
    #define LIB_EXT ".dll"
#endif
#define LIB_FILE_NAME(name) "lib" name LIB_EXT
int main(int argc, char** argv) {
    MuVM* mu_26;
    MuCtx* ctx_26;
    MuIRBuilder* bldr_26;
    MuID id_331;
    MuID id_332;
    MuID id_333;
    MuID id_334;
    MuID id_335;
    MuID id_336;
    MuID id_337;
    MuID id_338;
    MuID id_339;
    MuID id_340;
    MuID id_341;
    mu_26 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_26 = mu_26->new_context(mu_26);
    bldr_26 = ctx_26->new_ir_builder(ctx_26);
    id_331 = bldr_26->gen_sym(bldr_26, "@i64");
    bldr_26->new_type_int(bldr_26, id_331, 0x00000040ull);
    id_332 = bldr_26->gen_sym(bldr_26, "@sig_i64_i64");
    bldr_26->new_funcsig(bldr_26, id_332, (MuTypeNode [1]){id_331}, 1, (MuTypeNode [1]){id_331}, 1);
    id_333 = bldr_26->gen_sym(bldr_26, "@fnpsig_i64_i64");
    bldr_26->new_type_ufuncptr(bldr_26, id_333, id_332);
    id_334 = bldr_26->gen_sym(bldr_26, "@c_fnc");
    bldr_26->new_const_extern(bldr_26, id_334, id_333, "fnc");
    id_335 = bldr_26->gen_sym(bldr_26, "@test_ccall");
    bldr_26->new_func(bldr_26, id_335, id_332);
    id_336 = bldr_26->gen_sym(bldr_26, "@test_ccall_v1");
    id_337 = bldr_26->gen_sym(bldr_26, "@test_ccall_v1.blk0");
    id_338 = bldr_26->gen_sym(bldr_26, "@test_ccall_v1.blk0.k");
    id_339 = bldr_26->gen_sym(bldr_26, "@test_ccall_v1.blk0.res");
    id_340 = bldr_26->gen_sym(bldr_26, NULL);
    bldr_26->new_ccall(bldr_26, id_340, (MuID [1]){id_339}, 1, MU_CC_DEFAULT, id_333, id_332, id_334, (MuVarNode [1]){id_338}, 1, MU_NO_ID, MU_NO_ID);
    id_341 = bldr_26->gen_sym(bldr_26, NULL);
    bldr_26->new_ret(bldr_26, id_341, (MuVarNode [1]){id_339}, 1);
    bldr_26->new_bb(bldr_26, id_337, (MuID [1]){id_338}, (MuTypeNode [1]){id_331}, 1, MU_NO_ID, (MuInstNode [2]){id_340, id_341}, 2);
    bldr_26->new_func_ver(bldr_26, id_336, id_335, (MuBBNode [1]){id_337}, 1);
    bldr_26->load(bldr_26);
    mu_26->compile_to_sharedlib(mu_26, LIB_FILE_NAME("test_ccall"), (char*[]){&"suite/test_ccall_fnc.c"}, 1);
    return 0;
}
