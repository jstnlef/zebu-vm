
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
    MuVM* mu_30;
    MuCtx* ctx_30;
    MuIRBuilder* bldr_30;
    MuID id_379;
    MuID id_380;
    MuID id_381;
    MuID id_382;
    MuID id_383;
    MuID id_384;
    MuID id_385;
    MuID id_386;
    MuID id_387;
    MuID id_388;
    mu_30 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_30 = mu_30->new_context(mu_30);
    bldr_30 = ctx_30->new_ir_builder(ctx_30);
    id_379 = bldr_30->gen_sym(bldr_30, "@i32");
    bldr_30->new_type_int(bldr_30, id_379, 0x00000020ull);
    id_380 = bldr_30->gen_sym(bldr_30, "@i64");
    bldr_30->new_type_int(bldr_30, id_380, 0x00000040ull);
    id_381 = bldr_30->gen_sym(bldr_30, "@0xa8324b55_i32");
    bldr_30->new_const_int(bldr_30, id_381, id_379, 0x00000000a8324b55ull);
    id_382 = bldr_30->gen_sym(bldr_30, "@sig__i64");
    bldr_30->new_funcsig(bldr_30, id_382, NULL, 0, (MuTypeNode [1]){id_380}, 1);
    id_383 = bldr_30->gen_sym(bldr_30, "@test_fnc");
    bldr_30->new_func(bldr_30, id_383, id_382);
    id_384 = bldr_30->gen_sym(bldr_30, "@test_fnc_v1");
    id_385 = bldr_30->gen_sym(bldr_30, "@test_fnc_v1.blk0");
    id_386 = bldr_30->gen_sym(bldr_30, "@test_fnc_v1.blk0.res");
    id_387 = bldr_30->gen_sym(bldr_30, NULL);
    bldr_30->new_conv(bldr_30, id_387, id_386, MU_CONV_ZEXT, id_379, id_380, id_381);
    id_388 = bldr_30->gen_sym(bldr_30, NULL);
    bldr_30->new_ret(bldr_30, id_388, (MuVarNode [1]){id_386}, 1);
    bldr_30->new_bb(bldr_30, id_385, NULL, NULL, 0, MU_NO_ID, (MuInstNode [2]){id_387, id_388}, 2);
    bldr_30->new_func_ver(bldr_30, id_384, id_383, (MuBBNode [1]){id_385}, 1);
    bldr_30->load(bldr_30);
    mu_30->compile_to_sharedlib(mu_30, LIB_FILE_NAME("test_zext"), NULL, 0);
    return 0;
}
