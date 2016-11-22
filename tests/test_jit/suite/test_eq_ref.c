
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_11;
    MuCtx* ctx_11;
    MuIRBuilder* bldr_11;
    MuID id_104;
    MuID id_105;
    MuID id_106;
    MuID id_107;
    MuID id_108;
    MuID id_109;
    MuID id_110;
    MuID id_111;
    MuID id_112;
    MuID id_113;
    MuID id_114;
    MuID id_115;
    MuID id_116;
    MuID id_117;
    MuID id_118;
    mu_11 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_11 = mu_11->new_context(mu_11);
    bldr_11 = ctx_11->new_ir_builder(ctx_11);
    id_104 = bldr_11->gen_sym(bldr_11, "@i1");
    bldr_11->new_type_int(bldr_11, id_104, 1);
    id_105 = bldr_11->gen_sym(bldr_11, "@i64");
    bldr_11->new_type_int(bldr_11, id_105, 64);
    id_106 = bldr_11->gen_sym(bldr_11, "@refi64");
    bldr_11->new_type_ref(bldr_11, id_106, id_105);
    id_107 = bldr_11->gen_sym(bldr_11, "@NULL_refi64");
    bldr_11->new_const_null(bldr_11, id_107, id_106);
    id_108 = bldr_11->gen_sym(bldr_11, "@sig__i64");
    bldr_11->new_funcsig(bldr_11, id_108, NULL, 0, (MuTypeNode [1]){id_105}, 1);
    id_109 = bldr_11->gen_sym(bldr_11, "@test_fnc");
    bldr_11->new_func(bldr_11, id_109, id_108);
    id_110 = bldr_11->gen_sym(bldr_11, "@test_fnc_v1");
    id_111 = bldr_11->gen_sym(bldr_11, "@test_fnc_v1.blk0");
    id_112 = bldr_11->gen_sym(bldr_11, "@test_fnc_v1.blk0.r");
    id_113 = bldr_11->gen_sym(bldr_11, "@test_fnc_v1.blk0.cmp_res");
    id_114 = bldr_11->gen_sym(bldr_11, "@test_fnc_v1.blk0.res");
    id_115 = bldr_11->gen_sym(bldr_11, NULL);
    bldr_11->new_new(bldr_11, id_115, id_112, id_105, MU_NO_ID);
    id_116 = bldr_11->gen_sym(bldr_11, NULL);
    bldr_11->new_cmp(bldr_11, id_116, id_113, MU_CMP_EQ, id_106, id_112, id_107);
    id_117 = bldr_11->gen_sym(bldr_11, NULL);
    bldr_11->new_conv(bldr_11, id_117, id_114, MU_CONV_ZEXT, id_104, id_105, id_113);
    id_118 = bldr_11->gen_sym(bldr_11, NULL);
    bldr_11->new_ret(bldr_11, id_118, (MuVarNode [1]){id_114}, 1);
    bldr_11->new_bb(bldr_11, id_111, NULL, NULL, 0, MU_NO_ID, (MuInstNode [4]){id_115, id_116, id_117, id_118}, 4);
    bldr_11->new_func_ver(bldr_11, id_110, id_109, (MuBBNode [1]){id_111}, 1);
    bldr_11->load(bldr_11);
    mu_11->compile_to_sharedlib(mu_11, "test_eq_ref.dylib", NULL, 0);
    printf("%s\n", "test_eq_ref.dylib");
    return 0;
}
