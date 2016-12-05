
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
#ifdef __APPLE__
    #define LIB_EXT ".dylib"
#elif __linux__
    #define LIB_EXT ".so"
#elif _WIN32
    #define LIB_EXT ".dll"
#endif
#define LIB_FILE_NAME(name) "lib" name LIB_EXT
int main(int argc, char** argv) {
    MuVM* mu_28;
    MuCtx* ctx_28;
    MuIRBuilder* bldr_28;
    MuID id_359;
    MuID id_360;
    MuID id_361;
    MuID id_362;
    MuID id_363;
    MuID id_364;
    MuID id_365;
    MuID id_366;
    MuID id_367;
    MuID id_368;
    mu_28 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_28 = mu_28->new_context(mu_28);
    bldr_28 = ctx_28->new_ir_builder(ctx_28);
    id_359 = bldr_28->gen_sym(bldr_28, "@i32");
    bldr_28->new_type_int(bldr_28, id_359, 0x00000020ull);
    id_360 = bldr_28->gen_sym(bldr_28, "@i64");
    bldr_28->new_type_int(bldr_28, id_360, 0x00000040ull);
    id_361 = bldr_28->gen_sym(bldr_28, "@0x6d9f9c1d58324b55_i64");
    bldr_28->new_const_int(bldr_28, id_361, id_360, 0x6d9f9c1d58324b55ull);
    id_362 = bldr_28->gen_sym(bldr_28, "@sig__i32");
    bldr_28->new_funcsig(bldr_28, id_362, NULL, 0, (MuTypeNode [1]){id_359}, 1);
    id_363 = bldr_28->gen_sym(bldr_28, "@test_fnc");
    bldr_28->new_func(bldr_28, id_363, id_362);
    id_364 = bldr_28->gen_sym(bldr_28, "@test_fnc_v1");
    id_365 = bldr_28->gen_sym(bldr_28, "@test_fnc_v1.blk0");
    id_366 = bldr_28->gen_sym(bldr_28, "@test_fnc_v1.blk0.res");
    id_367 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_conv(bldr_28, id_367, id_366, MU_CONV_TRUNC, id_360, id_359, id_361);
    id_368 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_ret(bldr_28, id_368, (MuVarNode [1]){id_366}, 1);
    bldr_28->new_bb(bldr_28, id_365, NULL, NULL, 0, MU_NO_ID, (MuInstNode [2]){id_367, id_368}, 2);
    bldr_28->new_func_ver(bldr_28, id_364, id_363, (MuBBNode [1]){id_365}, 1);
    bldr_28->load(bldr_28);
    mu_28->compile_to_sharedlib(mu_28, LIB_FILE_NAME("test_trunc"), NULL, 0);
    return 0;
}
