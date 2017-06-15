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
    MuVM* mu_50;
    MuCtx* ctx_50;
    MuIRBuilder* bldr_50;
    MuID id_663;
    MuID id_664;
    MuID id_665;
    MuID id_666;
    MuID id_667;
    MuID id_668;
    MuID id_669;
    MuID id_670;
    MuID id_671;
    MuID id_672;
    MuID id_673;
    MuID id_674;
    MuID id_675;
    MuID id_676;
    mu_50 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_50 = mu_50->new_context(mu_50);
    bldr_50 = ctx_50->new_ir_builder(ctx_50);
    id_663 = bldr_50->gen_sym(bldr_50, "@i64");
    bldr_50->new_type_int(bldr_50, id_663, 0x00000040ull);
    id_664 = bldr_50->gen_sym(bldr_50, "@arr");
    bldr_50->new_type_array(bldr_50, id_664, id_663, 0x0000000000000005ull);
    id_665 = bldr_50->gen_sym(bldr_50, "@parr");
    bldr_50->new_type_uptr(bldr_50, id_665, id_664);
    id_666 = bldr_50->gen_sym(bldr_50, "@sig_parri64_i64");
    bldr_50->new_funcsig(bldr_50, id_666, (MuTypeNode [2]){id_665, id_663}, 2, (MuTypeNode [1]){id_663}, 1);
    id_667 = bldr_50->gen_sym(bldr_50, "@test_fnc");
    bldr_50->new_func(bldr_50, id_667, id_666);
    id_668 = bldr_50->gen_sym(bldr_50, "@test_fnc.v1");
    id_669 = bldr_50->gen_sym(bldr_50, "@test_fnc.v1.blk0");
    id_670 = bldr_50->gen_sym(bldr_50, "@test_fnc.v1.blk0.pa");
    id_671 = bldr_50->gen_sym(bldr_50, "@test_fnc.v1.blk0.idx");
    id_672 = bldr_50->gen_sym(bldr_50, "@test_fnc.v1.blk0.pelm");
    id_673 = bldr_50->gen_sym(bldr_50, "@test_fnc.v1.blk0.res");
    id_674 = bldr_50->gen_sym(bldr_50, NULL);
    bldr_50->new_getelemiref(bldr_50, id_674, id_672, true, id_664, id_663, id_670, id_671);
    id_675 = bldr_50->gen_sym(bldr_50, NULL);
    bldr_50->new_load(bldr_50, id_675, id_673, true, MU_ORD_NOT_ATOMIC, id_663, id_672, MU_NO_ID);
    id_676 = bldr_50->gen_sym(bldr_50, NULL);
    bldr_50->new_ret(bldr_50, id_676, (MuVarNode [1]){id_673}, 1);
    bldr_50->new_bb(bldr_50, id_669, (MuID [2]){id_670, id_671}, (MuTypeNode [2]){id_665, id_663}, 2, MU_NO_ID, (MuInstNode [3]){id_674, id_675, id_676}, 3);
    bldr_50->new_func_ver(bldr_50, id_668, id_667, (MuBBNode [1]){id_669}, 1);
    bldr_50->load(bldr_50);
    mu_50->compile_to_sharedlib(mu_50, LIB_FILE_NAME("test_getelemiref"), NULL, 0);
    return 0;
}
