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
    printf("Hello world!\n");

    MuVM *mvm = new_mock_micro_vm("cclientt");

    MuID forty_two = mvm->id_of(mvm, "@forty_two");

    printf("forty_two = %d\n", forty_two);

    MuName name_43 = mvm->name_of(mvm, 43);
    printf("name_43 = '%s'\n", name_43);

    MuName name_43_2 = mvm->name_of(mvm, 43);
    printf("name_43_2 = '%s'\n", name_43_2);

    printf("Setting trap handler...\n");
    mvm->set_trap_handler(mvm, my_trap_handler, "I am the trap handler!");

    // TODO: Register a trap handler to see if Rust can really call back.

    free_mock_micro_vm(mvm);

    return 0;
}
