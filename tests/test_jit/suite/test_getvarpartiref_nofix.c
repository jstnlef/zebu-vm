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
    MuVM* mu_52;
    MuCtx* ctx_52;
    MuIRBuilder* bldr_52;
    MuID id_692;
    MuID id_693;
    MuID id_694;
    MuID id_695;
    MuID id_696;
    MuID id_697;
    MuID id_698;
    MuID id_699;
    MuID id_700;
    MuID id_701;
    MuID id_702;
    MuID id_703;
    MuID id_704;
    MuID id_705;
    MuID id_706;
    mu_52 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_52 = mu_52->new_context(mu_52);
    bldr_52 = ctx_52->new_ir_builder(ctx_52);
    id_692 = bldr_52->gen_sym(bldr_52, "@i8");
    bldr_52->new_type_int(bldr_52, id_692, 0x00000008ull);
    id_693 = bldr_52->gen_sym(bldr_52, "@i32");
    bldr_52->new_type_int(bldr_52, id_693, 0x00000020ull);
    id_694 = bldr_52->gen_sym(bldr_52, "@i64");
    bldr_52->new_type_int(bldr_52, id_694, 0x00000040ull);
    id_695 = bldr_52->gen_sym(bldr_52, "@hyb");
    bldr_52->new_type_hybrid(bldr_52, id_695, NULL, 0, id_693);
    id_696 = bldr_52->gen_sym(bldr_52, "@phyb");
    bldr_52->new_type_uptr(bldr_52, id_696, id_695);
    id_697 = bldr_52->gen_sym(bldr_52, "@sig_phyb_i32");
    bldr_52->new_funcsig(bldr_52, id_697, (MuTypeNode [1]){id_696}, 1, (MuTypeNode [1]){id_693}, 1);
    id_698 = bldr_52->gen_sym(bldr_52, "@test_fnc");
    bldr_52->new_func(bldr_52, id_698, id_697);
    id_699 = bldr_52->gen_sym(bldr_52, "@test_fnc.v1");
    id_700 = bldr_52->gen_sym(bldr_52, "@test_fnc.v1.blk0");
    id_701 = bldr_52->gen_sym(bldr_52, "@test_fnc.v1.blk0.ps");
    id_702 = bldr_52->gen_sym(bldr_52, "@test_fnc.v1.blk0.pfld");
    id_703 = bldr_52->gen_sym(bldr_52, "@test_fnc.v1.blk0.res");
    id_704 = bldr_52->gen_sym(bldr_52, NULL);
    bldr_52->new_getvarpartiref(bldr_52, id_704, id_702, true, id_695, id_701);
    id_705 = bldr_52->gen_sym(bldr_52, NULL);
    bldr_52->new_load(bldr_52, id_705, id_703, true, MU_ORD_NOT_ATOMIC, id_693, id_702, MU_NO_ID);
    id_706 = bldr_52->gen_sym(bldr_52, NULL);
    bldr_52->new_ret(bldr_52, id_706, (MuVarNode [1]){id_703}, 1);
    bldr_52->new_bb(bldr_52, id_700, (MuID [1]){id_701}, (MuTypeNode [1]){id_696}, 1, MU_NO_ID, (MuInstNode [3]){id_704, id_705, id_706}, 3);
    bldr_52->new_func_ver(bldr_52, id_699, id_698, (MuBBNode [1]){id_700}, 1);
    bldr_52->load(bldr_52);
    mu_52->compile_to_sharedlib(mu_52, LIB_FILE_NAME("test_getvarpartiref_nofix"), NULL, 0);
    return 0;
}
