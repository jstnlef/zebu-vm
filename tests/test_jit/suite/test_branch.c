
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_19;
    MuCtx* ctx_19;
    MuIRBuilder* bldr_19;
    MuID id_232;
    MuID id_233;
    MuID id_234;
    MuID id_235;
    MuID id_236;
    MuID id_237;
    MuID id_238;
    MuID id_239;
    MuID id_240;
    MuID id_241;
    MuID id_242;
    MuID id_243;
    MuID id_244;
    MuID id_245;
    MuID id_246;
    mu_19 = mu_fastimpl_new();
    ctx_19 = mu_19->new_context(mu_19);
    bldr_19 = ctx_19->new_ir_builder(ctx_19);
    id_232 = bldr_19->gen_sym(bldr_19, "@i64");
    bldr_19->new_type_int(bldr_19, id_232, 64);
    id_233 = bldr_19->gen_sym(bldr_19, "@10_i64");
    bldr_19->new_const_int(bldr_19, id_233, id_232, 10);
    id_234 = bldr_19->gen_sym(bldr_19, "@20_i64");
    bldr_19->new_const_int(bldr_19, id_234, id_232, 20);
    id_235 = bldr_19->gen_sym(bldr_19, "@sig__i64");
    bldr_19->new_funcsig(bldr_19, id_235, NULL, 0, (MuTypeNode [1]){id_232}, 1);
    id_236 = bldr_19->gen_sym(bldr_19, "@test_fnc");
    bldr_19->new_func(bldr_19, id_236, id_235);
    id_237 = bldr_19->gen_sym(bldr_19, "@test_fnc.v1");
    id_238 = bldr_19->gen_sym(bldr_19, "@test_fnc.v1.blk0");
    id_239 = bldr_19->gen_sym(bldr_19, "@test_fnc.v1.blk1");
    id_240 = bldr_19->gen_sym(bldr_19, NULL);
    id_241 = bldr_19->gen_sym(bldr_19, NULL);
    bldr_19->new_dest_clause(bldr_19, id_241, id_239, (MuVarNode [2]){id_233, id_234}, 2);
    bldr_19->new_branch(bldr_19, id_240, id_241);
    bldr_19->new_bb(bldr_19, id_238, NULL, NULL, 0, MU_NO_ID, (MuInstNode [1]){id_240}, 1);
    id_242 = bldr_19->gen_sym(bldr_19, "@test_fnc.v1.blk1.a");
    id_243 = bldr_19->gen_sym(bldr_19, "@test_fnc.v1.blk1.b");
    id_244 = bldr_19->gen_sym(bldr_19, "@test_fnc.v1.blk1.res");
    id_245 = bldr_19->gen_sym(bldr_19, NULL);
    bldr_19->new_binop(bldr_19, id_245, id_244, MU_BINOP_ADD, id_232, id_242, id_243, MU_NO_ID);
    id_246 = bldr_19->gen_sym(bldr_19, NULL);
    bldr_19->new_ret(bldr_19, id_246, (MuVarNode [1]){id_244}, 1);
    bldr_19->new_bb(bldr_19, id_239, (MuID [2]){id_242, id_243}, (MuTypeNode [2]){id_232, id_232}, 2, MU_NO_ID, (MuInstNode [2]){id_245, id_246}, 2);
    bldr_19->new_func_ver(bldr_19, id_237, id_236, (MuBBNode [2]){id_238, id_239}, 2);
    bldr_19->load(bldr_19);
    mu_19->compile_to_sharedlib(mu_19, "test_branch.dylib", NULL, 0);
    printf("%s\n", "test_branch.dylib");
    return 0;
}
