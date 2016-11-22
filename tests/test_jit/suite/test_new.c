
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_26;
    MuCtx* ctx_26;
    MuIRBuilder* bldr_26;
    MuID id_332;
    MuID id_333;
    MuID id_334;
    MuID id_335;
    MuID id_336;
    MuID id_337;
    MuID id_338;
    MuID id_339;
    MuID id_340;
    MuID id_341;
    MuID id_342;
    MuID id_343;
    MuID id_344;
    MuID id_345;
    MuID id_346;
    mu_26 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_26 = mu_26->new_context(mu_26);
    bldr_26 = ctx_26->new_ir_builder(ctx_26);
    id_332 = bldr_26->gen_sym(bldr_26, "@i1");
    bldr_26->new_type_int(bldr_26, id_332, 1);
    id_333 = bldr_26->gen_sym(bldr_26, "@i64");
    bldr_26->new_type_int(bldr_26, id_333, 64);
    id_334 = bldr_26->gen_sym(bldr_26, "@refi64");
    bldr_26->new_type_ref(bldr_26, id_334, id_333);
    id_335 = bldr_26->gen_sym(bldr_26, "@NULL_refi64");
    bldr_26->new_const_null(bldr_26, id_335, id_334);
    id_336 = bldr_26->gen_sym(bldr_26, "@sig__i64");
    bldr_26->new_funcsig(bldr_26, id_336, NULL, 0, (MuTypeNode [1]){id_333}, 1);
    id_337 = bldr_26->gen_sym(bldr_26, "@test_fnc");
    bldr_26->new_func(bldr_26, id_337, id_336);
    id_338 = bldr_26->gen_sym(bldr_26, "@test_fnc.v1");
    id_339 = bldr_26->gen_sym(bldr_26, "@test_fnc.v1.blk0");
    id_340 = bldr_26->gen_sym(bldr_26, "@test_fnc.v1.blk0.r");
    id_341 = bldr_26->gen_sym(bldr_26, "@test_fnc.v1.blk0.cmpres");
    id_342 = bldr_26->gen_sym(bldr_26, "@test_fnc.v1.blk0.res");
    id_343 = bldr_26->gen_sym(bldr_26, NULL);
    bldr_26->new_new(bldr_26, id_343, id_340, id_333, MU_NO_ID);
    id_344 = bldr_26->gen_sym(bldr_26, NULL);
    bldr_26->new_cmp(bldr_26, id_344, id_341, MU_CMP_EQ, id_334, id_340, id_335);
    id_345 = bldr_26->gen_sym(bldr_26, NULL);
    bldr_26->new_conv(bldr_26, id_345, id_342, MU_CONV_ZEXT, id_332, id_333, id_341);
    id_346 = bldr_26->gen_sym(bldr_26, NULL);
    bldr_26->new_ret(bldr_26, id_346, (MuVarNode [1]){id_342}, 1);
    bldr_26->new_bb(bldr_26, id_339, NULL, NULL, 0, MU_NO_ID, (MuInstNode [4]){id_343, id_344, id_345, id_346}, 4);
    bldr_26->new_func_ver(bldr_26, id_338, id_337, (MuBBNode [1]){id_339}, 1);
    bldr_26->load(bldr_26);
    mu_26->compile_to_sharedlib(mu_26, "test_new.dylib", NULL, 0);
    printf("%s\n", "test_new.dylib");
    return 0;
}
