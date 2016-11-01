
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_7;
    MuCtx* ctx_7;
    MuIRBuilder* bldr_7;
    MuID id_61;
    MuID id_62;
    MuID id_63;
    MuID id_64;
    MuID id_65;
    MuID id_66;
    MuID id_67;
    MuID id_68;
    MuID id_69;
    MuID id_70;
    mu_7 = mu_fastimpl_new();
    ctx_7 = mu_7->new_context(mu_7);
    bldr_7 = ctx_7->new_ir_builder(ctx_7);
    id_61 = bldr_7->gen_sym(bldr_7, "@i64");
    bldr_7->new_type_int(bldr_7, id_61, 64);
    id_62 = bldr_7->gen_sym(bldr_7, "@0x8d9f9c1d58324b55_i64");
    bldr_7->new_const_int(bldr_7, id_62, id_61, 10205046930492509013);
    id_63 = bldr_7->gen_sym(bldr_7, "@0x0a_i64");
    bldr_7->new_const_int(bldr_7, id_63, id_61, 10);
    id_64 = bldr_7->gen_sym(bldr_7, "@sig__i64");
    bldr_7->new_funcsig(bldr_7, id_64, NULL, 0, (MuTypeNode [1]){id_61}, 1);
    id_65 = bldr_7->gen_sym(bldr_7, "@test_fnc");
    bldr_7->new_func(bldr_7, id_65, id_64);
    id_66 = bldr_7->gen_sym(bldr_7, "@test_fnc_v1");
    id_67 = bldr_7->gen_sym(bldr_7, "@test_fnc_v1.blk0");
    id_68 = bldr_7->gen_sym(bldr_7, "@test_fnc_v1.blk0.res");
    id_69 = bldr_7->gen_sym(bldr_7, NULL);
    bldr_7->new_binop(bldr_7, id_69, id_68, MU_BINOP_LSHR, id_61, id_62, id_63, MU_NO_ID);
    id_70 = bldr_7->gen_sym(bldr_7, NULL);
    bldr_7->new_ret(bldr_7, id_70, (MuVarNode [1]){id_68}, 1);
    bldr_7->new_bb(bldr_7, id_67, NULL, NULL, 0, MU_NO_ID, (MuInstNode [2]){id_69, id_70}, 2);
    bldr_7->new_func_ver(bldr_7, id_66, id_65, (MuBBNode [1]){id_67}, 1);
    bldr_7->load(bldr_7);
    mu_7->compile_to_sharedlib(mu_7, "test_lshr.dylib", NULL, 0);
    printf("%s\n", "test_lshr.dylib");
    return 0;
}
