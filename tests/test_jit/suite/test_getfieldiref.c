
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_29;
    MuCtx* ctx_29;
    MuIRBuilder* bldr_29;
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
    MuID id_424;
    MuID id_425;
    MuID id_426;
    mu_29 = mu_fastimpl_new();
    ctx_29 = mu_29->new_context(mu_29);
    bldr_29 = ctx_29->new_ir_builder(ctx_29);
    id_412 = bldr_29->gen_sym(bldr_29, "@i8");
    bldr_29->new_type_int(bldr_29, id_412, 8);
    id_413 = bldr_29->gen_sym(bldr_29, "@i32");
    bldr_29->new_type_int(bldr_29, id_413, 32);
    id_414 = bldr_29->gen_sym(bldr_29, "@i64");
    bldr_29->new_type_int(bldr_29, id_414, 64);
    id_415 = bldr_29->gen_sym(bldr_29, "@stt");
    bldr_29->new_type_struct(bldr_29, id_415, (MuTypeNode [3]){id_412, id_414, id_413}, 3);
    id_416 = bldr_29->gen_sym(bldr_29, "@pstt");
    bldr_29->new_type_uptr(bldr_29, id_416, id_415);
    id_417 = bldr_29->gen_sym(bldr_29, "@sig_pstt_i32");
    bldr_29->new_funcsig(bldr_29, id_417, (MuTypeNode [1]){id_416}, 1, (MuTypeNode [1]){id_413}, 1);
    id_418 = bldr_29->gen_sym(bldr_29, "@test_fnc");
    bldr_29->new_func(bldr_29, id_418, id_417);
    id_419 = bldr_29->gen_sym(bldr_29, "@test_fnc.v1");
    id_420 = bldr_29->gen_sym(bldr_29, "@test_fnc.v1.blk0");
    id_421 = bldr_29->gen_sym(bldr_29, "@test_fnc.v1.blk0.ps");
    id_422 = bldr_29->gen_sym(bldr_29, "@test_fnc.v1.blk0.pfld");
    id_423 = bldr_29->gen_sym(bldr_29, "@test_fnc.v1.blk0.res");
    id_424 = bldr_29->gen_sym(bldr_29, NULL);
    bldr_29->new_getfieldiref(bldr_29, id_424, id_422, true, id_415, 2, id_421);
    id_425 = bldr_29->gen_sym(bldr_29, NULL);
    bldr_29->new_load(bldr_29, id_425, id_423, true, MU_ORD_NOT_ATOMIC, id_413, id_422, MU_NO_ID);
    id_426 = bldr_29->gen_sym(bldr_29, NULL);
    bldr_29->new_ret(bldr_29, id_426, (MuVarNode [1]){id_423}, 1);
    bldr_29->new_bb(bldr_29, id_420, (MuID [1]){id_421}, (MuTypeNode [1]){id_416}, 1, MU_NO_ID, (MuInstNode [3]){id_424, id_425, id_426}, 3);
    bldr_29->new_func_ver(bldr_29, id_419, id_418, (MuBBNode [1]){id_420}, 1);
    bldr_29->load(bldr_29);
    mu_29->compile_to_sharedlib(mu_29, "test_getfieldiref.dylib", NULL, 0);
    printf("%s\n", "test_getfieldiref.dylib");
    return 0;
}
