
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_37;
    MuCtx* ctx_37;
    MuIRBuilder* bldr_37;
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
    mu_37 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_37 = mu_37->new_context(mu_37);
    bldr_37 = ctx_37->new_ir_builder(ctx_37);
    id_467 = bldr_37->gen_sym(bldr_37, "@dbl");
    bldr_37->new_type_double(bldr_37, id_467);
    id_468 = bldr_37->gen_sym(bldr_37, "@i1");
    bldr_37->new_type_int(bldr_37, id_468, 1);
    id_469 = bldr_37->gen_sym(bldr_37, "@i64");
    bldr_37->new_type_int(bldr_37, id_469, 64);
    id_470 = bldr_37->gen_sym(bldr_37, "@k");
    bldr_37->new_const_int(bldr_37, id_470, id_469, -42);
    id_471 = bldr_37->gen_sym(bldr_37, "@sig__dbl");
    bldr_37->new_funcsig(bldr_37, id_471, NULL, 0, (MuTypeNode [1]){id_467}, 1);
    id_472 = bldr_37->gen_sym(bldr_37, "@test_fnc");
    bldr_37->new_func(bldr_37, id_472, id_471);
    id_473 = bldr_37->gen_sym(bldr_37, "@test_fnc.v1");
    id_474 = bldr_37->gen_sym(bldr_37, "@test_fnc.v1.blk0");
    id_475 = bldr_37->gen_sym(bldr_37, "@test_fnc.v1.blk0.res");
    id_476 = bldr_37->gen_sym(bldr_37, NULL);
    bldr_37->new_conv(bldr_37, id_476, id_475, MU_CONV_SITOFP, id_469, id_467, id_470);
    id_477 = bldr_37->gen_sym(bldr_37, NULL);
    bldr_37->new_ret(bldr_37, id_477, (MuVarNode [1]){id_475}, 1);
    bldr_37->new_bb(bldr_37, id_474, NULL, NULL, 0, MU_NO_ID, (MuInstNode [2]){id_476, id_477}, 2);
    bldr_37->new_func_ver(bldr_37, id_473, id_472, (MuBBNode [1]){id_474}, 1);
    bldr_37->load(bldr_37);
    mu_37->compile_to_sharedlib(mu_37, "test_double_sitofp.dylib", NULL, 0);
    printf("%s\n", "test_double_sitofp.dylib");
    return 0;
}
