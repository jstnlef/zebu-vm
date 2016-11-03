
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_22;
    MuCtx* ctx_22;
    MuIRBuilder* bldr_22;
    MuID id_262;
    MuID id_263;
    MuID id_264;
    MuID id_265;
    MuID id_266;
    MuID id_267;
    MuID id_268;
    MuID id_269;
    MuID id_270;
    MuID id_271;
    MuID id_272;
    MuID id_273;
    MuID id_274;
    MuID id_275;
    MuID id_276;
    mu_22 = mu_fastimpl_new();
    ctx_22 = mu_22->new_context(mu_22);
    bldr_22 = ctx_22->new_ir_builder(ctx_22);
    id_262 = bldr_22->gen_sym(bldr_22, "@i1");
    bldr_22->new_type_int(bldr_22, id_262, 1);
    id_263 = bldr_22->gen_sym(bldr_22, "@i64");
    bldr_22->new_type_int(bldr_22, id_263, 64);
    id_264 = bldr_22->gen_sym(bldr_22, "@refi64");
    bldr_22->new_type_ref(bldr_22, id_264, id_263);
    id_265 = bldr_22->gen_sym(bldr_22, "@NULL_refi64");
    bldr_22->new_const_null(bldr_22, id_265, id_264);
    id_266 = bldr_22->gen_sym(bldr_22, "@sig__i64");
    bldr_22->new_funcsig(bldr_22, id_266, NULL, 0, (MuTypeNode [1]){id_263}, 1);
    id_267 = bldr_22->gen_sym(bldr_22, "@test_fnc");
    bldr_22->new_func(bldr_22, id_267, id_266);
    id_268 = bldr_22->gen_sym(bldr_22, "@test_fnc.v1");
    id_269 = bldr_22->gen_sym(bldr_22, "@test_fnc.v1.blk0");
    id_270 = bldr_22->gen_sym(bldr_22, "@test_fnc.v1.blk0.r");
    id_271 = bldr_22->gen_sym(bldr_22, "@test_fnc.v1.blk0.cmpres");
    id_272 = bldr_22->gen_sym(bldr_22, "@test_fnc.v1.blk0.res");
    id_273 = bldr_22->gen_sym(bldr_22, NULL);
    bldr_22->new_new(bldr_22, id_273, id_270, id_263, MU_NO_ID);
    id_274 = bldr_22->gen_sym(bldr_22, NULL);
    bldr_22->new_cmp(bldr_22, id_274, id_271, MU_CMP_EQ, id_264, id_270, id_265);
    id_275 = bldr_22->gen_sym(bldr_22, NULL);
    bldr_22->new_conv(bldr_22, id_275, id_272, MU_CONV_ZEXT, id_262, id_263, id_271);
    id_276 = bldr_22->gen_sym(bldr_22, NULL);
    bldr_22->new_ret(bldr_22, id_276, (MuVarNode [1]){id_272}, 1);
    bldr_22->new_bb(bldr_22, id_269, NULL, NULL, 0, MU_NO_ID, (MuInstNode [4]){id_273, id_274, id_275, id_276}, 4);
    bldr_22->new_func_ver(bldr_22, id_268, id_267, (MuBBNode [1]){id_269}, 1);
    bldr_22->load(bldr_22);
    mu_22->compile_to_sharedlib(mu_22, "test_new.dylib", NULL, 0);
    printf("%s\n", "test_new.dylib");
    return 0;
}
