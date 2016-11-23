
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_30;
    MuCtx* ctx_30;
    MuIRBuilder* bldr_30;
    MuID id_372;
    MuID id_373;
    MuID id_374;
    MuID id_375;
    MuID id_376;
    MuID id_377;
    MuID id_378;
    MuID id_379;
    MuID id_380;
    MuID id_381;
    MuID id_382;
    MuID id_383;
    MuID id_384;
    MuID id_385;
    mu_30 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_30 = mu_30->new_context(mu_30);
    bldr_30 = ctx_30->new_ir_builder(ctx_30);
    id_372 = bldr_30->gen_sym(bldr_30, "@dbl");
    bldr_30->new_type_double(bldr_30, id_372);
    id_373 = bldr_30->gen_sym(bldr_30, "@i1");
    bldr_30->new_type_int(bldr_30, id_373, 1);
    id_374 = bldr_30->gen_sym(bldr_30, "@i64");
    bldr_30->new_type_int(bldr_30, id_374, 64);
    id_375 = bldr_30->gen_sym(bldr_30, "@pi");
    bldr_30->new_const_double(bldr_30, id_375, id_372, 3.14159299999999985786);
    id_376 = bldr_30->gen_sym(bldr_30, "@e");
    bldr_30->new_const_double(bldr_30, id_376, id_372, 2.71828000000000002956);
    id_377 = bldr_30->gen_sym(bldr_30, "@sig__i64");
    bldr_30->new_funcsig(bldr_30, id_377, NULL, 0, (MuTypeNode [1]){id_374}, 1);
    id_378 = bldr_30->gen_sym(bldr_30, "@test_fnc");
    bldr_30->new_func(bldr_30, id_378, id_377);
    id_379 = bldr_30->gen_sym(bldr_30, "@test_fnc.v1");
    id_380 = bldr_30->gen_sym(bldr_30, "@test_fnc.v1.blk0");
    id_381 = bldr_30->gen_sym(bldr_30, "@test_fnc.v1.blk0.cmpres");
    id_382 = bldr_30->gen_sym(bldr_30, "@test_fnc.v1.blk0.res");
    id_383 = bldr_30->gen_sym(bldr_30, NULL);
    bldr_30->new_cmp(bldr_30, id_383, id_381, MU_CMP_FOEQ, id_372, id_375, id_376);
    id_384 = bldr_30->gen_sym(bldr_30, NULL);
    bldr_30->new_conv(bldr_30, id_384, id_382, MU_CONV_ZEXT, id_373, id_374, id_381);
    id_385 = bldr_30->gen_sym(bldr_30, NULL);
    bldr_30->new_ret(bldr_30, id_385, (MuVarNode [1]){id_382}, 1);
    bldr_30->new_bb(bldr_30, id_380, NULL, NULL, 0, MU_NO_ID, (MuInstNode [3]){id_383, id_384, id_385}, 3);
    bldr_30->new_func_ver(bldr_30, id_379, id_378, (MuBBNode [1]){id_380}, 1);
    bldr_30->load(bldr_30);
    mu_30->compile_to_sharedlib(mu_30, "test_double_ordered_eq.dylib", NULL, 0);
    printf("%s\n", "test_double_ordered_eq.dylib");
    return 0;
}
