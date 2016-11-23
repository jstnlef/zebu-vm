
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_34;
    MuCtx* ctx_34;
    MuIRBuilder* bldr_34;
    MuID id_430;
    MuID id_431;
    MuID id_432;
    MuID id_433;
    MuID id_434;
    MuID id_435;
    MuID id_436;
    MuID id_437;
    MuID id_438;
    MuID id_439;
    MuID id_440;
    MuID id_441;
    MuID id_442;
    mu_34 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_34 = mu_34->new_context(mu_34);
    bldr_34 = ctx_34->new_ir_builder(ctx_34);
    id_430 = bldr_34->gen_sym(bldr_34, "@dbl");
    bldr_34->new_type_double(bldr_34, id_430);
    id_431 = bldr_34->gen_sym(bldr_34, "@i1");
    bldr_34->new_type_int(bldr_34, id_431, 1);
    id_432 = bldr_34->gen_sym(bldr_34, "@i64");
    bldr_34->new_type_int(bldr_34, id_432, 64);
    id_433 = bldr_34->gen_sym(bldr_34, "@e");
    bldr_34->new_const_double(bldr_34, id_433, id_430, 2.71828000000000002956);
    id_434 = bldr_34->gen_sym(bldr_34, "@sig__i64");
    bldr_34->new_funcsig(bldr_34, id_434, NULL, 0, (MuTypeNode [1]){id_432}, 1);
    id_435 = bldr_34->gen_sym(bldr_34, "@test_fnc");
    bldr_34->new_func(bldr_34, id_435, id_434);
    id_436 = bldr_34->gen_sym(bldr_34, "@test_fnc.v1");
    id_437 = bldr_34->gen_sym(bldr_34, "@test_fnc.v1.blk0");
    id_438 = bldr_34->gen_sym(bldr_34, "@test_fnc.v1.blk0.cmpres");
    id_439 = bldr_34->gen_sym(bldr_34, "@test_fnc.v1.blk0.res");
    id_440 = bldr_34->gen_sym(bldr_34, NULL);
    bldr_34->new_cmp(bldr_34, id_440, id_438, MU_CMP_FOGE, id_430, id_433, id_433);
    id_441 = bldr_34->gen_sym(bldr_34, NULL);
    bldr_34->new_conv(bldr_34, id_441, id_439, MU_CONV_ZEXT, id_431, id_432, id_438);
    id_442 = bldr_34->gen_sym(bldr_34, NULL);
    bldr_34->new_ret(bldr_34, id_442, (MuVarNode [1]){id_439}, 1);
    bldr_34->new_bb(bldr_34, id_437, NULL, NULL, 0, MU_NO_ID, (MuInstNode [3]){id_440, id_441, id_442}, 3);
    bldr_34->new_func_ver(bldr_34, id_436, id_435, (MuBBNode [1]){id_437}, 1);
    bldr_34->load(bldr_34);
    mu_34->compile_to_sharedlib(mu_34, "test_double_ordered_ge.dylib", NULL, 0);
    printf("%s\n", "test_double_ordered_ge.dylib");
    return 0;
}
