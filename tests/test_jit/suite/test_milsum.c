
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
    MuID id_30;
    MuID id_31;
    MuID id_32;
    mu = mu_fastimpl_new();
    ctx = mu->new_context(mu);
    bldr = ctx->new_ir_builder(ctx);
    id = bldr->gen_sym(bldr, "@i64");
    bldr->new_type_int(bldr, id, 64);
    id_2 = bldr->gen_sym(bldr, "@0_i64");
    bldr->new_const_int(bldr, id_2, id, 0);
    id_3 = bldr->gen_sym(bldr, "@1_i64");
    bldr->new_const_int(bldr, id_3, id, 1);
    id_4 = bldr->gen_sym(bldr, "@sig_i64_i64");
    bldr->new_funcsig(bldr, id_4, (MuTypeNode [1]){id}, 1, (MuTypeNode [1]){id}, 1);
    id_5 = bldr->gen_sym(bldr, "@milsum");
    bldr->new_func(bldr, id_5, id_4);
    id_6 = bldr->gen_sym(bldr, "@milsum.v1");
    id_7 = bldr->gen_sym(bldr, "@milsum.v1.blk0");
    id_8 = bldr->gen_sym(bldr, "@milsum.v1.blk1");
    id_9 = bldr->gen_sym(bldr, "@milsum.v1.blk2");
    id_10 = bldr->gen_sym(bldr, "@milsum.v1.blk3");
    id_11 = bldr->gen_sym(bldr, "@milsum.v1.blk0.k");
    id_12 = bldr->gen_sym(bldr, NULL);
    bldr->new_dest_clause(bldr, id_12, id_8, (MuVarNode [3]){id_2, id_2, id_11}, 3);
    id_13 = bldr->gen_sym(bldr, NULL);
    bldr->new_branch(bldr, id_13, id_12);
    bldr->new_bb(bldr, id_7, (MuID [1]){id_11}, (MuTypeNode [1]){id}, 1, MU_NO_ID, (MuInstNode [1]){id_13}, 1);
    id_14 = bldr->gen_sym(bldr, "@milsum.v1.blk1.acc");
    id_15 = bldr->gen_sym(bldr, "@milsum.v1.blk1.i");
    id_16 = bldr->gen_sym(bldr, "@milsum.v1.blk1.end");
    id_17 = bldr->gen_sym(bldr, "@milsum.v1.blk1.cmpres");
    id_18 = bldr->gen_sym(bldr, NULL);
    bldr->new_cmp(bldr, id_18, id_17, MU_CMP_EQ, id, id_15, id_16);
    id_19 = bldr->gen_sym(bldr, NULL);
    id_20 = bldr->gen_sym(bldr, NULL);
    bldr->new_dest_clause(bldr, id_20, id_10, (MuVarNode [1]){id_14}, 1);
    id_21 = bldr->gen_sym(bldr, NULL);
    bldr->new_dest_clause(bldr, id_21, id_9, (MuVarNode [3]){id_14, id_15, id_16}, 3);
    bldr->new_branch2(bldr, id_19, id_17, id_20, id_21);
    bldr->new_bb(bldr, id_8, (MuID [3]){id_14, id_15, id_16}, (MuTypeNode [3]){id, id, id}, 3, MU_NO_ID, (MuInstNode [2]){id_18, id_19}, 2);
    id_22 = bldr->gen_sym(bldr, "@milsum.v1.blk2.acc");
    id_23 = bldr->gen_sym(bldr, "@milsum.v1.blk2.i");
    id_24 = bldr->gen_sym(bldr, "@milsum.v1.blk2.end");
    id_25 = bldr->gen_sym(bldr, "@milsum.v1.blk2.acc_res");
    id_26 = bldr->gen_sym(bldr, "@milsum.v1.blk2.i_res");
    id_27 = bldr->gen_sym(bldr, NULL);
    bldr->new_binop(bldr, id_27, id_26, MU_BINOP_ADD, id, id_23, id_3, MU_NO_ID);
    id_28 = bldr->gen_sym(bldr, NULL);
    bldr->new_binop(bldr, id_28, id_25, MU_BINOP_ADD, id, id_22, id_26, MU_NO_ID);
    id_29 = bldr->gen_sym(bldr, NULL);
    bldr->new_dest_clause(bldr, id_29, id_8, (MuVarNode [3]){id_25, id_26, id_24}, 3);
    id_30 = bldr->gen_sym(bldr, NULL);
    bldr->new_branch(bldr, id_30, id_29);
    bldr->new_bb(bldr, id_9, (MuID [3]){id_22, id_23, id_24}, (MuTypeNode [3]){id, id, id}, 3, MU_NO_ID, (MuInstNode [3]){id_27, id_28, id_30}, 3);
    id_31 = bldr->gen_sym(bldr, "@milsum.v1.blk3.rtn");
    id_32 = bldr->gen_sym(bldr, NULL);
    bldr->new_ret(bldr, id_32, (MuVarNode [1]){id_31}, 1);
    bldr->new_bb(bldr, id_10, (MuID [1]){id_31}, (MuTypeNode [1]){id}, 1, MU_NO_ID, (MuInstNode [1]){id_32}, 1);
    bldr->new_func_ver(bldr, id_6, id_5, (MuBBNode [4]){id_7, id_8, id_9, id_10}, 4);
    bldr->load(bldr);
    mu->compile_to_sharedlib(mu, "test_milsum.dylib", NULL, 0);
    printf("%s\n", "test_milsum.dylib");
    return 0;
}
