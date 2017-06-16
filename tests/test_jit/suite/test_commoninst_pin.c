// Copyright 2017 The Australian National University
// 
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// 
//     http://www.apache.org/licenses/LICENSE-2.0
// 
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


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
    MuVM* mu_59;
    MuCtx* ctx_59;
    MuIRBuilder* bldr_59;
    MuID id_872;
    MuID id_873;
    MuID id_874;
    MuID id_875;
    MuID id_876;
    MuID id_877;
    MuID id_878;
    MuID id_879;
    MuID id_880;
    MuID id_881;
    MuID id_882;
    MuID id_883;
    MuID id_884;
    MuID id_885;
    MuID id_886;
    MuID id_887;
    MuID id_888;
    MuID id_889;
    MuID id_890;
    MuID id_891;
    MuID id_892;
    MuID id_893;
    MuID id_894;
    MuID id_895;
    MuID id_896;
    MuID id_897;
    MuID id_898;
    MuID id_899;
    MuID id_900;
    MuID id_901;
    MuID id_902;
    MuID id_903;
    MuID id_904;
    MuID id_905;
    MuID id_906;
    MuID id_907;
    MuID id_908;
    MuID id_909;
    MuID id_910;
    MuID id_911;
    MuID id_912;
    MuID id_913;
    MuID id_914;
    MuID id_915;
    MuID id_916;
    MuID id_917;
    MuID id_918;
    MuID id_919;
    MuID id_920;
    MuID id_921;
    MuID id_922;
    MuID id_923;
    MuID id_924;
    MuID id_925;
    mu_59 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_59 = mu_59->new_context(mu_59);
    bldr_59 = ctx_59->new_ir_builder(ctx_59);
    id_872 = bldr_59->gen_sym(bldr_59, "@i8");
    bldr_59->new_type_int(bldr_59, id_872, 0x00000008ull);
    id_873 = bldr_59->gen_sym(bldr_59, "@i32");
    bldr_59->new_type_int(bldr_59, id_873, 0x00000020ull);
    id_874 = bldr_59->gen_sym(bldr_59, "@i64");
    bldr_59->new_type_int(bldr_59, id_874, 0x00000040ull);
    id_875 = bldr_59->gen_sym(bldr_59, "@void");
    bldr_59->new_type_void(bldr_59, id_875);
    id_876 = bldr_59->gen_sym(bldr_59, "@voidp");
    bldr_59->new_type_uptr(bldr_59, id_876, id_875);
    id_877 = bldr_59->gen_sym(bldr_59, "@hyb");
    bldr_59->new_type_hybrid(bldr_59, id_877, NULL, 0, id_872);
    id_878 = bldr_59->gen_sym(bldr_59, "@rhyb");
    bldr_59->new_type_ref(bldr_59, id_878, id_877);
    id_879 = bldr_59->gen_sym(bldr_59, "@phyb");
    bldr_59->new_type_uptr(bldr_59, id_879, id_877);
    id_880 = bldr_59->gen_sym(bldr_59, "@fd_stdout");
    bldr_59->new_const_int(bldr_59, id_880, id_873, 0x0000000000000001ull);
    id_881 = bldr_59->gen_sym(bldr_59, "@c_h");
    bldr_59->new_const_int(bldr_59, id_881, id_872, 0x0000000000000068ull);
    id_882 = bldr_59->gen_sym(bldr_59, "@c_e");
    bldr_59->new_const_int(bldr_59, id_882, id_872, 0x0000000000000065ull);
    id_883 = bldr_59->gen_sym(bldr_59, "@c_l");
    bldr_59->new_const_int(bldr_59, id_883, id_872, 0x000000000000006cull);
    id_884 = bldr_59->gen_sym(bldr_59, "@c_o");
    bldr_59->new_const_int(bldr_59, id_884, id_872, 0x000000000000006full);
    id_885 = bldr_59->gen_sym(bldr_59, "@c_0");
    bldr_59->new_const_int(bldr_59, id_885, id_874, 0x0000000000000000ull);
    id_886 = bldr_59->gen_sym(bldr_59, "@c_1");
    bldr_59->new_const_int(bldr_59, id_886, id_874, 0x0000000000000001ull);
    id_887 = bldr_59->gen_sym(bldr_59, "@c_len");
    bldr_59->new_const_int(bldr_59, id_887, id_874, 0x0000000000000005ull);
    id_888 = bldr_59->gen_sym(bldr_59, "@c_bufsz");
    bldr_59->new_const_int(bldr_59, id_888, id_874, 0x0000000000000006ull);
    id_889 = bldr_59->gen_sym(bldr_59, "@sig__i64");
    bldr_59->new_funcsig(bldr_59, id_889, (MuTypeNode [2]){id_876, id_874}, 2, (MuTypeNode [1]){id_874}, 1);
    id_890 = bldr_59->gen_sym(bldr_59, "@sig_i32voidpi64_i64");
    bldr_59->new_funcsig(bldr_59, id_890, (MuTypeNode [3]){id_873, id_876, id_874}, 3, (MuTypeNode [1]){id_874}, 1);
    id_891 = bldr_59->gen_sym(bldr_59, "@fnpsig_i32voidpi64_i64");
    bldr_59->new_type_ufuncptr(bldr_59, id_891, id_890);
    id_892 = bldr_59->gen_sym(bldr_59, "@c_write");
    bldr_59->new_const_extern(bldr_59, id_892, id_891, "write");
    id_893 = bldr_59->gen_sym(bldr_59, "@test_pin");
    bldr_59->new_func(bldr_59, id_893, id_889);
    id_894 = bldr_59->gen_sym(bldr_59, "@test_write_v1");
    id_895 = bldr_59->gen_sym(bldr_59, "@test_write_v1.blk0");
    id_896 = bldr_59->gen_sym(bldr_59, "@test_pin.v1.blk0.rs");
    id_897 = bldr_59->gen_sym(bldr_59, "@test_pin.v1.blk0.irs");
    id_898 = bldr_59->gen_sym(bldr_59, "@test_pin.v1.blk0.irelm_0");
    id_899 = bldr_59->gen_sym(bldr_59, "@test_pin.v1.blk0.irelm_1");
    id_900 = bldr_59->gen_sym(bldr_59, "@test_pin.v1.blk0.irelm_2");
    id_901 = bldr_59->gen_sym(bldr_59, "@test_pin.v1.blk0.irelm_3");
    id_902 = bldr_59->gen_sym(bldr_59, "@test_pin.v1.blk0.irelm_4");
    id_903 = bldr_59->gen_sym(bldr_59, "@test_pin.v1.blk0.irelm_5");
    id_904 = bldr_59->gen_sym(bldr_59, "@test_pin.v1.blk0.ps");
    id_905 = bldr_59->gen_sym(bldr_59, "@test_pin.v1.blk0.buf");
    id_906 = bldr_59->gen_sym(bldr_59, "@test_pin.v1.blk0.res");
    id_907 = bldr_59->gen_sym(bldr_59, NULL);
    bldr_59->new_newhybrid(bldr_59, id_907, id_896, id_877, id_874, id_888, MU_NO_ID);
    id_908 = bldr_59->gen_sym(bldr_59, NULL);
    bldr_59->new_getiref(bldr_59, id_908, id_897, id_877, id_896);
    id_909 = bldr_59->gen_sym(bldr_59, NULL);
    bldr_59->new_getvarpartiref(bldr_59, id_909, id_898, false, id_877, id_897);
    id_910 = bldr_59->gen_sym(bldr_59, NULL);
    bldr_59->new_store(bldr_59, id_910, false, MU_ORD_NOT_ATOMIC, id_872, id_898, id_881, MU_NO_ID);
    id_911 = bldr_59->gen_sym(bldr_59, NULL);
    bldr_59->new_shiftiref(bldr_59, id_911, id_899, false, id_872, id_874, id_898, id_886);
    id_912 = bldr_59->gen_sym(bldr_59, NULL);
    bldr_59->new_store(bldr_59, id_912, false, MU_ORD_NOT_ATOMIC, id_872, id_899, id_882, MU_NO_ID);
    id_913 = bldr_59->gen_sym(bldr_59, NULL);
    bldr_59->new_shiftiref(bldr_59, id_913, id_900, false, id_872, id_874, id_899, id_886);
    id_914 = bldr_59->gen_sym(bldr_59, NULL);
    bldr_59->new_store(bldr_59, id_914, false, MU_ORD_NOT_ATOMIC, id_872, id_900, id_883, MU_NO_ID);
    id_915 = bldr_59->gen_sym(bldr_59, NULL);
    bldr_59->new_shiftiref(bldr_59, id_915, id_901, false, id_872, id_874, id_900, id_886);
    id_916 = bldr_59->gen_sym(bldr_59, NULL);
    bldr_59->new_store(bldr_59, id_916, false, MU_ORD_NOT_ATOMIC, id_872, id_901, id_883, MU_NO_ID);
    id_917 = bldr_59->gen_sym(bldr_59, NULL);
    bldr_59->new_shiftiref(bldr_59, id_917, id_902, false, id_872, id_874, id_901, id_886);
    id_918 = bldr_59->gen_sym(bldr_59, NULL);
    bldr_59->new_store(bldr_59, id_918, false, MU_ORD_NOT_ATOMIC, id_872, id_902, id_884, MU_NO_ID);
    id_919 = bldr_59->gen_sym(bldr_59, NULL);
    bldr_59->new_shiftiref(bldr_59, id_919, id_903, false, id_872, id_874, id_902, id_886);
    id_920 = bldr_59->gen_sym(bldr_59, NULL);
    bldr_59->new_store(bldr_59, id_920, false, MU_ORD_NOT_ATOMIC, id_872, id_903, id_885, MU_NO_ID);
    id_921 = bldr_59->gen_sym(bldr_59, NULL);
    bldr_59->new_comminst(bldr_59, id_921, (MuID [1]){id_904}, 1, MU_CI_UVM_NATIVE_PIN, NULL, 0, (MuTypeNode [1]){id_878}, 1, NULL, 0, (MuVarNode [1]){id_896}, 1, MU_NO_ID, MU_NO_ID);
    id_922 = bldr_59->gen_sym(bldr_59, NULL);
    bldr_59->new_conv(bldr_59, id_922, id_905, MU_CONV_PTRCAST, id_879, id_876, id_904);
    id_923 = bldr_59->gen_sym(bldr_59, NULL);
    bldr_59->new_ccall(bldr_59, id_923, (MuID [1]){id_906}, 1, MU_CC_DEFAULT, id_891, id_890, id_892, (MuVarNode [3]){id_880, id_905, id_888}, 3, MU_NO_ID, MU_NO_ID);
    id_924 = bldr_59->gen_sym(bldr_59, NULL);
    bldr_59->new_comminst(bldr_59, id_924, NULL, 0, MU_CI_UVM_NATIVE_UNPIN, NULL, 0, (MuTypeNode [1]){id_878}, 1, NULL, 0, (MuVarNode [1]){id_896}, 1, MU_NO_ID, MU_NO_ID);
    id_925 = bldr_59->gen_sym(bldr_59, NULL);
    bldr_59->new_ret(bldr_59, id_925, (MuVarNode [1]){id_906}, 1);
    bldr_59->new_bb(bldr_59, id_895, NULL, NULL, 0, MU_NO_ID, (MuInstNode [19]){id_907, id_908, id_909, id_910, id_911, id_912, id_913, id_914, id_915, id_916, id_917, id_918, id_919, id_920, id_921, id_922, id_923, id_924, id_925}, 19);
    bldr_59->new_func_ver(bldr_59, id_894, id_893, (MuBBNode [1]){id_895}, 1);
    bldr_59->load(bldr_59);
    mu_59->compile_to_sharedlib(mu_59, LIB_FILE_NAME("test_commoninst_pin"), NULL, 0);
    return 0;
}
