
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
#ifdef __APPLE__
    #define LIB_EXT ".dylib"
#elif __linux__
    #define LIB_EXT ".so"
#elif _WIN32
    #define LIB_EXT ".dll"
#endif
#define LIB_FILE_NAME(name) "lib" name LIB_EXT
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
    MuID id_33;
    MuID id_34;
    MuID id_35;
    MuID id_36;
    MuID id_37;
    mu = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx = mu->new_context(mu);
    bldr = ctx->new_ir_builder(ctx);
    id = bldr->gen_sym(bldr, "@i64");
    bldr->new_type_int(bldr, id, 0x00000040ull);
    id_2 = bldr->gen_sym(bldr, "@0_i64");
    bldr->new_const_int(bldr, id_2, id, 0x0000000000000000ull);
    id_3 = bldr->gen_sym(bldr, "@1_i64");
    bldr->new_const_int(bldr, id_3, id, 0x0000000000000001ull);
    id_4 = bldr->gen_sym(bldr, "@2_i64");
    bldr->new_const_int(bldr, id_4, id, 0x0000000000000002ull);
    id_5 = bldr->gen_sym(bldr, "@20_i64");
    bldr->new_const_int(bldr, id_5, id, 0x0000000000000014ull);
    id_6 = bldr->gen_sym(bldr, "@sig_i64_i64");
    bldr->new_funcsig(bldr, id_6, (MuTypeNode [1]){id}, 1, (MuTypeNode [1]){id}, 1);
    id_7 = bldr->gen_sym(bldr, "@sig__i64");
    bldr->new_funcsig(bldr, id_7, NULL, 0, (MuTypeNode [1]){id}, 1);
    id_8 = bldr->gen_sym(bldr, "@fib");
    bldr->new_func(bldr, id_8, id_6);
    id_9 = bldr->gen_sym(bldr, "@fib_v1");
    id_10 = bldr->gen_sym(bldr, "@fib_v1.blk0");
    id_11 = bldr->gen_sym(bldr, "@fib_v1.blk1");
    id_12 = bldr->gen_sym(bldr, "@fib_v1.blk2");
    id_13 = bldr->gen_sym(bldr, "@fib_v1.blk0.k");
    id_14 = bldr->gen_sym(bldr, NULL);
    id_15 = bldr->gen_sym(bldr, NULL);
    id_16 = bldr->gen_sym(bldr, NULL);
    bldr->new_dest_clause(bldr, id_14, id_12, (MuVarNode [1]){id_13}, 1);
    bldr->new_dest_clause(bldr, id_15, id_11, (MuVarNode [1]){id_2}, 1);
    bldr->new_dest_clause(bldr, id_16, id_11, (MuVarNode [1]){id_3}, 1);
    id_17 = bldr->gen_sym(bldr, NULL);
    bldr->new_switch(bldr, id_17, id, id_13, id_14, (MuConstNode [2]){id_2, id_3}, (MuDestClause [2]){id_15, id_16}, 2);
    bldr->new_bb(bldr, id_10, (MuID [1]){id_13}, (MuTypeNode [1]){id}, 1, MU_NO_ID, (MuInstNode [1]){id_17}, 1);
    id_18 = bldr->gen_sym(bldr, "@fig_v1.blk1.rtn");
    id_19 = bldr->gen_sym(bldr, NULL);
    bldr->new_ret(bldr, id_19, (MuVarNode [1]){id_18}, 1);
    bldr->new_bb(bldr, id_11, (MuID [1]){id_18}, (MuTypeNode [1]){id}, 1, MU_NO_ID, (MuInstNode [1]){id_19}, 1);
    id_20 = bldr->gen_sym(bldr, "@fig_v1.blk2.k");
    id_21 = bldr->gen_sym(bldr, "@fig_v1.blk2.k_1");
    id_22 = bldr->gen_sym(bldr, "@fig_v1.blk2.k_2");
    id_23 = bldr->gen_sym(bldr, "@fig_v1.blk2.res");
    id_24 = bldr->gen_sym(bldr, "@fig_v1.blk2.res1");
    id_25 = bldr->gen_sym(bldr, "@fig_v1.blk2.res2");
    id_26 = bldr->gen_sym(bldr, NULL);
    bldr->new_binop(bldr, id_26, id_21, MU_BINOP_SUB, id, id_20, id_3, MU_NO_ID);
    id_27 = bldr->gen_sym(bldr, NULL);
    bldr->new_call(bldr, id_27, (MuID [1]){id_24}, 1, id_6, id_8, (MuVarNode [1]){id_21}, 1, MU_NO_ID, MU_NO_ID);
    id_28 = bldr->gen_sym(bldr, NULL);
    bldr->new_binop(bldr, id_28, id_22, MU_BINOP_SUB, id, id_20, id_4, MU_NO_ID);
    id_29 = bldr->gen_sym(bldr, NULL);
    bldr->new_call(bldr, id_29, (MuID [1]){id_25}, 1, id_6, id_8, (MuVarNode [1]){id_22}, 1, MU_NO_ID, MU_NO_ID);
    id_30 = bldr->gen_sym(bldr, NULL);
    bldr->new_binop(bldr, id_30, id_23, MU_BINOP_ADD, id, id_24, id_25, MU_NO_ID);
    id_31 = bldr->gen_sym(bldr, NULL);
    bldr->new_ret(bldr, id_31, (MuVarNode [1]){id_23}, 1);
    bldr->new_bb(bldr, id_12, (MuID [1]){id_20}, (MuTypeNode [1]){id}, 1, MU_NO_ID, (MuInstNode [6]){id_26, id_27, id_28, id_29, id_30, id_31}, 6);
    bldr->new_func_ver(bldr, id_9, id_8, (MuBBNode [3]){id_10, id_11, id_12}, 3);
    id_32 = bldr->gen_sym(bldr, "@entry");
    bldr->new_func(bldr, id_32, id_7);
    id_33 = bldr->gen_sym(bldr, "@entry_v1");
    id_34 = bldr->gen_sym(bldr, "@entry_v1.blk0");
    id_35 = bldr->gen_sym(bldr, "@entry_v1.blk0.res");
    id_36 = bldr->gen_sym(bldr, NULL);
    bldr->new_call(bldr, id_36, (MuID [1]){id_35}, 1, id_6, id_8, (MuVarNode [1]){id_5}, 1, MU_NO_ID, MU_NO_ID);
    id_37 = bldr->gen_sym(bldr, NULL);
    bldr->new_ret(bldr, id_37, (MuVarNode [1]){id_35}, 1);
    bldr->new_bb(bldr, id_34, NULL, NULL, 0, MU_NO_ID, (MuInstNode [2]){id_36, id_37}, 2);
    bldr->new_func_ver(bldr, id_33, id_32, (MuBBNode [1]){id_34}, 1);
    bldr->load(bldr);
    mu->compile_to_sharedlib(mu, LIB_FILE_NAME("test_multifunc"), NULL, 0);
    return 0;
}
