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
    MuVM* mu_18;
    MuCtx* ctx_18;
    MuIRBuilder* bldr_18;
    MuID id_187;
    MuID id_188;
    MuID id_189;
    MuID id_190;
    MuID id_191;
    MuID id_192;
    MuID id_193;
    MuID id_194;
    MuID id_195;
    MuID id_196;
    MuID id_197;
    MuID id_198;
    MuID id_199;
    MuID id_200;
    MuID id_201;
    MuID id_202;
    MuID id_203;
    mu_18 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_18 = mu_18->new_context(mu_18);
    bldr_18 = ctx_18->new_ir_builder(ctx_18);
    id_187 = bldr_18->gen_sym(bldr_18, "@i1");
    bldr_18->new_type_int(bldr_18, id_187, 0x00000001ull);
    id_188 = bldr_18->gen_sym(bldr_18, "@i8");
    bldr_18->new_type_int(bldr_18, id_188, 0x00000008ull);
    id_189 = bldr_18->gen_sym(bldr_18, "@0xff_i8");
    bldr_18->new_const_int(bldr_18, id_189, id_188, 0x00000000000000ffull);
    id_190 = bldr_18->gen_sym(bldr_18, "@0x0a_i8");
    bldr_18->new_const_int(bldr_18, id_190, id_188, 0x000000000000000aull);
    id_191 = bldr_18->gen_sym(bldr_18, "@sig__i8");
    bldr_18->new_funcsig(bldr_18, id_191, NULL, 0, (MuTypeNode [1]){id_188}, 1);
    id_192 = bldr_18->gen_sym(bldr_18, "@test_fnc");
    bldr_18->new_func(bldr_18, id_192, id_191);
    id_193 = bldr_18->gen_sym(bldr_18, "@test_fnc_v1");
    id_194 = bldr_18->gen_sym(bldr_18, "@test_fnc_v1.blk0");
    id_195 = bldr_18->gen_sym(bldr_18, "@test_fnc_v1.blk0.cmp_res_1");
    id_196 = bldr_18->gen_sym(bldr_18, "@test_fnc_v1.blk0.cmp_res_2");
    id_197 = bldr_18->gen_sym(bldr_18, "@test_fnc_v1.blk0.bin_res");
    id_198 = bldr_18->gen_sym(bldr_18, "@test_fnc_v1.blk0.res");
    id_199 = bldr_18->gen_sym(bldr_18, NULL);
    bldr_18->new_cmp(bldr_18, id_199, id_195, MU_CMP_SGE, id_188, id_189, id_190);
    id_200 = bldr_18->gen_sym(bldr_18, NULL);
    bldr_18->new_cmp(bldr_18, id_200, id_196, MU_CMP_SGE, id_188, id_189, id_189);
    id_201 = bldr_18->gen_sym(bldr_18, NULL);
    bldr_18->new_binop(bldr_18, id_201, id_197, MU_BINOP_XOR, id_187, id_195, id_196, MU_NO_ID);
    id_202 = bldr_18->gen_sym(bldr_18, NULL);
    bldr_18->new_conv(bldr_18, id_202, id_198, MU_CONV_ZEXT, id_187, id_188, id_197);
    id_203 = bldr_18->gen_sym(bldr_18, NULL);
    bldr_18->new_ret(bldr_18, id_203, (MuVarNode [1]){id_198}, 1);
    bldr_18->new_bb(bldr_18, id_194, NULL, NULL, 0, MU_NO_ID, (MuInstNode [5]){id_199, id_200, id_201, id_202, id_203}, 5);
    bldr_18->new_func_ver(bldr_18, id_193, id_192, (MuBBNode [1]){id_194}, 1);
    bldr_18->load(bldr_18);
    mu_18->compile_to_sharedlib(mu_18, LIB_FILE_NAME("test_sge"), NULL, 0);
    return 0;
}
