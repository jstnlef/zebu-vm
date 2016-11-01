
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_16;
    MuCtx* ctx_16;
    MuIRBuilder* bldr_16;
    MuID id_181;
    MuID id_182;
    MuID id_183;
    MuID id_184;
    MuID id_185;
    MuID id_186;
    MuID id_187;
    MuID id_188;
    MuID id_189;
    MuID id_190;
    MuID id_191;
    MuID id_192;
    MuID id_193;
    MuID id_194;
    MuID id_195;
    MuID id_196;
    MuID id_197;
    mu_16 = mu_fastimpl_new();
    ctx_16 = mu_16->new_context(mu_16);
    bldr_16 = ctx_16->new_ir_builder(ctx_16);
    id_181 = bldr_16->gen_sym(bldr_16, "@i1");
    bldr_16->new_type_int(bldr_16, id_181, 1);
    id_182 = bldr_16->gen_sym(bldr_16, "@i8");
    bldr_16->new_type_int(bldr_16, id_182, 8);
    id_183 = bldr_16->gen_sym(bldr_16, "@0xff_i8");
    bldr_16->new_const_int(bldr_16, id_183, id_182, 255);
    id_184 = bldr_16->gen_sym(bldr_16, "@0x0a_i8");
    bldr_16->new_const_int(bldr_16, id_184, id_182, 10);
    id_185 = bldr_16->gen_sym(bldr_16, "@sig__i8");
    bldr_16->new_funcsig(bldr_16, id_185, NULL, 0, (MuTypeNode [1]){id_182}, 1);
    id_186 = bldr_16->gen_sym(bldr_16, "@test_fnc");
    bldr_16->new_func(bldr_16, id_186, id_185);
    id_187 = bldr_16->gen_sym(bldr_16, "@test_fnc_v1");
    id_188 = bldr_16->gen_sym(bldr_16, "@test_fnc_v1.blk0");
    id_189 = bldr_16->gen_sym(bldr_16, "@test_fnc_v1.blk0.cmp_res_1");
    id_190 = bldr_16->gen_sym(bldr_16, "@test_fnc_v1.blk0.cmp_res_2");
    id_191 = bldr_16->gen_sym(bldr_16, "@test_fnc_v1.blk0.bin_res");
    id_192 = bldr_16->gen_sym(bldr_16, "@test_fnc_v1.blk0.res");
    id_193 = bldr_16->gen_sym(bldr_16, NULL);
    bldr_16->new_cmp(bldr_16, id_193, id_189, MU_CMP_SLE, id_182, id_184, id_183);
    id_194 = bldr_16->gen_sym(bldr_16, NULL);
    bldr_16->new_cmp(bldr_16, id_194, id_190, MU_CMP_SLE, id_182, id_183, id_183);
    id_195 = bldr_16->gen_sym(bldr_16, NULL);
    bldr_16->new_binop(bldr_16, id_195, id_191, MU_BINOP_XOR, id_181, id_189, id_190, MU_NO_ID);
    id_196 = bldr_16->gen_sym(bldr_16, NULL);
    bldr_16->new_conv(bldr_16, id_196, id_192, MU_CONV_ZEXT, id_181, id_182, id_191);
    id_197 = bldr_16->gen_sym(bldr_16, NULL);
    bldr_16->new_ret(bldr_16, id_197, (MuVarNode [1]){id_192}, 1);
    bldr_16->new_bb(bldr_16, id_188, NULL, NULL, 0, MU_NO_ID, (MuInstNode [5]){id_193, id_194, id_195, id_196, id_197}, 5);
    bldr_16->new_func_ver(bldr_16, id_187, id_186, (MuBBNode [1]){id_188}, 1);
    bldr_16->load(bldr_16);
    mu_16->compile_to_sharedlib(mu_16, "test_sle.dylib", NULL, 0);
    printf("%s\n", "test_sle.dylib");
    return 0;
}
