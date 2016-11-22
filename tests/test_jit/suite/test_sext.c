
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_24;
    MuCtx* ctx_24;
    MuIRBuilder* bldr_24;
    MuID id_312;
    MuID id_313;
    MuID id_314;
    MuID id_315;
    MuID id_316;
    MuID id_317;
    MuID id_318;
    MuID id_319;
    MuID id_320;
    MuID id_321;
    mu_24 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_24 = mu_24->new_context(mu_24);
    bldr_24 = ctx_24->new_ir_builder(ctx_24);
    id_312 = bldr_24->gen_sym(bldr_24, "@i32");
    bldr_24->new_type_int(bldr_24, id_312, 32);
    id_313 = bldr_24->gen_sym(bldr_24, "@i64");
    bldr_24->new_type_int(bldr_24, id_313, 64);
    id_314 = bldr_24->gen_sym(bldr_24, "@0xa8324b55_i32");
    bldr_24->new_const_int(bldr_24, id_314, id_312, 2821868373);
    id_315 = bldr_24->gen_sym(bldr_24, "@sig__i64");
    bldr_24->new_funcsig(bldr_24, id_315, NULL, 0, (MuTypeNode [1]){id_313}, 1);
    id_316 = bldr_24->gen_sym(bldr_24, "@test_fnc");
    bldr_24->new_func(bldr_24, id_316, id_315);
    id_317 = bldr_24->gen_sym(bldr_24, "@test_fnc_v1");
    id_318 = bldr_24->gen_sym(bldr_24, "@test_fnc_v1.blk0");
    id_319 = bldr_24->gen_sym(bldr_24, "@test_fnc_v1.blk0.res");
    id_320 = bldr_24->gen_sym(bldr_24, NULL);
    bldr_24->new_conv(bldr_24, id_320, id_319, MU_CONV_SEXT, id_312, id_313, id_314);
    id_321 = bldr_24->gen_sym(bldr_24, NULL);
    bldr_24->new_ret(bldr_24, id_321, (MuVarNode [1]){id_319}, 1);
    bldr_24->new_bb(bldr_24, id_318, NULL, NULL, 0, MU_NO_ID, (MuInstNode [2]){id_320, id_321}, 2);
    bldr_24->new_func_ver(bldr_24, id_317, id_316, (MuBBNode [1]){id_318}, 1);
    bldr_24->load(bldr_24);
    mu_24->compile_to_sharedlib(mu_24, "test_sext.dylib", NULL, 0);
    printf("%s\n", "test_sext.dylib");
    return 0;
}
