
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
    MuVM* mu_57;
    MuCtx* ctx_57;
    MuIRBuilder* bldr_57;
    MuID id_808;
    MuID id_809;
    MuID id_810;
    MuID id_811;
    MuID id_812;
    MuID id_813;
    MuID id_814;
    MuID id_815;
    MuID id_816;
    MuID id_817;
    MuID id_818;
    MuID id_819;
    MuID id_820;
    MuID id_821;
    MuID id_822;
    MuID id_823;
    mu_57 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_57 = mu_57->new_context(mu_57);
    bldr_57 = ctx_57->new_ir_builder(ctx_57);
    id_808 = bldr_57->gen_sym(bldr_57, "@i1");
    bldr_57->new_type_int(bldr_57, id_808, 0x00000001ull);
    id_809 = bldr_57->gen_sym(bldr_57, "@i8");
    bldr_57->new_type_int(bldr_57, id_809, 0x00000008ull);
    id_810 = bldr_57->gen_sym(bldr_57, "@i64");
    bldr_57->new_type_int(bldr_57, id_810, 0x00000040ull);
    id_811 = bldr_57->gen_sym(bldr_57, "@10_i64");
    bldr_57->new_const_int(bldr_57, id_811, id_810, 0x000000000000000aull);
    id_812 = bldr_57->gen_sym(bldr_57, "@20_i64");
    bldr_57->new_const_int(bldr_57, id_812, id_810, 0x0000000000000014ull);
    id_813 = bldr_57->gen_sym(bldr_57, "@TRUE");
    bldr_57->new_const_int(bldr_57, id_813, id_809, 0x0000000000000001ull);
    id_814 = bldr_57->gen_sym(bldr_57, "@sig_i8_i64");
    bldr_57->new_funcsig(bldr_57, id_814, (MuTypeNode [1]){id_809}, 1, (MuTypeNode [1]){id_810}, 1);
    id_815 = bldr_57->gen_sym(bldr_57, "@test_fnc");
    bldr_57->new_func(bldr_57, id_815, id_814);
    id_816 = bldr_57->gen_sym(bldr_57, "@test_fnc.v1");
    id_817 = bldr_57->gen_sym(bldr_57, "@test_fnc.v1.blk0");
    id_818 = bldr_57->gen_sym(bldr_57, "@test_fnc.v1.blk0.flag");
    id_819 = bldr_57->gen_sym(bldr_57, "@test_fnc.v1.blk0.cmpres");
    id_820 = bldr_57->gen_sym(bldr_57, "@test_fnc.v1.blk0.res");
    id_821 = bldr_57->gen_sym(bldr_57, NULL);
    bldr_57->new_cmp(bldr_57, id_821, id_819, MU_CMP_EQ, id_810, id_818, id_813);
    id_822 = bldr_57->gen_sym(bldr_57, NULL);
    bldr_57->new_select(bldr_57, id_822, id_820, id_808, id_810, id_819, id_811, id_812);
    id_823 = bldr_57->gen_sym(bldr_57, NULL);
    bldr_57->new_ret(bldr_57, id_823, (MuVarNode [1]){id_820}, 1);
    bldr_57->new_bb(bldr_57, id_817, (MuID [1]){id_818}, (MuTypeNode [1]){id_809}, 1, MU_NO_ID, (MuInstNode [3]){id_821, id_822, id_823}, 3);
    bldr_57->new_func_ver(bldr_57, id_816, id_815, (MuBBNode [1]){id_817}, 1);
    bldr_57->load(bldr_57);
    mu_57->compile_to_sharedlib(mu_57, LIB_FILE_NAME("test_select"), NULL, 0);
    return 0;
}
