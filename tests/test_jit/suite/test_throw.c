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
    MuVM* mu_28;
    MuCtx* ctx_28;
    MuIRBuilder* bldr_28;
    MuID id_359;
    MuID id_360;
    MuID id_361;
    MuID id_362;
    MuID id_363;
    MuID id_364;
    MuID id_365;
    MuID id_366;
    MuID id_367;
    MuID id_368;
    MuID id_369;
    MuID id_370;
    MuID id_371;
    MuID id_372;
    MuID id_373;
    MuID id_374;
    MuID id_375;
    MuID id_376;
    MuID id_377;
    MuID id_378;
    MuID id_379;
    MuID id_380;
    MuID id_381;
    MuID id_382;
    MuID id_383;
    MuID id_384;
    MuID id_385;
    MuID id_386;
    MuID id_387;
    MuID id_388;
    MuID id_389;
    MuID id_390;
    MuID id_391;
    MuID id_392;
    MuID id_393;
    MuID id_394;
    MuID id_395;
    MuID id_396;
    MuID id_397;
    MuID id_398;
    MuID id_399;
    MuID id_400;
    MuID id_401;
    MuID id_402;
    MuID id_403;
    MuID id_404;
    MuID id_405;
    MuID id_406;
    mu_28 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_28 = mu_28->new_context(mu_28);
    bldr_28 = ctx_28->new_ir_builder(ctx_28);
    id_359 = bldr_28->gen_sym(bldr_28, "@void");
    bldr_28->new_type_void(bldr_28, id_359);
    id_360 = bldr_28->gen_sym(bldr_28, "@i64");
    bldr_28->new_type_int(bldr_28, id_360, 0x00000040ull);
    id_361 = bldr_28->gen_sym(bldr_28, "@refi64");
    bldr_28->new_type_ref(bldr_28, id_361, id_360);
    id_362 = bldr_28->gen_sym(bldr_28, "@refvoid");
    bldr_28->new_type_ref(bldr_28, id_362, id_359);
    id_363 = bldr_28->gen_sym(bldr_28, "@c_10");
    bldr_28->new_const_int(bldr_28, id_363, id_360, 0x000000000000000aull);
    id_364 = bldr_28->gen_sym(bldr_28, "@c_20");
    bldr_28->new_const_int(bldr_28, id_364, id_360, 0x0000000000000014ull);
    id_365 = bldr_28->gen_sym(bldr_28, "@c_42");
    bldr_28->new_const_int(bldr_28, id_365, id_360, 0x000000000000002aull);
    id_366 = bldr_28->gen_sym(bldr_28, "@sig_i64_i64");
    bldr_28->new_funcsig(bldr_28, id_366, (MuTypeNode [1]){id_360}, 1, (MuTypeNode [1]){id_360}, 1);
    id_367 = bldr_28->gen_sym(bldr_28, "@test_fnc");
    bldr_28->new_func(bldr_28, id_367, id_366);
    id_368 = bldr_28->gen_sym(bldr_28, "@throw_fnc");
    bldr_28->new_func(bldr_28, id_368, id_366);
    id_369 = bldr_28->gen_sym(bldr_28, "@throw_fnc.v1");
    id_370 = bldr_28->gen_sym(bldr_28, "@throw_fnc.v1.blk0");
    id_371 = bldr_28->gen_sym(bldr_28, "@throw_fnc.v1.blk1");
    id_372 = bldr_28->gen_sym(bldr_28, "@throw_fnc.v1.blk2");
    id_373 = bldr_28->gen_sym(bldr_28, "@throw_fnc.v1.blk0.num");
    id_374 = bldr_28->gen_sym(bldr_28, "@throw_fnc.v1.blk0.cmpres");
    id_375 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_cmp(bldr_28, id_375, id_374, MU_CMP_SLT, id_360, id_373, id_365);
    id_376 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_dest_clause(bldr_28, id_376, id_371, NULL, 0);
    id_377 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_dest_clause(bldr_28, id_377, id_372, NULL, 0);
    id_378 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_branch2(bldr_28, id_378, id_374, id_376, id_377);
    bldr_28->new_bb(bldr_28, id_370, (MuID [1]){id_373}, (MuTypeNode [1]){id_360}, 1, MU_NO_ID, (MuInstNode [2]){id_375, id_378}, 2);
    id_379 = bldr_28->gen_sym(bldr_28, "@throw_fnc.v1.blk1.excobj");
    id_380 = bldr_28->gen_sym(bldr_28, "@throw_fnc.v1.blk1.iref_obj");
    id_381 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_new(bldr_28, id_381, id_379, id_360, MU_NO_ID);
    id_382 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_getiref(bldr_28, id_382, id_380, id_360, id_379);
    id_383 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_store(bldr_28, id_383, false, MU_ORD_NOT_ATOMIC, id_360, id_380, id_364, MU_NO_ID);
    id_384 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_throw(bldr_28, id_384, id_379);
    bldr_28->new_bb(bldr_28, id_371, NULL, NULL, 0, MU_NO_ID, (MuInstNode [4]){id_381, id_382, id_383, id_384}, 4);
    id_385 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_ret(bldr_28, id_385, (MuVarNode [1]){id_363}, 1);
    bldr_28->new_bb(bldr_28, id_372, NULL, NULL, 0, MU_NO_ID, (MuInstNode [1]){id_385}, 1);
    bldr_28->new_func_ver(bldr_28, id_369, id_368, (MuBBNode [3]){id_370, id_371, id_372}, 3);
    id_386 = bldr_28->gen_sym(bldr_28, "@test_fnc.v1");
    id_387 = bldr_28->gen_sym(bldr_28, "@test_fnc.v1.blk0");
    id_388 = bldr_28->gen_sym(bldr_28, "@test_fnc.v1.blk1");
    id_389 = bldr_28->gen_sym(bldr_28, "@test_fnc.v1.blk2");
    id_390 = bldr_28->gen_sym(bldr_28, "@test_fnc.v1.blk0.num");
    id_391 = bldr_28->gen_sym(bldr_28, "@test_fnc.v1.blk0.res");
    id_392 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_dest_clause(bldr_28, id_392, id_388, (MuVarNode [1]){id_391}, 1);
    id_393 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_dest_clause(bldr_28, id_393, id_389, NULL, 0);
    id_394 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_exc_clause(bldr_28, id_394, id_392, id_393);
    id_395 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_call(bldr_28, id_395, (MuID [1]){id_391}, 1, id_366, id_368, (MuVarNode [1]){id_390}, 1, id_394, MU_NO_ID);
    bldr_28->new_bb(bldr_28, id_387, (MuID [1]){id_390}, (MuTypeNode [1]){id_360}, 1, MU_NO_ID, (MuInstNode [1]){id_395}, 1);
    id_396 = bldr_28->gen_sym(bldr_28, "@test_fnc.v1.blk1.rtn");
    id_397 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_ret(bldr_28, id_397, (MuVarNode [1]){id_396}, 1);
    bldr_28->new_bb(bldr_28, id_388, (MuID [1]){id_396}, (MuTypeNode [1]){id_360}, 1, MU_NO_ID, (MuInstNode [1]){id_397}, 1);
    id_398 = bldr_28->gen_sym(bldr_28, "@test_fnc.v1.blk2.excobj");
    id_399 = bldr_28->gen_sym(bldr_28, "@test_fnc.v1.blk2.ri64");
    id_400 = bldr_28->gen_sym(bldr_28, "@test_fnc.v1.blk2.iri64");
    id_401 = bldr_28->gen_sym(bldr_28, "@test_fnc.v1.blk2.obj");
    id_402 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_conv(bldr_28, id_402, id_399, MU_CONV_REFCAST, id_362, id_361, id_398);
    id_403 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_getiref(bldr_28, id_403, id_400, id_360, id_399);
    id_404 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_load(bldr_28, id_404, id_401, false, MU_ORD_NOT_ATOMIC, id_360, id_400, MU_NO_ID);
    id_405 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_dest_clause(bldr_28, id_405, id_388, (MuVarNode [1]){id_401}, 1);
    id_406 = bldr_28->gen_sym(bldr_28, NULL);
    bldr_28->new_branch(bldr_28, id_406, id_405);
    bldr_28->new_bb(bldr_28, id_389, NULL, NULL, 0, id_398, (MuInstNode [4]){id_402, id_403, id_404, id_406}, 4);
    bldr_28->new_func_ver(bldr_28, id_386, id_367, (MuBBNode [3]){id_387, id_388, id_389}, 3);
    bldr_28->load(bldr_28);
    mu_28->compile_to_sharedlib(mu_28, LIB_FILE_NAME("test_throw"), NULL, 0);
    return 0;
}
