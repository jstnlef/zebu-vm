
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_38;
    MuCtx* ctx_38;
    MuIRBuilder* bldr_38;
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
    mu_38 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_38 = mu_38->new_context(mu_38);
    bldr_38 = ctx_38->new_ir_builder(ctx_38);
    id_478 = bldr_38->gen_sym(bldr_38, "@dbl");
    bldr_38->new_type_double(bldr_38, id_478);
    id_479 = bldr_38->gen_sym(bldr_38, "@i1");
    bldr_38->new_type_int(bldr_38, id_479, 1);
    id_480 = bldr_38->gen_sym(bldr_38, "@i64");
    bldr_38->new_type_int(bldr_38, id_480, 64);
    id_481 = bldr_38->gen_sym(bldr_38, "@npi");
    bldr_38->new_const_double(bldr_38, id_481, id_478, -3.14159260000000006841);
    id_482 = bldr_38->gen_sym(bldr_38, "@sig__i64");
    bldr_38->new_funcsig(bldr_38, id_482, NULL, 0, (MuTypeNode [1]){id_480}, 1);
    id_483 = bldr_38->gen_sym(bldr_38, "@test_fnc");
    bldr_38->new_func(bldr_38, id_483, id_482);
    id_484 = bldr_38->gen_sym(bldr_38, "@test_fnc.v1");
    id_485 = bldr_38->gen_sym(bldr_38, "@test_fnc.v1.blk0");
    id_486 = bldr_38->gen_sym(bldr_38, "@test_fnc.v1.blk0.res");
    id_487 = bldr_38->gen_sym(bldr_38, NULL);
    bldr_38->new_conv(bldr_38, id_487, id_486, MU_CONV_FPTOSI, id_478, id_480, id_481);
    id_488 = bldr_38->gen_sym(bldr_38, NULL);
    bldr_38->new_ret(bldr_38, id_488, (MuVarNode [1]){id_486}, 1);
    bldr_38->new_bb(bldr_38, id_485, NULL, NULL, 0, MU_NO_ID, (MuInstNode [2]){id_487, id_488}, 2);
    bldr_38->new_func_ver(bldr_38, id_484, id_483, (MuBBNode [1]){id_485}, 1);
    bldr_38->load(bldr_38);
    mu_38->compile_to_sharedlib(mu_38, "test_double_fptosi.dylib", NULL, 0);
    printf("%s\n", "test_double_fptosi.dylib");
    return 0;
}
