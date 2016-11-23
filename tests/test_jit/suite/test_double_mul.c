
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_28;
    MuCtx* ctx_28;
    MuIRBuilder* bldr_28;
    MuID id_352;
    MuID id_353;
    MuID id_354;
    MuID id_355;
    MuID id_356;
    MuID id_357;
    MuID id_358;
    MuID id_359;
    MuID id_360;
    MuID id_361;
    mu_28 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_28 = mu_28->new_context(mu_28);
    bldr_28 = ctx_28->new_ir_builder(ctx_28);
    id_352 = bldr_28->gen_sym(bldr_28, "@dbl");
    bldr_28->new_type_double(bldr_28, id_352);
    id_353 = bldr_28->gen_sym(bldr_28, "@pi");
    bldr_28->new_const_double(bldr_28, id_353, id_352, 3.14159299999999985786);
    id_354 = bldr_28->gen_sym(bldr_28, "@e");
    bldr_28->new_const_double(bldr_28, id_354, id_352, 2.71828000000000002956);
    id_355 = bldr_28->gen_sym(bldr_28, "@sig__dbl");
    bldr_28->new_funcsig(bldr_28, id_355, NULL, 0, (MuTypeNode [1]){id_352}, 1);
    id_356 = bldr_28->gen_sym(bldr_28, "@test_fnc");
    bldr_28->new_func(bldr_28, id_356, id_355);
    id_357 = bldr_28->gen_sym(bldr_28, "@test_fnc.v1");
    id_358 = bldr_28->gen_sym(bldr_28, "@test_fnc.v1.blk0");
    id_359 = bldr_28->gen_sym(bldr_28, "@test_fnc.v1.blk0.res");
    id_360 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_binop(bldr_28, id_360, id_359, MU_BINOP_FMUL, id_352, id_353, id_354, MU_NO_ID);
    id_361 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_ret(bldr_28, id_361, (MuVarNode [1]){id_359}, 1);
    bldr_28->new_bb(bldr_28, id_358, NULL, NULL, 0, MU_NO_ID, (MuInstNode [2]){id_360, id_361}, 2);
    bldr_28->new_func_ver(bldr_28, id_357, id_356, (MuBBNode [1]){id_358}, 1);
    bldr_28->load(bldr_28);
    mu_28->compile_to_sharedlib(mu_28, "test_double_mul.dylib", NULL, 0);
    printf("%s\n", "test_double_mul.dylib");
    return 0;
}
