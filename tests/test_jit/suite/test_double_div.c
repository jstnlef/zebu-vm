
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_29;
    MuCtx* ctx_29;
    MuIRBuilder* bldr_29;
    MuID id_362;
    MuID id_363;
    MuID id_364;
    MuID id_365;
    MuID id_366;
    MuID id_367;
    MuID id_368;
    MuID id_369;
    MuID id_370;
    MuID id_371;
    mu_29 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_29 = mu_29->new_context(mu_29);
    bldr_29 = ctx_29->new_ir_builder(ctx_29);
    id_362 = bldr_29->gen_sym(bldr_29, "@dbl");
    bldr_29->new_type_double(bldr_29, id_362);
    id_363 = bldr_29->gen_sym(bldr_29, "@pi");
    bldr_29->new_const_double(bldr_29, id_363, id_362, 3.14159299999999985786);
    id_364 = bldr_29->gen_sym(bldr_29, "@e");
    bldr_29->new_const_double(bldr_29, id_364, id_362, 2.71828000000000002956);
    id_365 = bldr_29->gen_sym(bldr_29, "@sig__dbl");
    bldr_29->new_funcsig(bldr_29, id_365, NULL, 0, (MuTypeNode [1]){id_362}, 1);
    id_366 = bldr_29->gen_sym(bldr_29, "@test_fnc");
    bldr_29->new_func(bldr_29, id_366, id_365);
    id_367 = bldr_29->gen_sym(bldr_29, "@test_fnc.v1");
    id_368 = bldr_29->gen_sym(bldr_29, "@test_fnc.v1.blk0");
    id_369 = bldr_29->gen_sym(bldr_29, "@test_fnc.v1.blk0.res");
    id_370 = bldr_29->gen_sym(bldr_29, NULL);
    bldr_29->new_binop(bldr_29, id_370, id_369, MU_BINOP_FMUL, id_362, id_363, id_364, MU_NO_ID);
    id_371 = bldr_29->gen_sym(bldr_29, NULL);
    bldr_29->new_ret(bldr_29, id_371, (MuVarNode [1]){id_369}, 1);
    bldr_29->new_bb(bldr_29, id_368, NULL, NULL, 0, MU_NO_ID, (MuInstNode [2]){id_370, id_371}, 2);
    bldr_29->new_func_ver(bldr_29, id_367, id_366, (MuBBNode [1]){id_368}, 1);
    bldr_29->load(bldr_29);
    mu_29->compile_to_sharedlib(mu_29, "test_double_div.dylib", NULL, 0);
    printf("%s\n", "test_double_div.dylib");
    return 0;
}
