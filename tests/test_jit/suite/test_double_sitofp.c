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
    MuVM* mu_42;
    MuCtx* ctx_42;
    MuIRBuilder* bldr_42;
    MuID id_524;
    MuID id_525;
    MuID id_526;
    MuID id_527;
    MuID id_528;
    MuID id_529;
    MuID id_530;
    MuID id_531;
    MuID id_532;
    MuID id_533;
    MuID id_534;
    mu_42 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_42 = mu_42->new_context(mu_42);
    bldr_42 = ctx_42->new_ir_builder(ctx_42);
    id_524 = bldr_42->gen_sym(bldr_42, "@dbl");
    bldr_42->new_type_double(bldr_42, id_524);
    id_525 = bldr_42->gen_sym(bldr_42, "@i1");
    bldr_42->new_type_int(bldr_42, id_525, 0x00000001ull);
    id_526 = bldr_42->gen_sym(bldr_42, "@i64");
    bldr_42->new_type_int(bldr_42, id_526, 0x00000040ull);
    id_527 = bldr_42->gen_sym(bldr_42, "@k");
    bldr_42->new_const_int(bldr_42, id_527, id_526, 0xffffffffffffffd6ull);
    id_528 = bldr_42->gen_sym(bldr_42, "@sig__dbl");
    bldr_42->new_funcsig(bldr_42, id_528, NULL, 0, (MuTypeNode [1]){id_524}, 1);
    id_529 = bldr_42->gen_sym(bldr_42, "@test_fnc");
    bldr_42->new_func(bldr_42, id_529, id_528);
    id_530 = bldr_42->gen_sym(bldr_42, "@test_fnc.v1");
    id_531 = bldr_42->gen_sym(bldr_42, "@test_fnc.v1.blk0");
    id_532 = bldr_42->gen_sym(bldr_42, "@test_fnc.v1.blk0.res");
    id_533 = bldr_42->gen_sym(bldr_42, NULL);
    bldr_42->new_conv(bldr_42, id_533, id_532, MU_CONV_SITOFP, id_526, id_524, id_527);
    id_534 = bldr_42->gen_sym(bldr_42, NULL);
    bldr_42->new_ret(bldr_42, id_534, (MuVarNode [1]){id_532}, 1);
    bldr_42->new_bb(bldr_42, id_531, NULL, NULL, 0, MU_NO_ID, (MuInstNode [2]){id_533, id_534}, 2);
    bldr_42->new_func_ver(bldr_42, id_530, id_529, (MuBBNode [1]){id_531}, 1);
    bldr_42->load(bldr_42);
    mu_42->compile_to_sharedlib(mu_42, LIB_FILE_NAME("test_double_sitofp"), NULL, 0);
    return 0;
}
