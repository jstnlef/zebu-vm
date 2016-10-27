
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_15;
    MuCtx* ctx_15;
    MuIRBuilder* bldr_15;
    MuID id_164;
    MuID id_165;
    MuID id_166;
    MuID id_167;
    MuID id_168;
    MuID id_169;
    MuID id_170;
    MuID id_171;
    MuID id_172;
    MuID id_173;
    MuID id_174;
    MuID id_175;
    MuID id_176;
    MuID id_177;
    MuID id_178;
    MuID id_179;
    MuID id_180;
    mu_15 = mu_fastimpl_new();
    ctx_15 = mu_15->new_context(mu_15);
    bldr_15 = ctx_15->new_ir_builder(ctx_15);
    id_164 = bldr_15->gen_sym(bldr_15, "@i1");
    bldr_15->new_type_int(bldr_15, id_164, 1);
    id_165 = bldr_15->gen_sym(bldr_15, "@i8");
    bldr_15->new_type_int(bldr_15, id_165, 8);
    id_166 = bldr_15->gen_sym(bldr_15, "@0xff_i8");
    bldr_15->new_const_int(bldr_15, id_166, id_165, 255);
    id_167 = bldr_15->gen_sym(bldr_15, "@0x0a_i8");
    bldr_15->new_const_int(bldr_15, id_167, id_165, 10);
    id_168 = bldr_15->gen_sym(bldr_15, "@sig__i8");
    bldr_15->new_funcsig(bldr_15, id_168, NULL, 0, (MuTypeNode [1]){id_165}, 1);
    id_169 = bldr_15->gen_sym(bldr_15, "@test_fnc");
    bldr_15->new_func(bldr_15, id_169, id_168);
    id_170 = bldr_15->gen_sym(bldr_15, "@test_fnc_v1");
    id_171 = bldr_15->gen_sym(bldr_15, "@test_fnc_v1.blk0");
    id_172 = bldr_15->gen_sym(bldr_15, "@test_fnc_v1.blk0.cmp_res_1");
    id_173 = bldr_15->gen_sym(bldr_15, "@test_fnc_v1.blk0.cmp_res_2");
    id_174 = bldr_15->gen_sym(bldr_15, "@test_fnc_v1.blk0.bin_res");
    id_175 = bldr_15->gen_sym(bldr_15, "@test_fnc_v1.blk0.res");
    id_176 = bldr_15->gen_sym(bldr_15, NULL);
    bldr_15->new_cmp(bldr_15, id_176, id_172, MU_CMP_SGT, id_165, id_166, id_167);
    id_177 = bldr_15->gen_sym(bldr_15, NULL);
    bldr_15->new_cmp(bldr_15, id_177, id_173, MU_CMP_SGT, id_165, id_166, id_166);
    id_178 = bldr_15->gen_sym(bldr_15, NULL);
    bldr_15->new_binop(bldr_15, id_178, id_174, MU_BINOP_OR, id_164, id_172, id_173, MU_NO_ID);
    id_179 = bldr_15->gen_sym(bldr_15, NULL);
    bldr_15->new_conv(bldr_15, id_179, id_175, MU_CONV_ZEXT, id_164, id_165, id_174);
    id_180 = bldr_15->gen_sym(bldr_15, NULL);
    bldr_15->new_ret(bldr_15, id_180, (MuVarNode [1]){id_175}, 1);
    bldr_15->new_bb(bldr_15, id_171, NULL, NULL, 0, MU_NO_ID, (MuInstNode [5]){id_176, id_177, id_178, id_179, id_180}, 5);
    bldr_15->new_func_ver(bldr_15, id_170, id_169, (MuBBNode [1]){id_171}, 1);
    bldr_15->load(bldr_15);
    mu_15->compile_to_sharedlib(mu_15, "test_sgt.dylib");
    printf("%s\n", "test_sgt.dylib");
    return 0;
}
