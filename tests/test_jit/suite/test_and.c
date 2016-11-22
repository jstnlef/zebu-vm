
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_8;
    MuCtx* ctx_8;
    MuIRBuilder* bldr_8;
    MuID id_71;
    MuID id_72;
    MuID id_73;
    MuID id_74;
    MuID id_75;
    MuID id_76;
    MuID id_77;
    MuID id_78;
    MuID id_79;
    MuID id_80;
    mu_8 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_8 = mu_8->new_context(mu_8);
    bldr_8 = ctx_8->new_ir_builder(ctx_8);
    id_71 = bldr_8->gen_sym(bldr_8, "@i64");
    bldr_8->new_type_int(bldr_8, id_71, 64);
    id_72 = bldr_8->gen_sym(bldr_8, "@0x8d9f9c1d58324b55_i64");
    bldr_8->new_const_int(bldr_8, id_72, id_71, 10205046930492509013);
    id_73 = bldr_8->gen_sym(bldr_8, "@0xd5a8f2deb00debb4_i64");
    bldr_8->new_const_int(bldr_8, id_73, id_71, 15395822364416404404);
    id_74 = bldr_8->gen_sym(bldr_8, "@sig__i64");
    bldr_8->new_funcsig(bldr_8, id_74, NULL, 0, (MuTypeNode [1]){id_71}, 1);
    id_75 = bldr_8->gen_sym(bldr_8, "@test_fnc");
    bldr_8->new_func(bldr_8, id_75, id_74);
    id_76 = bldr_8->gen_sym(bldr_8, "@test_fnc_v1");
    id_77 = bldr_8->gen_sym(bldr_8, "@test_fnc_v1.blk0");
    id_78 = bldr_8->gen_sym(bldr_8, "@test_fnc_v1.blk0.res");
    id_79 = bldr_8->gen_sym(bldr_8, NULL);
    bldr_8->new_binop(bldr_8, id_79, id_78, MU_BINOP_AND, id_71, id_72, id_73, MU_NO_ID);
    id_80 = bldr_8->gen_sym(bldr_8, NULL);
    bldr_8->new_ret(bldr_8, id_80, (MuVarNode [1]){id_78}, 1);
    bldr_8->new_bb(bldr_8, id_77, NULL, NULL, 0, MU_NO_ID, (MuInstNode [2]){id_79, id_80}, 2);
    bldr_8->new_func_ver(bldr_8, id_76, id_75, (MuBBNode [1]){id_77}, 1);
    bldr_8->load(bldr_8);
    mu_8->compile_to_sharedlib(mu_8, "test_and.dylib", NULL, 0);
    printf("%s\n", "test_and.dylib");
    return 0;
}
