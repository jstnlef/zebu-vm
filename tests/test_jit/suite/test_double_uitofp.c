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
    MuVM* mu_44;
    MuCtx* ctx_44;
    MuIRBuilder* bldr_44;
    MuID id_546;
    MuID id_547;
    MuID id_548;
    MuID id_549;
    MuID id_550;
    MuID id_551;
    MuID id_552;
    MuID id_553;
    MuID id_554;
    MuID id_555;
    MuID id_556;
    mu_44 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_44 = mu_44->new_context(mu_44);
    bldr_44 = ctx_44->new_ir_builder(ctx_44);
    id_546 = bldr_44->gen_sym(bldr_44, "@dbl");
    bldr_44->new_type_double(bldr_44, id_546);
    id_547 = bldr_44->gen_sym(bldr_44, "@i1");
    bldr_44->new_type_int(bldr_44, id_547, 0x00000001ull);
    id_548 = bldr_44->gen_sym(bldr_44, "@i64");
    bldr_44->new_type_int(bldr_44, id_548, 0x00000040ull);
    id_549 = bldr_44->gen_sym(bldr_44, "@k");
    bldr_44->new_const_int(bldr_44, id_549, id_548, 0x000000000000002aull);
    id_550 = bldr_44->gen_sym(bldr_44, "@sig__dbl");
    bldr_44->new_funcsig(bldr_44, id_550, NULL, 0, (MuTypeNode [1]){id_546}, 1);
    id_551 = bldr_44->gen_sym(bldr_44, "@test_fnc");
    bldr_44->new_func(bldr_44, id_551, id_550);
    id_552 = bldr_44->gen_sym(bldr_44, "@test_fnc.v1");
    id_553 = bldr_44->gen_sym(bldr_44, "@test_fnc.v1.blk0");
    id_554 = bldr_44->gen_sym(bldr_44, "@test_fnc.v1.blk0.res");
    id_555 = bldr_44->gen_sym(bldr_44, NULL);
    bldr_44->new_conv(bldr_44, id_555, id_554, MU_CONV_UITOFP, id_548, id_546, id_549);
    id_556 = bldr_44->gen_sym(bldr_44, NULL);
    bldr_44->new_ret(bldr_44, id_556, (MuVarNode [1]){id_554}, 1);
    bldr_44->new_bb(bldr_44, id_553, NULL, NULL, 0, MU_NO_ID, (MuInstNode [2]){id_555, id_556}, 2);
    bldr_44->new_func_ver(bldr_44, id_552, id_551, (MuBBNode [1]){id_553}, 1);
    bldr_44->load(bldr_44);
    mu_44->compile_to_sharedlib(mu_44, LIB_FILE_NAME("test_double_uitofp"), NULL, 0);
    return 0;
}
