
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_10;
    MuCtx* ctx_10;
    MuIRBuilder* bldr_10;
    MuID id_91;
    MuID id_92;
    MuID id_93;
    MuID id_94;
    MuID id_95;
    MuID id_96;
    MuID id_97;
    MuID id_98;
    MuID id_99;
    MuID id_100;
    MuID id_101;
    MuID id_102;
    MuID id_103;
    mu_10 = mu_fastimpl_new();
    ctx_10 = mu_10->new_context(mu_10);
    bldr_10 = ctx_10->new_ir_builder(ctx_10);
    id_91 = bldr_10->gen_sym(bldr_10, "@i1");
    bldr_10->new_type_int(bldr_10, id_91, 1);
    id_92 = bldr_10->gen_sym(bldr_10, "@i64");
    bldr_10->new_type_int(bldr_10, id_92, 64);
    id_93 = bldr_10->gen_sym(bldr_10, "@0x8d9f9c1d58324b55_i64");
    bldr_10->new_const_int(bldr_10, id_93, id_92, 10205046930492509013);
    id_94 = bldr_10->gen_sym(bldr_10, "@0xd5a8f2deb00debb4_i64");
    bldr_10->new_const_int(bldr_10, id_94, id_92, 15395822364416404404);
    id_95 = bldr_10->gen_sym(bldr_10, "@sig__i64");
    bldr_10->new_funcsig(bldr_10, id_95, NULL, 0, (MuTypeNode [1]){id_92}, 1);
    id_96 = bldr_10->gen_sym(bldr_10, "@test_fnc");
    bldr_10->new_func(bldr_10, id_96, id_95);
    id_97 = bldr_10->gen_sym(bldr_10, "@test_fnc_v1");
    id_98 = bldr_10->gen_sym(bldr_10, "@test_fnc_v1.blk0");
    id_99 = bldr_10->gen_sym(bldr_10, "@test_fnc_v1.blk0.cmp_res");
    id_100 = bldr_10->gen_sym(bldr_10, "@test_fnc_v1.blk0.res");
    id_101 = bldr_10->gen_sym(bldr_10, NULL);
    bldr_10->new_cmp(bldr_10, id_101, id_99, MU_CMP_EQ, id_92, id_93, id_94);
    id_102 = bldr_10->gen_sym(bldr_10, NULL);
    bldr_10->new_conv(bldr_10, id_102, id_100, MU_CONV_ZEXT, id_91, id_92, id_99);
    id_103 = bldr_10->gen_sym(bldr_10, NULL);
    bldr_10->new_ret(bldr_10, id_103, (MuVarNode [1]){id_100}, 1);
    bldr_10->new_bb(bldr_10, id_98, NULL, NULL, 0, MU_NO_ID, (MuInstNode [3]){id_101, id_102, id_103}, 3);
    bldr_10->new_func_ver(bldr_10, id_97, id_96, (MuBBNode [1]){id_98}, 1);
    bldr_10->load(bldr_10);
    mu_10->compile_to_sharedlib(mu_10, "test_eq_int.dylib", NULL, 0);
    printf("%s\n", "test_eq_int.dylib");
    return 0;
}
