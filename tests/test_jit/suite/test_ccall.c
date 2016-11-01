
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
    MuID id_11;
    mu = mu_fastimpl_new();
    ctx = mu->new_context(mu);
    bldr = ctx->new_ir_builder(ctx);
    id = bldr->gen_sym(bldr, "@i64");
    bldr->new_type_int(bldr, id, 64);
    id_2 = bldr->gen_sym(bldr, "@sig_i64_i64");
    bldr->new_funcsig(bldr, id_2, (MuTypeNode [1]){id}, 1, (MuTypeNode [1]){id}, 1);
    id_3 = bldr->gen_sym(bldr, "@fnpsig_i64_i64");
    bldr->new_type_ufuncptr(bldr, id_3, id_2);
    id_4 = bldr->gen_sym(bldr, "@c_fnc");
    bldr->new_const_extern(bldr, id_4, id_3, "fnc");
    id_5 = bldr->gen_sym(bldr, "@test_ccall");
    bldr->new_func(bldr, id_5, id_2);
    id_6 = bldr->gen_sym(bldr, "@test_ccall_v1");
    id_7 = bldr->gen_sym(bldr, "@test_ccall_v1.blk0");
    id_8 = bldr->gen_sym(bldr, "@test_ccall_v1.blk0.k");
    id_9 = bldr->gen_sym(bldr, "@test_ccall_v1.blk0.res");
    id_10 = bldr->gen_sym(bldr, NULL);
    bldr->new_ccall(bldr, id_10, (MuID [1]){id_9}, 1, MU_CC_DEFAULT, id_3, id_2, id_4, (MuVarNode [1]){id_8}, 1, MU_NO_ID, MU_NO_ID);
    id_11 = bldr->gen_sym(bldr, NULL);
    bldr->new_ret(bldr, id_11, (MuVarNode [1]){id_9}, 1);
    bldr->new_bb(bldr, id_7, (MuID [1]){id_8}, (MuTypeNode [1]){id}, 1, MU_NO_ID, (MuInstNode [2]){id_10, id_11}, 2);
    bldr->new_func_ver(bldr, id_6, id_5, (MuBBNode [1]){id_7}, 1);
    bldr->load(bldr);
    mu->compile_to_sharedlib(mu, "test_ccall.dylib", (char**){&"test_ccall_fnc.c"}, 1);
    printf("%s\n", "test_ccall.dylib");
    return 0;
}
