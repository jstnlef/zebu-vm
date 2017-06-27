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
    MuVM* mu_32;
    MuCtx* ctx_32;
    MuIRBuilder* bldr_32;
    MuID id_399;
    MuID id_400;
    MuID id_401;
    MuID id_402;
    MuID id_403;
    MuID id_404;
    MuID id_405;
    MuID id_406;
    MuID id_407;
    MuID id_408;
    mu_32 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_32 = mu_32->new_context(mu_32);
    bldr_32 = ctx_32->new_ir_builder(ctx_32);
    id_399 = bldr_32->gen_sym(bldr_32, "@dbl");
    bldr_32->new_type_double(bldr_32, id_399);
    id_400 = bldr_32->gen_sym(bldr_32, "@pi");
    bldr_32->new_const_double(bldr_32, id_400, id_399, *(double*)(uint64_t [1]){0x400921fb82c2bd7full});
    id_401 = bldr_32->gen_sym(bldr_32, "@e");
    bldr_32->new_const_double(bldr_32, id_401, id_399, *(double*)(uint64_t [1]){0x4005bf0995aaf790ull});
    id_402 = bldr_32->gen_sym(bldr_32, "@sig__dbl");
    bldr_32->new_funcsig(bldr_32, id_402, NULL, 0, (MuTypeNode [1]){id_399}, 1);
    id_403 = bldr_32->gen_sym(bldr_32, "@test_fnc");
    bldr_32->new_func(bldr_32, id_403, id_402);
    id_404 = bldr_32->gen_sym(bldr_32, "@test_fnc.v1");
    id_405 = bldr_32->gen_sym(bldr_32, "@test_fnc.v1.blk0");
    id_406 = bldr_32->gen_sym(bldr_32, "@test_fnc.v1.blk0.res");
    id_407 = bldr_32->gen_sym(bldr_32, NULL);
    bldr_32->new_binop(bldr_32, id_407, id_406, MU_BINOP_FSUB, id_399, id_400, id_401, MU_NO_ID);
    id_408 = bldr_32->gen_sym(bldr_32, NULL);
    bldr_32->new_ret(bldr_32, id_408, (MuVarNode [1]){id_406}, 1);
    bldr_32->new_bb(bldr_32, id_405, NULL, NULL, 0, MU_NO_ID, (MuInstNode [2]){id_407, id_408}, 2);
    bldr_32->new_func_ver(bldr_32, id_404, id_403, (MuBBNode [1]){id_405}, 1);
    bldr_32->load(bldr_32);
    mu_32->compile_to_sharedlib(mu_32, LIB_FILE_NAME("test_double_sub"), NULL, 0);
    return 0;
}
