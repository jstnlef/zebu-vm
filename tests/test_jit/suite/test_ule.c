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
    MuVM* mu_22;
    MuCtx* ctx_22;
    MuIRBuilder* bldr_22;
    MuID id_255;
    MuID id_256;
    MuID id_257;
    MuID id_258;
    MuID id_259;
    MuID id_260;
    MuID id_261;
    MuID id_262;
    MuID id_263;
    MuID id_264;
    MuID id_265;
    MuID id_266;
    MuID id_267;
    MuID id_268;
    MuID id_269;
    MuID id_270;
    MuID id_271;
    mu_22 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_22 = mu_22->new_context(mu_22);
    bldr_22 = ctx_22->new_ir_builder(ctx_22);
    id_255 = bldr_22->gen_sym(bldr_22, "@i1");
    bldr_22->new_type_int(bldr_22, id_255, 0x00000001ull);
    id_256 = bldr_22->gen_sym(bldr_22, "@i8");
    bldr_22->new_type_int(bldr_22, id_256, 0x00000008ull);
    id_257 = bldr_22->gen_sym(bldr_22, "@0xff_i8");
    bldr_22->new_const_int(bldr_22, id_257, id_256, 0x00000000000000ffull);
    id_258 = bldr_22->gen_sym(bldr_22, "@0x0a_i8");
    bldr_22->new_const_int(bldr_22, id_258, id_256, 0x000000000000000aull);
    id_259 = bldr_22->gen_sym(bldr_22, "@sig__i8");
    bldr_22->new_funcsig(bldr_22, id_259, NULL, 0, (MuTypeNode [1]){id_256}, 1);
    id_260 = bldr_22->gen_sym(bldr_22, "@test_fnc");
    bldr_22->new_func(bldr_22, id_260, id_259);
    id_261 = bldr_22->gen_sym(bldr_22, "@test_fnc_v1");
    id_262 = bldr_22->gen_sym(bldr_22, "@test_fnc_v1.blk0");
    id_263 = bldr_22->gen_sym(bldr_22, "@test_fnc_v1.blk0.cmp_res_1");
    id_264 = bldr_22->gen_sym(bldr_22, "@test_fnc_v1.blk0.cmp_res_2");
    id_265 = bldr_22->gen_sym(bldr_22, "@test_fnc_v1.blk0.bin_res");
    id_266 = bldr_22->gen_sym(bldr_22, "@test_fnc_v1.blk0.res");
    id_267 = bldr_22->gen_sym(bldr_22, NULL);
    bldr_22->new_cmp(bldr_22, id_267, id_263, MU_CMP_ULE, id_256, id_258, id_257);
    id_268 = bldr_22->gen_sym(bldr_22, NULL);
    bldr_22->new_cmp(bldr_22, id_268, id_264, MU_CMP_ULE, id_256, id_257, id_257);
    id_269 = bldr_22->gen_sym(bldr_22, NULL);
    bldr_22->new_binop(bldr_22, id_269, id_265, MU_BINOP_AND, id_255, id_263, id_264, MU_NO_ID);
    id_270 = bldr_22->gen_sym(bldr_22, NULL);
    bldr_22->new_conv(bldr_22, id_270, id_266, MU_CONV_ZEXT, id_255, id_256, id_265);
    id_271 = bldr_22->gen_sym(bldr_22, NULL);
    bldr_22->new_ret(bldr_22, id_271, (MuVarNode [1]){id_266}, 1);
    bldr_22->new_bb(bldr_22, id_262, NULL, NULL, 0, MU_NO_ID, (MuInstNode [5]){id_267, id_268, id_269, id_270, id_271}, 5);
    bldr_22->new_func_ver(bldr_22, id_261, id_260, (MuBBNode [1]){id_262}, 1);
    bldr_22->load(bldr_22);
    mu_22->compile_to_sharedlib(mu_22, LIB_FILE_NAME("test_ule"), NULL, 0);
    return 0;
}
