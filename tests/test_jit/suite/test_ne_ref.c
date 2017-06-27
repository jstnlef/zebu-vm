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
    MuVM* mu_17;
    MuCtx* ctx_17;
    MuIRBuilder* bldr_17;
    MuID id_172;
    MuID id_173;
    MuID id_174;
    MuID id_175;
    MuID id_176;
    MuID id_177;
    MuID id_178;
    MuID id_179;
    MuID id_180;
    MuID id_181;
    MuID id_182;
    MuID id_183;
    MuID id_184;
    MuID id_185;
    MuID id_186;
    mu_17 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_17 = mu_17->new_context(mu_17);
    bldr_17 = ctx_17->new_ir_builder(ctx_17);
    id_172 = bldr_17->gen_sym(bldr_17, "@i1");
    bldr_17->new_type_int(bldr_17, id_172, 0x00000001ull);
    id_173 = bldr_17->gen_sym(bldr_17, "@i64");
    bldr_17->new_type_int(bldr_17, id_173, 0x00000040ull);
    id_174 = bldr_17->gen_sym(bldr_17, "@refi64");
    bldr_17->new_type_ref(bldr_17, id_174, id_173);
    id_175 = bldr_17->gen_sym(bldr_17, "@NULL_refi64");
    bldr_17->new_const_null(bldr_17, id_175, id_174);
    id_176 = bldr_17->gen_sym(bldr_17, "@sig__i64");
    bldr_17->new_funcsig(bldr_17, id_176, NULL, 0, (MuTypeNode [1]){id_173}, 1);
    id_177 = bldr_17->gen_sym(bldr_17, "@test_fnc");
    bldr_17->new_func(bldr_17, id_177, id_176);
    id_178 = bldr_17->gen_sym(bldr_17, "@test_fnc_v1");
    id_179 = bldr_17->gen_sym(bldr_17, "@test_fnc_v1.blk0");
    id_180 = bldr_17->gen_sym(bldr_17, "@test_fnc_v1.blk0.r");
    id_181 = bldr_17->gen_sym(bldr_17, "@test_fnc_v1.blk0.cmp_res");
    id_182 = bldr_17->gen_sym(bldr_17, "@test_fnc_v1.blk0.res");
    id_183 = bldr_17->gen_sym(bldr_17, NULL);
    bldr_17->new_new(bldr_17, id_183, id_180, id_173, MU_NO_ID);
    id_184 = bldr_17->gen_sym(bldr_17, NULL);
    bldr_17->new_cmp(bldr_17, id_184, id_181, MU_CMP_NE, id_174, id_180, id_175);
    id_185 = bldr_17->gen_sym(bldr_17, NULL);
    bldr_17->new_conv(bldr_17, id_185, id_182, MU_CONV_ZEXT, id_172, id_173, id_181);
    id_186 = bldr_17->gen_sym(bldr_17, NULL);
    bldr_17->new_ret(bldr_17, id_186, (MuVarNode [1]){id_182}, 1);
    bldr_17->new_bb(bldr_17, id_179, NULL, NULL, 0, MU_NO_ID, (MuInstNode [4]){id_183, id_184, id_185, id_186}, 4);
    bldr_17->new_func_ver(bldr_17, id_178, id_177, (MuBBNode [1]){id_179}, 1);
    bldr_17->load(bldr_17);
    mu_17->compile_to_sharedlib(mu_17, LIB_FILE_NAME("test_ne_ref"), NULL, 0);
    return 0;
}
