
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_20;
    MuCtx* ctx_20;
    MuIRBuilder* bldr_20;
    MuID id_247;
    MuID id_248;
    MuID id_249;
    MuID id_250;
    MuID id_251;
    MuID id_252;
    MuID id_253;
    MuID id_254;
    MuID id_255;
    MuID id_256;
    MuID id_257;
    MuID id_258;
    MuID id_259;
    MuID id_260;
    MuID id_261;
    MuID id_262;
    MuID id_263;
    MuID id_264;
    MuID id_265;
    MuID id_266;
    MuID id_267;
    MuID id_268;
    MuID id_269;
    MuID id_270;
    MuID id_271;
    MuID id_272;
    MuID id_273;
    mu_20 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_20 = mu_20->new_context(mu_20);
    bldr_20 = ctx_20->new_ir_builder(ctx_20);
    id_247 = bldr_20->gen_sym(bldr_20, "@i8");
    bldr_20->new_type_int(bldr_20, id_247, 8);
    id_248 = bldr_20->gen_sym(bldr_20, "@i64");
    bldr_20->new_type_int(bldr_20, id_248, 64);
    id_249 = bldr_20->gen_sym(bldr_20, "@TRUE");
    bldr_20->new_const_int(bldr_20, id_249, id_247, 1);
    id_250 = bldr_20->gen_sym(bldr_20, "@10_i64");
    bldr_20->new_const_int(bldr_20, id_250, id_248, 10);
    id_251 = bldr_20->gen_sym(bldr_20, "@20_i64");
    bldr_20->new_const_int(bldr_20, id_251, id_248, 20);
    id_252 = bldr_20->gen_sym(bldr_20, "@sig_i8_i64");
    bldr_20->new_funcsig(bldr_20, id_252, (MuTypeNode [1]){id_247}, 1, (MuTypeNode [1]){id_248}, 1);
    id_253 = bldr_20->gen_sym(bldr_20, "@test_fnc");
    bldr_20->new_func(bldr_20, id_253, id_252);
    id_254 = bldr_20->gen_sym(bldr_20, "@test_fnc.v1");
    id_255 = bldr_20->gen_sym(bldr_20, "@test_fnc.v1.blk0");
    id_256 = bldr_20->gen_sym(bldr_20, "@test_fnc.v1.blk1");
    id_257 = bldr_20->gen_sym(bldr_20, "@test_fnc.v1.blk2");
    id_258 = bldr_20->gen_sym(bldr_20, "@test_fnc.v1.blk0.sel");
    id_259 = bldr_20->gen_sym(bldr_20, "@test_fnc.v1.blk0.flag");
    id_260 = bldr_20->gen_sym(bldr_20, NULL);
    bldr_20->new_cmp(bldr_20, id_260, id_259, MU_CMP_EQ, id_247, id_258, id_249);
    id_261 = bldr_20->gen_sym(bldr_20, NULL);
    id_262 = bldr_20->gen_sym(bldr_20, NULL);
    bldr_20->new_dest_clause(bldr_20, id_262, id_256, (MuVarNode [2]){id_250, id_251}, 2);
    id_263 = bldr_20->gen_sym(bldr_20, NULL);
    bldr_20->new_dest_clause(bldr_20, id_263, id_257, (MuVarNode [2]){id_250, id_251}, 2);
    bldr_20->new_branch2(bldr_20, id_261, id_259, id_262, id_263);
    bldr_20->new_bb(bldr_20, id_255, (MuID [1]){id_258}, (MuTypeNode [1]){id_247}, 1, MU_NO_ID, (MuInstNode [2]){id_260, id_261}, 2);
    id_264 = bldr_20->gen_sym(bldr_20, "@test_fnc.v1.blk1.a");
    id_265 = bldr_20->gen_sym(bldr_20, "@test_fnc.v1.blk1.b");
    id_266 = bldr_20->gen_sym(bldr_20, "@test_fnc.v1.blk1.res");
    id_267 = bldr_20->gen_sym(bldr_20, NULL);
    bldr_20->new_binop(bldr_20, id_267, id_266, MU_BINOP_ADD, id_248, id_264, id_265, MU_NO_ID);
    id_268 = bldr_20->gen_sym(bldr_20, NULL);
    bldr_20->new_ret(bldr_20, id_268, (MuVarNode [1]){id_266}, 1);
    bldr_20->new_bb(bldr_20, id_256, (MuID [2]){id_264, id_265}, (MuTypeNode [2]){id_248, id_248}, 2, MU_NO_ID, (MuInstNode [2]){id_267, id_268}, 2);
    id_269 = bldr_20->gen_sym(bldr_20, "@test_fnc.v1.blk2.a");
    id_270 = bldr_20->gen_sym(bldr_20, "@test_fnc.v1.blk2.b");
    id_271 = bldr_20->gen_sym(bldr_20, "@test_fnc.v1.blk2.res");
    id_272 = bldr_20->gen_sym(bldr_20, NULL);
    bldr_20->new_binop(bldr_20, id_272, id_271, MU_BINOP_MUL, id_248, id_269, id_270, MU_NO_ID);
    id_273 = bldr_20->gen_sym(bldr_20, NULL);
    bldr_20->new_ret(bldr_20, id_273, (MuVarNode [1]){id_271}, 1);
    bldr_20->new_bb(bldr_20, id_257, (MuID [2]){id_269, id_270}, (MuTypeNode [2]){id_248, id_248}, 2, MU_NO_ID, (MuInstNode [2]){id_272, id_273}, 2);
    bldr_20->new_func_ver(bldr_20, id_254, id_253, (MuBBNode [3]){id_255, id_256, id_257}, 3);
    bldr_20->load(bldr_20);
    mu_20->compile_to_sharedlib(mu_20, "test_branch2.dylib", NULL, 0);
    printf("%s\n", "test_branch2.dylib");
    return 0;
}
