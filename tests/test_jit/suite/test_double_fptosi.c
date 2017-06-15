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
    MuVM* mu_43;
    MuCtx* ctx_43;
    MuIRBuilder* bldr_43;
    MuID id_535;
    MuID id_536;
    MuID id_537;
    MuID id_538;
    MuID id_539;
    MuID id_540;
    MuID id_541;
    MuID id_542;
    MuID id_543;
    MuID id_544;
    MuID id_545;
    mu_43 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_43 = mu_43->new_context(mu_43);
    bldr_43 = ctx_43->new_ir_builder(ctx_43);
    id_535 = bldr_43->gen_sym(bldr_43, "@dbl");
    bldr_43->new_type_double(bldr_43, id_535);
    id_536 = bldr_43->gen_sym(bldr_43, "@i1");
    bldr_43->new_type_int(bldr_43, id_536, 0x00000001ull);
    id_537 = bldr_43->gen_sym(bldr_43, "@i64");
    bldr_43->new_type_int(bldr_43, id_537, 0x00000040ull);
    id_538 = bldr_43->gen_sym(bldr_43, "@npi");
    bldr_43->new_const_double(bldr_43, id_538, id_535, *(double*)(uint64_t [1]){0xc00921fb4d12d84aull});
    id_539 = bldr_43->gen_sym(bldr_43, "@sig__i64");
    bldr_43->new_funcsig(bldr_43, id_539, NULL, 0, (MuTypeNode [1]){id_537}, 1);
    id_540 = bldr_43->gen_sym(bldr_43, "@test_fnc");
    bldr_43->new_func(bldr_43, id_540, id_539);
    id_541 = bldr_43->gen_sym(bldr_43, "@test_fnc.v1");
    id_542 = bldr_43->gen_sym(bldr_43, "@test_fnc.v1.blk0");
    id_543 = bldr_43->gen_sym(bldr_43, "@test_fnc.v1.blk0.res");
    id_544 = bldr_43->gen_sym(bldr_43, NULL);
    bldr_43->new_conv(bldr_43, id_544, id_543, MU_CONV_FPTOSI, id_535, id_537, id_538);
    id_545 = bldr_43->gen_sym(bldr_43, NULL);
    bldr_43->new_ret(bldr_43, id_545, (MuVarNode [1]){id_543}, 1);
    bldr_43->new_bb(bldr_43, id_542, NULL, NULL, 0, MU_NO_ID, (MuInstNode [2]){id_544, id_545}, 2);
    bldr_43->new_func_ver(bldr_43, id_541, id_540, (MuBBNode [1]){id_542}, 1);
    bldr_43->load(bldr_43);
    mu_43->compile_to_sharedlib(mu_43, LIB_FILE_NAME("test_double_fptosi"), NULL, 0);
    return 0;
}
