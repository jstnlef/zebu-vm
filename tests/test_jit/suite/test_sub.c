
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
    MuVM* mu_2;
    MuCtx* ctx_2;
    MuIRBuilder* bldr_2;
    MuID id_11;
    MuID id_12;
    MuID id_13;
    MuID id_14;
    MuID id_15;
    MuID id_16;
    MuID id_17;
    MuID id_18;
    MuID id_19;
    MuID id_20;
    mu_2 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_2 = mu_2->new_context(mu_2);
    bldr_2 = ctx_2->new_ir_builder(ctx_2);
    id_11 = bldr_2->gen_sym(bldr_2, "@i8");
    bldr_2->new_type_int(bldr_2, id_11, 0x00000008ull);
    id_12 = bldr_2->gen_sym(bldr_2, "@0xff_i8");
    bldr_2->new_const_int(bldr_2, id_12, id_11, 0x00000000000000ffull);
    id_13 = bldr_2->gen_sym(bldr_2, "@0x0a_i8");
    bldr_2->new_const_int(bldr_2, id_13, id_11, 0x000000000000000aull);
    id_14 = bldr_2->gen_sym(bldr_2, "@sig__i8");
    bldr_2->new_funcsig(bldr_2, id_14, NULL, 0, (MuTypeNode [1]){id_11}, 1);
    id_15 = bldr_2->gen_sym(bldr_2, "@test_fnc");
    bldr_2->new_func(bldr_2, id_15, id_14);
    id_16 = bldr_2->gen_sym(bldr_2, "@test_fnc_v1");
    id_17 = bldr_2->gen_sym(bldr_2, "@test_fnc_v1.blk0");
    id_18 = bldr_2->gen_sym(bldr_2, "@test_fnc_v1.blk0.res");
    id_19 = bldr_2->gen_sym(bldr_2, NULL);
    bldr_2->new_binop(bldr_2, id_19, id_18, MU_BINOP_SUB, id_11, id_13, id_12, MU_NO_ID);
    id_20 = bldr_2->gen_sym(bldr_2, NULL);
    bldr_2->new_ret(bldr_2, id_20, (MuVarNode [1]){id_18}, 1);
    bldr_2->new_bb(bldr_2, id_17, NULL, NULL, 0, MU_NO_ID, (MuInstNode [2]){id_19, id_20}, 2);
    bldr_2->new_func_ver(bldr_2, id_16, id_15, (MuBBNode [1]){id_17}, 1);
    bldr_2->load(bldr_2);
    mu_2->compile_to_sharedlib(mu_2, LIB_FILE_NAME("test_sub"), NULL, 0);
    return 0;
}
