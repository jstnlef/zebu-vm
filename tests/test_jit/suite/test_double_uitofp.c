
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_39;
    MuCtx* ctx_39;
    MuIRBuilder* bldr_39;
    MuID id_489;
    MuID id_490;
    MuID id_491;
    MuID id_492;
    MuID id_493;
    MuID id_494;
    MuID id_495;
    MuID id_496;
    MuID id_497;
    MuID id_498;
    MuID id_499;
    mu_39 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_39 = mu_39->new_context(mu_39);
    bldr_39 = ctx_39->new_ir_builder(ctx_39);
    id_489 = bldr_39->gen_sym(bldr_39, "@dbl");
    bldr_39->new_type_double(bldr_39, id_489);
    id_490 = bldr_39->gen_sym(bldr_39, "@i1");
    bldr_39->new_type_int(bldr_39, id_490, 1);
    id_491 = bldr_39->gen_sym(bldr_39, "@i64");
    bldr_39->new_type_int(bldr_39, id_491, 64);
    id_492 = bldr_39->gen_sym(bldr_39, "@k");
    bldr_39->new_const_int(bldr_39, id_492, id_491, 42);
    id_493 = bldr_39->gen_sym(bldr_39, "@sig__dbl");
    bldr_39->new_funcsig(bldr_39, id_493, NULL, 0, (MuTypeNode [1]){id_489}, 1);
    id_494 = bldr_39->gen_sym(bldr_39, "@test_fnc");
    bldr_39->new_func(bldr_39, id_494, id_493);
    id_495 = bldr_39->gen_sym(bldr_39, "@test_fnc.v1");
    id_496 = bldr_39->gen_sym(bldr_39, "@test_fnc.v1.blk0");
    id_497 = bldr_39->gen_sym(bldr_39, "@test_fnc.v1.blk0.res");
    id_498 = bldr_39->gen_sym(bldr_39, NULL);
    bldr_39->new_conv(bldr_39, id_498, id_497, MU_CONV_UITOFP, id_491, id_489, id_492);
    id_499 = bldr_39->gen_sym(bldr_39, NULL);
    bldr_39->new_ret(bldr_39, id_499, (MuVarNode [1]){id_497}, 1);
    bldr_39->new_bb(bldr_39, id_496, NULL, NULL, 0, MU_NO_ID, (MuInstNode [2]){id_498, id_499}, 2);
    bldr_39->new_func_ver(bldr_39, id_495, id_494, (MuBBNode [1]){id_496}, 1);
    bldr_39->load(bldr_39);
    mu_39->compile_to_sharedlib(mu_39, "test_double_uitofp.dylib", NULL, 0);
    printf("%s\n", "test_double_uitofp.dylib");
    return 0;
}
