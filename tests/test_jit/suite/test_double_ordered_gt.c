
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_35;
    MuCtx* ctx_35;
    MuIRBuilder* bldr_35;
    MuID id_443;
    MuID id_444;
    MuID id_445;
    MuID id_446;
    MuID id_447;
    MuID id_448;
    MuID id_449;
    MuID id_450;
    MuID id_451;
    MuID id_452;
    MuID id_453;
    MuID id_454;
    MuID id_455;
    MuID id_456;
    mu_35 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_35 = mu_35->new_context(mu_35);
    bldr_35 = ctx_35->new_ir_builder(ctx_35);
    id_443 = bldr_35->gen_sym(bldr_35, "@dbl");
    bldr_35->new_type_double(bldr_35, id_443);
    id_444 = bldr_35->gen_sym(bldr_35, "@i1");
    bldr_35->new_type_int(bldr_35, id_444, 1);
    id_445 = bldr_35->gen_sym(bldr_35, "@i64");
    bldr_35->new_type_int(bldr_35, id_445, 64);
    id_446 = bldr_35->gen_sym(bldr_35, "@pi");
    bldr_35->new_const_double(bldr_35, id_446, id_443, 3.14159299999999985786);
    id_447 = bldr_35->gen_sym(bldr_35, "@e");
    bldr_35->new_const_double(bldr_35, id_447, id_443, 2.71828000000000002956);
    id_448 = bldr_35->gen_sym(bldr_35, "@sig__i64");
    bldr_35->new_funcsig(bldr_35, id_448, NULL, 0, (MuTypeNode [1]){id_445}, 1);
    id_449 = bldr_35->gen_sym(bldr_35, "@test_fnc");
    bldr_35->new_func(bldr_35, id_449, id_448);
    id_450 = bldr_35->gen_sym(bldr_35, "@test_fnc.v1");
    id_451 = bldr_35->gen_sym(bldr_35, "@test_fnc.v1.blk0");
    id_452 = bldr_35->gen_sym(bldr_35, "@test_fnc.v1.blk0.cmpres");
    id_453 = bldr_35->gen_sym(bldr_35, "@test_fnc.v1.blk0.res");
    id_454 = bldr_35->gen_sym(bldr_35, NULL);
    bldr_35->new_cmp(bldr_35, id_454, id_452, MU_CMP_FOGT, id_443, id_446, id_447);
    id_455 = bldr_35->gen_sym(bldr_35, NULL);
    bldr_35->new_conv(bldr_35, id_455, id_453, MU_CONV_ZEXT, id_444, id_445, id_452);
    id_456 = bldr_35->gen_sym(bldr_35, NULL);
    bldr_35->new_ret(bldr_35, id_456, (MuVarNode [1]){id_453}, 1);
    bldr_35->new_bb(bldr_35, id_451, NULL, NULL, 0, MU_NO_ID, (MuInstNode [3]){id_454, id_455, id_456}, 3);
    bldr_35->new_func_ver(bldr_35, id_450, id_449, (MuBBNode [1]){id_451}, 1);
    bldr_35->load(bldr_35);
    mu_35->compile_to_sharedlib(mu_35, "test_double_ordered_gt.dylib", NULL, 0);
    printf("%s\n", "test_double_ordered_gt.dylib");
    return 0;
}
