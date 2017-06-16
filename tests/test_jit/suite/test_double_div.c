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
    MuVM* mu_34;
    MuCtx* ctx_34;
    MuIRBuilder* bldr_34;
    MuID id_419;
    MuID id_420;
    MuID id_421;
    MuID id_422;
    MuID id_423;
    MuID id_424;
    MuID id_425;
    MuID id_426;
    MuID id_427;
    MuID id_428;
    mu_34 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_34 = mu_34->new_context(mu_34);
    bldr_34 = ctx_34->new_ir_builder(ctx_34);
    id_419 = bldr_34->gen_sym(bldr_34, "@dbl");
    bldr_34->new_type_double(bldr_34, id_419);
    id_420 = bldr_34->gen_sym(bldr_34, "@pi");
    bldr_34->new_const_double(bldr_34, id_420, id_419, *(double*)(uint64_t [1]){0x400921fb82c2bd7full});
    id_421 = bldr_34->gen_sym(bldr_34, "@e");
    bldr_34->new_const_double(bldr_34, id_421, id_419, *(double*)(uint64_t [1]){0x4005bf0995aaf790ull});
    id_422 = bldr_34->gen_sym(bldr_34, "@sig__dbl");
    bldr_34->new_funcsig(bldr_34, id_422, NULL, 0, (MuTypeNode [1]){id_419}, 1);
    id_423 = bldr_34->gen_sym(bldr_34, "@test_fnc");
    bldr_34->new_func(bldr_34, id_423, id_422);
    id_424 = bldr_34->gen_sym(bldr_34, "@test_fnc.v1");
    id_425 = bldr_34->gen_sym(bldr_34, "@test_fnc.v1.blk0");
    id_426 = bldr_34->gen_sym(bldr_34, "@test_fnc.v1.blk0.res");
    id_427 = bldr_34->gen_sym(bldr_34, NULL);
    bldr_34->new_binop(bldr_34, id_427, id_426, MU_BINOP_FDIV, id_419, id_420, id_421, MU_NO_ID);
    id_428 = bldr_34->gen_sym(bldr_34, NULL);
    bldr_34->new_ret(bldr_34, id_428, (MuVarNode [1]){id_426}, 1);
    bldr_34->new_bb(bldr_34, id_425, NULL, NULL, 0, MU_NO_ID, (MuInstNode [2]){id_427, id_428}, 2);
    bldr_34->new_func_ver(bldr_34, id_424, id_423, (MuBBNode [1]){id_425}, 1);
    bldr_34->load(bldr_34);
    mu_34->compile_to_sharedlib(mu_34, LIB_FILE_NAME("test_double_div"), NULL, 0);
    return 0;
}
