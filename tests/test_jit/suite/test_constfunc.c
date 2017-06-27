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
    MuVM* mu_53;
    MuCtx* ctx_53;
    MuIRBuilder* bldr_53;
    MuID id_707;
    MuID id_708;
    MuID id_709;
    MuID id_710;
    MuID id_711;
    MuID id_712;
    MuID id_713;
    MuID id_714;
    mu_53 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_53 = mu_53->new_context(mu_53);
    bldr_53 = ctx_53->new_ir_builder(ctx_53);
    id_707 = bldr_53->gen_sym(bldr_53, "@i32");
    bldr_53->new_type_int(bldr_53, id_707, 0x00000008ull);
    id_708 = bldr_53->gen_sym(bldr_53, "@0_i32");
    bldr_53->new_const_int(bldr_53, id_708, id_707, 0x0000000000000000ull);
    id_709 = bldr_53->gen_sym(bldr_53, "@sig__i32");
    bldr_53->new_funcsig(bldr_53, id_709, NULL, 0, (MuTypeNode [1]){id_707}, 1);
    id_710 = bldr_53->gen_sym(bldr_53, "@test_fnc");
    bldr_53->new_func(bldr_53, id_710, id_709);
    id_711 = bldr_53->gen_sym(bldr_53, "@test_fnc_v1");
    id_712 = bldr_53->gen_sym(bldr_53, "@test_fnc_v1.blk0");
    id_713 = bldr_53->gen_sym(bldr_53, "@test_fnc_v1.blk0.res");
    id_714 = bldr_53->gen_sym(bldr_53, NULL);
    bldr_53->new_ret(bldr_53, id_714, (MuVarNode [1]){id_708}, 1);
    bldr_53->new_bb(bldr_53, id_712, NULL, NULL, 0, MU_NO_ID, (MuInstNode [1]){id_714}, 1);
    bldr_53->new_func_ver(bldr_53, id_711, id_710, (MuBBNode [1]){id_712}, 1);
    bldr_53->load(bldr_53);
    mu_53->compile_to_sharedlib(mu_53, LIB_FILE_NAME("test_constfunc"), NULL, 0);
    return 0;
}
