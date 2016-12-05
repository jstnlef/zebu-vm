
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
    MuVM* mu_46;
    MuCtx* ctx_46;
    MuIRBuilder* bldr_46;
    MuID id_568;
    MuID id_569;
    MuID id_570;
    MuID id_571;
    MuID id_572;
    MuID id_573;
    MuID id_574;
    MuID id_575;
    MuID id_576;
    MuID id_577;
    MuID id_578;
    MuID id_579;
    MuID id_580;
    MuID id_581;
    MuID id_582;
    mu_46 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_46 = mu_46->new_context(mu_46);
    bldr_46 = ctx_46->new_ir_builder(ctx_46);
    id_568 = bldr_46->gen_sym(bldr_46, "@i1");
    bldr_46->new_type_int(bldr_46, id_568, 0x00000001ull);
    id_569 = bldr_46->gen_sym(bldr_46, "@i64");
    bldr_46->new_type_int(bldr_46, id_569, 0x00000040ull);
    id_570 = bldr_46->gen_sym(bldr_46, "@refi64");
    bldr_46->new_type_ref(bldr_46, id_570, id_569);
    id_571 = bldr_46->gen_sym(bldr_46, "@NULL_refi64");
    bldr_46->new_const_null(bldr_46, id_571, id_570);
    id_572 = bldr_46->gen_sym(bldr_46, "@sig__i64");
    bldr_46->new_funcsig(bldr_46, id_572, NULL, 0, (MuTypeNode [1]){id_569}, 1);
    id_573 = bldr_46->gen_sym(bldr_46, "@test_fnc");
    bldr_46->new_func(bldr_46, id_573, id_572);
    id_574 = bldr_46->gen_sym(bldr_46, "@test_fnc.v1");
    id_575 = bldr_46->gen_sym(bldr_46, "@test_fnc.v1.blk0");
    id_576 = bldr_46->gen_sym(bldr_46, "@test_fnc.v1.blk0.r");
    id_577 = bldr_46->gen_sym(bldr_46, "@test_fnc.v1.blk0.cmpres");
    id_578 = bldr_46->gen_sym(bldr_46, "@test_fnc.v1.blk0.res");
    id_579 = bldr_46->gen_sym(bldr_46, NULL);
    bldr_46->new_new(bldr_46, id_579, id_576, id_569, MU_NO_ID);
    id_580 = bldr_46->gen_sym(bldr_46, NULL);
    bldr_46->new_cmp(bldr_46, id_580, id_577, MU_CMP_EQ, id_570, id_576, id_571);
    id_581 = bldr_46->gen_sym(bldr_46, NULL);
    bldr_46->new_conv(bldr_46, id_581, id_578, MU_CONV_ZEXT, id_568, id_569, id_577);
    id_582 = bldr_46->gen_sym(bldr_46, NULL);
    bldr_46->new_ret(bldr_46, id_582, (MuVarNode [1]){id_578}, 1);
    bldr_46->new_bb(bldr_46, id_575, NULL, NULL, 0, MU_NO_ID, (MuInstNode [4]){id_579, id_580, id_581, id_582}, 4);
    bldr_46->new_func_ver(bldr_46, id_574, id_573, (MuBBNode [1]){id_575}, 1);
    bldr_46->load(bldr_46);
    mu_46->compile_to_sharedlib(mu_46, LIB_FILE_NAME("test_new"), NULL, 0);
    return 0;
}
