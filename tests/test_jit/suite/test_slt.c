
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
#ifdef __APPLE__
    #define LIB_EXT ".dylib"
#elif __linux__
    #define LIB_EXT ".so"
#elif _WIN32
    #define LIB_EXT ".dll"
#endif
#define LIB_FILE_NAME(name) "lib" name LIB_EXT
int main(int argc, char** argv) {
    MuVM* mu_21;
    MuCtx* ctx_21;
    MuIRBuilder* bldr_21;
    MuID id_238;
    MuID id_239;
    MuID id_240;
    MuID id_241;
    MuID id_242;
    MuID id_243;
    MuID id_244;
    MuID id_245;
    MuID id_246;
    MuID id_247;
    MuID id_248;
    MuID id_249;
    MuID id_250;
    MuID id_251;
    MuID id_252;
    MuID id_253;
    MuID id_254;
    mu_21 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_21 = mu_21->new_context(mu_21);
    bldr_21 = ctx_21->new_ir_builder(ctx_21);
    id_238 = bldr_21->gen_sym(bldr_21, "@i1");
    bldr_21->new_type_int(bldr_21, id_238, 0x00000001ull);
    id_239 = bldr_21->gen_sym(bldr_21, "@i8");
    bldr_21->new_type_int(bldr_21, id_239, 0x00000008ull);
    id_240 = bldr_21->gen_sym(bldr_21, "@0xff_i8");
    bldr_21->new_const_int(bldr_21, id_240, id_239, 0x00000000000000ffull);
    id_241 = bldr_21->gen_sym(bldr_21, "@0x0a_i8");
    bldr_21->new_const_int(bldr_21, id_241, id_239, 0x000000000000000aull);
    id_242 = bldr_21->gen_sym(bldr_21, "@sig__i8");
    bldr_21->new_funcsig(bldr_21, id_242, NULL, 0, (MuTypeNode [1]){id_239}, 1);
    id_243 = bldr_21->gen_sym(bldr_21, "@test_fnc");
    bldr_21->new_func(bldr_21, id_243, id_242);
    id_244 = bldr_21->gen_sym(bldr_21, "@test_fnc_v1");
    id_245 = bldr_21->gen_sym(bldr_21, "@test_fnc_v1.blk0");
    id_246 = bldr_21->gen_sym(bldr_21, "@test_fnc_v1.blk0.cmp_res_1");
    id_247 = bldr_21->gen_sym(bldr_21, "@test_fnc_v1.blk0.cmp_res_2");
    id_248 = bldr_21->gen_sym(bldr_21, "@test_fnc_v1.blk0.bin_res");
    id_249 = bldr_21->gen_sym(bldr_21, "@test_fnc_v1.blk0.res");
    id_250 = bldr_21->gen_sym(bldr_21, NULL);
    bldr_21->new_cmp(bldr_21, id_250, id_246, MU_CMP_SLT, id_239, id_241, id_240);
    id_251 = bldr_21->gen_sym(bldr_21, NULL);
    bldr_21->new_cmp(bldr_21, id_251, id_247, MU_CMP_SLT, id_239, id_240, id_240);
    id_252 = bldr_21->gen_sym(bldr_21, NULL);
    bldr_21->new_binop(bldr_21, id_252, id_248, MU_BINOP_OR, id_238, id_246, id_247, MU_NO_ID);
    id_253 = bldr_21->gen_sym(bldr_21, NULL);
    bldr_21->new_conv(bldr_21, id_253, id_249, MU_CONV_ZEXT, id_238, id_239, id_248);
    id_254 = bldr_21->gen_sym(bldr_21, NULL);
    bldr_21->new_ret(bldr_21, id_254, (MuVarNode [1]){id_249}, 1);
    bldr_21->new_bb(bldr_21, id_245, NULL, NULL, 0, MU_NO_ID, (MuInstNode [5]){id_250, id_251, id_252, id_253, id_254}, 5);
    bldr_21->new_func_ver(bldr_21, id_244, id_243, (MuBBNode [1]){id_245}, 1);
    bldr_21->load(bldr_21);
    mu_21->compile_to_sharedlib(mu_21, LIB_FILE_NAME("test_slt"), NULL, 0);
    return 0;
}
