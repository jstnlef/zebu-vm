
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_26;
    MuCtx* ctx_26;
    MuIRBuilder* bldr_26;
    MuID id_355;
    MuID id_356;
    MuID id_357;
    MuID id_358;
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
    MuID id_369;
    MuID id_370;
    MuID id_371;
    MuID id_372;
    MuID id_373;
    MuID id_374;
    MuID id_375;
    MuID id_376;
    MuID id_377;
    MuID id_378;
    MuID id_379;
    MuID id_380;
    MuID id_381;
    MuID id_382;
    MuID id_383;
    mu_26 = mu_fastimpl_new();
    ctx_26 = mu_26->new_context(mu_26);
    bldr_26 = ctx_26->new_ir_builder(ctx_26);
    id_355 = bldr_26->gen_sym(bldr_26, "@i8");
    bldr_26->new_type_int(bldr_26, id_355, 8);
    id_356 = bldr_26->gen_sym(bldr_26, "@i32");
    bldr_26->new_type_int(bldr_26, id_356, 32);
    id_357 = bldr_26->gen_sym(bldr_26, "@pi8");
    bldr_26->new_type_uptr(bldr_26, id_357, 8);
    id_358 = bldr_26->gen_sym(bldr_26, "@pi32");
    bldr_26->new_type_uptr(bldr_26, id_358, 32);
    id_359 = bldr_26->gen_sym(bldr_26, "@1_i8");
    bldr_26->new_const_int(bldr_26, id_359, id_355, 1);
    id_360 = bldr_26->gen_sym(bldr_26, "@0x8d_i8");
    bldr_26->new_const_int(bldr_26, id_360, id_355, 141);
    id_361 = bldr_26->gen_sym(bldr_26, "@0x9f_i8");
    bldr_26->new_const_int(bldr_26, id_361, id_355, 159);
    id_362 = bldr_26->gen_sym(bldr_26, "@0x9c_i8");
    bldr_26->new_const_int(bldr_26, id_362, id_355, 156);
    id_363 = bldr_26->gen_sym(bldr_26, "@0x1d_i8");
    bldr_26->new_const_int(bldr_26, id_363, id_355, 29);
    id_364 = bldr_26->gen_sym(bldr_26, "@sig_pi32_i32");
    bldr_26->new_funcsig(bldr_26, id_364, (MuTypeNode [1]){id_358}, 1, (MuTypeNode [1]){id_356}, 1);
    id_365 = bldr_26->gen_sym(bldr_26, "@test_fnc");
    bldr_26->new_func(bldr_26, id_365, id_364);
    id_366 = bldr_26->gen_sym(bldr_26, "@test_fnc.v1");
    id_367 = bldr_26->gen_sym(bldr_26, "@test_fnc.v1.blk0");
    id_368 = bldr_26->gen_sym(bldr_26, "@test_fnc.v1.blk0.pi32x");
    id_369 = bldr_26->gen_sym(bldr_26, "@test_fnc.v1.blk0.pi8x_0");
    id_370 = bldr_26->gen_sym(bldr_26, "@test_fnc.v1.blk0.pi8x_1");
    id_371 = bldr_26->gen_sym(bldr_26, "@test_fnc.v1.blk0.pi8x_2");
    id_372 = bldr_26->gen_sym(bldr_26, "@test_fnc.v1.blk0.pi8x_3");
    id_373 = bldr_26->gen_sym(bldr_26, "@test_fnc.v1.blk0.res");
    id_374 = bldr_26->gen_sym(bldr_26, NULL);
    bldr_26->new_conv(bldr_26, id_374, id_369, MU_CONV_PTRCAST, id_358, id_357, id_368);
    id_375 = bldr_26->gen_sym(bldr_26, NULL);
    bldr_26->new_store(bldr_26, id_375, true, MU_ORD_NOT_ATOMIC, id_355, id_369, id_363, MU_NO_ID);
    id_376 = bldr_26->gen_sym(bldr_26, NULL);
    bldr_26->new_shiftiref(bldr_26, id_376, id_370, true, id_355, id_355, id_369, id_359);
    id_377 = bldr_26->gen_sym(bldr_26, NULL);
    bldr_26->new_store(bldr_26, id_377, true, MU_ORD_NOT_ATOMIC, id_355, id_370, id_362, MU_NO_ID);
    id_378 = bldr_26->gen_sym(bldr_26, NULL);
    bldr_26->new_shiftiref(bldr_26, id_378, id_371, true, id_355, id_355, id_370, id_359);
    id_379 = bldr_26->gen_sym(bldr_26, NULL);
    bldr_26->new_store(bldr_26, id_379, true, MU_ORD_NOT_ATOMIC, id_355, id_371, id_361, MU_NO_ID);
    id_380 = bldr_26->gen_sym(bldr_26, NULL);
    bldr_26->new_shiftiref(bldr_26, id_380, id_372, true, id_355, id_355, id_371, id_359);
    id_381 = bldr_26->gen_sym(bldr_26, NULL);
    bldr_26->new_store(bldr_26, id_381, true, MU_ORD_NOT_ATOMIC, id_355, id_372, id_360, MU_NO_ID);
    id_382 = bldr_26->gen_sym(bldr_26, NULL);
    bldr_26->new_load(bldr_26, id_382, id_373, true, MU_ORD_NOT_ATOMIC, id_356, id_368, MU_NO_ID);
    id_383 = bldr_26->gen_sym(bldr_26, NULL);
    bldr_26->new_ret(bldr_26, id_383, (MuVarNode [1]){id_373}, 1);
    bldr_26->new_bb(bldr_26, id_367, (MuID [1]){id_368}, (MuTypeNode [1]){id_358}, 1, MU_NO_ID, (MuInstNode [10]){id_374, id_375, id_376, id_377, id_378, id_379, id_380, id_381, id_382, id_383}, 10);
    bldr_26->new_func_ver(bldr_26, id_366, id_365, (MuBBNode [1]){id_367}, 1);
    bldr_26->load(bldr_26);
    mu_26->compile_to_sharedlib(mu_26, "test_uptr_bytestore_load.dylib", (char**){&"entry_test_uptr_bytestore_load.c"}, 1);
    printf("%s\n", "test_uptr_bytestore_load.dylib");
    return 0;
}
