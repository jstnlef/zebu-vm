
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
    MuID id_12;
    MuID id_13;
    MuID id_14;
    MuID id_15;
    MuID id_16;
    MuID id_17;
    MuID id_18;
    MuID id_19;
    MuID id_20;
    MuID id_21;
    MuID id_22;
    MuID id_23;
    MuID id_24;
    MuID id_25;
    MuID id_26;
    MuID id_27;
    MuID id_28;
    MuID id_29;
    mu = mu_fastimpl_new();
    ctx = mu->new_context(mu);
    bldr = ctx->new_ir_builder(ctx);
    id = bldr->gen_sym(bldr, "@i64");
    bldr->new_type_int(bldr, id, 64);
    id_2 = bldr->gen_sym(bldr, "@0_i64");
    bldr->new_const_int(bldr, id_2, id, 0);
    id_3 = bldr->gen_sym(bldr, "@1_i64");
    bldr->new_const_int(bldr, id_3, id, 1);
    id_4 = bldr->gen_sym(bldr, "@2_i64");
    bldr->new_const_int(bldr, id_4, id, 2);
    id_5 = bldr->gen_sym(bldr, "@sig_i64_i64");
    bldr->new_funcsig(bldr, id_5, (MuTypeNode [1]){id}, 1, (MuTypeNode [1]){id}, 1);
    id_6 = bldr->gen_sym(bldr, "@fib");
    bldr->new_func(bldr, id_6, id_5);
    id_7 = bldr->gen_sym(bldr, "@fib_v1");
    id_8 = bldr->gen_sym(bldr, "@fib_v1.blk0");
    id_9 = bldr->gen_sym(bldr, "@fib_v1.blk1");
    id_10 = bldr->gen_sym(bldr, "@fib_v1.blk2");
    id_11 = bldr->gen_sym(bldr, "@fib_v1.blk0.k");
    id_12 = bldr->gen_sym(bldr, NULL);
    id_13 = bldr->gen_sym(bldr, NULL);
    id_14 = bldr->gen_sym(bldr, NULL);
    bldr->new_dest_clause(bldr, id_12, id_10, (MuVarNode [1]){id_11}, 1);
    bldr->new_dest_clause(bldr, id_13, id_9, (MuVarNode [1]){id_2}, 1);
    bldr->new_dest_clause(bldr, id_14, id_9, (MuVarNode [1]){id_3}, 1);
    id_15 = bldr->gen_sym(bldr, NULL);
    bldr->new_switch(bldr, id_15, id, id_11, id_12, (MuConstNode [2]){id_2, id_3}, (MuDestClause [2]){id_13, id_14}, 2);
    bldr->new_bb(bldr, id_8, (MuID [1]){id_11}, (MuTypeNode [1]){id}, 1, MU_NO_ID, (MuInstNode [1]){id_15}, 1);
    id_16 = bldr->gen_sym(bldr, "@fig_v1.blk1.rtn");
    id_17 = bldr->gen_sym(bldr, NULL);
    bldr->new_ret(bldr, id_17, (MuVarNode [1]){id_16}, 1);
    bldr->new_bb(bldr, id_9, (MuID [1]){id_16}, (MuTypeNode [1]){id}, 1, MU_NO_ID, (MuInstNode [1]){id_17}, 1);
    id_18 = bldr->gen_sym(bldr, "@fig_v1.blk2.k");
    id_19 = bldr->gen_sym(bldr, "@fig_v1.blk2.k_1");
    id_20 = bldr->gen_sym(bldr, "@fig_v1.blk2.k_2");
    id_21 = bldr->gen_sym(bldr, "@fig_v1.blk2.res");
    id_22 = bldr->gen_sym(bldr, "@fig_v1.blk2.res1");
    id_23 = bldr->gen_sym(bldr, "@fig_v1.blk2.res2");
    id_24 = bldr->gen_sym(bldr, NULL);
    bldr->new_binop(bldr, id_24, id_19, MU_BINOP_SUB, id, id_18, id_3, MU_NO_ID);
    id_25 = bldr->gen_sym(bldr, NULL);
    bldr->new_call(bldr, id_25, (MuID [1]){id_22}, 1, id_5, id_6, (MuVarNode [1]){id_19}, 1, MU_NO_ID, MU_NO_ID);
    id_26 = bldr->gen_sym(bldr, NULL);
    bldr->new_binop(bldr, id_26, id_20, MU_BINOP_SUB, id, id_18, id_4, MU_NO_ID);
    id_27 = bldr->gen_sym(bldr, NULL);
    bldr->new_call(bldr, id_27, (MuID [1]){id_23}, 1, id_5, id_6, (MuVarNode [1]){id_20}, 1, MU_NO_ID, MU_NO_ID);
    id_28 = bldr->gen_sym(bldr, NULL);
    bldr->new_binop(bldr, id_28, id_21, MU_BINOP_ADD, id, id_22, id_23, MU_NO_ID);
    id_29 = bldr->gen_sym(bldr, NULL);
    bldr->new_ret(bldr, id_29, (MuVarNode [1]){id_21}, 1);
    bldr->new_bb(bldr, id_10, (MuID [1]){id_18}, (MuTypeNode [1]){id}, 1, MU_NO_ID, (MuInstNode [6]){id_24, id_25, id_26, id_27, id_28, id_29}, 6);
    bldr->new_func_ver(bldr, id_7, id_6, (MuBBNode [3]){id_8, id_9, id_10}, 3);
    bldr->load(bldr);
    mu->compile_to_sharedlib(mu, "test_fib.dylib", NULL, 0);
    printf("%s\n", "test_fib.dylib");
    return 0;
}
