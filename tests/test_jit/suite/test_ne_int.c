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
    MuVM* mu_16;
    MuCtx* ctx_16;
    MuIRBuilder* bldr_16;
    MuID id_159;
    MuID id_160;
    MuID id_161;
    MuID id_162;
    MuID id_163;
    MuID id_164;
    MuID id_165;
    MuID id_166;
    MuID id_167;
    MuID id_168;
    MuID id_169;
    MuID id_170;
    MuID id_171;
    mu_16 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_16 = mu_16->new_context(mu_16);
    bldr_16 = ctx_16->new_ir_builder(ctx_16);
    id_159 = bldr_16->gen_sym(bldr_16, "@i1");
    bldr_16->new_type_int(bldr_16, id_159, 0x00000001ull);
    id_160 = bldr_16->gen_sym(bldr_16, "@i64");
    bldr_16->new_type_int(bldr_16, id_160, 0x00000040ull);
    id_161 = bldr_16->gen_sym(bldr_16, "@0x8d9f9c1d58324b55_i64");
    bldr_16->new_const_int(bldr_16, id_161, id_160, 0x8d9f9c1d58324b55ull);
    id_162 = bldr_16->gen_sym(bldr_16, "@0xd5a8f2deb00debb4_i64");
    bldr_16->new_const_int(bldr_16, id_162, id_160, 0xd5a8f2deb00debb4ull);
    id_163 = bldr_16->gen_sym(bldr_16, "@sig__i64");
    bldr_16->new_funcsig(bldr_16, id_163, NULL, 0, (MuTypeNode [1]){id_160}, 1);
    id_164 = bldr_16->gen_sym(bldr_16, "@test_fnc");
    bldr_16->new_func(bldr_16, id_164, id_163);
    id_165 = bldr_16->gen_sym(bldr_16, "@test_fnc_v1");
    id_166 = bldr_16->gen_sym(bldr_16, "@test_fnc_v1.blk0");
    id_167 = bldr_16->gen_sym(bldr_16, "@test_fnc_v1.blk0.cmp_res");
    id_168 = bldr_16->gen_sym(bldr_16, "@test_fnc_v1.blk0.res");
    id_169 = bldr_16->gen_sym(bldr_16, NULL);
    bldr_16->new_cmp(bldr_16, id_169, id_167, MU_CMP_NE, id_160, id_161, id_162);
    id_170 = bldr_16->gen_sym(bldr_16, NULL);
    bldr_16->new_conv(bldr_16, id_170, id_168, MU_CONV_ZEXT, id_159, id_160, id_167);
    id_171 = bldr_16->gen_sym(bldr_16, NULL);
    bldr_16->new_ret(bldr_16, id_171, (MuVarNode [1]){id_168}, 1);
    bldr_16->new_bb(bldr_16, id_166, NULL, NULL, 0, MU_NO_ID, (MuInstNode [3]){id_169, id_170, id_171}, 3);
    bldr_16->new_func_ver(bldr_16, id_165, id_164, (MuBBNode [1]){id_166}, 1);
    bldr_16->load(bldr_16);
    mu_16->compile_to_sharedlib(mu_16, LIB_FILE_NAME("test_ne_int"), NULL, 0);
    return 0;
}
