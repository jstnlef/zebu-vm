
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
    MuVM* mu_59;
    MuCtx* ctx_59;
    MuIRBuilder* bldr_59;
    MuID id_911;
    MuID id_912;
    MuID id_913;
    MuID id_914;
    MuID id_915;
    MuID id_916;
    MuID id_917;
    MuID id_918;
    MuID id_919;
    MuID id_920;
    MuID id_921;
    MuID id_922;
    MuID id_923;
    MuID id_924;
    MuID id_925;
    MuID id_926;
    mu_59 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_59 = mu_59->new_context(mu_59);
    bldr_59 = ctx_59->new_ir_builder(ctx_59);
    id_911 = bldr_59->gen_sym(bldr_59, "@i1");
    bldr_59->new_type_int(bldr_59, id_911, 0x00000001ull);
    id_912 = bldr_59->gen_sym(bldr_59, "@i8");
    bldr_59->new_type_int(bldr_59, id_912, 0x00000008ull);
    id_913 = bldr_59->gen_sym(bldr_59, "@i64");
    bldr_59->new_type_int(bldr_59, id_913, 0x00000040ull);
    id_914 = bldr_59->gen_sym(bldr_59, "@10_i64");
    bldr_59->new_const_int(bldr_59, id_914, id_913, 0x000000000000000aull);
    id_915 = bldr_59->gen_sym(bldr_59, "@20_i64");
    bldr_59->new_const_int(bldr_59, id_915, id_913, 0x0000000000000014ull);
    id_916 = bldr_59->gen_sym(bldr_59, "@TRUE");
    bldr_59->new_const_int(bldr_59, id_916, id_912, 0x0000000000000001ull);
    id_917 = bldr_59->gen_sym(bldr_59, "@sig_i8_i64");
    bldr_59->new_funcsig(bldr_59, id_917, (MuTypeNode [1]){id_912}, 1, (MuTypeNode [1]){id_913}, 1);
    id_918 = bldr_59->gen_sym(bldr_59, "@test_fnc");
    bldr_59->new_func(bldr_59, id_918, id_917);
    id_919 = bldr_59->gen_sym(bldr_59, "@test_fnc.v1");
    id_920 = bldr_59->gen_sym(bldr_59, "@test_fnc.v1.blk0");
    id_921 = bldr_59->gen_sym(bldr_59, "@test_fnc.v1.blk0.flag");
    id_922 = bldr_59->gen_sym(bldr_59, "@test_fnc.v1.blk0.cmpres");
    id_923 = bldr_59->gen_sym(bldr_59, "@test_fnc.v1.blk0.res");
    id_924 = bldr_59->gen_sym(bldr_59, NULL);
    bldr_59->new_cmp(bldr_59, id_924, id_922, MU_CMP_EQ, id_912, id_921, id_916);
    id_925 = bldr_59->gen_sym(bldr_59, NULL);
    bldr_59->new_select(bldr_59, id_925, id_923, id_911, id_913, id_922, id_914, id_915);
    id_926 = bldr_59->gen_sym(bldr_59, NULL);
    bldr_59->new_ret(bldr_59, id_926, (MuVarNode [1]){id_923}, 1);
    bldr_59->new_bb(bldr_59, id_920, (MuID [1]){id_921}, (MuTypeNode [1]){id_912}, 1, MU_NO_ID, (MuInstNode [3]){id_924, id_925, id_926}, 3);
    bldr_59->new_func_ver(bldr_59, id_919, id_918, (MuBBNode [1]){id_920}, 1);
    bldr_59->load(bldr_59);
    mu_59->compile_to_sharedlib(mu_59, LIB_FILE_NAME("test_select"), NULL, 0);
    return 0;
}
