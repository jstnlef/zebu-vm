
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_35;
    MuCtx* ctx_35;
    MuIRBuilder* bldr_35;
    MuID id_526;
    MuID id_527;
    MuID id_528;
    MuID id_529;
    MuID id_530;
    MuID id_531;
    MuID id_532;
    MuID id_533;
    MuID id_534;
    MuID id_535;
    MuID id_536;
    MuID id_537;
    MuID id_538;
    MuID id_539;
    MuID id_540;
    MuID id_541;
    MuID id_542;
    MuID id_543;
    MuID id_544;
    MuID id_545;
    MuID id_546;
    MuID id_547;
    MuID id_548;
    MuID id_549;
    MuID id_550;
    MuID id_551;
    MuID id_552;
    MuID id_553;
    MuID id_554;
    MuID id_555;
    MuID id_556;
    MuID id_557;
    mu_35 = mu_fastimpl_new_with_opts("init_muinit_mu --log-level=none --aot-emit-dir=emit");
    ctx_35 = mu_35->new_context(mu_35);
    bldr_35 = ctx_35->new_ir_builder(ctx_35);
    id_526 = bldr_35->gen_sym(bldr_35, "@i64");
    bldr_35->new_type_int(bldr_35, id_526, 64);
    id_527 = bldr_35->gen_sym(bldr_35, "@0_i64");
    bldr_35->new_const_int(bldr_35, id_527, id_526, 0);
    id_528 = bldr_35->gen_sym(bldr_35, "@1_i64");
    bldr_35->new_const_int(bldr_35, id_528, id_526, 1);
    id_529 = bldr_35->gen_sym(bldr_35, "@sig_i64_i64");
    bldr_35->new_funcsig(bldr_35, id_529, (MuTypeNode [1]){id_526}, 1, (MuTypeNode [1]){id_526}, 1);
    id_530 = bldr_35->gen_sym(bldr_35, "@milsum");
    bldr_35->new_func(bldr_35, id_530, id_529);
    id_531 = bldr_35->gen_sym(bldr_35, "@milsum.v1");
    id_532 = bldr_35->gen_sym(bldr_35, "@milsum.v1.blk0");
    id_533 = bldr_35->gen_sym(bldr_35, "@milsum.v1.blk1");
    id_534 = bldr_35->gen_sym(bldr_35, "@milsum.v1.blk2");
    id_535 = bldr_35->gen_sym(bldr_35, "@milsum.v1.blk3");
    id_536 = bldr_35->gen_sym(bldr_35, "@milsum.v1.blk0.k");
    id_537 = bldr_35->gen_sym(bldr_35, NULL);
    bldr_35->new_dest_clause(bldr_35, id_537, id_533, (MuVarNode [3]){id_527, id_527, id_536}, 3);
    id_538 = bldr_35->gen_sym(bldr_35, NULL);
    bldr_35->new_branch(bldr_35, id_538, id_537);
    bldr_35->new_bb(bldr_35, id_532, (MuID [1]){id_536}, (MuTypeNode [1]){id_526}, 1, MU_NO_ID, (MuInstNode [1]){id_538}, 1);
    id_539 = bldr_35->gen_sym(bldr_35, "@milsum.v1.blk1.acc");
    id_540 = bldr_35->gen_sym(bldr_35, "@milsum.v1.blk1.i");
    id_541 = bldr_35->gen_sym(bldr_35, "@milsum.v1.blk1.end");
    id_542 = bldr_35->gen_sym(bldr_35, "@milsum.v1.blk1.cmpres");
    id_543 = bldr_35->gen_sym(bldr_35, NULL);
    bldr_35->new_cmp(bldr_35, id_543, id_542, MU_CMP_EQ, id_526, id_540, id_541);
    id_544 = bldr_35->gen_sym(bldr_35, NULL);
    id_545 = bldr_35->gen_sym(bldr_35, NULL);
    bldr_35->new_dest_clause(bldr_35, id_545, id_535, (MuVarNode [1]){id_539}, 1);
    id_546 = bldr_35->gen_sym(bldr_35, NULL);
    bldr_35->new_dest_clause(bldr_35, id_546, id_534, (MuVarNode [3]){id_539, id_540, id_541}, 3);
    bldr_35->new_branch2(bldr_35, id_544, id_542, id_545, id_546);
    bldr_35->new_bb(bldr_35, id_533, (MuID [3]){id_539, id_540, id_541}, (MuTypeNode [3]){id_526, id_526, id_526}, 3, MU_NO_ID, (MuInstNode [2]){id_543, id_544}, 2);
    id_547 = bldr_35->gen_sym(bldr_35, "@milsum.v1.blk2.acc");
    id_548 = bldr_35->gen_sym(bldr_35, "@milsum.v1.blk2.i");
    id_549 = bldr_35->gen_sym(bldr_35, "@milsum.v1.blk2.end");
    id_550 = bldr_35->gen_sym(bldr_35, "@milsum.v1.blk2.acc_res");
    id_551 = bldr_35->gen_sym(bldr_35, "@milsum.v1.blk2.i_res");
    id_552 = bldr_35->gen_sym(bldr_35, NULL);
    bldr_35->new_binop(bldr_35, id_552, id_551, MU_BINOP_ADD, id_526, id_548, id_528, MU_NO_ID);
    id_553 = bldr_35->gen_sym(bldr_35, NULL);
    bldr_35->new_binop(bldr_35, id_553, id_550, MU_BINOP_ADD, id_526, id_547, id_551, MU_NO_ID);
    id_554 = bldr_35->gen_sym(bldr_35, NULL);
    bldr_35->new_dest_clause(bldr_35, id_554, id_533, (MuVarNode [3]){id_550, id_551, id_549}, 3);
    id_555 = bldr_35->gen_sym(bldr_35, NULL);
    bldr_35->new_branch(bldr_35, id_555, id_554);
    bldr_35->new_bb(bldr_35, id_534, (MuID [3]){id_547, id_548, id_549}, (MuTypeNode [3]){id_526, id_526, id_526}, 3, MU_NO_ID, (MuInstNode [3]){id_552, id_553, id_555}, 3);
    id_556 = bldr_35->gen_sym(bldr_35, "@milsum.v1.blk3.rtn");
    id_557 = bldr_35->gen_sym(bldr_35, NULL);
    bldr_35->new_ret(bldr_35, id_557, (MuVarNode [1]){id_556}, 1);
    bldr_35->new_bb(bldr_35, id_535, (MuID [1]){id_556}, (MuTypeNode [1]){id_526}, 1, MU_NO_ID, (MuInstNode [1]){id_557}, 1);
    bldr_35->new_func_ver(bldr_35, id_531, id_530, (MuBBNode [4]){id_532, id_533, id_534, id_535}, 4);
    bldr_35->load(bldr_35);
    mu_35->compile_to_sharedlib(mu_35, "test_milsum.dylib", NULL, 0);
    printf("%s\n", "test_milsum.dylib");
    return 0;
}
