
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_30;
    MuCtx* ctx_30;
    MuIRBuilder* bldr_30;
    MuID id_427;
    MuID id_428;
    MuID id_429;
    MuID id_430;
    MuID id_431;
    MuID id_432;
    MuID id_433;
    MuID id_434;
    MuID id_435;
    MuID id_436;
    MuID id_437;
    MuID id_438;
    MuID id_439;
    MuID id_440;
    MuID id_441;
    mu_30 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_30 = mu_30->new_context(mu_30);
    bldr_30 = ctx_30->new_ir_builder(ctx_30);
    id_427 = bldr_30->gen_sym(bldr_30, "@i8");
    bldr_30->new_type_int(bldr_30, id_427, 8);
    id_428 = bldr_30->gen_sym(bldr_30, "@i32");
    bldr_30->new_type_int(bldr_30, id_428, 32);
    id_429 = bldr_30->gen_sym(bldr_30, "@i64");
    bldr_30->new_type_int(bldr_30, id_429, 64);
    id_430 = bldr_30->gen_sym(bldr_30, "@hyb");
    bldr_30->new_type_hybrid(bldr_30, id_430, (MuTypeNode [2]){id_427, id_429}, 2, id_428);
    id_431 = bldr_30->gen_sym(bldr_30, "@phyb");
    bldr_30->new_type_uptr(bldr_30, id_431, id_430);
    id_432 = bldr_30->gen_sym(bldr_30, "@sig_phyb_i32");
    bldr_30->new_funcsig(bldr_30, id_432, (MuTypeNode [1]){id_431}, 1, (MuTypeNode [1]){id_428}, 1);
    id_433 = bldr_30->gen_sym(bldr_30, "@test_fnc");
    bldr_30->new_func(bldr_30, id_433, id_432);
    id_434 = bldr_30->gen_sym(bldr_30, "@test_fnc.v1");
    id_435 = bldr_30->gen_sym(bldr_30, "@test_fnc.v1.blk0");
    id_436 = bldr_30->gen_sym(bldr_30, "@test_fnc.v1.blk0.ps");
    id_437 = bldr_30->gen_sym(bldr_30, "@test_fnc.v1.blk0.pfld");
    id_438 = bldr_30->gen_sym(bldr_30, "@test_fnc.v1.blk0.res");
    id_439 = bldr_30->gen_sym(bldr_30, NULL);
    bldr_30->new_getvarpartiref(bldr_30, id_439, id_437, true, id_430, id_436);
    id_440 = bldr_30->gen_sym(bldr_30, NULL);
    bldr_30->new_load(bldr_30, id_440, id_438, true, MU_ORD_NOT_ATOMIC, id_428, id_437, MU_NO_ID);
    id_441 = bldr_30->gen_sym(bldr_30, NULL);
    bldr_30->new_ret(bldr_30, id_441, (MuVarNode [1]){id_438}, 1);
    bldr_30->new_bb(bldr_30, id_435, (MuID [1]){id_436}, (MuTypeNode [1]){id_431}, 1, MU_NO_ID, (MuInstNode [3]){id_439, id_440, id_441}, 3);
    bldr_30->new_func_ver(bldr_30, id_434, id_433, (MuBBNode [1]){id_435}, 1);
    bldr_30->load(bldr_30);
    mu_30->compile_to_sharedlib(mu_30, "test_getvarpartiref.dylib", NULL, 0);
    printf("%s\n", "test_getvarpartiref.dylib");
    return 0;
}
