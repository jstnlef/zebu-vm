
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_34;
    MuCtx* ctx_34;
    MuIRBuilder* bldr_34;
    MuID id_494;
    MuID id_495;
    MuID id_496;
    MuID id_497;
    MuID id_498;
    MuID id_499;
    MuID id_500;
    MuID id_501;
    MuID id_502;
    MuID id_503;
    MuID id_504;
    MuID id_505;
    MuID id_506;
    MuID id_507;
    MuID id_508;
    MuID id_509;
    MuID id_510;
    MuID id_511;
    MuID id_512;
    MuID id_513;
    MuID id_514;
    MuID id_515;
    MuID id_516;
    MuID id_517;
    MuID id_518;
    MuID id_519;
    MuID id_520;
    MuID id_521;
    MuID id_522;
    MuID id_523;
    MuID id_524;
    MuID id_525;
    mu_34 = mu_fastimpl_new_with_opts("init_muinit_mu --log-level=none --aot-emit-dir=emit");
    ctx_34 = mu_34->new_context(mu_34);
    bldr_34 = ctx_34->new_ir_builder(ctx_34);
    id_494 = bldr_34->gen_sym(bldr_34, "@i64");
    bldr_34->new_type_int(bldr_34, id_494, 64);
    id_495 = bldr_34->gen_sym(bldr_34, "@0_i64");
    bldr_34->new_const_int(bldr_34, id_495, id_494, 0);
    id_496 = bldr_34->gen_sym(bldr_34, "@1_i64");
    bldr_34->new_const_int(bldr_34, id_496, id_494, 1);
    id_497 = bldr_34->gen_sym(bldr_34, "@sig_i64_i64");
    bldr_34->new_funcsig(bldr_34, id_497, (MuTypeNode [1]){id_494}, 1, (MuTypeNode [1]){id_494}, 1);
    id_498 = bldr_34->gen_sym(bldr_34, "@fac");
    bldr_34->new_func(bldr_34, id_498, id_497);
    id_499 = bldr_34->gen_sym(bldr_34, "@fac.v1");
    id_500 = bldr_34->gen_sym(bldr_34, "@fac.v1.blk0");
    id_501 = bldr_34->gen_sym(bldr_34, "@fac.v1.blk1");
    id_502 = bldr_34->gen_sym(bldr_34, "@fac.v1.blk2");
    id_503 = bldr_34->gen_sym(bldr_34, "@fac.v1.blk3");
    id_504 = bldr_34->gen_sym(bldr_34, "@fac.v1.blk0.k");
    id_505 = bldr_34->gen_sym(bldr_34, NULL);
    bldr_34->new_dest_clause(bldr_34, id_505, id_501, (MuVarNode [3]){id_496, id_495, id_504}, 3);
    id_506 = bldr_34->gen_sym(bldr_34, NULL);
    bldr_34->new_branch(bldr_34, id_506, id_505);
    bldr_34->new_bb(bldr_34, id_500, (MuID [1]){id_504}, (MuTypeNode [1]){id_494}, 1, MU_NO_ID, (MuInstNode [1]){id_506}, 1);
    id_507 = bldr_34->gen_sym(bldr_34, "@fac.v1.blk1.prod");
    id_508 = bldr_34->gen_sym(bldr_34, "@fac.v1.blk1.i");
    id_509 = bldr_34->gen_sym(bldr_34, "@fac.v1.blk1.end");
    id_510 = bldr_34->gen_sym(bldr_34, "@fac.v1.blk1.cmpres");
    id_511 = bldr_34->gen_sym(bldr_34, NULL);
    bldr_34->new_cmp(bldr_34, id_511, id_510, MU_CMP_EQ, id_494, id_508, id_509);
    id_512 = bldr_34->gen_sym(bldr_34, NULL);
    id_513 = bldr_34->gen_sym(bldr_34, NULL);
    bldr_34->new_dest_clause(bldr_34, id_513, id_503, (MuVarNode [1]){id_507}, 1);
    id_514 = bldr_34->gen_sym(bldr_34, NULL);
    bldr_34->new_dest_clause(bldr_34, id_514, id_502, (MuVarNode [3]){id_507, id_508, id_509}, 3);
    bldr_34->new_branch2(bldr_34, id_512, id_510, id_513, id_514);
    bldr_34->new_bb(bldr_34, id_501, (MuID [3]){id_507, id_508, id_509}, (MuTypeNode [3]){id_494, id_494, id_494}, 3, MU_NO_ID, (MuInstNode [2]){id_511, id_512}, 2);
    id_515 = bldr_34->gen_sym(bldr_34, "@fac.v1.blk2.prod");
    id_516 = bldr_34->gen_sym(bldr_34, "@fac.v1.blk2.i");
    id_517 = bldr_34->gen_sym(bldr_34, "@fac.v1.blk2.end");
    id_518 = bldr_34->gen_sym(bldr_34, "@fac.v1.blk2.prod_res");
    id_519 = bldr_34->gen_sym(bldr_34, "@fac.v1.blk2.i_res");
    id_520 = bldr_34->gen_sym(bldr_34, NULL);
    bldr_34->new_binop(bldr_34, id_520, id_519, MU_BINOP_ADD, id_494, id_516, id_496, MU_NO_ID);
    id_521 = bldr_34->gen_sym(bldr_34, NULL);
    bldr_34->new_binop(bldr_34, id_521, id_518, MU_BINOP_MUL, id_494, id_515, id_519, MU_NO_ID);
    id_522 = bldr_34->gen_sym(bldr_34, NULL);
    bldr_34->new_dest_clause(bldr_34, id_522, id_501, (MuVarNode [3]){id_518, id_519, id_517}, 3);
    id_523 = bldr_34->gen_sym(bldr_34, NULL);
    bldr_34->new_branch(bldr_34, id_523, id_522);
    bldr_34->new_bb(bldr_34, id_502, (MuID [3]){id_515, id_516, id_517}, (MuTypeNode [3]){id_494, id_494, id_494}, 3, MU_NO_ID, (MuInstNode [3]){id_520, id_521, id_523}, 3);
    id_524 = bldr_34->gen_sym(bldr_34, "@fac.v1.blk3.rtn");
    id_525 = bldr_34->gen_sym(bldr_34, NULL);
    bldr_34->new_ret(bldr_34, id_525, (MuVarNode [1]){id_524}, 1);
    bldr_34->new_bb(bldr_34, id_503, (MuID [1]){id_524}, (MuTypeNode [1]){id_494}, 1, MU_NO_ID, (MuInstNode [1]){id_525}, 1);
    bldr_34->new_func_ver(bldr_34, id_499, id_498, (MuBBNode [4]){id_500, id_501, id_502, id_503}, 4);
    bldr_34->load(bldr_34);
    mu_34->compile_to_sharedlib(mu_34, "test_fac.dylib", NULL, 0);
    printf("%s\n", "test_fac.dylib");
    return 0;
}
