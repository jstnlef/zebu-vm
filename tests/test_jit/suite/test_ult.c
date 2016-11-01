
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_18;
    MuCtx* ctx_18;
    MuIRBuilder* bldr_18;
    MuID id_215;
    MuID id_216;
    MuID id_217;
    MuID id_218;
    MuID id_219;
    MuID id_220;
    MuID id_221;
    MuID id_222;
    MuID id_223;
    MuID id_224;
    MuID id_225;
    MuID id_226;
    MuID id_227;
    MuID id_228;
    MuID id_229;
    MuID id_230;
    MuID id_231;
    mu_18 = mu_fastimpl_new();
    ctx_18 = mu_18->new_context(mu_18);
    bldr_18 = ctx_18->new_ir_builder(ctx_18);
    id_215 = bldr_18->gen_sym(bldr_18, "@i1");
    bldr_18->new_type_int(bldr_18, id_215, 1);
    id_216 = bldr_18->gen_sym(bldr_18, "@i8");
    bldr_18->new_type_int(bldr_18, id_216, 8);
    id_217 = bldr_18->gen_sym(bldr_18, "@0xff_i8");
    bldr_18->new_const_int(bldr_18, id_217, id_216, 255);
    id_218 = bldr_18->gen_sym(bldr_18, "@0x0a_i8");
    bldr_18->new_const_int(bldr_18, id_218, id_216, 10);
    id_219 = bldr_18->gen_sym(bldr_18, "@sig__i8");
    bldr_18->new_funcsig(bldr_18, id_219, NULL, 0, (MuTypeNode [1]){id_216}, 1);
    id_220 = bldr_18->gen_sym(bldr_18, "@test_fnc");
    bldr_18->new_func(bldr_18, id_220, id_219);
    id_221 = bldr_18->gen_sym(bldr_18, "@test_fnc_v1");
    id_222 = bldr_18->gen_sym(bldr_18, "@test_fnc_v1.blk0");
    id_223 = bldr_18->gen_sym(bldr_18, "@test_fnc_v1.blk0.cmp_res_1");
    id_224 = bldr_18->gen_sym(bldr_18, "@test_fnc_v1.blk0.cmp_res_2");
    id_225 = bldr_18->gen_sym(bldr_18, "@test_fnc_v1.blk0.bin_res");
    id_226 = bldr_18->gen_sym(bldr_18, "@test_fnc_v1.blk0.res");
    id_227 = bldr_18->gen_sym(bldr_18, NULL);
    bldr_18->new_cmp(bldr_18, id_227, id_223, MU_CMP_ULT, id_216, id_217, id_218);
    id_228 = bldr_18->gen_sym(bldr_18, NULL);
    bldr_18->new_cmp(bldr_18, id_228, id_224, MU_CMP_ULT, id_216, id_217, id_217);
    id_229 = bldr_18->gen_sym(bldr_18, NULL);
    bldr_18->new_binop(bldr_18, id_229, id_225, MU_BINOP_OR, id_215, id_223, id_224, MU_NO_ID);
    id_230 = bldr_18->gen_sym(bldr_18, NULL);
    bldr_18->new_conv(bldr_18, id_230, id_226, MU_CONV_ZEXT, id_215, id_216, id_225);
    id_231 = bldr_18->gen_sym(bldr_18, NULL);
    bldr_18->new_ret(bldr_18, id_231, (MuVarNode [1]){id_226}, 1);
    bldr_18->new_bb(bldr_18, id_222, NULL, NULL, 0, MU_NO_ID, (MuInstNode [5]){id_227, id_228, id_229, id_230, id_231}, 5);
    bldr_18->new_func_ver(bldr_18, id_221, id_220, (MuBBNode [1]){id_222}, 1);
    bldr_18->load(bldr_18);
    mu_18->compile_to_sharedlib(mu_18, "test_ult.dylib", NULL, 0);
    printf("%s\n", "test_ult.dylib");
    return 0;
}
