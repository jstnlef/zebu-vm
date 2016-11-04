
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_28;
    MuCtx* ctx_28;
    MuIRBuilder* bldr_28;
    MuID id_392;
    MuID id_393;
    MuID id_394;
    MuID id_395;
    MuID id_396;
    MuID id_397;
    MuID id_398;
    MuID id_399;
    MuID id_400;
    MuID id_401;
    MuID id_402;
    MuID id_403;
    MuID id_404;
    MuID id_405;
    MuID id_406;
    MuID id_407;
    MuID id_408;
    MuID id_409;
    MuID id_410;
    MuID id_411;
    MuID id_412;
    MuID id_413;
    MuID id_414;
    MuID id_415;
    MuID id_416;
    MuID id_417;
    MuID id_418;
    MuID id_419;
    MuID id_420;
    MuID id_421;
    MuID id_422;
    MuID id_423;
    mu_28 = mu_fastimpl_new();
    ctx_28 = mu_28->new_context(mu_28);
    bldr_28 = ctx_28->new_ir_builder(ctx_28);
    id_392 = bldr_28->gen_sym(bldr_28, "@i64");
    bldr_28->new_type_int(bldr_28, id_392, 64);
    id_393 = bldr_28->gen_sym(bldr_28, "@0_i64");
    bldr_28->new_const_int(bldr_28, id_393, id_392, 0);
    id_394 = bldr_28->gen_sym(bldr_28, "@1_i64");
    bldr_28->new_const_int(bldr_28, id_394, id_392, 1);
    id_395 = bldr_28->gen_sym(bldr_28, "@sig_i64_i64");
    bldr_28->new_funcsig(bldr_28, id_395, (MuTypeNode [1]){id_392}, 1, (MuTypeNode [1]){id_392}, 1);
    id_396 = bldr_28->gen_sym(bldr_28, "@fac");
    bldr_28->new_func(bldr_28, id_396, id_395);
    id_397 = bldr_28->gen_sym(bldr_28, "@fac.v1");
    id_398 = bldr_28->gen_sym(bldr_28, "@fac.v1.blk0");
    id_399 = bldr_28->gen_sym(bldr_28, "@fac.v1.blk1");
    id_400 = bldr_28->gen_sym(bldr_28, "@fac.v1.blk2");
    id_401 = bldr_28->gen_sym(bldr_28, "@fac.v1.blk3");
    id_402 = bldr_28->gen_sym(bldr_28, "@fac.v1.blk0.k");
    id_403 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_dest_clause(bldr_28, id_403, id_399, (MuVarNode [3]){id_394, id_393, id_402}, 3);
    id_404 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_branch(bldr_28, id_404, id_403);
    bldr_28->new_bb(bldr_28, id_398, (MuID [1]){id_402}, (MuTypeNode [1]){id_392}, 1, MU_NO_ID, (MuInstNode [1]){id_404}, 1);
    id_405 = bldr_28->gen_sym(bldr_28, "@fac.v1.blk1.prod");
    id_406 = bldr_28->gen_sym(bldr_28, "@fac.v1.blk1.i");
    id_407 = bldr_28->gen_sym(bldr_28, "@fac.v1.blk1.end");
    id_408 = bldr_28->gen_sym(bldr_28, "@fac.v1.blk1.cmpres");
    id_409 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_cmp(bldr_28, id_409, id_408, MU_CMP_EQ, id_392, id_406, id_407);
    id_410 = bldr_28->gen_sym(bldr_28, NULL);
    id_411 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_dest_clause(bldr_28, id_411, id_401, (MuVarNode [1]){id_405}, 1);
    id_412 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_dest_clause(bldr_28, id_412, id_400, (MuVarNode [3]){id_405, id_406, id_407}, 3);
    bldr_28->new_branch2(bldr_28, id_410, id_408, id_411, id_412);
    bldr_28->new_bb(bldr_28, id_399, (MuID [3]){id_405, id_406, id_407}, (MuTypeNode [3]){id_392, id_392, id_392}, 3, MU_NO_ID, (MuInstNode [2]){id_409, id_410}, 2);
    id_413 = bldr_28->gen_sym(bldr_28, "@fac.v1.blk2.prod");
    id_414 = bldr_28->gen_sym(bldr_28, "@fac.v1.blk2.i");
    id_415 = bldr_28->gen_sym(bldr_28, "@fac.v1.blk2.end");
    id_416 = bldr_28->gen_sym(bldr_28, "@fac.v1.blk2.prod_res");
    id_417 = bldr_28->gen_sym(bldr_28, "@fac.v1.blk2.i_res");
    id_418 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_binop(bldr_28, id_418, id_417, MU_BINOP_ADD, id_392, id_414, id_394, MU_NO_ID);
    id_419 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_binop(bldr_28, id_419, id_416, MU_BINOP_MUL, id_392, id_413, id_417, MU_NO_ID);
    id_420 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_dest_clause(bldr_28, id_420, id_399, (MuVarNode [3]){id_416, id_417, id_415}, 3);
    id_421 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_branch(bldr_28, id_421, id_420);
    bldr_28->new_bb(bldr_28, id_400, (MuID [3]){id_413, id_414, id_415}, (MuTypeNode [3]){id_392, id_392, id_392}, 3, MU_NO_ID, (MuInstNode [3]){id_418, id_419, id_421}, 3);
    id_422 = bldr_28->gen_sym(bldr_28, "@fac.v1.blk3.rtn");
    id_423 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_ret(bldr_28, id_423, (MuVarNode [1]){id_422}, 1);
    bldr_28->new_bb(bldr_28, id_401, (MuID [1]){id_422}, (MuTypeNode [1]){id_392}, 1, MU_NO_ID, (MuInstNode [1]){id_423}, 1);
    bldr_28->new_func_ver(bldr_28, id_397, id_396, (MuBBNode [4]){id_398, id_399, id_400, id_401}, 4);
    bldr_28->load(bldr_28);
    mu_28->compile_to_sharedlib(mu_28, "test_fac.dylib", NULL, 0);
    printf("%s\n", "test_fac.dylib");
    return 0;
}
