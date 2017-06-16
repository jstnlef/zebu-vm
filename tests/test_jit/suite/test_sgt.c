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
    MuVM* mu_19;
    MuCtx* ctx_19;
    MuIRBuilder* bldr_19;
    MuID id_204;
    MuID id_205;
    MuID id_206;
    MuID id_207;
    MuID id_208;
    MuID id_209;
    MuID id_210;
    MuID id_211;
    MuID id_212;
    MuID id_213;
    MuID id_214;
    MuID id_215;
    MuID id_216;
    MuID id_217;
    MuID id_218;
    MuID id_219;
    MuID id_220;
    mu_19 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_19 = mu_19->new_context(mu_19);
    bldr_19 = ctx_19->new_ir_builder(ctx_19);
    id_204 = bldr_19->gen_sym(bldr_19, "@i1");
    bldr_19->new_type_int(bldr_19, id_204, 0x00000001ull);
    id_205 = bldr_19->gen_sym(bldr_19, "@i8");
    bldr_19->new_type_int(bldr_19, id_205, 0x00000008ull);
    id_206 = bldr_19->gen_sym(bldr_19, "@0xff_i8");
    bldr_19->new_const_int(bldr_19, id_206, id_205, 0x00000000000000ffull);
    id_207 = bldr_19->gen_sym(bldr_19, "@0x0a_i8");
    bldr_19->new_const_int(bldr_19, id_207, id_205, 0x000000000000000aull);
    id_208 = bldr_19->gen_sym(bldr_19, "@sig__i8");
    bldr_19->new_funcsig(bldr_19, id_208, NULL, 0, (MuTypeNode [1]){id_205}, 1);
    id_209 = bldr_19->gen_sym(bldr_19, "@test_fnc");
    bldr_19->new_func(bldr_19, id_209, id_208);
    id_210 = bldr_19->gen_sym(bldr_19, "@test_fnc_v1");
    id_211 = bldr_19->gen_sym(bldr_19, "@test_fnc_v1.blk0");
    id_212 = bldr_19->gen_sym(bldr_19, "@test_fnc_v1.blk0.cmp_res_1");
    id_213 = bldr_19->gen_sym(bldr_19, "@test_fnc_v1.blk0.cmp_res_2");
    id_214 = bldr_19->gen_sym(bldr_19, "@test_fnc_v1.blk0.bin_res");
    id_215 = bldr_19->gen_sym(bldr_19, "@test_fnc_v1.blk0.res");
    id_216 = bldr_19->gen_sym(bldr_19, NULL);
    bldr_19->new_cmp(bldr_19, id_216, id_212, MU_CMP_SGT, id_205, id_206, id_207);
    id_217 = bldr_19->gen_sym(bldr_19, NULL);
    bldr_19->new_cmp(bldr_19, id_217, id_213, MU_CMP_SGT, id_205, id_206, id_206);
    id_218 = bldr_19->gen_sym(bldr_19, NULL);
    bldr_19->new_binop(bldr_19, id_218, id_214, MU_BINOP_OR, id_204, id_212, id_213, MU_NO_ID);
    id_219 = bldr_19->gen_sym(bldr_19, NULL);
    bldr_19->new_conv(bldr_19, id_219, id_215, MU_CONV_ZEXT, id_204, id_205, id_214);
    id_220 = bldr_19->gen_sym(bldr_19, NULL);
    bldr_19->new_ret(bldr_19, id_220, (MuVarNode [1]){id_215}, 1);
    bldr_19->new_bb(bldr_19, id_211, NULL, NULL, 0, MU_NO_ID, (MuInstNode [5]){id_216, id_217, id_218, id_219, id_220}, 5);
    bldr_19->new_func_ver(bldr_19, id_210, id_209, (MuBBNode [1]){id_211}, 1);
    bldr_19->load(bldr_19);
    mu_19->compile_to_sharedlib(mu_19, LIB_FILE_NAME("test_sgt"), NULL, 0);
    return 0;
}
