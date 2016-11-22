
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_17;
    MuCtx* ctx_17;
    MuIRBuilder* bldr_17;
    MuID id_198;
    MuID id_199;
    MuID id_200;
    MuID id_201;
    MuID id_202;
    MuID id_203;
    MuID id_204;
    MuID id_205;
    MuID id_206;
    MuID id_207;
    MuID id_208;
    MuID id_209;
    MuID id_210;
    MuID id_211;
    MuID id_212;
    MuID id_213;
    MuID id_214;
    mu_17 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_17 = mu_17->new_context(mu_17);
    bldr_17 = ctx_17->new_ir_builder(ctx_17);
    id_198 = bldr_17->gen_sym(bldr_17, "@i1");
    bldr_17->new_type_int(bldr_17, id_198, 1);
    id_199 = bldr_17->gen_sym(bldr_17, "@i8");
    bldr_17->new_type_int(bldr_17, id_199, 8);
    id_200 = bldr_17->gen_sym(bldr_17, "@0xff_i8");
    bldr_17->new_const_int(bldr_17, id_200, id_199, 255);
    id_201 = bldr_17->gen_sym(bldr_17, "@0x0a_i8");
    bldr_17->new_const_int(bldr_17, id_201, id_199, 10);
    id_202 = bldr_17->gen_sym(bldr_17, "@sig__i8");
    bldr_17->new_funcsig(bldr_17, id_202, NULL, 0, (MuTypeNode [1]){id_199}, 1);
    id_203 = bldr_17->gen_sym(bldr_17, "@test_fnc");
    bldr_17->new_func(bldr_17, id_203, id_202);
    id_204 = bldr_17->gen_sym(bldr_17, "@test_fnc_v1");
    id_205 = bldr_17->gen_sym(bldr_17, "@test_fnc_v1.blk0");
    id_206 = bldr_17->gen_sym(bldr_17, "@test_fnc_v1.blk0.cmp_res_1");
    id_207 = bldr_17->gen_sym(bldr_17, "@test_fnc_v1.blk0.cmp_res_2");
    id_208 = bldr_17->gen_sym(bldr_17, "@test_fnc_v1.blk0.bin_res");
    id_209 = bldr_17->gen_sym(bldr_17, "@test_fnc_v1.blk0.res");
    id_210 = bldr_17->gen_sym(bldr_17, NULL);
    bldr_17->new_cmp(bldr_17, id_210, id_206, MU_CMP_SLT, id_199, id_201, id_200);
    id_211 = bldr_17->gen_sym(bldr_17, NULL);
    bldr_17->new_cmp(bldr_17, id_211, id_207, MU_CMP_SLT, id_199, id_200, id_200);
    id_212 = bldr_17->gen_sym(bldr_17, NULL);
    bldr_17->new_binop(bldr_17, id_212, id_208, MU_BINOP_OR, id_198, id_206, id_207, MU_NO_ID);
    id_213 = bldr_17->gen_sym(bldr_17, NULL);
    bldr_17->new_conv(bldr_17, id_213, id_209, MU_CONV_ZEXT, id_198, id_199, id_208);
    id_214 = bldr_17->gen_sym(bldr_17, NULL);
    bldr_17->new_ret(bldr_17, id_214, (MuVarNode [1]){id_209}, 1);
    bldr_17->new_bb(bldr_17, id_205, NULL, NULL, 0, MU_NO_ID, (MuInstNode [5]){id_210, id_211, id_212, id_213, id_214}, 5);
    bldr_17->new_func_ver(bldr_17, id_204, id_203, (MuBBNode [1]){id_205}, 1);
    bldr_17->load(bldr_17);
    mu_17->compile_to_sharedlib(mu_17, "test_slt.dylib", NULL, 0);
    printf("%s\n", "test_slt.dylib");
    return 0;
}
