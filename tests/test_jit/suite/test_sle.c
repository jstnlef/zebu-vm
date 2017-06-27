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
    MuVM* mu_20;
    MuCtx* ctx_20;
    MuIRBuilder* bldr_20;
    MuID id_221;
    MuID id_222;
    MuID id_223;
    MuID id_224;
    MuID id_225;
    MuID id_226;
    MuID id_227;
    MuID id_228;
    MuID id_229;
    MuID id_230;
    MuID id_231;
    MuID id_232;
    MuID id_233;
    MuID id_234;
    MuID id_235;
    MuID id_236;
    MuID id_237;
    mu_20 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_20 = mu_20->new_context(mu_20);
    bldr_20 = ctx_20->new_ir_builder(ctx_20);
    id_221 = bldr_20->gen_sym(bldr_20, "@i1");
    bldr_20->new_type_int(bldr_20, id_221, 0x00000001ull);
    id_222 = bldr_20->gen_sym(bldr_20, "@i8");
    bldr_20->new_type_int(bldr_20, id_222, 0x00000008ull);
    id_223 = bldr_20->gen_sym(bldr_20, "@0xff_i8");
    bldr_20->new_const_int(bldr_20, id_223, id_222, 0x00000000000000ffull);
    id_224 = bldr_20->gen_sym(bldr_20, "@0x0a_i8");
    bldr_20->new_const_int(bldr_20, id_224, id_222, 0x000000000000000aull);
    id_225 = bldr_20->gen_sym(bldr_20, "@sig__i8");
    bldr_20->new_funcsig(bldr_20, id_225, NULL, 0, (MuTypeNode [1]){id_222}, 1);
    id_226 = bldr_20->gen_sym(bldr_20, "@test_fnc");
    bldr_20->new_func(bldr_20, id_226, id_225);
    id_227 = bldr_20->gen_sym(bldr_20, "@test_fnc_v1");
    id_228 = bldr_20->gen_sym(bldr_20, "@test_fnc_v1.blk0");
    id_229 = bldr_20->gen_sym(bldr_20, "@test_fnc_v1.blk0.cmp_res_1");
    id_230 = bldr_20->gen_sym(bldr_20, "@test_fnc_v1.blk0.cmp_res_2");
    id_231 = bldr_20->gen_sym(bldr_20, "@test_fnc_v1.blk0.bin_res");
    id_232 = bldr_20->gen_sym(bldr_20, "@test_fnc_v1.blk0.res");
    id_233 = bldr_20->gen_sym(bldr_20, NULL);
    bldr_20->new_cmp(bldr_20, id_233, id_229, MU_CMP_SLE, id_222, id_224, id_223);
    id_234 = bldr_20->gen_sym(bldr_20, NULL);
    bldr_20->new_cmp(bldr_20, id_234, id_230, MU_CMP_SLE, id_222, id_223, id_223);
    id_235 = bldr_20->gen_sym(bldr_20, NULL);
    bldr_20->new_binop(bldr_20, id_235, id_231, MU_BINOP_XOR, id_221, id_229, id_230, MU_NO_ID);
    id_236 = bldr_20->gen_sym(bldr_20, NULL);
    bldr_20->new_conv(bldr_20, id_236, id_232, MU_CONV_ZEXT, id_221, id_222, id_231);
    id_237 = bldr_20->gen_sym(bldr_20, NULL);
    bldr_20->new_ret(bldr_20, id_237, (MuVarNode [1]){id_232}, 1);
    bldr_20->new_bb(bldr_20, id_228, NULL, NULL, 0, MU_NO_ID, (MuInstNode [5]){id_233, id_234, id_235, id_236, id_237}, 5);
    bldr_20->new_func_ver(bldr_20, id_227, id_226, (MuBBNode [1]){id_228}, 1);
    bldr_20->load(bldr_20);
    mu_20->compile_to_sharedlib(mu_20, LIB_FILE_NAME("test_sle"), NULL, 0);
    return 0;
}
