
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
    MuVM* mu_11;
    MuCtx* ctx_11;
    MuIRBuilder* bldr_11;
    MuID id_101;
    MuID id_102;
    MuID id_103;
    MuID id_104;
    MuID id_105;
    MuID id_106;
    MuID id_107;
    MuID id_108;
    MuID id_109;
    MuID id_110;
    mu_11 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_11 = mu_11->new_context(mu_11);
    bldr_11 = ctx_11->new_ir_builder(ctx_11);
    id_101 = bldr_11->gen_sym(bldr_11, "@i64");
    bldr_11->new_type_int(bldr_11, id_101, 0x00000040ull);
    id_102 = bldr_11->gen_sym(bldr_11, "@0x8d9f9c1d58324b55_i64");
    bldr_11->new_const_int(bldr_11, id_102, id_101, 0x8d9f9c1d58324b55ull);
    id_103 = bldr_11->gen_sym(bldr_11, "@0xd5a8f2deb00debb4_i64");
    bldr_11->new_const_int(bldr_11, id_103, id_101, 0xd5a8f2deb00debb4ull);
    id_104 = bldr_11->gen_sym(bldr_11, "@sig__i64");
    bldr_11->new_funcsig(bldr_11, id_104, NULL, 0, (MuTypeNode [1]){id_101}, 1);
    id_105 = bldr_11->gen_sym(bldr_11, "@test_fnc");
    bldr_11->new_func(bldr_11, id_105, id_104);
    id_106 = bldr_11->gen_sym(bldr_11, "@test_fnc_v1");
    id_107 = bldr_11->gen_sym(bldr_11, "@test_fnc_v1.blk0");
    id_108 = bldr_11->gen_sym(bldr_11, "@test_fnc_v1.blk0.res");
    id_109 = bldr_11->gen_sym(bldr_11, NULL);
    bldr_11->new_binop(bldr_11, id_109, id_108, MU_BINOP_AND, id_101, id_102, id_103, MU_NO_ID);
    id_110 = bldr_11->gen_sym(bldr_11, NULL);
    bldr_11->new_ret(bldr_11, id_110, (MuVarNode [1]){id_108}, 1);
    bldr_11->new_bb(bldr_11, id_107, NULL, NULL, 0, MU_NO_ID, (MuInstNode [2]){id_109, id_110}, 2);
    bldr_11->new_func_ver(bldr_11, id_106, id_105, (MuBBNode [1]){id_107}, 1);
    bldr_11->load(bldr_11);
    mu_11->compile_to_sharedlib(mu_11, LIB_FILE_NAME("test_and"), NULL, 0);
    return 0;
}
