
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_21;
    MuCtx* ctx_21;
    MuIRBuilder* bldr_21;
    MuID id_252;
    MuID id_253;
    MuID id_254;
    MuID id_255;
    MuID id_256;
    MuID id_257;
    MuID id_258;
    MuID id_259;
    MuID id_260;
    MuID id_261;
    mu_21 = mu_fastimpl_new();
    ctx_21 = mu_21->new_context(mu_21);
    bldr_21 = ctx_21->new_ir_builder(ctx_21);
    id_252 = bldr_21->gen_sym(bldr_21, "@i32");
    bldr_21->new_type_int(bldr_21, id_252, 32);
    id_253 = bldr_21->gen_sym(bldr_21, "@i64");
    bldr_21->new_type_int(bldr_21, id_253, 64);
    id_254 = bldr_21->gen_sym(bldr_21, "@0xa8324b55_i32");
    bldr_21->new_const_int(bldr_21, id_254, id_253, 2821868373);
    id_255 = bldr_21->gen_sym(bldr_21, "@sig__i64");
    bldr_21->new_funcsig(bldr_21, id_255, NULL, 0, (MuTypeNode [1]){id_253}, 1);
    id_256 = bldr_21->gen_sym(bldr_21, "@test_fnc");
    bldr_21->new_func(bldr_21, id_256, id_255);
    id_257 = bldr_21->gen_sym(bldr_21, "@test_fnc_v1");
    id_258 = bldr_21->gen_sym(bldr_21, "@test_fnc_v1.blk0");
    id_259 = bldr_21->gen_sym(bldr_21, "@test_fnc_v1.blk0.res");
    id_260 = bldr_21->gen_sym(bldr_21, NULL);
    bldr_21->new_conv(bldr_21, id_260, id_259, MU_CONV_ZEXT, id_252, id_253, id_254);
    id_261 = bldr_21->gen_sym(bldr_21, NULL);
    bldr_21->new_ret(bldr_21, id_261, (MuVarNode [1]){id_259}, 1);
    bldr_21->new_bb(bldr_21, id_258, NULL, NULL, 0, MU_NO_ID, (MuInstNode [2]){id_260, id_261}, 2);
    bldr_21->new_func_ver(bldr_21, id_257, id_256, (MuBBNode [1]){id_258}, 1);
    bldr_21->load(bldr_21);
    mu_21->compile_to_sharedlib(mu_21, "test_zext.dylib", NULL, 0);
    printf("%s\n", "test_zext.dylib");
    return 0;
}
