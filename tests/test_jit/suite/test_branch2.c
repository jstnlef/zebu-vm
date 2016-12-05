
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
    MuVM* mu_25;
    MuCtx* ctx_25;
    MuIRBuilder* bldr_25;
    MuID id_304;
    MuID id_305;
    MuID id_306;
    MuID id_307;
    MuID id_308;
    MuID id_309;
    MuID id_310;
    MuID id_311;
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
    MuID id_322;
    MuID id_323;
    MuID id_324;
    MuID id_325;
    MuID id_326;
    MuID id_327;
    MuID id_328;
    MuID id_329;
    MuID id_330;
    mu_25 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_25 = mu_25->new_context(mu_25);
    bldr_25 = ctx_25->new_ir_builder(ctx_25);
    id_304 = bldr_25->gen_sym(bldr_25, "@i8");
    bldr_25->new_type_int(bldr_25, id_304, 0x00000008ull);
    id_305 = bldr_25->gen_sym(bldr_25, "@i64");
    bldr_25->new_type_int(bldr_25, id_305, 0x00000040ull);
    id_306 = bldr_25->gen_sym(bldr_25, "@TRUE");
    bldr_25->new_const_int(bldr_25, id_306, id_304, 0x0000000000000001ull);
    id_307 = bldr_25->gen_sym(bldr_25, "@10_i64");
    bldr_25->new_const_int(bldr_25, id_307, id_305, 0x000000000000000aull);
    id_308 = bldr_25->gen_sym(bldr_25, "@20_i64");
    bldr_25->new_const_int(bldr_25, id_308, id_305, 0x0000000000000014ull);
    id_309 = bldr_25->gen_sym(bldr_25, "@sig_i8_i64");
    bldr_25->new_funcsig(bldr_25, id_309, (MuTypeNode [1]){id_304}, 1, (MuTypeNode [1]){id_305}, 1);
    id_310 = bldr_25->gen_sym(bldr_25, "@test_fnc");
    bldr_25->new_func(bldr_25, id_310, id_309);
    id_311 = bldr_25->gen_sym(bldr_25, "@test_fnc.v1");
    id_312 = bldr_25->gen_sym(bldr_25, "@test_fnc.v1.blk0");
    id_313 = bldr_25->gen_sym(bldr_25, "@test_fnc.v1.blk1");
    id_314 = bldr_25->gen_sym(bldr_25, "@test_fnc.v1.blk2");
    id_315 = bldr_25->gen_sym(bldr_25, "@test_fnc.v1.blk0.sel");
    id_316 = bldr_25->gen_sym(bldr_25, "@test_fnc.v1.blk0.flag");
    id_317 = bldr_25->gen_sym(bldr_25, NULL);
    bldr_25->new_cmp(bldr_25, id_317, id_316, MU_CMP_EQ, id_304, id_315, id_306);
    id_318 = bldr_25->gen_sym(bldr_25, NULL);
    id_319 = bldr_25->gen_sym(bldr_25, NULL);
    bldr_25->new_dest_clause(bldr_25, id_319, id_313, (MuVarNode [2]){id_307, id_308}, 2);
    id_320 = bldr_25->gen_sym(bldr_25, NULL);
    bldr_25->new_dest_clause(bldr_25, id_320, id_314, (MuVarNode [2]){id_307, id_308}, 2);
    bldr_25->new_branch2(bldr_25, id_318, id_316, id_319, id_320);
    bldr_25->new_bb(bldr_25, id_312, (MuID [1]){id_315}, (MuTypeNode [1]){id_304}, 1, MU_NO_ID, (MuInstNode [2]){id_317, id_318}, 2);
    id_321 = bldr_25->gen_sym(bldr_25, "@test_fnc.v1.blk1.a");
    id_322 = bldr_25->gen_sym(bldr_25, "@test_fnc.v1.blk1.b");
    id_323 = bldr_25->gen_sym(bldr_25, "@test_fnc.v1.blk1.res");
    id_324 = bldr_25->gen_sym(bldr_25, NULL);
    bldr_25->new_binop(bldr_25, id_324, id_323, MU_BINOP_ADD, id_305, id_321, id_322, MU_NO_ID);
    id_325 = bldr_25->gen_sym(bldr_25, NULL);
    bldr_25->new_ret(bldr_25, id_325, (MuVarNode [1]){id_323}, 1);
    bldr_25->new_bb(bldr_25, id_313, (MuID [2]){id_321, id_322}, (MuTypeNode [2]){id_305, id_305}, 2, MU_NO_ID, (MuInstNode [2]){id_324, id_325}, 2);
    id_326 = bldr_25->gen_sym(bldr_25, "@test_fnc.v1.blk2.a");
    id_327 = bldr_25->gen_sym(bldr_25, "@test_fnc.v1.blk2.b");
    id_328 = bldr_25->gen_sym(bldr_25, "@test_fnc.v1.blk2.res");
    id_329 = bldr_25->gen_sym(bldr_25, NULL);
    bldr_25->new_binop(bldr_25, id_329, id_328, MU_BINOP_MUL, id_305, id_326, id_327, MU_NO_ID);
    id_330 = bldr_25->gen_sym(bldr_25, NULL);
    bldr_25->new_ret(bldr_25, id_330, (MuVarNode [1]){id_328}, 1);
    bldr_25->new_bb(bldr_25, id_314, (MuID [2]){id_326, id_327}, (MuTypeNode [2]){id_305, id_305}, 2, MU_NO_ID, (MuInstNode [2]){id_329, id_330}, 2);
    bldr_25->new_func_ver(bldr_25, id_311, id_310, (MuBBNode [3]){id_312, id_313, id_314}, 3);
    bldr_25->load(bldr_25);
    mu_25->compile_to_sharedlib(mu_25, LIB_FILE_NAME("test_branch2"), NULL, 0);
    return 0;
}
