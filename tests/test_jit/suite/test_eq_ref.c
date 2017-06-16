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
    MuVM* mu_15;
    MuCtx* ctx_15;
    MuIRBuilder* bldr_15;
    MuID id_144;
    MuID id_145;
    MuID id_146;
    MuID id_147;
    MuID id_148;
    MuID id_149;
    MuID id_150;
    MuID id_151;
    MuID id_152;
    MuID id_153;
    MuID id_154;
    MuID id_155;
    MuID id_156;
    MuID id_157;
    MuID id_158;
    mu_15 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_15 = mu_15->new_context(mu_15);
    bldr_15 = ctx_15->new_ir_builder(ctx_15);
    id_144 = bldr_15->gen_sym(bldr_15, "@i1");
    bldr_15->new_type_int(bldr_15, id_144, 0x00000001ull);
    id_145 = bldr_15->gen_sym(bldr_15, "@i64");
    bldr_15->new_type_int(bldr_15, id_145, 0x00000040ull);
    id_146 = bldr_15->gen_sym(bldr_15, "@refi64");
    bldr_15->new_type_ref(bldr_15, id_146, id_145);
    id_147 = bldr_15->gen_sym(bldr_15, "@NULL_refi64");
    bldr_15->new_const_null(bldr_15, id_147, id_146);
    id_148 = bldr_15->gen_sym(bldr_15, "@sig__i64");
    bldr_15->new_funcsig(bldr_15, id_148, NULL, 0, (MuTypeNode [1]){id_145}, 1);
    id_149 = bldr_15->gen_sym(bldr_15, "@test_fnc");
    bldr_15->new_func(bldr_15, id_149, id_148);
    id_150 = bldr_15->gen_sym(bldr_15, "@test_fnc_v1");
    id_151 = bldr_15->gen_sym(bldr_15, "@test_fnc_v1.blk0");
    id_152 = bldr_15->gen_sym(bldr_15, "@test_fnc_v1.blk0.r");
    id_153 = bldr_15->gen_sym(bldr_15, "@test_fnc_v1.blk0.cmp_res");
    id_154 = bldr_15->gen_sym(bldr_15, "@test_fnc_v1.blk0.res");
    id_155 = bldr_15->gen_sym(bldr_15, NULL);
    bldr_15->new_new(bldr_15, id_155, id_152, id_145, MU_NO_ID);
    id_156 = bldr_15->gen_sym(bldr_15, NULL);
    bldr_15->new_cmp(bldr_15, id_156, id_153, MU_CMP_EQ, id_146, id_152, id_147);
    id_157 = bldr_15->gen_sym(bldr_15, NULL);
    bldr_15->new_conv(bldr_15, id_157, id_154, MU_CONV_ZEXT, id_144, id_145, id_153);
    id_158 = bldr_15->gen_sym(bldr_15, NULL);
    bldr_15->new_ret(bldr_15, id_158, (MuVarNode [1]){id_154}, 1);
    bldr_15->new_bb(bldr_15, id_151, NULL, NULL, 0, MU_NO_ID, (MuInstNode [4]){id_155, id_156, id_157, id_158}, 4);
    bldr_15->new_func_ver(bldr_15, id_150, id_149, (MuBBNode [1]){id_151}, 1);
    bldr_15->load(bldr_15);
    mu_15->compile_to_sharedlib(mu_15, LIB_FILE_NAME("test_eq_ref"), NULL, 0);
    return 0;
}
