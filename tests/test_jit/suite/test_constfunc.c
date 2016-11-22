
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_32;
    MuCtx* ctx_32;
    MuIRBuilder* bldr_32;
    MuID id_457;
    MuID id_458;
    MuID id_459;
    MuID id_460;
    MuID id_461;
    MuID id_462;
    MuID id_463;
    MuID id_464;
    mu_32 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_32 = mu_32->new_context(mu_32);
    bldr_32 = ctx_32->new_ir_builder(ctx_32);
    id_457 = bldr_32->gen_sym(bldr_32, "@i32");
    bldr_32->new_type_int(bldr_32, id_457, 8);
    id_458 = bldr_32->gen_sym(bldr_32, "@0_i32");
    bldr_32->new_const_int(bldr_32, id_458, id_457, 0);
    id_459 = bldr_32->gen_sym(bldr_32, "@sig__i32");
    bldr_32->new_funcsig(bldr_32, id_459, NULL, 0, (MuTypeNode [1]){id_457}, 1);
    id_460 = bldr_32->gen_sym(bldr_32, "@test_fnc");
    bldr_32->new_func(bldr_32, id_460, id_459);
    id_461 = bldr_32->gen_sym(bldr_32, "@test_fnc_v1");
    id_462 = bldr_32->gen_sym(bldr_32, "@test_fnc_v1.blk0");
    id_463 = bldr_32->gen_sym(bldr_32, "@test_fnc_v1.blk0.res");
    id_464 = bldr_32->gen_sym(bldr_32, NULL);
    bldr_32->new_ret(bldr_32, id_464, (MuVarNode [1]){id_458}, 1);
    bldr_32->new_bb(bldr_32, id_462, NULL, NULL, 0, MU_NO_ID, (MuInstNode [1]){id_464}, 1);
    bldr_32->new_func_ver(bldr_32, id_461, id_460, (MuBBNode [1]){id_462}, 1);
    bldr_32->load(bldr_32);
    mu_32->compile_to_sharedlib(mu_32, "test_constfunc.dylib", NULL, 0);
    printf("%s\n", "test_constfunc.dylib");
    return 0;
}
