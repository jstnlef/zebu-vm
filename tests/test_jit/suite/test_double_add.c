
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
    mu_26 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_26 = mu_26->new_context(mu_26);
    bldr_26 = ctx_26->new_ir_builder(ctx_26);
    id_332 = bldr_26->gen_sym(bldr_26, "@dbl");
    bldr_26->new_type_double(bldr_26, id_332);
    id_333 = bldr_26->gen_sym(bldr_26, "@pi");
    bldr_26->new_const_double(bldr_26, id_333, id_332, 3.14159299999999985786);
    id_334 = bldr_26->gen_sym(bldr_26, "@e");
    bldr_26->new_const_double(bldr_26, id_334, id_332, 2.71828000000000002956);
    id_335 = bldr_26->gen_sym(bldr_26, "@sig__dbl");
    bldr_26->new_funcsig(bldr_26, id_335, NULL, 0, (MuTypeNode [1]){id_332}, 1);
    id_336 = bldr_26->gen_sym(bldr_26, "@test_fnc");
    bldr_26->new_func(bldr_26, id_336, id_335);
    id_337 = bldr_26->gen_sym(bldr_26, "@test_fnc.v1");
    id_338 = bldr_26->gen_sym(bldr_26, "@test_fnc.v1.blk0");
    id_339 = bldr_26->gen_sym(bldr_26, "@test_fnc.v1.blk0.res");
    id_340 = bldr_26->gen_sym(bldr_26, NULL);
    bldr_26->new_binop(bldr_26, id_340, id_339, MU_BINOP_FADD, id_332, id_333, id_334, MU_NO_ID);
    id_341 = bldr_26->gen_sym(bldr_26, NULL);
    bldr_26->new_ret(bldr_26, id_341, (MuVarNode [1]){id_339}, 1);
    bldr_26->new_bb(bldr_26, id_338, NULL, NULL, 0, MU_NO_ID, (MuInstNode [2]){id_340, id_341}, 2);
    bldr_26->new_func_ver(bldr_26, id_337, id_336, (MuBBNode [1]){id_338}, 1);
    bldr_26->load(bldr_26);
    mu_26->compile_to_sharedlib(mu_26, "test_double_add.dylib", NULL, 0);
    printf("%s\n", "test_double_add.dylib");
    return 0;
}
