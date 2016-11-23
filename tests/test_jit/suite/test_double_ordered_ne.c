
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_31;
    MuCtx* ctx_31;
    MuIRBuilder* bldr_31;
    MuID id_386;
    MuID id_387;
    MuID id_388;
    MuID id_389;
    MuID id_390;
    MuID id_391;
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
    mu_31 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_31 = mu_31->new_context(mu_31);
    bldr_31 = ctx_31->new_ir_builder(ctx_31);
    id_386 = bldr_31->gen_sym(bldr_31, "@dbl");
    bldr_31->new_type_double(bldr_31, id_386);
    id_387 = bldr_31->gen_sym(bldr_31, "@i1");
    bldr_31->new_type_int(bldr_31, id_387, 1);
    id_388 = bldr_31->gen_sym(bldr_31, "@i64");
    bldr_31->new_type_int(bldr_31, id_388, 64);
    id_389 = bldr_31->gen_sym(bldr_31, "@1_dbl");
    bldr_31->new_const_double(bldr_31, id_389, id_386, 1.00000000000000000000);
    id_390 = bldr_31->gen_sym(bldr_31, "@3_dbl");
    bldr_31->new_const_double(bldr_31, id_390, id_386, 3.00000000000000000000);
    id_391 = bldr_31->gen_sym(bldr_31, "@zp3");
    bldr_31->new_const_double(bldr_31, id_391, id_386, 0.29999999999999998890);
    id_392 = bldr_31->gen_sym(bldr_31, "@sig__i64");
    bldr_31->new_funcsig(bldr_31, id_392, NULL, 0, (MuTypeNode [1]){id_388}, 1);
    id_393 = bldr_31->gen_sym(bldr_31, "@test_fnc");
    bldr_31->new_func(bldr_31, id_393, id_392);
    id_394 = bldr_31->gen_sym(bldr_31, "@test_fnc.v1");
    id_395 = bldr_31->gen_sym(bldr_31, "@test_fnc.v1.blk0");
    id_396 = bldr_31->gen_sym(bldr_31, "@test_fnc.v1.blk0.k");
    id_397 = bldr_31->gen_sym(bldr_31, "@test_fnc.v1.blk0.cmpres");
    id_398 = bldr_31->gen_sym(bldr_31, "@test_fnc.v1.blk0.res");
    id_399 = bldr_31->gen_sym(bldr_31, NULL);
    bldr_31->new_binop(bldr_31, id_399, id_396, MU_BINOP_FDIV, id_386, id_389, id_390, MU_NO_ID);
    id_400 = bldr_31->gen_sym(bldr_31, NULL);
    bldr_31->new_cmp(bldr_31, id_400, id_397, MU_CMP_FONE, id_386, id_396, id_391);
    id_401 = bldr_31->gen_sym(bldr_31, NULL);
    bldr_31->new_conv(bldr_31, id_401, id_398, MU_CONV_ZEXT, id_387, id_388, id_397);
    id_402 = bldr_31->gen_sym(bldr_31, NULL);
    bldr_31->new_ret(bldr_31, id_402, (MuVarNode [1]){id_398}, 1);
    bldr_31->new_bb(bldr_31, id_395, NULL, NULL, 0, MU_NO_ID, (MuInstNode [4]){id_399, id_400, id_401, id_402}, 4);
    bldr_31->new_func_ver(bldr_31, id_394, id_393, (MuBBNode [1]){id_395}, 1);
    bldr_31->load(bldr_31);
    mu_31->compile_to_sharedlib(mu_31, "test_double_ordered_ne.dylib", NULL, 0);
    printf("%s\n", "test_double_ordered_ne.dylib");
    return 0;
}
