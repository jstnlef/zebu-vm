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
    MuVM* mu_29;
    MuCtx* ctx_29;
    MuIRBuilder* bldr_29;
    MuID id_407;
    MuID id_408;
    MuID id_409;
    MuID id_410;
    MuID id_411;
    MuID id_412;
    MuID id_413;
    MuID id_414;
    MuID id_415;
    MuID id_416;
    MuID id_417;
    MuID id_418;
    MuID id_419;
    MuID id_420;
    MuID id_421;
    MuID id_422;
    MuID id_423;
    MuID id_424;
    MuID id_425;
    MuID id_426;
    MuID id_427;
    MuID id_428;
    MuID id_429;
    MuID id_430;
    MuID id_431;
    MuID id_432;
    MuID id_433;
    MuID id_434;
    MuID id_435;
    MuID id_436;
    MuID id_437;
    MuID id_438;
    MuID id_439;
    MuID id_440;
    MuID id_441;
    MuID id_442;
    MuID id_443;
    MuID id_444;
    MuID id_445;
    MuID id_446;
    MuID id_447;
    MuID id_448;
    MuID id_449;
    MuID id_450;
    MuID id_451;
    MuID id_452;
    MuID id_453;
    MuID id_454;
    MuID id_455;
    MuID id_456;
    MuID id_457;
    MuID id_458;
    MuID id_459;
    MuID id_460;
    MuID id_461;
    mu_29 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_29 = mu_29->new_context(mu_29);
    bldr_29 = ctx_29->new_ir_builder(ctx_29);
    id_407 = bldr_29->gen_sym(bldr_29, "@void");
    bldr_29->new_type_void(bldr_29, id_407);
    id_408 = bldr_29->gen_sym(bldr_29, "@i64");
    bldr_29->new_type_int(bldr_29, id_408, 0x00000040ull);
    id_409 = bldr_29->gen_sym(bldr_29, "@refi64");
    bldr_29->new_type_ref(bldr_29, id_409, id_408);
    id_410 = bldr_29->gen_sym(bldr_29, "@refvoid");
    bldr_29->new_type_ref(bldr_29, id_410, id_407);
    id_411 = bldr_29->gen_sym(bldr_29, "@c_10");
    bldr_29->new_const_int(bldr_29, id_411, id_408, 0x000000000000000aull);
    id_412 = bldr_29->gen_sym(bldr_29, "@c_20");
    bldr_29->new_const_int(bldr_29, id_412, id_408, 0x0000000000000014ull);
    id_413 = bldr_29->gen_sym(bldr_29, "@c_42");
    bldr_29->new_const_int(bldr_29, id_413, id_408, 0x000000000000002aull);
    id_414 = bldr_29->gen_sym(bldr_29, "@sig_i64_i64");
    bldr_29->new_funcsig(bldr_29, id_414, (MuTypeNode [1]){id_408}, 1, (MuTypeNode [1]){id_408}, 1);
    id_415 = bldr_29->gen_sym(bldr_29, "@test_fnc");
    bldr_29->new_func(bldr_29, id_415, id_414);
    id_416 = bldr_29->gen_sym(bldr_29, "@throw_fnc");
    bldr_29->new_func(bldr_29, id_416, id_414);
    id_417 = bldr_29->gen_sym(bldr_29, "@intermediate_fnc");
    bldr_29->new_func(bldr_29, id_417, id_414);
    id_418 = bldr_29->gen_sym(bldr_29, "@throw_fnc.v1");
    id_419 = bldr_29->gen_sym(bldr_29, "@throw_fnc.v1.blk0");
    id_420 = bldr_29->gen_sym(bldr_29, "@throw_fnc.v1.blk1");
    id_421 = bldr_29->gen_sym(bldr_29, "@throw_fnc.v1.blk2");
    id_422 = bldr_29->gen_sym(bldr_29, "@throw_fnc.v1.blk0.num");
    id_423 = bldr_29->gen_sym(bldr_29, "@throw_fnc.v1.blk0.cmpres");
    id_424 = bldr_29->gen_sym(bldr_29, NULL);
    bldr_29->new_cmp(bldr_29, id_424, id_423, MU_CMP_SLT, id_408, id_422, id_413);
    id_425 = bldr_29->gen_sym(bldr_29, NULL);
    bldr_29->new_dest_clause(bldr_29, id_425, id_420, NULL, 0);
    id_426 = bldr_29->gen_sym(bldr_29, NULL);
    bldr_29->new_dest_clause(bldr_29, id_426, id_421, NULL, 0);
    id_427 = bldr_29->gen_sym(bldr_29, NULL);
    bldr_29->new_branch2(bldr_29, id_427, id_423, id_425, id_426);
    bldr_29->new_bb(bldr_29, id_419, (MuID [1]){id_422}, (MuTypeNode [1]){id_408}, 1, MU_NO_ID, (MuInstNode [2]){id_424, id_427}, 2);
    id_428 = bldr_29->gen_sym(bldr_29, "@throw_fnc.v1.blk1.excobj");
    id_429 = bldr_29->gen_sym(bldr_29, "@throw_fnc.v1.blk1.iref_obj");
    id_430 = bldr_29->gen_sym(bldr_29, NULL);
    bldr_29->new_new(bldr_29, id_430, id_428, id_408, MU_NO_ID);
    id_431 = bldr_29->gen_sym(bldr_29, NULL);
    bldr_29->new_getiref(bldr_29, id_431, id_429, id_408, id_428);
    id_432 = bldr_29->gen_sym(bldr_29, NULL);
    bldr_29->new_store(bldr_29, id_432, false, MU_ORD_NOT_ATOMIC, id_408, id_429, id_412, MU_NO_ID);
    id_433 = bldr_29->gen_sym(bldr_29, NULL);
    bldr_29->new_throw(bldr_29, id_433, id_428);
    bldr_29->new_bb(bldr_29, id_420, NULL, NULL, 0, MU_NO_ID, (MuInstNode [4]){id_430, id_431, id_432, id_433}, 4);
    id_434 = bldr_29->gen_sym(bldr_29, NULL);
    bldr_29->new_ret(bldr_29, id_434, (MuVarNode [1]){id_411}, 1);
    bldr_29->new_bb(bldr_29, id_421, NULL, NULL, 0, MU_NO_ID, (MuInstNode [1]){id_434}, 1);
    bldr_29->new_func_ver(bldr_29, id_418, id_416, (MuBBNode [3]){id_419, id_420, id_421}, 3);
    id_435 = bldr_29->gen_sym(bldr_29, "@intermediate_fnc.v1");
    id_436 = bldr_29->gen_sym(bldr_29, "@intermediate_fnc.v1.blk0");
    id_437 = bldr_29->gen_sym(bldr_29, "@intermediate_fnc.v1.blk0.num");
    id_438 = bldr_29->gen_sym(bldr_29, "@intermediate_fnc.v1.blk0.res");
    id_439 = bldr_29->gen_sym(bldr_29, NULL);
    bldr_29->new_call(bldr_29, id_439, (MuID [1]){id_438}, 1, id_414, id_416, (MuVarNode [1]){id_437}, 1, MU_NO_ID, MU_NO_ID);
    id_440 = bldr_29->gen_sym(bldr_29, NULL);
    bldr_29->new_ret(bldr_29, id_440, (MuVarNode [1]){id_438}, 1);
    bldr_29->new_bb(bldr_29, id_436, (MuID [1]){id_437}, (MuTypeNode [1]){id_408}, 1, MU_NO_ID, (MuInstNode [2]){id_439, id_440}, 2);
    bldr_29->new_func_ver(bldr_29, id_435, id_417, (MuBBNode [1]){id_436}, 1);
    id_441 = bldr_29->gen_sym(bldr_29, "@test_fnc.v1");
    id_442 = bldr_29->gen_sym(bldr_29, "@test_fnc.v1.blk0");
    id_443 = bldr_29->gen_sym(bldr_29, "@test_fnc.v1.blk1");
    id_444 = bldr_29->gen_sym(bldr_29, "@test_fnc.v1.blk2");
    id_445 = bldr_29->gen_sym(bldr_29, "@test_fnc.v1.blk0.num");
    id_446 = bldr_29->gen_sym(bldr_29, "@test_fnc.v1.blk0.res");
    id_447 = bldr_29->gen_sym(bldr_29, NULL);
    bldr_29->new_dest_clause(bldr_29, id_447, id_443, (MuVarNode [1]){id_446}, 1);
    id_448 = bldr_29->gen_sym(bldr_29, NULL);
    bldr_29->new_dest_clause(bldr_29, id_448, id_444, NULL, 0);
    id_449 = bldr_29->gen_sym(bldr_29, NULL);
    bldr_29->new_exc_clause(bldr_29, id_449, id_447, id_448);
    id_450 = bldr_29->gen_sym(bldr_29, NULL);
    bldr_29->new_call(bldr_29, id_450, (MuID [1]){id_446}, 1, id_414, id_417, (MuVarNode [1]){id_445}, 1, id_449, MU_NO_ID);
    bldr_29->new_bb(bldr_29, id_442, (MuID [1]){id_445}, (MuTypeNode [1]){id_408}, 1, MU_NO_ID, (MuInstNode [1]){id_450}, 1);
    id_451 = bldr_29->gen_sym(bldr_29, "@test_fnc.v1.blk1.rtn");
    id_452 = bldr_29->gen_sym(bldr_29, NULL);
    bldr_29->new_ret(bldr_29, id_452, (MuVarNode [1]){id_451}, 1);
    bldr_29->new_bb(bldr_29, id_443, (MuID [1]){id_451}, (MuTypeNode [1]){id_408}, 1, MU_NO_ID, (MuInstNode [1]){id_452}, 1);
    id_453 = bldr_29->gen_sym(bldr_29, "@test_fnc.v1.blk2.excobj");
    id_454 = bldr_29->gen_sym(bldr_29, "@test_fnc.v1.blk2.ri64");
    id_455 = bldr_29->gen_sym(bldr_29, "@test_fnc.v1.blk2.iri64");
    id_456 = bldr_29->gen_sym(bldr_29, "@test_fnc.v1.blk2.obj");
    id_457 = bldr_29->gen_sym(bldr_29, NULL);
    bldr_29->new_conv(bldr_29, id_457, id_454, MU_CONV_REFCAST, id_410, id_409, id_453);
    id_458 = bldr_29->gen_sym(bldr_29, NULL);
    bldr_29->new_getiref(bldr_29, id_458, id_455, id_408, id_454);
    id_459 = bldr_29->gen_sym(bldr_29, NULL);
    bldr_29->new_load(bldr_29, id_459, id_456, false, MU_ORD_NOT_ATOMIC, id_408, id_455, MU_NO_ID);
    id_460 = bldr_29->gen_sym(bldr_29, NULL);
    bldr_29->new_dest_clause(bldr_29, id_460, id_443, (MuVarNode [1]){id_456}, 1);
    id_461 = bldr_29->gen_sym(bldr_29, NULL);
    bldr_29->new_branch(bldr_29, id_461, id_460);
    bldr_29->new_bb(bldr_29, id_444, NULL, NULL, 0, id_453, (MuInstNode [4]){id_457, id_458, id_459, id_461}, 4);
    bldr_29->new_func_ver(bldr_29, id_441, id_415, (MuBBNode [3]){id_442, id_443, id_444}, 3);
    bldr_29->load(bldr_29);
    mu_29->compile_to_sharedlib(mu_29, LIB_FILE_NAME("test_exception_stack_unwind"), NULL, 0);
    return 0;
}
