
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
    MuVM* mu_4;
    MuCtx* ctx_4;
    MuIRBuilder* bldr_4;
    MuID id_31;
    MuID id_32;
    MuID id_33;
    MuID id_34;
    MuID id_35;
    MuID id_36;
    MuID id_37;
    MuID id_38;
    MuID id_39;
    MuID id_40;
    mu_4 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_4 = mu_4->new_context(mu_4);
    bldr_4 = ctx_4->new_ir_builder(ctx_4);
    id_31 = bldr_4->gen_sym(bldr_4, "@i8");
    bldr_4->new_type_int(bldr_4, id_31, 0x00000008ull);
    id_32 = bldr_4->gen_sym(bldr_4, "@0x80_i8");
    bldr_4->new_const_int(bldr_4, id_32, id_31, 0x0000000000000080ull);
    id_33 = bldr_4->gen_sym(bldr_4, "@0x0a_i8");
    bldr_4->new_const_int(bldr_4, id_33, id_31, 0x000000000000000aull);
    id_34 = bldr_4->gen_sym(bldr_4, "@sig__i8");
    bldr_4->new_funcsig(bldr_4, id_34, NULL, 0, (MuTypeNode [1]){id_31}, 1);
    id_35 = bldr_4->gen_sym(bldr_4, "@test_fnc");
    bldr_4->new_func(bldr_4, id_35, id_34);
    id_36 = bldr_4->gen_sym(bldr_4, "@test_fnc_v1");
    id_37 = bldr_4->gen_sym(bldr_4, "@test_fnc_v1.blk0");
    id_38 = bldr_4->gen_sym(bldr_4, "@test_fnc_v1.blk0.res");
    id_39 = bldr_4->gen_sym(bldr_4, NULL);
    bldr_4->new_binop(bldr_4, id_39, id_38, MU_BINOP_UDIV, id_31, id_32, id_33, MU_NO_ID);
    id_40 = bldr_4->gen_sym(bldr_4, NULL);
    bldr_4->new_ret(bldr_4, id_40, (MuVarNode [1]){id_38}, 1);
    bldr_4->new_bb(bldr_4, id_37, NULL, NULL, 0, MU_NO_ID, (MuInstNode [2]){id_39, id_40}, 2);
    bldr_4->new_func_ver(bldr_4, id_36, id_35, (MuBBNode [1]){id_37}, 1);
    bldr_4->load(bldr_4);
    mu_4->compile_to_sharedlib(mu_4, LIB_FILE_NAME("test_udiv"), NULL, 0);
    return 0;
}
