
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_6;
    MuCtx* ctx_6;
    MuIRBuilder* bldr_6;
    MuID id_51;
    MuID id_52;
    MuID id_53;
    MuID id_54;
    MuID id_55;
    MuID id_56;
    MuID id_57;
    MuID id_58;
    MuID id_59;
    MuID id_60;
    mu_6 = mu_fastimpl_new_with_opts("init_muinit_mu --log-level=none --aot-emit-dir=emit");
    ctx_6 = mu_6->new_context(mu_6);
    bldr_6 = ctx_6->new_ir_builder(ctx_6);
    id_51 = bldr_6->gen_sym(bldr_6, "@i64");
    bldr_6->new_type_int(bldr_6, id_51, 64);
    id_52 = bldr_6->gen_sym(bldr_6, "@0x6d9f9c1d58324b55_i64");
    bldr_6->new_const_int(bldr_6, id_52, id_51, 7899203921278815061);
    id_53 = bldr_6->gen_sym(bldr_6, "@0x0a_i64");
    bldr_6->new_const_int(bldr_6, id_53, id_51, 10);
    id_54 = bldr_6->gen_sym(bldr_6, "@sig__i64");
    bldr_6->new_funcsig(bldr_6, id_54, NULL, 0, (MuTypeNode [1]){id_51}, 1);
    id_55 = bldr_6->gen_sym(bldr_6, "@test_fnc");
    bldr_6->new_func(bldr_6, id_55, id_54);
    id_56 = bldr_6->gen_sym(bldr_6, "@test_fnc_v1");
    id_57 = bldr_6->gen_sym(bldr_6, "@test_fnc_v1.blk0");
    id_58 = bldr_6->gen_sym(bldr_6, "@test_fnc_v1.blk0.res");
    id_59 = bldr_6->gen_sym(bldr_6, NULL);
    bldr_6->new_binop(bldr_6, id_59, id_58, MU_BINOP_SHL, id_51, id_52, id_53, MU_NO_ID);
    id_60 = bldr_6->gen_sym(bldr_6, NULL);
    bldr_6->new_ret(bldr_6, id_60, (MuVarNode [1]){id_58}, 1);
    bldr_6->new_bb(bldr_6, id_57, NULL, NULL, 0, MU_NO_ID, (MuInstNode [2]){id_59, id_60}, 2);
    bldr_6->new_func_ver(bldr_6, id_56, id_55, (MuBBNode [1]){id_57}, 1);
    bldr_6->load(bldr_6);
    mu_6->compile_to_sharedlib(mu_6, "test_shl.dylib", NULL, 0);
    printf("%s\n", "test_shl.dylib");
    return 0;
}
