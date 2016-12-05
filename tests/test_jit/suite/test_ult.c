
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
    MuVM* mu_23;
    MuCtx* ctx_23;
    MuIRBuilder* bldr_23;
    MuID id_272;
    MuID id_273;
    MuID id_274;
    MuID id_275;
    MuID id_276;
    MuID id_277;
    MuID id_278;
    MuID id_279;
    MuID id_280;
    MuID id_281;
    MuID id_282;
    MuID id_283;
    MuID id_284;
    MuID id_285;
    MuID id_286;
    MuID id_287;
    MuID id_288;
    mu_23 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_23 = mu_23->new_context(mu_23);
    bldr_23 = ctx_23->new_ir_builder(ctx_23);
    id_272 = bldr_23->gen_sym(bldr_23, "@i1");
    bldr_23->new_type_int(bldr_23, id_272, 0x00000001ull);
    id_273 = bldr_23->gen_sym(bldr_23, "@i8");
    bldr_23->new_type_int(bldr_23, id_273, 0x00000008ull);
    id_274 = bldr_23->gen_sym(bldr_23, "@0xff_i8");
    bldr_23->new_const_int(bldr_23, id_274, id_273, 0x00000000000000ffull);
    id_275 = bldr_23->gen_sym(bldr_23, "@0x0a_i8");
    bldr_23->new_const_int(bldr_23, id_275, id_273, 0x000000000000000aull);
    id_276 = bldr_23->gen_sym(bldr_23, "@sig__i8");
    bldr_23->new_funcsig(bldr_23, id_276, NULL, 0, (MuTypeNode [1]){id_273}, 1);
    id_277 = bldr_23->gen_sym(bldr_23, "@test_fnc");
    bldr_23->new_func(bldr_23, id_277, id_276);
    id_278 = bldr_23->gen_sym(bldr_23, "@test_fnc_v1");
    id_279 = bldr_23->gen_sym(bldr_23, "@test_fnc_v1.blk0");
    id_280 = bldr_23->gen_sym(bldr_23, "@test_fnc_v1.blk0.cmp_res_1");
    id_281 = bldr_23->gen_sym(bldr_23, "@test_fnc_v1.blk0.cmp_res_2");
    id_282 = bldr_23->gen_sym(bldr_23, "@test_fnc_v1.blk0.bin_res");
    id_283 = bldr_23->gen_sym(bldr_23, "@test_fnc_v1.blk0.res");
    id_284 = bldr_23->gen_sym(bldr_23, NULL);
    bldr_23->new_cmp(bldr_23, id_284, id_280, MU_CMP_ULT, id_273, id_274, id_275);
    id_285 = bldr_23->gen_sym(bldr_23, NULL);
    bldr_23->new_cmp(bldr_23, id_285, id_281, MU_CMP_ULT, id_273, id_274, id_274);
    id_286 = bldr_23->gen_sym(bldr_23, NULL);
    bldr_23->new_binop(bldr_23, id_286, id_282, MU_BINOP_OR, id_272, id_280, id_281, MU_NO_ID);
    id_287 = bldr_23->gen_sym(bldr_23, NULL);
    bldr_23->new_conv(bldr_23, id_287, id_283, MU_CONV_ZEXT, id_272, id_273, id_282);
    id_288 = bldr_23->gen_sym(bldr_23, NULL);
    bldr_23->new_ret(bldr_23, id_288, (MuVarNode [1]){id_283}, 1);
    bldr_23->new_bb(bldr_23, id_279, NULL, NULL, 0, MU_NO_ID, (MuInstNode [5]){id_284, id_285, id_286, id_287, id_288}, 5);
    bldr_23->new_func_ver(bldr_23, id_278, id_277, (MuBBNode [1]){id_279}, 1);
    bldr_23->load(bldr_23);
    mu_23->compile_to_sharedlib(mu_23, LIB_FILE_NAME("test_ult"), NULL, 0);
    return 0;
}
