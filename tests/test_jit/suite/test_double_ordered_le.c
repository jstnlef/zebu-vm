
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_33;
    MuCtx* ctx_33;
    MuIRBuilder* bldr_33;
    MuID id_417;
    MuID id_418;
    MuID id_419;
    MuID id_420;
    MuID id_421;
    MuID id_422;
    MuID id_423;
    MuID id_424;
    MuID id_425;
    MuID id_426;
    MuID id_427;
    MuID id_428;
    MuID id_429;
    mu_33 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_33 = mu_33->new_context(mu_33);
    bldr_33 = ctx_33->new_ir_builder(ctx_33);
    id_417 = bldr_33->gen_sym(bldr_33, "@dbl");
    bldr_33->new_type_double(bldr_33, id_417);
    id_418 = bldr_33->gen_sym(bldr_33, "@i1");
    bldr_33->new_type_int(bldr_33, id_418, 1);
    id_419 = bldr_33->gen_sym(bldr_33, "@i64");
    bldr_33->new_type_int(bldr_33, id_419, 64);
    id_420 = bldr_33->gen_sym(bldr_33, "@pi");
    bldr_33->new_const_double(bldr_33, id_420, id_417, 3.14159299999999985786);
    id_421 = bldr_33->gen_sym(bldr_33, "@sig__i64");
    bldr_33->new_funcsig(bldr_33, id_421, NULL, 0, (MuTypeNode [1]){id_419}, 1);
    id_422 = bldr_33->gen_sym(bldr_33, "@test_fnc");
    bldr_33->new_func(bldr_33, id_422, id_421);
    id_423 = bldr_33->gen_sym(bldr_33, "@test_fnc.v1");
    id_424 = bldr_33->gen_sym(bldr_33, "@test_fnc.v1.blk0");
    id_425 = bldr_33->gen_sym(bldr_33, "@test_fnc.v1.blk0.cmpres");
    id_426 = bldr_33->gen_sym(bldr_33, "@test_fnc.v1.blk0.res");
    id_427 = bldr_33->gen_sym(bldr_33, NULL);
    bldr_33->new_cmp(bldr_33, id_427, id_425, MU_CMP_FOLE, id_417, id_420, id_420);
    id_428 = bldr_33->gen_sym(bldr_33, NULL);
    bldr_33->new_conv(bldr_33, id_428, id_426, MU_CONV_ZEXT, id_418, id_419, id_425);
    id_429 = bldr_33->gen_sym(bldr_33, NULL);
    bldr_33->new_ret(bldr_33, id_429, (MuVarNode [1]){id_426}, 1);
    bldr_33->new_bb(bldr_33, id_424, NULL, NULL, 0, MU_NO_ID, (MuInstNode [3]){id_427, id_428, id_429}, 3);
    bldr_33->new_func_ver(bldr_33, id_423, id_422, (MuBBNode [1]){id_424}, 1);
    bldr_33->load(bldr_33);
    mu_33->compile_to_sharedlib(mu_33, "test_double_ordered_le.dylib", NULL, 0);
    printf("%s\n", "test_double_ordered_le.dylib");
    return 0;
}
