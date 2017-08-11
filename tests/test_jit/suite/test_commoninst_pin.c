
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
    MuVM* mu_60;
    MuCtx* ctx_60;
    MuIRBuilder* bldr_60;
    MuID id_927;
    MuID id_928;
    MuID id_929;
    MuID id_930;
    MuID id_931;
    MuID id_932;
    MuID id_933;
    MuID id_934;
    MuID id_935;
    MuID id_936;
    MuID id_937;
    MuID id_938;
    MuID id_939;
    MuID id_940;
    MuID id_941;
    MuID id_942;
    MuID id_943;
    MuID id_944;
    MuID id_945;
    MuID id_946;
    MuID id_947;
    MuID id_948;
    MuID id_949;
    MuID id_950;
    MuID id_951;
    MuID id_952;
    MuID id_953;
    MuID id_954;
    MuID id_955;
    MuID id_956;
    MuID id_957;
    MuID id_958;
    MuID id_959;
    MuID id_960;
    MuID id_961;
    MuID id_962;
    MuID id_963;
    MuID id_964;
    MuID id_965;
    MuID id_966;
    MuID id_967;
    MuID id_968;
    MuID id_969;
    MuID id_970;
    MuID id_971;
    MuID id_972;
    MuID id_973;
    MuID id_974;
    MuID id_975;
    MuID id_976;
    MuID id_977;
    MuID id_978;
    MuID id_979;
    MuID id_980;
    mu_60 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_60 = mu_60->new_context(mu_60);
    bldr_60 = ctx_60->new_ir_builder(ctx_60);
    id_927 = bldr_60->gen_sym(bldr_60, "@i8");
    bldr_60->new_type_int(bldr_60, id_927, 0x00000008ull);
    id_928 = bldr_60->gen_sym(bldr_60, "@i32");
    bldr_60->new_type_int(bldr_60, id_928, 0x00000020ull);
    id_929 = bldr_60->gen_sym(bldr_60, "@i64");
    bldr_60->new_type_int(bldr_60, id_929, 0x00000040ull);
    id_930 = bldr_60->gen_sym(bldr_60, "@void");
    bldr_60->new_type_void(bldr_60, id_930);
    id_931 = bldr_60->gen_sym(bldr_60, "@voidp");
    bldr_60->new_type_uptr(bldr_60, id_931, id_930);
    id_932 = bldr_60->gen_sym(bldr_60, "@hyb");
    bldr_60->new_type_hybrid(bldr_60, id_932, NULL, 0, id_927);
    id_933 = bldr_60->gen_sym(bldr_60, "@rhyb");
    bldr_60->new_type_ref(bldr_60, id_933, id_932);
    id_934 = bldr_60->gen_sym(bldr_60, "@phyb");
    bldr_60->new_type_uptr(bldr_60, id_934, id_932);
    id_935 = bldr_60->gen_sym(bldr_60, "@fd_stdout");
    bldr_60->new_const_int(bldr_60, id_935, id_928, 0x0000000000000001ull);
    id_936 = bldr_60->gen_sym(bldr_60, "@c_h");
    bldr_60->new_const_int(bldr_60, id_936, id_927, 0x0000000000000068ull);
    id_937 = bldr_60->gen_sym(bldr_60, "@c_e");
    bldr_60->new_const_int(bldr_60, id_937, id_927, 0x0000000000000065ull);
    id_938 = bldr_60->gen_sym(bldr_60, "@c_l");
    bldr_60->new_const_int(bldr_60, id_938, id_927, 0x000000000000006cull);
    id_939 = bldr_60->gen_sym(bldr_60, "@c_o");
    bldr_60->new_const_int(bldr_60, id_939, id_927, 0x000000000000006full);
    id_940 = bldr_60->gen_sym(bldr_60, "@c_0");
    bldr_60->new_const_int(bldr_60, id_940, id_927, 0x0000000000000000ull);
    id_941 = bldr_60->gen_sym(bldr_60, "@c_1");
    bldr_60->new_const_int(bldr_60, id_941, id_929, 0x0000000000000001ull);
    id_942 = bldr_60->gen_sym(bldr_60, "@c_len");
    bldr_60->new_const_int(bldr_60, id_942, id_929, 0x0000000000000005ull);
    id_943 = bldr_60->gen_sym(bldr_60, "@c_bufsz");
    bldr_60->new_const_int(bldr_60, id_943, id_929, 0x0000000000000006ull);
    id_944 = bldr_60->gen_sym(bldr_60, "@sig__i64");
    bldr_60->new_funcsig(bldr_60, id_944, (MuTypeNode [2]){id_931, id_929}, 2, (MuTypeNode [1]){id_929}, 1);
    id_945 = bldr_60->gen_sym(bldr_60, "@sig_i32voidpi64_i64");
    bldr_60->new_funcsig(bldr_60, id_945, (MuTypeNode [3]){id_928, id_931, id_929}, 3, (MuTypeNode [1]){id_929}, 1);
    id_946 = bldr_60->gen_sym(bldr_60, "@fnpsig_i32voidpi64_i64");
    bldr_60->new_type_ufuncptr(bldr_60, id_946, id_945);
    id_947 = bldr_60->gen_sym(bldr_60, "@c_write");
    bldr_60->new_const_extern(bldr_60, id_947, id_946, "write");
    id_948 = bldr_60->gen_sym(bldr_60, "@test_pin");
    bldr_60->new_func(bldr_60, id_948, id_944);
    id_949 = bldr_60->gen_sym(bldr_60, "@test_write_v1");
    id_950 = bldr_60->gen_sym(bldr_60, "@test_write_v1.blk0");
    id_951 = bldr_60->gen_sym(bldr_60, "@test_pin.v1.blk0.rs");
    id_952 = bldr_60->gen_sym(bldr_60, "@test_pin.v1.blk0.irs");
    id_953 = bldr_60->gen_sym(bldr_60, "@test_pin.v1.blk0.irelm_0");
    id_954 = bldr_60->gen_sym(bldr_60, "@test_pin.v1.blk0.irelm_1");
    id_955 = bldr_60->gen_sym(bldr_60, "@test_pin.v1.blk0.irelm_2");
    id_956 = bldr_60->gen_sym(bldr_60, "@test_pin.v1.blk0.irelm_3");
    id_957 = bldr_60->gen_sym(bldr_60, "@test_pin.v1.blk0.irelm_4");
    id_958 = bldr_60->gen_sym(bldr_60, "@test_pin.v1.blk0.irelm_5");
    id_959 = bldr_60->gen_sym(bldr_60, "@test_pin.v1.blk0.ps");
    id_960 = bldr_60->gen_sym(bldr_60, "@test_pin.v1.blk0.buf");
    id_961 = bldr_60->gen_sym(bldr_60, "@test_pin.v1.blk0.res");
    id_962 = bldr_60->gen_sym(bldr_60, NULL);
    bldr_60->new_newhybrid(bldr_60, id_962, id_951, id_932, id_929, id_943, MU_NO_ID);
    id_963 = bldr_60->gen_sym(bldr_60, NULL);
    bldr_60->new_getiref(bldr_60, id_963, id_952, id_932, id_951);
    id_964 = bldr_60->gen_sym(bldr_60, NULL);
    bldr_60->new_getvarpartiref(bldr_60, id_964, id_953, false, id_932, id_952);
    id_965 = bldr_60->gen_sym(bldr_60, NULL);
    bldr_60->new_store(bldr_60, id_965, false, MU_ORD_NOT_ATOMIC, id_927, id_953, id_936, MU_NO_ID);
    id_966 = bldr_60->gen_sym(bldr_60, NULL);
    bldr_60->new_shiftiref(bldr_60, id_966, id_954, false, id_927, id_929, id_953, id_941);
    id_967 = bldr_60->gen_sym(bldr_60, NULL);
    bldr_60->new_store(bldr_60, id_967, false, MU_ORD_NOT_ATOMIC, id_927, id_954, id_937, MU_NO_ID);
    id_968 = bldr_60->gen_sym(bldr_60, NULL);
    bldr_60->new_shiftiref(bldr_60, id_968, id_955, false, id_927, id_929, id_954, id_941);
    id_969 = bldr_60->gen_sym(bldr_60, NULL);
    bldr_60->new_store(bldr_60, id_969, false, MU_ORD_NOT_ATOMIC, id_927, id_955, id_938, MU_NO_ID);
    id_970 = bldr_60->gen_sym(bldr_60, NULL);
    bldr_60->new_shiftiref(bldr_60, id_970, id_956, false, id_927, id_929, id_955, id_941);
    id_971 = bldr_60->gen_sym(bldr_60, NULL);
    bldr_60->new_store(bldr_60, id_971, false, MU_ORD_NOT_ATOMIC, id_927, id_956, id_938, MU_NO_ID);
    id_972 = bldr_60->gen_sym(bldr_60, NULL);
    bldr_60->new_shiftiref(bldr_60, id_972, id_957, false, id_927, id_929, id_956, id_941);
    id_973 = bldr_60->gen_sym(bldr_60, NULL);
    bldr_60->new_store(bldr_60, id_973, false, MU_ORD_NOT_ATOMIC, id_927, id_957, id_939, MU_NO_ID);
    id_974 = bldr_60->gen_sym(bldr_60, NULL);
    bldr_60->new_shiftiref(bldr_60, id_974, id_958, false, id_927, id_929, id_957, id_941);
    id_975 = bldr_60->gen_sym(bldr_60, NULL);
    bldr_60->new_store(bldr_60, id_975, false, MU_ORD_NOT_ATOMIC, id_927, id_958, id_940, MU_NO_ID);
    id_976 = bldr_60->gen_sym(bldr_60, NULL);
    bldr_60->new_comminst(bldr_60, id_976, (MuID [1]){id_959}, 1, MU_CI_UVM_NATIVE_PIN, NULL, 0, (MuTypeNode [1]){id_933}, 1, NULL, 0, (MuVarNode [1]){id_951}, 1, MU_NO_ID, MU_NO_ID);
    id_977 = bldr_60->gen_sym(bldr_60, NULL);
    bldr_60->new_conv(bldr_60, id_977, id_960, MU_CONV_PTRCAST, id_934, id_931, id_959);
    id_978 = bldr_60->gen_sym(bldr_60, NULL);
    bldr_60->new_ccall(bldr_60, id_978, (MuID [1]){id_961}, 1, MU_CC_DEFAULT, id_946, id_945, id_947, (MuVarNode [3]){id_935, id_960, id_943}, 3, MU_NO_ID, MU_NO_ID);
    id_979 = bldr_60->gen_sym(bldr_60, NULL);
    bldr_60->new_comminst(bldr_60, id_979, NULL, 0, MU_CI_UVM_NATIVE_UNPIN, NULL, 0, (MuTypeNode [1]){id_933}, 1, NULL, 0, (MuVarNode [1]){id_951}, 1, MU_NO_ID, MU_NO_ID);
    id_980 = bldr_60->gen_sym(bldr_60, NULL);
    bldr_60->new_ret(bldr_60, id_980, (MuVarNode [1]){id_961}, 1);
    bldr_60->new_bb(bldr_60, id_950, NULL, NULL, 0, MU_NO_ID, (MuInstNode [19]){id_962, id_963, id_964, id_965, id_966, id_967, id_968, id_969, id_970, id_971, id_972, id_973, id_974, id_975, id_976, id_977, id_978, id_979, id_980}, 19);
    bldr_60->new_func_ver(bldr_60, id_949, id_948, (MuBBNode [1]){id_950}, 1);
    bldr_60->load(bldr_60);
    mu_60->compile_to_sharedlib(mu_60, LIB_FILE_NAME("test_commoninst_pin"), NULL, 0);
    return 0;
}
