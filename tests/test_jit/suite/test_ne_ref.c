
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_13;
    MuCtx* ctx_13;
    MuIRBuilder* bldr_13;
    MuID id_132;
    MuID id_133;
    MuID id_134;
    MuID id_135;
    MuID id_136;
    MuID id_137;
    MuID id_138;
    MuID id_139;
    MuID id_140;
    MuID id_141;
    MuID id_142;
    MuID id_143;
    MuID id_144;
    MuID id_145;
    MuID id_146;
    mu_13 = mu_fastimpl_new();
    ctx_13 = mu_13->new_context(mu_13);
    bldr_13 = ctx_13->new_ir_builder(ctx_13);
    id_132 = bldr_13->gen_sym(bldr_13, "@i1");
    bldr_13->new_type_int(bldr_13, id_132, 1);
    id_133 = bldr_13->gen_sym(bldr_13, "@i64");
    bldr_13->new_type_int(bldr_13, id_133, 64);
    id_134 = bldr_13->gen_sym(bldr_13, "@refi64");
    bldr_13->new_type_ref(bldr_13, id_134, id_133);
    id_135 = bldr_13->gen_sym(bldr_13, "@NULL_refi64");
    bldr_13->new_const_null(bldr_13, id_135, id_134);
    id_136 = bldr_13->gen_sym(bldr_13, "@sig__i64");
    bldr_13->new_funcsig(bldr_13, id_136, NULL, 0, (MuTypeNode [1]){id_133}, 1);
    id_137 = bldr_13->gen_sym(bldr_13, "@test_fnc");
    bldr_13->new_func(bldr_13, id_137, id_136);
    id_138 = bldr_13->gen_sym(bldr_13, "@test_fnc_v1");
    id_139 = bldr_13->gen_sym(bldr_13, "@test_fnc_v1.blk0");
    id_140 = bldr_13->gen_sym(bldr_13, "@test_fnc_v1.blk0.r");
    id_141 = bldr_13->gen_sym(bldr_13, "@test_fnc_v1.blk0.cmp_res");
    id_142 = bldr_13->gen_sym(bldr_13, "@test_fnc_v1.blk0.res");
    id_143 = bldr_13->gen_sym(bldr_13, NULL);
    bldr_13->new_new(bldr_13, id_143, id_140, id_133, MU_NO_ID);
    id_144 = bldr_13->gen_sym(bldr_13, NULL);
    bldr_13->new_cmp(bldr_13, id_144, id_141, MU_CMP_NE, id_134, id_140, id_135);
    id_145 = bldr_13->gen_sym(bldr_13, NULL);
    bldr_13->new_conv(bldr_13, id_145, id_142, MU_CONV_ZEXT, id_132, id_133, id_141);
    id_146 = bldr_13->gen_sym(bldr_13, NULL);
    bldr_13->new_ret(bldr_13, id_146, (MuVarNode [1]){id_142}, 1);
    bldr_13->new_bb(bldr_13, id_139, NULL, NULL, 0, MU_NO_ID, (MuInstNode [4]){id_143, id_144, id_145, id_146}, 4);
    bldr_13->new_func_ver(bldr_13, id_138, id_137, (MuBBNode [1]){id_139}, 1);
    bldr_13->load(bldr_13);
    mu_13->compile_to_sharedlib(mu_13, "test_ne_ref.dylib", NULL, 0);
    printf("%s\n", "test_ne_ref.dylib");
    return 0;
}
