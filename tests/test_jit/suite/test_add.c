
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu;
    MuCtx* ctx;
    MuIRBuilder* bldr;
    MuID id;
    MuID id_2;
    MuID id_3;
    MuID id_4;
    MuID id_5;
    MuID id_6;
    MuID id_7;
    MuID id_8;
    MuID id_9;
    MuID id_10;
    mu = mu_fastimpl_new();
    ctx = mu->new_context(mu);
    bldr = ctx->new_ir_builder(ctx);
    id = bldr->gen_sym(bldr, "@i8");
    bldr->new_type_int(bldr, id, 8);
    id_2 = bldr->gen_sym(bldr, "@0xff_i8");
    bldr->new_const_int(bldr, id_2, id, 255);
    id_3 = bldr->gen_sym(bldr, "@0x0a_i8");
    bldr->new_const_int(bldr, id_3, id, 10);
    id_4 = bldr->gen_sym(bldr, "@sig__i8");
    bldr->new_funcsig(bldr, id_4, NULL, 0, (MuTypeNode [1]){id}, 1);
    id_5 = bldr->gen_sym(bldr, "@test_fnc");
    bldr->new_func(bldr, id_5, id_4);
    id_6 = bldr->gen_sym(bldr, "@test_fnc_v1");
    id_7 = bldr->gen_sym(bldr, "@test_fnc_v1.blk0");
    id_8 = bldr->gen_sym(bldr, "@test_fnc_v1.blk0.res");
    id_9 = bldr->gen_sym(bldr, NULL);
    bldr->new_binop(bldr, id_9, id_8, MU_BINOP_ADD, id, id_2, id_3, MU_NO_ID);
    id_10 = bldr->gen_sym(bldr, NULL);
    bldr->new_ret(bldr, id_10, (MuVarNode [1]){id_8}, 1);
    bldr->new_bb(bldr, id_7, NULL, NULL, 0, MU_NO_ID, (MuInstNode [2]){id_9, id_10}, 2);
    bldr->new_func_ver(bldr, id_6, id_5, (MuBBNode [1]){id_7}, 1);
    bldr->load(bldr);
    mu->compile_to_sharedlib(mu, "test_add.dylib", NULL, 0);
    printf("%s\n", "test_add.dylib");
    return 0;
}
