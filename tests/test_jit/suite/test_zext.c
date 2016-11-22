
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_25;
    MuCtx* ctx_25;
    MuIRBuilder* bldr_25;
    MuID id_322;
    MuID id_323;
    MuID id_324;
    MuID id_325;
    MuID id_326;
    MuID id_327;
    MuID id_328;
    MuID id_329;
    MuID id_330;
    MuID id_331;
    mu_25 = mu_fastimpl_new_with_opts("init_muinit_mu --log-level=none --aot-emit-dir=emit");
    ctx_25 = mu_25->new_context(mu_25);
    bldr_25 = ctx_25->new_ir_builder(ctx_25);
    id_322 = bldr_25->gen_sym(bldr_25, "@i32");
    bldr_25->new_type_int(bldr_25, id_322, 32);
    id_323 = bldr_25->gen_sym(bldr_25, "@i64");
    bldr_25->new_type_int(bldr_25, id_323, 64);
    id_324 = bldr_25->gen_sym(bldr_25, "@0xa8324b55_i32");
    bldr_25->new_const_int(bldr_25, id_324, id_322, 2821868373);
    id_325 = bldr_25->gen_sym(bldr_25, "@sig__i64");
    bldr_25->new_funcsig(bldr_25, id_325, NULL, 0, (MuTypeNode [1]){id_323}, 1);
    id_326 = bldr_25->gen_sym(bldr_25, "@test_fnc");
    bldr_25->new_func(bldr_25, id_326, id_325);
    id_327 = bldr_25->gen_sym(bldr_25, "@test_fnc_v1");
    id_328 = bldr_25->gen_sym(bldr_25, "@test_fnc_v1.blk0");
    id_329 = bldr_25->gen_sym(bldr_25, "@test_fnc_v1.blk0.res");
    id_330 = bldr_25->gen_sym(bldr_25, NULL);
    bldr_25->new_conv(bldr_25, id_330, id_329, MU_CONV_ZEXT, id_322, id_323, id_324);
    id_331 = bldr_25->gen_sym(bldr_25, NULL);
    bldr_25->new_ret(bldr_25, id_331, (MuVarNode [1]){id_329}, 1);
    bldr_25->new_bb(bldr_25, id_328, NULL, NULL, 0, MU_NO_ID, (MuInstNode [2]){id_330, id_331}, 2);
    bldr_25->new_func_ver(bldr_25, id_327, id_326, (MuBBNode [1]){id_328}, 1);
    bldr_25->load(bldr_25);
    mu_25->compile_to_sharedlib(mu_25, "test_zext.dylib", NULL, 0);
    printf("%s\n", "test_zext.dylib");
    return 0;
}
