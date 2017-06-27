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
    MuVM* mu_41;
    MuCtx* ctx_41;
    MuIRBuilder* bldr_41;
    MuID id_514;
    MuID id_515;
    MuID id_516;
    MuID id_517;
    MuID id_518;
    MuID id_519;
    MuID id_520;
    MuID id_521;
    MuID id_522;
    MuID id_523;
    mu_41 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_41 = mu_41->new_context(mu_41);
    bldr_41 = ctx_41->new_ir_builder(ctx_41);
    id_514 = bldr_41->gen_sym(bldr_41, "@dbl");
    bldr_41->new_type_double(bldr_41, id_514);
    id_515 = bldr_41->gen_sym(bldr_41, "@sig_dbldbl_dbl");
    bldr_41->new_funcsig(bldr_41, id_515, (MuTypeNode [2]){id_514, id_514}, 2, (MuTypeNode [1]){id_514}, 1);
    id_516 = bldr_41->gen_sym(bldr_41, "@test_fnc");
    bldr_41->new_func(bldr_41, id_516, id_515);
    id_517 = bldr_41->gen_sym(bldr_41, "@test_fnc.v1");
    id_518 = bldr_41->gen_sym(bldr_41, "@test_fnc.v1.blk0");
    id_519 = bldr_41->gen_sym(bldr_41, "@test_fnc.v1.blk0.a");
    id_520 = bldr_41->gen_sym(bldr_41, "@test_fnc.v1.blk0.b");
    id_521 = bldr_41->gen_sym(bldr_41, "@test_fnc.v1.blk0.res");
    id_522 = bldr_41->gen_sym(bldr_41, NULL);
    bldr_41->new_binop(bldr_41, id_522, id_521, MU_BINOP_FADD, id_514, id_519, id_520, MU_NO_ID);
    id_523 = bldr_41->gen_sym(bldr_41, NULL);
    bldr_41->new_ret(bldr_41, id_523, (MuVarNode [1]){id_521}, 1);
    bldr_41->new_bb(bldr_41, id_518, (MuID [2]){id_519, id_520}, (MuTypeNode [2]){id_514, id_514}, 2, MU_NO_ID, (MuInstNode [2]){id_522, id_523}, 2);
    bldr_41->new_func_ver(bldr_41, id_517, id_516, (MuBBNode [1]){id_518}, 1);
    bldr_41->load(bldr_41);
    mu_41->compile_to_sharedlib(mu_41, LIB_FILE_NAME("test_double_arg_pass"), NULL, 0);
    return 0;
}
