
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
    MuVM* mu_45;
    MuCtx* ctx_45;
    MuIRBuilder* bldr_45;
    MuID id_557;
    MuID id_558;
    MuID id_559;
    MuID id_560;
    MuID id_561;
    MuID id_562;
    MuID id_563;
    MuID id_564;
    MuID id_565;
    MuID id_566;
    MuID id_567;
    mu_45 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_45 = mu_45->new_context(mu_45);
    bldr_45 = ctx_45->new_ir_builder(ctx_45);
    id_557 = bldr_45->gen_sym(bldr_45, "@dbl");
    bldr_45->new_type_double(bldr_45, id_557);
    id_558 = bldr_45->gen_sym(bldr_45, "@i1");
    bldr_45->new_type_int(bldr_45, id_558, 0x00000001ull);
    id_559 = bldr_45->gen_sym(bldr_45, "@i64");
    bldr_45->new_type_int(bldr_45, id_559, 0x00000040ull);
    id_560 = bldr_45->gen_sym(bldr_45, "@pi");
    bldr_45->new_const_double(bldr_45, id_560, id_557, *(double*)(uint64_t [1]){0x400921fb4d12d84aull});
    id_561 = bldr_45->gen_sym(bldr_45, "@sig__i64");
    bldr_45->new_funcsig(bldr_45, id_561, NULL, 0, (MuTypeNode [1]){id_559}, 1);
    id_562 = bldr_45->gen_sym(bldr_45, "@test_fnc");
    bldr_45->new_func(bldr_45, id_562, id_561);
    id_563 = bldr_45->gen_sym(bldr_45, "@test_fnc.v1");
    id_564 = bldr_45->gen_sym(bldr_45, "@test_fnc.v1.blk0");
    id_565 = bldr_45->gen_sym(bldr_45, "@test_fnc.v1.blk0.res");
    id_566 = bldr_45->gen_sym(bldr_45, NULL);
    bldr_45->new_conv(bldr_45, id_566, id_565, MU_CONV_FPTOUI, id_557, id_559, id_560);
    id_567 = bldr_45->gen_sym(bldr_45, NULL);
    bldr_45->new_ret(bldr_45, id_567, (MuVarNode [1]){id_565}, 1);
    bldr_45->new_bb(bldr_45, id_564, NULL, NULL, 0, MU_NO_ID, (MuInstNode [2]){id_566, id_567}, 2);
    bldr_45->new_func_ver(bldr_45, id_563, id_562, (MuBBNode [1]){id_564}, 1);
    bldr_45->load(bldr_45);
    mu_45->compile_to_sharedlib(mu_45, LIB_FILE_NAME("test_double_fptoui"), NULL, 0);
    return 0;
}
