
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
    MuVM* mu_47;
    MuCtx* ctx_47;
    MuIRBuilder* bldr_47;
    MuID id_583;
    MuID id_584;
    MuID id_585;
    MuID id_586;
    MuID id_587;
    MuID id_588;
    MuID id_589;
    MuID id_590;
    MuID id_591;
    MuID id_592;
    MuID id_593;
    MuID id_594;
    MuID id_595;
    MuID id_596;
    MuID id_597;
    MuID id_598;
    MuID id_599;
    MuID id_600;
    MuID id_601;
    MuID id_602;
    MuID id_603;
    MuID id_604;
    MuID id_605;
    MuID id_606;
    MuID id_607;
    MuID id_608;
    MuID id_609;
    MuID id_610;
    MuID id_611;
    MuID id_612;
    MuID id_613;
    MuID id_614;
    MuID id_615;
    MuID id_616;
    MuID id_617;
    MuID id_618;
    mu_47 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_47 = mu_47->new_context(mu_47);
    bldr_47 = ctx_47->new_ir_builder(ctx_47);
    id_583 = bldr_47->gen_sym(bldr_47, "@i8");
    bldr_47->new_type_int(bldr_47, id_583, 0x00000008ull);
    id_584 = bldr_47->gen_sym(bldr_47, "@i32");
    bldr_47->new_type_int(bldr_47, id_584, 0x00000020ull);
    id_585 = bldr_47->gen_sym(bldr_47, "@refi8");
    bldr_47->new_type_ref(bldr_47, id_585, id_583);
    id_586 = bldr_47->gen_sym(bldr_47, "@irefi8");
    bldr_47->new_type_iref(bldr_47, id_586, id_583);
    id_587 = bldr_47->gen_sym(bldr_47, "@refi32");
    bldr_47->new_type_ref(bldr_47, id_587, id_584);
    id_588 = bldr_47->gen_sym(bldr_47, "@iref32");
    bldr_47->new_type_iref(bldr_47, id_588, id_584);
    id_589 = bldr_47->gen_sym(bldr_47, "@1_i8");
    bldr_47->new_const_int(bldr_47, id_589, id_583, 0x0000000000000001ull);
    id_590 = bldr_47->gen_sym(bldr_47, "@0x8d_i8");
    bldr_47->new_const_int(bldr_47, id_590, id_583, 0x000000000000008dull);
    id_591 = bldr_47->gen_sym(bldr_47, "@0x9f_i8");
    bldr_47->new_const_int(bldr_47, id_591, id_583, 0x000000000000009full);
    id_592 = bldr_47->gen_sym(bldr_47, "@0x9c_i8");
    bldr_47->new_const_int(bldr_47, id_592, id_583, 0x000000000000009cull);
    id_593 = bldr_47->gen_sym(bldr_47, "@0x1d_i8");
    bldr_47->new_const_int(bldr_47, id_593, id_583, 0x000000000000001dull);
    id_594 = bldr_47->gen_sym(bldr_47, "@sig__i32");
    bldr_47->new_funcsig(bldr_47, id_594, NULL, 0, (MuTypeNode [1]){id_584}, 1);
    id_595 = bldr_47->gen_sym(bldr_47, "@test_fnc");
    bldr_47->new_func(bldr_47, id_595, id_594);
    id_596 = bldr_47->gen_sym(bldr_47, "@test_fnc.v1");
    id_597 = bldr_47->gen_sym(bldr_47, "@test_fnc.v1.blk0");
    id_598 = bldr_47->gen_sym(bldr_47, "@test_fnc.v1.blk0.r32x");
    id_599 = bldr_47->gen_sym(bldr_47, "@test_fnc.v1.blk0.r8x");
    id_600 = bldr_47->gen_sym(bldr_47, "@test_fnc.v1.blk0.ir8x_0");
    id_601 = bldr_47->gen_sym(bldr_47, "@test_fnc.v1.blk0.ir8x_1");
    id_602 = bldr_47->gen_sym(bldr_47, "@test_fnc.v1.blk0.ir8x_2");
    id_603 = bldr_47->gen_sym(bldr_47, "@test_fnc.v1.blk0.ir8x_3");
    id_604 = bldr_47->gen_sym(bldr_47, "@test_fnc.v1.blk0.ir32x");
    id_605 = bldr_47->gen_sym(bldr_47, "@test_fnc.v1.blk0.res");
    id_606 = bldr_47->gen_sym(bldr_47, NULL);
    bldr_47->new_new(bldr_47, id_606, id_598, id_584, MU_NO_ID);
    id_607 = bldr_47->gen_sym(bldr_47, NULL);
    bldr_47->new_conv(bldr_47, id_607, id_599, MU_CONV_REFCAST, id_587, id_585, id_598);
    id_608 = bldr_47->gen_sym(bldr_47, NULL);
    bldr_47->new_getiref(bldr_47, id_608, id_600, id_583, id_599);
    id_609 = bldr_47->gen_sym(bldr_47, NULL);
    bldr_47->new_store(bldr_47, id_609, false, MU_ORD_NOT_ATOMIC, id_583, id_600, id_593, MU_NO_ID);
    id_610 = bldr_47->gen_sym(bldr_47, NULL);
    bldr_47->new_shiftiref(bldr_47, id_610, id_601, false, id_583, id_583, id_600, id_589);
    id_611 = bldr_47->gen_sym(bldr_47, NULL);
    bldr_47->new_store(bldr_47, id_611, false, MU_ORD_NOT_ATOMIC, id_583, id_601, id_592, MU_NO_ID);
    id_612 = bldr_47->gen_sym(bldr_47, NULL);
    bldr_47->new_shiftiref(bldr_47, id_612, id_602, false, id_583, id_583, id_601, id_589);
    id_613 = bldr_47->gen_sym(bldr_47, NULL);
    bldr_47->new_store(bldr_47, id_613, false, MU_ORD_NOT_ATOMIC, id_583, id_602, id_591, MU_NO_ID);
    id_614 = bldr_47->gen_sym(bldr_47, NULL);
    bldr_47->new_shiftiref(bldr_47, id_614, id_603, false, id_583, id_583, id_602, id_589);
    id_615 = bldr_47->gen_sym(bldr_47, NULL);
    bldr_47->new_store(bldr_47, id_615, false, MU_ORD_NOT_ATOMIC, id_583, id_603, id_590, MU_NO_ID);
    id_616 = bldr_47->gen_sym(bldr_47, NULL);
    bldr_47->new_getiref(bldr_47, id_616, id_604, id_584, id_598);
    id_617 = bldr_47->gen_sym(bldr_47, NULL);
    bldr_47->new_load(bldr_47, id_617, id_605, false, MU_ORD_NOT_ATOMIC, id_584, id_604, MU_NO_ID);
    id_618 = bldr_47->gen_sym(bldr_47, NULL);
    bldr_47->new_ret(bldr_47, id_618, (MuVarNode [1]){id_605}, 1);
    bldr_47->new_bb(bldr_47, id_597, NULL, NULL, 0, MU_NO_ID, (MuInstNode [13]){id_606, id_607, id_608, id_609, id_610, id_611, id_612, id_613, id_614, id_615, id_616, id_617, id_618}, 13);
    bldr_47->new_func_ver(bldr_47, id_596, id_595, (MuBBNode [1]){id_597}, 1);
    bldr_47->load(bldr_47);
    mu_47->compile_to_sharedlib(mu_47, LIB_FILE_NAME("test_new_bytestore_load"), NULL, 0);
    return 0;
}
