
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_19;
    MuCtx* ctx_19;
    MuIRBuilder* bldr_19;
    MuID id_232;
    MuID id_233;
    MuID id_234;
    MuID id_235;
    MuID id_236;
    MuID id_237;
    MuID id_238;
    MuID id_239;
    MuID id_240;
    MuID id_241;
    mu_19 = mu_fastimpl_new();
    ctx_19 = mu_19->new_context(mu_19);
    bldr_19 = ctx_19->new_ir_builder(ctx_19);
    id_232 = bldr_19->gen_sym(bldr_19, "@i32");
    bldr_19->new_type_int(bldr_19, id_232, 32);
    id_233 = bldr_19->gen_sym(bldr_19, "@i64");
    bldr_19->new_type_int(bldr_19, id_233, 64);
    id_234 = bldr_19->gen_sym(bldr_19, "@0x6d9f9c1d58324b55_i64");
    bldr_19->new_const_int(bldr_19, id_234, id_233, 7899203921278815061);
    id_235 = bldr_19->gen_sym(bldr_19, "@sig__i32");
    bldr_19->new_funcsig(bldr_19, id_235, NULL, 0, (MuTypeNode [1]){id_232}, 1);
    id_236 = bldr_19->gen_sym(bldr_19, "@test_fnc");
    bldr_19->new_func(bldr_19, id_236, id_235);
    id_237 = bldr_19->gen_sym(bldr_19, "@test_fnc_v1");
    id_238 = bldr_19->gen_sym(bldr_19, "@test_fnc_v1.blk0");
    id_239 = bldr_19->gen_sym(bldr_19, "@test_fnc_v1.blk0.res");
    id_240 = bldr_19->gen_sym(bldr_19, NULL);
    bldr_19->new_conv(bldr_19, id_240, id_239, MU_CONV_TRUNC, id_233, id_232, id_234);
    id_241 = bldr_19->gen_sym(bldr_19, NULL);
    bldr_19->new_ret(bldr_19, id_241, (MuVarNode [1]){id_239}, 1);
    bldr_19->new_bb(bldr_19, id_238, NULL, NULL, 0, MU_NO_ID, (MuInstNode [2]){id_240, id_241}, 2);
    bldr_19->new_func_ver(bldr_19, id_237, id_236, (MuBBNode [1]){id_238}, 1);
    bldr_19->load(bldr_19);
    mu_19->compile_to_sharedlib(mu_19, "test_trunc.dylib", NULL, 0);
    printf("%s\n", "test_trunc.dylib");
    return 0;
}
