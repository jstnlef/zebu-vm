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
    MuVM* mu_48;
    MuCtx* ctx_48;
    MuIRBuilder* bldr_48;
    MuID id_619;
    MuID id_620;
    MuID id_621;
    MuID id_622;
    MuID id_623;
    MuID id_624;
    MuID id_625;
    MuID id_626;
    MuID id_627;
    MuID id_628;
    MuID id_629;
    MuID id_630;
    MuID id_631;
    MuID id_632;
    MuID id_633;
    MuID id_634;
    MuID id_635;
    MuID id_636;
    MuID id_637;
    MuID id_638;
    MuID id_639;
    MuID id_640;
    MuID id_641;
    MuID id_642;
    MuID id_643;
    MuID id_644;
    MuID id_645;
    MuID id_646;
    MuID id_647;
    mu_48 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_48 = mu_48->new_context(mu_48);
    bldr_48 = ctx_48->new_ir_builder(ctx_48);
    id_619 = bldr_48->gen_sym(bldr_48, "@i8");
    bldr_48->new_type_int(bldr_48, id_619, 0x00000008ull);
    id_620 = bldr_48->gen_sym(bldr_48, "@i32");
    bldr_48->new_type_int(bldr_48, id_620, 0x00000020ull);
    id_621 = bldr_48->gen_sym(bldr_48, "@pi8");
    bldr_48->new_type_uptr(bldr_48, id_621, id_619);
    id_622 = bldr_48->gen_sym(bldr_48, "@pi32");
    bldr_48->new_type_uptr(bldr_48, id_622, id_620);
    id_623 = bldr_48->gen_sym(bldr_48, "@1_i8");
    bldr_48->new_const_int(bldr_48, id_623, id_619, 0x0000000000000001ull);
    id_624 = bldr_48->gen_sym(bldr_48, "@0x8d_i8");
    bldr_48->new_const_int(bldr_48, id_624, id_619, 0x000000000000008dull);
    id_625 = bldr_48->gen_sym(bldr_48, "@0x9f_i8");
    bldr_48->new_const_int(bldr_48, id_625, id_619, 0x000000000000009full);
    id_626 = bldr_48->gen_sym(bldr_48, "@0x9c_i8");
    bldr_48->new_const_int(bldr_48, id_626, id_619, 0x000000000000009cull);
    id_627 = bldr_48->gen_sym(bldr_48, "@0x1d_i8");
    bldr_48->new_const_int(bldr_48, id_627, id_619, 0x000000000000001dull);
    id_628 = bldr_48->gen_sym(bldr_48, "@sig_pi32_i32");
    bldr_48->new_funcsig(bldr_48, id_628, (MuTypeNode [1]){id_622}, 1, (MuTypeNode [1]){id_620}, 1);
    id_629 = bldr_48->gen_sym(bldr_48, "@test_fnc");
    bldr_48->new_func(bldr_48, id_629, id_628);
    id_630 = bldr_48->gen_sym(bldr_48, "@test_fnc.v1");
    id_631 = bldr_48->gen_sym(bldr_48, "@test_fnc.v1.blk0");
    id_632 = bldr_48->gen_sym(bldr_48, "@test_fnc.v1.blk0.pi32x");
    id_633 = bldr_48->gen_sym(bldr_48, "@test_fnc.v1.blk0.pi8x_0");
    id_634 = bldr_48->gen_sym(bldr_48, "@test_fnc.v1.blk0.pi8x_1");
    id_635 = bldr_48->gen_sym(bldr_48, "@test_fnc.v1.blk0.pi8x_2");
    id_636 = bldr_48->gen_sym(bldr_48, "@test_fnc.v1.blk0.pi8x_3");
    id_637 = bldr_48->gen_sym(bldr_48, "@test_fnc.v1.blk0.res");
    id_638 = bldr_48->gen_sym(bldr_48, NULL);
    bldr_48->new_conv(bldr_48, id_638, id_633, MU_CONV_PTRCAST, id_622, id_621, id_632);
    id_639 = bldr_48->gen_sym(bldr_48, NULL);
    bldr_48->new_store(bldr_48, id_639, true, MU_ORD_NOT_ATOMIC, id_619, id_633, id_627, MU_NO_ID);
    id_640 = bldr_48->gen_sym(bldr_48, NULL);
    bldr_48->new_shiftiref(bldr_48, id_640, id_634, true, id_619, id_619, id_633, id_623);
    id_641 = bldr_48->gen_sym(bldr_48, NULL);
    bldr_48->new_store(bldr_48, id_641, true, MU_ORD_NOT_ATOMIC, id_619, id_634, id_626, MU_NO_ID);
    id_642 = bldr_48->gen_sym(bldr_48, NULL);
    bldr_48->new_shiftiref(bldr_48, id_642, id_635, true, id_619, id_619, id_634, id_623);
    id_643 = bldr_48->gen_sym(bldr_48, NULL);
    bldr_48->new_store(bldr_48, id_643, true, MU_ORD_NOT_ATOMIC, id_619, id_635, id_625, MU_NO_ID);
    id_644 = bldr_48->gen_sym(bldr_48, NULL);
    bldr_48->new_shiftiref(bldr_48, id_644, id_636, true, id_619, id_619, id_635, id_623);
    id_645 = bldr_48->gen_sym(bldr_48, NULL);
    bldr_48->new_store(bldr_48, id_645, true, MU_ORD_NOT_ATOMIC, id_619, id_636, id_624, MU_NO_ID);
    id_646 = bldr_48->gen_sym(bldr_48, NULL);
    bldr_48->new_load(bldr_48, id_646, id_637, true, MU_ORD_NOT_ATOMIC, id_620, id_632, MU_NO_ID);
    id_647 = bldr_48->gen_sym(bldr_48, NULL);
    bldr_48->new_ret(bldr_48, id_647, (MuVarNode [1]){id_637}, 1);
    bldr_48->new_bb(bldr_48, id_631, (MuID [1]){id_632}, (MuTypeNode [1]){id_622}, 1, MU_NO_ID, (MuInstNode [10]){id_638, id_639, id_640, id_641, id_642, id_643, id_644, id_645, id_646, id_647}, 10);
    bldr_48->new_func_ver(bldr_48, id_630, id_629, (MuBBNode [1]){id_631}, 1);
    bldr_48->load(bldr_48);
    mu_48->compile_to_sharedlib(mu_48, LIB_FILE_NAME("test_uptr_bytestore_load"), NULL, 0);
    return 0;
}
