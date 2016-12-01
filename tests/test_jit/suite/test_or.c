
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_12;
    MuCtx* ctx_12;
    MuIRBuilder* bldr_12;
    MuID id_111;
    MuID id_112;
    MuID id_113;
    MuID id_114;
    MuID id_115;
    MuID id_116;
    MuID id_117;
    MuID id_118;
    MuID id_119;
    MuID id_120;
    mu_12 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_12 = mu_12->new_context(mu_12);
    bldr_12 = ctx_12->new_ir_builder(ctx_12);
    id_111 = bldr_12->gen_sym(bldr_12, "@i64");
    bldr_12->new_type_int(bldr_12, id_111, 64);
    id_112 = bldr_12->gen_sym(bldr_12, "@0x8d9f9c1d58324b55_i64");
    bldr_12->new_const_int(bldr_12, id_112, id_111, 10205046930492509013);
    id_113 = bldr_12->gen_sym(bldr_12, "@0xd5a8f2deb00debb4_i64");
    bldr_12->new_const_int(bldr_12, id_113, id_111, 15395822364416404404);
    id_114 = bldr_12->gen_sym(bldr_12, "@sig__i64");
    bldr_12->new_funcsig(bldr_12, id_114, NULL, 0, (MuTypeNode [1]){id_111}, 1);
    id_115 = bldr_12->gen_sym(bldr_12, "@test_fnc");
    bldr_12->new_func(bldr_12, id_115, id_114);
    id_116 = bldr_12->gen_sym(bldr_12, "@test_fnc_v1");
    id_117 = bldr_12->gen_sym(bldr_12, "@test_fnc_v1.blk0");
    id_118 = bldr_12->gen_sym(bldr_12, "@test_fnc_v1.blk0.res");
    id_119 = bldr_12->gen_sym(bldr_12, NULL);
    bldr_12->new_binop(bldr_12, id_119, id_118, MU_BINOP_OR, id_111, id_112, id_113, MU_NO_ID);
    id_120 = bldr_12->gen_sym(bldr_12, NULL);
    bldr_12->new_ret(bldr_12, id_120, (MuVarNode [1]){id_118}, 1);
    bldr_12->new_bb(bldr_12, id_117, NULL, NULL, 0, MU_NO_ID, (MuInstNode [2]){id_119, id_120}, 2);
    bldr_12->new_func_ver(bldr_12, id_116, id_115, (MuBBNode [1]){id_117}, 1);
    bldr_12->load(bldr_12);
    mu_12->compile_to_sharedlib(mu_12, "test_or.dylib", NULL, 0);
    printf("%s\n", "test_or.dylib");
    return 0;
}
