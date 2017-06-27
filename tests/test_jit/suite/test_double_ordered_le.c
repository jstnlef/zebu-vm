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
    MuVM* mu_38;
    MuCtx* ctx_38;
    MuIRBuilder* bldr_38;
    MuID id_474;
    MuID id_475;
    MuID id_476;
    MuID id_477;
    MuID id_478;
    MuID id_479;
    MuID id_480;
    MuID id_481;
    MuID id_482;
    MuID id_483;
    MuID id_484;
    MuID id_485;
    MuID id_486;
    mu_38 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_38 = mu_38->new_context(mu_38);
    bldr_38 = ctx_38->new_ir_builder(ctx_38);
    id_474 = bldr_38->gen_sym(bldr_38, "@dbl");
    bldr_38->new_type_double(bldr_38, id_474);
    id_475 = bldr_38->gen_sym(bldr_38, "@i1");
    bldr_38->new_type_int(bldr_38, id_475, 0x00000001ull);
    id_476 = bldr_38->gen_sym(bldr_38, "@i64");
    bldr_38->new_type_int(bldr_38, id_476, 0x00000040ull);
    id_477 = bldr_38->gen_sym(bldr_38, "@pi");
    bldr_38->new_const_double(bldr_38, id_477, id_474, *(double*)(uint64_t [1]){0x400921fb82c2bd7full});
    id_478 = bldr_38->gen_sym(bldr_38, "@sig__i64");
    bldr_38->new_funcsig(bldr_38, id_478, NULL, 0, (MuTypeNode [1]){id_476}, 1);
    id_479 = bldr_38->gen_sym(bldr_38, "@test_fnc");
    bldr_38->new_func(bldr_38, id_479, id_478);
    id_480 = bldr_38->gen_sym(bldr_38, "@test_fnc.v1");
    id_481 = bldr_38->gen_sym(bldr_38, "@test_fnc.v1.blk0");
    id_482 = bldr_38->gen_sym(bldr_38, "@test_fnc.v1.blk0.cmpres");
    id_483 = bldr_38->gen_sym(bldr_38, "@test_fnc.v1.blk0.res");
    id_484 = bldr_38->gen_sym(bldr_38, NULL);
    bldr_38->new_cmp(bldr_38, id_484, id_482, MU_CMP_FOLE, id_474, id_477, id_477);
    id_485 = bldr_38->gen_sym(bldr_38, NULL);
    bldr_38->new_conv(bldr_38, id_485, id_483, MU_CONV_ZEXT, id_475, id_476, id_482);
    id_486 = bldr_38->gen_sym(bldr_38, NULL);
    bldr_38->new_ret(bldr_38, id_486, (MuVarNode [1]){id_483}, 1);
    bldr_38->new_bb(bldr_38, id_481, NULL, NULL, 0, MU_NO_ID, (MuInstNode [3]){id_484, id_485, id_486}, 3);
    bldr_38->new_func_ver(bldr_38, id_480, id_479, (MuBBNode [1]){id_481}, 1);
    bldr_38->load(bldr_38);
    mu_38->compile_to_sharedlib(mu_38, LIB_FILE_NAME("test_double_ordered_le"), NULL, 0);
    return 0;
}
