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
    MuVM* mu_29;
    MuCtx* ctx_29;
    MuIRBuilder* bldr_29;
    MuID id_369;
    MuID id_370;
    MuID id_371;
    MuID id_372;
    MuID id_373;
    MuID id_374;
    MuID id_375;
    MuID id_376;
    MuID id_377;
    MuID id_378;
    mu_29 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_29 = mu_29->new_context(mu_29);
    bldr_29 = ctx_29->new_ir_builder(ctx_29);
    id_369 = bldr_29->gen_sym(bldr_29, "@i32");
    bldr_29->new_type_int(bldr_29, id_369, 0x00000020ull);
    id_370 = bldr_29->gen_sym(bldr_29, "@i64");
    bldr_29->new_type_int(bldr_29, id_370, 0x00000040ull);
    id_371 = bldr_29->gen_sym(bldr_29, "@0xa8324b55_i32");
    bldr_29->new_const_int(bldr_29, id_371, id_369, 0x00000000a8324b55ull);
    id_372 = bldr_29->gen_sym(bldr_29, "@sig__i64");
    bldr_29->new_funcsig(bldr_29, id_372, NULL, 0, (MuTypeNode [1]){id_370}, 1);
    id_373 = bldr_29->gen_sym(bldr_29, "@test_fnc");
    bldr_29->new_func(bldr_29, id_373, id_372);
    id_374 = bldr_29->gen_sym(bldr_29, "@test_fnc_v1");
    id_375 = bldr_29->gen_sym(bldr_29, "@test_fnc_v1.blk0");
    id_376 = bldr_29->gen_sym(bldr_29, "@test_fnc_v1.blk0.res");
    id_377 = bldr_29->gen_sym(bldr_29, NULL);
    bldr_29->new_conv(bldr_29, id_377, id_376, MU_CONV_SEXT, id_369, id_370, id_371);
    id_378 = bldr_29->gen_sym(bldr_29, NULL);
    bldr_29->new_ret(bldr_29, id_378, (MuVarNode [1]){id_376}, 1);
    bldr_29->new_bb(bldr_29, id_375, NULL, NULL, 0, MU_NO_ID, (MuInstNode [2]){id_377, id_378}, 2);
    bldr_29->new_func_ver(bldr_29, id_374, id_373, (MuBBNode [1]){id_375}, 1);
    bldr_29->load(bldr_29);
    mu_29->compile_to_sharedlib(mu_29, LIB_FILE_NAME("test_sext"), NULL, 0);
    return 0;
}
