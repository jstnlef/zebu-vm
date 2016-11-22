
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_21;
    MuCtx* ctx_21;
    MuIRBuilder* bldr_21;
    MuID id_274;
    MuID id_275;
    MuID id_276;
    MuID id_277;
    MuID id_278;
    MuID id_279;
    MuID id_280;
    MuID id_281;
    MuID id_282;
    MuID id_283;
    MuID id_284;
    mu_21 = mu_fastimpl_new_with_opts("init_muinit_mu --log-level=none --aot-emit-dir=emit");
    ctx_21 = mu_21->new_context(mu_21);
    bldr_21 = ctx_21->new_ir_builder(ctx_21);
    id_274 = bldr_21->gen_sym(bldr_21, "@i64");
    bldr_21->new_type_int(bldr_21, id_274, 64);
    id_275 = bldr_21->gen_sym(bldr_21, "@sig_i64_i64");
    bldr_21->new_funcsig(bldr_21, id_275, (MuTypeNode [1]){id_274}, 1, (MuTypeNode [1]){id_274}, 1);
    id_276 = bldr_21->gen_sym(bldr_21, "@fnpsig_i64_i64");
    bldr_21->new_type_ufuncptr(bldr_21, id_276, id_275);
    id_277 = bldr_21->gen_sym(bldr_21, "@c_fnc");
    bldr_21->new_const_extern(bldr_21, id_277, id_276, "fnc");
    id_278 = bldr_21->gen_sym(bldr_21, "@test_ccall");
    bldr_21->new_func(bldr_21, id_278, id_275);
    id_279 = bldr_21->gen_sym(bldr_21, "@test_ccall_v1");
    id_280 = bldr_21->gen_sym(bldr_21, "@test_ccall_v1.blk0");
    id_281 = bldr_21->gen_sym(bldr_21, "@test_ccall_v1.blk0.k");
    id_282 = bldr_21->gen_sym(bldr_21, "@test_ccall_v1.blk0.res");
    id_283 = bldr_21->gen_sym(bldr_21, NULL);
    bldr_21->new_ccall(bldr_21, id_283, (MuID [1]){id_282}, 1, MU_CC_DEFAULT, id_276, id_275, id_277, (MuVarNode [1]){id_281}, 1, MU_NO_ID, MU_NO_ID);
    id_284 = bldr_21->gen_sym(bldr_21, NULL);
    bldr_21->new_ret(bldr_21, id_284, (MuVarNode [1]){id_282}, 1);
    bldr_21->new_bb(bldr_21, id_280, (MuID [1]){id_281}, (MuTypeNode [1]){id_274}, 1, MU_NO_ID, (MuInstNode [2]){id_283, id_284}, 2);
    bldr_21->new_func_ver(bldr_21, id_279, id_278, (MuBBNode [1]){id_280}, 1);
    bldr_21->load(bldr_21);
    mu_21->compile_to_sharedlib(mu_21, "test_ccall.dylib", (char*[]){&"suite/test_ccall_fnc.c"}, 1);
    printf("%s\n", "test_ccall.dylib");
    return 0;
}
