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
    MuVM* mu_31;
    MuCtx* ctx_31;
    MuIRBuilder* bldr_31;
    MuID id_389;
    MuID id_390;
    MuID id_391;
    MuID id_392;
    MuID id_393;
    MuID id_394;
    MuID id_395;
    MuID id_396;
    MuID id_397;
    MuID id_398;
    mu_31 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_31 = mu_31->new_context(mu_31);
    bldr_31 = ctx_31->new_ir_builder(ctx_31);
    id_389 = bldr_31->gen_sym(bldr_31, "@dbl");
    bldr_31->new_type_double(bldr_31, id_389);
    id_390 = bldr_31->gen_sym(bldr_31, "@pi");
    bldr_31->new_const_double(bldr_31, id_390, id_389, *(double*)(uint64_t [1]){0x400921fb82c2bd7full});
    id_391 = bldr_31->gen_sym(bldr_31, "@e");
    bldr_31->new_const_double(bldr_31, id_391, id_389, *(double*)(uint64_t [1]){0x4005bf0995aaf790ull});
    id_392 = bldr_31->gen_sym(bldr_31, "@sig__dbl");
    bldr_31->new_funcsig(bldr_31, id_392, NULL, 0, (MuTypeNode [1]){id_389}, 1);
    id_393 = bldr_31->gen_sym(bldr_31, "@test_fnc");
    bldr_31->new_func(bldr_31, id_393, id_392);
    id_394 = bldr_31->gen_sym(bldr_31, "@test_fnc.v1");
    id_395 = bldr_31->gen_sym(bldr_31, "@test_fnc.v1.blk0");
    id_396 = bldr_31->gen_sym(bldr_31, "@test_fnc.v1.blk0.res");
    id_397 = bldr_31->gen_sym(bldr_31, NULL);
    bldr_31->new_binop(bldr_31, id_397, id_396, MU_BINOP_FADD, id_389, id_390, id_391, MU_NO_ID);
    id_398 = bldr_31->gen_sym(bldr_31, NULL);
    bldr_31->new_ret(bldr_31, id_398, (MuVarNode [1]){id_396}, 1);
    bldr_31->new_bb(bldr_31, id_395, NULL, NULL, 0, MU_NO_ID, (MuInstNode [2]){id_397, id_398}, 2);
    bldr_31->new_func_ver(bldr_31, id_394, id_393, (MuBBNode [1]){id_395}, 1);
    bldr_31->load(bldr_31);
    mu_31->compile_to_sharedlib(mu_31, LIB_FILE_NAME("test_double_add"), NULL, 0);
    return 0;
}
