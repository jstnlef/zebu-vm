
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_5;
    MuCtx* ctx_5;
    MuIRBuilder* bldr_5;
    MuID id_41;
    MuID id_42;
    MuID id_43;
    MuID id_44;
    MuID id_45;
    MuID id_46;
    MuID id_47;
    MuID id_48;
    MuID id_49;
    MuID id_50;
    MuCString var_5;
    mu_5 = mu_fastimpl_new();
    ctx_5 = mu_5->new_context(mu_5);
    bldr_5 = ctx_5->new_ir_builder(ctx_5);
    id_41 = bldr_5->gen_sym(bldr_5, "@i8");
    bldr_5->new_type_int(bldr_5, id_41, 8);
    id_42 = bldr_5->gen_sym(bldr_5, "@0xff_i8");
    bldr_5->new_const_int(bldr_5, id_42, id_41, 255);
    id_43 = bldr_5->gen_sym(bldr_5, "@0x0a_i8");
    bldr_5->new_const_int(bldr_5, id_43, id_41, 10);
    id_44 = bldr_5->gen_sym(bldr_5, "@sig__i8");
    bldr_5->new_funcsig(bldr_5, id_44, NULL, 0, (MuTypeNode [1]){id_41}, 1);
    id_45 = bldr_5->gen_sym(bldr_5, "@test_fnc");
    bldr_5->new_func(bldr_5, id_45, id_44);
    id_46 = bldr_5->gen_sym(bldr_5, "@test_fnc_v1");
    id_47 = bldr_5->gen_sym(bldr_5, "@test_fnc_v1.blk0");
    id_48 = bldr_5->gen_sym(bldr_5, "@test_fnc_v1.blk0.res");
    id_49 = bldr_5->gen_sym(bldr_5, NULL);
    bldr_5->new_binop(bldr_5, id_49, id_48, MU_BINOP_UREM, id_41, id_42, id_43, MU_NO_ID);
    id_50 = bldr_5->gen_sym(bldr_5, NULL);
    bldr_5->new_ret(bldr_5, id_50, (MuVarNode [1]){id_48}, 1);
    bldr_5->new_bb(bldr_5, id_47, NULL, NULL, 0, MU_NO_ID, (MuInstNode [2]){id_49, id_50}, 2);
    bldr_5->new_func_ver(bldr_5, id_46, id_45, (MuBBNode [1]){id_47}, 1);
    bldr_5->load(bldr_5);
    var_5 = mu_5->compile_to_sharedlib(mu_5, id_45);
    printf("%s\n", var_5);
    return 0;
}
