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
    MuVM* mu_55;
    MuCtx* ctx_55;
    MuIRBuilder* bldr_55;
    MuID id_744;
    MuID id_745;
    MuID id_746;
    MuID id_747;
    MuID id_748;
    MuID id_749;
    MuID id_750;
    MuID id_751;
    MuID id_752;
    MuID id_753;
    MuID id_754;
    MuID id_755;
    MuID id_756;
    MuID id_757;
    MuID id_758;
    MuID id_759;
    MuID id_760;
    MuID id_761;
    MuID id_762;
    MuID id_763;
    MuID id_764;
    MuID id_765;
    MuID id_766;
    MuID id_767;
    MuID id_768;
    MuID id_769;
    MuID id_770;
    MuID id_771;
    MuID id_772;
    MuID id_773;
    MuID id_774;
    MuID id_775;
    mu_55 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_55 = mu_55->new_context(mu_55);
    bldr_55 = ctx_55->new_ir_builder(ctx_55);
    id_744 = bldr_55->gen_sym(bldr_55, "@i64");
    bldr_55->new_type_int(bldr_55, id_744, 0x00000040ull);
    id_745 = bldr_55->gen_sym(bldr_55, "@0_i64");
    bldr_55->new_const_int(bldr_55, id_745, id_744, 0x0000000000000000ull);
    id_746 = bldr_55->gen_sym(bldr_55, "@1_i64");
    bldr_55->new_const_int(bldr_55, id_746, id_744, 0x0000000000000001ull);
    id_747 = bldr_55->gen_sym(bldr_55, "@sig_i64_i64");
    bldr_55->new_funcsig(bldr_55, id_747, (MuTypeNode [1]){id_744}, 1, (MuTypeNode [1]){id_744}, 1);
    id_748 = bldr_55->gen_sym(bldr_55, "@fac");
    bldr_55->new_func(bldr_55, id_748, id_747);
    id_749 = bldr_55->gen_sym(bldr_55, "@fac.v1");
    id_750 = bldr_55->gen_sym(bldr_55, "@fac.v1.blk0");
    id_751 = bldr_55->gen_sym(bldr_55, "@fac.v1.blk1");
    id_752 = bldr_55->gen_sym(bldr_55, "@fac.v1.blk2");
    id_753 = bldr_55->gen_sym(bldr_55, "@fac.v1.blk3");
    id_754 = bldr_55->gen_sym(bldr_55, "@fac.v1.blk0.k");
    id_755 = bldr_55->gen_sym(bldr_55, NULL);
    bldr_55->new_dest_clause(bldr_55, id_755, id_751, (MuVarNode [3]){id_746, id_745, id_754}, 3);
    id_756 = bldr_55->gen_sym(bldr_55, NULL);
    bldr_55->new_branch(bldr_55, id_756, id_755);
    bldr_55->new_bb(bldr_55, id_750, (MuID [1]){id_754}, (MuTypeNode [1]){id_744}, 1, MU_NO_ID, (MuInstNode [1]){id_756}, 1);
    id_757 = bldr_55->gen_sym(bldr_55, "@fac.v1.blk1.prod");
    id_758 = bldr_55->gen_sym(bldr_55, "@fac.v1.blk1.i");
    id_759 = bldr_55->gen_sym(bldr_55, "@fac.v1.blk1.end");
    id_760 = bldr_55->gen_sym(bldr_55, "@fac.v1.blk1.cmpres");
    id_761 = bldr_55->gen_sym(bldr_55, NULL);
    bldr_55->new_cmp(bldr_55, id_761, id_760, MU_CMP_EQ, id_744, id_758, id_759);
    id_762 = bldr_55->gen_sym(bldr_55, NULL);
    id_763 = bldr_55->gen_sym(bldr_55, NULL);
    bldr_55->new_dest_clause(bldr_55, id_763, id_753, (MuVarNode [1]){id_757}, 1);
    id_764 = bldr_55->gen_sym(bldr_55, NULL);
    bldr_55->new_dest_clause(bldr_55, id_764, id_752, (MuVarNode [3]){id_757, id_758, id_759}, 3);
    bldr_55->new_branch2(bldr_55, id_762, id_760, id_763, id_764);
    bldr_55->new_bb(bldr_55, id_751, (MuID [3]){id_757, id_758, id_759}, (MuTypeNode [3]){id_744, id_744, id_744}, 3, MU_NO_ID, (MuInstNode [2]){id_761, id_762}, 2);
    id_765 = bldr_55->gen_sym(bldr_55, "@fac.v1.blk2.prod");
    id_766 = bldr_55->gen_sym(bldr_55, "@fac.v1.blk2.i");
    id_767 = bldr_55->gen_sym(bldr_55, "@fac.v1.blk2.end");
    id_768 = bldr_55->gen_sym(bldr_55, "@fac.v1.blk2.prod_res");
    id_769 = bldr_55->gen_sym(bldr_55, "@fac.v1.blk2.i_res");
    id_770 = bldr_55->gen_sym(bldr_55, NULL);
    bldr_55->new_binop(bldr_55, id_770, id_769, MU_BINOP_ADD, id_744, id_766, id_746, MU_NO_ID);
    id_771 = bldr_55->gen_sym(bldr_55, NULL);
    bldr_55->new_binop(bldr_55, id_771, id_768, MU_BINOP_MUL, id_744, id_765, id_769, MU_NO_ID);
    id_772 = bldr_55->gen_sym(bldr_55, NULL);
    bldr_55->new_dest_clause(bldr_55, id_772, id_751, (MuVarNode [3]){id_768, id_769, id_767}, 3);
    id_773 = bldr_55->gen_sym(bldr_55, NULL);
    bldr_55->new_branch(bldr_55, id_773, id_772);
    bldr_55->new_bb(bldr_55, id_752, (MuID [3]){id_765, id_766, id_767}, (MuTypeNode [3]){id_744, id_744, id_744}, 3, MU_NO_ID, (MuInstNode [3]){id_770, id_771, id_773}, 3);
    id_774 = bldr_55->gen_sym(bldr_55, "@fac.v1.blk3.rtn");
    id_775 = bldr_55->gen_sym(bldr_55, NULL);
    bldr_55->new_ret(bldr_55, id_775, (MuVarNode [1]){id_774}, 1);
    bldr_55->new_bb(bldr_55, id_753, (MuID [1]){id_774}, (MuTypeNode [1]){id_744}, 1, MU_NO_ID, (MuInstNode [1]){id_775}, 1);
    bldr_55->new_func_ver(bldr_55, id_749, id_748, (MuBBNode [4]){id_750, id_751, id_752, id_753}, 4);
    bldr_55->load(bldr_55);
    mu_55->compile_to_sharedlib(mu_55, LIB_FILE_NAME("test_fac"), NULL, 0);
    return 0;
}
