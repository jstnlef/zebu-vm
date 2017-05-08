
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
    MuVM* mu_3;
    MuCtx* ctx_3;
    MuIRBuilder* bldr_3;
    MuID id_21;
    MuID id_22;
    MuID id_23;
    MuID id_24;
    MuID id_25;
    MuID id_26;
    MuID id_27;
    MuID id_28;
    MuID id_29;
    MuID id_30;
    mu_3 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_3 = mu_3->new_context(mu_3);
    bldr_3 = ctx_3->new_ir_builder(ctx_3);
    id_21 = bldr_3->gen_sym(bldr_3, "@i8");
    bldr_3->new_type_int(bldr_3, id_21, 0x00000008ull);
    id_22 = bldr_3->gen_sym(bldr_3, "@0xff_i8");
    bldr_3->new_const_int(bldr_3, id_22, id_21, 0x00000000000000ffull);
    id_23 = bldr_3->gen_sym(bldr_3, "@0x0a_i8");
    bldr_3->new_const_int(bldr_3, id_23, id_21, 0x000000000000000aull);
    id_24 = bldr_3->gen_sym(bldr_3, "@sig__i8");
    bldr_3->new_funcsig(bldr_3, id_24, NULL, 0, (MuTypeNode [1]){id_21}, 1);
    id_25 = bldr_3->gen_sym(bldr_3, "@test_fnc");
    bldr_3->new_func(bldr_3, id_25, id_24);
    id_26 = bldr_3->gen_sym(bldr_3, "@test_fnc_v1");
    id_27 = bldr_3->gen_sym(bldr_3, "@test_fnc_v1.blk0");
    id_28 = bldr_3->gen_sym(bldr_3, "@test_fnc_v1.blk0.res");
    id_29 = bldr_3->gen_sym(bldr_3, NULL);
    bldr_3->new_binop(bldr_3, id_29, id_28, MU_BINOP_MUL, id_21, id_22, id_23, MU_NO_ID);
    id_30 = bldr_3->gen_sym(bldr_3, NULL);
    bldr_3->new_ret(bldr_3, id_30, (MuVarNode [1]){id_28}, 1);
    bldr_3->new_bb(bldr_3, id_27, NULL, NULL, 0, MU_NO_ID, (MuInstNode [2]){id_29, id_30}, 2);
    bldr_3->new_func_ver(bldr_3, id_26, id_25, (MuBBNode [1]){id_27}, 1);
    bldr_3->load(bldr_3);
    mu_3->compile_to_sharedlib(mu_3, LIB_FILE_NAME("test_mul"), NULL, 0);
    return 0;
}
