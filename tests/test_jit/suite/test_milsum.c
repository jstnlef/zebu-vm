
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
    MuVM* mu_56;
    MuCtx* ctx_56;
    MuIRBuilder* bldr_56;
    MuID id_776;
    MuID id_777;
    MuID id_778;
    MuID id_779;
    MuID id_780;
    MuID id_781;
    MuID id_782;
    MuID id_783;
    MuID id_784;
    MuID id_785;
    MuID id_786;
    MuID id_787;
    MuID id_788;
    MuID id_789;
    MuID id_790;
    MuID id_791;
    MuID id_792;
    MuID id_793;
    MuID id_794;
    MuID id_795;
    MuID id_796;
    MuID id_797;
    MuID id_798;
    MuID id_799;
    MuID id_800;
    MuID id_801;
    MuID id_802;
    MuID id_803;
    MuID id_804;
    MuID id_805;
    MuID id_806;
    MuID id_807;
    mu_56 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_56 = mu_56->new_context(mu_56);
    bldr_56 = ctx_56->new_ir_builder(ctx_56);
    id_776 = bldr_56->gen_sym(bldr_56, "@i64");
    bldr_56->new_type_int(bldr_56, id_776, 0x00000040ull);
    id_777 = bldr_56->gen_sym(bldr_56, "@0_i64");
    bldr_56->new_const_int(bldr_56, id_777, id_776, 0x0000000000000000ull);
    id_778 = bldr_56->gen_sym(bldr_56, "@1_i64");
    bldr_56->new_const_int(bldr_56, id_778, id_776, 0x0000000000000001ull);
    id_779 = bldr_56->gen_sym(bldr_56, "@sig_i64_i64");
    bldr_56->new_funcsig(bldr_56, id_779, (MuTypeNode [1]){id_776}, 1, (MuTypeNode [1]){id_776}, 1);
    id_780 = bldr_56->gen_sym(bldr_56, "@milsum");
    bldr_56->new_func(bldr_56, id_780, id_779);
    id_781 = bldr_56->gen_sym(bldr_56, "@milsum.v1");
    id_782 = bldr_56->gen_sym(bldr_56, "@milsum.v1.blk0");
    id_783 = bldr_56->gen_sym(bldr_56, "@milsum.v1.blk1");
    id_784 = bldr_56->gen_sym(bldr_56, "@milsum.v1.blk2");
    id_785 = bldr_56->gen_sym(bldr_56, "@milsum.v1.blk3");
    id_786 = bldr_56->gen_sym(bldr_56, "@milsum.v1.blk0.k");
    id_787 = bldr_56->gen_sym(bldr_56, NULL);
    bldr_56->new_dest_clause(bldr_56, id_787, id_783, (MuVarNode [3]){id_777, id_777, id_786}, 3);
    id_788 = bldr_56->gen_sym(bldr_56, NULL);
    bldr_56->new_branch(bldr_56, id_788, id_787);
    bldr_56->new_bb(bldr_56, id_782, (MuID [1]){id_786}, (MuTypeNode [1]){id_776}, 1, MU_NO_ID, (MuInstNode [1]){id_788}, 1);
    id_789 = bldr_56->gen_sym(bldr_56, "@milsum.v1.blk1.acc");
    id_790 = bldr_56->gen_sym(bldr_56, "@milsum.v1.blk1.i");
    id_791 = bldr_56->gen_sym(bldr_56, "@milsum.v1.blk1.end");
    id_792 = bldr_56->gen_sym(bldr_56, "@milsum.v1.blk1.cmpres");
    id_793 = bldr_56->gen_sym(bldr_56, NULL);
    bldr_56->new_cmp(bldr_56, id_793, id_792, MU_CMP_EQ, id_776, id_790, id_791);
    id_794 = bldr_56->gen_sym(bldr_56, NULL);
    id_795 = bldr_56->gen_sym(bldr_56, NULL);
    bldr_56->new_dest_clause(bldr_56, id_795, id_785, (MuVarNode [1]){id_789}, 1);
    id_796 = bldr_56->gen_sym(bldr_56, NULL);
    bldr_56->new_dest_clause(bldr_56, id_796, id_784, (MuVarNode [3]){id_789, id_790, id_791}, 3);
    bldr_56->new_branch2(bldr_56, id_794, id_792, id_795, id_796);
    bldr_56->new_bb(bldr_56, id_783, (MuID [3]){id_789, id_790, id_791}, (MuTypeNode [3]){id_776, id_776, id_776}, 3, MU_NO_ID, (MuInstNode [2]){id_793, id_794}, 2);
    id_797 = bldr_56->gen_sym(bldr_56, "@milsum.v1.blk2.acc");
    id_798 = bldr_56->gen_sym(bldr_56, "@milsum.v1.blk2.i");
    id_799 = bldr_56->gen_sym(bldr_56, "@milsum.v1.blk2.end");
    id_800 = bldr_56->gen_sym(bldr_56, "@milsum.v1.blk2.acc_res");
    id_801 = bldr_56->gen_sym(bldr_56, "@milsum.v1.blk2.i_res");
    id_802 = bldr_56->gen_sym(bldr_56, NULL);
    bldr_56->new_binop(bldr_56, id_802, id_801, MU_BINOP_ADD, id_776, id_798, id_778, MU_NO_ID);
    id_803 = bldr_56->gen_sym(bldr_56, NULL);
    bldr_56->new_binop(bldr_56, id_803, id_800, MU_BINOP_ADD, id_776, id_797, id_801, MU_NO_ID);
    id_804 = bldr_56->gen_sym(bldr_56, NULL);
    bldr_56->new_dest_clause(bldr_56, id_804, id_783, (MuVarNode [3]){id_800, id_801, id_799}, 3);
    id_805 = bldr_56->gen_sym(bldr_56, NULL);
    bldr_56->new_branch(bldr_56, id_805, id_804);
    bldr_56->new_bb(bldr_56, id_784, (MuID [3]){id_797, id_798, id_799}, (MuTypeNode [3]){id_776, id_776, id_776}, 3, MU_NO_ID, (MuInstNode [3]){id_802, id_803, id_805}, 3);
    id_806 = bldr_56->gen_sym(bldr_56, "@milsum.v1.blk3.rtn");
    id_807 = bldr_56->gen_sym(bldr_56, NULL);
    bldr_56->new_ret(bldr_56, id_807, (MuVarNode [1]){id_806}, 1);
    bldr_56->new_bb(bldr_56, id_785, (MuID [1]){id_806}, (MuTypeNode [1]){id_776}, 1, MU_NO_ID, (MuInstNode [1]){id_807}, 1);
    bldr_56->new_func_ver(bldr_56, id_781, id_780, (MuBBNode [4]){id_782, id_783, id_784, id_785}, 4);
    bldr_56->load(bldr_56);
    mu_56->compile_to_sharedlib(mu_56, LIB_FILE_NAME("test_milsum"), NULL, 0);
    return 0;
}
