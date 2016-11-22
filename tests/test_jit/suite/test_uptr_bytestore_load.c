
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_28;
    MuCtx* ctx_28;
    MuIRBuilder* bldr_28;
    MuID id_383;
    MuID id_384;
    MuID id_385;
    MuID id_386;
    MuID id_387;
    MuID id_388;
    MuID id_389;
    MuID id_390;
    MuID id_391;
    MuID id_392;
    MuID id_393;
    MuID id_394;
    MuID id_395;
    MuID id_396;
    MuID id_397;
    MuID id_398;
    MuID id_399;
    MuID id_400;
    MuID id_401;
    MuID id_402;
    MuID id_403;
    MuID id_404;
    MuID id_405;
    MuID id_406;
    MuID id_407;
    MuID id_408;
    MuID id_409;
    MuID id_410;
    MuID id_411;
    mu_28 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_28 = mu_28->new_context(mu_28);
    bldr_28 = ctx_28->new_ir_builder(ctx_28);
    id_383 = bldr_28->gen_sym(bldr_28, "@i8");
    bldr_28->new_type_int(bldr_28, id_383, 8);
    id_384 = bldr_28->gen_sym(bldr_28, "@i32");
    bldr_28->new_type_int(bldr_28, id_384, 32);
    id_385 = bldr_28->gen_sym(bldr_28, "@pi8");
    bldr_28->new_type_uptr(bldr_28, id_385, id_383);
    id_386 = bldr_28->gen_sym(bldr_28, "@pi32");
    bldr_28->new_type_uptr(bldr_28, id_386, id_384);
    id_387 = bldr_28->gen_sym(bldr_28, "@1_i8");
    bldr_28->new_const_int(bldr_28, id_387, id_383, 1);
    id_388 = bldr_28->gen_sym(bldr_28, "@0x8d_i8");
    bldr_28->new_const_int(bldr_28, id_388, id_383, 141);
    id_389 = bldr_28->gen_sym(bldr_28, "@0x9f_i8");
    bldr_28->new_const_int(bldr_28, id_389, id_383, 159);
    id_390 = bldr_28->gen_sym(bldr_28, "@0x9c_i8");
    bldr_28->new_const_int(bldr_28, id_390, id_383, 156);
    id_391 = bldr_28->gen_sym(bldr_28, "@0x1d_i8");
    bldr_28->new_const_int(bldr_28, id_391, id_383, 29);
    id_392 = bldr_28->gen_sym(bldr_28, "@sig_pi32_i32");
    bldr_28->new_funcsig(bldr_28, id_392, (MuTypeNode [1]){id_386}, 1, (MuTypeNode [1]){id_384}, 1);
    id_393 = bldr_28->gen_sym(bldr_28, "@test_fnc");
    bldr_28->new_func(bldr_28, id_393, id_392);
    id_394 = bldr_28->gen_sym(bldr_28, "@test_fnc.v1");
    id_395 = bldr_28->gen_sym(bldr_28, "@test_fnc.v1.blk0");
    id_396 = bldr_28->gen_sym(bldr_28, "@test_fnc.v1.blk0.pi32x");
    id_397 = bldr_28->gen_sym(bldr_28, "@test_fnc.v1.blk0.pi8x_0");
    id_398 = bldr_28->gen_sym(bldr_28, "@test_fnc.v1.blk0.pi8x_1");
    id_399 = bldr_28->gen_sym(bldr_28, "@test_fnc.v1.blk0.pi8x_2");
    id_400 = bldr_28->gen_sym(bldr_28, "@test_fnc.v1.blk0.pi8x_3");
    id_401 = bldr_28->gen_sym(bldr_28, "@test_fnc.v1.blk0.res");
    id_402 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_conv(bldr_28, id_402, id_397, MU_CONV_PTRCAST, id_386, id_385, id_396);
    id_403 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_store(bldr_28, id_403, true, MU_ORD_NOT_ATOMIC, id_383, id_397, id_391, MU_NO_ID);
    id_404 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_shiftiref(bldr_28, id_404, id_398, true, id_383, id_383, id_397, id_387);
    id_405 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_store(bldr_28, id_405, true, MU_ORD_NOT_ATOMIC, id_383, id_398, id_390, MU_NO_ID);
    id_406 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_shiftiref(bldr_28, id_406, id_399, true, id_383, id_383, id_398, id_387);
    id_407 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_store(bldr_28, id_407, true, MU_ORD_NOT_ATOMIC, id_383, id_399, id_389, MU_NO_ID);
    id_408 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_shiftiref(bldr_28, id_408, id_400, true, id_383, id_383, id_399, id_387);
    id_409 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_store(bldr_28, id_409, true, MU_ORD_NOT_ATOMIC, id_383, id_400, id_388, MU_NO_ID);
    id_410 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_load(bldr_28, id_410, id_401, true, MU_ORD_NOT_ATOMIC, id_384, id_396, MU_NO_ID);
    id_411 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_ret(bldr_28, id_411, (MuVarNode [1]){id_401}, 1);
    bldr_28->new_bb(bldr_28, id_395, (MuID [1]){id_396}, (MuTypeNode [1]){id_386}, 1, MU_NO_ID, (MuInstNode [10]){id_402, id_403, id_404, id_405, id_406, id_407, id_408, id_409, id_410, id_411}, 10);
    bldr_28->new_func_ver(bldr_28, id_394, id_393, (MuBBNode [1]){id_395}, 1);
    bldr_28->load(bldr_28);
    mu_28->compile_to_sharedlib(mu_28, "test_uptr_bytestore_load.dylib", NULL, 0);
    printf("%s\n", "test_uptr_bytestore_load.dylib");
    return 0;
}
