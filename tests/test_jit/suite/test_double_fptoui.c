
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_40;
    MuCtx* ctx_40;
    MuIRBuilder* bldr_40;
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
    mu_40 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_40 = mu_40->new_context(mu_40);
    bldr_40 = ctx_40->new_ir_builder(ctx_40);
    id_500 = bldr_40->gen_sym(bldr_40, "@dbl");
    bldr_40->new_type_double(bldr_40, id_500);
    id_501 = bldr_40->gen_sym(bldr_40, "@i1");
    bldr_40->new_type_int(bldr_40, id_501, 1);
    id_502 = bldr_40->gen_sym(bldr_40, "@i64");
    bldr_40->new_type_int(bldr_40, id_502, 64);
    id_503 = bldr_40->gen_sym(bldr_40, "@pi");
    bldr_40->new_const_double(bldr_40, id_503, id_500, 3.14159260000000006841);
    id_504 = bldr_40->gen_sym(bldr_40, "@sig__i64");
    bldr_40->new_funcsig(bldr_40, id_504, NULL, 0, (MuTypeNode [1]){id_502}, 1);
    id_505 = bldr_40->gen_sym(bldr_40, "@test_fnc");
    bldr_40->new_func(bldr_40, id_505, id_504);
    id_506 = bldr_40->gen_sym(bldr_40, "@test_fnc.v1");
    id_507 = bldr_40->gen_sym(bldr_40, "@test_fnc.v1.blk0");
    id_508 = bldr_40->gen_sym(bldr_40, "@test_fnc.v1.blk0.res");
    id_509 = bldr_40->gen_sym(bldr_40, NULL);
    bldr_40->new_conv(bldr_40, id_509, id_508, MU_CONV_FPTOUI, id_500, id_502, id_503);
    id_510 = bldr_40->gen_sym(bldr_40, NULL);
    bldr_40->new_ret(bldr_40, id_510, (MuVarNode [1]){id_508}, 1);
    bldr_40->new_bb(bldr_40, id_507, NULL, NULL, 0, MU_NO_ID, (MuInstNode [2]){id_509, id_510}, 2);
    bldr_40->new_func_ver(bldr_40, id_506, id_505, (MuBBNode [1]){id_507}, 1);
    bldr_40->load(bldr_40);
    mu_40->compile_to_sharedlib(mu_40, "test_double_fptoui.dylib", NULL, 0);
    printf("%s\n", "test_double_fptoui.dylib");
    return 0;
}
