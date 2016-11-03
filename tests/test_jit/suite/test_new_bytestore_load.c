
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
    MuID id_277;
    MuID id_278;
    MuID id_279;
    MuID id_280;
    MuID id_281;
    MuID id_282;
    MuID id_283;
    MuID id_284;
    MuID id_285;
    MuID id_286;
    MuID id_287;
    MuID id_288;
    MuID id_289;
    MuID id_290;
    MuID id_291;
    MuID id_292;
    MuID id_293;
    MuID id_294;
    MuID id_295;
    MuID id_296;
    MuID id_297;
    MuID id_298;
    MuID id_299;
    MuID id_300;
    MuID id_301;
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
    MuID id_312;
    mu_23 = mu_fastimpl_new();
    ctx_23 = mu_23->new_context(mu_23);
    bldr_23 = ctx_23->new_ir_builder(ctx_23);
    id_277 = bldr_23->gen_sym(bldr_23, "@i8");
    bldr_23->new_type_int(bldr_23, id_277, 8);
    id_278 = bldr_23->gen_sym(bldr_23, "@i32");
    bldr_23->new_type_int(bldr_23, id_278, 32);
    id_279 = bldr_23->gen_sym(bldr_23, "@refi8");
    bldr_23->new_type_ref(bldr_23, id_279, id_277);
    id_280 = bldr_23->gen_sym(bldr_23, "@irefi8");
    bldr_23->new_type_iref(bldr_23, id_280, id_277);
    id_281 = bldr_23->gen_sym(bldr_23, "@refi32");
    bldr_23->new_type_ref(bldr_23, id_281, id_278);
    id_282 = bldr_23->gen_sym(bldr_23, "@iref32");
    bldr_23->new_type_iref(bldr_23, id_282, id_278);
    id_283 = bldr_23->gen_sym(bldr_23, "@1_i8");
    bldr_23->new_const_int(bldr_23, id_283, id_277, 1);
    id_284 = bldr_23->gen_sym(bldr_23, "@0x8d_i8");
    bldr_23->new_const_int(bldr_23, id_284, id_277, 141);
    id_285 = bldr_23->gen_sym(bldr_23, "@0x9f_i8");
    bldr_23->new_const_int(bldr_23, id_285, id_277, 159);
    id_286 = bldr_23->gen_sym(bldr_23, "@0x9c_i8");
    bldr_23->new_const_int(bldr_23, id_286, id_277, 156);
    id_287 = bldr_23->gen_sym(bldr_23, "@0x1d_i8");
    bldr_23->new_const_int(bldr_23, id_287, id_277, 29);
    id_288 = bldr_23->gen_sym(bldr_23, "@sig__i32");
    bldr_23->new_funcsig(bldr_23, id_288, NULL, 0, (MuTypeNode [1]){id_278}, 1);
    id_289 = bldr_23->gen_sym(bldr_23, "@test_fnc");
    bldr_23->new_func(bldr_23, id_289, id_288);
    id_290 = bldr_23->gen_sym(bldr_23, "@test_fnc.v1");
    id_291 = bldr_23->gen_sym(bldr_23, "@test_fnc.v1.blk0");
    id_292 = bldr_23->gen_sym(bldr_23, "@test_fnc.v1.blk0.r32x");
    id_293 = bldr_23->gen_sym(bldr_23, "@test_fnc.v1.blk0.r8x");
    id_294 = bldr_23->gen_sym(bldr_23, "@test_fnc.v1.blk0.ir8x_0");
    id_295 = bldr_23->gen_sym(bldr_23, "@test_fnc.v1.blk0.ir8x_1");
    id_296 = bldr_23->gen_sym(bldr_23, "@test_fnc.v1.blk0.ir8x_2");
    id_297 = bldr_23->gen_sym(bldr_23, "@test_fnc.v1.blk0.ir8x_3");
    id_298 = bldr_23->gen_sym(bldr_23, "@test_fnc.v1.blk0.ir32x");
    id_299 = bldr_23->gen_sym(bldr_23, "@test_fnc.v1.blk0.res");
    id_300 = bldr_23->gen_sym(bldr_23, NULL);
    bldr_23->new_new(bldr_23, id_300, id_292, id_278, MU_NO_ID);
    id_301 = bldr_23->gen_sym(bldr_23, NULL);
    bldr_23->new_conv(bldr_23, id_301, id_293, MU_CONV_REFCAST, id_281, id_279, id_292);
    id_302 = bldr_23->gen_sym(bldr_23, NULL);
    bldr_23->new_getiref(bldr_23, id_302, id_294, id_277, id_293);
    id_303 = bldr_23->gen_sym(bldr_23, NULL);
    bldr_23->new_store(bldr_23, id_303, false, MU_ORD_NOT_ATOMIC, id_277, id_294, id_287, MU_NO_ID);
    id_304 = bldr_23->gen_sym(bldr_23, NULL);
    bldr_23->new_shiftiref(bldr_23, id_304, id_295, false, id_277, id_277, id_294, id_283);
    id_305 = bldr_23->gen_sym(bldr_23, NULL);
    bldr_23->new_store(bldr_23, id_305, false, MU_ORD_NOT_ATOMIC, id_277, id_295, id_286, MU_NO_ID);
    id_306 = bldr_23->gen_sym(bldr_23, NULL);
    bldr_23->new_shiftiref(bldr_23, id_306, id_296, false, id_277, id_277, id_295, id_283);
    id_307 = bldr_23->gen_sym(bldr_23, NULL);
    bldr_23->new_store(bldr_23, id_307, false, MU_ORD_NOT_ATOMIC, id_277, id_296, id_285, MU_NO_ID);
    id_308 = bldr_23->gen_sym(bldr_23, NULL);
    bldr_23->new_shiftiref(bldr_23, id_308, id_297, false, id_277, id_277, id_296, id_283);
    id_309 = bldr_23->gen_sym(bldr_23, NULL);
    bldr_23->new_store(bldr_23, id_309, false, MU_ORD_NOT_ATOMIC, id_277, id_297, id_284, MU_NO_ID);
    id_310 = bldr_23->gen_sym(bldr_23, NULL);
    bldr_23->new_getiref(bldr_23, id_310, id_298, id_278, id_292);
    id_311 = bldr_23->gen_sym(bldr_23, NULL);
    bldr_23->new_load(bldr_23, id_311, id_299, false, MU_ORD_NOT_ATOMIC, id_278, id_298, MU_NO_ID);
    id_312 = bldr_23->gen_sym(bldr_23, NULL);
    bldr_23->new_ret(bldr_23, id_312, (MuVarNode [1]){id_299}, 1);
    bldr_23->new_bb(bldr_23, id_291, NULL, NULL, 0, MU_NO_ID, (MuInstNode [13]){id_300, id_301, id_302, id_303, id_304, id_305, id_306, id_307, id_308, id_309, id_310, id_311, id_312}, 13);
    bldr_23->new_func_ver(bldr_23, id_290, id_289, (MuBBNode [1]){id_291}, 1);
    bldr_23->load(bldr_23);
    mu_23->compile_to_sharedlib(mu_23, "test_new_bytestore_load.dylib", NULL, 0);
    printf("%s\n", "test_new_bytestore_load.dylib");
    return 0;
}
