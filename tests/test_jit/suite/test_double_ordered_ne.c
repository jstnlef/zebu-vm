
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
    MuVM* mu_36;
    MuCtx* ctx_36;
    MuIRBuilder* bldr_36;
    MuID id_443;
    MuID id_444;
    MuID id_445;
    MuID id_446;
    MuID id_447;
    MuID id_448;
    MuID id_449;
    MuID id_450;
    MuID id_451;
    MuID id_452;
    MuID id_453;
    MuID id_454;
    MuID id_455;
    MuID id_456;
    MuID id_457;
    MuID id_458;
    MuID id_459;
    mu_36 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_36 = mu_36->new_context(mu_36);
    bldr_36 = ctx_36->new_ir_builder(ctx_36);
    id_443 = bldr_36->gen_sym(bldr_36, "@dbl");
    bldr_36->new_type_double(bldr_36, id_443);
    id_444 = bldr_36->gen_sym(bldr_36, "@i1");
    bldr_36->new_type_int(bldr_36, id_444, 0x00000001ull);
    id_445 = bldr_36->gen_sym(bldr_36, "@i64");
    bldr_36->new_type_int(bldr_36, id_445, 0x00000040ull);
    id_446 = bldr_36->gen_sym(bldr_36, "@1_dbl");
    bldr_36->new_const_double(bldr_36, id_446, id_443, *(double*)(uint64_t [1]){0x3ff0000000000000ull});
    id_447 = bldr_36->gen_sym(bldr_36, "@3_dbl");
    bldr_36->new_const_double(bldr_36, id_447, id_443, *(double*)(uint64_t [1]){0x4008000000000000ull});
    id_448 = bldr_36->gen_sym(bldr_36, "@zp3");
    bldr_36->new_const_double(bldr_36, id_448, id_443, *(double*)(uint64_t [1]){0x3fd3333333333333ull});
    id_449 = bldr_36->gen_sym(bldr_36, "@sig__i64");
    bldr_36->new_funcsig(bldr_36, id_449, NULL, 0, (MuTypeNode [1]){id_445}, 1);
    id_450 = bldr_36->gen_sym(bldr_36, "@test_fnc");
    bldr_36->new_func(bldr_36, id_450, id_449);
    id_451 = bldr_36->gen_sym(bldr_36, "@test_fnc.v1");
    id_452 = bldr_36->gen_sym(bldr_36, "@test_fnc.v1.blk0");
    id_453 = bldr_36->gen_sym(bldr_36, "@test_fnc.v1.blk0.k");
    id_454 = bldr_36->gen_sym(bldr_36, "@test_fnc.v1.blk0.cmpres");
    id_455 = bldr_36->gen_sym(bldr_36, "@test_fnc.v1.blk0.res");
    id_456 = bldr_36->gen_sym(bldr_36, NULL);
    bldr_36->new_binop(bldr_36, id_456, id_453, MU_BINOP_FDIV, id_443, id_446, id_447, MU_NO_ID);
    id_457 = bldr_36->gen_sym(bldr_36, NULL);
    bldr_36->new_cmp(bldr_36, id_457, id_454, MU_CMP_FONE, id_443, id_453, id_448);
    id_458 = bldr_36->gen_sym(bldr_36, NULL);
    bldr_36->new_conv(bldr_36, id_458, id_455, MU_CONV_ZEXT, id_444, id_445, id_454);
    id_459 = bldr_36->gen_sym(bldr_36, NULL);
    bldr_36->new_ret(bldr_36, id_459, (MuVarNode [1]){id_455}, 1);
    bldr_36->new_bb(bldr_36, id_452, NULL, NULL, 0, MU_NO_ID, (MuInstNode [4]){id_456, id_457, id_458, id_459}, 4);
    bldr_36->new_func_ver(bldr_36, id_451, id_450, (MuBBNode [1]){id_452}, 1);
    bldr_36->load(bldr_36);
    mu_36->compile_to_sharedlib(mu_36, LIB_FILE_NAME("test_double_ordered_ne"), NULL, 0);
    return 0;
}
