
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
    MuVM* mu_13;
    MuCtx* ctx_13;
    MuIRBuilder* bldr_13;
    MuID id_121;
    MuID id_122;
    MuID id_123;
    MuID id_124;
    MuID id_125;
    MuID id_126;
    MuID id_127;
    MuID id_128;
    MuID id_129;
    MuID id_130;
    mu_13 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_13 = mu_13->new_context(mu_13);
    bldr_13 = ctx_13->new_ir_builder(ctx_13);
    id_121 = bldr_13->gen_sym(bldr_13, "@i64");
    bldr_13->new_type_int(bldr_13, id_121, 0x00000040ull);
    id_122 = bldr_13->gen_sym(bldr_13, "@0x8d9f9c1d58324b55_i64");
    bldr_13->new_const_int(bldr_13, id_122, id_121, 0x8d9f9c1d58324b55ull);
    id_123 = bldr_13->gen_sym(bldr_13, "@0xd5a8f2deb00debb4_i64");
    bldr_13->new_const_int(bldr_13, id_123, id_121, 0xd5a8f2deb00debb4ull);
    id_124 = bldr_13->gen_sym(bldr_13, "@sig__i64");
    bldr_13->new_funcsig(bldr_13, id_124, NULL, 0, (MuTypeNode [1]){id_121}, 1);
    id_125 = bldr_13->gen_sym(bldr_13, "@test_fnc");
    bldr_13->new_func(bldr_13, id_125, id_124);
    id_126 = bldr_13->gen_sym(bldr_13, "@test_fnc_v1");
    id_127 = bldr_13->gen_sym(bldr_13, "@test_fnc_v1.blk0");
    id_128 = bldr_13->gen_sym(bldr_13, "@test_fnc_v1.blk0.res");
    id_129 = bldr_13->gen_sym(bldr_13, NULL);
    bldr_13->new_binop(bldr_13, id_129, id_128, MU_BINOP_XOR, id_121, id_122, id_123, MU_NO_ID);
    id_130 = bldr_13->gen_sym(bldr_13, NULL);
    bldr_13->new_ret(bldr_13, id_130, (MuVarNode [1]){id_128}, 1);
    bldr_13->new_bb(bldr_13, id_127, NULL, NULL, 0, MU_NO_ID, (MuInstNode [2]){id_129, id_130}, 2);
    bldr_13->new_func_ver(bldr_13, id_126, id_125, (MuBBNode [1]){id_127}, 1);
    bldr_13->load(bldr_13);
    mu_13->compile_to_sharedlib(mu_13, LIB_FILE_NAME("test_xor"), NULL, 0);
    return 0;
}
