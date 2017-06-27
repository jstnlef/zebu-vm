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
    MuVM* mu_49;
    MuCtx* ctx_49;
    MuIRBuilder* bldr_49;
    MuID id_648;
    MuID id_649;
    MuID id_650;
    MuID id_651;
    MuID id_652;
    MuID id_653;
    MuID id_654;
    MuID id_655;
    MuID id_656;
    MuID id_657;
    MuID id_658;
    MuID id_659;
    MuID id_660;
    MuID id_661;
    MuID id_662;
    mu_49 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_49 = mu_49->new_context(mu_49);
    bldr_49 = ctx_49->new_ir_builder(ctx_49);
    id_648 = bldr_49->gen_sym(bldr_49, "@i8");
    bldr_49->new_type_int(bldr_49, id_648, 0x00000008ull);
    id_649 = bldr_49->gen_sym(bldr_49, "@i32");
    bldr_49->new_type_int(bldr_49, id_649, 0x00000020ull);
    id_650 = bldr_49->gen_sym(bldr_49, "@i64");
    bldr_49->new_type_int(bldr_49, id_650, 0x00000040ull);
    id_651 = bldr_49->gen_sym(bldr_49, "@stt");
    bldr_49->new_type_struct(bldr_49, id_651, (MuTypeNode [3]){id_648, id_650, id_649}, 3);
    id_652 = bldr_49->gen_sym(bldr_49, "@pstt");
    bldr_49->new_type_uptr(bldr_49, id_652, id_651);
    id_653 = bldr_49->gen_sym(bldr_49, "@sig_pstt_i32");
    bldr_49->new_funcsig(bldr_49, id_653, (MuTypeNode [1]){id_652}, 1, (MuTypeNode [1]){id_649}, 1);
    id_654 = bldr_49->gen_sym(bldr_49, "@test_fnc");
    bldr_49->new_func(bldr_49, id_654, id_653);
    id_655 = bldr_49->gen_sym(bldr_49, "@test_fnc.v1");
    id_656 = bldr_49->gen_sym(bldr_49, "@test_fnc.v1.blk0");
    id_657 = bldr_49->gen_sym(bldr_49, "@test_fnc.v1.blk0.ps");
    id_658 = bldr_49->gen_sym(bldr_49, "@test_fnc.v1.blk0.pfld");
    id_659 = bldr_49->gen_sym(bldr_49, "@test_fnc.v1.blk0.res");
    id_660 = bldr_49->gen_sym(bldr_49, NULL);
    bldr_49->new_getfieldiref(bldr_49, id_660, id_658, true, id_651, 0x00000002ull, id_657);
    id_661 = bldr_49->gen_sym(bldr_49, NULL);
    bldr_49->new_load(bldr_49, id_661, id_659, true, MU_ORD_NOT_ATOMIC, id_649, id_658, MU_NO_ID);
    id_662 = bldr_49->gen_sym(bldr_49, NULL);
    bldr_49->new_ret(bldr_49, id_662, (MuVarNode [1]){id_659}, 1);
    bldr_49->new_bb(bldr_49, id_656, (MuID [1]){id_657}, (MuTypeNode [1]){id_652}, 1, MU_NO_ID, (MuInstNode [3]){id_660, id_661, id_662}, 3);
    bldr_49->new_func_ver(bldr_49, id_655, id_654, (MuBBNode [1]){id_656}, 1);
    bldr_49->load(bldr_49);
    mu_49->compile_to_sharedlib(mu_49, LIB_FILE_NAME("test_getfieldiref"), NULL, 0);
    return 0;
}
