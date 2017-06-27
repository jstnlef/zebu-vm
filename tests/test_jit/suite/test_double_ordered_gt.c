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
    MuVM* mu_40;
    MuCtx* ctx_40;
    MuIRBuilder* bldr_40;
    MuID id_500;
    MuID id_501;
    MuID id_502;
    MuID id_503;
    MuID id_504;
    MuID id_505;
    MuID id_506;
    MuID id_507;
    MuID id_508;
    MuID id_509;
    MuID id_510;
    MuID id_511;
    MuID id_512;
    MuID id_513;
    mu_40 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_40 = mu_40->new_context(mu_40);
    bldr_40 = ctx_40->new_ir_builder(ctx_40);
    id_500 = bldr_40->gen_sym(bldr_40, "@dbl");
    bldr_40->new_type_double(bldr_40, id_500);
    id_501 = bldr_40->gen_sym(bldr_40, "@i1");
    bldr_40->new_type_int(bldr_40, id_501, 0x00000001ull);
    id_502 = bldr_40->gen_sym(bldr_40, "@i64");
    bldr_40->new_type_int(bldr_40, id_502, 0x00000040ull);
    id_503 = bldr_40->gen_sym(bldr_40, "@pi");
    bldr_40->new_const_double(bldr_40, id_503, id_500, *(double*)(uint64_t [1]){0x400921fb82c2bd7full});
    id_504 = bldr_40->gen_sym(bldr_40, "@e");
    bldr_40->new_const_double(bldr_40, id_504, id_500, *(double*)(uint64_t [1]){0x4005bf0995aaf790ull});
    id_505 = bldr_40->gen_sym(bldr_40, "@sig__i64");
    bldr_40->new_funcsig(bldr_40, id_505, NULL, 0, (MuTypeNode [1]){id_502}, 1);
    id_506 = bldr_40->gen_sym(bldr_40, "@test_fnc");
    bldr_40->new_func(bldr_40, id_506, id_505);
    id_507 = bldr_40->gen_sym(bldr_40, "@test_fnc.v1");
    id_508 = bldr_40->gen_sym(bldr_40, "@test_fnc.v1.blk0");
    id_509 = bldr_40->gen_sym(bldr_40, "@test_fnc.v1.blk0.cmpres");
    id_510 = bldr_40->gen_sym(bldr_40, "@test_fnc.v1.blk0.res");
    id_511 = bldr_40->gen_sym(bldr_40, NULL);
    bldr_40->new_cmp(bldr_40, id_511, id_509, MU_CMP_FOGT, id_500, id_503, id_504);
    id_512 = bldr_40->gen_sym(bldr_40, NULL);
    bldr_40->new_conv(bldr_40, id_512, id_510, MU_CONV_ZEXT, id_501, id_502, id_509);
    id_513 = bldr_40->gen_sym(bldr_40, NULL);
    bldr_40->new_ret(bldr_40, id_513, (MuVarNode [1]){id_510}, 1);
    bldr_40->new_bb(bldr_40, id_508, NULL, NULL, 0, MU_NO_ID, (MuInstNode [3]){id_511, id_512, id_513}, 3);
    bldr_40->new_func_ver(bldr_40, id_507, id_506, (MuBBNode [1]){id_508}, 1);
    bldr_40->load(bldr_40);
    mu_40->compile_to_sharedlib(mu_40, LIB_FILE_NAME("test_double_ordered_gt"), NULL, 0);
    return 0;
}
