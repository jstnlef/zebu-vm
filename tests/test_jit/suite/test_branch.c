
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
    MuVM* mu_24;
    MuCtx* ctx_24;
    MuIRBuilder* bldr_24;
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
    mu_24 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_24 = mu_24->new_context(mu_24);
    bldr_24 = ctx_24->new_ir_builder(ctx_24);
    id_289 = bldr_24->gen_sym(bldr_24, "@i64");
    bldr_24->new_type_int(bldr_24, id_289, 0x00000040ull);
    id_290 = bldr_24->gen_sym(bldr_24, "@10_i64");
    bldr_24->new_const_int(bldr_24, id_290, id_289, 0x000000000000000aull);
    id_291 = bldr_24->gen_sym(bldr_24, "@20_i64");
    bldr_24->new_const_int(bldr_24, id_291, id_289, 0x0000000000000014ull);
    id_292 = bldr_24->gen_sym(bldr_24, "@sig__i64");
    bldr_24->new_funcsig(bldr_24, id_292, NULL, 0, (MuTypeNode [1]){id_289}, 1);
    id_293 = bldr_24->gen_sym(bldr_24, "@test_fnc");
    bldr_24->new_func(bldr_24, id_293, id_292);
    id_294 = bldr_24->gen_sym(bldr_24, "@test_fnc.v1");
    id_295 = bldr_24->gen_sym(bldr_24, "@test_fnc.v1.blk0");
    id_296 = bldr_24->gen_sym(bldr_24, "@test_fnc.v1.blk1");
    id_297 = bldr_24->gen_sym(bldr_24, NULL);
    id_298 = bldr_24->gen_sym(bldr_24, NULL);
    bldr_24->new_dest_clause(bldr_24, id_298, id_296, (MuVarNode [2]){id_290, id_291}, 2);
    bldr_24->new_branch(bldr_24, id_297, id_298);
    bldr_24->new_bb(bldr_24, id_295, NULL, NULL, 0, MU_NO_ID, (MuInstNode [1]){id_297}, 1);
    id_299 = bldr_24->gen_sym(bldr_24, "@test_fnc.v1.blk1.a");
    id_300 = bldr_24->gen_sym(bldr_24, "@test_fnc.v1.blk1.b");
    id_301 = bldr_24->gen_sym(bldr_24, "@test_fnc.v1.blk1.res");
    id_302 = bldr_24->gen_sym(bldr_24, NULL);
    bldr_24->new_binop(bldr_24, id_302, id_301, MU_BINOP_ADD, id_289, id_299, id_300, MU_NO_ID);
    id_303 = bldr_24->gen_sym(bldr_24, NULL);
    bldr_24->new_ret(bldr_24, id_303, (MuVarNode [1]){id_301}, 1);
    bldr_24->new_bb(bldr_24, id_296, (MuID [2]){id_299, id_300}, (MuTypeNode [2]){id_289, id_289}, 2, MU_NO_ID, (MuInstNode [2]){id_302, id_303}, 2);
    bldr_24->new_func_ver(bldr_24, id_294, id_293, (MuBBNode [2]){id_295, id_296}, 2);
    bldr_24->load(bldr_24);
    mu_24->compile_to_sharedlib(mu_24, LIB_FILE_NAME("test_branch"), NULL, 0);
    return 0;
}
