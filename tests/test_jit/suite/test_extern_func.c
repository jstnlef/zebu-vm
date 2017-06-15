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
    MuVM* mu_27;
    MuCtx* ctx_27;
    MuIRBuilder* bldr_27;
    MuID id_342;
    MuID id_343;
    MuID id_344;
    MuID id_345;
    MuID id_346;
    MuID id_347;
    MuID id_348;
    MuID id_349;
    MuID id_350;
    MuID id_351;
    MuID id_352;
    MuID id_353;
    MuID id_354;
    MuID id_355;
    MuID id_356;
    MuID id_357;
    MuID id_358;
    mu_27 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_27 = mu_27->new_context(mu_27);
    bldr_27 = ctx_27->new_ir_builder(ctx_27);
    id_342 = bldr_27->gen_sym(bldr_27, "@i32");
    bldr_27->new_type_int(bldr_27, id_342, 0x00000020ull);
    id_343 = bldr_27->gen_sym(bldr_27, "@i64");
    bldr_27->new_type_int(bldr_27, id_343, 0x00000040ull);
    id_344 = bldr_27->gen_sym(bldr_27, "@void");
    bldr_27->new_type_void(bldr_27, id_344);
    id_345 = bldr_27->gen_sym(bldr_27, "@voidp");
    bldr_27->new_type_uptr(bldr_27, id_345, id_344);
    id_346 = bldr_27->gen_sym(bldr_27, "@fd_stdout");
    bldr_27->new_const_int(bldr_27, id_346, id_342, 0x0000000000000001ull);
    id_347 = bldr_27->gen_sym(bldr_27, "@sig_voidpi64_i64");
    bldr_27->new_funcsig(bldr_27, id_347, (MuTypeNode [2]){id_345, id_343}, 2, (MuTypeNode [1]){id_343}, 1);
    id_348 = bldr_27->gen_sym(bldr_27, "@sig_i32voidpi64_i64");
    bldr_27->new_funcsig(bldr_27, id_348, (MuTypeNode [3]){id_342, id_345, id_343}, 3, (MuTypeNode [1]){id_343}, 1);
    id_349 = bldr_27->gen_sym(bldr_27, "@fnpsig_i32voidpi64_i64");
    bldr_27->new_type_ufuncptr(bldr_27, id_349, id_348);
    id_350 = bldr_27->gen_sym(bldr_27, "@c_write");
    bldr_27->new_const_extern(bldr_27, id_350, id_349, "write");
    id_351 = bldr_27->gen_sym(bldr_27, "@test_write");
    bldr_27->new_func(bldr_27, id_351, id_347);
    id_352 = bldr_27->gen_sym(bldr_27, "@test_write_v1");
    id_353 = bldr_27->gen_sym(bldr_27, "@test_write_v1.blk0");
    id_354 = bldr_27->gen_sym(bldr_27, "@test_write_v1.blk0.buf");
    id_355 = bldr_27->gen_sym(bldr_27, "@test_write_v1.blk0.sz");
    id_356 = bldr_27->gen_sym(bldr_27, "@test_write_v1.blk0.res");
    id_357 = bldr_27->gen_sym(bldr_27, NULL);
    bldr_27->new_ccall(bldr_27, id_357, (MuID [1]){id_356}, 1, MU_CC_DEFAULT, id_349, id_348, id_350, (MuVarNode [3]){id_346, id_354, id_355}, 3, MU_NO_ID, MU_NO_ID);
    id_358 = bldr_27->gen_sym(bldr_27, NULL);
    bldr_27->new_ret(bldr_27, id_358, (MuVarNode [1]){id_356}, 1);
    bldr_27->new_bb(bldr_27, id_353, (MuID [2]){id_354, id_355}, (MuTypeNode [2]){id_345, id_343}, 2, MU_NO_ID, (MuInstNode [2]){id_357, id_358}, 2);
    bldr_27->new_func_ver(bldr_27, id_352, id_351, (MuBBNode [1]){id_353}, 1);
    bldr_27->load(bldr_27);
    mu_27->compile_to_sharedlib(mu_27, LIB_FILE_NAME("test_extern_func"), NULL, 0);
    return 0;
}
