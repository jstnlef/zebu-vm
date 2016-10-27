
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_14;
    MuCtx* ctx_14;
    MuIRBuilder* bldr_14;
    MuID id_147;
    MuID id_148;
    MuID id_149;
    MuID id_150;
    MuID id_151;
    MuID id_152;
    MuID id_153;
    MuID id_154;
    MuID id_155;
    MuID id_156;
    MuID id_157;
    MuID id_158;
    MuID id_159;
    MuID id_160;
    MuID id_161;
    MuID id_162;
    MuID id_163;
    mu_14 = mu_fastimpl_new();
    ctx_14 = mu_14->new_context(mu_14);
    bldr_14 = ctx_14->new_ir_builder(ctx_14);
    id_147 = bldr_14->gen_sym(bldr_14, "@i1");
    bldr_14->new_type_int(bldr_14, id_147, 1);
    id_148 = bldr_14->gen_sym(bldr_14, "@i8");
    bldr_14->new_type_int(bldr_14, id_148, 8);
    id_149 = bldr_14->gen_sym(bldr_14, "@0xff_i8");
    bldr_14->new_const_int(bldr_14, id_149, id_148, 255);
    id_150 = bldr_14->gen_sym(bldr_14, "@0x0a_i8");
    bldr_14->new_const_int(bldr_14, id_150, id_148, 10);
    id_151 = bldr_14->gen_sym(bldr_14, "@sig__i8");
    bldr_14->new_funcsig(bldr_14, id_151, NULL, 0, (MuTypeNode [1]){id_148}, 1);
    id_152 = bldr_14->gen_sym(bldr_14, "@test_fnc");
    bldr_14->new_func(bldr_14, id_152, id_151);
    id_153 = bldr_14->gen_sym(bldr_14, "@test_fnc_v1");
    id_154 = bldr_14->gen_sym(bldr_14, "@test_fnc_v1.blk0");
    id_155 = bldr_14->gen_sym(bldr_14, "@test_fnc_v1.blk0.cmp_res_1");
    id_156 = bldr_14->gen_sym(bldr_14, "@test_fnc_v1.blk0.cmp_res_2");
    id_157 = bldr_14->gen_sym(bldr_14, "@test_fnc_v1.blk0.bin_res");
    id_158 = bldr_14->gen_sym(bldr_14, "@test_fnc_v1.blk0.res");
    id_159 = bldr_14->gen_sym(bldr_14, NULL);
    bldr_14->new_cmp(bldr_14, id_159, id_155, MU_CMP_SGE, id_148, id_149, id_150);
    id_160 = bldr_14->gen_sym(bldr_14, NULL);
    bldr_14->new_cmp(bldr_14, id_160, id_156, MU_CMP_SGE, id_148, id_149, id_149);
    id_161 = bldr_14->gen_sym(bldr_14, NULL);
    bldr_14->new_binop(bldr_14, id_161, id_157, MU_BINOP_XOR, id_147, id_155, id_156, MU_NO_ID);
    id_162 = bldr_14->gen_sym(bldr_14, NULL);
    bldr_14->new_conv(bldr_14, id_162, id_158, MU_CONV_ZEXT, id_147, id_148, id_157);
    id_163 = bldr_14->gen_sym(bldr_14, NULL);
    bldr_14->new_ret(bldr_14, id_163, (MuVarNode [1]){id_158}, 1);
    bldr_14->new_bb(bldr_14, id_154, NULL, NULL, 0, MU_NO_ID, (MuInstNode [5]){id_159, id_160, id_161, id_162, id_163}, 5);
    bldr_14->new_func_ver(bldr_14, id_153, id_152, (MuBBNode [1]){id_154}, 1);
    bldr_14->load(bldr_14);
    mu_14->compile_to_sharedlib(mu_14, "test_sge.dylib");
    printf("%s\n", "test_sge.dylib");
    return 0;
}
