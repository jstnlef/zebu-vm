
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
    MuVM* mu_39;
    MuCtx* ctx_39;
    MuIRBuilder* bldr_39;
    MuID id_487;
    MuID id_488;
    MuID id_489;
    MuID id_490;
    MuID id_491;
    MuID id_492;
    MuID id_493;
    MuID id_494;
    MuID id_495;
    MuID id_496;
    MuID id_497;
    MuID id_498;
    MuID id_499;
    mu_39 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_39 = mu_39->new_context(mu_39);
    bldr_39 = ctx_39->new_ir_builder(ctx_39);
    id_487 = bldr_39->gen_sym(bldr_39, "@dbl");
    bldr_39->new_type_double(bldr_39, id_487);
    id_488 = bldr_39->gen_sym(bldr_39, "@i1");
    bldr_39->new_type_int(bldr_39, id_488, 0x00000001ull);
    id_489 = bldr_39->gen_sym(bldr_39, "@i64");
    bldr_39->new_type_int(bldr_39, id_489, 0x00000040ull);
    id_490 = bldr_39->gen_sym(bldr_39, "@e");
    bldr_39->new_const_double(bldr_39, id_490, id_487, *(double*)(uint64_t [1]){0x4005bf0995aaf790ull});
    id_491 = bldr_39->gen_sym(bldr_39, "@sig__i64");
    bldr_39->new_funcsig(bldr_39, id_491, NULL, 0, (MuTypeNode [1]){id_489}, 1);
    id_492 = bldr_39->gen_sym(bldr_39, "@test_fnc");
    bldr_39->new_func(bldr_39, id_492, id_491);
    id_493 = bldr_39->gen_sym(bldr_39, "@test_fnc.v1");
    id_494 = bldr_39->gen_sym(bldr_39, "@test_fnc.v1.blk0");
    id_495 = bldr_39->gen_sym(bldr_39, "@test_fnc.v1.blk0.cmpres");
    id_496 = bldr_39->gen_sym(bldr_39, "@test_fnc.v1.blk0.res");
    id_497 = bldr_39->gen_sym(bldr_39, NULL);
    bldr_39->new_cmp(bldr_39, id_497, id_495, MU_CMP_FOGE, id_487, id_490, id_490);
    id_498 = bldr_39->gen_sym(bldr_39, NULL);
    bldr_39->new_conv(bldr_39, id_498, id_496, MU_CONV_ZEXT, id_488, id_489, id_495);
    id_499 = bldr_39->gen_sym(bldr_39, NULL);
    bldr_39->new_ret(bldr_39, id_499, (MuVarNode [1]){id_496}, 1);
    bldr_39->new_bb(bldr_39, id_494, NULL, NULL, 0, MU_NO_ID, (MuInstNode [3]){id_497, id_498, id_499}, 3);
    bldr_39->new_func_ver(bldr_39, id_493, id_492, (MuBBNode [1]){id_494}, 1);
    bldr_39->load(bldr_39);
    mu_39->compile_to_sharedlib(mu_39, LIB_FILE_NAME("test_double_ordered_ge"), NULL, 0);
    return 0;
}
