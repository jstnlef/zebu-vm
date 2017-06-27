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
    MuVM* mu_5;
    MuCtx* ctx_5;
    MuIRBuilder* bldr_5;
    MuID id_41;
    MuID id_42;
    MuID id_43;
    MuID id_44;
    MuID id_45;
    MuID id_46;
    MuID id_47;
    MuID id_48;
    MuID id_49;
    MuID id_50;
    mu_5 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_5 = mu_5->new_context(mu_5);
    bldr_5 = ctx_5->new_ir_builder(ctx_5);
    id_41 = bldr_5->gen_sym(bldr_5, "@i8");
    bldr_5->new_type_int(bldr_5, id_41, 0x00000008ull);
    id_42 = bldr_5->gen_sym(bldr_5, "@0x80_i8");
    bldr_5->new_const_int(bldr_5, id_42, id_41, 0x0000000000000080ull);
    id_43 = bldr_5->gen_sym(bldr_5, "@0x0a_i8");
    bldr_5->new_const_int(bldr_5, id_43, id_41, 0x000000000000000aull);
    id_44 = bldr_5->gen_sym(bldr_5, "@sig__i8");
    bldr_5->new_funcsig(bldr_5, id_44, NULL, 0, (MuTypeNode [1]){id_41}, 1);
    id_45 = bldr_5->gen_sym(bldr_5, "@test_fnc");
    bldr_5->new_func(bldr_5, id_45, id_44);
    id_46 = bldr_5->gen_sym(bldr_5, "@test_fnc_v1");
    id_47 = bldr_5->gen_sym(bldr_5, "@test_fnc_v1.blk0");
    id_48 = bldr_5->gen_sym(bldr_5, "@test_fnc_v1.blk0.res");
    id_49 = bldr_5->gen_sym(bldr_5, NULL);
    bldr_5->new_binop(bldr_5, id_49, id_48, MU_BINOP_SDIV, id_41, id_42, id_43, MU_NO_ID);
    id_50 = bldr_5->gen_sym(bldr_5, NULL);
    bldr_5->new_ret(bldr_5, id_50, (MuVarNode [1]){id_48}, 1);
    bldr_5->new_bb(bldr_5, id_47, NULL, NULL, 0, MU_NO_ID, (MuInstNode [2]){id_49, id_50}, 2);
    bldr_5->new_func_ver(bldr_5, id_46, id_45, (MuBBNode [1]){id_47}, 1);
    bldr_5->load(bldr_5);
    mu_5->compile_to_sharedlib(mu_5, LIB_FILE_NAME("test_sdiv"), NULL, 0);
    return 0;
}
