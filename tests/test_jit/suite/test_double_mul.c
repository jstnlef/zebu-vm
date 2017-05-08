
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
    MuVM* mu_33;
    MuCtx* ctx_33;
    MuIRBuilder* bldr_33;
    MuID id_409;
    MuID id_410;
    MuID id_411;
    MuID id_412;
    MuID id_413;
    MuID id_414;
    MuID id_415;
    MuID id_416;
    MuID id_417;
    MuID id_418;
    mu_33 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_33 = mu_33->new_context(mu_33);
    bldr_33 = ctx_33->new_ir_builder(ctx_33);
    id_409 = bldr_33->gen_sym(bldr_33, "@dbl");
    bldr_33->new_type_double(bldr_33, id_409);
    id_410 = bldr_33->gen_sym(bldr_33, "@pi");
    bldr_33->new_const_double(bldr_33, id_410, id_409, *(double*)(uint64_t [1]){0x400921fb82c2bd7full});
    id_411 = bldr_33->gen_sym(bldr_33, "@e");
    bldr_33->new_const_double(bldr_33, id_411, id_409, *(double*)(uint64_t [1]){0x4005bf0995aaf790ull});
    id_412 = bldr_33->gen_sym(bldr_33, "@sig__dbl");
    bldr_33->new_funcsig(bldr_33, id_412, NULL, 0, (MuTypeNode [1]){id_409}, 1);
    id_413 = bldr_33->gen_sym(bldr_33, "@test_fnc");
    bldr_33->new_func(bldr_33, id_413, id_412);
    id_414 = bldr_33->gen_sym(bldr_33, "@test_fnc.v1");
    id_415 = bldr_33->gen_sym(bldr_33, "@test_fnc.v1.blk0");
    id_416 = bldr_33->gen_sym(bldr_33, "@test_fnc.v1.blk0.res");
    id_417 = bldr_33->gen_sym(bldr_33, NULL);
    bldr_33->new_binop(bldr_33, id_417, id_416, MU_BINOP_FMUL, id_409, id_410, id_411, MU_NO_ID);
    id_418 = bldr_33->gen_sym(bldr_33, NULL);
    bldr_33->new_ret(bldr_33, id_418, (MuVarNode [1]){id_416}, 1);
    bldr_33->new_bb(bldr_33, id_415, NULL, NULL, 0, MU_NO_ID, (MuInstNode [2]){id_417, id_418}, 2);
    bldr_33->new_func_ver(bldr_33, id_414, id_413, (MuBBNode [1]){id_415}, 1);
    bldr_33->load(bldr_33);
    mu_33->compile_to_sharedlib(mu_33, LIB_FILE_NAME("test_double_mul"), NULL, 0);
    return 0;
}
