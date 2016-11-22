
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
    MuID id_119;
    MuID id_120;
    MuID id_121;
    MuID id_122;
    MuID id_123;
    MuID id_124;
    MuID id_125;
    MuID id_126;
    MuID id_127;
    MuID id_128;
    MuID id_129;
    MuID id_130;
    MuID id_131;
    mu_12 = mu_fastimpl_new_with_opts("init_muinit_mu --log-level=none --aot-emit-dir=emit");
    ctx_12 = mu_12->new_context(mu_12);
    bldr_12 = ctx_12->new_ir_builder(ctx_12);
    id_119 = bldr_12->gen_sym(bldr_12, "@i1");
    bldr_12->new_type_int(bldr_12, id_119, 1);
    id_120 = bldr_12->gen_sym(bldr_12, "@i64");
    bldr_12->new_type_int(bldr_12, id_120, 64);
    id_121 = bldr_12->gen_sym(bldr_12, "@0x8d9f9c1d58324b55_i64");
    bldr_12->new_const_int(bldr_12, id_121, id_120, 10205046930492509013);
    id_122 = bldr_12->gen_sym(bldr_12, "@0xd5a8f2deb00debb4_i64");
    bldr_12->new_const_int(bldr_12, id_122, id_120, 15395822364416404404);
    id_123 = bldr_12->gen_sym(bldr_12, "@sig__i64");
    bldr_12->new_funcsig(bldr_12, id_123, NULL, 0, (MuTypeNode [1]){id_120}, 1);
    id_124 = bldr_12->gen_sym(bldr_12, "@test_fnc");
    bldr_12->new_func(bldr_12, id_124, id_123);
    id_125 = bldr_12->gen_sym(bldr_12, "@test_fnc_v1");
    id_126 = bldr_12->gen_sym(bldr_12, "@test_fnc_v1.blk0");
    id_127 = bldr_12->gen_sym(bldr_12, "@test_fnc_v1.blk0.cmp_res");
    id_128 = bldr_12->gen_sym(bldr_12, "@test_fnc_v1.blk0.res");
    id_129 = bldr_12->gen_sym(bldr_12, NULL);
    bldr_12->new_cmp(bldr_12, id_129, id_127, MU_CMP_NE, id_120, id_121, id_122);
    id_130 = bldr_12->gen_sym(bldr_12, NULL);
    bldr_12->new_conv(bldr_12, id_130, id_128, MU_CONV_ZEXT, id_119, id_120, id_127);
    id_131 = bldr_12->gen_sym(bldr_12, NULL);
    bldr_12->new_ret(bldr_12, id_131, (MuVarNode [1]){id_128}, 1);
    bldr_12->new_bb(bldr_12, id_126, NULL, NULL, 0, MU_NO_ID, (MuInstNode [3]){id_129, id_130, id_131}, 3);
    bldr_12->new_func_ver(bldr_12, id_125, id_124, (MuBBNode [1]){id_126}, 1);
    bldr_12->load(bldr_12);
    mu_12->compile_to_sharedlib(mu_12, "test_ne_int.dylib", NULL, 0);
    printf("%s\n", "test_ne_int.dylib");
    return 0;
}
