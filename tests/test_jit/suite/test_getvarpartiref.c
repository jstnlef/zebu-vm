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
    MuVM* mu_51;
    MuCtx* ctx_51;
    MuIRBuilder* bldr_51;
    MuID id_677;
    MuID id_678;
    MuID id_679;
    MuID id_680;
    MuID id_681;
    MuID id_682;
    MuID id_683;
    MuID id_684;
    MuID id_685;
    MuID id_686;
    MuID id_687;
    MuID id_688;
    MuID id_689;
    MuID id_690;
    MuID id_691;
    mu_51 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_51 = mu_51->new_context(mu_51);
    bldr_51 = ctx_51->new_ir_builder(ctx_51);
    id_677 = bldr_51->gen_sym(bldr_51, "@i8");
    bldr_51->new_type_int(bldr_51, id_677, 0x00000008ull);
    id_678 = bldr_51->gen_sym(bldr_51, "@i32");
    bldr_51->new_type_int(bldr_51, id_678, 0x00000020ull);
    id_679 = bldr_51->gen_sym(bldr_51, "@i64");
    bldr_51->new_type_int(bldr_51, id_679, 0x00000040ull);
    id_680 = bldr_51->gen_sym(bldr_51, "@hyb");
    bldr_51->new_type_hybrid(bldr_51, id_680, (MuTypeNode [2]){id_677, id_679}, 2, id_678);
    id_681 = bldr_51->gen_sym(bldr_51, "@phyb");
    bldr_51->new_type_uptr(bldr_51, id_681, id_680);
    id_682 = bldr_51->gen_sym(bldr_51, "@sig_phyb_i32");
    bldr_51->new_funcsig(bldr_51, id_682, (MuTypeNode [1]){id_681}, 1, (MuTypeNode [1]){id_678}, 1);
    id_683 = bldr_51->gen_sym(bldr_51, "@test_fnc");
    bldr_51->new_func(bldr_51, id_683, id_682);
    id_684 = bldr_51->gen_sym(bldr_51, "@test_fnc.v1");
    id_685 = bldr_51->gen_sym(bldr_51, "@test_fnc.v1.blk0");
    id_686 = bldr_51->gen_sym(bldr_51, "@test_fnc.v1.blk0.ps");
    id_687 = bldr_51->gen_sym(bldr_51, "@test_fnc.v1.blk0.pfld");
    id_688 = bldr_51->gen_sym(bldr_51, "@test_fnc.v1.blk0.res");
    id_689 = bldr_51->gen_sym(bldr_51, NULL);
    bldr_51->new_getvarpartiref(bldr_51, id_689, id_687, true, id_680, id_686);
    id_690 = bldr_51->gen_sym(bldr_51, NULL);
    bldr_51->new_load(bldr_51, id_690, id_688, true, MU_ORD_NOT_ATOMIC, id_678, id_687, MU_NO_ID);
    id_691 = bldr_51->gen_sym(bldr_51, NULL);
    bldr_51->new_ret(bldr_51, id_691, (MuVarNode [1]){id_688}, 1);
    bldr_51->new_bb(bldr_51, id_685, (MuID [1]){id_686}, (MuTypeNode [1]){id_681}, 1, MU_NO_ID, (MuInstNode [3]){id_689, id_690, id_691}, 3);
    bldr_51->new_func_ver(bldr_51, id_684, id_683, (MuBBNode [1]){id_685}, 1);
    bldr_51->load(bldr_51);
    mu_51->compile_to_sharedlib(mu_51, LIB_FILE_NAME("test_getvarpartiref"), NULL, 0);
    return 0;
}
