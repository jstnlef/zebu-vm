
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_32;
    MuCtx* ctx_32;
    MuIRBuilder* bldr_32;
    MuID id_403;
    MuID id_404;
    MuID id_405;
    MuID id_406;
    MuID id_407;
    MuID id_408;
    MuID id_409;
    MuID id_410;
    MuID id_411;
    MuID id_412;
    MuID id_413;
    MuID id_414;
    MuID id_415;
    MuID id_416;
    mu_32 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_32 = mu_32->new_context(mu_32);
    bldr_32 = ctx_32->new_ir_builder(ctx_32);
    id_403 = bldr_32->gen_sym(bldr_32, "@dbl");
    bldr_32->new_type_double(bldr_32, id_403);
    id_404 = bldr_32->gen_sym(bldr_32, "@i1");
    bldr_32->new_type_int(bldr_32, id_404, 1);
    id_405 = bldr_32->gen_sym(bldr_32, "@i64");
    bldr_32->new_type_int(bldr_32, id_405, 64);
    id_406 = bldr_32->gen_sym(bldr_32, "@pi");
    bldr_32->new_const_double(bldr_32, id_406, id_403, 3.14159299999999985786);
    id_407 = bldr_32->gen_sym(bldr_32, "@e");
    bldr_32->new_const_double(bldr_32, id_407, id_403, 2.71828000000000002956);
    id_408 = bldr_32->gen_sym(bldr_32, "@sig__i64");
    bldr_32->new_funcsig(bldr_32, id_408, NULL, 0, (MuTypeNode [1]){id_405}, 1);
    id_409 = bldr_32->gen_sym(bldr_32, "@test_fnc");
    bldr_32->new_func(bldr_32, id_409, id_408);
    id_410 = bldr_32->gen_sym(bldr_32, "@test_fnc.v1");
    id_411 = bldr_32->gen_sym(bldr_32, "@test_fnc.v1.blk0");
    id_412 = bldr_32->gen_sym(bldr_32, "@test_fnc.v1.blk0.cmpres");
    id_413 = bldr_32->gen_sym(bldr_32, "@test_fnc.v1.blk0.res");
    id_414 = bldr_32->gen_sym(bldr_32, NULL);
    bldr_32->new_cmp(bldr_32, id_414, id_412, MU_CMP_FOLT, id_403, id_407, id_406);
    id_415 = bldr_32->gen_sym(bldr_32, NULL);
    bldr_32->new_conv(bldr_32, id_415, id_413, MU_CONV_ZEXT, id_404, id_405, id_412);
    id_416 = bldr_32->gen_sym(bldr_32, NULL);
    bldr_32->new_ret(bldr_32, id_416, (MuVarNode [1]){id_413}, 1);
    bldr_32->new_bb(bldr_32, id_411, NULL, NULL, 0, MU_NO_ID, (MuInstNode [3]){id_414, id_415, id_416}, 3);
    bldr_32->new_func_ver(bldr_32, id_410, id_409, (MuBBNode [1]){id_411}, 1);
    bldr_32->load(bldr_32);
    mu_32->compile_to_sharedlib(mu_32, "test_double_ordered_lt.dylib", NULL, 0);
    printf("%s\n", "test_double_ordered_lt.dylib");
    return 0;
}
