
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_27;
    MuCtx* ctx_27;
    MuIRBuilder* bldr_27;
    MuID id_342;
    MuID id_343;
    MuID id_344;
    MuID id_345;
    MuID id_346;
    MuID id_347;
    MuID id_348;
    MuID id_349;
    MuID id_350;
    MuID id_351;
    mu_27 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_27 = mu_27->new_context(mu_27);
    bldr_27 = ctx_27->new_ir_builder(ctx_27);
    id_342 = bldr_27->gen_sym(bldr_27, "@dbl");
    bldr_27->new_type_double(bldr_27, id_342);
    id_343 = bldr_27->gen_sym(bldr_27, "@pi");
    bldr_27->new_const_double(bldr_27, id_343, id_342, 3.14159299999999985786);
    id_344 = bldr_27->gen_sym(bldr_27, "@e");
    bldr_27->new_const_double(bldr_27, id_344, id_342, 2.71828000000000002956);
    id_345 = bldr_27->gen_sym(bldr_27, "@sig__dbl");
    bldr_27->new_funcsig(bldr_27, id_345, NULL, 0, (MuTypeNode [1]){id_342}, 1);
    id_346 = bldr_27->gen_sym(bldr_27, "@test_fnc");
    bldr_27->new_func(bldr_27, id_346, id_345);
    id_347 = bldr_27->gen_sym(bldr_27, "@test_fnc.v1");
    id_348 = bldr_27->gen_sym(bldr_27, "@test_fnc.v1.blk0");
    id_349 = bldr_27->gen_sym(bldr_27, "@test_fnc.v1.blk0.res");
    id_350 = bldr_27->gen_sym(bldr_27, NULL);
    bldr_27->new_binop(bldr_27, id_350, id_349, MU_BINOP_FSUB, id_342, id_343, id_344, MU_NO_ID);
    id_351 = bldr_27->gen_sym(bldr_27, NULL);
    bldr_27->new_ret(bldr_27, id_351, (MuVarNode [1]){id_349}, 1);
    bldr_27->new_bb(bldr_27, id_348, NULL, NULL, 0, MU_NO_ID, (MuInstNode [2]){id_350, id_351}, 2);
    bldr_27->new_func_ver(bldr_27, id_347, id_346, (MuBBNode [1]){id_348}, 1);
    bldr_27->load(bldr_27);
    mu_27->compile_to_sharedlib(mu_27, "test_double_sub.dylib", NULL, 0);
    printf("%s\n", "test_double_sub.dylib");
    return 0;
}
