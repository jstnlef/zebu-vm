
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_9;
    MuCtx* ctx_9;
    MuIRBuilder* bldr_9;
    MuID id_81;
    MuID id_82;
    MuID id_83;
    MuID id_84;
    MuID id_85;
    MuID id_86;
    MuID id_87;
    MuID id_88;
    MuID id_89;
    MuID id_90;
    mu_9 = mu_fastimpl_new();
    ctx_9 = mu_9->new_context(mu_9);
    bldr_9 = ctx_9->new_ir_builder(ctx_9);
    id_81 = bldr_9->gen_sym(bldr_9, "@i64");
    bldr_9->new_type_int(bldr_9, id_81, 64);
    id_82 = bldr_9->gen_sym(bldr_9, "@0x8d9f9c1d58324b55_i64");
    bldr_9->new_const_int(bldr_9, id_82, id_81, 10205046930492509013);
    id_83 = bldr_9->gen_sym(bldr_9, "@0xd5a8f2deb00debb4_i64");
    bldr_9->new_const_int(bldr_9, id_83, id_81, 15395822364416404404);
    id_84 = bldr_9->gen_sym(bldr_9, "@sig__i64");
    bldr_9->new_funcsig(bldr_9, id_84, NULL, 0, (MuTypeNode [1]){id_81}, 1);
    id_85 = bldr_9->gen_sym(bldr_9, "@test_fnc");
    bldr_9->new_func(bldr_9, id_85, id_84);
    id_86 = bldr_9->gen_sym(bldr_9, "@test_fnc_v1");
    id_87 = bldr_9->gen_sym(bldr_9, "@test_fnc_v1.blk0");
    id_88 = bldr_9->gen_sym(bldr_9, "@test_fnc_v1.blk0.res");
    id_89 = bldr_9->gen_sym(bldr_9, NULL);
    bldr_9->new_binop(bldr_9, id_89, id_88, MU_BINOP_XOR, id_81, id_82, id_83, MU_NO_ID);
    id_90 = bldr_9->gen_sym(bldr_9, NULL);
    bldr_9->new_ret(bldr_9, id_90, (MuVarNode [1]){id_88}, 1);
    bldr_9->new_bb(bldr_9, id_87, NULL, NULL, 0, MU_NO_ID, (MuInstNode [2]){id_89, id_90}, 2);
    bldr_9->new_func_ver(bldr_9, id_86, id_85, (MuBBNode [1]){id_87}, 1);
    bldr_9->load(bldr_9);
    mu_9->compile_to_sharedlib(mu_9, "test_xor.dylib", NULL, 0);
    printf("%s\n", "test_xor.dylib");
    return 0;
}
