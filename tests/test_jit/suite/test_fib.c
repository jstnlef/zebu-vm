
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_33;
    MuCtx* ctx_33;
    MuIRBuilder* bldr_33;
    MuID id_465;
    MuID id_466;
    MuID id_467;
    MuID id_468;
    MuID id_469;
    MuID id_470;
    MuID id_471;
    MuID id_472;
    MuID id_473;
    MuID id_474;
    MuID id_475;
    MuID id_476;
    MuID id_477;
    MuID id_478;
    MuID id_479;
    MuID id_480;
    MuID id_481;
    MuID id_482;
    MuID id_483;
    MuID id_484;
    MuID id_485;
    MuID id_486;
    MuID id_487;
    MuID id_488;
    MuID id_489;
    MuID id_490;
    MuID id_491;
    MuID id_492;
    MuID id_493;
    mu_33 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_33 = mu_33->new_context(mu_33);
    bldr_33 = ctx_33->new_ir_builder(ctx_33);
    id_465 = bldr_33->gen_sym(bldr_33, "@i64");
    bldr_33->new_type_int(bldr_33, id_465, 64);
    id_466 = bldr_33->gen_sym(bldr_33, "@0_i64");
    bldr_33->new_const_int(bldr_33, id_466, id_465, 0);
    id_467 = bldr_33->gen_sym(bldr_33, "@1_i64");
    bldr_33->new_const_int(bldr_33, id_467, id_465, 1);
    id_468 = bldr_33->gen_sym(bldr_33, "@2_i64");
    bldr_33->new_const_int(bldr_33, id_468, id_465, 2);
    id_469 = bldr_33->gen_sym(bldr_33, "@sig_i64_i64");
    bldr_33->new_funcsig(bldr_33, id_469, (MuTypeNode [1]){id_465}, 1, (MuTypeNode [1]){id_465}, 1);
    id_470 = bldr_33->gen_sym(bldr_33, "@fib");
    bldr_33->new_func(bldr_33, id_470, id_469);
    id_471 = bldr_33->gen_sym(bldr_33, "@fib_v1");
    id_472 = bldr_33->gen_sym(bldr_33, "@fib_v1.blk0");
    id_473 = bldr_33->gen_sym(bldr_33, "@fib_v1.blk1");
    id_474 = bldr_33->gen_sym(bldr_33, "@fib_v1.blk2");
    id_475 = bldr_33->gen_sym(bldr_33, "@fib_v1.blk0.k");
    id_476 = bldr_33->gen_sym(bldr_33, NULL);
    id_477 = bldr_33->gen_sym(bldr_33, NULL);
    id_478 = bldr_33->gen_sym(bldr_33, NULL);
    bldr_33->new_dest_clause(bldr_33, id_476, id_474, (MuVarNode [1]){id_475}, 1);
    bldr_33->new_dest_clause(bldr_33, id_477, id_473, (MuVarNode [1]){id_466}, 1);
    bldr_33->new_dest_clause(bldr_33, id_478, id_473, (MuVarNode [1]){id_467}, 1);
    id_479 = bldr_33->gen_sym(bldr_33, NULL);
    bldr_33->new_switch(bldr_33, id_479, id_465, id_475, id_476, (MuConstNode [2]){id_466, id_467}, (MuDestClause [2]){id_477, id_478}, 2);
    bldr_33->new_bb(bldr_33, id_472, (MuID [1]){id_475}, (MuTypeNode [1]){id_465}, 1, MU_NO_ID, (MuInstNode [1]){id_479}, 1);
    id_480 = bldr_33->gen_sym(bldr_33, "@fig_v1.blk1.rtn");
    id_481 = bldr_33->gen_sym(bldr_33, NULL);
    bldr_33->new_ret(bldr_33, id_481, (MuVarNode [1]){id_480}, 1);
    bldr_33->new_bb(bldr_33, id_473, (MuID [1]){id_480}, (MuTypeNode [1]){id_465}, 1, MU_NO_ID, (MuInstNode [1]){id_481}, 1);
    id_482 = bldr_33->gen_sym(bldr_33, "@fig_v1.blk2.k");
    id_483 = bldr_33->gen_sym(bldr_33, "@fig_v1.blk2.k_1");
    id_484 = bldr_33->gen_sym(bldr_33, "@fig_v1.blk2.k_2");
    id_485 = bldr_33->gen_sym(bldr_33, "@fig_v1.blk2.res");
    id_486 = bldr_33->gen_sym(bldr_33, "@fig_v1.blk2.res1");
    id_487 = bldr_33->gen_sym(bldr_33, "@fig_v1.blk2.res2");
    id_488 = bldr_33->gen_sym(bldr_33, NULL);
    bldr_33->new_binop(bldr_33, id_488, id_483, MU_BINOP_SUB, id_465, id_482, id_467, MU_NO_ID);
    id_489 = bldr_33->gen_sym(bldr_33, NULL);
    bldr_33->new_call(bldr_33, id_489, (MuID [1]){id_486}, 1, id_469, id_470, (MuVarNode [1]){id_483}, 1, MU_NO_ID, MU_NO_ID);
    id_490 = bldr_33->gen_sym(bldr_33, NULL);
    bldr_33->new_binop(bldr_33, id_490, id_484, MU_BINOP_SUB, id_465, id_482, id_468, MU_NO_ID);
    id_491 = bldr_33->gen_sym(bldr_33, NULL);
    bldr_33->new_call(bldr_33, id_491, (MuID [1]){id_487}, 1, id_469, id_470, (MuVarNode [1]){id_484}, 1, MU_NO_ID, MU_NO_ID);
    id_492 = bldr_33->gen_sym(bldr_33, NULL);
    bldr_33->new_binop(bldr_33, id_492, id_485, MU_BINOP_ADD, id_465, id_486, id_487, MU_NO_ID);
    id_493 = bldr_33->gen_sym(bldr_33, NULL);
    bldr_33->new_ret(bldr_33, id_493, (MuVarNode [1]){id_485}, 1);
    bldr_33->new_bb(bldr_33, id_474, (MuID [1]){id_482}, (MuTypeNode [1]){id_465}, 1, MU_NO_ID, (MuInstNode [6]){id_488, id_489, id_490, id_491, id_492, id_493}, 6);
    bldr_33->new_func_ver(bldr_33, id_471, id_470, (MuBBNode [3]){id_472, id_473, id_474}, 3);
    bldr_33->load(bldr_33);
    mu_33->compile_to_sharedlib(mu_33, "test_fib.dylib", NULL, 0);
    printf("%s\n", "test_fib.dylib");
    return 0;
}
