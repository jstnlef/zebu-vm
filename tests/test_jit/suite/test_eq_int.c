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
    MuVM* mu_14;
    MuCtx* ctx_14;
    MuIRBuilder* bldr_14;
    MuID id_131;
    MuID id_132;
    MuID id_133;
    MuID id_134;
    MuID id_135;
    MuID id_136;
    MuID id_137;
    MuID id_138;
    MuID id_139;
    MuID id_140;
    MuID id_141;
    MuID id_142;
    MuID id_143;
    mu_14 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_14 = mu_14->new_context(mu_14);
    bldr_14 = ctx_14->new_ir_builder(ctx_14);
    id_131 = bldr_14->gen_sym(bldr_14, "@i1");
    bldr_14->new_type_int(bldr_14, id_131, 0x00000001ull);
    id_132 = bldr_14->gen_sym(bldr_14, "@i64");
    bldr_14->new_type_int(bldr_14, id_132, 0x00000040ull);
    id_133 = bldr_14->gen_sym(bldr_14, "@0x8d9f9c1d58324b55_i64");
    bldr_14->new_const_int(bldr_14, id_133, id_132, 0x8d9f9c1d58324b55ull);
    id_134 = bldr_14->gen_sym(bldr_14, "@0xd5a8f2deb00debb4_i64");
    bldr_14->new_const_int(bldr_14, id_134, id_132, 0xd5a8f2deb00debb4ull);
    id_135 = bldr_14->gen_sym(bldr_14, "@sig__i64");
    bldr_14->new_funcsig(bldr_14, id_135, NULL, 0, (MuTypeNode [1]){id_132}, 1);
    id_136 = bldr_14->gen_sym(bldr_14, "@test_fnc");
    bldr_14->new_func(bldr_14, id_136, id_135);
    id_137 = bldr_14->gen_sym(bldr_14, "@test_fnc_v1");
    id_138 = bldr_14->gen_sym(bldr_14, "@test_fnc_v1.blk0");
    id_139 = bldr_14->gen_sym(bldr_14, "@test_fnc_v1.blk0.cmp_res");
    id_140 = bldr_14->gen_sym(bldr_14, "@test_fnc_v1.blk0.res");
    id_141 = bldr_14->gen_sym(bldr_14, NULL);
    bldr_14->new_cmp(bldr_14, id_141, id_139, MU_CMP_EQ, id_132, id_133, id_134);
    id_142 = bldr_14->gen_sym(bldr_14, NULL);
    bldr_14->new_conv(bldr_14, id_142, id_140, MU_CONV_ZEXT, id_131, id_132, id_139);
    id_143 = bldr_14->gen_sym(bldr_14, NULL);
    bldr_14->new_ret(bldr_14, id_143, (MuVarNode [1]){id_140}, 1);
    bldr_14->new_bb(bldr_14, id_138, NULL, NULL, 0, MU_NO_ID, (MuInstNode [3]){id_141, id_142, id_143}, 3);
    bldr_14->new_func_ver(bldr_14, id_137, id_136, (MuBBNode [1]){id_138}, 1);
    bldr_14->load(bldr_14);
    mu_14->compile_to_sharedlib(mu_14, LIB_FILE_NAME("test_eq_int"), NULL, 0);
    return 0;
}
