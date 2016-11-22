
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_31;
    MuCtx* ctx_31;
    MuIRBuilder* bldr_31;
    MuID id_442;
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
    mu_31 = mu_fastimpl_new_with_opts("init_muinit_mu --log-level=none --aot-emit-dir=emit");
    ctx_31 = mu_31->new_context(mu_31);
    bldr_31 = ctx_31->new_ir_builder(ctx_31);
    id_442 = bldr_31->gen_sym(bldr_31, "@i8");
    bldr_31->new_type_int(bldr_31, id_442, 8);
    id_443 = bldr_31->gen_sym(bldr_31, "@i32");
    bldr_31->new_type_int(bldr_31, id_443, 32);
    id_444 = bldr_31->gen_sym(bldr_31, "@i64");
    bldr_31->new_type_int(bldr_31, id_444, 64);
    id_445 = bldr_31->gen_sym(bldr_31, "@hyb");
    bldr_31->new_type_hybrid(bldr_31, id_445, NULL, 0, id_443);
    id_446 = bldr_31->gen_sym(bldr_31, "@phyb");
    bldr_31->new_type_uptr(bldr_31, id_446, id_445);
    id_447 = bldr_31->gen_sym(bldr_31, "@sig_phyb_i32");
    bldr_31->new_funcsig(bldr_31, id_447, (MuTypeNode [1]){id_446}, 1, (MuTypeNode [1]){id_443}, 1);
    id_448 = bldr_31->gen_sym(bldr_31, "@test_fnc");
    bldr_31->new_func(bldr_31, id_448, id_447);
    id_449 = bldr_31->gen_sym(bldr_31, "@test_fnc.v1");
    id_450 = bldr_31->gen_sym(bldr_31, "@test_fnc.v1.blk0");
    id_451 = bldr_31->gen_sym(bldr_31, "@test_fnc.v1.blk0.ps");
    id_452 = bldr_31->gen_sym(bldr_31, "@test_fnc.v1.blk0.pfld");
    id_453 = bldr_31->gen_sym(bldr_31, "@test_fnc.v1.blk0.res");
    id_454 = bldr_31->gen_sym(bldr_31, NULL);
    bldr_31->new_getvarpartiref(bldr_31, id_454, id_452, true, id_445, id_451);
    id_455 = bldr_31->gen_sym(bldr_31, NULL);
    bldr_31->new_load(bldr_31, id_455, id_453, true, MU_ORD_NOT_ATOMIC, id_443, id_452, MU_NO_ID);
    id_456 = bldr_31->gen_sym(bldr_31, NULL);
    bldr_31->new_ret(bldr_31, id_456, (MuVarNode [1]){id_453}, 1);
    bldr_31->new_bb(bldr_31, id_450, (MuID [1]){id_451}, (MuTypeNode [1]){id_446}, 1, MU_NO_ID, (MuInstNode [3]){id_454, id_455, id_456}, 3);
    bldr_31->new_func_ver(bldr_31, id_449, id_448, (MuBBNode [1]){id_450}, 1);
    bldr_31->load(bldr_31);
    mu_31->compile_to_sharedlib(mu_31, "test_getvarpartiref_nofix.dylib", NULL, 0);
    printf("%s\n", "test_getvarpartiref_nofix.dylib");
    return 0;
}
