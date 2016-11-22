
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_36;
    MuCtx* ctx_36;
    MuIRBuilder* bldr_36;
    MuID id_558;
    MuID id_559;
    MuID id_560;
    MuID id_561;
    MuID id_562;
    MuID id_563;
    MuID id_564;
    MuID id_565;
    MuID id_566;
    MuID id_567;
    MuID id_568;
    MuID id_569;
    MuID id_570;
    MuID id_571;
    MuID id_572;
    MuID id_573;
    mu_36 = mu_fastimpl_new_with_opts("init_muinit_mu --log-level=none --aot-emit-dir=emit");
    ctx_36 = mu_36->new_context(mu_36);
    bldr_36 = ctx_36->new_ir_builder(ctx_36);
    id_558 = bldr_36->gen_sym(bldr_36, "@i1");
    bldr_36->new_type_int(bldr_36, id_558, 1);
    id_559 = bldr_36->gen_sym(bldr_36, "@i8");
    bldr_36->new_type_int(bldr_36, id_559, 8);
    id_560 = bldr_36->gen_sym(bldr_36, "@i64");
    bldr_36->new_type_int(bldr_36, id_560, 64);
    id_561 = bldr_36->gen_sym(bldr_36, "@10_i64");
    bldr_36->new_const_int(bldr_36, id_561, id_560, 10);
    id_562 = bldr_36->gen_sym(bldr_36, "@20_i64");
    bldr_36->new_const_int(bldr_36, id_562, id_560, 20);
    id_563 = bldr_36->gen_sym(bldr_36, "@TRUE");
    bldr_36->new_const_int(bldr_36, id_563, id_559, 1);
    id_564 = bldr_36->gen_sym(bldr_36, "@sig_i8_i64");
    bldr_36->new_funcsig(bldr_36, id_564, (MuTypeNode [1]){id_559}, 1, (MuTypeNode [1]){id_560}, 1);
    id_565 = bldr_36->gen_sym(bldr_36, "@test_fnc");
    bldr_36->new_func(bldr_36, id_565, id_564);
    id_566 = bldr_36->gen_sym(bldr_36, "@test_fnc.v1");
    id_567 = bldr_36->gen_sym(bldr_36, "@test_fnc.v1.blk0");
    id_568 = bldr_36->gen_sym(bldr_36, "@test_fnc.v1.blk0.flag");
    id_569 = bldr_36->gen_sym(bldr_36, "@test_fnc.v1.blk0.cmpres");
    id_570 = bldr_36->gen_sym(bldr_36, "@test_fnc.v1.blk0.res");
    id_571 = bldr_36->gen_sym(bldr_36, NULL);
    bldr_36->new_cmp(bldr_36, id_571, id_569, MU_CMP_EQ, id_560, id_568, id_563);
    id_572 = bldr_36->gen_sym(bldr_36, NULL);
    bldr_36->new_select(bldr_36, id_572, id_570, id_558, id_560, id_569, id_561, id_562);
    id_573 = bldr_36->gen_sym(bldr_36, NULL);
    bldr_36->new_ret(bldr_36, id_573, (MuVarNode [1]){id_570}, 1);
    bldr_36->new_bb(bldr_36, id_567, (MuID [1]){id_568}, (MuTypeNode [1]){id_559}, 1, MU_NO_ID, (MuInstNode [3]){id_571, id_572, id_573}, 3);
    bldr_36->new_func_ver(bldr_36, id_566, id_565, (MuBBNode [1]){id_567}, 1);
    bldr_36->load(bldr_36);
    mu_36->compile_to_sharedlib(mu_36, "test_select.dylib", NULL, 0);
    printf("%s\n", "test_select.dylib");
    return 0;
}
