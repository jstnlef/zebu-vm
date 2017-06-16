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
#include <stdint.h>
#include <inttypes.h>
#include <stdlib.h>

#include <muapi.h>
#include "cpart.h"

void my_mem_freer(MuValue *values, MuCPtr freerdata) {
    printf("[C] in my_mem_freer, values=%p\n", values);
    printf("[C] freerdata (as string) = %s\n", (char*)freerdata);

    free(values);
}

void my_trap_handler(
        // input parameters
        MuCtx *ctx,
        MuThreadRefValue thread,
        MuStackRefValue stack,
        MuWPID wpid,
        // output parameters
        MuTrapHandlerResult *result,
        MuStackRefValue *new_stack,
        MuValue **values,
        MuArraySize *nvalues,
        MuValuesFreer *freer,
        MuCPtr *freerdata,
        MuRefValue *exception,
        // input parameter (userdata)
        MuCPtr userdata) {
    printf("[C] in my_trap_handler, ctx=%p, thread=%p, stak=%p, wpid=%u\n",
            ctx, thread, stack, wpid);

    printf("[C] userdata (as string) = %s\n", (char*)userdata);

    *result = MU_REBIND_PASS_VALUES;
    *new_stack = stack;
    MuValue *values_array = (MuValue*)calloc(sizeof(MuValue), 2);
    printf("[C] values_array = %p allocated.\n", values_array);

    values_array[0] = stack;
    values_array[1] = thread;
    *values = values_array;
    *nvalues = 2;

    *freer = my_mem_freer;
    *freerdata = (MuCPtr)"Free me!";
    *exception = NULL;

    return;
}

int main() {
    printf("[C] Hello world!\n");

    MuVM *mvm = new_mock_micro_vm("cclientt");

    MuID forty_two = mvm->id_of(mvm, "@forty_two");

    printf("[C] forty_two = %d\n", forty_two);

    MuName name_43 = mvm->name_of(mvm, 43);
    printf("[C] name_43 = '%s'\n", name_43);

    MuName name_43_2 = mvm->name_of(mvm, 43);
    printf("[C] name_43_2 = '%s'\n", name_43_2);

    printf("[C] Setting trap handler...\n");
    mvm->set_trap_handler(mvm, my_trap_handler, "I am the trap handler!");

    printf("[C] Asking for a context...\n");
    MuCtx *ctx = mvm->new_context(mvm);
    printf("[C] Context created. ctx=%p\n", ctx);

    MuName name_43_3 = ctx->name_of(ctx, 43);
    printf("[C] name_43_3 = '%s'\n", name_43_3);

    MuIntValue *v1 = ctx->handle_from_sint32(ctx, 345, 32);
    MuIntValue *v2 = ctx->handle_from_sint32(ctx, 678, 32);
    ctx->delete_value(ctx, v1);

    ctx->close_context(ctx);

    free_mock_micro_vm(mvm);

    return 0;
}
