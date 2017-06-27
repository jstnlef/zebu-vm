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
    MuVM* mu_35;
    MuCtx* ctx_35;
    MuIRBuilder* bldr_35;
    MuID id_429;
    MuID id_430;
    MuID id_431;
    MuID id_432;
    MuID id_433;
    MuID id_434;
    MuID id_435;
    MuID id_436;
    MuID id_437;
    MuID id_438;
    MuID id_439;
    MuID id_440;
    MuID id_441;
    MuID id_442;
    mu_35 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_35 = mu_35->new_context(mu_35);
    bldr_35 = ctx_35->new_ir_builder(ctx_35);
    id_429 = bldr_35->gen_sym(bldr_35, "@dbl");
    bldr_35->new_type_double(bldr_35, id_429);
    id_430 = bldr_35->gen_sym(bldr_35, "@i1");
    bldr_35->new_type_int(bldr_35, id_430, 0x00000001ull);
    id_431 = bldr_35->gen_sym(bldr_35, "@i64");
    bldr_35->new_type_int(bldr_35, id_431, 0x00000040ull);
    id_432 = bldr_35->gen_sym(bldr_35, "@pi");
    bldr_35->new_const_double(bldr_35, id_432, id_429, *(double*)(uint64_t [1]){0x400921fb82c2bd7full});
    id_433 = bldr_35->gen_sym(bldr_35, "@e");
    bldr_35->new_const_double(bldr_35, id_433, id_429, *(double*)(uint64_t [1]){0x4005bf0995aaf790ull});
    id_434 = bldr_35->gen_sym(bldr_35, "@sig__i64");
    bldr_35->new_funcsig(bldr_35, id_434, NULL, 0, (MuTypeNode [1]){id_431}, 1);
    id_435 = bldr_35->gen_sym(bldr_35, "@test_fnc");
    bldr_35->new_func(bldr_35, id_435, id_434);
    id_436 = bldr_35->gen_sym(bldr_35, "@test_fnc.v1");
    id_437 = bldr_35->gen_sym(bldr_35, "@test_fnc.v1.blk0");
    id_438 = bldr_35->gen_sym(bldr_35, "@test_fnc.v1.blk0.cmpres");
    id_439 = bldr_35->gen_sym(bldr_35, "@test_fnc.v1.blk0.res");
    id_440 = bldr_35->gen_sym(bldr_35, NULL);
    bldr_35->new_cmp(bldr_35, id_440, id_438, MU_CMP_FOEQ, id_429, id_432, id_433);
    id_441 = bldr_35->gen_sym(bldr_35, NULL);
    bldr_35->new_conv(bldr_35, id_441, id_439, MU_CONV_ZEXT, id_430, id_431, id_438);
    id_442 = bldr_35->gen_sym(bldr_35, NULL);
    bldr_35->new_ret(bldr_35, id_442, (MuVarNode [1]){id_439}, 1);
    bldr_35->new_bb(bldr_35, id_437, NULL, NULL, 0, MU_NO_ID, (MuInstNode [3]){id_440, id_441, id_442}, 3);
    bldr_35->new_func_ver(bldr_35, id_436, id_435, (MuBBNode [1]){id_437}, 1);
    bldr_35->load(bldr_35);
    mu_35->compile_to_sharedlib(mu_35, LIB_FILE_NAME("test_double_ordered_eq"), NULL, 0);
    return 0;
}
