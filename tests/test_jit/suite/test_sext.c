
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_20;
    MuCtx* ctx_20;
    MuIRBuilder* bldr_20;
    MuID id_242;
    MuID id_243;
    MuID id_244;
    MuID id_245;
    MuID id_246;
    MuID id_247;
    MuID id_248;
    MuID id_249;
    MuID id_250;
    MuID id_251;
    mu_20 = mu_fastimpl_new();
    ctx_20 = mu_20->new_context(mu_20);
    bldr_20 = ctx_20->new_ir_builder(ctx_20);
    id_242 = bldr_20->gen_sym(bldr_20, "@i32");
    bldr_20->new_type_int(bldr_20, id_242, 32);
    id_243 = bldr_20->gen_sym(bldr_20, "@i64");
    bldr_20->new_type_int(bldr_20, id_243, 64);
    id_244 = bldr_20->gen_sym(bldr_20, "@0xa8324b55_i32");
    bldr_20->new_const_int(bldr_20, id_244, id_243, 2821868373);
    id_245 = bldr_20->gen_sym(bldr_20, "@sig__i64");
    bldr_20->new_funcsig(bldr_20, id_245, NULL, 0, (MuTypeNode [1]){id_243}, 1);
    id_246 = bldr_20->gen_sym(bldr_20, "@test_fnc");
    bldr_20->new_func(bldr_20, id_246, id_245);
    id_247 = bldr_20->gen_sym(bldr_20, "@test_fnc_v1");
    id_248 = bldr_20->gen_sym(bldr_20, "@test_fnc_v1.blk0");
    id_249 = bldr_20->gen_sym(bldr_20, "@test_fnc_v1.blk0.res");
    id_250 = bldr_20->gen_sym(bldr_20, NULL);
    bldr_20->new_conv(bldr_20, id_250, id_249, MU_CONV_SEXT, id_242, id_243, id_244);
    id_251 = bldr_20->gen_sym(bldr_20, NULL);
    bldr_20->new_ret(bldr_20, id_251, (MuVarNode [1]){id_249}, 1);
    bldr_20->new_bb(bldr_20, id_248, NULL, NULL, 0, MU_NO_ID, (MuInstNode [2]){id_250, id_251}, 2);
    bldr_20->new_func_ver(bldr_20, id_247, id_246, (MuBBNode [1]){id_248}, 1);
    bldr_20->load(bldr_20);
    mu_20->compile_to_sharedlib(mu_20, "test_sext.dylib", NULL, 0);
    printf("%s\n", "test_sext.dylib");
    return 0;
}
