
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_23;
    MuCtx* ctx_23;
    MuIRBuilder* bldr_23;
    MuID id_302;
    MuID id_303;
    MuID id_304;
    MuID id_305;
    MuID id_306;
    MuID id_307;
    MuID id_308;
    MuID id_309;
    MuID id_310;
    MuID id_311;
    mu_23 = mu_fastimpl_new_with_opts("init_muinit_mu --log-level=none --aot-emit-dir=emit");
    ctx_23 = mu_23->new_context(mu_23);
    bldr_23 = ctx_23->new_ir_builder(ctx_23);
    id_302 = bldr_23->gen_sym(bldr_23, "@i32");
    bldr_23->new_type_int(bldr_23, id_302, 32);
    id_303 = bldr_23->gen_sym(bldr_23, "@i64");
    bldr_23->new_type_int(bldr_23, id_303, 64);
    id_304 = bldr_23->gen_sym(bldr_23, "@0x6d9f9c1d58324b55_i64");
    bldr_23->new_const_int(bldr_23, id_304, id_303, 7899203921278815061);
    id_305 = bldr_23->gen_sym(bldr_23, "@sig__i32");
    bldr_23->new_funcsig(bldr_23, id_305, NULL, 0, (MuTypeNode [1]){id_302}, 1);
    id_306 = bldr_23->gen_sym(bldr_23, "@test_fnc");
    bldr_23->new_func(bldr_23, id_306, id_305);
    id_307 = bldr_23->gen_sym(bldr_23, "@test_fnc_v1");
    id_308 = bldr_23->gen_sym(bldr_23, "@test_fnc_v1.blk0");
    id_309 = bldr_23->gen_sym(bldr_23, "@test_fnc_v1.blk0.res");
    id_310 = bldr_23->gen_sym(bldr_23, NULL);
    bldr_23->new_conv(bldr_23, id_310, id_309, MU_CONV_TRUNC, id_303, id_302, id_304);
    id_311 = bldr_23->gen_sym(bldr_23, NULL);
    bldr_23->new_ret(bldr_23, id_311, (MuVarNode [1]){id_309}, 1);
    bldr_23->new_bb(bldr_23, id_308, NULL, NULL, 0, MU_NO_ID, (MuInstNode [2]){id_310, id_311}, 2);
    bldr_23->new_func_ver(bldr_23, id_307, id_306, (MuBBNode [1]){id_308}, 1);
    bldr_23->load(bldr_23);
    mu_23->compile_to_sharedlib(mu_23, "test_trunc.dylib", NULL, 0);
    printf("%s\n", "test_trunc.dylib");
    return 0;
}
