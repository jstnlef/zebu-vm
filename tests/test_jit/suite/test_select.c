
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
    MuID id_513;
    MuID id_514;
    MuID id_515;
    MuID id_516;
    MuID id_517;
    MuID id_518;
    MuID id_519;
    MuID id_520;
    MuID id_521;
    MuID id_522;
    MuID id_523;
    MuID id_524;
    MuID id_525;
    MuID id_526;
    MuID id_527;
    MuID id_528;
    mu_33 = mu_fastimpl_new();
    ctx_33 = mu_33->new_context(mu_33);
    bldr_33 = ctx_33->new_ir_builder(ctx_33);
    id_513 = bldr_33->gen_sym(bldr_33, "@i1");
    bldr_33->new_type_int(bldr_33, id_513, 1);
    id_514 = bldr_33->gen_sym(bldr_33, "@i8");
    bldr_33->new_type_int(bldr_33, id_514, 8);
    id_515 = bldr_33->gen_sym(bldr_33, "@i64");
    bldr_33->new_type_int(bldr_33, id_515, 64);
    id_516 = bldr_33->gen_sym(bldr_33, "@10_i64");
    bldr_33->new_const_int(bldr_33, id_516, id_515, 10);
    id_517 = bldr_33->gen_sym(bldr_33, "@20_i64");
    bldr_33->new_const_int(bldr_33, id_517, id_515, 20);
    id_518 = bldr_33->gen_sym(bldr_33, "@TRUE");
    bldr_33->new_const_int(bldr_33, id_518, id_514, 1);
    id_519 = bldr_33->gen_sym(bldr_33, "@sig_i8_i64");
    bldr_33->new_funcsig(bldr_33, id_519, (MuTypeNode [1]){id_514}, 1, (MuTypeNode [1]){id_515}, 1);
    id_520 = bldr_33->gen_sym(bldr_33, "@test_fnc");
    bldr_33->new_func(bldr_33, id_520, id_519);
    id_521 = bldr_33->gen_sym(bldr_33, "@test_fnc.v1");
    id_522 = bldr_33->gen_sym(bldr_33, "@test_fnc.v1.blk0");
    id_523 = bldr_33->gen_sym(bldr_33, "@test_fnc.v1.blk0.flag");
    id_524 = bldr_33->gen_sym(bldr_33, "@test_fnc.v1.blk0.cmpres");
    id_525 = bldr_33->gen_sym(bldr_33, "@test_fnc.v1.blk0.res");
    id_526 = bldr_33->gen_sym(bldr_33, NULL);
    bldr_33->new_cmp(bldr_33, id_526, id_524, MU_CMP_EQ, id_515, id_523, id_518);
    id_527 = bldr_33->gen_sym(bldr_33, NULL);
    bldr_33->new_select(bldr_33, id_527, id_525, id_513, id_515, id_524, id_516, id_517);
    id_528 = bldr_33->gen_sym(bldr_33, NULL);
    bldr_33->new_ret(bldr_33, id_528, (MuVarNode [1]){id_525}, 1);
    bldr_33->new_bb(bldr_33, id_522, (MuID [1]){id_523}, (MuTypeNode [1]){id_514}, 1, MU_NO_ID, (MuInstNode [3]){id_526, id_527, id_528}, 3);
    bldr_33->new_func_ver(bldr_33, id_521, id_520, (MuBBNode [1]){id_522}, 1);
    bldr_33->load(bldr_33);
    mu_33->compile_to_sharedlib(mu_33, "test_select.dylib", NULL, 0);
    printf("%s\n", "test_select.dylib");
    return 0;
}
