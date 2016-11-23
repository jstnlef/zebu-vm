
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_36;
    MuCtx* ctx_36;
    MuIRBuilder* bldr_36;
    MuID id_457;
    MuID id_458;
    MuID id_459;
    MuID id_460;
    MuID id_461;
    MuID id_462;
    MuID id_463;
    MuID id_464;
    MuID id_465;
    MuID id_466;
    mu_36 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_36 = mu_36->new_context(mu_36);
    bldr_36 = ctx_36->new_ir_builder(ctx_36);
    id_457 = bldr_36->gen_sym(bldr_36, "@dbl");
    bldr_36->new_type_double(bldr_36, id_457);
    id_458 = bldr_36->gen_sym(bldr_36, "@sig_dbldbl_dbl");
    bldr_36->new_funcsig(bldr_36, id_458, (MuTypeNode [2]){id_457, id_457}, 2, (MuTypeNode [1]){id_457}, 1);
    id_459 = bldr_36->gen_sym(bldr_36, "@test_fnc");
    bldr_36->new_func(bldr_36, id_459, id_458);
    id_460 = bldr_36->gen_sym(bldr_36, "@test_fnc.v1");
    id_461 = bldr_36->gen_sym(bldr_36, "@test_fnc.v1.blk0");
    id_462 = bldr_36->gen_sym(bldr_36, "@test_fnc.v1.blk0.a");
    id_463 = bldr_36->gen_sym(bldr_36, "@test_fnc.v1.blk0.b");
    id_464 = bldr_36->gen_sym(bldr_36, "@test_fnc.v1.blk0.res");
    id_465 = bldr_36->gen_sym(bldr_36, NULL);
    bldr_36->new_binop(bldr_36, id_465, id_464, MU_BINOP_FADD, id_457, id_462, id_463, MU_NO_ID);
    id_466 = bldr_36->gen_sym(bldr_36, NULL);
    bldr_36->new_ret(bldr_36, id_466, (MuVarNode [1]){id_464}, 1);
    bldr_36->new_bb(bldr_36, id_461, (MuID [2]){id_462, id_463}, (MuTypeNode [2]){id_457, id_457}, 2, MU_NO_ID, (MuInstNode [2]){id_465, id_466}, 2);
    bldr_36->new_func_ver(bldr_36, id_460, id_459, (MuBBNode [1]){id_461}, 1);
    bldr_36->load(bldr_36);
    mu_36->compile_to_sharedlib(mu_36, "test_double_arg_pass.dylib", NULL, 0);
    printf("%s\n", "test_double_arg_pass.dylib");
    return 0;
}
