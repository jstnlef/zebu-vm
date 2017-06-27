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
    MuVM* mu_37;
    MuCtx* ctx_37;
    MuIRBuilder* bldr_37;
    MuID id_460;
    MuID id_461;
    MuID id_462;
    MuID id_463;
    MuID id_464;
    MuID id_465;
    MuID id_466;
    MuID id_467;
    MuID id_468;
    MuID id_469;
    MuID id_470;
    MuID id_471;
    MuID id_472;
    MuID id_473;
    mu_37 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_37 = mu_37->new_context(mu_37);
    bldr_37 = ctx_37->new_ir_builder(ctx_37);
    id_460 = bldr_37->gen_sym(bldr_37, "@dbl");
    bldr_37->new_type_double(bldr_37, id_460);
    id_461 = bldr_37->gen_sym(bldr_37, "@i1");
    bldr_37->new_type_int(bldr_37, id_461, 0x00000001ull);
    id_462 = bldr_37->gen_sym(bldr_37, "@i64");
    bldr_37->new_type_int(bldr_37, id_462, 0x00000040ull);
    id_463 = bldr_37->gen_sym(bldr_37, "@pi");
    bldr_37->new_const_double(bldr_37, id_463, id_460, *(double*)(uint64_t [1]){0x400921fb82c2bd7full});
    id_464 = bldr_37->gen_sym(bldr_37, "@e");
    bldr_37->new_const_double(bldr_37, id_464, id_460, *(double*)(uint64_t [1]){0x4005bf0995aaf790ull});
    id_465 = bldr_37->gen_sym(bldr_37, "@sig__i64");
    bldr_37->new_funcsig(bldr_37, id_465, NULL, 0, (MuTypeNode [1]){id_462}, 1);
    id_466 = bldr_37->gen_sym(bldr_37, "@test_fnc");
    bldr_37->new_func(bldr_37, id_466, id_465);
    id_467 = bldr_37->gen_sym(bldr_37, "@test_fnc.v1");
    id_468 = bldr_37->gen_sym(bldr_37, "@test_fnc.v1.blk0");
    id_469 = bldr_37->gen_sym(bldr_37, "@test_fnc.v1.blk0.cmpres");
    id_470 = bldr_37->gen_sym(bldr_37, "@test_fnc.v1.blk0.res");
    id_471 = bldr_37->gen_sym(bldr_37, NULL);
    bldr_37->new_cmp(bldr_37, id_471, id_469, MU_CMP_FOLT, id_460, id_464, id_463);
    id_472 = bldr_37->gen_sym(bldr_37, NULL);
    bldr_37->new_conv(bldr_37, id_472, id_470, MU_CONV_ZEXT, id_461, id_462, id_469);
    id_473 = bldr_37->gen_sym(bldr_37, NULL);
    bldr_37->new_ret(bldr_37, id_473, (MuVarNode [1]){id_470}, 1);
    bldr_37->new_bb(bldr_37, id_468, NULL, NULL, 0, MU_NO_ID, (MuInstNode [3]){id_471, id_472, id_473}, 3);
    bldr_37->new_func_ver(bldr_37, id_467, id_466, (MuBBNode [1]){id_468}, 1);
    bldr_37->load(bldr_37);
    mu_37->compile_to_sharedlib(mu_37, LIB_FILE_NAME("test_double_ordered_lt"), NULL, 0);
    return 0;
}
