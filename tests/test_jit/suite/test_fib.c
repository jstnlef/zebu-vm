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
    MuVM* mu_54;
    MuCtx* ctx_54;
    MuIRBuilder* bldr_54;
    MuID id_715;
    MuID id_716;
    MuID id_717;
    MuID id_718;
    MuID id_719;
    MuID id_720;
    MuID id_721;
    MuID id_722;
    MuID id_723;
    MuID id_724;
    MuID id_725;
    MuID id_726;
    MuID id_727;
    MuID id_728;
    MuID id_729;
    MuID id_730;
    MuID id_731;
    MuID id_732;
    MuID id_733;
    MuID id_734;
    MuID id_735;
    MuID id_736;
    MuID id_737;
    MuID id_738;
    MuID id_739;
    MuID id_740;
    MuID id_741;
    MuID id_742;
    MuID id_743;
    mu_54 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_54 = mu_54->new_context(mu_54);
    bldr_54 = ctx_54->new_ir_builder(ctx_54);
    id_715 = bldr_54->gen_sym(bldr_54, "@i64");
    bldr_54->new_type_int(bldr_54, id_715, 0x00000040ull);
    id_716 = bldr_54->gen_sym(bldr_54, "@0_i64");
    bldr_54->new_const_int(bldr_54, id_716, id_715, 0x0000000000000000ull);
    id_717 = bldr_54->gen_sym(bldr_54, "@1_i64");
    bldr_54->new_const_int(bldr_54, id_717, id_715, 0x0000000000000001ull);
    id_718 = bldr_54->gen_sym(bldr_54, "@2_i64");
    bldr_54->new_const_int(bldr_54, id_718, id_715, 0x0000000000000002ull);
    id_719 = bldr_54->gen_sym(bldr_54, "@sig_i64_i64");
    bldr_54->new_funcsig(bldr_54, id_719, (MuTypeNode [1]){id_715}, 1, (MuTypeNode [1]){id_715}, 1);
    id_720 = bldr_54->gen_sym(bldr_54, "@fib");
    bldr_54->new_func(bldr_54, id_720, id_719);
    id_721 = bldr_54->gen_sym(bldr_54, "@fib_v1");
    id_722 = bldr_54->gen_sym(bldr_54, "@fib_v1.blk0");
    id_723 = bldr_54->gen_sym(bldr_54, "@fib_v1.blk1");
    id_724 = bldr_54->gen_sym(bldr_54, "@fib_v1.blk2");
    id_725 = bldr_54->gen_sym(bldr_54, "@fib_v1.blk0.k");
    id_726 = bldr_54->gen_sym(bldr_54, NULL);
    id_727 = bldr_54->gen_sym(bldr_54, NULL);
    id_728 = bldr_54->gen_sym(bldr_54, NULL);
    bldr_54->new_dest_clause(bldr_54, id_726, id_724, (MuVarNode [1]){id_725}, 1);
    bldr_54->new_dest_clause(bldr_54, id_727, id_723, (MuVarNode [1]){id_716}, 1);
    bldr_54->new_dest_clause(bldr_54, id_728, id_723, (MuVarNode [1]){id_717}, 1);
    id_729 = bldr_54->gen_sym(bldr_54, NULL);
    bldr_54->new_switch(bldr_54, id_729, id_715, id_725, id_726, (MuConstNode [2]){id_716, id_717}, (MuDestClause [2]){id_727, id_728}, 2);
    bldr_54->new_bb(bldr_54, id_722, (MuID [1]){id_725}, (MuTypeNode [1]){id_715}, 1, MU_NO_ID, (MuInstNode [1]){id_729}, 1);
    id_730 = bldr_54->gen_sym(bldr_54, "@fig_v1.blk1.rtn");
    id_731 = bldr_54->gen_sym(bldr_54, NULL);
    bldr_54->new_ret(bldr_54, id_731, (MuVarNode [1]){id_730}, 1);
    bldr_54->new_bb(bldr_54, id_723, (MuID [1]){id_730}, (MuTypeNode [1]){id_715}, 1, MU_NO_ID, (MuInstNode [1]){id_731}, 1);
    id_732 = bldr_54->gen_sym(bldr_54, "@fig_v1.blk2.k");
    id_733 = bldr_54->gen_sym(bldr_54, "@fig_v1.blk2.k_1");
    id_734 = bldr_54->gen_sym(bldr_54, "@fig_v1.blk2.k_2");
    id_735 = bldr_54->gen_sym(bldr_54, "@fig_v1.blk2.res");
    id_736 = bldr_54->gen_sym(bldr_54, "@fig_v1.blk2.res1");
    id_737 = bldr_54->gen_sym(bldr_54, "@fig_v1.blk2.res2");
    id_738 = bldr_54->gen_sym(bldr_54, NULL);
    bldr_54->new_binop(bldr_54, id_738, id_733, MU_BINOP_SUB, id_715, id_732, id_717, MU_NO_ID);
    id_739 = bldr_54->gen_sym(bldr_54, NULL);
    bldr_54->new_call(bldr_54, id_739, (MuID [1]){id_736}, 1, id_719, id_720, (MuVarNode [1]){id_733}, 1, MU_NO_ID, MU_NO_ID);
    id_740 = bldr_54->gen_sym(bldr_54, NULL);
    bldr_54->new_binop(bldr_54, id_740, id_734, MU_BINOP_SUB, id_715, id_732, id_718, MU_NO_ID);
    id_741 = bldr_54->gen_sym(bldr_54, NULL);
    bldr_54->new_call(bldr_54, id_741, (MuID [1]){id_737}, 1, id_719, id_720, (MuVarNode [1]){id_734}, 1, MU_NO_ID, MU_NO_ID);
    id_742 = bldr_54->gen_sym(bldr_54, NULL);
    bldr_54->new_binop(bldr_54, id_742, id_735, MU_BINOP_ADD, id_715, id_736, id_737, MU_NO_ID);
    id_743 = bldr_54->gen_sym(bldr_54, NULL);
    bldr_54->new_ret(bldr_54, id_743, (MuVarNode [1]){id_735}, 1);
    bldr_54->new_bb(bldr_54, id_724, (MuID [1]){id_732}, (MuTypeNode [1]){id_715}, 1, MU_NO_ID, (MuInstNode [6]){id_738, id_739, id_740, id_741, id_742, id_743}, 6);
    bldr_54->new_func_ver(bldr_54, id_721, id_720, (MuBBNode [3]){id_722, id_723, id_724}, 3);
    bldr_54->load(bldr_54);
    mu_54->compile_to_sharedlib(mu_54, LIB_FILE_NAME("test_fib"), NULL, 0);
    return 0;
}
