
// Compile with flag -std=c99
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
int main(int argc, char** argv) {
    MuVM* mu_22;
    MuCtx* ctx_22;
    MuIRBuilder* bldr_22;
    MuID id_285;
    MuID id_286;
    MuID id_287;
    MuID id_288;
    MuID id_289;
    MuID id_290;
    MuID id_291;
    MuID id_292;
    MuID id_293;
    MuID id_294;
    MuID id_295;
    MuID id_296;
    MuID id_297;
    MuID id_298;
    MuID id_299;
    MuID id_300;
    MuID id_301;
    mu_22 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_22 = mu_22->new_context(mu_22);
    bldr_22 = ctx_22->new_ir_builder(ctx_22);
    id_285 = bldr_22->gen_sym(bldr_22, "@i32");
    bldr_22->new_type_int(bldr_22, id_285, 32);
    id_286 = bldr_22->gen_sym(bldr_22, "@i64");
    bldr_22->new_type_int(bldr_22, id_286, 64);
    id_287 = bldr_22->gen_sym(bldr_22, "@void");
    bldr_22->new_type_void(bldr_22, id_287);
    id_288 = bldr_22->gen_sym(bldr_22, "@voidp");
    bldr_22->new_type_uptr(bldr_22, id_288, id_287);
    id_289 = bldr_22->gen_sym(bldr_22, "@fd_stdout");
    bldr_22->new_const_int(bldr_22, id_289, id_285, 1);
    id_290 = bldr_22->gen_sym(bldr_22, "@sig_voidpi64_i64");
    bldr_22->new_funcsig(bldr_22, id_290, (MuTypeNode [2]){id_288, id_286}, 2, (MuTypeNode [1]){id_286}, 1);
    id_291 = bldr_22->gen_sym(bldr_22, "@sig_i32voidpi64_i64");
    bldr_22->new_funcsig(bldr_22, id_291, (MuTypeNode [3]){id_285, id_288, id_286}, 3, (MuTypeNode [1]){id_286}, 1);
    id_292 = bldr_22->gen_sym(bldr_22, "@fnpsig_i32voidpi64_i64");
    bldr_22->new_type_ufuncptr(bldr_22, id_292, id_291);
    id_293 = bldr_22->gen_sym(bldr_22, "@c_write");
    bldr_22->new_const_extern(bldr_22, id_293, id_292, "write");
    id_294 = bldr_22->gen_sym(bldr_22, "@test_write");
    bldr_22->new_func(bldr_22, id_294, id_290);
    id_295 = bldr_22->gen_sym(bldr_22, "@test_write_v1");
    id_296 = bldr_22->gen_sym(bldr_22, "@test_write_v1.blk0");
    id_297 = bldr_22->gen_sym(bldr_22, "@test_write_v1.blk0.buf");
    id_298 = bldr_22->gen_sym(bldr_22, "@test_write_v1.blk0.sz");
    id_299 = bldr_22->gen_sym(bldr_22, "@test_write_v1.blk0.res");
    id_300 = bldr_22->gen_sym(bldr_22, NULL);
    bldr_22->new_ccall(bldr_22, id_300, (MuID [1]){id_299}, 1, MU_CC_DEFAULT, id_292, id_291, id_293, (MuVarNode [3]){id_289, id_297, id_298}, 3, MU_NO_ID, MU_NO_ID);
    id_301 = bldr_22->gen_sym(bldr_22, NULL);
    bldr_22->new_ret(bldr_22, id_301, (MuVarNode [1]){id_299}, 1);
    bldr_22->new_bb(bldr_22, id_296, (MuID [2]){id_297, id_298}, (MuTypeNode [2]){id_288, id_286}, 2, MU_NO_ID, (MuInstNode [2]){id_300, id_301}, 2);
    bldr_22->new_func_ver(bldr_22, id_295, id_294, (MuBBNode [1]){id_296}, 1);
    bldr_22->load(bldr_22);
    mu_22->compile_to_sharedlib(mu_22, "test_extern_func.dylib", NULL, 0);
    printf("%s\n", "test_extern_func.dylib");
    return 0;
}
